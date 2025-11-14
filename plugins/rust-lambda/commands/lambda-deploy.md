---
description: Deploy Rust Lambda function to AWS
---

You are helping the user deploy their Rust Lambda function to AWS.

## Your Task

Guide the user through deploying their Lambda function to AWS:

1. **Prerequisites check**:
   - Function is built: `cargo lambda build --release` completed
   - AWS credentials configured
   - IAM role for Lambda execution exists (or will be created)

2. **Verify AWS credentials**:
   ```bash
   aws sts get-caller-identity
   ```

   If not configured:
   ```bash
   aws configure
   # Or use environment variables:
   # export AWS_ACCESS_KEY_ID=...
   # export AWS_SECRET_ACCESS_KEY=...
   # export AWS_REGION=us-east-1
   ```

3. **Basic deployment**:
   ```bash
   cargo lambda deploy
   ```

   This will:
   - Use the function name from Cargo.toml (binary name)
   - Deploy to default AWS region
   - Create function if it doesn't exist
   - Update function if it exists

4. **Deployment with options**:

   **Specify function name**:
   ```bash
   cargo lambda deploy <function-name>
   ```

   **Specify region**:
   ```bash
   cargo lambda deploy --region us-west-2
   ```

   **Set IAM role**:
   ```bash
   cargo lambda deploy --iam-role arn:aws:iam::123456789012:role/lambda-execution-role
   ```

   **Configure memory**:
   ```bash
   cargo lambda deploy --memory 512
   ```
   - Default: 128 MB
   - Range: 128 MB - 10,240 MB
   - More memory = more CPU (proportional)
   - Cost increases with memory

   **Set timeout**:
   ```bash
   cargo lambda deploy --timeout 30
   ```
   - Default: 3 seconds
   - Maximum: 900 seconds (15 minutes)

   **Environment variables**:
   ```bash
   cargo lambda deploy \
     --env-var RUST_LOG=info \
     --env-var DATABASE_URL=postgres://... \
     --env-var API_KEY=secret
   ```

   **Architecture** (must match build):
   ```bash
   # For ARM64 build
   cargo lambda deploy --arch arm64

   # For x86_64 (default)
   cargo lambda deploy --arch x86_64
   ```

5. **Complete deployment example**:
   ```bash
   cargo lambda deploy my-function \
     --iam-role arn:aws:iam::123456789012:role/lambda-exec \
     --region us-east-1 \
     --memory 512 \
     --timeout 30 \
     --arch arm64 \
     --env-var RUST_LOG=info \
     --env-var API_URL=https://api.example.com
   ```

## IAM Role Setup

If user doesn't have an IAM role, guide them:

### Option 1: Let cargo-lambda create it
```bash
cargo lambda deploy --create-iam-role
```
This creates a basic execution role with CloudWatch Logs permissions.

### Option 2: Create manually with AWS CLI
```bash
# Create trust policy
cat > trust-policy.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {"Service": "lambda.amazonaws.com"},
    "Action": "sts:AssumeRole"
  }]
}
EOF

# Create role
aws iam create-role \
  --role-name lambda-execution-role \
  --assume-role-policy-document file://trust-policy.json

# Attach basic execution policy
aws iam attach-role-policy \
  --role-name lambda-execution-role \
  --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

# Get role ARN
aws iam get-role --role-name lambda-execution-role --query 'Role.Arn'
```

### Option 3: Create with additional permissions
```bash
# For S3 access
aws iam attach-role-policy \
  --role-name lambda-execution-role \
  --policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess

# For DynamoDB access
aws iam attach-role-policy \
  --role-name lambda-execution-role \
  --policy-arn arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess

# For SQS access
aws iam attach-role-policy \
  --role-name lambda-execution-role \
  --policy-arn arn:aws:iam::aws:policy/AmazonSQSFullAccess
```

## Memory Configuration Guide

Help user choose appropriate memory:

| Memory | vCPU | Use Case | Cost Multiplier |
|--------|------|----------|-----------------|
| 128 MB | 0.08 | Minimal functions | 1x |
| 512 MB | 0.33 | Standard workloads | 4x |
| 1024 MB | 0.58 | Medium compute | 8x |
| 1769 MB | 1.00 | Full 1 vCPU | 13.8x |
| 3008 MB | 1.77 | Heavy compute | 23.4x |
| 10240 MB | 6.00 | Maximum | 80x |

**Guidelines**:
- IO-intensive: 512-1024 MB usually sufficient
- Compute-intensive: 1024-3008 MB for more CPU
- Test different settings to optimize cost vs. performance

## Timeout Configuration Guide

| Timeout | Use Case |
|---------|----------|
| 3s (default) | Fast API responses, simple operations |
| 10-30s | Database queries, API calls |
| 60-300s | Data processing, file operations |
| 900s (max) | Heavy processing, batch jobs |

**Note**: Longer timeout = higher potential cost if function hangs

## Deployment Verification

After deployment, verify it works:

1. **Invoke via AWS CLI**:
   ```bash
   aws lambda invoke \
     --function-name my-function \
     --payload '{"key": "value"}' \
     response.json

   cat response.json
   ```

2. **Check logs**:
   ```bash
   aws logs tail /aws/lambda/my-function --follow
   ```

3. **Get function info**:
   ```bash
   aws lambda get-function --function-name my-function
   ```

4. **Invoke with cargo-lambda**:
   ```bash
   cargo lambda invoke --remote --data-ascii '{"test": "data"}'
   ```

## Update vs. Create

**First deployment** (function doesn't exist):
- cargo-lambda creates new function
- Requires IAM role (or use --create-iam-role)

**Subsequent deployments** (function exists):
- cargo-lambda updates function code
- Can also update configuration (memory, timeout, env vars)
- Maintains existing triggers and permissions

## Advanced Deployment Options

### Deploy from zip file
```bash
cargo lambda build --release --output-format zip
cargo lambda deploy --deployment-package target/lambda/my-function.zip
```

### Deploy with layers
```bash
cargo lambda deploy --layers arn:aws:lambda:us-east-1:123456789012:layer:my-layer:1
```

### Deploy with VPC configuration
```bash
cargo lambda deploy \
  --subnet-ids subnet-12345 subnet-67890 \
  --security-group-ids sg-12345
```

### Deploy with reserved concurrency
```bash
cargo lambda deploy --reserved-concurrency 10
```

### Deploy with tags
```bash
cargo lambda deploy \
  --tags Environment=production,Team=backend
```

## Deployment via AWS Console (Alternative)

If user prefers console:

1. Build with zip output:
   ```bash
   cargo lambda build --release --output-format zip
   ```

2. Upload via AWS Console:
   - Go to AWS Lambda Console
   - Create function or open existing
   - Upload `target/lambda/<function-name>.zip`
   - Configure runtime: "Custom runtime on Amazon Linux 2023"
   - Set handler: "bootstrap" (not needed, but convention)
   - Configure memory, timeout, env vars in console

## Multi-Function Deployment

For workspace with multiple functions:

```bash
# Deploy all
cargo lambda deploy --all

# Deploy specific
cargo lambda deploy --bin function1
cargo lambda deploy --bin function2
```

## Environment-Specific Deployment

Suggest deployment patterns:

**Development**:
```bash
cargo lambda deploy my-function-dev \
  --memory 256 \
  --timeout 10 \
  --env-var RUST_LOG=debug \
  --env-var ENV=development
```

**Production**:
```bash
cargo lambda deploy my-function \
  --memory 1024 \
  --timeout 30 \
  --arch arm64 \
  --env-var RUST_LOG=info \
  --env-var ENV=production
```

## Cost Optimization Tips

1. **Use ARM64**: 20% cheaper for same performance
2. **Right-size memory**: Test to find optimal memory/CPU
3. **Optimize timeout**: Don't set higher than needed
4. **Monitor invocations**: Use CloudWatch to track usage
5. **Consider reserved concurrency**: For predictable workloads

## Troubleshooting Deployment

### Issue: "AccessDenied"
**Solution**: Check AWS credentials and IAM permissions
```bash
aws sts get-caller-identity
```

### Issue: "Function code too large"
**Solution**:
- Uncompressed: 250 MB limit
- Compressed: 50 MB limit
- Optimize binary size (see `/lambda-build`)

### Issue: "InvalidParameterValueException: IAM role not found"
**Solution**: Create IAM role first or use --create-iam-role

### Issue: Function deployed but fails
**Solution**:
- Check CloudWatch Logs
- Verify architecture matches build (arm64 vs x86_64)
- Test locally first with `cargo lambda watch`

## Post-Deployment

After successful deployment:

1. **Test the function**:
   ```bash
   cargo lambda invoke --remote --data-ascii '{"test": "data"}'
   ```

2. **Monitor logs**:
   ```bash
   aws logs tail /aws/lambda/my-function --follow
   ```

3. **Check metrics** in AWS CloudWatch

4. **Set up CI/CD**: Use `/lambda-github-actions` for automated deployment

5. **Configure triggers** (API Gateway, S3, SQS, etc.) via AWS Console or IaC

Report deployment results including:
- Function ARN
- Region
- Memory/timeout configuration
- Invocation test results
