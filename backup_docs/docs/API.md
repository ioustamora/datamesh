# DataMesh API Documentation

This document provides comprehensive API documentation for DataMesh's advanced features including load balancing, failover, performance optimization, and billing systems.

## 🚀 Getting Started

### Base URL
```
https://api.datamesh.example.com/v1
```

### Authentication
All API requests require authentication using JWT tokens:

```bash
# Login to get token
curl -X POST /auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Use token in requests
curl -X GET /system/status \
  -H "Authorization: Bearer <your-jwt-token>"
```

## 🏗️ Core System APIs

### System Status

Get overall system health and statistics:

```http
GET /system/status
```

**Response:**
```json
{
  "status": "healthy",
  "uptime": "2d 14h 32m",
  "version": "1.0.0",
  "components": {
    "load_balancer": {
      "status": "active",
      "strategy": "AdaptiveIntelligent",
      "node_count": 15,
      "average_load": 0.65
    },
    "failover": {
      "status": "active",
      "healthy_nodes": 14,
      "failed_nodes": 1,
      "circuit_breakers_open": 0
    },
    "performance": {
      "status": "optimized",
      "cpu_usage": 0.45,
      "memory_usage": 0.62,
      "optimization_active": true
    },
    "billing": {
      "status": "active",
      "active_subscriptions": 1250,
      "total_revenue": 125000.00,
      "pending_invoices": 45
    }
  }
}
```

### System Metrics

Get detailed system metrics:

```http
GET /system/metrics
```

**Query Parameters:**
- `period`: Time period (1h, 24h, 7d, 30d)
- `metric`: Specific metric type (cpu, memory, network, billing)

**Response:**
```json
{
  "period": "24h",
  "metrics": {
    "cpu": {
      "average": 0.45,
      "peak": 0.89,
      "current": 0.52
    },
    "memory": {
      "average": 0.62,
      "peak": 0.85,
      "current": 0.64
    },
    "network": {
      "throughput": 1250.5,
      "latency": 45.2,
      "error_rate": 0.002
    }
  }
}
```

## ⚖️ Load Balancer APIs

### Load Balancer Status

Get current load balancer configuration and statistics:

```http
GET /load-balancer/status
```

**Response:**
```json
{
  "strategy": "AdaptiveIntelligent",
  "node_count": 15,
  "average_load": 0.65,
  "total_connections": 2840,
  "average_latency": 45,
  "auto_scaling": {
    "enabled": true,
    "min_nodes": 3,
    "max_nodes": 20,
    "current_threshold": 0.65
  }
}
```

### Node Selection

Get optimal node for a request:

```http
POST /load-balancer/select-node
```

**Request Body:**
```json
{
  "request_type": "upload",
  "metadata": {
    "file_size": 1048576,
    "priority": "high"
  }
}
```

**Response:**
```json
{
  "selected_node": "12D3KooW...",
  "node_info": {
    "cpu_usage": 0.35,
    "memory_usage": 0.45,
    "active_connections": 12,
    "estimated_latency": 35
  },
  "selection_reason": "optimal_resources"
}
```

### Load Balancer Configuration

Update load balancer settings:

```http
PUT /load-balancer/config
```

**Request Body:**
```json
{
  "strategy": "AdaptiveIntelligent",
  "auto_scaling": {
    "enabled": true,
    "min_nodes": 5,
    "max_nodes": 25,
    "scale_up_threshold": 0.8,
    "scale_down_threshold": 0.3
  }
}
```

## 🛡️ Failover APIs

### Failover Status

Get failover system status:

```http
GET /failover/status
```

**Response:**
```json
{
  "strategy": "CircuitBreaker",
  "total_nodes": 15,
  "healthy_nodes": 14,
  "failed_nodes": 1,
  "circuit_breakers": {
    "open": 0,
    "half_open": 1,
    "closed": 14
  },
  "recent_failures": [
    {
      "node_id": "12D3KooW...",
      "failure_time": "2025-01-09T10:30:00Z",
      "error": "connection_timeout"
    }
  ]
}
```

### Node Health

Get health status for all nodes:

```http
GET /failover/nodes
```

**Response:**
```json
{
  "nodes": [
    {
      "node_id": "12D3KooW...",
      "status": "healthy",
      "response_time": 45,
      "uptime": "2d 14h",
      "circuit_breaker": "closed"
    },
    {
      "node_id": "12D3KooX...",
      "status": "degraded",
      "response_time": 156,
      "uptime": "1d 8h",
      "circuit_breaker": "half_open"
    }
  ]
}
```

### Manual Failover

Trigger manual failover for a node:

```http
POST /failover/trigger
```

**Request Body:**
```json
{
  "node_id": "12D3KooW...",
  "reason": "maintenance",
  "force": false
}
```

## 🚀 Performance Optimization APIs

### Performance Status

Get current performance optimization status:

```http
GET /performance/status
```

**Response:**
```json
{
  "optimization_active": true,
  "strategy": "Adaptive",
  "current_metrics": {
    "cpu_usage": 0.45,
    "memory_usage": 0.62,
    "network_latency": 45.2,
    "throughput": 1250.5
  },
  "active_optimizations": 3,
  "performance_improvement": 0.25
}
```

### Optimization Recommendations

Get performance optimization recommendations:

```http
GET /performance/recommendations
```

**Response:**
```json
{
  "recommendations": [
    {
      "category": "CPU",
      "priority": 8,
      "description": "High CPU usage detected. Consider load balancing.",
      "expected_improvement": 0.30,
      "risk_level": "low",
      "auto_applicable": false
    },
    {
      "category": "Cache",
      "priority": 5,
      "description": "Low cache hit rate. Consider cache optimization.",
      "expected_improvement": 0.20,
      "risk_level": "low",
      "auto_applicable": true
    }
  ]
}
```

### Apply Optimization

Apply a performance optimization:

```http
POST /performance/optimize
```

**Request Body:**
```json
{
  "category": "Cache",
  "auto_apply": true
}
```

### Performance Predictions

Get performance predictions:

```http
GET /performance/predictions
```

**Response:**
```json
{
  "predictions": [
    {
      "metric": "CPU",
      "predicted_value": 0.52,
      "confidence": 0.85,
      "time_horizon": "5m"
    },
    {
      "metric": "Memory",
      "predicted_value": 0.68,
      "confidence": 0.78,
      "time_horizon": "5m"
    }
  ]
}
```

## 💰 Billing APIs

### Billing Overview

Get billing system overview:

```http
GET /billing/overview
```

**Response:**
```json
{
  "total_subscriptions": 1250,
  "active_subscriptions": 1180,
  "total_revenue": 125000.00,
  "pending_revenue": 8500.00,
  "currency": "USD",
  "billing_period": "monthly"
}
```

### Subscriptions

List all subscriptions:

```http
GET /billing/subscriptions
```

**Query Parameters:**
- `status`: Filter by status (active, suspended, cancelled)
- `tier`: Filter by tier (free, basic, pro, enterprise)
- `limit`: Number of results per page
- `offset`: Pagination offset

**Response:**
```json
{
  "subscriptions": [
    {
      "id": "sub_123",
      "user_id": "user_456",
      "tier": "pro",
      "status": "active",
      "price": 29.99,
      "billing_cycle": "monthly",
      "created_at": "2025-01-01T00:00:00Z",
      "expires_at": "2025-02-01T00:00:00Z"
    }
  ],
  "total": 1250,
  "page": 1,
  "per_page": 20
}
```

### Create Subscription

Create a new subscription:

```http
POST /billing/subscriptions
```

**Request Body:**
```json
{
  "user_id": "user_456",
  "tier": "pro",
  "billing_cycle": "monthly",
  "payment_method": {
    "type": "credit_card",
    "last_four": "1234",
    "expiry": "12/26"
  }
}
```

### Usage Tracking

Record usage for billing:

```http
POST /billing/usage
```

**Request Body:**
```json
{
  "user_id": "user_456",
  "resource_type": "storage",
  "amount": 10.5,
  "unit": "GB",
  "metadata": {
    "operation": "file_upload",
    "region": "us-west"
  }
}
```

### Invoices

Get user invoices:

```http
GET /billing/invoices
```

**Query Parameters:**
- `user_id`: Filter by user
- `status`: Filter by status (draft, issued, paid, overdue)
- `date_from`: Start date
- `date_to`: End date

**Response:**
```json
{
  "invoices": [
    {
      "id": "inv_789",
      "user_id": "user_456",
      "amount": 29.99,
      "status": "paid",
      "issued_at": "2025-01-01T00:00:00Z",
      "paid_at": "2025-01-03T10:30:00Z",
      "line_items": [
        {
          "description": "Pro subscription",
          "quantity": 1,
          "unit_price": 29.99,
          "total": 29.99
        }
      ]
    }
  ]
}
```

### Payment Processing

Process a payment:

```http
POST /billing/payments
```

**Request Body:**
```json
{
  "invoice_id": "inv_789",
  "payment_method": {
    "type": "credit_card",
    "token": "tok_xyz123"
  }
}
```

## 🏛️ Governance APIs

### Governance Overview

Get governance system status:

```http
GET /governance/overview
```

**Response:**
```json
{
  "active_proposals": 3,
  "total_users": 1250,
  "bootstrap_operators": 5,
  "voting_power_distribution": {
    "operators": 0.60,
    "users": 0.40
  }
}
```

### Proposals

List governance proposals:

```http
GET /governance/proposals
```

**Response:**
```json
{
  "proposals": [
    {
      "id": "prop_123",
      "title": "Network Upgrade v2.0",
      "description": "Upgrade to new protocol version",
      "status": "voting",
      "votes_for": 150,
      "votes_against": 25,
      "created_at": "2025-01-01T00:00:00Z",
      "voting_ends_at": "2025-01-15T00:00:00Z"
    }
  ]
}
```

### Submit Proposal

Submit a new governance proposal:

```http
POST /governance/proposals
```

**Request Body:**
```json
{
  "title": "Network Upgrade v2.0",
  "description": "Upgrade to new protocol version",
  "type": "network_upgrade",
  "voting_period": "14d",
  "required_quorum": 0.5
}
```

### Vote on Proposal

Vote on a governance proposal:

```http
POST /governance/proposals/{proposal_id}/vote
```

**Request Body:**
```json
{
  "vote": "for",
  "reason": "This upgrade will improve network performance"
}
```

## 🔍 Network Diagnostics APIs

### Network Status

Get network topology and health:

```http
GET /network/status
```

**Response:**
```json
{
  "total_peers": 15,
  "connected_peers": 14,
  "network_health": 0.93,
  "average_latency": 45.2,
  "total_bandwidth": 1250.5,
  "regions": {
    "us-west": 6,
    "us-east": 4,
    "eu-west": 3,
    "asia-pacific": 2
  }
}
```

### Peer Information

Get detailed peer information:

```http
GET /network/peers
```

**Response:**
```json
{
  "peers": [
    {
      "peer_id": "12D3KooW...",
      "address": "/ip4/192.168.1.100/tcp/40000",
      "status": "connected",
      "uptime": "2d 14h",
      "region": "us-west",
      "version": "1.0.0",
      "metrics": {
        "cpu_usage": 0.45,
        "memory_usage": 0.62,
        "storage_usage": 0.35
      }
    }
  ]
}
```

### Network Diagnostics

Run network diagnostics:

```http
POST /network/diagnostics
```

**Request Body:**
```json
{
  "test_type": "bandwidth",
  "duration": 30,
  "peer_id": "12D3KooW..."
}
```

## 🔧 Configuration APIs

### System Configuration

Get system configuration:

```http
GET /config/system
```

**Response:**
```json
{
  "network": {
    "port": 40000,
    "max_peers": 50,
    "bootstrap_peers": ["12D3KooW..."]
  },
  "performance": {
    "optimization_enabled": true,
    "monitoring_interval": 60
  },
  "billing": {
    "currency": "USD",
    "tax_rate": 0.08
  }
}
```

### Update Configuration

Update system configuration:

```http
PUT /config/system
```

**Request Body:**
```json
{
  "network": {
    "max_peers": 100
  },
  "performance": {
    "monitoring_interval": 30
  }
}
```

## 🚨 Alerts and Notifications

### Alert Configuration

Get alert rules:

```http
GET /alerts/rules
```

**Response:**
```json
{
  "rules": [
    {
      "id": "cpu_high",
      "name": "High CPU Usage",
      "condition": "cpu_usage > 0.9",
      "severity": "warning",
      "enabled": true
    }
  ]
}
```

### Create Alert Rule

Create a new alert rule:

```http
POST /alerts/rules
```

**Request Body:**
```json
{
  "name": "High Memory Usage",
  "condition": "memory_usage > 0.85",
  "severity": "warning",
  "notification_channels": ["email", "slack"]
}
```

### Alert History

Get alert history:

```http
GET /alerts/history
```

**Response:**
```json
{
  "alerts": [
    {
      "id": "alert_123",
      "rule_id": "cpu_high",
      "severity": "warning",
      "message": "CPU usage is 92%",
      "triggered_at": "2025-01-09T10:30:00Z",
      "resolved_at": "2025-01-09T10:45:00Z"
    }
  ]
}
```

## 🔒 Security APIs

### Security Status

Get security overview:

```http
GET /security/status
```

**Response:**
```json
{
  "threat_level": "low",
  "active_incidents": 0,
  "blocked_ips": 12,
  "failed_logins": 3,
  "security_events": 156
}
```

### Audit Logs

Get audit logs:

```http
GET /security/audit
```

**Query Parameters:**
- `user_id`: Filter by user
- `action`: Filter by action type
- `date_from`: Start date
- `date_to`: End date

**Response:**
```json
{
  "events": [
    {
      "id": "audit_123",
      "user_id": "user_456",
      "action": "login",
      "resource": "/api/v1/auth/login",
      "timestamp": "2025-01-09T10:30:00Z",
      "ip_address": "192.168.1.100",
      "user_agent": "DataMesh CLI/1.0.0"
    }
  ]
}
```

## 📊 Analytics APIs

### Usage Analytics

Get usage analytics:

```http
GET /analytics/usage
```

**Query Parameters:**
- `metric`: Metric type (storage, bandwidth, api_calls)
- `period`: Time period (1d, 7d, 30d)
- `granularity`: Data granularity (hour, day, week)

**Response:**
```json
{
  "metric": "storage",
  "period": "7d",
  "data": [
    {
      "timestamp": "2025-01-09T00:00:00Z",
      "value": 1250.5
    },
    {
      "timestamp": "2025-01-09T01:00:00Z",
      "value": 1252.1
    }
  ]
}
```

### Business Analytics

Get business metrics:

```http
GET /analytics/business
```

**Response:**
```json
{
  "revenue": {
    "total": 125000.00,
    "growth": 0.15,
    "forecast": 145000.00
  },
  "users": {
    "active": 1180,
    "growth": 0.08,
    "retention": 0.92
  },
  "churn": {
    "rate": 0.05,
    "reasons": ["price", "features", "support"]
  }
}
```

## 🔄 Webhooks

### Webhook Configuration

Configure webhooks for events:

```http
POST /webhooks
```

**Request Body:**
```json
{
  "url": "https://your-app.com/webhook",
  "events": ["subscription.created", "payment.success", "alert.triggered"],
  "secret": "your-webhook-secret"
}
```

### Webhook Events

Common webhook events:
- `subscription.created`
- `subscription.cancelled`
- `payment.success`
- `payment.failed`
- `alert.triggered`
- `alert.resolved`
- `system.maintenance`

## 🛠️ SDKs and Libraries

### JavaScript SDK

```javascript
import { DataMeshClient } from 'datamesh-sdk';

const client = new DataMeshClient({
  baseUrl: 'https://api.datamesh.example.com/v1',
  apiKey: 'your-api-key'
});

// Get system status
const status = await client.system.getStatus();

// Create subscription
const subscription = await client.billing.createSubscription({
  userId: 'user_123',
  tier: 'pro',
  billingCycle: 'monthly'
});
```

### Python SDK

```python
from datamesh import DataMeshClient

client = DataMeshClient(
    base_url='https://api.datamesh.example.com/v1',
    api_key='your-api-key'
)

# Get system status
status = client.system.get_status()

# Create subscription
subscription = client.billing.create_subscription(
    user_id='user_123',
    tier='pro',
    billing_cycle='monthly'
)
```

## 📝 Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "tier",
      "reason": "Invalid subscription tier"
    }
  }
}
```

### Common Error Codes

- `AUTHENTICATION_ERROR`: Invalid or missing authentication
- `AUTHORIZATION_ERROR`: Insufficient permissions
- `VALIDATION_ERROR`: Invalid request parameters
- `RATE_LIMIT_ERROR`: Too many requests
- `SYSTEM_ERROR`: Internal system error
- `MAINTENANCE_MODE`: System under maintenance

## 🔗 Rate Limiting

API requests are rate-limited based on subscription tier:

- **Free Tier**: 100 requests/hour
- **Basic Tier**: 1,000 requests/hour  
- **Pro Tier**: 10,000 requests/hour
- **Enterprise Tier**: 100,000 requests/hour

Rate limit headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1641024000
```

---

This API documentation provides comprehensive coverage of all DataMesh advanced features. For additional support or questions, please refer to the main documentation or contact the development team.