---
chapter: 7
title: "Deployment Overview"
part: "Part III: Deployment"
use_when: "Choosing a deployment target (Lambda, Cloudflare Workers, Cloud Run), cost model, security boundaries."
suggested_prerequisites: [ch02]
source_kb: 85
extracted_kb: 40
code_blocks: 59
quality: auto-extracted
---

# Ch 7: Deployment Overview

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Deployment Overview

In Part 1 and Part 2, we built MCP servers that run locally on your development machine. But what happens when you want to share your MCP server with your entire organization?


### The Developer vs Business User Gap

Local MCP servers have a fundamental limitation: they require technical setup on each user's machine.

```text
                 LOCAL MCP SERVER
Developer's Machine
   IDE (VS Code, Cursor, etc.)
   MCP Server Process
   Local Database / Files / APIs
[GOOD] Works great for developers
[BAD] Requires local setup, Rust toolchain, database access
[BAD] Each developer runs their own instance
[BAD] No centralized access control or monitoring
```

For a sales team to query CRM data through Claude, or for analysts to access business metrics, they shouldn't need to:

```text
                  REMOTE MCP SERVER
  Cloud Platform (AWS, GCP, Cloudflare)
     MCP Server (managed)
     Production Database / Internal APIs
            HTTPS
Developer      Analyst        Sales Rep      Support Agent
(Claude.ai)   (Claude.ai)    (Claude.ai)    (Claude.ai)
  [GOOD] No local setup required
  [GOOD] Centralized access control (OAuth, SSO)
  [GOOD] Server is close to the data (low latency)
  [GOOD] IT/Ops team manages the deployment
```


### Data Proximity: Network Latency Matters

MCP servers often need to access databases, internal APIs, and file systems.

(table/list, cells in reading order)
- Scenario
- Network Latency
- Impact on 10 DB Queries
- Server in same AWS VPC as RDS
- Server in same region, different VPC
- ~1ms
- ~5ms
- ~10ms total
- ~50ms total
- Scenario
- Network Latency
- Impact on 10 DB Queries
- Server on user's laptop, DB in cloud
- ~50-200ms
- ~500-2000ms total


### Deployment Targets

PMCP supports three primary deployment targets, each optimized for different use cases:


### AWS Lambda (Serverless)

Best for: Most production deployments, pay-per-use, AWS-native environments

```bash
cargo pmcp deploy init --target aws-lambda
cargo pmcp deploy
```


### Architecture:

```text
API Gateway     ▶   Lambda           ▶   RDS / DynamoDB /
(HTTPS)             (your server)        S3 / Internal APIs
```

(table/list, cells in reading order)
- Why Rust Excels on Lambda:
- Metric
- Rust
- Python
- Node.js
- Cold start
- ~50-100ms
- ~500-1500ms
- ~200-500ms
- Warm latency
- ~5-10ms
- ~20-50ms
- Memory footprint
- ~128MB typical
- ~256-512MB
- ~15-30ms
- ~256MB
- Binary size
- ~5-15MB
- N/A (interpreted)
- N/A

Rust's compiled binaries start almost instantly and use minimal memory.


### Features:

[GOOD] Pay only for actual usage (no idle costs) [GOOD] Automatic scaling to thousands of concurrent users [GOOD] VPC integration for private database access [GOOD] CDK-based infrastructure as code [GOOD] OAuth support via Cognito


### Cloudflare Workers (Edge + WASM)

Best for: Global distribution, sub-millisecond latency, WASM-compatible workloads

```bash
cargo pmcp deploy init --target cloudflare-workers
cargo pmcp deploy --target cloudflare-workers
```


### Architecture:

```text
    User in Tokyo          User in London         User in New York

         ▼                      ▼                       ▼
```

```text
Edge                   Edge                   Edge
(Tokyo)                (London)               (NYC)
```

```text
Origin APIs / KV
(if needed)
```


### Why Rust Excels on Cloudflare Workers:

Cloudflare Workers runs WebAssembly (WASM), and Rust has first-class WASM support:

(table/list, cells in reading order)
- Metric
- Rust → WASM
- JavaScript
- Cold start
- ~0-5ms
- CPU efficiency
- ~10x faster
- ~0-5ms
- Baseline
- Bundle size
- ~500KB-2MB
- Varies
- Memory safety
- Compile-time
- Runtime


### Features:

[GOOD] Global edge network (300+ locations) [GOOD] Sub-millisecond cold starts [GOOD] KV storage for caching [GOOD] R2 for object storage [GOOD] D1 for SQLite at the edge


### Considerations:

WASM has some limitations (no raw filesystem, limited networking) Best for stateless, CPU-bound workloads May need to adapt database access patterns


### Google Cloud Run (Containers)

Best for: Docker-based workflows, GCP-native environments, long-running requests

```bash
cargo pmcp deploy init --target google-cloud-run
cargo pmcp deploy --target google-cloud-run
```


### Architecture:

```text
Cloud Run       ▶   Container                ▶   Cloud SQL /
(HTTPS)             (your server image)          Firestore /
                                                 GCS
```

(table/list, cells in reading order)
- Why Rust Excels on Cloud Run:
- Metric
- Rust Container
- Python Container
- Image size
- ~10-20MB
- ~200-500MB
- Startup time
- ~100-300ms
- ~1-3s
- Memory at idle
- ~10-30MB
- ~100-200MB
- Min instances cost
- Lower
- Higher

Rust's tiny, statically-linked binaries produce minimal Docker images that start quickly and use less memory.


### Features:

[GOOD] Full Docker compatibility (any dependencies) [GOOD] VPC connector for private networks [GOOD] Request timeout up to 60 minutes [GOOD] Automatic HTTPS with managed certificates [GOOD] Cloud Build integration for CI/CD

```bash
# Initialize deployment configuration
cargo pmcp deploy init --target <aws-lambda|cloudflare-workers|google-cloud-run>

# Deploy to the cloud
cargo pmcp deploy [--target <target>]

# View deployment outputs (URL, etc.)
cargo pmcp deploy outputs

# View logs
cargo pmcp deploy logs [--tail]

# Manage secrets
cargo pmcp deploy secrets set <key> [--from-env <ENV_VAR>]
cargo pmcp deploy secrets list

# Destroy deployment
cargo pmcp deploy destroy [--clean]
```


### Target Selection

The  --target  flag specifies which platform to deploy to.

```toml
# .pmcp/deploy.toml
[target]
target_type = "aws-lambda"  # or "cloudflare-workers", "google-cloud-run"

[server]
name = "my-mcp-server"

[aws]
region = "us-east-1"
```

```bash
# 1. Initialize (one-time setup)
cargo pmcp deploy init --target aws-lambda

# 2. Build and deploy
cargo pmcp deploy

# 3. Verify
cargo pmcp deploy outputs
cargo pmcp deploy test

# 4. Monitor
cargo pmcp deploy logs --tail

# 5. Update (re-run deploy with new code)
cargo pmcp deploy

# 6. Rollback if needed
cargo pmcp deploy rollback

# 7. Cleanup when done
cargo pmcp deploy destroy --clean
```

```text
Which deployment target?
```

```text
              ▼               ▼               ▼
         Need global    Using AWS       Using GCP
         edge latency?  infrastructure? infrastructure?

              ▼               ▼               ▼
```

```text
Cloudflare         AWS Lambda      Google Cloud
Workers                            Run
Best for:          Best for:       Best for:
- Global users     - VPC/RDS       - Cloud SQL
- Static data      - Cognito       - Long requests
- Edge compute     - Most apps     - Docker deps
```

(table/list, cells in reading order)
- Factor
- AWS Lambda
- Cloudflare Workers
- Cloud Run
- Cold start
- ~50-100ms
- ~0-5ms
- ~100-300ms
- Max request duration
- 15 min
- 30s (50ms CPU)
- 60 min
- Private network
- VPC
- Limited
- VPC Connector
- Database access
- RDS, DynamoDB
- D1, external
- Cloud SQL
- Pricing model
- Per-request
- Per-request
- Per-container-second
- Best for
- General purpose
- Edge/global
- Long-running
- pmcp.run: Managed Hosting (Coming Soon)

```bash
cargo pmcp deploy --target pmcp-run
```


### Benefits:

No AWS/GCP/Cloudflare account needed Automatic HTTPS, scaling, and monitoring OAuth integration out of the box Pay-as-you-go pricing


### Enterprise Private Hosting

For organizations with compliance requirements, pmcp.run offers private deployments:


### Current Status

The pmcp.run service is currently in development.

```bash
# Login to pmcp.run
cargo pmcp deploy login --target pmcp-run

# Deploy
cargo pmcp deploy --target pmcp-run

# View your servers
cargo pmcp deploy outputs --target pmcp-run
```


### Summary

Remote MCP deployments transform your server from a developer tool into an organization-wide service.


### Rust excels on all deployment targets because of its:

Fast cold starts: No JIT warmup or interpreter startup Low memory usage: Efficient binaries reduce costs Small artifacts: Tiny Docker images and WASM bundles Predictable performance: No garbage collection pauses


### What's Next

Chapter 8: AWS Lambda deep dive with VPC, Cognito, and CDK Chapter 9: Cloudflare Workers for edge deployment Chapter 10: Google Cloud Run with Cloud SQL Chapter 11: Authentication and authorization patterns Chapter 12: Monitoring, logging, and observability


### Serverless vs Containers vs Edge

When deploying MCP servers to the cloud, you have three fundamental architectural choices: serverless functions, containers, and edge computing.

```text
                 SERVERLESS ARCHITECTURE
Request   ▶ API Gateway   ▶ Lambda Function   ▶ Response
                             Your Code
                             (frozen)
Between requests: Function is frozen or terminated
Scaling: Cloud spawns new instances automatically
Billing: Pay only for execution time (GB-seconds)
```


### How it works:

1.


### Rust-specific behavior:

```rust
// Lambda handler - runs for each request
async fn handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // This code runs per-request
    let response = process_mcp_request(event.payload).await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // This runs ONCE during cold start
    // Initialize expensive resources here
    tracing_subscriber::fmt::init();

    run(service_fn(handler)).await
}
```


### Containers (Google Cloud Run)

Containers package your application with its dependencies into a portable image that runs on managed infrastructure.

```text
                 CONTAINER ARCHITECTURE
Request   ▶ Load Balancer   ▶ Container Instance   ▶ Response
                              Your Server
                              (always on)
                              HTTP :8080
Between requests: Server stays running, handles concurrency
Scaling: Platform adjusts container count based on load
Billing: Pay for container uptime (vCPU-seconds + memory)
```


### How it works:

1.


### Rust container example:

```text
# Multi-stage build for minimal image
```

FROM rust:1.75 AS builder WORKDIR /app COPY . .

```rust
// Container server - runs continuously
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize once at startup
    let server = build_mcp_server().await?;

    // Run HTTP server - handles many requests
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    StreamableHttpServer::new(addr, server)
        .run()
        .await
}
```


### Edge Computing (Cloudflare Workers)

Edge functions run your code at network edge locations, close to users worldwide.

```text
                   EDGE ARCHITECTURE
       Tokyo         London        NYC
       Edge          Edge          Edge
   User          User          User
   (5ms)         (5ms)         (5ms)
Your code: Compiled to WebAssembly, distributed globally
Execution: Runs in V8 isolates (not containers)
Scaling: Automatic across 300+ locations
Billing: Pay per request + CPU time
```


### How it works:

1.


### Rust WASM example:

```rust
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Each request runs in its own isolate
    let router = Router::new();

    router
        .post_async("/mcp", |req, _| async move {
            let body = req.text().await?;
            let response = handle_mcp_request(&body).await?;
            Response::ok(response)
        })
        .run(req, env)
        .await
}
```

Cold starts occur when the platform must initialize a new execution environment:

(table/list, cells in reading order)
- Platform
- Cold Start Cause
- Typical Duration (Rust)
- Lambda
- No warm instance available
- 50-150ms
- Cloud Run
- Container scaling up
- 100-500ms
- Workers
- First request to edge location
- 0-5ms
- Lambda cold start breakdown:

```text
                 LAMBDA COLD START TIMELINE
0ms          50ms         100ms        150ms       200ms
    MicroVM      Runtime       Your        Handler
    Init         Init          main()      Exec
    (~30ms)      (~10ms)      (~10ms)      (~50ms)
 Rust advantage: main() initialization is minimal
 Python/Node: Interpreter startup adds 200-500ms
```


### Strategies to minimize cold starts:

```rust
// Lambda: Initialize expensive resources once
static DB_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

async fn get_pool() -> &'static Pool<Postgres> {
    DB_POOL.get_or_init(|| async {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap()
    }).await
}

async fn handler(event: Request) -> Result<Response> {
    // Pool is reused across warm invocations
    let pool = get_pool().await;
    // ...
}
```

```text
                  CONCURRENCY MODELS
LAMBDA (1 request per instance):
  Request 1   ▶ [Instance A]   ▶ Response 1
  Request 2   ▶ [Instance B]   ▶ Response 2
  Request 3   ▶ [Instance C]   ▶ Response 3
  Scaling: New instance for each concurrent request
  Memory: Separate per instance (128MB-10GB configurable)
CLOUD RUN (many requests per container):
  Request 1
  Request 2      ▶ [Container A]      ▶ Response 1
  Request 3                           ▶ Response 2
                                      ▶ Response 3
                 (async runtime)
  Scaling: Container handles up to 80 concurrent requests
  Memory: Shared within container (configurable 128MB-32GB)
WORKERS (isolated per request):
  Request 1   ▶ [Isolate A]   ▶ Response 1
  Request 2   ▶ [Isolate B]   ▶ Response 2
  Request 3   ▶ [Isolate C]   ▶ Response 3
  Scaling: Isolates are lightweight (microseconds to create)
  Memory: 128MB limit per isolate
```

(table/list, cells in reading order)
- Resource Limits
- Resource
- Lambda
- Cloud Run
- Workers
- Memory
- 128MB - 10GB
- 128MB - 32GB
- 128MB
- CPU
- Proportional to memory
- 1-8 vCPU
- 10-50ms CPU time
- Timeout
- 15 minutes
- 60 minutes
- 30 seconds
- Payload size
- 6MB (sync) / 20MB (async)
- 32MB
- 100MB
- Tmp storage
- 512MB - 10GB
- Ephemeral disk
- None
- When to Choose Each Option
- Choose Lambda When:

[GOOD] Sporadic traffic - Pay nothing during idle periods [GOOD] AWS-native environment - VPC, RDS, DynamoDB integration [GOOD] Unpredictable scaling - 0 to thousands of concurrent users [GOOD] Simple deployment - No container management [GOOD] OAuth with Cognito - Built-in user management

```bash
# Ideal Lambda use case: Internal business tool
cargo pmcp deploy init --target aws-lambda
cargo pmcp deploy

# Result: HTTPS endpoint with automatic scaling
# Cost: ~$0.20 per million requests (128MB, 100ms avg)
```


### Choose Cloud Run When:

[GOOD] Long-running requests - Up to 60 minutes per request [GOOD] High concurrency per instance - Efficient resource usage [GOOD] Custom dependencies - Docker flexibility [GOOD] GCP-native environment - Cloud SQL, Firestore [GOOD] Minimum instances needed - Avoid cold starts entirely

```bash
# Ideal Cloud Run use case: Data processing with large queries
cargo pmcp deploy init --target google-cloud-run
cargo pmcp deploy --target google-cloud-run

# Result: Container-based deployment with persistent connections
# Cost: ~$0.00002400/vCPU-second + memory
```


### Choose Workers When:

[GOOD] Global user base - Minimize latency worldwide [GOOD] Stateless operations - No database, or using KV/D1 [GOOD] High request volume - Millions of requests/day [GOOD] CPU-bound tasks - Parsing, transformation, validation

```bash
# Ideal Workers use case: Global API with caching
cargo pmcp deploy init --target cloudflare-workers
cargo pmcp deploy --target cloudflare-workers

# Result: Edge deployment to 300+ locations
# Cost: $0.50 per million requests (free tier: 100K/day)
```

```text
                  HYBRID DEPLOYMENT
Global Users
         Cloudflare Workers (Edge)
   - Request routing
   - Caching
   - Rate limiting
   - Authentication
          AWS Lambda (Serverless)
   - Business logic
   - Database queries
   - Complex processing
     RDS / DynamoDB (Data Layer)
```

```bash
# Migration path
cargo pmcp deploy init --target google-cloud-run
# Update environment variables
cargo pmcp deploy --target google-cloud-run
# Verify, then destroy Lambda
cargo pmcp deploy destroy --target aws-lambda --clean
```

Need global low-latency Workload is stateless Can use KV/D1 instead of RDS


### Considerations:

WASM has different capabilities than native code Database access patterns may need redesign Some crates don't compile to WASM

(table/list, cells in reading order)
- Summary
- Aspect
- Lambda
- Cloud Run
- Workers
- Execution model
- Function per request
- Container server
- WASM isolate
- Cold start (Rust)
- 50-150ms
- 100-500ms
- 0-5ms
- Concurrency
- 1 per instance
- Many per container
- 1 per isolate
- Max timeout
- 15 min
- 60 min
- 30s
- Best for
- General serverless
- Long-running, GCP
- Global edge
- Rust advantage
- Fast cold start
- Tiny images
- Native WASM
- Choose based on your specific requirements:


### Cost Analysis Framework

Understanding cloud costs is essential for production MCP deployments.

```text
                   TOTAL COST
Total Cost = Compute + Data Transfer + Storage + Extras
   Compute          Network          Storage
   - CPU time       - Egress         - Database
   - Memory         - API GW         - Logs
   - Requests       - CDN            - Secrets
```

```text
                  AWS LAMBDA PRICING
Requests:     $0.20 per 1 million requests
Duration:     $0.0000166667 per GB-second (x86)
              $0.0000133334 per GB-second (ARM64) ← 20% cheaper
Free tier:    1M requests + 400,000 GB-seconds per month
API Gateway:  $1.00 per million requests (HTTP API)
              $3.50 per million requests (REST API)
```

```yaml
  Requests:     100,000 × $0.20/1M = $0.02
  GB-seconds:   100,000 × 0.1s × 0.128GB = 1,280 GB-seconds
  Duration:     1,280 × $0.0000133334 = $0.017 (ARM64)
```


### API Gateway:

```text
  HTTP API:     100,000 × $1.00/1M = $0.10
```

```yaml
  GB-seconds:   100,000 × 0.5s × 0.256GB = 12,800 GB-seconds
  Duration:     12,800 × $0.0000133334 = $0.17

  Total:        $0.02 + $0.17 + $0.10 = $0.29/month (2.1× more)
```

Rust advantage: Faster execution and lower memory = lower costs.

```text
                GOOGLE CLOUD RUN PRICING
CPU:          $0.00002400 per vCPU-second
Memory:       $0.00000250 per GiB-second
Requests:     $0.40 per million requests
Free tier:    2M requests, 360,000 vCPU-seconds,
              180,000 GiB-seconds per month
Min instances: Billed even when idle (if configured)
```

```text
             10,000 × $0.000024 = $0.24
```


### Memory:      100,000 × 0.1s × 0.5 GiB = 5,000 GiB-seconds

```text
             5,000 × $0.0000025 = $0.0125
```

(table/list, cells in reading order)
- Requests:    100,000 × $0.40/1M = $0.04
- Total:       $0.24 + $0.0125 + $0.04 = $0.29/month
- With minimum instances (avoid cold starts):
- 1 min instance, always on:

```yaml
  Hours/month:  730 hours × 3600s = 2,628,000 seconds
  CPU:          2,628,000 × 1 vCPU × $0.000024 = $63.07
  Memory:       2,628,000 × 0.5 GiB × $0.0000025 = $3.29
```

```text
               CLOUDFLARE WORKERS PRICING
Free plan:    100,000 requests/day (no cost)
Paid plan:    $5/month base
              First 10M requests included
              $0.50 per additional million requests
CPU time:     10ms free, then $0.02 per additional million ms
KV storage:   Free reads, $0.50/million writes
D1 database:  $0.75/million rows read, $1.00/million written
```

```yaml
  Cost: $0/month
```


### Paid plan (for higher volume):

```yaml
  Base:      $5/month
  Requests:  Included (under 10M)
```

(table/list, cells in reading order)
- Total:       $5/month (flat)
- Cost Comparison by Usage Pattern
- Low Volume (10K requests/month)
- Platform
- Monthly Cost
- Notes
- Lambda (ARM64)
- Cloud Run
- Workers
- pmcp.run
- ~$0.01
- ~$0.03
- $0.00
- TBD
- Free tier covers most
- Free tier covers most
- Free plan
- Coming soon
- Medium Volume (1M requests/month)
- Platform
- Monthly Cost
- Notes
- Lambda (ARM64)
- Cloud Run
- ~$3.50
- ~$5.00
- $0.20 requests + $0.17 compute + $1 API GW
- Higher per-request compute
- Platform
- Monthly Cost
- Notes
- Workers
- pmcp.run
- $5.00
- TBD
- Flat rate (paid plan)
- Coming soon
- High Volume (100M requests/month)
- Platform
- Monthly Cost

```text
                  DATA TRANSFER COSTS
AWS (Lambda):
  First 10TB:   $0.09/GB
  Next 40TB:    $0.085/GB
  Over 150TB:   $0.07/GB
GCP (Cloud Run):
  First 1TB:    Free
  1-10TB:       $0.12/GB
  Over 10TB:    $0.11/GB
Cloudflare:
  All egress:   Free (included in plan)
```

AWS cost:    10GB × $0.09 = $0.90/month GCP cost:    Free (under 1TB) Cloudflare:  Free

```yaml
  Ingestion:  $0.50/GB
  Storage:    $0.03/GB/month
  Queries:    $0.005/GB scanned
```


### Cloud Logging (GCP):

```text
  First 50GB: Free
  Over 50GB:  $0.50/GB
```


### Cloudflare:

```yaml
  Workers logs: Included
  Analytics:    Included in paid plan
```


### Cost optimization:

```rust
// Bad: Verbose logging in production
tracing::info!("Processing request: {:?}", full_request_body);

// Good: Log only essential data
tracing::info!(
    request_id = %request.id,
    tool = %request.method,
    "MCP request"
);
```

```text
                  DATABASE COSTS
RDS PostgreSQL (db.t3.micro):
  Instance:   ~$15/month
  Storage:    $0.115/GB/month
DynamoDB (on-demand):
  Reads:      $0.25 per million
  Writes:     $1.25 per million
Cloud SQL (db-f1-micro):
  Instance:   ~$9/month
  Storage:    $0.17/GB/month
Cloudflare D1:
  Reads:      $0.75 per million rows
  Writes:     $1.00 per million rows
  Storage:    First 5GB free
```

- To eliminate cold starts, you pay for always-on capacity:

```text
  $0.000004167 per GB-second (on top of regular pricing)
```


### Example: 10 provisioned instances, 128MB

```yaml
  Monthly: 10 × 0.128GB × 2,628,000s × $0.000004167 = $14.02
```


### Cloud Run Min Instances:

```text
  Same as regular instance pricing when idle
  1 min instance (1 vCPU, 512MB): ~$66/month
```

```text
            MEMORY VS COST OPTIMIZATION
128MB:  Slowest, cheapest per GB-second, often most expensive
256MB:  2× CPU, often 2× faster, same total cost
512MB:  4× CPU, diminishing returns for IO-bound work
1GB+:   For CPU-heavy processing only
Optimal for Rust MCP servers: 256-512MB
(Fast enough for instant response, not paying for unused CPU)
```


### Benchmarking approach:

```bash
# Test different memory configurations
for mem in 128 256 512 1024; do
  echo "Testing ${mem}MB..."
  # Update Lambda config and run load test
  aws lambda update-function-configuration \
    --function-name my-mcp-server \
    --memory-size $mem

  # Run benchmark
  hey -n 1000 -c 10 https://api.example.com/mcp

  # Calculate cost per request
```

```toml
# .pmcp/deploy.toml
[lambda]
architecture = "arm64"  # Default in PMCP

# Building for ARM64
# cargo pmcp deploy automatically uses cargo-lambda with ARM64 target
```

- Instead of many small requests, batch operations:

```rust
// Expensive: 10 separate tool calls
for item in items {
    client.call_tool("process_item", json!({ "item": item })).await?;
}

// Cheaper: 1 batched call
```

```rust
use moka::future::Cache;

// In-memory cache for Lambda warm instances
static CACHE: Lazy<Cache<String, Vec<User>>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(Duration::from_secs(300))
        .build()
});

async fn get_users(department: &str) -> Result<Vec<User>> {
    if let Some(users) = CACHE.get(department).await {
        return Ok(users);
    }

    let users = db.query_users(department).await?;
    CACHE.insert(department.to_string(), users.clone()).await;
    Ok(users)
}
```

- Don't pay for hung requests:

```toml
# .pmcp/deploy.toml
[lambda]
timeout_seconds = 30  # Default: 30s, max: 900s

[cloud_run]
timeout_seconds = 60  # Default: 60s, max: 3600s
```

```bash
# View Lambda costs for last 30 days
aws ce get-cost-and-usage \
  --time-period Start=2024-01-01,End=2024-01-31 \
  --granularity MONTHLY \
  --metrics BlendedCost \
  --filter '{"Dimensions":{"Key":"SERVICE","Values":["AWS Lambda"]}}'
```

```bash
gcloud billing budgets create \
  --billing-account=ACCOUNT_ID \
  --display-name="MCP Server Budget" \
  --budget-amount=100USD \
  --threshold-rules=percent=80,percent=100
```


### Setting Up Alerts

```rust
# AWS CloudWatch alarm for unexpected costs
Resources:
  CostAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: MCPServerCostAlert
      MetricName: EstimatedCharges
      Namespace: AWS/Billing
      Statistic: Maximum
      Period: 86400
      EvaluationPeriods: 1
      Threshold: 50
      ComparisonOperator: GreaterThanThreshold
```

(table/list, cells in reading order)
- Total Cost of Ownership (TCO)
- Beyond cloud bills, consider:
- Factor
- Lambda
- Cloud Run
- Workers
- Development time
- Low
- Medium
- Medium (WASM)
- Operational overhead
- Very low
- Low
- Very low
- Factor
- Lambda
- Cloud Run
- Workers
- Debugging complexity
- Vendor lock-in
- Medium
- Medium
- Low
- Low
- Medium
- High
- Team expertise needed
- AWS
- Docker/GCP
- WASM
- Summary

```text
                  COST DECISION MATRIX
Low volume (<100K/month):
  → Workers free tier or Lambda free tier
  → Cost: $0-5/month
Medium volume (100K-10M/month):
  → Lambda (ARM64) or Workers paid
  → Cost: $5-50/month
High volume (>10M/month):
  → Workers (best per-request cost)
  → Or Lambda with reserved concurrency
  → Cost: $50+/month, optimize aggressively
Need zero cold starts:
  → Cloud Run with min instances
  → Or Lambda with provisioned concurrency
  → Cost: $50-100+/month baseline
Rust advantage across all platforms:
  → 50-80% lower compute costs vs Python/Node
  → Faster execution = better user experience
  → Lower memory = cheaper instances
```


### Key takeaways:

1.


### Security Boundaries

Remote MCP deployments introduce security considerations that don't exist with local servers.


### The Security Landscape

When you deploy an MCP server remotely, you're exposing functionality over the internet:

```text
                  SECURITY BOUNDARIES
Internet (Untrusted)
               PUBLIC BOUNDARY
   - TLS termination
   - Authentication (OAuth, API keys)
   - Rate limiting
   - Request validation
               MCP SERVER
   - Tool authorization
   - Input validation
   - Output sanitization
   - Audit logging
               PRIVATE BOUNDARY
   - VPC isolation
   - Database credentials
   - Internal API access
   - Secrets management
```

```bash
# Initialize with OAuth support
cargo pmcp deploy init --target aws-lambda --oauth-provider cognito

# This creates:
# - Cognito User Pool for user management
# - Lambda authorizer for token validation
# - OAuth endpoints (/oauth2/authorize, /oauth2/token)
```


### OAuth flow:

```text
                  OAUTH 2.0 FLOW
1. Client Registration (one-time):
   POST /oauth2/register
   → Returns client_id, client_secret
2. Authorization Request:
   GET /oauth2/authorize?client_id=...&redirect_uri=...
   → User logs in, grants permission
   → Redirects with authorization code
3. Token Exchange:
   POST /oauth2/token
   grant_type=authorization_code&code=...
   → Returns access_token, refresh_token
4. API Access:
   POST /mcp
   Authorization: Bearer <access_token>
   → MCP request processed
```


### Token Validation

The Lambda authorizer validates tokens before requests reach your MCP server:

```rust
// Simplified authorizer logic (generated by cargo pmcp deploy)
async fn validate_token(token: &str) -> Result<AuthContext> {
    // Decode JWT header to get key ID
    let header = decode_header(token)?;
    let kid = header.kid.ok_or(AuthError::MissingKeyId)?;

    // Fetch Cognito public keys (cached)
    let jwks = get_cognito_jwks().await?;
    let key = jwks.find(&kid).ok_or(AuthError::UnknownKey)?;

    // Verify signature and claims
    let claims = decode::<Claims>(token, &key, &validation)?;

    // Check expiration
    if claims.exp < current_timestamp() {
        return Err(AuthError::TokenExpired);
    }

    // Check issuer
    if claims.iss != expected_issuer() {
        return Err(AuthError::InvalidIssuer);
    }

    Ok(AuthContext {
        user_id: claims.sub,
        scopes: claims.scope.split(' ').collect(),
        email: claims.email,
    })
}
```

```rust
// In your MCP server
use pmcp::middleware::ApiKeyAuth;

let server = Server::builder()
    .name("my-server")
    .middleware(ApiKeyAuth::new(|api_key| {
        // Validate API key against your store
        validate_api_key(api_key).await
    }))
    .tool("query_data", ...)
    .build()?;
```


### API keys should be:

Generated with sufficient entropy (256+ bits) Stored hashed (bcrypt/argon2) Transmitted only over HTTPS Rotatable without downtime


### Authorization

Authentication tells you who is making a request.

```rust
#[derive(Debug, Clone)]
enum Scope {
    Read,      // Read-only access to data
    Write,     // Modify data
    Admin,     // Administrative operations
}

async fn check_authorization(
    auth_context: &AuthContext,
    tool: &str,
    required_scope: Scope,
) -> Result<()> {
    // Check if user has required scope
    let has_scope = match required_scope {
        Scope::Read => auth_context.scopes.contains(&"mcp:read"),
        Scope::Write => auth_context.scopes.contains(&"mcp:write"),
        Scope::Admin => auth_context.scopes.contains(&"mcp:admin"),
    };

    if !has_scope {
        return Err(AuthError::InsufficientPermissions {
            user: auth_context.user_id.clone(),
            tool: tool.to_string(),
            required: format!("{:?}", required_scope),
        });
    }

    // Log access for audit
    tracing::info!(
        user = %auth_context.user_id,
        tool = %tool,
        scope = ?required_scope,
        "Authorization granted"
    );

    Ok(())
}
```

- Annotate tools with required permissions:

```rust
use pmcp::server::TypedTool;

// Read-only tool - anyone with 'read' scope can use
let read_tool = TypedTool::new("list_items", |input: ListInput| async move {
    // Tool implementation
})
.read_only()  // Hint for clients
.with_required_scope("mcp:read");  // Actual enforcement

// Destructive tool - requires 'write' scope
let write_tool = TypedTool::new("delete_item", |input: DeleteInput| async move {
    // Tool implementation
})
.destructive()  // Hint: this modifies data
.with_required_scope("mcp:write");

// Admin tool - requires elevated permissions
let admin_tool = TypedTool::new("purge_all", |input: PurgeInput| async move {
    // Tool implementation
})
.destructive()
.with_required_scope("mcp:admin");
```

```text
                  AWS VPC
               PUBLIC SUBNET
NAT Gateway                API Gateway
(for egress)        (HTTPS termination)
               PRIVATE SUBNET
             Lambda Function
          (your MCP server)
                 RDS PostgreSQL
        (private, no public access)
```


### CDK configuration:

```yaml
// In deploy/lib/stack.ts
const vpc = new ec2.Vpc(this, 'Vpc', {
  maxAzs: 2,
  natGateways: 1,  // For Lambda to reach internet
});
```


### const mcpFunction = new lambda.Function(this, 'McpFunction', {

```yaml
  // ... other config
  vpc,
  vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
  securityGroups: [mcpSecurityGroup],
});

// Database in same VPC
const database = new rds.DatabaseInstance(this, 'Database', {
  vpc,
  vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_ISOLATED },
  publiclyAccessible: false,  // No internet access
});

// Allow Lambda to connect to database
```

```yaml
// Lambda security group - outbound only to database and internet
const lambdaSg = new ec2.SecurityGroup(this, 'LambdaSg', {
  vpc,
  description: 'MCP Lambda security group',
  allowAllOutbound: false,
});

// Allow HTTPS to internet (for external APIs)
```

lambdaSg.addEgressRule(ec2.Peer.anyIpv4(), ec2.Port.tcp(443), 'HTTPS');

```yaml
// Allow connection to database
lambdaSg.addEgressRule(
  databaseSg,
  ec2.Port.tcp(5432),
  'PostgreSQL'
);

// Database security group - inbound only from Lambda
const databaseSg = new ec2.SecurityGroup(this, 'DatabaseSg', {
  vpc,
  description: 'Database security group',
  allowAllOutbound: false,
});
```


### databaseSg.addIngressRule(

```text
  lambdaSg,
  ec2.Port.tcp(5432),
  'From Lambda'
);
```


### Secrets Management

Never hardcode secrets in your code or deployment configuration.

```rust
use aws_sdk_secretsmanager::Client;

async fn get_database_credentials() -> Result<DbCredentials> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let response = client
        .get_secret_value()
        .secret_id("mcp-server/database")
        .send()
        .await?;

    let secret_string = response.secret_string().ok_or(Error::NoSecret)?;
    let credentials: DbCredentials = serde_json::from_str(secret_string)?;

    Ok(credentials)
}

// Use in Lambda initialization (cached across warm invocations)
static DB_CREDENTIALS: OnceCell<DbCredentials> = OnceCell::new();

async fn get_credentials() -> &'static DbCredentials {
    DB_CREDENTIALS.get_or_init(|| async {
        get_database_credentials().await.expect("Failed to get credentials")
    }).await
}
```

```toml
# .pmcp/deploy.toml
[lambda.environment]
RUST_LOG = "info"
DATABASE_HOST = "db.internal.example.com"  # Not a secret
# DATABASE_PASSWORD = "..."  # NEVER DO THIS

# Instead, use:
DATABASE_SECRET_ARN = "arn:aws:secretsmanager:us-east-1:123456789:secret:db-creds"
```


### Cloudflare Workers Secrets

```rust
# Set secrets via wrangler
wrangler secret put DATABASE_PASSWORD
# Enter secret value interactively (not stored in shell history)

# Access in code
async fn handler(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let db_password = env.secret("DATABASE_PASSWORD")?.to_string();
    // ...
}
```


### Input Validation

All input from MCP clients must be validated.

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct QueryInput {
    /// Table name (alphanumeric only)
    #[schemars(regex(pattern = r"^[a-zA-Z][a-zA-Z0-9_]*$"))]
    table: String,

    /// Maximum rows to return (1-1000)
    #[serde(default = "default_limit")]
    #[schemars(range(min = 1, max = 1000))]
    limit: u32,

    /// Filter conditions
    #[serde(default)]
    filters: Vec<Filter>,
}

fn default_limit() -> u32 { 100 }

#[derive(Deserialize, JsonSchema)]
pub struct Filter {
    /// Column name (alphanumeric only)
    #[schemars(regex(pattern = r"^[a-zA-Z][a-zA-Z0-9_]*$"))]
    column: String,

    /// Comparison operator
    operator: Operator,

    /// Value to compare (sanitized)
    value: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    Eq,      // =
    Ne,      // !=

    Lt,      // <
    Gt,      // >
    Like,    // LIKE (with escaping)
    In,      // IN (parameterized)
}
```

- Always use parameterized queries:

```rust
// DANGEROUS: String interpolation
let query = format!(
    "SELECT * FROM {} WHERE name = '{}'",
    input.table, input.name  // SQL INJECTION VULNERABILITY
);

// SAFE: Parameterized query with allowlist
async fn query_table(input: QueryInput) -> Result<Vec<Row>> {
    // Allowlist tables
    const ALLOWED_TABLES: &[&str] = &["users", "orders", "products"];
    if !ALLOWED_TABLES.contains(&input.table.as_str()) {
        return Err(Error::InvalidTable(input.table));
    }

    // Build parameterized query
    let mut query = sqlx::query_as::<_, Row>(
        &format!("SELECT * FROM {} WHERE 1=1", input.table)  // Table name validated above
    );

    // Add parameterized filters
    for filter in &input.filters {
        // Column name validated by regex in schema
        // Value is parameterized
        query = match filter.operator {
            Operator::Eq => query.bind(&filter.value),
            Operator::Like => {
                // Escape LIKE wildcards
                let escaped = escape_like(&filter.value.to_string());
                query.bind(format!("%{}%", escaped))
            }
            // ...
        };
    }

    query.fetch_all(&pool).await
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")

     .replace('%', "\\%")
     .replace('_', "\\_")
}
```

```rust
use tracing::{info, warn};

#[derive(Debug, Serialize)]
struct AuditEvent {
    timestamp: DateTime<Utc>,
    event_type: &'static str,
    user_id: String,
    tool: String,
    input_hash: String,  // Hash of input, not raw data
    result: AuditResult,
    duration_ms: u64,
    source_ip: Option<String>,
}

#[derive(Debug, Serialize)]
enum AuditResult {
    Success,
    AuthFailure { reason: String },
    ValidationError { field: String },
    ExecutionError { error_type: String },
}

async fn audit_tool_call(
    auth: &AuthContext,
    tool: &str,
    input: &serde_json::Value,
    result: &Result<serde_json::Value, Error>,
    duration: Duration,
) {
    let event = AuditEvent {
        timestamp: Utc::now(),
        event_type: "tool_call",
        user_id: auth.user_id.clone(),
        tool: tool.to_string(),
        input_hash: sha256_hex(&input.to_string()),
        result: match result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::ExecutionError {
                error_type: e.to_string(),

            },
        },
        duration_ms: duration.as_millis() as u64,
        source_ip: auth.source_ip.clone(),
    };

    // Log structured audit event
    info!(
        audit = ?event,
        "Tool call audit"
    );

    // Optionally send to dedicated audit service
    if let Err(e) = send_to_audit_service(&event).await {
        warn!(error = %e, "Failed to send audit event");
    }
}
```

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Per-user rate limiter: 100 requests per minute
static RATE_LIMITER: Lazy<RateLimiter<String, _, _>> = Lazy::new(|| {
    RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(100).unwrap()))
});

async fn check_rate_limit(user_id: &str) -> Result<()> {
    match RATE_LIMITER.check_key(&user_id.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::RateLimitExceeded {
            retry_after: Duration::from_secs(60),
        }),
    }
}

// In your handler
async fn handle_mcp_request(auth: AuthContext, request: Request) -> Response {
    // Check rate limit first
    if let Err(e) = check_rate_limit(&auth.user_id).await {
        return Response::json(&json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "Rate limit exceeded",
                "data": { "retry_after": e.retry_after.as_secs() }
            }
        })).status(429);
    }

    // Process request...
}
```


### Network Security

TLS enforced (HTTPS only) Database in private subnet (no public access) Security groups restrict traffic to necessary ports VPC endpoints for AWS services (avoid internet)


### Secrets Management

No secrets in code or config files Secrets stored in Secrets Manager/Vault Secrets rotated regularly Least-privilege IAM roles


### Input Validation

All input validated against schema SQL injection prevented (parameterized queries) Table/column names allowlisted File paths validated (no traversal)


### Monitoring & Response

Audit logging enabled Rate limiting configured Alerts for suspicious activity Incident response plan documented

- Security for remote MCP deployments requires defense in depth:

```text
                  SECURITY LAYERS
Layer 1: Network Perimeter
  - TLS/HTTPS only
  - DDoS protection (CloudFlare, AWS Shield)
  - Rate limiting
Layer 2: Authentication
  - OAuth 2.0 / API keys
  - Token validation
  - Session management
Layer 3: Authorization
  - Scope-based access control
  - Tool-level permissions
  - Data-level filtering
Layer 4: Input Validation
  - Schema validation
  - Parameterized queries
  - Output sanitization
Layer 5: Infrastructure
  - VPC isolation
  - Secrets management
  - Least-privilege IAM
Layer 6: Detection & Response
  - Audit logging
  - Anomaly detection
  - Incident response
```
