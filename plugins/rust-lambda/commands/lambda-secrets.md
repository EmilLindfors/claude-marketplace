---
description: Manage secrets and configuration for Rust Lambda functions using AWS Secrets Manager and Parameter Store
---

You are helping the user securely manage secrets and configuration for their Rust Lambda functions.

## Your Task

Guide the user through implementing secure secrets management using AWS Secrets Manager, Systems Manager Parameter Store, and the Parameters and Secrets Lambda Extension.

## Secrets Management Options

### Option 1: AWS Parameters and Secrets Lambda Extension (Recommended)

**Best for**:
- Production workloads
- Cost-conscious applications
- Low-latency requirements
- Local caching needs

**Advantages**:
- Cached locally (reduces latency and cost)
- No SDK calls needed
- Automatic refresh
- Works with both Secrets Manager and Parameter Store

#### Setup

1. **Add the extension layer** to your Lambda:

```bash
cargo lambda deploy \
  --layers arn:aws:lambda:us-east-1:177933569100:layer:AWS-Parameters-and-Secrets-Lambda-Extension-Arm64:11
```

For x86_64:
```bash
cargo lambda deploy \
  --layers arn:aws:lambda:us-east-1:177933569100:layer:AWS-Parameters-and-Secrets-Lambda-Extension:11
```

2. **Add IAM permissions**:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue",
        "ssm:GetParameter"
      ],
      "Resource": [
        "arn:aws:secretsmanager:us-east-1:123456789012:secret:my-secret-*",
        "arn:aws:ssm:us-east-1:123456789012:parameter/myapp/*"
      ]
    },
    {
      "Effect": "Allow",
      "Action": "kms:Decrypt",
      "Resource": "arn:aws:kms:us-east-1:123456789012:key/key-id"
    }
  ]
}
```

3. **Use the Rust client**:

Add to `Cargo.toml`:
```toml
[dependencies]
aws-parameters-and-secrets-lambda = "0.1"
serde_json = "1"
```

Basic usage:
```rust
use aws_parameters_and_secrets_lambda::{Manager, ParameterError};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::env;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Get secret from Secrets Manager
    let manager = Manager::new();
    let secret_value = manager
        .get_secret("my-database-password")
        .await?;

    // Parse as JSON if needed
    let db_config: DatabaseConfig = serde_json::from_str(&secret_value)?;

    // Use the secret
    let connection = connect_to_db(&db_config).await?;

    Ok(Response { success: true })
}

#[derive(Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}
```

#### Get Parameter Store Values

```rust
use aws_parameters_and_secrets_lambda::Manager;

async fn get_config() -> Result<AppConfig, Error> {
    let manager = Manager::new();

    // Get simple parameter
    let api_url = manager
        .get_parameter("/myapp/api-url")
        .await?;

    // Get SecureString parameter (automatically decrypted)
    let api_key = manager
        .get_parameter("/myapp/api-key")
        .await?;

    Ok(AppConfig {
        api_url,
        api_key,
    })
}
```

#### Caching and TTL

The extension caches secrets/parameters automatically. Configure TTL:

```bash
cargo lambda deploy \
  --layers arn:aws:lambda:...:layer:AWS-Parameters-and-Secrets-Lambda-Extension-Arm64:11 \
  --env-var PARAMETERS_SECRETS_EXTENSION_CACHE_ENABLED=true \
  --env-var PARAMETERS_SECRETS_EXTENSION_CACHE_SIZE=1000 \
  --env-var PARAMETERS_SECRETS_EXTENSION_MAX_CONNECTIONS=3
```

### Option 2: AWS SDK Direct Calls

**Best for**:
- Simple use cases
- One-time secret retrieval
- When extension layer isn't available

#### Using AWS SDK for Secrets Manager

Add to `Cargo.toml`:
```toml
[dependencies]
aws-config = "1"
aws-sdk-secretsmanager = "1"
```

Usage:
```rust
use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use std::sync::OnceLock;

static SECRETS_CLIENT: OnceLock<SecretsManagerClient> = OnceLock::new();

async fn get_secrets_client() -> &'static SecretsManagerClient {
    SECRETS_CLIENT.get_or_init(|| async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        SecretsManagerClient::new(&config)
    }).await
}

async fn get_database_password() -> Result<String, Error> {
    let client = get_secrets_client().await;

    let response = client
        .get_secret_value()
        .secret_id("prod/database/password")
        .send()
        .await?;

    Ok(response.secret_string().unwrap().to_string())
}

// For JSON secrets
async fn get_database_config() -> Result<DatabaseConfig, Error> {
    let client = get_secrets_client().await;

    let response = client
        .get_secret_value()
        .secret_id("prod/database/config")
        .send()
        .await?;

    let secret_string = response.secret_string().unwrap();
    let config: DatabaseConfig = serde_json::from_str(secret_string)?;

    Ok(config)
}
```

#### Using AWS SDK for Parameter Store

Add to `Cargo.toml`:
```toml
[dependencies]
aws-config = "1"
aws-sdk-ssm = "1"
```

Usage:
```rust
use aws_sdk_ssm::Client as SsmClient;
use std::sync::OnceLock;

static SSM_CLIENT: OnceLock<SsmClient> = OnceLock::new();

async fn get_ssm_client() -> &'static SsmClient {
    SSM_CLIENT.get_or_init(|| async {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        SsmClient::new(&config)
    }).await
}

async fn get_parameter(name: &str) -> Result<String, Error> {
    let client = get_ssm_client().await;

    let response = client
        .get_parameter()
        .name(name)
        .with_decryption(true)  // Decrypt SecureString
        .send()
        .await?;

    Ok(response.parameter().unwrap().value().unwrap().to_string())
}

// Get multiple parameters
async fn get_parameters_by_path(path: &str) -> Result<HashMap<String, String>, Error> {
    let client = get_ssm_client().await;

    let mut parameters = HashMap::new();
    let mut next_token = None;

    loop {
        let mut request = client
            .get_parameters_by_path()
            .path(path)
            .with_decryption(true)
            .recursive(true);

        if let Some(token) = next_token {
            request = request.next_token(token);
        }

        let response = request.send().await?;

        for param in response.parameters() {
            parameters.insert(
                param.name().unwrap().to_string(),
                param.value().unwrap().to_string(),
            );
        }

        next_token = response.next_token().map(|s| s.to_string());
        if next_token.is_none() {
            break;
        }
    }

    Ok(parameters)
}
```

### Option 3: Environment Variables (For Non-Sensitive Config)

**Best for**:
- Non-sensitive configuration
- Simple deployments
- Configuration that changes per environment

```rust
use std::env;

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let api_url = env::var("API_URL")
        .expect("API_URL must be set");

    let timeout_secs: u64 = env::var("TIMEOUT_SECONDS")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .expect("TIMEOUT_SECONDS must be a number");

    // Use configuration
    let client = build_client(&api_url, timeout_secs);

    Ok(Response { })
}
```

Deploy with environment variables:
```bash
cargo lambda deploy \
  --env-var API_URL=https://api.example.com \
  --env-var TIMEOUT_SECONDS=30 \
  --env-var ENVIRONMENT=production
```

## Best Practices

### 1. Initialize Secrets at Startup

```rust
use std::sync::OnceLock;

struct AppSecrets {
    database_password: String,
    api_key: String,
    encryption_key: String,
}

static SECRETS: OnceLock<AppSecrets> = OnceLock::new();

async fn init_secrets() -> Result<&'static AppSecrets, Error> {
    SECRETS.get_or_try_init(|| async {
        let manager = Manager::new();

        Ok(AppSecrets {
            database_password: manager.get_secret("db-password").await?,
            api_key: manager.get_parameter("/myapp/api-key").await?,
            encryption_key: manager.get_secret("encryption-key").await?,
        })
    }).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load secrets once at startup
    init_secrets().await?;

    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Access pre-loaded secrets
    let secrets = SECRETS.get().unwrap();

    let connection = connect_with_password(&secrets.database_password).await?;

    Ok(Response {})
}
```

### 2. Separate Secrets by Environment

```
# Development
/dev/myapp/database/password
/dev/myapp/api-key

# Staging
/staging/myapp/database/password
/staging/myapp/api-key

# Production
/prod/myapp/database/password
/prod/myapp/api-key
```

Usage:
```rust
let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
let param_name = format!("/{}/myapp/database/password", env);

let password = manager.get_parameter(&param_name).await?;
```

### 3. Handle Secret Rotation

```rust
use std::sync::RwLock;
use std::time::{Duration, Instant};

struct CachedSecret {
    value: String,
    last_updated: Instant,
    ttl: Duration,
}

static SECRET_CACHE: OnceLock<RwLock<HashMap<String, CachedSecret>>> = OnceLock::new();

async fn get_secret_with_ttl(name: &str, ttl: Duration) -> Result<String, Error> {
    let cache = SECRET_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    // Check cache
    {
        let cache = cache.read().unwrap();
        if let Some(cached) = cache.get(name) {
            if cached.last_updated.elapsed() < cached.ttl {
                return Ok(cached.value.clone());
            }
        }
    }

    // Fetch new value
    let manager = Manager::new();
    let value = manager.get_secret(name).await?;

    // Update cache
    {
        let mut cache = cache.write().unwrap();
        cache.insert(name.to_string(), CachedSecret {
            value: value.clone(),
            last_updated: Instant::now(),
            ttl,
        });
    }

    Ok(value)
}
```

### 4. Validate Secrets Format

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum SecretError {
    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

fn validate_database_config(config: &DatabaseConfig) -> Result<(), SecretError> {
    if config.host.is_empty() {
        return Err(SecretError::MissingField("host".to_string()));
    }

    if config.port == 0 {
        return Err(SecretError::InvalidFormat("Port must be non-zero".to_string()));
    }

    if config.password.len() < 12 {
        return Err(SecretError::InvalidFormat(
            "Password must be at least 12 characters".to_string()
        ));
    }

    Ok(())
}
```

## Creating Secrets

### Via AWS CLI

**Secrets Manager**:
```bash
# Simple string secret
aws secretsmanager create-secret \
  --name prod/database/password \
  --secret-string "MySuperSecretPassword123!"

# JSON secret
aws secretsmanager create-secret \
  --name prod/database/config \
  --secret-string '{
    "host": "db.example.com",
    "port": 5432,
    "username": "app_user",
    "password": "MySuperSecretPassword123!",
    "database": "myapp"
  }'
```

**Parameter Store**:
```bash
# String parameter
aws ssm put-parameter \
  --name /myapp/api-url \
  --value "https://api.example.com" \
  --type String

# SecureString parameter (encrypted)
aws ssm put-parameter \
  --name /myapp/api-key \
  --value "sk_live_abc123" \
  --type SecureString

# With KMS key
aws ssm put-parameter \
  --name /myapp/encryption-key \
  --value "my-encryption-key" \
  --type SecureString \
  --key-id alias/myapp-key
```

### Via Terraform

**Secrets Manager**:
```hcl
resource "aws_secretsmanager_secret" "database_password" {
  name        = "prod/database/password"
  description = "Database password for production"
}

resource "aws_secretsmanager_secret_version" "database_password" {
  secret_id     = aws_secretsmanager_secret.database_password.id
  secret_string = var.database_password  # From Terraform variables
}

# JSON secret
resource "aws_secretsmanager_secret" "database_config" {
  name = "prod/database/config"
}

resource "aws_secretsmanager_secret_version" "database_config" {
  secret_id = aws_secretsmanager_secret.database_config.id
  secret_string = jsonencode({
    host     = "db.example.com"
    port     = 5432
    username = "app_user"
    password = var.database_password
    database = "myapp"
  })
}
```

**Parameter Store**:
```hcl
resource "aws_ssm_parameter" "api_url" {
  name  = "/myapp/api-url"
  type  = "String"
  value = "https://api.example.com"
}

resource "aws_ssm_parameter" "api_key" {
  name  = "/myapp/api-key"
  type  = "SecureString"
  value = var.api_key
}
```

## Secrets Manager vs Parameter Store

| Feature | Secrets Manager | Parameter Store |
|---------|----------------|-----------------|
| Cost | $0.40/secret/month + API calls | Free (Standard), $0.05/param/month (Advanced) |
| Max size | 65 KB | 4 KB (Standard), 8 KB (Advanced) |
| Rotation | Built-in | Manual |
| Versioning | Yes | Yes |
| Cross-account | Yes | Yes (Advanced) |
| Best for | Passwords, API keys | Configuration, non-rotated secrets |

## Complete Example

```rust
use aws_parameters_and_secrets_lambda::Manager;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;
use std::sync::OnceLock;
use tracing::info;

#[derive(Deserialize, Clone)]
struct AppConfig {
    database: DatabaseConfig,
    api_key: String,
    feature_flags: FeatureFlags,
}

#[derive(Deserialize, Clone)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

#[derive(Deserialize, Clone)]
struct FeatureFlags {
    new_feature_enabled: bool,
    max_batch_size: usize,
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

async fn load_config() -> Result<&'static AppConfig, Error> {
    CONFIG.get_or_try_init(|| async {
        let manager = Manager::new();
        let env = std::env::var("ENVIRONMENT")?;

        // Get database config from Secrets Manager
        let db_secret = manager
            .get_secret(&format!("{}/database/config", env))
            .await?;
        let database: DatabaseConfig = serde_json::from_str(&db_secret)?;

        // Get API key from Parameter Store
        let api_key = manager
            .get_parameter(&format!("/{}/api-key", env))
            .await?;

        // Get feature flags from Parameter Store
        let flags_json = manager
            .get_parameter(&format!("/{}/feature-flags", env))
            .await?;
        let feature_flags: FeatureFlags = serde_json::from_str(&flags_json)?;

        Ok(AppConfig {
            database,
            api_key,
            feature_flags,
        })
    }).await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    // Load configuration at startup
    info!("Loading configuration...");
    load_config().await?;
    info!("Configuration loaded successfully");

    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let config = CONFIG.get().unwrap();

    info!("Processing request with feature flags: {:?}", config.feature_flags);

    // Use configuration
    let db = connect_to_database(&config.database).await?;
    let api_client = ApiClient::new(&config.api_key);

    // Your business logic here

    Ok(Response { success: true })
}
```

## Security Checklist

- [ ] Use Secrets Manager for sensitive data (passwords, keys)
- [ ] Use Parameter Store for configuration
- [ ] Never log secret values
- [ ] Use IAM policies to restrict access
- [ ] Enable encryption at rest (KMS)
- [ ] Use separate secrets per environment
- [ ] Implement secret rotation
- [ ] Validate secret format at startup
- [ ] Cache secrets to reduce API calls
- [ ] Use extension layer for production
- [ ] Set appropriate TTL for cached secrets
- [ ] Monitor secret access in CloudTrail
- [ ] Use least privilege IAM permissions

Guide the user through implementing secure secrets management appropriate for their needs.
