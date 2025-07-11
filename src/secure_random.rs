/// Secure Random Number Generation Module
///
/// This module provides cryptographically secure random number generation
/// using OsRng to replace insecure usage of thread_rng and fastrand.
use rand::rngs::OsRng;
use rand::RngCore;

/// Generate a cryptographically secure random nonce
pub fn generate_secure_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Generate a cryptographically secure random salt
pub fn generate_secure_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generate cryptographically secure random bytes
pub fn generate_secure_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Fill a mutable byte slice with cryptographically secure random data
pub fn fill_secure_bytes(bytes: &mut [u8]) {
    OsRng.fill_bytes(bytes);
}

/// Generate a cryptographically secure random u32
pub fn secure_u32() -> u32 {
    OsRng.next_u32()
}

/// Generate a cryptographically secure random u64
pub fn secure_u64() -> u64 {
    OsRng.next_u64()
}

/// Generate a cryptographically secure random f64 between 0.0 and 1.0
pub fn secure_f64() -> f64 {
    // Generate 53 random bits for f64 mantissa
    let random_bits = OsRng.next_u64() >> 11;
    (random_bits as f64) / ((1u64 << 53) as f64)
}

/// Generate a cryptographically secure random u32 in range [min, max)
pub fn secure_u32_range(min: u32, max: u32) -> u32 {
    if min >= max {
        return min;
    }
    let range = max - min;
    min + (OsRng.next_u32() % range)
}

/// Generate a cryptographically secure random f64 in range [min, max)
pub fn secure_f64_range(min: f64, max: f64) -> f64 {
    if min >= max {
        return min;
    }
    min + secure_f64() * (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_nonce_generation() {
        let nonce1 = generate_secure_nonce();
        let nonce2 = generate_secure_nonce();

        // Nonces should be different (extremely unlikely to be same)
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 12);
    }

    #[test]
    fn test_secure_salt_generation() {
        let salt1 = generate_secure_salt();
        let salt2 = generate_secure_salt();

        // Salts should be different
        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 32);
    }

    #[test]
    fn test_secure_bytes_generation() {
        let bytes1 = generate_secure_bytes(64);
        let bytes2 = generate_secure_bytes(64);

        assert_ne!(bytes1, bytes2);
        assert_eq!(bytes1.len(), 64);
        assert_eq!(bytes2.len(), 64);
    }

    #[test]
    fn test_secure_f64_range() {
        for _ in 0..1000 {
            let val = secure_f64();
            assert!(val >= 0.0 && val < 1.0);
        }

        for _ in 0..1000 {
            let val = secure_f64_range(10.0, 20.0);
            assert!(val >= 10.0 && val < 20.0);
        }
    }

    #[test]
    fn test_secure_u32_range() {
        for _ in 0..1000 {
            let val = secure_u32_range(100, 200);
            assert!(val >= 100 && val < 200);
        }
    }
}
