---
description: Set up Infrastructure as Code for Rust Lambda functions using SAM, Terraform, or CDK
---

You are helping the user set up Infrastructure as Code (IaC) for their Rust Lambda functions.

## Your Task

Guide the user through deploying and managing Lambda infrastructure using their preferred IaC tool.

## Infrastructure as Code Options

### Option 1: AWS SAM (Serverless Application Model)

**Best for**:
- Serverless-focused projects
- Quick prototyping
- Built-in local testing
- Teams familiar with CloudFormation

**Advantages**:
- Official AWS support for Rust with cargo-lambda
- Built-in local testing with `sam local`
- Simpler for pure serverless applications
- Good integration with Lambda features

#### Basic SAM Template

Create `template.yaml`:

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31
Description: Rust Lambda Function

Globals:
  Function:
    Timeout: 30
    MemorySize: 512
    Runtime: provided.al2023
    Architectures:
      - arm64
    Environment:
      Variables:
        RUST_LOG: info

Resources:
  MyRustFunction:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: my-function
    Properties:
      CodeUri: .
      Handler: bootstrap
      Events:
        ApiEvent:
          Type: Api
          Properties:
            Path: /hello
            Method: get
      Policies:
        - AWSLambdaBasicExecutionRole

  ComputeFunction:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
    Properties:
      CodeUri: .
      Handler: bootstrap
      MemorySize: 2048
      Timeout: 300
      Events:
        S3Event:
          Type: S3
          Properties:
            Bucket: !Ref ProcessingBucket
            Events: s3:ObjectCreated:*

  ProcessingBucket:
    Type: AWS::S3::Bucket

Outputs:
  ApiUrl:
    Description: "API Gateway endpoint URL"
    Value: !Sub "https://${ServerlessRestApi}.execute-api.${AWS::Region}.amazonaws.com/Prod/hello/"
```

#### SAM Commands

```bash
# Build
sam build

# Test locally
sam local invoke MyRustFunction -e events/test.json

# Start local API
sam local start-api

# Deploy
sam deploy --guided

# Deploy with parameters
sam deploy \
  --stack-name my-rust-lambda \
  --capabilities CAPABILITY_IAM \
  --region us-east-1
```

#### Multi-Function SAM Template

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Globals:
  Function:
    Runtime: provided.al2023
    Architectures:
      - arm64
    Environment:
      Variables:
        RUST_LOG: info

Resources:
  # API Handler - IO-optimized
  ApiHandler:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: api-handler
    Properties:
      CodeUri: .
      Handler: bootstrap
      MemorySize: 512
      Timeout: 30
      Events:
        GetUsers:
          Type: Api
          Properties:
            Path: /users
            Method: get
      Environment:
        Variables:
          DATABASE_URL: !Sub "{{resolve:secretsmanager:${DBSecret}:SecretString:connection_string}}"
      Policies:
        - AWSLambdaBasicExecutionRole
        - AWSSecretsManagerGetSecretValuePolicy:
            SecretArn: !Ref DBSecret

  # Data Processor - Compute-optimized
  DataProcessor:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: data-processor
    Properties:
      CodeUri: .
      Handler: bootstrap
      MemorySize: 3008
      Timeout: 300
      Events:
        S3Upload:
          Type: S3
          Properties:
            Bucket: !Ref DataBucket
            Events: s3:ObjectCreated:*
            Filter:
              S3Key:
                Rules:
                  - Name: prefix
                    Value: raw/
      Policies:
        - AWSLambdaBasicExecutionRole
        - S3ReadPolicy:
            BucketName: !Ref DataBucket
        - S3WritePolicy:
            BucketName: !Ref DataBucket

  # Event Consumer - SQS triggered
  EventConsumer:
    Type: AWS::Serverless::Function
    Metadata:
      BuildMethod: rust-cargolambda
      BuildProperties:
        Binary: event-consumer
    Properties:
      CodeUri: .
      Handler: bootstrap
      MemorySize: 1024
      Timeout: 60
      Events:
        SQSEvent:
          Type: SQS
          Properties:
            Queue: !GetAtt EventQueue.Arn
            BatchSize: 10
      Policies:
        - AWSLambdaBasicExecutionRole
        - SQSPollerPolicy:
            QueueName: !GetAtt EventQueue.QueueName

  DataBucket:
    Type: AWS::S3::Bucket

  EventQueue:
    Type: AWS::SQS::Queue
    Properties:
      VisibilityTimeout: 360

  DBSecret:
    Type: AWS::SecretsManager::Secret
    Properties:
      Description: Database connection string
      GenerateSecretString:
        SecretStringTemplate: '{"username": "admin"}'
        GenerateStringKey: "password"
        PasswordLength: 32

Outputs:
  ApiEndpoint:
    Value: !Sub "https://${ServerlessRestApi}.execute-api.${AWS::Region}.amazonaws.com/Prod/"
  DataBucket:
    Value: !Ref DataBucket
  QueueUrl:
    Value: !Ref EventQueue
```

### Option 2: Terraform

**Best for**:
- Multi-cloud or hybrid infrastructure
- Complex infrastructure requirements
- Teams already using Terraform
- More control over AWS resources

**Advantages**:
- Broader ecosystem (300+ providers)
- State management
- Module reusability
- Better for mixed workloads (Lambda + EC2 + RDS, etc.)

#### Basic Terraform Configuration

Create `main.tf`:

```hcl
terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# IAM Role for Lambda
resource "aws_iam_role" "lambda_role" {
  name = "${var.function_name}-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# Lambda Function
resource "aws_lambda_function" "rust_function" {
  filename         = "target/lambda/${var.function_name}/bootstrap.zip"
  function_name    = var.function_name
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  source_code_hash = filebase64sha256("target/lambda/${var.function_name}/bootstrap.zip")
  runtime         = "provided.al2023"
  architectures   = ["arm64"]
  memory_size     = var.memory_size
  timeout         = var.timeout

  environment {
    variables = {
      RUST_LOG = var.log_level
    }
  }

  tracing_config {
    mode = "Active"
  }
}

# CloudWatch Log Group
resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = 14
}

# API Gateway (Optional)
resource "aws_apigatewayv2_api" "lambda_api" {
  name          = "${var.function_name}-api"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "lambda_stage" {
  api_id      = aws_apigatewayv2_api.lambda_api.id
  name        = "prod"
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "lambda_integration" {
  api_id           = aws_apigatewayv2_api.lambda_api.id
  integration_type = "AWS_PROXY"
  integration_uri  = aws_lambda_function.rust_function.invoke_arn
}

resource "aws_apigatewayv2_route" "lambda_route" {
  api_id    = aws_apigatewayv2_api.lambda_api.id
  route_key = "GET /hello"
  target    = "integrations/${aws_apigatewayv2_integration.lambda_integration.id}"
}

resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.rust_function.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.lambda_api.execution_arn}/*/*"
}

# Outputs
output "function_arn" {
  value = aws_lambda_function.rust_function.arn
}

output "api_endpoint" {
  value = aws_apigatewayv2_stage.lambda_stage.invoke_url
}
```

Create `variables.tf`:

```hcl
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "function_name" {
  description = "Lambda function name"
  type        = string
}

variable "memory_size" {
  description = "Lambda memory size in MB"
  type        = number
  default     = 512
}

variable "timeout" {
  description = "Lambda timeout in seconds"
  type        = number
  default     = 30
}

variable "log_level" {
  description = "Rust log level"
  type        = string
  default     = "info"
}
```

#### Terraform Module for Rust Lambda

Create `modules/rust-lambda/main.tf`:

```hcl
resource "aws_iam_role" "lambda_role" {
  name = "${var.function_name}-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_basic" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function" "function" {
  filename         = var.zip_file
  function_name    = var.function_name
  role            = aws_iam_role.lambda_role.arn
  handler         = "bootstrap"
  source_code_hash = filebase64sha256(var.zip_file)
  runtime         = "provided.al2023"
  architectures   = [var.architecture]
  memory_size     = var.memory_size
  timeout         = var.timeout

  environment {
    variables = var.environment_variables
  }

  dynamic "vpc_config" {
    for_each = var.vpc_config != null ? [var.vpc_config] : []
    content {
      subnet_ids         = vpc_config.value.subnet_ids
      security_group_ids = vpc_config.value.security_group_ids
    }
  }

  tracing_config {
    mode = var.enable_xray ? "Active" : "PassThrough"
  }
}

resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = var.log_retention_days
}
```

Usage:

```hcl
module "api_handler" {
  source = "./modules/rust-lambda"

  function_name          = "api-handler"
  zip_file              = "target/lambda/api-handler/bootstrap.zip"
  memory_size           = 512
  timeout               = 30
  architecture          = "arm64"
  enable_xray           = true
  log_retention_days    = 7

  environment_variables = {
    RUST_LOG     = "info"
    DATABASE_URL = data.aws_secretsmanager_secret_version.db.secret_string
  }
}

module "data_processor" {
  source = "./modules/rust-lambda"

  function_name          = "data-processor"
  zip_file              = "target/lambda/data-processor/bootstrap.zip"
  memory_size           = 3008
  timeout               = 300
  architecture          = "arm64"
  enable_xray           = true
  log_retention_days    = 7

  environment_variables = {
    RUST_LOG = "info"
  }
}
```

#### Terraform Commands

```bash
# Initialize
terraform init

# Plan
terraform plan -var="function_name=my-rust-lambda"

# Apply
terraform apply -var="function_name=my-rust-lambda" -auto-approve

# Destroy
terraform destroy -var="function_name=my-rust-lambda"
```

### Option 3: AWS CDK (TypeScript/Python)

**Best for**:
- Type-safe infrastructure definitions
- Complex constructs and patterns
- Teams comfortable with programming languages
- Reusable infrastructure components

#### CDK Example (TypeScript)

```typescript
import * as cdk from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as apigateway from 'aws-cdk-lib/aws-apigatewayv2';
import * as integrations from 'aws-cdk-lib/aws-apigatewayv2-integrations';

export class RustLambdaStack extends cdk.Stack {
  constructor(scope: cdk.App, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const rustFunction = new lambda.Function(this, 'RustFunction', {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: 'bootstrap',
      code: lambda.Code.fromAsset('target/lambda/my-function/bootstrap.zip'),
      architecture: lambda.Architecture.ARM_64,
      memorySize: 512,
      timeout: cdk.Duration.seconds(30),
      environment: {
        RUST_LOG: 'info',
      },
      tracing: lambda.Tracing.ACTIVE,
    });

    const api = new apigateway.HttpApi(this, 'RustApi', {
      defaultIntegration: new integrations.HttpLambdaIntegration(
        'RustIntegration',
        rustFunction
      ),
    });

    new cdk.CfnOutput(this, 'ApiUrl', {
      value: api.url!,
    });
  }
}
```

## Comparison Table

| Feature | SAM | Terraform | CDK |
|---------|-----|-----------|-----|
| Learning Curve | Low | Medium | Medium-High |
| Rust Support | Excellent | Good | Good |
| Local Testing | Built-in | Limited | Limited |
| Multi-Cloud | No | Yes | No |
| Type Safety | No | HCL | Yes |
| Community | AWS-focused | Large | Growing |
| State Management | CloudFormation | Terraform State | CloudFormation |

## Integration with cargo-lambda

All IaC tools work well with cargo-lambda:

```bash
# Build for deployment
cargo lambda build --release --arm64 --output-format zip

# Then deploy with your IaC tool
sam deploy
# or
terraform apply
# or
cdk deploy
```

## Best Practices

1. **Version Control**: Store IaC templates in Git
2. **Separate Environments**: Use workspaces/stages for dev/staging/prod
3. **Secrets Management**: Use AWS Secrets Manager, never hardcode
4. **Outputs**: Export important values (ARNs, URLs)
5. **Modules**: Create reusable components
6. **Testing**: Validate templates before deployment
7. **CI/CD**: Automate IaC deployment
8. **State Management**: Secure Terraform state (S3 + DynamoDB)
9. **Documentation**: Comment complex configurations
10. **Tagging**: Tag resources for cost tracking

## Local Testing with SAM

```bash
# Test function locally
sam local invoke MyRustFunction -e events/test.json

# Start local API Gateway
sam local start-api

# Start local Lambda endpoint
sam local start-lambda

# Generate sample events
sam local generate-event apigateway aws-proxy > event.json
sam local generate-event s3 put > s3-event.json
```

## Using with LocalStack

For full local AWS emulation:

```bash
# Install LocalStack
pip install localstack

# Start LocalStack
localstack start

# Deploy to LocalStack with SAM
samlocal deploy

# Or with Terraform
terraform apply \
  -var="aws_region=us-east-1" \
  -var="endpoint=http://localhost:4566"
```

## Migration Path

**Starting fresh**:
- Choose SAM for pure serverless, simple projects
- Choose Terraform for complex, multi-service infrastructure
- Choose CDK for type-safe, programmatic definitions

**Existing infrastructure**:
- Import existing resources into Terraform/CDK
- Use CloudFormation template generation from SAM
- Gradual migration with hybrid approach

## Recommended Structure

```
my-rust-lambda/
├── src/
│   └── main.rs
├── Cargo.toml
├── template.yaml         # SAM
├── terraform/           # Terraform
│   ├── main.tf
│   ├── variables.tf
│   └── outputs.tf
├── cdk/                 # CDK
│   ├── lib/
│   │   └── stack.ts
│   └── bin/
│       └── app.ts
└── events/             # Test events
    ├── api-event.json
    └── s3-event.json
```

Help the user choose the right IaC tool based on their needs and guide them through setup and deployment.
