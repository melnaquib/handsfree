---
chapter: 15
title: "Google Cloud Run"
part: "Part III: Deployment"
use_when: "Deploying to Google Cloud Run: containers, config, secrets, scaling."
suggested_prerequisites: [ch07]
source_kb: 108
extracted_kb: 58
code_blocks: 128
quality: auto-extracted
---

# Ch 15: Google Cloud Run

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Google Cloud Run Deployment

Google Cloud Run provides a fully managed container runtime that combines the simplicity of serverless with the flexibility of containers. For MCP servers, this means you get standard Docker deployments with automatic scaling, making it an excellent choice when you need more control than Lambda offers but don't want to manage infrastructure.

Deploy MCP servers to Cloud Run using containers Configure auto-scaling for optimal cost and performance Integrate with Cloud SQL and other GCP services Implement proper secrets management Set up monitoring and alerting Understand when to choose Cloud Run over Lambda or Workers


### The Container Advantage

```text
                  Deployment Model Comparison
AWS Lambda              Cloudflare Workers        Cloud Run
ZIP Package            WASM Binary               Docker Image
Custom Runtime         V8 Isolate                Full Linux
15min timeout          30s-15min timeout         60min timeout
10GB memory max        128MB memory              32GB memory
/tmp filesystem        No filesystem             Full filesystem
AWS-specific           CF-specific               Portable
Best for:              Best for:                 Best for:
Event-driven           Edge/global               Complex workloads
Quick operations       Low latency               Long operations
AWS ecosystem          Simple compute            GCP ecosystem
```

(table/list, cells in reading order)
- When to Choose Cloud Run
- Cloud Run excels for MCP servers that need:
- Requirement
- Why Cloud Run
- Long-running operations
- Up to 60 minute timeout (vs 15min Lambda)
- Large memory workloads
- Up to 32GB RAM (vs 10GB Lambda)
- Requirement
- Why Cloud Run
- Complex dependencies
- Full Docker environment
- GPU access
- Cloud Run supports GPUs
- File system access
- Writable filesystem (in-memory)
- Portability
- Standard containers run anywhere
- GCP ecosystem
- Native integration with GCP services
- Architecture Overview

```text
                Cloud Run MCP Architecture
Claude Desktop      ▶    Cloud Run
Claude.ai           ▶    Service
Custom Client       ▶
 Cloud SQL             Secret               Cloud
 (Postgres)            Manager              Storage
                    VPC Network
    Internal          Private          Cloud
    Services          APIs             NAT
```


### Request Flow

1.

```text
# Install Google Cloud CLI
```


### brew install google-cloud-sdk  # macOS

```bash
# Or download from https://cloud.google.com/sdk/docs/install

# Authenticate
gcloud auth login
gcloud auth configure-docker

# Set project
gcloud config set project YOUR_PROJECT_ID

# Enable required APIs
gcloud services enable \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  secretmanager.googleapis.com \
  sqladmin.googleapis.com \
  artifactregistry.googleapis.com
```


### Create MCP Server Project

```bash
# Using cargo-pmcp
cargo pmcp new my-mcp-server --template cloud-run

# Or manually create project structure
```


### mkdir my-mcp-server && cd my-mcp-server

```bash
cargo init
```


### Cargo.toml Configuration

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP SDK
pmcp-sdk = { version = "0.1", features = ["http"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "tls-rustls"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Configuration
config = "0.14"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

[profile.release]

opt-level = 3
lto = true
codegen-units = 1
strip = true
```

```text
# Stage 1: Build environment
```


### FROM rust:1.75-slim-bookworm AS builder

```text
# Install build dependencies
```


### RUN apt-get update && apt-get install -y \

```text
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
```


### WORKDIR /app

```text
# Copy manifests first for dependency caching
```


### COPY Cargo.toml Cargo.lock ./

```text
# Create dummy main.rs for dependency compilation
```


### RUN mkdir src && echo "fn main() {}" > src/main.rs

```text
# Build dependencies only (cached layer)
```


### RUN cargo build --release && rm -rf src

```text
# Copy actual source code
```


### COPY src ./src

```text
# Build the application
```


### RUN touch src/main.rs && cargo build --release

```text
# Stage 2: Runtime environment
```


### FROM debian:bookworm-slim AS runtime

```text
# Install runtime dependencies
```


### RUN apt-get update && apt-get install -y \

```text
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
```

```text
# Copy binary from builder
```

COPY --from=builder /app/target/release/my-mcp-server .

```text
# Set ownership
```


### RUN chown -R appuser:appuser /app

```text
# Switch to non-root user
```


### USER appuser

```text
# Cloud Run expects PORT environment variable
```


### ENV PORT=8080 EXPOSE 8080

```text
# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
```

```text
# .dockerignore
```

target/ .git/ .gitignore .env *.md Dockerfile .dockerignore tests/ examples/ benches/


### Local Docker Testing

```bash
# Build locally
docker build -t my-mcp-server:local .

# Run locally
docker run -p 8080:8080 \
  -e DATABASE_URL="postgres://..." \
  my-mcp-server:local

# Test the server
curl http://localhost:8080/health
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```

```rust
// src/main.rs
use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
    extract::State,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod mcp;
mod tools;
mod error;

use config::Config;
use mcp::McpServer;

#[derive(Clone)]
struct AppState {
    mcp_server: Arc<McpServer>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (Cloud Run captures stdout)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))

        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize MCP server
    let mcp_server = Arc::new(McpServer::new(&config).await?);

    let state = AppState { mcp_server };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/mcp", post(handle_mcp))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Get port from environment (Cloud Run sets PORT)
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting MCP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
// ... truncated, 22 more lines in source book ...
```


### Configuration Management

```rust
// src/config.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub allowed_origins: Vec<String>,
    pub max_query_rows: usize,
    pub request_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Cloud Run injects secrets as environment variables
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?;

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(String::from)
            .collect();

        let max_query_rows = std::env::var("MAX_QUERY_ROWS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()?;

        let request_timeout_secs = std::env::var("REQUEST_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()?;

        Ok(Self {
            database_url,
            allowed_origins,
            max_query_rows,
            request_timeout_secs,

        })
    }
}
```


### MCP Server Core

```rust
// src/mcp.rs
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::config::Config;
use crate::tools;
use crate::error::McpError;

pub struct McpServer {
    pool: PgPool,
    config: Config,
}

impl McpServer {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let pool = PgPool::connect(&config.database_url).await?;

        // Run migrations if needed
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            pool,
            config: config.clone(),
        })
    }

    pub async fn handle_request(&self, request: Value) -> Result<Value, McpError> {
        let method = request["method"]
            .as_str()
            .ok_or_else(|| McpError::InvalidRequest("Missing method".into()))?;

        let id = &request["id"];
        let params = &request["params"];

        let result = match method {

            "initialize" => self.handle_initialize(params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tool_call(params).await,
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resource_read(params).await,
            _ => Err(McpError::MethodNotFound(method.to_string())),
        }?;

        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    fn handle_initialize(&self, _params: &Value) -> Result<Value, McpError> {
        Ok(json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
// ... truncated, 63 more lines in source book ...
```

```bash
# Deploy to Cloud Run
cargo pmcp deploy cloud-run \
  --project my-gcp-project \
  --region us-central1 \
  --service my-mcp-server

# With additional options
cargo pmcp deploy cloud-run \
  --project my-gcp-project \
  --region us-central1 \
  --service my-mcp-server \
  --memory 1Gi \
  --cpu 2 \
  --min-instances 1 \
  --max-instances 10 \
  --concurrency 80 \
  --timeout 300
```


### Manual Deployment

```bash
# Build and push to Artifact Registry
gcloud builds submit --tag gcr.io/PROJECT_ID/my-mcp-server

# Or use Artifact Registry (recommended)
gcloud artifacts repositories create mcp-servers \
  --repository-format=docker \
  --location=us-central1

docker tag my-mcp-server:local \
  us-central1-docker.pkg.dev/PROJECT_ID/mcp-servers/my-mcp-server:v1

docker push us-central1-docker.pkg.dev/PROJECT_ID/mcp-servers/my-mcp-server:v1

# Deploy to Cloud Run
gcloud run deploy my-mcp-server \
  --image us-central1-docker.pkg.dev/PROJECT_ID/mcp-servers/my-mcp-server:v1 \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 2 \
  --min-instances 1 \
  --max-instances 10 \
  --concurrency 80 \
  --timeout 300 \
  --set-env-vars "RUST_LOG=info"
```

```yaml
# service.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: my-mcp-server
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        # Scaling configuration
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "10"
        # CPU allocation
        run.googleapis.com/cpu-throttling: "false"
        # VPC connector for private resources
        run.googleapis.com/vpc-access-connector: projects/PROJECT/locations/REGION/connectors/CONNECTOR
        run.googleapis.com/vpc-access-egress: private-ranges-only
    spec:
      containerConcurrency: 80
      timeoutSeconds: 300
      containers:
        - image: us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1
          ports:
            - containerPort: 8080
          resources:
            limits:
              memory: 1Gi
              cpu: "2"
          env:
            - name: RUST_LOG
              value: info
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: database-url
                  key: latest
          startupProbe:

            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 0
            timeoutSeconds: 3
            periodSeconds: 3
            failureThreshold: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            periodSeconds: 30
```


### Deploy with:

```bash
gcloud run services replace service.yaml --region us-central1
```

```bash
# Create a secret
echo -n "postgres://user:pass@host:5432/db" | \
  gcloud secrets create database-url --data-file=-

# Grant Cloud Run access
gcloud secrets add-iam-policy-binding database-url \
  --member="serviceAccount:PROJECT_NUMBER-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```


### Mounting Secrets

```bash
# As environment variable
gcloud run deploy my-mcp-server \
  --set-secrets="DATABASE_URL=database-url:latest"

# As file (for certificates, etc.)
gcloud run deploy my-mcp-server \
  --set-secrets="/secrets/db-cert=db-certificate:latest"
```


### Accessing Secrets in Code

```rust
// Secrets are injected as environment variables
let database_url = std::env::var("DATABASE_URL")?;

// Or read from mounted file
let cert = std::fs::read_to_string("/secrets/db-cert")?;
```

```bash
# Create Cloud SQL instance
gcloud sql instances create mcp-database \
  --database-version=POSTGRES_15 \
  --tier=db-f1-micro \
  --region=us-central1 \
  --root-password=YOUR_PASSWORD

# Create database
gcloud sql databases create mcp_db --instance=mcp-database

# Create user
gcloud sql users create mcp_user \
  --instance=mcp-database \
  --password=USER_PASSWORD
```


### VPC Connector for Private IP

```bash
# Create VPC connector
gcloud compute networks vpc-access connectors create mcp-connector \
  --region us-central1 \
  --network default \
  --range 10.8.0.0/28

# Deploy with VPC connector
gcloud run deploy my-mcp-server \
  --vpc-connector mcp-connector \
  --vpc-egress private-ranges-only
```


### Connection String

```text
# Private IP (via VPC connector)
```


### DATABASE_URL=postgres://mcp_user:PASSWORD@PRIVATE_IP:5432/mcp_db

```text
# Or Cloud SQL Auth Proxy (in sidecar)
```

```yaml
# service.yaml with Cloud SQL proxy
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cloudsql-instances: PROJECT:REGION:mcp-database
    spec:
      containers:
        - image: us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1
          env:
            - name: DATABASE_URL
              value: postgres://mcp_user:PASSWORD@localhost:5432/mcp_db
```

Cloud Run automatically captures stdout/stderr.

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
        )
        .init();
}

// Usage
tracing::info!(
    tool = tool_name,
    duration_ms = elapsed.as_millis(),
    "Tool execution completed"
);
```


### Cloud Monitoring Metrics

```rust
# View metrics
gcloud run services describe my-mcp-server --format="value(status.url)"

# Custom metrics via OpenTelemetry
# Add to Cargo.toml:
# opentelemetry = "0.21"
# opentelemetry-gcp = "0.10"

use opentelemetry::metrics::{Counter, Histogram};
use once_cell::sync::Lazy;

static TOOL_CALLS: Lazy<Counter<u64>> = Lazy::new(|| {
    let meter = opentelemetry::global::meter("mcp-server");
    meter.u64_counter("mcp.tool.calls").init()
});

static TOOL_LATENCY: Lazy<Histogram<f64>> = Lazy::new(|| {
    let meter = opentelemetry::global::meter("mcp-server");
    meter.f64_histogram("mcp.tool.latency").init()
});

// Record metrics
```

TOOL_CALLS.add(1, &[KeyValue::new("tool", tool_name)]); TOOL_LATENCY.record(elapsed.as_secs_f64(), &[KeyValue::new("tool", tool_name)]);


### Alerting

```bash
# Create alert policy for high error rate
gcloud alpha monitoring policies create \
  --policy-from-file=alert-policy.yaml

# alert-policy.yaml
displayName: "MCP Server High Error Rate"
conditions:
  - displayName: "Error rate > 1%"
    conditionThreshold:
      filter: >
        resource.type="cloud_run_revision"
        AND resource.labels.service_name="my-mcp-server"
        AND metric.type="run.googleapis.com/request_count"
        AND metric.labels.response_code_class="5xx"
      comparison: COMPARISON_GT
      thresholdValue: 0.01
      duration: 300s
      aggregations:
        - alignmentPeriod: 60s
          perSeriesAligner: ALIGN_RATE
notificationChannels:
  - projects/PROJECT/notificationChannels/CHANNEL_ID
```

```yaml
# cloudbuild.yaml
steps:
  # Run tests
  - name: 'rust:1.75'
    entrypoint: 'cargo'
    args: ['test']

  # Build Docker image
  - name: 'gcr.io/cloud-builders/docker'
    args:
      - 'build'
      - '-t'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:$COMMIT_SHA'
      - '-t'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:latest'
      - '.'

  # Push to Artifact Registry
  - name: 'gcr.io/cloud-builders/docker'
    args:
      - 'push'
      - '--all-tags'
      - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server'

  # Deploy to Cloud Run
  - name: 'gcr.io/google.com/cloudsdktool/cloud-sdk'
    entrypoint: 'gcloud'
    args:
      - 'run'
      - 'deploy'
      - 'my-mcp-server'
      - '--image'

      - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:$COMMIT_SHA'
      - '--region'
      - 'us-central1'
      - '--platform'
      - 'managed'

images:
  - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:$COMMIT_SHA'
  - 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:latest'

options:
  logging: CLOUD_LOGGING_ONLY
```


### Trigger Setup

```bash
# Create trigger for main branch
gcloud builds triggers create github \
  --repo-name=my-mcp-server \
  --repo-owner=myorg \
  --branch-pattern="^main$" \
  --build-config=cloudbuild.yaml
```

```bash
gcloud run services describe my-mcp-server \
  --region us-central1 \
  --format="value(status.url)"

# Example: https://my-mcp-server-abc123-uc.a.run.app
```


### Claude Desktop Configuration

```json
{
  "mcpServers": {
    "cloud-run-server": {
      "url": "https://my-mcp-server-abc123-uc.a.run.app/mcp",
      "transport": "http"
    }
  }
}
```

```bash
# Require authentication
gcloud run deploy my-mcp-server --no-allow-unauthenticated

# Get identity token
```


### TOKEN=$(gcloud auth print-identity-token)

```bash
# Use with curl
curl -H "Authorization: Bearer $TOKEN" \
  https://my-mcp-server-abc123-uc.a.run.app/mcp
```


### For service-to-service authentication:

```json
{
  "mcpServers": {
    "cloud-run-server": {
      "url": "https://my-mcp-server-abc123-uc.a.run.app/mcp",
      "transport": "http",
      "headers": {
        "Authorization": "Bearer ${GOOGLE_ID_TOKEN}"
      }
    }
  }
}
```


### Summary

Google Cloud Run provides a powerful platform for MCP servers when you need:


### Key deployment steps:

1.


### Container-Based Deployment

Building optimized Docker containers for Rust MCP servers requires understanding the unique characteristics of Rust binaries and the Cloud Run execution environment.


### Why Container Size Matters

```text
                  Container Size Impact
Image Size    Pull Time     Cold Start    Registry Cost
10MB         ~1s           ~2s           $0.10/GB
50MB         ~3s           ~4s           $0.10/GB
100MB        ~5s           ~6s           $0.10/GB
500MB        ~15s          ~17s          $0.10/GB
1GB+         ~30s+         ~35s+         $0.10/GB
Target for Rust MCP servers: <50MB
```


### Rust's Container Advantage

Rust produces statically-linked binaries that can run in minimal containers:

```text
                  Language Container Comparison
Language      Base Image        Runtime Deps    Typical Size
Python       python:3.11       pip packages    500MB-1GB
Node.js      node:20           npm packages    300MB-800MB
Java         eclipse-temurin   JRE             400MB-600MB
Go           scratch/alpine    none            10MB-50MB
Rust         scratch/alpine    ca-certs only   5MB-30MB
```

```text
# Stage 1: Build
```


### RUN apt-get update && apt-get install -y \

```text
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy and build
```


### COPY . . RUN cargo build --release

```text
# Stage 2: Runtime
```

```text
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
```

This pattern separates dependency compilation from source compilation for much faster rebuilds:

```text
# Stage 1: Chef - prepare recipe
```

```text
# Stage 2: Planner - analyze dependencies
```

```text
# Stage 3: Builder - build dependencies first, then source
```


### FROM chef AS builder


### RUN apt-get update && apt-get install -y \

```text
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (cached layer)
```

COPY --from=planner /app/recipe.json recipe.json RUN cargo chef cook --release --recipe-path recipe.json

```text
# Build application
```


### COPY . . RUN cargo build --release

```text
# Stage 4: Runtime
```


### FROM debian:bookworm-slim AS runtime

```text
# Install only runtime dependencies
```


### RUN apt-get update && apt-get install -y \


### RUN useradd -m -u 1000 appuser USER appuser

WORKDIR /app COPY --from=builder /app/target/release/my-mcp-server .

(table/list, cells in reading order)
- ENV PORT=8080 EXPOSE 8080
- CMD ["./my-mcp-server"]
- Minimal Scratch-Based Container
- For the smallest possible image when you don't need a shell:

```text
# Stage 1: Build with musl for static linking
```

```text
# Build with musl target
```


### COPY . . RUN RUSTFLAGS='-C target-feature=+crt-static' \

```bash
    cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Scratch runtime (no OS, just binary)
```


### FROM scratch

```text
# Copy CA certificates for HTTPS
```


### COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

```text
# Copy binary
```

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/my-mcp-server /

```text
# Set user (numeric, since scratch has no /etc/passwd)
```

(table/list, cells in reading order)
- USER 1000
- ENV PORT=8080 EXPOSE 8080
- ENTRYPOINT ["/my-mcp-server"]
- Distroless Runtime

```text
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
```


### WORKDIR /app COPY . . RUN cargo build --release

```text
# Stage 2: Distroless runtime
```

(table/list, cells in reading order)
- FROM gcr.io/distroless/cc-debian12
- COPY --from=builder /app/target/release/my-mcp-server /
- ENV PORT=8080 EXPOSE 8080
- USER nonroot
- ENTRYPOINT ["/my-mcp-server"]
- Build Optimization Strategies
- Cargo Configuration for Smaller Binaries

```toml
# Cargo.toml
[profile.release]
opt-level = "z"        # Optimize for size (smallest)
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit
panic = "abort"        # No unwinding code
strip = true           # Strip symbols

# For production with balance of size and speed
[profile.release-optimized]
inherits = "release"
opt-level = 3          # Optimize for speed
lto = "thin"           # Faster LTO
```


### Reducing Binary Size

```bash
# Check binary size before optimization
cargo build --release
```


### ls -lh target/release/my-mcp-server

```bash
# Before: 15MB

# After Cargo.toml optimizations
cargo build --release
```


### ls -lh target/release/my-mcp-server

```text
# After: 5MB

# Additional stripping (if strip=true not in Cargo.toml)
```


### strip target/release/my-mcp-server

```text
# After strip: 3MB

# UPX compression (optional, trades startup time for size)
```


### upx --best target/release/my-mcp-server

```text
# After UPX: 1.5MB (but slower startup)
```

```bash
# Find unused dependencies
cargo install cargo-udeps
cargo +nightly udeps

# Analyze dependency tree
cargo tree --duplicates

# Check feature flags being used
cargo tree -e features
```

```toml
# Cargo.toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
sqlite = ["sqlx/sqlite"]
full = ["postgres", "mysql", "sqlite"]

# Build with specific features
```

Cloud Run supports both AMD64 and ARM64.

```text
# Cross-compilation for ARM64
```

```text
# Install cross-compilation tools
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
        apt-get update && apt-get install -y \
            gcc-aarch64-linux-gnu \
            libc6-dev-arm64-cross \
        && rustup target add aarch64-unknown-linux-gnu \
        ;; \
    "linux/amd64") \
        apt-get update && apt-get install -y \
            gcc \
            libc6-dev \
        ;; \
    esac && rm -rf /var/lib/apt/lists/*
```

```bash
# Build for target platform
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
        cargo build --release --target aarch64-unknown-linux-gnu \
        && cp target/aarch64-unknown-linux-gnu/release/my-mcp-server target/release/ \
        ;; \
    "linux/amd64") \
        cargo build --release \
        ;; \
    esac

# Runtime stage
```


### FROM --platform=$TARGETPLATFORM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

```bash
# Enable buildx
docker buildx create --use

# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1 \
  --push \
  .
```


### Deploying ARM64 to Cloud Run

```bash
# Deploy specifying ARM64
gcloud run deploy my-mcp-server \
  --image us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1 \
  --platform managed \
  --cpu-boost \
  --execution-environment gen2  # Required for ARM
```

- Always run as non-root:

```text
# Create user in builder stage if needed
```


### FROM debian:bookworm-slim AS runtime

```text
# Create non-root user with specific UID
```


### RUN groupadd -r -g 1000 appgroup && \

```text
    useradd -r -u 1000 -g appgroup -s /sbin/nologin appuser

# Set ownership of application files
```

COPY --from=builder --chown=appuser:appgroup /app/target/release/my-mcp-server /app/

(table/list, cells in reading order)
- USER appuser
- WORKDIR /app CMD ["./my-mcp-server"]
- Read-Only Filesystem
- Configure Cloud Run to use read-only container filesystem:

```yaml
# service.yaml
spec:
  template:
    spec:
      containers:
        - image: my-image
          securityContext:
            readOnlyRootFilesystem: true
          volumeMounts:
            - name: tmp
              mountPath: /tmp
      volumes:
        - name: tmp
          emptyDir:
            medium: Memory
            sizeLimit: 100Mi
```


### Vulnerability Scanning

```text
# Scan with Trivy
```

trivy image us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1

```bash
# Scan with Google's scanner
gcloud artifacts docker images scan \
  us-central1-docker.pkg.dev/PROJECT/mcp-servers/my-mcp-server:v1

# Enable automatic scanning in Artifact Registry
gcloud artifacts repositories update mcp-servers \
  --location=us-central1 \
  --enable-vulnerability-scanning
```


### Security Labels

```text
# Add security-related labels
LABEL org.opencontainers.image.source="https://github.com/org/repo" \
      org.opencontainers.image.revision="abc123" \
      org.opencontainers.image.created="2024-01-15T10:00:00Z" \
      org.opencontainers.image.licenses="MIT"
```

```text
# Install curl for health checks (debian-based)
```

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

```text
    CMD curl -f http://localhost:8080/health || exit 1
```

```rust
// src/bin/healthcheck.rs
use std::net::TcpStream;
use std::process::exit;
use std::time::Duration;

fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("127.0.0.1:{}", port);

    match TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_secs(2),
    ) {
        Ok(_) => exit(0),
        Err(_) => exit(1),
    }
}

# Copy both binaries
```

```text
    CMD ["./healthcheck"]
```


### Cloud Run Probes

```yaml
# service.yaml
spec:
  template:
    spec:
      containers:
        - image: my-image
          # Startup probe - gives time for initialization
          startupProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 0
            periodSeconds: 2
            timeoutSeconds: 3
            failureThreshold: 30  # 60 seconds max startup
          # Liveness probe - restart if unhealthy
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            periodSeconds: 30
            timeoutSeconds: 3
            failureThreshold: 3
```

```text
# Build-time arguments (baked into image)
```

```text
# Runtime environment variables (overridable at deploy)
ENV PORT=8080 \
    RUST_LOG=info \
    RUST_BACKTRACE=0

# Labels from build args
LABEL build.date="${BUILD_DATE}" \
      build.commit="${GIT_COMMIT}"
```


### Handling Secrets at Build Time

Never embed secrets in images. Use multi-stage builds to ensure secrets don't leak:

```text
# BAD - secret in final image
```

```bash
# GOOD - secret only in builder, not in final image
FROM rust:1.75 AS builder
# Secret used only during build (e.g., private registry)
RUN --mount=type=secret,id=cargo_token \
    CARGO_REGISTRIES_MY_REGISTRY_TOKEN=$(cat /run/secrets/cargo_token) \
    cargo build --release

# Final image has no secrets
```


### Build with secrets:

```bash
docker build --secret id=cargo_token,src=.cargo_token -t my-image .
```

```text
# Dockerfile.dev
```

```text
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install development tools
```

```text
# Mount source code, don't copy
```

```text
# Auto-reload on changes
```

```text
# docker-compose.yml
```


### version: '3.8'

```bash
services:
  mcp-server:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8080:8080"
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/mcp
      - RUST_LOG=debug
    depends_on:
      - db

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: mcp
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  cargo-cache:
  postgres-data:

# Start development environment
docker compose up

# Rebuild after dependency changes
docker compose up --build
```

```text
                  Layer Caching Hierarchy
Layer 1: Base image          (cached across all builds)
Layer 2: System packages     (cached if unchanged)
Layer 3: Cargo dependencies  (cached if Cargo.toml unchanged)
Layer 4: Source code         (rebuilt on code changes)
Layer 5: Final binary        (rebuilt if any above changed)
Key: Structure Dockerfile to maximize cache hits
```


### Build Cache with Cloud Build

```bash
# cloudbuild.yaml with caching
steps:
  - name: 'gcr.io/cloud-builders/docker'
    entrypoint: 'bash'
    args:
      - '-c'
      - |
        docker pull us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:cache || true
        docker build \
          --cache-from us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:cache \
          --build-arg BUILDKIT_INLINE_CACHE=1 \
          -t us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:$COMMIT_SHA \
          -t us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server:cache \
          .

  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', '--all-tags', 'us-central1-docker.pkg.dev/$PROJECT_ID/mcp-servers/my-mcp-server']
```

1.


### Practice Ideas

These informal exercises help reinforce the concepts.


### Practice 1: Size Reduction Challenge

Take an existing Rust project and create a Dockerfile that produces an image under 20MB.


### Practice 2: Build Time Optimization

Measure build times with and without cargo-chef caching.


### Practice 3: Multi-Architecture Build

Create a CI/CD pipeline that builds and pushes images for both AMD64 and ARM64.


### Auto-Scaling Configuration

Cloud Run automatically scales your MCP servers based on incoming traffic, but fine-tuning the scaling parameters is crucial for balancing cost, performance, and user experience.

```text
                  Cloud Run Scaling Model
Requests/sec    Active Instances    Scaling Behavior
     0         minInstances        Idle (scale to min)
     1-10      1-2                 Gradual scale up
     50        3-5                 Moderate load
     200       10-15               Heavy load
     1000+     50+ (up to max)     Burst scaling
   Instances
    50
    25
     5
     1
                                                        ▶
            Traffic over time
```

(table/list, cells in reading order)
- Scaling Triggers
- Cloud Run scales based on these factors:
- Trigger
- Description
- Default
- Request concurrency
- Requests per instance
- CPU utilization
- Target CPU percentage
- Startup time
- Queue depth
- Time to accept requests
- Pending requests
- 80
- 60%
- -
- -
- Request Lifecycle

```text
                  Request Lifecycle
Request Arrives
  Is there an instance with capacity?
          Yes                       No
    Route to instance         Is max instances reached?
                               Yes       No
                           Queue or   Start new instance
                           429 error   (cold start)
                  Process request
  Instance idle for scale-down period?
          No                       Yes
      Keep warm              Scale down (if > min)
```

```bash
# Basic scaling configuration
gcloud run deploy my-mcp-server \
  --min-instances 1 \       # Always keep 1 instance warm
  --max-instances 100       # Maximum scale limit

# Zero to N scaling (scale to zero when idle)
gcloud run deploy my-mcp-server \
  --min-instances 0 \       # Scale to zero
  --max-instances 50
```

(table/list, cells in reading order)
- Choosing Min Instances
- Scenario
- Recommended Min
- Reason
- Development
- Low-traffic production
- Business-critical
- 0
- 1
- 2+
- Cost savings
- Avoid cold starts
- High availability
- Predictable traffic
- Based on baseline
- Match minimum load

```yaml
# service.yaml
spec:
  template:
    metadata:
      annotations:
        # Min instances annotation
        autoscaling.knative.dev/minScale: "2"
        # Max instances annotation
        autoscaling.knative.dev/maxScale: "100"
```


### Concurrency Settings

Concurrency determines how many requests a single instance handles simultaneously:

```bash
# Set concurrency
gcloud run deploy my-mcp-server \
  --concurrency 80  # Default

# Single-threaded workloads
gcloud run deploy my-mcp-server \
  --concurrency 1

# High-concurrency async workloads
gcloud run deploy my-mcp-server \
  --concurrency 250
```


### Choosing Concurrency for MCP Servers

```text
                  Concurrency Selection Guide
MCP Server Type              Recommended Concurrency
CPU-intensive tools          10-20
Database query tools         50-80
Simple HTTP proxy            100-250
Stateless transforms         100-200
Formula: concurrency = (CPU cores × target_utilization) /
         average_request_duration_seconds
Example: 2 cores × 0.7 / 0.1s = 14 concurrent requests
```

```rust
// Measuring actual concurrency capacity
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

static ACTIVE_REQUESTS: AtomicUsize = AtomicUsize::new(0);

async fn handle_mcp_request(request: McpRequest) -> McpResponse {
    let current = ACTIVE_REQUESTS.fetch_add(1, Ordering::SeqCst);
    tracing::info!(active_requests = current + 1, "Request started");

    let result = process_request(request).await;

    let current = ACTIVE_REQUESTS.fetch_sub(1, Ordering::SeqCst);
    tracing::info!(active_requests = current - 1, "Request completed");

    result
}
```

- Always-On CPU

By default, Cloud Run throttles CPU between requests.

```bash
# Always allocate CPU (no throttling)
gcloud run deploy my-mcp-server \
  --no-cpu-throttling

# Default behavior (CPU throttled between requests)
gcloud run deploy my-mcp-server \
  --cpu-throttling

# service.yaml
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/cpu-throttling: "false"
```

(table/list, cells in reading order)
- When to Use Always-On CPU
- Use Case
- CPU Throttling
- Reason
- Standard HTTP APIs
- Yes (default)
- Cost savings
- WebSocket connections
- Background processing
- MCP with long operations
- No
- No
- No
- Maintains connections
- Consistent performance
- Predictable latency
- Cold Start Optimization
- Understanding Cold Starts

```text
                  Cold Start Timeline
Python/Node.js MCP Server:
    Container start            2-5s
    Runtime initialization     1-3s
    Dependency loading         2-10s
    Application startup        1-5s
    Total                      6-23s
Rust MCP Server:
    Container start            0.5-2s
    Binary loading             0.1-0.5s
    Application startup        0.1-1s
    Total                      0.7-3.5s
Rust advantage: 3-10x faster cold starts
```


### Optimizing Startup Time

```rust
// Lazy initialization for faster startup
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

// AVOID: Blocking initialization at startup
fn main() {
    let pool = PgPool::connect_blocking(&database_url); // Blocks startup
    run_server(pool);
}

// BETTER: Lazy initialization
static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

async fn get_pool() -> &'static PgPool {
    DB_POOL.get_or_init(|| async {
        PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Failed to connect to database")
    }).await
}

#[tokio::main]
async fn main() {
    // Start accepting requests immediately
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/mcp", post(handle_mcp));

    // Server starts fast, DB connection happens on first request
    serve(app).await;
}
```

```bash
gcloud run deploy my-mcp-server \
  --cpu-boost  # Temporarily allocate more CPU during startup

# service.yaml
spec:
  template:
    metadata:
      annotations:
        run.googleapis.com/startup-cpu-boost: "true"
```

```rust
# service.yaml
spec:
  template:
    spec:
      containers:
        - image: my-image
          startupProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 0
            periodSeconds: 2
            timeoutSeconds: 3
            failureThreshold: 30  # Allow 60 seconds for startup

// Health check that reflects actual readiness
use std::sync::atomic::{AtomicBool, Ordering};

static READY: AtomicBool = AtomicBool::new(false);

async fn health_check() -> impl IntoResponse {
    if READY.load(Ordering::SeqCst) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn initialize_app() {
    // Perform initialization
    let _ = get_pool().await;  // Initialize DB connection
    // Mark as ready
    READY.store(true, Ordering::SeqCst);
}
```

```yaml
# service.yaml - Low latency configuration
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "3"    # Always warm
        autoscaling.knative.dev/maxScale: "100"
        run.googleapis.com/cpu-throttling: "false"
        run.googleapis.com/startup-cpu-boost: "true"
    spec:
      containerConcurrency: 50  # Conservative concurrency
      timeoutSeconds: 30
      containers:
        - resources:
            limits:
              cpu: "2"
              memory: 2Gi
```

```yaml
# service.yaml - Cost optimized configuration
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "0"    # Scale to zero
        autoscaling.knative.dev/maxScale: "10"
        run.googleapis.com/cpu-throttling: "true"  # Throttle CPU
    spec:
      containerConcurrency: 100  # High concurrency
      timeoutSeconds: 300
      containers:
        - resources:
            limits:
              cpu: "1"
              memory: 512Mi
```

```yaml
# service.yaml - Burst traffic configuration
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "1"    # Minimum warm
        autoscaling.knative.dev/maxScale: "500"   # High burst capacity
        run.googleapis.com/startup-cpu-boost: "true"
    spec:
      containerConcurrency: 80
      timeoutSeconds: 60
      containers:
        - resources:
            limits:
              cpu: "2"
              memory: 1Gi
```

When all instances are at maximum concurrency, Cloud Run queues requests:

```text
                  Request Queuing Behavior
Scenario: max_instances=3, concurrency=2, 10 concurrent requests
Instance 1: [req1] [req2]  ← at capacity
Instance 2: [req3] [req4]  ← at capacity
Instance 3: [req5] [req6]  ← at capacity
Queue: [req7, req8, req9, req10]  ← waiting for capacity
If queue wait exceeds timeout → 429 Too Many Requests
```

```rust
// Client-side retry with backoff
use backoff::{ExponentialBackoff, future::retry};

async fn call_mcp_with_retry(request: McpRequest) -> Result<McpResponse> {
    let backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(30)),
        ..Default::default()
    };

    retry(backoff, || async {
        match call_mcp(&request).await {
            Ok(response) => Ok(response),
            Err(e) if e.is_rate_limited() => {
                tracing::warn!("Rate limited, retrying...");
                Err(backoff::Error::transient(e))
            }
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    }).await
}
```

```bash
# View scaling metrics
gcloud monitoring dashboards create --config-from-file=scaling-dashboard.yaml

# scaling-dashboard.yaml
displayName: "MCP Server Scaling"
mosaicLayout:
  tiles:
    - widget:
        title: "Active Instances"
        xyChart:
          dataSets:
            - timeSeriesQuery:
                timeSeriesFilter:
                  filter: >
                    resource.type="cloud_run_revision"
                    AND metric.type="run.googleapis.com/container/instance_count"
    - widget:
        title: "Request Latency (p99)"
        xyChart:
          dataSets:
            - timeSeriesQuery:
                timeSeriesFilter:
                  filter: >
                    resource.type="cloud_run_revision"
                    AND metric.type="run.googleapis.com/request_latencies"
    - widget:
        title: "Container CPU Utilization"
        xyChart:
          dataSets:
            - timeSeriesQuery:
                timeSeriesFilter:
                  filter: >
                    resource.type="cloud_run_revision"
                    AND metric.type="run.googleapis.com/container/cpu/utilizations"
    - widget:
        title: "Concurrent Requests"
        xyChart:
          dataSets:
            - timeSeriesQuery:
                timeSeriesFilter:
                  filter: >

                    resource.type="cloud_run_revision"
                    AND metric.type="run.googleapis.com/container/max_request_concurrencies"
```


### Tuning Based on Metrics

```text
                  Scaling Tuning Guide
Symptom                        Action
High latency spikes            Increase min instances
CPU utilization > 80%          Decrease concurrency
Memory pressure                Increase memory limit
Frequent cold starts           Increase min instances
429 errors during peaks        Increase max instances
High costs during idle         Decrease min instances
Inconsistent response times    Disable CPU throttling
```


### Load Testing

```text
# Install hey for load testing
```


### brew install hey

```text
# Test with increasing concurrency
```

hey -n 1000 -c 10 https://my-mcp-server.run.app/mcp hey -n 1000 -c 50 https://my-mcp-server.run.app/mcp hey -n 1000 -c 100 https://my-mcp-server.run.app/mcp

```text
# Test with sustained load
```

(table/list, cells in reading order)
- hey -z 5m -c 50 https://my-mcp-server.run.app/mcp
- Multi-Region Scaling
- Global Load Balancing
- For global MCP deployments:

```bash
# Deploy to multiple regions
gcloud run deploy my-mcp-server --region us-central1
gcloud run deploy my-mcp-server --region europe-west1
gcloud run deploy my-mcp-server --region asia-northeast1

# Create global load balancer
gcloud compute backend-services create my-mcp-backend \
  --global \
  --load-balancing-scheme=EXTERNAL_MANAGED

# Add region NEGs
gcloud compute network-endpoint-groups create my-mcp-neg-us \
  --region=us-central1 \
  --network-endpoint-type=SERVERLESS \
  --cloud-run-service=my-mcp-server
```


### Region-Specific Scaling

```text
# Different scaling per region
# us-central1 (high traffic)
```

autoscaling.knative.dev/minScale: "5" autoscaling.knative.dev/maxScale: "200"

```text
# europe-west1 (medium traffic)
```

```text
# asia-northeast1 (low traffic)
```

- Effective auto-scaling for MCP servers requires:

CPU allocation strategy - Throttling vs always-on based on use case 5.

(table/list, cells in reading order)
- Key configuration summary:
- Setting
- Low Latency
- Cost Optimized
- Balanced
- Min instances
- 3+
- Max instances
- 100+
- 0
- 10
- 1
- 50
- Setting
- Low Latency
- Cost Optimized
- Balanced
- Concurrency
- CPU throttling
- CPU boost
- 50
- No
- Yes
- 100
- Yes
- No
- 80
- No
- Yes
- Practice Ideas


### Practice 1: Load Test Analysis

Run load tests against your MCP server and identify the optimal concurrency setting.


### Practice 2: Cold Start Measurement

Measure cold start times with different configurations (CPU boost, min instances) and document the results.


### Practice 3: Cost Optimization

Calculate the monthly cost difference between min=0 and min=1 configurations for your workload.


### Comparison: Cloud Run vs Lambda vs Workers

Choosing the right deployment platform for your MCP server is one of the most impactful architectural decisions you'll make.

```text
                  Platform Architecture Comparison
AWS Lambda
   ZIP Package → Lambda Runtime → Firecracker microVM
   Event-driven, 15min timeout, 10GB memory
Cloudflare Workers
   WASM Binary → V8 Isolate → Edge Network (300+ locations)
   Request-driven, 30s CPU time, 128MB memory
Google Cloud Run
   Docker Image → gVisor Sandbox → Managed Kubernetes
   Request-driven, 60min timeout, 32GB memory
```

(table/list, cells in reading order)
- Capability Matrix
- Capability
- Lambda
- Workers
- Cloud Run
- Max timeout
- 15 min
- 30s (CPU)
- 60 min
- Capability
- Lambda
- Workers
- Cloud Run
- Max memory
- Max request size
- Max response size
- 10 GB
- 6 MB
- 6 MB
- Filesystem
- /tmp (10 GB)
- 128 MB
- 100 MB
- 100 MB
- None
- 32 GB
- 32 MB
- 32 MB
- In-memory
- Concurrency
- 1 per instance
- 1 per isolate
- Configurable
- Cold start
- 100-500ms (Rust)
- <5ms
- 500ms-3s
- GPU support
- No
- No

```text
                  Cold Start Times (Rust MCP Server)
Platform           p50        p95        p99
Workers            2ms        5ms        10ms
Lambda (SnapStart) 50ms       150ms      300ms
Lambda (standard)  100ms      300ms      500ms
Cloud Run          400ms      1.2s       2.5s
Cold Start Breakdown:
Workers:
    WASM instantiation     1-3ms
    Total                 ~5ms
Lambda (Rust):
    Environment setup      50-100ms
    Binary loading          10-30ms
    Runtime init            10-50ms
    Total                 70-180ms
Cloud Run:
    Container pull          200-500ms (cached)
    Container start         100-300ms
    Application init        50-200ms
    Total                 350-1000ms
```

(table/list, cells in reading order)
- Cold Start Mitigation
- Platform
- Mitigation Strategy
- Cost Impact
- Lambda
- Lambda
- Provisioned concurrency
- SnapStart (Java)
- Workers
- Always fast (by design)
- Cloud Run
- Min instances
- Cloud Run
- CPU boost
- $$$
- Free
- Free
- $$
- $
- Cost Comparison
- Pricing Models

```text
                  Pricing Model Comparison
AWS Lambda
    Requests: $0.20 per 1M requests
    Duration: $0.0000166667 per GB-second
    Free tier: 1M requests, 400,000 GB-seconds/month
Cloudflare Workers
    Requests: $0.30 per 1M requests (after 10M free)
    Duration: $12.50 per 1M GB-seconds
    Free tier: 100,000 requests/day, 10ms CPU/request
Google Cloud Run
    CPU: $0.00002400 per vCPU-second
    Memory: $0.00000250 per GiB-second
    Requests: $0.40 per 1M requests
    Free tier: 2M requests, 180,000 vCPU-seconds/month
```

```text
Assumptions: 10,000 requests/month, 200ms avg duration
             512MB memory (Lambda/Cloud Run)
Lambda:
    Requests: 10K × $0.0000002 = $0.002
    Duration: 10K × 0.2s × 0.5GB × $0.0000166667 = $0.017
    Total: $0.02 (within free tier)
Workers:
    Requests: Within free tier
    Total: $0.00
Cloud Run (min=0):
    Requests: 10K × $0.0000004 = $0.004
    CPU: 10K × 0.2s × 1vCPU × $0.000024 = $0.048
    Memory: 10K × 0.2s × 0.5GB × $0.0000025 = $0.0025
    Total: $0.05 (within free tier)
Winner: Workers (always free at this volume)
```


### Scenario 2: Medium Volume (1M requests/month)

```text
Assumptions: 1M requests/month, 200ms avg duration
             512MB memory, consistent traffic
Lambda:
    Requests: 1M × $0.0000002 = $0.20
    Duration: 1M × 0.2s × 0.5GB × $0.0000166667 = $1.67
    Total: ~$1.87/month
Workers:
    Requests: (1M - 300K free) × $0.0000003 = $0.21
    Total: ~$0.21/month
Cloud Run (min=0):
    Requests: (1M - 2M free) = $0 (within free tier)
    CPU: 1M × 0.2s × 1vCPU × $0.000024 = $4.80
    Memory: 1M × 0.2s × 0.5GB × $0.0000025 = $0.25
    Total: ~$5.05/month
Cloud Run (min=1):
    Base: 720h × 1vCPU × $0.0864/h = $62.21
    Total: ~$62/month (always-on instance)
Winner: Workers ($0.21) < Lambda ($1.87) < Cloud Run ($5-62)
```


### Scenario 3: High Volume (100M requests/month)

```text
Assumptions: 100M requests/month, 200ms avg duration
             1GB memory, peak traffic patterns
Lambda:
    Requests: 100M × $0.0000002 = $20
    Duration: 100M × 0.2s × 1GB × $0.0000166667 = $333
    Total: ~$353/month
Workers:
    Requests: (100M - 10M) × $0.0000003 = $27
    Duration: 100M × 0.01s × $0.0000125 = $12.50
    Total: ~$40/month
Cloud Run (min=5, max=50):
    Base min instances: 720h × 5 × $0.12/h = $432
    Burst capacity: variable
    Total: ~$500-800/month
Winner: Workers ($40) < Lambda ($353) < Cloud Run ($500+)
```

(table/list, cells in reading order)
- Cost Summary
- Volume
- Best Choice
- Monthly Cost
- <100K
- 100K-1M
- 1M-10M
- Workers (free)
- Workers
- Workers
- $0
- $0-1
- $1-30
- 10M-100M
- Workers or Lambda
- $30-400
- Volume
- Best Choice
- Monthly Cost
- 100M+
- Workers
- $40+

Note: Cloud Run becomes competitive when you need features it uniquely provides (long timeouts, large memory, GPUs).

```text
                  Platform Selection Flowchart
Start
Need GPU acceleration?
 Yes                                       ▶ Cloud Run
 No
Need >15 minute timeout?
 Yes                                       ▶ Cloud Run
 No
Need >128MB memory?
 Yes
Need >10GB memory?
 Yes                                       ▶ Cloud Run
 No                                        ▶ Lambda
```

```text
 No (≤128MB)
Need global edge deployment?
 Yes
Operations take <30s CPU time?
 Yes                                       ▶ Workers
 No                                        ▶ Lambda + CloudFront
 No (regional is fine)
In AWS ecosystem?
 Yes                                       ▶ Lambda
 No                                        ▶ Workers (default)
```

```rust
// Lambda excels at AWS integrations
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, LambdaEvent};

async fn handler(event: LambdaEvent<McpRequest>) -> Result<McpResponse, Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    // Native DynamoDB integration
    let result = client
        .get_item()
        .table_name("mcp-data")
        .key("id", AttributeValue::S(event.payload.id))
        .send()
        .await?;

    Ok(process_result(result))
}
```


### Choose Workers When:

Global edge deployment: Sub-50ms latency worldwide Low memory requirements: ≤128MB is sufficient Simple compute: Transformations, routing, caching Cost sensitivity: Best pricing at most volumes Fast cold starts: User-facing APIs

```rust
// Workers excels at edge compute
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Request processed at edge location closest to user
    let cache = env.kv("CACHE")?;

    // Check edge cache first
    if let Some(cached) = cache.get("result").text().await? {
        return Response::ok(cached);
    }

    // Process and cache at edge
    let result = process_request(&req).await?;
    cache.put("result", &result)?.execute().await?;

    Response::ok(result)
}
```


### Choose Cloud Run When:

Long operations: Processing takes >15 minutes Large memory: Need 10GB+ for ML models, large datasets GPU workloads: ML inference, image processing Complex containers: Multiple processes, specific OS needs Portability: Same container runs anywhere

```rust
// Cloud Run excels at long/heavy operations
use axum::{routing::post, Router};
use tokio::time::Duration;

async fn ml_inference(input: Json<InferenceRequest>) -> Json<InferenceResponse> {
    // Load large model into memory (needs >10GB)
    let model = load_model("s3://models/large-llm.bin").await;

    // Long-running inference (can take 5+ minutes)
    let result = model.infer(&input.prompt).await;

    Json(InferenceResponse { result })
}
```

```text
                  Lambda → Cloud Run Migration
What Changes:
    ZIP → Docker image
    Handler function → HTTP server
    AWS SDK → GCP SDK (or keep AWS with credentials)
    CloudWatch → Cloud Logging/Monitoring
    IAM roles → Service accounts
What Stays:
    Rust code (mostly)
    Business logic
    MCP protocol handling
    External API integrations
Effort: Medium (1-2 weeks for typical MCP server)
```


### Lambda to Workers

```text
                  Lambda → Workers Migration
What Changes:
    ZIP → WASM binary
    tokio → wasm-bindgen-futures
    AWS SDK → Workers bindings (KV, D1, R2)
    std::fs → Workers storage APIs
    Some crates may not compile to WASM
What Stays:
    Pure Rust logic
    serde serialization
    MCP protocol handling
    HTTP request/response patterns
Effort: High (2-4 weeks, WASM compatibility work)
```


### Workers to Lambda

```text
                  Workers → Lambda Migration
What Changes:
    WASM → Native binary (easier)
    Workers bindings → AWS SDK
    KV/D1 → DynamoDB/RDS
    R2 → S3
    Edge deployment → Regional deployment
What Stays:
    All Rust code (WASM subset compiles to native)
    Business logic
    MCP protocol handling
    HTTP patterns
Effort: Low-Medium (1-2 weeks, mostly SDK swaps)
```

```text
                  Hybrid MCP Architecture
  Client Request     ▶  Workers (Edge)
                         - Auth check
                         - Rate limiting
                         - Caching
     Lambda               Lambda             Cloud Run
  - Quick tools        - DB queries         - ML inference
  - <100ms             - AWS integr.        - Long ops
Benefits:
    Edge caching reduces backend calls
    Route to best platform per operation type
    Scale each tier independently
    Graceful fallback between platforms
```


### Implementation

```rust
// Workers edge router
#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let mcp_request: McpRequest = req.json().await?;

    // Route based on tool type
    let backend_url = match mcp_request.tool_name.as_str() {
        // Quick operations → Lambda
        "search" | "lookup" | "validate" => {
            env.var("LAMBDA_URL")?.to_string()
        }
        // Database operations → Lambda (AWS integration)
        "query" | "insert" | "update" => {
            env.var("LAMBDA_DB_URL")?.to_string()
        }
        // Heavy operations → Cloud Run
        "analyze" | "generate" | "process" => {
            env.var("CLOUD_RUN_URL")?.to_string()
        }
        // Default to Lambda
        _ => env.var("LAMBDA_URL")?.to_string()
    };

    // Forward to appropriate backend
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    Fetch::Request(Request::new_with_init(
        &backend_url,
        RequestInit::new()
            .with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(serde_json::to_string(&mcp_request)?.into())),
    )?)
    .send()

    .await
}
```

(table/list, cells in reading order)
- Summary
- Quick Reference
- Factor
- Lambda
- Workers
- Cloud Run
- Best for
- AWS integration
- Global edge
- Heavy workloads
- Cold start
- 100-500ms
- Max memory
- 10 GB
- Max timeout
- 15 min
- <5ms
- 128 MB
- 30s CPU
- 500ms-3s
- 32 GB
- 60 min
- Pricing model
- Per request + duration
- Per request
- Per resource
- Cost at scale
- Medium
- Deployment
- ZIP or Container
- Lowest
- WASM
- Highest
- Container
- Ecosystem
- AWS
- Cloudflare
- GCP
- Recommendations by Use Case
- MCP Server Type

1. Use Lambda for AWS ecosystem integration or when you need more memory/time 3. Choose Cloud Run when you need maximum flexibility, GPUs, or very long operations 4.


### Practice Ideas

These informal exercises help reinforce the concepts.


### Practice 1: Platform Comparison

Deploy the same MCP server to all three platforms and measure cold start times, response latency, and costs.


### Practice 2: Cost Analysis

Calculate the monthly cost for your expected traffic pattern on each platform and identify the break-even points.


### Practice 3: Migration Plan

Create a migration plan for moving an existing MCP server from one platform to another, identifying all required changes.
