---
description: Deploy your MCP server to production
---

You are helping deploy an MCP server to production.

## Your Task

Set up production deployment with proper configuration, monitoring, and best practices.

## Steps

### 1. Choose Deployment Strategy

Ask the user:
```
I'll help you deploy your MCP server. Please choose:

1. Deployment target:
   - Local binary (stdio)
   - Docker container
   - Kubernetes cluster
   - Cloud platform (AWS, GCP, Azure)
   - Serverless (Lambda, Cloud Run)

2. Scale requirements:
   - Single instance
   - Multiple instances (load balanced)
   - Auto-scaling

3. Monitoring needs:
   - Basic logging
   - Metrics and dashboards
   - Alerting
   - Distributed tracing
```

### 2. Build Optimized Binary

Create `scripts/build-release.sh`:

```bash
#!/bin/bash
set -e

echo "Building optimized release binary..."

# Clean previous builds
cargo clean --release

# Build with all optimizations
RUSTFLAGS="-C target-cpu=native" \
  cargo build --release \
  --locked \
  --all-features

echo "Binary built: ./target/release/{binary_name}"
echo "Size: $(du -h ./target/release/{binary_name} | cut -f1)"

# Strip debug symbols (further size reduction)
strip ./target/release/{binary_name}

echo "Stripped size: $(du -h ./target/release/{binary_name} | cut -f1)"

# Run smoke test
echo "Running smoke test..."
./target/release/{binary_name} --version

echo "Build complete!"
```

### 3. Create Dockerfile

For Docker deployment, create `Dockerfile`:

```dockerfile
# Multi-stage build for minimal image size
FROM rust:1.85-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src

# Build with optimizations
RUN cargo build --release --locked

# Runtime image
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 mcp

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/{binary_name} ./server
COPY config ./config

# Set ownership
RUN chown -R mcp:mcp /app

# Switch to non-root user
USER mcp

# Expose port (for HTTP/SSE)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run server
CMD ["./server"]
```

Create `.dockerignore`:
```
target/
.git/
.github/
tests/
*.md
Dockerfile
.dockerignore
```

Build and test:
```bash
# Build image
docker build -t mcp-server:latest .

# Test locally
docker run -p 3000:3000 mcp-server:latest

# Push to registry
docker tag mcp-server:latest registry.example.com/mcp-server:latest
docker push registry.example.com/mcp-server:latest
```

### 4. Create Kubernetes Deployment

Generate `k8s/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
  labels:
    app: mcp-server
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
    spec:
      containers:
      - name: mcp-server
        image: registry.example.com/mcp-server:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: APP_SERVER_HOST
          value: "0.0.0.0"
        - name: APP_SERVER_PORT
          value: "3000"
        envFrom:
        - secretRef:
            name: mcp-server-secrets
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
---
apiVersion: v1
kind: Service
metadata:
  name: mcp-server
spec:
  selector:
    app: mcp-server
  ports:
  - port: 80
    targetPort: 3000
    protocol: TCP
    name: http
  type: LoadBalancer
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mcp-server-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mcp-server
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

Deploy:
```bash
# Apply deployment
kubectl apply -f k8s/deployment.yaml

# Check status
kubectl get pods -l app=mcp-server
kubectl get svc mcp-server

# View logs
kubectl logs -f deployment/mcp-server

# Scale manually
kubectl scale deployment mcp-server --replicas=5
```

### 5. Add Health Checks

Update `src/main.rs` to add health endpoints:

```rust
use axum::{{routing::get, Router}};

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> Result<&'static str, StatusCode> {
    // Check if service is ready (DB connected, etc.)
    if service_is_ready().await {
        Ok("Ready")
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// Add to router
let app = Router::new()
    .route("/mcp", post(handle_mcp_request))
    .route("/health", get(health_check))
    .route("/ready", get(readiness_check))
    .with_state(service);
```

### 6. Set Up Monitoring

Add Prometheus metrics in `src/metrics.rs`:

```rust
use prometheus::{{Counter, Histogram, Registry, Encoder, TextEncoder}};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref REQUEST_COUNTER: Counter =
        Counter::new("mcp_requests_total", "Total requests").unwrap();

    pub static ref REQUEST_DURATION: Histogram =
        Histogram::new("mcp_request_duration_seconds", "Request duration").unwrap();

    pub static ref ERROR_COUNTER: Counter =
        Counter::new("mcp_errors_total", "Total errors").unwrap();

    pub static ref ACTIVE_CONNECTIONS: prometheus::IntGauge =
        prometheus::IntGauge::new("mcp_active_connections", "Active connections").unwrap();
}

pub fn init_metrics() {
    REGISTRY.register(Box::new(REQUEST_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(ERROR_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
}

pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

Add metrics endpoint:
```rust
.route("/metrics", get(metrics_handler))
```

### 7. Configure Logging

Update logging for production in `src/main.rs`:

```rust
use tracing_subscriber::{{layer::SubscriberExt, util::SubscriberInitExt}};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(false)
        )
        .init();
}
```

### 8. Create Production Config

Generate `config/production.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000
timeout_seconds = 30
max_connections = 1000

[database]
max_connections = 20
min_connections = 5
connect_timeout_seconds = 10
idle_timeout_seconds = 600

[cache]
ttl_seconds = 3600
max_size = 10000

[logging]
level = "info"
format = "json"

[metrics]
enabled = true
port = 9090
```

### 9. Create Deployment Script

Generate `scripts/deploy.sh`:

```bash
#!/bin/bash
set -e

ENV=${1:-production}
VERSION=${2:-latest}

echo "Deploying MCP server to $ENV..."

# Build and push image
echo "Building Docker image..."
docker build -t mcp-server:$VERSION .
docker tag mcp-server:$VERSION registry.example.com/mcp-server:$VERSION
docker push registry.example.com/mcp-server:$VERSION

# Update Kubernetes deployment
echo "Updating Kubernetes deployment..."
kubectl set image deployment/mcp-server \
  mcp-server=registry.example.com/mcp-server:$VERSION

# Wait for rollout
echo "Waiting for rollout..."
kubectl rollout status deployment/mcp-server --timeout=5m

# Verify deployment
echo "Verifying deployment..."
kubectl get pods -l app=mcp-server

# Run smoke tests
echo "Running smoke tests..."
./scripts/smoke-test.sh

echo "Deployment complete!"
```

### 10. Create Smoke Tests

Generate `scripts/smoke-test.sh`:

```bash
#!/bin/bash
set -e

API_URL=${1:-http://localhost:3000}

echo "Running smoke tests against $API_URL..."

# Test health endpoint
echo "Testing health endpoint..."
curl -f $API_URL/health || exit 1

# Test MCP endpoint
echo "Testing MCP endpoint..."
curl -f -X POST $API_URL/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"ping","id":1}' || exit 1

echo "Smoke tests passed!"
```

### 11. Set Up Alerts

Create `monitoring/alerts.yaml`:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: mcp-server-alerts
spec:
  groups:
  - name: mcp-server
    interval: 30s
    rules:
    - alert: HighErrorRate
      expr: rate(mcp_errors_total[5m]) > 0.05
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value }} errors/sec"

    - alert: HighLatency
      expr: histogram_quantile(0.95, mcp_request_duration_seconds_bucket) > 1.0
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High request latency"
        description: "P95 latency is {{ $value }}s"

    - alert: ServiceDown
      expr: up{job="mcp-server"} == 0
      for: 2m
      labels:
        severity: critical
      annotations:
        summary: "MCP server is down"
        description: "MCP server has been down for 2 minutes"
```

### 12. Create Deployment Documentation

Generate `docs/DEPLOYMENT.md`:

```markdown
# Deployment Guide

## Prerequisites
- Docker
- Kubernetes cluster
- kubectl configured
- Access to container registry

## Deployment Steps

1. **Build and test locally:**
   \```bash
   ./scripts/build-release.sh
   cargo test --release
   \```

2. **Build Docker image:**
   \```bash
   docker build -t mcp-server:v1.0.0 .
   \```

3. **Deploy to Kubernetes:**
   \```bash
   ./scripts/deploy.sh production v1.0.0
   \```

4. **Verify deployment:**
   \```bash
   kubectl get pods -l app=mcp-server
   kubectl logs -f deployment/mcp-server
   \```

5. **Run smoke tests:**
   \```bash
   ./scripts/smoke-test.sh https://your-domain.com
   \```

## Rollback

If deployment fails:
\```bash
kubectl rollout undo deployment/mcp-server
\```

## Monitoring

- Metrics: https://your-domain.com/metrics
- Grafana: https://grafana.your-domain.com
- Logs: kubectl logs -f deployment/mcp-server

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
```

## After Setup

```
✅ Deployment setup complete!

## Files Created:
- scripts/build-release.sh - Build optimized binary
- Dockerfile - Container image
- k8s/ - Kubernetes manifests
- config/production.toml - Production config
- scripts/deploy.sh - Deployment automation
- scripts/smoke-test.sh - Post-deployment tests
- monitoring/alerts.yaml - Alert rules
- docs/DEPLOYMENT.md - Deployment guide

## Next Steps:

1. **Configure secrets:**
   \```bash
   kubectl create secret generic mcp-server-secrets \
     --from-literal=DATABASE_URL=... \
     --from-literal=API_KEY=...
   \```

2. **Deploy:**
   \```bash
   ./scripts/deploy.sh production v1.0.0
   \```

3. **Monitor:**
   - Check metrics at /metrics
   - View logs: kubectl logs -f deployment/mcp-server
   - Set up dashboards in Grafana

4. **Set up CI/CD:**
   - Add deployment to GitHub Actions
   - Automate on main branch push

## Production Checklist:

- [ ] Build optimized binary
- [ ] Configure secrets
- [ ] Set up monitoring
- [ ] Configure alerts
- [ ] Test health checks
- [ ] Run smoke tests
- [ ] Document rollback procedure
- [ ] Set up backup/restore
- [ ] Configure auto-scaling
- [ ] Enable HTTPS/TLS

Good luck with your deployment! 🚀
```
