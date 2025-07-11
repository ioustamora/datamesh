use crate::error::{DfsError, DfsResult};
/// Resilience Module
///
/// This module provides resilience mechanisms for the distributed file system,
/// including retry logic, circuit breakers, and fault tolerance features.
///
/// Key features:
/// - Configurable retry with exponential backoff
/// - Operation timeouts
/// - Error handling and recovery strategies
/// - Monitoring for circuit breaking
///
/// These resilience features ensure the system can continue operating
/// despite network failures, peer unavailability, or other transient errors.
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Retry configuration for resilient operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a future operation with exponential backoff
pub async fn retry_async<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> DfsResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = DfsResult<T>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        debug!(
            "Attempting {} (attempt {}/{})",
            operation_name, attempt, config.max_attempts
        );

        let start = Instant::now();
        match operation().await {
            Ok(result) => {
                debug!(
                    "{} succeeded on attempt {} after {:?}",
                    operation_name,
                    attempt,
                    start.elapsed()
                );
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_attempts {
                    warn!(
                        "{} failed on attempt {}, retrying in {:?}: {}",
                        operation_name,
                        attempt,
                        delay,
                        last_error.as_ref().unwrap()
                    );
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                        ),
                        config.max_delay,
                    );
                } else {
                    error!(
                        "{} failed after {} attempts: {}",
                        operation_name,
                        config.max_attempts,
                        last_error.as_ref().unwrap()
                    );
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| DfsError::Generic("Retry failed with no error".to_string())))
}

/// Timeout wrapper for operations
pub async fn with_timeout<F>(
    future: F,
    timeout: Duration,
    operation_name: &str,
) -> DfsResult<F::Output>
where
    F: std::future::Future,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => Ok(result),
        Err(_) => {
            error!("{} timed out after {:?}", operation_name, timeout);
            Err(DfsError::Network(format!(
                "{} operation timed out",
                operation_name
            )))
        }
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: Arc<std::sync::atomic::AtomicUsize>,
    success_count: Arc<std::sync::atomic::AtomicUsize>,
    last_failure: Arc<std::sync::Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            failure_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            success_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            last_failure: Arc::new(std::sync::Mutex::new(None)),
            config,
        }
    }

    pub fn state(&self) -> CircuitBreakerState {
        let failure_count = self
            .failure_count
            .load(std::sync::atomic::Ordering::Relaxed);

        if failure_count >= self.config.failure_threshold {
            let last_failure = self.last_failure.lock().unwrap();
            if let Some(last_fail_time) = *last_failure {
                if last_fail_time.elapsed() > self.config.timeout {
                    CircuitBreakerState::HalfOpen
                } else {
                    CircuitBreakerState::Open
                }
            } else {
                CircuitBreakerState::Open
            }
        } else {
            CircuitBreakerState::Closed
        }
    }

    pub async fn call<F, Fut, T>(&self, operation: F) -> DfsResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = DfsResult<T>>,
    {
        match self.state() {
            CircuitBreakerState::Open => {
                return Err(DfsError::Network("Circuit breaker is open".to_string()));
            }
            CircuitBreakerState::HalfOpen => {
                // Reset success count for testing
                self.success_count
                    .store(0, std::sync::atomic::Ordering::Relaxed);
            }
            CircuitBreakerState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    fn on_success(&self) {
        self.success_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // If we're in half-open state and have enough successes, reset
        if self.state() == CircuitBreakerState::HalfOpen {
            let success_count = self
                .success_count
                .load(std::sync::atomic::Ordering::Relaxed);
            if success_count >= self.config.success_threshold {
                self.failure_count
                    .store(0, std::sync::atomic::Ordering::Relaxed);
                self.success_count
                    .store(0, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn on_failure(&self) {
        self.failure_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        *self.last_failure.lock().unwrap() = Some(Instant::now());
    }
}
