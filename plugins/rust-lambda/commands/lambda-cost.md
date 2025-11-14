---
description: Deep dive into Lambda cost optimization strategies for Rust functions
---

You are helping the user optimize the cost of their Rust Lambda functions.

## Your Task

Guide the user through advanced cost optimization techniques using AWS Lambda Power Tuning, memory configuration, and Rust-specific optimizations.

## Lambda Pricing Model

**Cost = (Requests × $0.20 per 1M) + (GB-seconds × $0.0000166667)**

- **Requests**: $0.20 per 1 million requests
- **Duration**: Charged per GB-second
  - 1 GB-second = 1 GB memory × 1 second execution
  - ARM64 (Graviton2): 20% cheaper than x86_64

**Example**:
- 512 MB, 100ms execution, 1M requests/month
- Duration: 0.5 GB × 0.1s × 1M = 50,000 GB-seconds
- Cost: $0.20 + (50,000 × $0.0000166667) = $0.20 + $0.83 = $1.03

## Memory vs CPU Allocation

Lambda allocates CPU proportional to memory:
- **128 MB**: 0.08 vCPU
- **512 MB**: 0.33 vCPU
- **1024 MB**: 0.58 vCPU
- **1769 MB**: 1.00 vCPU (full core)
- **3008 MB**: 1.77 vCPU
- **10240 MB**: 6.00 vCPU

**Key insight**: More memory = more CPU = faster execution = potentially lower cost

## AWS Lambda Power Tuning Tool

Automatically finds optimal memory configuration.

### Setup

```bash
# Deploy Power Tuning (one-time)
git clone https://github.com/alexcasalboni/aws-lambda-power-tuning
cd aws-lambda-power-tuning
sam deploy --guided

# Run power tuning
aws stepfunctions start-execution \
  --state-machine-arn arn:aws:states:REGION:ACCOUNT:stateMachine:powerTuningStateMachine \
  --input '{
    "lambdaARN": "arn:aws:lambda:REGION:ACCOUNT:function:my-rust-function",
    "powerValues": [128, 256, 512, 1024, 1536, 2048, 3008],
    "num": 10,
    "payload": "{\"test\": \"data\"}",
    "parallelInvocation": true,
    "strategy": "cost"
  }'
```

**Strategies**:
- `cost`: Minimize cost
- `speed`: Minimize duration
- `balanced`: Balance cost and speed

## Rust-Specific Optimizations

### 1. Binary Size Reduction

**Cargo.toml**:
```toml
[profile.release]
opt-level = 'z'       # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Single codegen unit
strip = true          # Strip symbols
panic = 'abort'       # Smaller panic handler

[profile.release.package."*"]
opt-level = 'z'       # Optimize dependencies too
```

**Result**: 3-5x smaller binary = faster cold starts = lower duration

### 2. Use ARM64 (Graviton2)

```bash
cargo lambda build --release --arm64
cargo lambda deploy --arch arm64
```

**Savings**: 20% lower cost for same performance

### 3. Dependency Optimization

```bash
# Analyze binary size
cargo install cargo-bloat
cargo bloat --release -n 20

# Find unused features
cargo install cargo-unused-features
cargo unused-features

# Remove unused dependencies
cargo install cargo-udeps
cargo +nightly udeps
```

**Example**:
```toml
# ❌ Full tokio (heavy)
tokio = { version = "1", features = ["full"] }

# ✅ Only needed features (light)
tokio = { version = "1", features = ["macros", "rt"] }
```

### 4. Use Lightweight Alternatives

- `ureq` instead of `reqwest` for simple HTTP
- `rustls` instead of `native-tls`
- `simdjson` instead of `serde_json` for large JSON
- Avoid `regex` for simple string operations

## Memory Configuration Strategies

### Strategy 1: Start Low, Test Up

```bash
# Test different memory sizes
for mem in 128 256 512 1024 2048; do
  echo "Testing ${mem}MB"
  cargo lambda deploy --memory $mem
  # Run load test
  # Measure duration and cost
done
```

### Strategy 2: Monitor CloudWatch Metrics

```bash
# Get duration statistics
aws cloudwatch get-metric-statistics \
  --namespace AWS/Lambda \
  --metric-name Duration \
  --dimensions Name=FunctionName,Value=my-function \
  --start-time 2025-01-01T00:00:00Z \
  --end-time 2025-01-02T00:00:00Z \
  --period 3600 \
  --statistics Average,Maximum

# Get memory usage
aws cloudwatch get-metric-statistics \
  --namespace AWS/Lambda \
  --metric-name MemoryUtilization \
  --dimensions Name=FunctionName,Value=my-function \
  --start-time 2025-01-01T00:00:00Z \
  --end-time 2025-01-02T00:00:00Z \
  --period 3600 \
  --statistics Maximum
```

### Strategy 3: Right-size Based on Workload

**IO-intensive** (API calls, DB queries):
- Start: 512 MB
- Sweet spot: Usually 512-1024 MB
- Reason: Limited by network, not CPU

**Compute-intensive** (data processing):
- Start: 1024 MB
- Sweet spot: Usually 1769-3008 MB
- Reason: More CPU = faster = lower total cost

**Mixed workload**:
- Start: 1024 MB
- Test: 1024, 1769, 2048 MB
- Use Power Tuning tool

## Cost Optimization Checklist

- [ ] Use ARM64 architecture (20% savings)
- [ ] Optimize binary size (faster cold starts)
- [ ] Remove unused dependencies
- [ ] Use lightweight alternatives
- [ ] Run AWS Lambda Power Tuning
- [ ] Right-size memory based on workload
- [ ] Set appropriate timeout (not too high)
- [ ] Reduce cold starts (keep functions warm if needed)
- [ ] Use reserved concurrency for predictable workloads
- [ ] Batch requests when possible
- [ ] Cache results (DynamoDB, ElastiCache)
- [ ] Monitor and alert on cost anomalies

## Advanced: Provisioned Concurrency

For latency-sensitive functions, pre-warm instances.

**Cost**: $0.015 per GB-hour (expensive!)

```bash
aws lambda put-provisioned-concurrency-config \
  --function-name my-function \
  --provisioned-concurrent-executions 5
```

**Use when**:
- Cold starts are unacceptable
- Predictable traffic patterns
- Cost justifies latency improvement

## Cost Monitoring

### CloudWatch Billing Alerts

```bash
aws cloudwatch put-metric-alarm \
  --alarm-name lambda-cost-alert \
  --alarm-description "Alert when Lambda costs exceed threshold" \
  --metric-name EstimatedCharges \
  --namespace AWS/Billing \
  --statistic Maximum \
  --period 21600 \
  --evaluation-periods 1 \
  --threshold 100 \
  --comparison-operator GreaterThanThreshold
```

### Cost Explorer Tags

```bash
cargo lambda deploy \
  --tags Environment=production,Team=backend,CostCenter=engineering
```

## Real-World Optimization Example

**Before**:
- Memory: 128 MB
- Duration: 2000ms
- Requests: 10M/month
- Cost: $0.20 + (0.128 GB × 2s × 10M × $0.0000166667) = $42.87/month

**After Optimization**:
- Binary size: 8MB → 2MB (cold start: 800ms → 300ms)
- Architecture: x86_64 → ARM64 (20% cheaper)
- Memory: 128 MB → 512 MB (duration: 2000ms → 600ms)
- Duration improvement: More CPU = faster execution

**Cost calculation**:
- Compute: 0.512 GB × 0.6s × 10M × $0.0000166667 × 0.8 (ARM discount) = $40.96
- Requests: $0.20
- **Total**: $41.16/month (4% savings with better performance!)

**Key lesson**: Sometimes more memory = lower total cost due to faster execution.

## Rust Performance Advantage

Rust vs Python/Node.js:
- **3-4x cheaper** on average
- **3-10x faster execution**
- **2-3x faster cold starts**
- **Lower memory usage**

Guide the user through cost optimization based on their workload and budget constraints.
