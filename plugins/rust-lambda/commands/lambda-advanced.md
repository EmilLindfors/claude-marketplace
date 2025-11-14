---
description: Advanced Lambda topics including extensions, container images, and local development
---

You are helping the user with advanced Rust Lambda topics including custom extensions, container images, and enhanced local development.

## Your Task

Guide the user through advanced Lambda patterns and deployment options.

## Lambda Extensions

Extensions run alongside your function to provide observability, security, or governance capabilities.

### When to Build Extensions

- Custom monitoring/observability
- Secret rotation
- Configuration management
- Security scanning
- Custom logging

### Creating a Rust Extension

Add to `Cargo.toml`:
```toml
[dependencies]
lambda-extension = "0.13"
tokio = { version = "1", features = ["macros"] }
tracing = "0.1"
```

Basic extension:
```rust
use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};
use tracing::info;

async fn handler(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            info!("Shutting down extension");
        }
        NextEvent::Invoke(_e) => {
            info!("Function invoked");
            // Collect telemetry, logs, etc.
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let extension_name = "my-rust-extension";
    lambda_extension::run(service_fn(handler)).await
}
```

### Deploy Extension as Layer

```bash
# Build extension
cargo lambda build --release --extension

# Create layer
aws lambda publish-layer-version \
  --layer-name my-rust-extension \
  --zip-file fileb://target/lambda/extensions/my-extension.zip \
  --compatible-runtimes provided.al2023 \
  --compatible-architectures arm64

# Add to function
cargo lambda deploy \
  --layers arn:aws:lambda:region:account:layer:my-rust-extension:1
```

### Logging Extension Example

```rust
use lambda_extension::{service_fn, Error, LambdaLog, LambdaLogRecord};
use std::fs::OpenOptions;
use std::io::Write;

async fn handler(logs: Vec<LambdaLog>) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/extension-logs.txt")?;

    for log in logs {
        match log.record {
            LambdaLogRecord::Function(record) => {
                writeln!(file, "[FUNCTION] {}", record)?;
            }
            LambdaLogRecord::Extension(record) => {
                writeln!(file, "[EXTENSION] {}", record)?;
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Container Images

Deploy Lambda as container image instead of ZIP (max 10GB vs 250MB).

### When to Use Containers

**Use containers when**:
- Large dependencies (>250MB uncompressed)
- Custom system libraries
- Complex build process
- Team familiar with Docker
- Need exact runtime control

**Use ZIP when**:
- Simple deployment
- Fast iteration
- Smaller functions
- Standard dependencies

### Dockerfile for Rust Lambda

```dockerfile
FROM public.ecr.aws/lambda/provided:al2023-arm64

# Install Rust
RUN yum install -y gcc && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    source $HOME/.cargo/env

# Copy source
WORKDIR /var/task
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build
RUN source $HOME/.cargo/env && \
    cargo build --release && \
    cp target/release/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["bootstrap"]
```

### Multi-stage Build (Smaller Image)

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM public.ecr.aws/lambda/provided:al2023

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/bootstrap \
    ${LAMBDA_RUNTIME_DIR}/bootstrap

CMD ["bootstrap"]
```

### Build and Deploy Container

```bash
# Build image
docker build -t my-rust-lambda .

# Tag for ECR
docker tag my-rust-lambda:latest \
  123456789012.dkr.ecr.us-east-1.amazonaws.com/my-rust-lambda:latest

# Login to ECR
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin \
  123456789012.dkr.ecr.us-east-1.amazonaws.com

# Push
docker push 123456789012.dkr.ecr.us-east-1.amazonaws.com/my-rust-lambda:latest

# Create/update Lambda
aws lambda create-function \
  --function-name my-rust-lambda \
  --package-type Image \
  --code ImageUri=123456789012.dkr.ecr.us-east-1.amazonaws.com/my-rust-lambda:latest \
  --role arn:aws:iam::123456789012:role/lambda-role
```

## Local Development

### Option 1: cargo-lambda watch (Recommended)

```bash
# Start local Lambda emulator
cargo lambda watch

# Invoke in another terminal
cargo lambda invoke --data-ascii '{"test": "data"}'

# With specific event file
cargo lambda invoke --data-file events/api-gateway.json
```

### Option 2: LocalStack (Full AWS Emulation)

```bash
# Install LocalStack
pip install localstack

# Start LocalStack
localstack start

# Deploy to LocalStack
samlocal deploy

# Or with cargo-lambda
cargo lambda build --release
aws --endpoint-url=http://localhost:4566 lambda create-function \
  --function-name my-function \
  --runtime provided.al2023 \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --handler bootstrap \
  --zip-file fileb://target/lambda/bootstrap.zip
```

### Option 3: SAM Local

```bash
# template.yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31
Resources:
  MyFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: .
      Handler: bootstrap
      Runtime: provided.al2023

# Start local API
sam local start-api

# Invoke function
sam local invoke MyFunction -e events/test.json
```

## Lambda Layers (Note: Not Recommended for Rust)

**AWS Recommendation**: Don't use layers for Rust dependencies.

**Why**: Rust compiles to a single static binary. All dependencies are included at compile time.

**Exception**: Use layers for:
- Lambda Extensions
- Shared native libraries (rare)
- Non-Rust resources (config files, ML models)

## VPC Configuration

Connect Lambda to VPC for private resource access.

```bash
cargo lambda deploy \
  --subnet-ids subnet-12345 subnet-67890 \
  --security-group-ids sg-12345
```

**Performance impact**:
- Cold start: +10-15 seconds (Hyperplane ENI creation)
- Warm start: No impact

**Mitigation**:
- Use multiple subnets/AZs
- Keep functions warm
- Consider NAT Gateway for internet access

## Reserved Concurrency

Limit concurrent executions:

```bash
aws lambda put-function-concurrency \
  --function-name my-function \
  --reserved-concurrent-executions 10
```

**Use cases**:
- Protect downstream resources
- Cost control
- Predictable scaling

## Asynchronous Invocation

### Configure Destinations

```bash
# On success, send to SQS
aws lambda put-function-event-invoke-config \
  --function-name my-function \
  --destination-config '{
    "OnSuccess": {
      "Destination": "arn:aws:sqs:us-east-1:123:success-queue"
    },
    "OnFailure": {
      "Destination": "arn:aws:sns:us-east-1:123:failure-topic"
    }
  }'
```

### Dead Letter Queue

```rust
// Lambda automatically retries failed async invocations
// Configure DLQ for ultimate failures

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // If this fails after retries, goes to DLQ
    process_event(&event.payload).await?;
    Ok(Response::success())
}
```

## Event Source Mappings

### SQS with Batch Processing

```rust
use aws_lambda_events::event::sqs::SqsEvent;

async fn handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Process batch concurrently
    let futures = event.payload.records
        .into_iter()
        .map(|record| async move {
            let body: Message = serde_json::from_str(&record.body?)?;
            process_message(body).await
        });

    futures::future::try_join_all(futures).await?;

    Ok(())
}
```

Configure batch size:
```bash
aws lambda create-event-source-mapping \
  --function-name my-function \
  --event-source-arn arn:aws:sqs:us-east-1:123:my-queue \
  --batch-size 10 \
  --maximum-batching-window-in-seconds 5
```

## Advanced Error Handling

### Partial Batch Responses (SQS)

```rust
use lambda_runtime::{LambdaEvent, Error};
use aws_lambda_events::event::sqs::{SqsEvent, SqsBatchResponse};

async fn handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    let mut failed_ids = Vec::new();

    for record in event.payload.records {
        match process_record(&record).await {
            Ok(_) => {},
            Err(e) => {
                tracing::error!("Failed to process: {}", e);
                if let Some(msg_id) = record.message_id {
                    failed_ids.push(msg_id);
                }
            }
        }
    }

    Ok(SqsBatchResponse {
        batch_item_failures: failed_ids
            .into_iter()
            .map(|id| sqs::SqsBatchItemFailure { item_identifier: id })
            .collect(),
    })
}
```

## Multi-Region Deployment

```bash
# Deploy to multiple regions
for region in us-east-1 us-west-2 eu-west-1; do
  echo "Deploying to $region"
  cargo lambda deploy --region $region my-function
done
```

## Blue/Green Deployments

```bash
# Create alias
aws lambda create-alias \
  --function-name my-function \
  --name production \
  --function-version 1

# Gradual rollout
aws lambda update-alias \
  --function-name my-function \
  --name production \
  --routing-config '{"AdditionalVersionWeights": {"2": 0.1}}'

# Full cutover
aws lambda update-alias \
  --function-name my-function \
  --name production \
  --function-version 2
```

## Testing Strategies

### Integration Tests with LocalStack

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_localstack() {
        // Set LocalStack endpoint
        std::env::set_var("AWS_ENDPOINT_URL", "http://localhost:4566");

        let event = create_test_event();
        let response = function_handler(event).await.unwrap();

        assert_eq!(response.status, "success");
    }
}
```

### Load Testing

```bash
# Artillery config (artillery.yml)
config:
  target: "https://function-url.lambda-url.us-east-1.on.aws"
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 300
      arrivalRate: 100
      name: "Load test"

scenarios:
  - flow:
      - post:
          url: "/"
          json:
            test: "data"

# Run
artillery run artillery.yml
```

Guide the user through these advanced topics based on their specific needs and architecture requirements.
