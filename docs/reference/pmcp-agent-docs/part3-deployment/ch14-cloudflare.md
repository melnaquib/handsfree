---
chapter: 14
title: "Cloudflare Workers (WASM)"
part: "Part III: Deployment"
use_when: "Deploying to Cloudflare Workers: wrangler, bindings, WASM limits, what does not work."
suggested_prerequisites: [ch07]
source_kb: 56
extracted_kb: 35
code_blocks: 65
quality: auto-extracted
---

# Ch 14: Cloudflare Workers (WASM)

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Cloudflare Workers Deployment

Cloudflare Workers runs your MCP server as WebAssembly (WASM) on Cloudflare's global edge network.

```text
               CLOUDFLARE EDGE NETWORK
                    Your MCP Server
               (compiled to WebAssembly)
    Tokyo            London          New York
    (5ms)            (5ms)            (5ms)
  Users in         Users in         Users in
  Asia             Europe           Americas
Benefits:
• 300+ edge locations worldwide
• Sub-millisecond cold starts (V8 isolates)
• Unlimited free egress bandwidth
• Built-in DDoS protection
• Integrated storage (KV, D1, R2)
```

(table/list, cells in reading order)
- When to Choose Workers
- Use Case
- Global low-latency
- Stateless API
- Database access
- Long computations
- File system access
- Complex dependencies


### Lambda

[BAD] Regional only [GOOD] Good [GOOD] RDS/DynamoDB [GOOD] 15min limit [GOOD] /tmp available [GOOD] Full native


### Prerequisites

```text
# Node.js (for wrangler)
```


### node --version  # 18+ recommended

```bash
# Wrangler CLI
npm install -g wrangler

# Login to Cloudflare
wrangler login

# Rust with WASM target
rustup target add wasm32-unknown-unknown

# wasm-pack for building
cargo install wasm-pack
```

- Workers uses V8 isolates instead of containers:

```text
                  WORKERS EXECUTION MODEL
TRADITIONAL CONTAINER                    V8 ISOLATE
      Container                               V8 Engine
       OS Layer                               Isolate A
                                              (your WASM)
       Runtime
                                              Isolate B
       Your Code                              (other user)
                                              Isolate C
                                              (other user)
Startup: 50-500ms
Memory: Dedicated
                                        Startup: <1ms
                                        Memory: Shared engine
```


### V8 isolates are lightweight sandboxes that:

Share the V8 JavaScript/WASM engine Start in microseconds (not milliseconds) Provide strong security isolation Have 128MB memory limit per request

```bash
# From your MCP server project
cargo pmcp deploy init --target cloudflare-workers
```

```text
deploy.toml              # Deployment configuration
workers/
    wrangler.toml        # Wrangler configuration
    src/
        lib.rs           # Worker entry point (generated)
    Cargo.toml           # WASM-specific dependencies
```

```toml
[target]
target_type = "cloudflare-workers"

[server]
name = "my-mcp-server"

[cloudflare]
account_id = "your-account-id"  # From Cloudflare dashboard
zone_id = "your-zone-id"        # Optional: for custom domains

[workers]
name = "my-mcp-server"
compatibility_date = "2024-01-01"
main = "build/worker/shim.mjs"

# Environment variables (non-secret)
[workers.vars]
RUST_LOG = "info"
ENVIRONMENT = "production"

# Bindings to Cloudflare services
[workers.kv_namespaces]
# KV_CACHE = "your-kv-namespace-id"

[workers.d1_databases]
# DB = "your-d1-database-id"

[workers.r2_buckets]
# STORAGE = "your-r2-bucket-name"
```


### Edit the generated  wrangler.toml :

```rust
name = "my-mcp-server"
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[build]
command = "cargo pmcp deploy build --target cloudflare-workers"

# Route configuration
[[routes]]
pattern = "mcp.example.com/*"
zone_id = "your-zone-id"

# Or use workers.dev subdomain (default)
# workers_dev = true
```


### Step 3: Build and Deploy

```bash
# Build WASM and deploy
cargo pmcp deploy --target cloudflare-workers

# Or step by step:
cargo pmcp deploy build --target cloudflare-workers
wrangler deploy
```


### First deployment creates:

Worker script on Cloudflare's network workers.dev subdomain (e.g.,  my-mcp-server.username.workers.dev ) KV/D1/R2 bindings if configured


### Step 4: Verify Deployment

```bash
# Get deployment URL
cargo pmcp deploy outputs --target cloudflare-workers

# Output:
# WorkerUrl: https://my-mcp-server.username.workers.dev
# McpEndpoint: https://my-mcp-server.username.workers.dev/mcp

# Test the endpoint
curl -X POST https://my-mcp-server.username.workers.dev/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":
```

```rust
// .pmcp/workers/src/lib.rs
use worker::*;
use pmcp::server::Server;
use pmcp::transport::WorkersTransport;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Initialize router
    let router = Router::new();

    router
        // Health check
        .get("/health", |_, _| Response::ok("OK"))

        // MCP endpoint
        .post_async("/mcp", |mut req, ctx| async move {
            let body = req.text().await?;

            // Build MCP server (stateless per request)
            let server = build_mcp_server(&ctx.env)?;

            // Process MCP request
            let response = server.handle_request(&body).await?;

            Response::from_json(&response)
        })

        // Run router
        .run(req, env)
        .await
}

fn build_mcp_server(env: &Env) -> Result<Server> {
    Server::builder()
        .name("my-mcp-server")
        .version("1.0.0")
        .tool("query", TypedTool::new("query", |input: QueryInput| async move {
            // Tool implementation
            Ok(json!({"result": "data"}))

        }))
        .build()
        .map_err(|e| Error::from(e.to_string()))
}
```


### Workers Bindings

Cloudflare provides integrated storage services accessible via bindings.

```rust
use worker::*;

async fn cache_handler(env: &Env, key: &str) -> Result<Option<String>> {
    // Get KV namespace from binding
    let kv = env.kv("CACHE")?;

    // Read value
    let value = kv.get(key).text().await?;

    Ok(value)
}

async fn cache_set(env: &Env, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
    let kv = env.kv("CACHE")?;

    // Write with expiration
    kv.put(key, value)?
        .expiration_ttl(ttl_seconds)
        .execute()
        .await?;

    Ok(())
}
```


### Configure in  wrangler.toml :

```toml
[[kv_namespaces]]
binding = "CACHE"
id = "your-namespace-id"
# preview_id = "preview-namespace-id"  # For local dev
```

```rust
use worker::*;

async fn query_users(env: &Env, department: &str) -> Result<Vec<User>> {
    let db = env.d1("DB")?;

    let statement = db.prepare("SELECT * FROM users WHERE department = ?1");
    let results = statement
        .bind(&[department.into()])?
        .all()
        .await?;

    let users: Vec<User> = results.results()?;
    Ok(users)
}

async fn insert_user(env: &Env, user: &User) -> Result<()> {
    let db = env.d1("DB")?;

    db.prepare("INSERT INTO users (name, email, department) VALUES (?1, ?2, ?3)")
        .bind(&[user.name.into(), user.email.into(), user.department.into()])?
        .run()
        .await?;

    Ok(())
}
```


### Configure in  wrangler.toml :

```toml
[[d1_databases]]
binding = "DB"
database_name = "my-database"
database_id = "your-database-id"
```


### Create and migrate database:

```bash
# Create database
wrangler d1 create my-database

# Run migrations
wrangler d1 migrations apply my-database

# Query interactively
wrangler d1 execute my-database --command "SELECT * FROM users"
```

```rust
use worker::*;

async fn get_file(env: &Env, key: &str) -> Result<Option<Vec<u8>>> {
    let bucket = env.bucket("STORAGE")?;

    match bucket.get(key).execute().await? {
        Some(object) => {
            let bytes = object.body().unwrap().bytes().await?;
            Ok(Some(bytes))
        }
        None => Ok(None),
    }
}

async fn put_file(env: &Env, key: &str, data: Vec<u8>) -> Result<()> {
    let bucket = env.bucket("STORAGE")?;

    bucket.put(key, data).execute().await?;

    Ok(())
}
```


### Configure in  wrangler.toml :

```toml
[[r2_buckets]]
binding = "STORAGE"
bucket_name = "my-bucket"
```

```rust
use worker::*;

async fn query_external_db(env: &Env) -> Result<Vec<Record>> {
    // Hyperdrive provides a connection string
    let hyperdrive = env.hyperdrive("EXTERNAL_DB")?;
    let connection_string = hyperdrive.connection_string();

    // Use with your preferred database client
    // Note: Must be WASM-compatible (e.g., using HTTP-based drivers)

    Ok(records)
}
```


### Configure in  wrangler.toml :

```toml
[[hyperdrive]]
binding = "EXTERNAL_DB"
id = "your-hyperdrive-id"
```

```bash
# Set a secret (entered interactively, not in shell history)
wrangler secret put DATABASE_PASSWORD

# List secrets
wrangler secret list

# Delete a secret
wrangler secret delete DATABASE_PASSWORD
```


### Access in your worker:

```rust
async fn handler(env: &Env) -> Result<Response> {
    let api_key = env.secret("API_KEY")?.to_string();
    // Use api_key...
    Ok(Response::ok("OK"))
}
```

```toml
# wrangler.toml

# Option 1: Route pattern (requires zone in Cloudflare)
[[routes]]
pattern = "mcp.example.com/*"
zone_id = "your-zone-id"

# Option 2: Custom domain (simpler)
[[routes]]
pattern = "mcp.example.com"
custom_domain = true
```

Then add a DNS record in Cloudflare dashboard pointing to your worker.

```toml
# wrangler.toml
name = "my-mcp-server"
main = "build/worker/shim.mjs"

# Default (development)
[vars]
ENVIRONMENT = "development"

# Staging environment
[env.staging]
name = "my-mcp-server-staging"
[env.staging.vars]
ENVIRONMENT = "staging"

# Production environment
[env.production]
name = "my-mcp-server-prod"
[[env.production.routes]]
pattern = "mcp.example.com/*"
zone_id = "your-zone-id"
[env.production.vars]
ENVIRONMENT = "production"
```


### Deploy to specific environment:

```bash
# Deploy to staging
wrangler deploy --env staging

# Deploy to production
wrangler deploy --env production
```

```bash
# Tail logs in real-time
wrangler tail

# Filter by status
wrangler tail --status error

# Filter by search term
wrangler tail --search "tool_call"
```

```rust
use worker::console_log;

async fn handler(req: Request) -> Result<Response> {
    console_log!("Request received: {} {}", req.method(), req.path());

    let start = Date::now();

    // Process request...

    let duration = Date::now().as_millis() - start.as_millis();
    console_log!("Request completed in {}ms", duration);

    Ok(response)
}
```

(table/list, cells in reading order)
- Analytics
- View metrics in Cloudflare dashboard:
- Request count Error rate CPU time Response time percentiles
- Performance Optimization
- Bundle Size
- Keep WASM bundles small for faster cold starts:

```toml
# Cargo.toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true
codegen-units = 1
panic = "abort"
strip = true
```


### [profile.release.package."*"]

```toml
opt-level = "z"
```

(table/list, cells in reading order)
- Typical sizes:
- Minimal MCP server: ~500KB WASM With dependencies: 1-3MB WASM
- CPU Time Limits
- Workers has CPU time limits:
- Plan
- Free
- Paid
- CPU Time Limit
- 10ms
- 50ms

Important: This is CPU time, not wall-clock time.


### Optimize CPU-intensive operations:

```rust
// Bad: CPU-intensive in hot path
async fn handler(input: Input) -> Result<Response> {
    let result = expensive_computation(&input.data);  // Uses CPU time
    Ok(Response::from_json(&result)?)
}

// Good: Offload to Durable Objects or external service
async fn handler(input: Input) -> Result<Response> {
    // Light processing in Worker
    let key = hash(&input.data);

    // Heavy computation cached
    let cached = env.kv("CACHE")?.get(&key).text().await?;
    if let Some(result) = cached {
        return Ok(Response::from_json(&result)?);
    }

    // Compute once, cache result
    let result = expensive_computation(&input.data);
    env.kv("CACHE")?.put(&key, &result)?.execute().await?;

    Ok(Response::from_json(&result)?)
}
```

(table/list, cells in reading order)
- Limitations
- Workers has specific limitations to be aware of:
- Limitation
- Details
- No filesystem
- No  /tmp , no file I/O
- CPU time
- Memory
- 10-50ms per request
- 128MB per isolate
- Limitation
- Details
- Request size
- 100MB max
- Subrequest limit
- 50 subrequests per request (1000 on paid)
- No raw sockets
- HTTP/HTTPS only via fetch()
- What Works

HTTP client requests via  fetch() KV, D1, R2 storage Durable Objects for state WebSocket connections Crypto APIs


### What Doesn't Work

Raw TCP/UDP sockets Native database drivers (use Hyperdrive or HTTP APIs) File system operations Some Rust crates (see WASM Considerations chapter)

```json
{
  "mcpServers": {
    "my-workers-server": {
      "transport": "streamable-http",
      "url": "https://my-mcp-server.username.workers.dev/mcp"
    }
  }
}
```


### With API key authentication:

```json
{
  "mcpServers": {
    "my-workers-server": {
      "transport": "streamable-http",
      "url": "https://my-mcp-server.username.workers.dev/mcp",
      "headers": {
        "Authorization": "Bearer ${MCP_API_KEY}"
      }
    }
  }
}
```

```bash
# Start local dev server
wrangler dev

# With local KV/D1/R2 simulation
wrangler dev --local

# Specify port
wrangler dev --port 8787
```


### Test MCP locally:

```bash
curl -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":1}'
```

Global edge network - 300+ locations, minimal latency Sub-millisecond cold starts - V8 isolates, not containers Zero egress fees - Unlimited outbound bandwidth included Integrated storage - KV, D1, R2 with simple bindings Simple deployment -  wrangler deploy  handles everything


### Key commands:

```bash
cargo pmcp deploy init --target cloudflare-workers  # Initialize
cargo pmcp deploy --target cloudflare-workers       # Deploy
wrangler tail                                       # View logs
wrangler dev                                        # Local development
wrangler secret put KEY                             # Set secrets
```


### Best suited for:

Global APIs with low-latency requirements Stateless operations with caching MCP servers using D1/KV for data High-volume, cost-sensitive deployments


### Consider alternatives when:

You need raw database drivers (use Lambda) Long-running computations >50ms CPU (use Lambda/Cloud Run) Complex native dependencies (use Lambda/Cloud Run)


### WASM Considerations for Rust MCP Servers

WebAssembly (WASM) enables running Rust code on Cloudflare Workers' edge network, but it comes with specific constraints and patterns you need to understand.

```text
                  Traditional Container
                     Operating System
      Process 1       Process 2       Process 3
      (Your App)      (Runtime)       (Deps)
   Full syscall access, filesystem, threads
Startup: 50-500ms   Memory: 128MB-4GB   Isolation: Process
                  V8 Isolate (Workers)
                     V8 JavaScript Engine
      Isolate 1       Isolate 2       Isolate 3
      (WASM)          (WASM)          (WASM)
   Sandboxed, no syscalls, Web APIs only
Startup: <5ms   Memory: 128MB max   Isolation: V8 Sandbox
```

(table/list, cells in reading order)
- What WASM Provides
- What WASM Cannot Do
- Capability
- Available
- Notes
- CPU compute
- Memory allocation
- Async/await
- HTTP fetch
- Time/Date
- Crypto
- JSON parsing
- Yes
- Yes
- Yes
- Yes
- Yes
- Yes
- Yes
- Full Rust performance
- Up to 128MB
- Via JavaScript promises
- Via Workers Fetch API
- Via JavaScript Date
- Via Web Crypto API
- Native Rust serde
- Capability
- Available
- Alternative
- Filesystem
- Raw sockets
- Threads
- System calls
- No
- No
- No
- No
- Workers KV, R2
- HTTP via fetch
- Single-threaded async

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm tooling
cargo install worker-build
cargo install wasm-pack
cargo install wasm-opt  # For optimization
```

```text
Cargo.toml
wrangler.toml
src/
    lib.rs           # Worker entry point
    server.rs        # MCP server logic
    tools/           # Tool implementations
        mod.rs
        database.rs
    bindings.rs      # Workers API bindings
build.rs             # Build script for WASM
tests/
    wasm.rs          # WASM-specific tests
```


### Cargo.toml Configuration

```toml
[package]
name = "my-mcp-worker"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Workers runtime
worker = "0.4"
worker-macros = "0.4"

# MCP SDK (WASM-compatible)
pmcp-sdk = { version = "0.1", features = ["wasm"] }

# Async runtime (WASM-compatible)
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# WASM-compatible utilities
getrandom = { version = "0.2", features = ["js"] }
chrono = { version = "0.4", features = ["wasmbind"] }

# Console logging for WASM
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
# Optimize for size (important for cold starts)
opt-level = "s"

lto = true
codegen-units = 1
panic = "abort"
```


### [profile.release.package."*"]

```toml
opt-level = "s"
```


### Build Script

```rust
// build.rs
fn main() {
    // Ensure we're building for the correct target
    #[cfg(target_arch = "wasm32")]
    {
        println!("cargo:rerun-if-changed=src/");
    }
}
```

Many Rust crates assume a traditional runtime environment.

```text
                Crate Compatibility Matrix
[GOOD] Pure Rust, no std dependencies
   serde, serde_json, thiserror, anyhow
[GOOD] WASM-aware crates
   getrandom (with js feature), chrono (with wasmbind)
   uuid (with js feature), rand (with getrandom)
[warn]  Async crates (need configuration)
   tokio (NOT compatible), futures (compatible)
   async-std (limited), wasm-bindgen-futures (recommended)
[BAD] System-dependent crates
   tokio (uses mio), std::fs, std::net, std::thread
   ring (uses assembly), openssl, native-tls
```


### Handling tokio Dependencies

Many crates depend on tokio, which doesn't compile to WASM.

```rust
// In Cargo.toml, use feature flags
[features]
default = ["native"]
native = ["tokio/full"]
wasm = ["wasm-bindgen-futures"]
```


### [target.'cfg(not(target_arch = "wasm32"))'.dependencies]

```toml
tokio = { version = "1", features = ["full"] }
```


### [target.'cfg(target_arch = "wasm32")'.dependencies]

```rust
wasm-bindgen-futures = "0.4"

// In your code, use conditional imports
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

#[cfg(target_arch = "wasm32")]
async fn sleep(duration: std::time::Duration) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let window = web_sys::window().unwrap();
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                duration.as_millis() as i32,
            )
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}
```


### Random Number Generation

```rust
// Cargo.toml
[dependencies]
getrandom = { version = "0.2", features = ["js"] }
uuid = { version = "1.0", features = ["v4", "js"] }
rand = { version = "0.8", features = ["getrandom"] }

// Usage - works on both native and WASM
use uuid::Uuid;
use rand::Rng;

fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

fn generate_random_number() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
```


### Date/Time Handling

```rust
// Cargo.toml
[dependencies]
chrono = { version = "0.4", features = ["wasmbind"] }

// For Workers-specific time
use worker::Date;

fn get_current_time() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        Date::now().to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Utc::now().to_rfc3339()
    }
}
```


### Crypto Operations

```rust
// Native crypto won't work - use Web Crypto API
use worker::*;

async fn hash_data(data: &[u8]) -> Result<Vec<u8>> {
    let crypto = Crypto::new();
    let digest = crypto
        .subtle()
        .digest("SHA-256", data)
        .await?;
    Ok(digest.to_vec())
}

async fn generate_hmac(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let crypto = Crypto::new();

    // Import the key
    let crypto_key = crypto
        .subtle()
        .import_key_raw(
            key,
            "HMAC",
            &HmacImportParams::new("SHA-256"),
            false,
            &["sign"],
        )
        .await?;

    // Sign the data
    let signature = crypto
        .subtle()
        .sign("HMAC", &crypto_key, data)
        .await?;

    Ok(signature.to_vec())
}
```

```text
                JavaScript Event Loop
Request Arrives
    WASM           ▶   JS Promise      ▶   Event Loop
    Code                Queue              (V8)
Synchronous          Async I/O            Microtasks
computation          (fetch, KV)          scheduled
```


### Spawning Async Tasks

```rust
use worker::*;
use futures::future::join_all;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Parallel async operations (no tokio::spawn needed!)
    let results = join_all(vec![
        fetch_from_kv(&env, "key1"),
        fetch_from_kv(&env, "key2"),
        fetch_from_kv(&env, "key3"),
    ])
    .await;

    // Process results
    let combined: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    Response::from_json(&combined)
}

async fn fetch_from_kv(env: &Env, key: &str) -> Result<String> {
    let kv = env.kv("MY_KV")?;
    kv.get(key)
        .text()
        .await?
        .ok_or_else(|| Error::from("Key not found"))
}
```


### Timeouts and Cancellation

```rust
use worker::*;
use futures::future::{select, Either};
use std::time::Duration;

async fn with_timeout<T, F>(future: F, timeout_ms: u64) -> Result<T>
```


### where

```rust
    F: std::future::Future<Output = Result<T>>,
{
    let timeout = create_timeout(timeout_ms);

    match select(Box::pin(future), Box::pin(timeout)).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(Error::from("Operation timed out")),
    }
}

async fn create_timeout(ms: u64) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let promise = js_sys::Promise::new(&mut |resolve, _| {
        // Workers have a built-in scheduler
        let _ = js_sys::Reflect::set(
            &js_sys::global(),
            &"__timeout_resolve".into(),
            &resolve,
        );
    });

    // Use the scheduler API
    let _ = JsFuture::from(promise).await;
}

// Usage in MCP tool
async fn database_query_tool(env: &Env, query: &str) -> Result<String> {
    with_timeout(

        execute_d1_query(env, query),
        5000, // 5 second timeout
    )
    .await
}
```


### Error Handling Patterns

```rust
use worker::*;
use std::fmt;

// Custom error type that works in WASM
#[derive(Debug)]
pub enum McpError {
    InvalidRequest(String),
    DatabaseError(String),
    Timeout,
    Unauthorized,
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            McpError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            McpError::Timeout => write!(f, "Operation timed out"),
            McpError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl From<McpError> for worker::Error {
    fn from(e: McpError) -> Self {
        worker::Error::from(e.to_string())
    }
}

// Result type alias for cleaner code
pub type McpResult<T> = std::result::Result<T, McpError>;
```

```text
                  WASM Linear Memory (128MB Max)
  Stack (grows down)                                    4MB
                     Free Space
  Heap (grows up)                                     ~120MB
  Static Data (strings, constants)                    ~4MB
Note: Memory is NOT freed between requests in the same isolate!
```


### Memory-Efficient Patterns

```rust
use worker::*;

// BAD: Accumulates memory across requests
```


### static mut CACHE: Option<Vec<String>> = None;

```rust
// GOOD: Use Workers KV for caching
async fn cached_fetch(env: &Env, url: &str) -> Result<String> {
    let kv = env.kv("CACHE")?;
    let cache_key = format!("fetch:{}", url);

    // Check cache first
    if let Some(cached) = kv.get(&cache_key).text().await? {
        return Ok(cached);
    }

    // Fetch and cache
    let response = Fetch::Url(url.parse()?).send().await?;
    let body = response.text().await?;

    // Cache for 5 minutes
    kv.put(&cache_key, &body)?
        .expiration_ttl(300)
        .execute()
        .await?;

    Ok(body)
}
```


### Streaming Large Responses

```rust
use worker::*;

// For large responses, use streaming instead of buffering
async fn stream_large_result(env: &Env, query: &str) -> Result<Response> {
    let d1 = env.d1("DB")?;

    // Create a streaming response
    let (mut tx, rx) = futures::channel::mpsc::unbounded();

    // Spawn the query (conceptually - actual implementation varies)
    wasm_bindgen_futures::spawn_local(async move {
        let results = d1.prepare(query).all().await;

        match results {
            Ok(rows) => {
                for row in rows.results::<serde_json::Value>().unwrap_or_default() {
                    let json = serde_json::to_string(&row).unwrap_or_default();
                    let _ = tx.unbounded_send(json);
                }
            }
            Err(e) => {
                let _ = tx.unbounded_send(format!("Error: {}", e));
            }
        }
    });

    // Return streaming response
    let stream = rx.map(|chunk| Ok(chunk.into_bytes()));
    Response::from_stream(stream)
}
```


### Avoiding Memory Leaks

```rust
use worker::*;
use std::cell::RefCell;

// Use RefCell for request-scoped state (dropped after request)
thread_local! {
    static REQUEST_CONTEXT: RefCell<Option<RequestContext>> = RefCell::new(None);
}

struct RequestContext {
    request_id: String,
    start_time: f64,
}

fn init_request_context(request_id: String) {
    REQUEST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(RequestContext {
            request_id,
            start_time: js_sys::Date::now(),
        });
    });
}

fn cleanup_request_context() {
    REQUEST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let request_id = uuid::Uuid::new_v4().to_string();
    init_request_context(request_id);

    let result = handle_request(req, env).await;

    // Always cleanup, even on error

    cleanup_request_context();

    result
}
```

```text
                  Cold Start Impact
Binary Size    Parse Time    Compile Time    Total Cold Start
100KB          ~1ms          ~2ms            ~3ms
500KB          ~3ms          ~8ms            ~11ms
1MB            ~5ms          ~15ms           ~20ms
3MB            ~12ms         ~40ms           ~52ms
5MB+           ~20ms         ~70ms           ~90ms+
Target: <1MB for sub-20ms cold starts
```


### Cargo.toml Optimization

```toml
[profile.release]
# Size optimization
opt-level = "s"        # Optimize for size ('z' for even smaller)
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Don't include panic unwinding code
strip = true           # Strip symbols
```


### [profile.release.package."*"]


### Code-Level Optimizations

```rust
// AVOID: Generic functions create code bloat
fn process_generic<T: Serialize>(item: T) -> String {
    serde_json::to_string(&item).unwrap()
}

// BETTER: Use trait objects for smaller binary
fn process_dynamic(item: &dyn erased_serde::Serialize) -> String {
    serde_json::to_string(item).unwrap()
}

// AVOID: Large match statements with many arms
match tool_name {
    "tool1" => handle_tool1(),
    "tool2" => handle_tool2(),
    // ... 50 more tools
}

// BETTER: Use a lookup table
lazy_static! {
    static ref TOOL_HANDLERS: HashMap<&'static str, fn() -> Result<Value>> = {
        let mut m = HashMap::new();
        m.insert("tool1", handle_tool1 as fn() -> Result<Value>);
        m.insert("tool2", handle_tool2 as fn() -> Result<Value>);
        m
    };
}
```


### Measuring Binary Size

```bash
# Build for release
wrangler build

# Check size
```


### ls -lh build/worker/shim.mjs wasm-opt --print-size build/*.wasm

```bash
# Analyze what's taking space
cargo install twiggy
```

```bash
# Install wrangler
npm install -g wrangler

# Install wasm testing tools
cargo install wasm-pack

# Create test configuration
cat > wrangler.test.toml << 'EOF'
name = "my-mcp-worker-test"
main = "build/worker/shim.mjs"
compatibility_date = "2024-01-01"

[dev]
port = 8787
local_protocol = "http"

[[kv_namespaces]]
binding = "TEST_KV"
id = "test-kv-id"
preview_id = "test-kv-preview"

[[d1_databases]]
binding = "TEST_DB"
database_name = "test-db"
database_id = "local"
```

```rust
// tests/wasm.rs
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_json_parsing() {
    let json = r#"{"name": "test"}"#;
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["name"], "test");
}

#[wasm_bindgen_test]
async fn test_async_operation() {
    use wasm_bindgen_futures::JsFuture;

    // Test that async operations work
    let promise = js_sys::Promise::resolve(&42.into());
    let result = JsFuture::from(promise).await.unwrap();
    assert_eq!(result, 42);
}

#[wasm_bindgen_test]
fn test_uuid_generation() {
    // Ensure getrandom works in WASM
    let id = uuid::Uuid::new_v4();
    assert!(!id.is_nil());
}

# Run WASM tests
```

```text
// test/integration.mjs
import { Miniflare } from 'miniflare';
```


### const mf = new Miniflare({

```yaml
  scriptPath: './build/worker/shim.mjs',
  modules: true,
  kvNamespaces: ['KV'],
  d1Databases: ['DB'],
});

// Test MCP initialize
const initResponse = await mf.dispatchFetch('http://localhost/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'initialize',
    params: {
      protocolVersion: '2025-11-25',
      capabilities: {},
      clientInfo: { name: 'test', version: '1.0' }
    }
  })
});
```

const result = await initResponse.json(); console.assert(result.result.protocolVersion === '2025-11-25'); console.log('Initialize test passed!');

```bash
# Run integration tests
npx wrangler dev --test
```

```bash
# Start local dev server
wrangler dev

# In another terminal, test with curl
curl -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

```rust
use worker::console_log;

// Simple logging
console_log!("Processing request: {}", request_id);

// Structured logging
fn log_json(label: &str, value: &impl serde::Serialize) {
    let json = serde_json::to_string_pretty(value).unwrap_or_default();
    console_log!("{}: {}", label, json);
}

// Debug logging (only in dev)
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        console_log!("[DEBUG] {}", format!($($arg)*))
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}
```


### Panic Handling

```rust
use console_error_panic_hook;

// Set up panic hook at worker start
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// In your main function
#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    init_panic_hook();
    // ... rest of handler
}
```


### Source Maps

```toml
# wrangler.toml
[build]
command = "cargo install worker-build && worker-build --release"

[build.upload]
format = "modules"
main = "./build/worker/shim.mjs"

# Enable source maps for debugging
[env.dev]
[env.dev.build]
command = "worker-build --dev"
```


### Performance Profiling

```rust
use worker::*;

struct Timer {
    label: String,
    start: f64,
}

impl Timer {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            start: js_sys::Date::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = js_sys::Date::now() - self.start;
        console_log!("[PERF] {}: {:.2}ms", self.label, elapsed);
    }
}

// Usage
async fn handle_tool_call(env: &Env, tool: &str) -> Result<Value> {
    let _timer = Timer::new(&format!("tool:{}", tool));

    // ... tool implementation

    Ok(json!({"result": "done"}))
} // Timer logs duration when dropped
```

```rust
// src/lib.rs
use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod tools;
mod error;

use error::{McpError, McpResult};

// Initialize panic hook for better error messages
fn init() {
    console_error_panic_hook::set_once();
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    init();

    // CORS headers for browser clients
    let cors_headers = Headers::new();
    cors_headers.set("Access-Control-Allow-Origin", "*")?;
    cors_headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
    cors_headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    // Handle CORS preflight
    if req.method() == Method::Options {
        return Response::empty()
            .map(|r| r.with_headers(cors_headers));
    }

    // Only accept POST to /mcp
    if req.method() != Method::Post || req.path() != "/mcp" {
        return Response::error("Not Found", 404);
    }

    // Parse and handle MCP request
    let result = handle_mcp_request(req, &env).await;

    match result {
        Ok(response) => Response::from_json(&response)
            .map(|r| r.with_headers(cors_headers)),
        Err(e) => {
            console_log!("Error: {}", e);
            Response::from_json(&json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                }
            }))
            .map(|r| r.with_headers(cors_headers))
        }
    }
}

async fn handle_mcp_request(mut req: Request, env: &Env) -> McpResult<Value> {
    let body: Value = req.json().await
        .map_err(|e| McpError::InvalidRequest(e.to_string()))?;

// ... truncated, 156 more lines in source book ...
```

- Building WASM-compatible Rust MCP servers requires understanding:

1.


### Practice Ideas

These informal exercises help reinforce the concepts.


### Practice 1: Crate Audit

Review your existing Rust project's dependencies and identify which crates need WASM-specific configuration or replacement.


### Practice 2: Memory Profiling

Build a test Worker that processes large JSON payloads and measure memory usage across multiple requests.


### Practice 3: Binary Size Optimization

Take an existing Worker and reduce its binary size by 50% while maintaining functionality.


### Additional Resources

WebAssembly Specification Cloudflare Workers Rust Documentation wasm-bindgen Guide The  worker  Crate Documentation
