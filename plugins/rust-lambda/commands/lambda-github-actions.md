---
description: Set up GitHub Actions CI/CD pipeline for Rust Lambda deployment
---

You are helping the user set up a GitHub Actions workflow for automated Lambda deployment.

## Your Task

Create a complete GitHub Actions workflow for building, testing, and deploying Rust Lambda functions.

1. **Ask about deployment preferences**:
   - Which AWS region(s)?
   - Which architecture (x86_64, arm64, or both)?
   - Deploy on every push to main, or only on tags/releases?
   - AWS authentication method (OIDC or access keys)?
   - Single function or multiple functions?

2. **Create workflow file**:
   Create `.github/workflows/deploy-lambda.yml` with appropriate configuration

3. **Set up AWS authentication**:
   - OIDC (recommended, more secure)
   - Access keys (simpler setup)

4. **Configure required secrets** in GitHub repo settings

## Complete Workflow Examples

### Option 1: OIDC Authentication (Recommended)

```yaml
name: Deploy Lambda

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  AWS_REGION: us-east-1

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --verbose

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

  build-and-deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    permissions:
      id-token: write   # Required for OIDC
      contents: read

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Zig
        uses: goto-bus-stop/setup-zig@v2
        with:
          version: 0.11.0

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Build Lambda (ARM64)
        run: cargo lambda build --release --arm64

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Deploy to AWS Lambda
        run: |
          cargo lambda deploy \
            --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE_ARN }} \
            --region ${{ env.AWS_REGION }} \
            --arch arm64

      - name: Test deployed function
        run: |
          aws lambda invoke \
            --function-name ${{ secrets.LAMBDA_FUNCTION_NAME }} \
            --payload '{"test": true}' \
            response.json
          cat response.json
```

### Option 2: Access Keys Authentication

```yaml
name: Deploy Lambda

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  AWS_REGION: us-east-1

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --verbose

  build-and-deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Zig
        uses: goto-bus-stop/setup-zig@v2
        with:
          version: 0.11.0

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Build Lambda (ARM64)
        run: cargo lambda build --release --arm64

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Deploy to AWS Lambda
        run: |
          cargo lambda deploy \
            --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE_ARN }} \
            --region ${{ env.AWS_REGION }} \
            --arch arm64
```

### Option 3: Multi-Architecture Build

```yaml
name: Deploy Lambda

on:
  push:
    branches: [ main ]
  release:
    types: [ published ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build-matrix:
    strategy:
      matrix:
        include:
          - arch: x86_64
            aws_arch: x86_64
            build_flags: ""
          - arch: arm64
            aws_arch: arm64
            build_flags: "--arm64"

    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Zig
        uses: goto-bus-stop/setup-zig@v2

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Build for ${{ matrix.arch }}
        run: cargo lambda build --release ${{ matrix.build_flags }} --output-format zip

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: lambda-${{ matrix.arch }}
          path: target/lambda/**/*.zip

  deploy:
    needs: build-matrix
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    permissions:
      id-token: write
      contents: read

    strategy:
      matrix:
        arch: [arm64, x86_64]

    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: lambda-${{ matrix.arch }}
          path: target/lambda

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: us-east-1

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Deploy ${{ matrix.arch }}
        run: |
          cargo lambda deploy my-function-${{ matrix.arch }} \
            --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE_ARN }} \
            --arch ${{ matrix.arch }}
```

### Option 4: Multiple Functions

```yaml
name: Deploy Lambda Functions

on:
  push:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  AWS_REGION: us-east-1

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'

    permissions:
      id-token: write
      contents: read

    strategy:
      matrix:
        function:
          - name: api-handler
            memory: 512
            timeout: 30
          - name: data-processor
            memory: 2048
            timeout: 300
          - name: event-consumer
            memory: 1024
            timeout: 60

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Zig
        uses: goto-bus-stop/setup-zig@v2

      - name: Install cargo-lambda
        run: pip install cargo-lambda

      - name: Build ${{ matrix.function.name }}
        run: cargo lambda build --release --arm64 --bin ${{ matrix.function.name }}

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ${{ env.AWS_REGION }}

      - name: Deploy ${{ matrix.function.name }}
        run: |
          cargo lambda deploy ${{ matrix.function.name }} \
            --iam-role ${{ secrets.LAMBDA_EXECUTION_ROLE_ARN }} \
            --region ${{ env.AWS_REGION }} \
            --memory ${{ matrix.function.memory }} \
            --timeout ${{ matrix.function.timeout }} \
            --arch arm64 \
            --env-var RUST_LOG=info
```

## AWS OIDC Setup

For OIDC authentication (recommended), set up in AWS:

### 1. Create OIDC Provider in AWS IAM

```bash
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com \
  --thumbprint-list 6938fd4d98bab03faadb97b34396831e3780aea1
```

### 2. Create IAM Role for GitHub Actions

```bash
# Create trust policy
cat > github-actions-trust-policy.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::YOUR_ACCOUNT_ID:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
        },
        "StringLike": {
          "token.actions.githubusercontent.com:sub": "repo:YOUR_GITHUB_ORG/YOUR_REPO:*"
        }
      }
    }
  ]
}
EOF

# Create role
aws iam create-role \
  --role-name GitHubActionsLambdaDeployRole \
  --assume-role-policy-document file://github-actions-trust-policy.json

# Attach policies
aws iam attach-role-policy \
  --role-name GitHubActionsLambdaDeployRole \
  --policy-arn arn:aws:iam::aws:policy/AWSLambda_FullAccess

# Get role ARN (save this for GitHub secrets)
aws iam get-role --role-name GitHubActionsLambdaDeployRole --query 'Role.Arn'
```

## GitHub Secrets Configuration

Configure these secrets in GitHub repository settings (Settings → Secrets and variables → Actions):

### For OIDC:
- `AWS_ROLE_ARN`: ARN of the GitHub Actions IAM role
- `LAMBDA_EXECUTION_ROLE_ARN`: ARN of the Lambda execution role
- `LAMBDA_FUNCTION_NAME` (optional): Function name if different from repo

### For Access Keys:
- `AWS_ACCESS_KEY_ID`: AWS access key
- `AWS_SECRET_ACCESS_KEY`: AWS secret key
- `LAMBDA_EXECUTION_ROLE_ARN`: ARN of the Lambda execution role

### Optional secrets:
- `AWS_REGION`: Override default region
- Environment-specific variables as needed

## Advanced Patterns

### Deploy on Git Tags

```yaml
on:
  push:
    tags:
      - 'v*'

# In deploy step:
- name: Get tag version
  id: tag
  run: echo "version=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT

- name: Deploy with version tag
  run: |
    cargo lambda deploy \
      --tags Version=${{ steps.tag.outputs.version }}
```

### Environment-Specific Deployment

```yaml
on:
  push:
    branches:
      - main
      - develop

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Set environment
        id: env
        run: |
          if [ "${{ github.ref }}" = "refs/heads/main" ]; then
            echo "name=production" >> $GITHUB_OUTPUT
            echo "suffix=" >> $GITHUB_OUTPUT
          else
            echo "name=development" >> $GITHUB_OUTPUT
            echo "suffix=-dev" >> $GITHUB_OUTPUT
          fi

      - name: Deploy
        run: |
          cargo lambda deploy my-function${{ steps.env.outputs.suffix }} \
            --env-var ENVIRONMENT=${{ steps.env.outputs.name }}
```

### Conditional Deployment

```yaml
      - name: Check if Lambda code changed
        id: lambda-changed
        uses: dorny/paths-filter@v2
        with:
          filters: |
            lambda:
              - 'src/**'
              - 'Cargo.toml'
              - 'Cargo.lock'

      - name: Deploy Lambda
        if: steps.lambda-changed.outputs.lambda == 'true'
        run: cargo lambda deploy
```

### Slack Notifications

```yaml
      - name: Notify Slack on success
        if: success()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Lambda deployed successfully: ${{ github.repository }}"
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

## Performance Optimizations

### Faster Builds with Caching

```yaml
      - name: Cache Rust
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
```

### Parallel Jobs

```yaml
jobs:
  test:
    # Testing job

  build:
    # Build job (independent of test for speed)

  deploy:
    needs: [test, build]  # Only deploy if both succeed
```

## Troubleshooting CI/CD

### Issue: "cargo-lambda: command not found"
**Solution**: Ensure `pip install cargo-lambda` runs before use

### Issue: "Zig not found"
**Solution**: Add `goto-bus-stop/setup-zig@v2` step

### Issue: "AWS credentials not configured"
**Solution**: Verify secrets are set and aws-actions step is included

### Issue: Build caching not working
**Solution**: Use `Swatinem/rust-cache@v2` for better Rust caching

### Issue: Deployment fails intermittently
**Solution**: Add retry logic or use `aws lambda wait function-updated`

## Testing the Workflow

1. **Create workflow file** in `.github/workflows/deploy-lambda.yml`

2. **Configure secrets** in GitHub settings

3. **Push to trigger**:
   ```bash
   git add .github/workflows/deploy-lambda.yml
   git commit -m "Add Lambda deployment workflow"
   git push
   ```

4. **Monitor** in GitHub Actions tab

5. **Check logs** for any issues

## Best Practices

1. **Always run tests** before deployment
2. **Use OIDC** instead of long-lived credentials
3. **Cache dependencies** for faster builds
4. **Deploy on main branch** only, test on PRs
5. **Use matrix builds** for multiple architectures
6. **Tag deployments** with version info
7. **Add notifications** for deployment status
8. **Set up monitoring** and alerts in AWS
9. **Use environments** for production deployments
10. **Document secrets** needed in README

After creating the workflow, guide the user through:
1. Setting up required secrets
2. Testing the workflow
3. Monitoring deployments
4. Handling failures
