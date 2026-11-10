---
chapter: 23
title: "Middleware and Instrumentation"
part: "Part VII: Observability"
use_when: "Adding logging, metrics, tracing, health checks, resilience via built-in observability or custom middleware."
suggested_prerequisites: [ch02]
source_kb: 149
extracted_kb: 75
code_blocks: 73
quality: auto-extracted
---

# Ch 23: Middleware and Instrumentation

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Middleware and Instrumentation

Enterprise MCP servers require comprehensive observability—you can't fix what you can't see.


### Why Observability Matters for MCP

```text
                  The Observability Challenge
Without Observability:             With Observability:
AI Client                            AI Client
  MCP Server  ← "It's broken"        MCP Server
   [????]                              Logs         ← Request traced
   [????]                              Metrics      ← Duration: 250ms
   [????]                              Traces       ← Error: DB timeout
"No idea what                      "DB connection pool
 happened"                          exhausted at 14:23"
```

(table/list, cells in reading order)
- Enterprise Requirements
- Production MCP servers must answer:
- Question
- Required Capability

What requests are failing?


### Structured logging with context

How long do tools take?


### Request duration metrics

What's the error rate?


### Error tracking and categorization

Why did this request fail?


### Distributed tracing

Are my dependencies healthy?


### Health checks and circuit breakers

Who's using the server?

```rust
// The tracing ecosystem - structured, contextual logging
use tracing::{info, error, instrument, span, Level};

// Metrics with compile-time validation
use metrics::{counter, gauge, histogram};

// Async-first design works perfectly with MCP's async handlers
#[instrument(skip(input), fields(tool = "get-weather", city = %input.city))]
async fn handler(input: WeatherInput) -> Result<Weather> {
    let start = Instant::now();

    let result = fetch_weather(&input.city).await;

    histogram!("tool.duration_ms").record(start.elapsed().as_millis() as f64);
    counter!("tool.calls_total", "tool" => "get-weather").increment(1);

    result
}
```


### Built-in Observability Module (Recommended)

PMCP v1.9.2+ includes a built-in observability module that handles logging, metrics, and distributed tracing out of the box. For most use cases, this is the recommended approach—you get production-ready observability with a single method call.

```text
                  Built-in vs Custom Observability
Built-in Observability:              Custom Middleware:
ServerCoreBuilder::new()             ServerCoreBuilder::new()
    .name("my-server")                   .name("my-server")
    .tool("weather", WeatherTool)        .tool("weather", WeatherTool)
    .with_observability(config)  ←       .with_tool_middleware(...)
    .build()                             .with_tool_middleware(...)
                                         .with_tool_middleware(...)
One line, full observability!            .build()
Use built-in when:                   Use custom when:
• Starting a new project             • Need custom metrics
• Standard observability needs       • Complex business logic
• Quick setup required               • Custom backends
• CloudWatch or console output       • Non-standard integrations
```

```rust
    server::builder::ServerCoreBuilder,
    server::observability::ObservabilityConfig,
    ServerCapabilities,
};

fn main() -> pmcp::Result<()> {
    // Development: Pretty console output
    let config = ObservabilityConfig::development();

    let server = ServerCoreBuilder::new()
        .name("my-weather-server")
        .version("1.0.0")
        .tool("get_weather", GetWeatherTool)
        .capabilities(ServerCapabilities::tools_only())
        .with_observability(config)  // One line adds full observability!
        .build()?;

    Ok(())
}
```

(table/list, cells in reading order)
- Configuration Presets
- PMCP provides ready-to-use configuration presets:
- Preset
- Backend
- Use Case

```rust
ObservabilityConfig::development()
```

```rust
ObservabilityConfig::production()
```

```rust
ObservabilityConfig::disabled()

ObservabilityConfig::default()
```

(table/list, cells in reading order)
- None
- Console
- Testing, minimal overhead
- General purpose
- TOML Configuration
- Configure observability via  .pmcp-config.toml :

```toml
[observability]
enabled = true
backend = "console"  # or "cloudwatch"
sample_rate = 1.0    # 1.0 = 100% of requests
max_depth = 10       # Loop prevention for composed servers

[observability.console]
pretty = true
verbose = false

[observability.cloudwatch]
namespace = "MyApp/MCP"
emf_enabled = true   # CloudWatch Embedded Metric Format
```

```bash
# Master controls
export PMCP_OBSERVABILITY_ENABLED=true
export PMCP_OBSERVABILITY_BACKEND=cloudwatch
export PMCP_OBSERVABILITY_SAMPLE_RATE=0.1  # Sample 10% in high-traffic

# CloudWatch settings
export PMCP_CLOUDWATCH_NAMESPACE="Production/MCPServers"
export PMCP_CLOUDWATCH_EMF_ENABLED=true

# Console settings
export PMCP_CONSOLE_PRETTY=false  # JSON output for log aggregation
```

```rust
use pmcp::server::observability::TraceContext;

// Create a root trace for a new request
let root_trace = TraceContext::new_root();
println!("trace_id: {}", root_trace.short_trace_id());
println!("span_id: {}", &root_trace.span_id[..8]);

// Create child spans for sub-operations
let child_trace = root_trace.child();
println!("parent_span_id: {}", child_trace.parent_span_id.as_ref().unwrap());
```

(table/list, cells in reading order)
- What Gets Captured
- The built-in observability middleware automatically captures:
- Event Type
- Data Captured
- Request Events
- trace_id, span_id, server_name, method, tool_name, user_id, tenant_id
- Response Events
- duration_ms, success/failure, error_code, response_size
- Metrics
- request_count, request_duration, error_count, composition_depth
- CloudWatch EMF Integration

```rust
let config = ObservabilityConfig::production();
// EMF logs automatically become CloudWatch metrics:
// - MCP/RequestDuration
// - MCP/RequestCount
// - MCP/ErrorCount
```

```bash
cargo run --example 61_observability_middleware
```


### This demonstrates:

Development configuration (console output) Production configuration (CloudWatch EMF) Custom configuration (sampling, field capture) Disabled observability (for testing) Loading from file/environment Trace context propagation


### When to Use Custom Middleware Instead

The built-in observability is sufficient for most use cases. Consider custom middleware when you need:


### PMCP Middleware Architecture

PMCP provides a layered middleware system for both protocol-level and HTTP-level instrumentation:

```text
               PMCP Middleware Layers
                  HTTP Layer (Transport)
ServerHttpMiddleware / HttpMiddleware
• CORS headers
• Rate limiting
• OAuth token injection
• Request/response logging (with redaction)
• Compression
              Protocol Layer (JSON-RPC)
AdvancedMiddleware / Middleware
• Request validation
• Metrics collection
• Circuit breaker
• Request timing
• Context propagation
                 Tool Handlers
TypedToolWithOutput implementations
• Business logic
• Tool-specific metrics
• Domain logging
```

```text
pub enum MiddlewarePriority {
    Critical = 0,  // Security, validation - runs first
    High = 1,      // Authentication, rate limiting
    Normal = 2,    // Business logic middleware
    Low = 3,       // Logging, metrics
    Lowest = 4,    // Cleanup, finalization
}
```

(table/list, cells in reading order)
- Built-in Middleware
- PMCP includes production-ready middleware:
- Middleware
- Purpose
- Priority
- MetricsMiddleware
- LoggingMiddleware
- Performance metrics collection
- Request/response logging
- RateLimitMiddleware
- Request throttling
- CircuitBreakerMiddleware
- Failure isolation
- Low
- Low
- High
- High
- CompressionMiddleware
- Response compression
- Normal
- ServerHttpLoggingMiddleware
- HTTP-level logging with redaction
- Normal
- OAuthClientMiddleware
- Token injection
- High
- Testing as Observability
- Your test scenarios from earlier chapters become observability tools:

```text
               Testing as Observability
 CI/CD             Scheduled           Continuous Monitoring
Pipeline            Jobs
              cargo pmcp test --server <name>
scenarios/server-name/
    smoke.yaml        # Basic connectivity
    tools.yaml        # Tool functionality
    edge_cases.yaml   # Error handling
    perf.yaml         # Performance baselines
                   Observability Signals
[ok] Database still accessible    (data system availability)
[ok] API keys valid               (secret rotation check)
[ok] Response times normal        (performance regression)
[ok] Error rates acceptable       (quality baseline)
```

```text
# .github/workflows/health-check.yml
```


### name: MCP Server Health Check

```bash
on:
  schedule:
    - cron: '*/15 * * * *'  # Every 15 minutes
  workflow_dispatch:

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run smoke tests
        run: |
          cargo pmcp test --server weather --scenario smoke
        env:
          WEATHER_API_KEY: ${{ secrets.WEATHER_API_KEY }}

      - name: Alert on failure
        if: failure()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "MCP Server health check failed!"
            }
```

(table/list, cells in reading order)
- Detecting Issues
- Issue Type
- Detection Method
- Database unavailable
- Scenario step times out
- Secret rotation needed
- Authentication error in test
- Performance regression
- Duration assertion fails
- API breaking change
- Schema validation fails
- Rate limit exhausted
- Error response matches pattern
- Chapter Contents
- This chapter covers:

1.


### Key Takeaways

Observability is not optional for enterprise MCP servers Middleware provides the instrumentation hooks at both HTTP and protocol layers Rust's tracing ecosystem offers structured, zero-cost logging Metrics enable alerting before users notice problems Test scenarios become health checks when run peri


### Middleware Architecture

PMCP's middleware system provides extensible hooks for request/response processing.

```text
                  The Middleware Mental Model
Without Middleware:                 With Middleware:
Client → Tool Handler → Response    Client
Every handler must:                     ▼
• Validate requests
• Log operations                      Validation   ← Check request
• Track timing
• Handle rate limits
• Manage authentication                   ▼
• Record metrics
• ...for every single tool!           Auth Check   ← Verify identity
Problems:
• Duplicated code everywhere              ▼
• Easy to forget steps
• Inconsistent behavior                Rate Limit   ← Control traffic
• Hard to change globally
                                          Your
                                         Handler     ← Business logic
                                                       ONLY
                                         Logging     ← Record result
                                        Response
Benefits:
```

```text
[ok] Write validation ONCE, apply to ALL requests
[ok] Handlers focus purely on business logic
[ok] Consistent behavior across all tools
[ok] Easy to add/remove cross-cutting concerns
```


### Cross-Cutting Concerns

Middleware handles "cross-cutting concerns"—functionality that applies across your entire application rather than to specific features:

(table/list, cells in reading order)
- Concern
- Without Middleware
- With Middleware
- Logging
- Add log statements to every handler
- Single logging middleware logs all requests
- Authentication
- Check auth in every handler
- Auth middleware validates once, sets context
- Rate limiting
- Implement counters in each handler
- Rate limit middleware protects everything
- Metrics
- Record timing in every handler
- Metrics middleware measures automatically
- Error handling
- Try-catch in every handler
- Error middleware provides consistent responses
- The Pipeline Pattern

```text
// Each middleware can:
// 1. Inspect the request
// 2. Modify the request
// 3. Short-circuit (return early without calling the next middleware)
// 4. Pass to the next middleware
// 5. Inspect/modify the response on the way back
```

```rust
use async_trait::async_trait;
use pmcp::shared::{AdvancedMiddleware, MiddlewareContext, MiddlewarePriority};
use pmcp::types::{JSONRPCRequest, JSONRPCResponse};
use pmcp::Result;

#[async_trait]
pub trait AdvancedMiddleware: Send + Sync {
    /// Execution priority (lower = runs first)
    fn priority(&self) -> MiddlewarePriority {
        MiddlewarePriority::Normal
    }

    /// Middleware name for identification
    fn name(&self) -> &'static str {
        "unknown"
    }

    /// Conditional execution check
    async fn should_execute(&self, context: &MiddlewareContext) -> bool {
        true
    }

    /// Process outgoing request
    async fn on_request_with_context(
        &self,
        request: &mut JSONRPCRequest,
        context: &MiddlewareContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Process incoming response
    async fn on_response_with_context(
        &self,
        response: &mut JSONRPCResponse,
        context: &MiddlewareContext,
    ) -> Result<()> {
        Ok(())

    }
}
```


### Execution Order

```text
                  Middleware Execution Flow
                         REQUEST PATH
Client Request
  Critical (0)     ← Validation, security checks
  High (1)         ← Rate limiting, authentication
  Normal (2)       ← Business logic transforms
  Low (3)          ← Logging, metrics recording
  Lowest (4)       ← Cleanup, finalization
   Tool Handler
```

```text
                         RESPONSE PATH
  Lowest (4)       ← Response timing recorded
  Low (3)          ← Response logged
... (continues up to Critical)
Client Response
```

```rust
use async_trait::async_trait;
use pmcp::shared::{AdvancedMiddleware, MiddlewareContext, MiddlewarePriority};
use pmcp::types::{JSONRPCRequest, JSONRPCResponse};
use pmcp::Result;
use std::sync::Arc;
use dashmap::DashMap;
use std::time::Instant;

pub struct TimingMiddleware {
    start_times: DashMap<String, Instant>,
}

impl TimingMiddleware {
    pub fn new() -> Self {
        Self {
            start_times: DashMap::new(),
        }
    }
}

#[async_trait]
impl AdvancedMiddleware for TimingMiddleware {
    fn name(&self) -> &'static str {
        "timing"
    }

    fn priority(&self) -> MiddlewarePriority {
        MiddlewarePriority::Low  // Run late so we time everything
    }

    async fn on_request_with_context(
        &self,
        request: &mut JSONRPCRequest,
        context: &MiddlewareContext,
    ) -> Result<()> {
        // Record start time keyed by request ID
        if let Some(ref request_id) = context.request_id {
            self.start_times.insert(request_id.clone(), Instant::now());
        }

        tracing::debug!(
            method = %request.method,
            request_id = ?context.request_id,
            "Request started"
        );

        Ok(())
    }

    async fn on_response_with_context(
        &self,
        response: &mut JSONRPCResponse,
        context: &MiddlewareContext,
    ) -> Result<()> {
        // Calculate duration
        if let Some(ref request_id) = context.request_id {
            if let Some((_, start)) = self.start_times.remove(request_id) {
                let duration = start.elapsed();

                // Record in context metrics
// ... truncated, 16 more lines in source book ...
```

```rust
use pmcp::shared::{AdvancedMiddleware, MiddlewareContext, MiddlewarePriority};
use pmcp::Error;

pub struct ValidationMiddleware {
    strict_mode: bool,
}

#[async_trait]
impl AdvancedMiddleware for ValidationMiddleware {
    fn name(&self) -> &'static str {
        "validation"
    }

    fn priority(&self) -> MiddlewarePriority {
        MiddlewarePriority::Critical  // Run first - block invalid requests
    }

    async fn should_execute(&self, context: &MiddlewareContext) -> bool {
        // In non-strict mode, only validate high-priority requests
        if !self.strict_mode {
            matches!(
                context.priority,
                Some(pmcp::shared::transport::MessagePriority::High)
            )
        } else {
            true  // Always validate in strict mode
        }
    }

    async fn on_request_with_context(
        &self,
        request: &mut JSONRPCRequest,
        context: &MiddlewareContext,
    ) -> Result<()> {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            context.record_metric("validation_failures".to_string(), 1.0);
            return Err(Error::Validation(
                "Invalid JSON-RPC version".to_string()

            ));
        }


        // Validate method not empty
        if request.method.is_empty() {
            context.record_metric("validation_failures".to_string(), 1.0);
            return Err(Error::Validation(
                "Method name cannot be empty".to_string()
            ));
        }

        // Store method in context for later middleware
        context.set_metadata("method".to_string(), request.method.clone());
        context.record_metric("validation_passed".to_string(), 1.0);

        Ok(())
    }
}
```

```rust
use uuid::Uuid;

pub struct RequestIdMiddleware;

#[async_trait]
impl AdvancedMiddleware for RequestIdMiddleware {
    fn name(&self) -> &'static str {
        "request_id"
    }

    fn priority(&self) -> MiddlewarePriority {
        MiddlewarePriority::Critical  // Run first to set ID
    }

    async fn on_request_with_context(
        &self,
        request: &mut JSONRPCRequest,
        context: &MiddlewareContext,
    ) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();

        // Store in context for other middleware
        context.set_metadata("request_id".to_string(), request_id.clone());
        context.set_metadata("correlation_id".to_string(), request_id.clone());

        // Optionally inject into request params
        if let Some(params) = request.params.as_mut() {
            if let Some(obj) = params.as_object_mut() {
                obj.insert(
                    "_request_id".to_string(),
                    serde_json::json!(request_id)
                );
            }
        }

        tracing::info!(
            request_id = %request_id,
            method = %request.method,
            "Assigned request ID"

        );

        Ok(())
    }


    async fn on_response_with_context(
        &self,
        _response: &mut JSONRPCResponse,
        context: &MiddlewareContext,
    ) -> Result<()> {
        if let Some(request_id) = context.get_metadata("request_id") {
            tracing::debug!(
                request_id = %request_id,
                "Response completed for request"
            );
        }
        Ok(())
    }
}
```

```rust
use pmcp::shared::EnhancedMiddlewareChain;
use std::sync::Arc;

fn build_observability_chain() -> EnhancedMiddlewareChain {
    let mut chain = EnhancedMiddlewareChain::new();

    // Add middleware (automatically sorted by priority)
    chain.add(Arc::new(RequestIdMiddleware));
    chain.add(Arc::new(ValidationMiddleware { strict_mode: true }));
    chain.add(Arc::new(TimingMiddleware::new()));
    chain.add(Arc::new(MetricsMiddleware::new("my-server".to_string())));

    chain
}
```


### Using Built-in Observability (Recommended)

For standard observability needs, use the built-in module instead of building custom chains:

```rust
use pmcp::server::builder::ServerCoreBuilder;
use pmcp::server::observability::ObservabilityConfig;

// Using ServerCoreBuilder
let server = ServerCoreBuilder::new()
    .name("my-server")
    .version("1.0.0")
    .tool("echo", EchoTool)
    .capabilities(ServerCapabilities::tools_only())
    .with_observability(ObservabilityConfig::development())
    .build()?;

// Or using Server::builder() (same API)
let server = Server::builder()
    .name("my-server")
    .version("1.0.0")
    .tool("echo", EchoTool)
    .with_observability(ObservabilityConfig::production())
    .build()?;
```


### This adds a pre-configured  McpObservabilityMiddleware  that handles:

Distributed tracing with  TraceContext Request/response event logging Automatic metrics collection Console or CloudWatch output


### Integrating with ClientBuilder

```rust
use pmcp::{ClientBuilder, StdioTransport};

async fn create_instrumented_client() -> pmcp::Result<Client> {
    let transport = StdioTransport::new();

    let client = ClientBuilder::new(transport)
        .with_middleware(Arc::new(RequestIdMiddleware))
        .with_middleware(Arc::new(TimingMiddleware::new()))
        .with_middleware(Arc::new(MetricsMiddleware::new("my-client".to_string())))
        .build();

    Ok(client)
}
```

```rust
use pmcp::shared::middleware_presets::PresetConfig;
use pmcp::{ClientBuilder, StdioTransport};

// For stdio transport
let client = ClientBuilder::new(StdioTransport::new())
    .middleware_chain(PresetConfig::stdio().build_protocol_chain())
    .build();

// For HTTP transport
let http_chain = PresetConfig::http().build_protocol_chain();
```

```rust
use async_trait::async_trait;
use pmcp::server::http_middleware::{
    ServerHttpMiddleware, ServerHttpContext, ServerHttpResponse,
};

/// CORS middleware for browser clients
#[derive(Debug, Clone)]
struct CorsMiddleware {
    allowed_origins: Vec<String>,
}

#[async_trait]
impl ServerHttpMiddleware for CorsMiddleware {
    async fn on_response(
        &self,
        response: &mut ServerHttpResponse,
        _context: &ServerHttpContext,
    ) -> pmcp::Result<()> {
        response.add_header(
            "Access-Control-Allow-Origin",
            &self.allowed_origins.join(", ")
        );
        response.add_header(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS"
        );
        response.add_header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, MCP-Session-ID"
        );
        response.add_header("Access-Control-Max-Age", "86400");

        Ok(())
    }

    fn priority(&self) -> i32 {
        90  // Run after logging

    }
}
```

```rust
    ServerHttpLoggingMiddleware,
    ServerHttpMiddlewareChain,
};

let mut http_chain = ServerHttpMiddlewareChain::new();

let logging = ServerHttpLoggingMiddleware::new()
    .with_level(tracing::Level::INFO)
    .with_redact_query(true)        // Strip query params from logs
    .with_max_body_bytes(1024);     // Limit body logging size
```

(table/list, cells in reading order)
- http_chain.add(Arc::new(logging));
- Automatically redacted headers:
- Authorization
- Cookie
- X-Api-Key
- Complete Server Setup
- use pmcp::server::streamable_http_server::{

```rust
    StreamableHttpServer,
    StreamableHttpServerConfig,
};

// Build server with HTTP middleware
let server = Server::builder()
    .name("instrumented-server")
    .version("1.0.0")
    .capabilities(ServerCapabilities::tools_only())
    .tool("echo", EchoTool)
    .with_http_middleware(Arc::new(http_chain))
    .build()?;

// Create HTTP server config
let config = StreamableHttpServerConfig {
    http_middleware: server.http_middleware(),
    session_id_generator: Some(Box::new(|| {
        format!("session-{}", uuid::Uuid::new_v4())
    })),
    enable_json_response: true,
    ..Default::default()
};

let http_server = StreamableHttpServer::with_config(
    "0.0.0.0:8080".parse().unwrap(),
    Arc::new(Mutex::new(server)),
    config
);
```

```rust
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    /// Request ID for correlation
    pub request_id: Option<String>,

    /// Custom metadata (thread-safe)
    pub metadata: Arc<DashMap<String, String>>,

    /// Performance metrics
    pub metrics: Arc<PerformanceMetrics>,

    /// Request start time
    pub start_time: Instant,

    /// Priority level
    pub priority: Option<MessagePriority>,
}

impl MiddlewareContext {
    /// Store metadata for other middleware
    pub fn set_metadata(&self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Retrieve metadata from earlier middleware
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).map(|v| v.clone())
    }

    /// Record a metric value
    pub fn record_metric(&self, name: String, value: f64) {
        self.metrics.record(name, value);
    }

    /// Get elapsed time since request started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()

    }
}
```


### Context Usage Pattern

```rust
// Early middleware sets context
async fn on_request_with_context(
    &self,
    request: &mut JSONRPCRequest,
    context: &MiddlewareContext,
) -> Result<()> {
    // Set user ID from auth token
    context.set_metadata("user_id".to_string(), "user-123".to_string());
    context.set_metadata("tenant_id".to_string(), "acme-corp".to_string());
    Ok(())
}

// Later middleware reads context
async fn on_request_with_context(
    &self,
    request: &mut JSONRPCRequest,
    context: &MiddlewareContext,
) -> Result<()> {
    let user_id = context.get_metadata("user_id")
        .unwrap_or_else(|| "anonymous".to_string());

    tracing::info!(
        user_id = %user_id,
        method = %request.method,
        "Audit log: User invoked method"
    );
    Ok(())
}
```


### Resilience Patterns

Production systems fail. Resilience patterns are defensive programming techniques that help your system survive and recover from these failures gracefully, rather than cascading into complete outages.


### Rate Limiting

What is Rate Limiting?

```text
                  Rate Limiting Visualized
Without Rate Limiting:              With Rate Limiting:
   Client A                        Client A
   Client A                        Client A
   Client A                        Client A         Rate
   Client A     ▶ Server 💥        Client A        Limiter    ▶ Server
   Client A       (overwhelmed)    Client A
   Client A                                        5 req/s
                                   Client A
Result:                            Client A
• Server crashes                   Client A
• All users affected                                 ▼
• Potential data loss                        "Rate Limited"
                                             (try again later)
Result with limiting:
• Server stays healthy
• Fair access for all clients
• Excess requests get clear feedback
```

(table/list, cells in reading order)
- Why Rate Limiting Matters
- Problem
- How Rate Limiting Helps
- DoS attacks
- Prevents malicious clients from overwhelming your server
- Runaway AI loops
- Stops buggy AI clients from making infinite tool calls
- Resource exhaustion
- Protects expensive operations (database queries, API calls)
- Fair usage
- Ensures no single client monopolizes server capacity
- Problem
- How Rate Limiting Helps
- Cost control
- Limits calls to expensive external APIs (GPT-4, cloud services)
- The Token Bucket Algorithm

```text
                  Token Bucket Algorithm
     Token Bucket            Tokens added at fixed rate
                             (e.g., 5 tokens per second)
      ● ● ● ● ●        ◄     Bucket has max capacity
      ● ● ●                  (e.g., 10 tokens = burst capacity)
      Each request
      consumes 1 token
  Tokens available?      ▶ Process request
                         ▶ Reject (429)
         No
Example: 5 req/sec rate, 10 burst capacity
Time 0s: Bucket full (10 tokens)
Time 0s: 8 requests arrive → 8 processed, 2 tokens left
Time 1s: 5 tokens added → 7 tokens available
Time 1s: 3 requests arrive → 3 processed, 4 tokens left
Time 2s: 5 tokens added → 9 tokens (capped at 10)
Key: Burst allows brief spikes above the steady-state rate
```


### When to Use Rate Limiting

Always for public-facing MCP servers Always when calling expensive external APIs When serving multiple clients with shared resources When you have resource constraints (memory, CPU, database connections) When cost per request matters (cloud API calls, AI model inference)


### PMCP Rate Limiting Implementation

```rust
use pmcp::shared::RateLimitMiddleware;
use std::time::Duration;

// Configure the rate limiter
let rate_limiter = RateLimitMiddleware::new(
    5,                          // Requests per window (steady rate)
    10,                         // Burst capacity (max tokens in bucket)
    Duration::from_secs(1),     // Window size (token refill period)
);

// This configuration means:
// - Sustained rate: 5 requests per second
// - Burst: Up to 10 requests if bucket is full
// - After burst: Must wait for tokens to refill
```


### Circuit Breaker

What is a Circuit Breaker?

```text
                  Circuit Breaker States
            ▶   CLOSED   ◀
              (Normal)
        Failures exceed         Success in
        threshold               half-open state
                 OPEN
              (Failing)
        Timeout expires     Failure in
                            half-open state
              HALF-OPEN
              (Testing)
State Behaviors:
CLOSED (Normal):    All requests pass through to the handler
                    Track failure count
OPEN (Failing):     All requests IMMEDIATELY rejected (fail fast)
                    Don't even try calling the failing service
                    Wait for recovery timeout
```

```text
HALF-OPEN (Testing): Allow ONE request through to test recovery
                     If success → CLOSED (service recovered!)
                     If failure → OPEN (still broken)
```


### Why Circuit Breakers Matter

Without circuit breakers, a failing dependency causes cascade failures:

```text
                  Cascade Failure Without Circuit Breaker
1. Database becomes slow (overloaded)
2. MCP Server keeps trying
   • Requests pile up waiting for database
   • Thread pool exhausted
   • Memory fills with pending requests
3. MCP Server stops responding
   • AI client times out
   • Retries make it worse
4. Complete outage
   • Even requests that don't need the database fail
   • Recovery requires restart
With Circuit Breaker:
1. Database becomes slow
2. After N failures, circuit OPENS
   • Requests fail immediately (no waiting)
   • Clear error: "Service temporarily unavailable"
   • Resources freed instantly
3. Server stays healthy
   • Other tools continue working
   • No resource exhaustion
4. Automatic recovery testing
   • Circuit tries HALF-OPEN periodically
   • When database recovers, circuit CLOSES automatically
```

(table/list, cells in reading order)
- Problem
- How Circuit Breaker Helps
- Cascade failures
- Stops failure from spreading to healthy components
- Resource exhaustion
- Frees threads/memory instead of waiting on broken services
- Slow failures
- Converts slow timeouts into fast failures
- Automatic recovery
- Detects when service recovers, no manual intervention
- User experience
- Fast "service unavailable" beats slow timeout
- When to Use Circuit Breakers

When calling external APIs (weather services, AI models, databases) When a dependency failure shouldn't crash your entire server When you need automatic recovery detection When fast failure is better than slow failure (almost always!) When dealing with unreliable network connections


### PMCP Circuit Breaker Implementation

```rust
use pmcp::shared::CircuitBreakerMiddleware;
use std::time::Duration;

// Configure the circuit breaker
let circuit_breaker = CircuitBreakerMiddleware::new(
    3,                          // Failure threshold (trips after 3 failures)
    Duration::from_secs(10),    // Failure window (3 failures within 10s trips)
    Duration::from_secs(5),     // Recovery timeout (wait 5s before testing)
);

// This configuration means:
// - If 3 requests fail within a 10-second window, circuit OPENS
// - While OPEN, all requests immediately fail (no actual execution)
// - After 5 seconds, circuit goes HALF-OPEN to test recovery
// - One successful request closes circuit; one failure reopens it
```

```text
                  Resilience Defense in Depth
Incoming Request
   Rate Limiter     ▶ Too many requests? → 429 "Rate Limited"
          OK
 Circuit Breaker    ▶ Circuit open? → 503 "Service Unavailable"
          OK
   Tool Handler     ▶ Actual work happens here
Success or failure
(failure increments circuit breaker counter)
```


### Combined Resilience Chain

```rust
fn build_resilient_chain() -> EnhancedMiddlewareChain {
    let mut chain = EnhancedMiddlewareChain::new();

    // Resilience middleware (High priority - runs early)
    // Rate limiter first: reject excess traffic before it hits circuit breaker
    chain.add(Arc::new(RateLimitMiddleware::new(
        100, 200, Duration::from_secs(1)
    )));
    // Circuit breaker second: fast-fail if dependencies are down
    chain.add(Arc::new(CircuitBreakerMiddleware::new(
        5, Duration::from_secs(30), Duration::from_secs(10)
    )));

    // Observability middleware (Low priority - runs late)
    chain.add(Arc::new(TimingMiddleware::new()));
    chain.add(Arc::new(MetricsMiddleware::new("my-server".to_string())));

    chain
}
```

(table/list, cells in reading order)
- Choosing the Right Configuration
- Scenario
- Rate Limit
- Circuit Breaker
- AI chatbot backend
- 10 req/s, burst 20
- 5 failures in 30s, 10s recovery
- Internal tool server
- 100 req/s, burst 500
- 10 failures in 60s, 30s recovery
- Public API
- 5 req/s per client
- 3 failures in 10s, 5s recovery
- Database-heavy tools
- 20 req/s
- 3 failures in 5s, 15s recovery
- Guidelines:
- Rate limits: Start conservative, increase based on monitoring data

Circuit breaker threshold: Lower = faster failure detection, but more false positives Recovery timeout: Long enough for actual recovery, short enough to restore service promptly

(table/list, cells in reading order)
- Best Practices
- 1. Use Appropriate Priorities
- Middleware Type
- Priority
- Reason
- Request ID generation
- Critical
- Needed by all other middleware
- Validation
- Rate limiting
- Circuit breaker
- Business logic
- Logging
- Metrics
- Cleanup
- Critical
- Reject bad requests early
- High
- High
- Protect resources before processing
- Fail fast when unhealthy
- Normal
- After protection, before logging
- Low
- Low
- Capture complete request lifecycle
- Record after all processing
- Lowest
- Final resource release
- 2. Keep Middleware Focused

```text
// GOOD: Single responsibility
```

```text
// BAD: Too many responsibilities
```


### 3. Make Middleware Stateless When Possible

```yaml
// GOOD: Stateless (easily clonable, no synchronization)
struct ValidationMiddleware {
    strict_mode: bool,  // Configuration, not state
}

// OK: State with thread-safe access
struct TimingMiddleware {
    start_times: DashMap<String, Instant>,  // Thread-safe map
}

// BAD: Mutable state without synchronization
struct BrokenMiddleware {
    request_count: u64,  // Data race!
}
```


### 4. Handle Errors Gracefully

```rust
async fn on_request_with_context(
    &self,
    request: &mut JSONRPCRequest,
    context: &MiddlewareContext,
) -> Result<()> {
    // Log and continue if non-critical
    if let Err(e) = self.optional_check() {
        tracing::warn!(error = %e, "Optional check failed, continuing");
    }

    // Return error only for critical failures
    self.required_check()
        .map_err(|e| Error::Validation(format!("Critical check failed: {}", e)))
}
```

(table/list, cells in reading order)
- Summary
- PMCP's middleware architecture provides:
- Feature
- Benefit
- Priority ordering
- Predictable execution flow
- Context propagation
- Share data between middleware
- Two-layer system
- HTTP and protocol-level hooks
- Built-in middleware
- Production-ready rate limiting, circuit breaker
- Presets
- Async-first
- Quick setup for common scenarios
- Works naturally with MCP's async handlers

The middleware system enables comprehensive observability without modifying tool handlers—instrumentation is orthogonal to business logic.


### Logging Best Practices

Effective logging transforms debugging from guesswork into investigation.


### Why Logging Matters

If you're new to production logging, you might wonder why we need anything beyond  println!  or simple file writes.

```text
                  The Production Debugging Challenge
The Scenario:
It's 3 AM. Your MCP server is failing intermittently. Users report
"sometimes it works, sometimes it doesn't." You need to find out:
• Which requests are failing?
• What was the server doing when it failed?
• What external services was it calling?
• What user data was involved (without exposing PII)?
• How long did each step take?
• What happened BEFORE the failure?
With println! debugging:         With production logging:
"Request received"               {"timestamp": "2024-12-30T03:14:22",
"Processing..."                   "level": "ERROR",
"Error: something failed"         "request_id": "abc-123",
                                  "user_tier": "enterprise",
Problems:                         "tool": "database-query",
• No timestamp                    "duration_ms": 30042,
• No context                      "error": "Connection timeout",
• Can't search/filter             "span": {
• Can't correlate requests          "db_host": "prod-db-02",
• No way to analyze patterns        "query_type": "select"
                                  }}
                                 Benefits:
                                 [ok] Exact time of failure
                                 [ok] Which request failed
                                 [ok] Full context chain
                                 [ok] Searchable & filterable
                                 [ok] Correlate across services
```

(table/list, cells in reading order)
- The Three Purposes of Logging
- Purpose
- Example
- What Good Logging Provides
- Debugging
- "Why did this request fail?"
- Full context: request ID, user, inputs, error chain
- Auditing
- "Who accessed this data?"
- Immutable record: who, what, when (without sensitive data)
- Monitoring
- "Is the system healthy?"
- Patterns: error rates, latency trends, usage spikes
- Logging vs. Metrics vs. Tracing
- These three observability tools serve different purposes:

```text
                  The Three Pillars of Observability
LOGS                    METRICS                  TRACES
What happened?          How much/how many?       Where did time go?
• Detailed events       • Numeric measurements   • Request flow
• Error messages        • Aggregated over time   • Cross-service
• Context-rich          • Alerts & dashboards    • Latency breakdown
Example:                Example:                 Example:
"User X called tool Y   "95th percentile         "Request took 500ms:
 at time Z, got error   latency is 250ms"        - 50ms auth
 E because of F"                                 - 400ms database
                                                 - 50ms serialization"
Best for:               Best for:                Best for:
• Debugging             • Alerting               • Performance
• Auditing              • Capacity planning      • Bottleneck finding
• Investigation         • SLA monitoring         • Distributed systems
In this chapter, we focus on LOGS and touch on TRACES (spans).
Metrics are covered in the next chapter.
```


### The Tracing Ecosystem

Rust's  tracing  crate provides structured, contextual logging designed for async applications:

```text
                  Tracing vs Traditional Logging
Traditional Logging:
println!("User {} called tool {}", user_id, tool_name);
Output: "User user-123 called tool get-weather"
Problems:
• No structure - hard to parse
• No context across async calls
• No levels, filtering, or sampling
Structured Tracing:
tracing::info!(
    user_id = %user_id,
    tool = %tool_name,
    "Tool invocation"
);
Output: {
  "timestamp": "2024-12-30T10:15:30Z",
  "level": "INFO",
  "target": "weather_server::tools",
  "fields": {
    "user_id": "user-123",
    "tool": "get-weather",
    "message": "Tool invocation"
  },
  "span": {
    "request_id": "abc-123",
    "session_id": "session-456"
```

```text
  }
}
Benefits:
[ok] Machine-parseable JSON
[ok] Context from parent spans
[ok] Levels, filtering, sampling
[ok] Works naturally with async
```


### Setting Up Tracing

```rust
// Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

// main.rs
fn main() {
    // Initialize with JSON output for production
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info,pmcp=debug,my_server=trace")
        .with_current_span(true)
        .with_span_list(true)
        .init();

    // Now use tracing macros
    tracing::info!("Server starting");
}
```


### Log Levels

Choosing the right log level is crucial—too verbose and you'll drown in noise; too quiet and you'll miss important events.

```text
                  Log Level Pyramid
                            ERROR     ← Something broke, needs fixing
                                        (always logged)
                            WARN         ← Might become a problem
                                           (always logged)
                            INFO              ← Normal milestones
                                                (production default)
                          DEBUG                   ← Diagnostic details
                                                     (development)
                           TRACE                        ← Everything
                                                          (debugging)
Production typically runs at INFO level:
• ERROR [ok]  WARN [ok]  INFO [ok]  DEBUG [no]  TRACE [no]
Development runs at DEBUG or TRACE:
• ERROR [ok]  WARN [ok]  INFO [ok]  DEBUG [ok]  TRACE [ok]
```

(table/list, cells in reading order)
- Level
- ERROR
- When to Use
- Operation failed, needs attention
- Examples
- Common Mistakes
- Database down, API key invalid, unrecoverable error
- Using for expected failures (user not found)
- Level
- WARN
- INFO
- When to Use
- Examples
- Common Mistakes
- Degraded but working, or suspicious activity
- Normal milestones worth knowing
- Rate limit at 80%, deprecated API used, retry succeeded
- Server started, tool executed, request completed
- Using for normal operation
- Too verbose (every cache hit)
- DEBUG
- Detailed info for developers
- Cache hit/miss, full request params, decision paths
- Logging in hot paths (performance)
- TRACE
- Very fine-grained tracing
- Function entry/exit, loop iterations, wire format
- Using in production (extreme noise)

```rust
use tracing::{error, warn, info, debug, trace};

async fn handler(input: WeatherInput) -> Result<Weather> {
    trace!(city = %input.city, "Handler entry");

    debug!("Checking cache for {}", input.city);

    let weather = match cache.get(&input.city) {
        Some(cached) => {
            info!(city = %input.city, "Cache hit");
            cached
        }
        None => {
            debug!(city = %input.city, "Cache miss, fetching from API");
            let result = api.fetch(&input.city).await?;
            cache.insert(input.city.clone(), result.clone());
            result
        }
    };

    if weather.temperature > 40.0 {
        warn!(
            city = %input.city,
            temp = %weather.temperature,
            "Extreme heat detected"
        );
    }

    trace!(city = %input.city, "Handler exit");
    Ok(weather)
}
```

PMCP v1.9.2+ includes a  TraceContext  type that automatically handles distributed tracing when you use the built-in observability module.

```rust
use pmcp::server::observability::TraceContext;

// TraceContext is automatically created and propagated by the middleware
// But you can also create them manually for custom scenarios:

let root = TraceContext::new_root();
```

```rust
let child = root.child();
```

Spans are essential in async and distributed systems because traditional stack traces don't work when execution jumps between tasks and services.

```text
                  Why Spans Matter in Async Systems
The Problem with Async:
In synchronous code, you can look at the call stack:
  main() → handle_request() → fetch_weather() → ERROR
In async code, execution bounces between tasks:
  Task A: handle_request() starts, awaits...
  Task B: different_request() runs
  Task C: yet_another_request() runs
  Task A: ...fetch_weather() resumes, ERROR!
When the error happens, you can't see the original context!
The Solution - Spans:
Spans carry context through async boundaries:
  Span: "handle_request" (request_id=abc-123, user=enterprise)
      ▶ Span: "validate_input"
            ▶ log: "Input validated"
      ▶ Span: "fetch_weather" (city=London)
            ▶ Span: "cache_lookup"
                  ▶ log: "Cache miss"
            ▶ Span: "api_call" (endpoint=weather-api)
                  ▶ log: "ERROR: Connection timeout"  ← HERE!
```

```text
      ▶ Total duration: 30,042ms
Now when you see the error, you know:
• request_id: abc-123 (find all logs for this request)
• user: enterprise (who was affected)
• city: London (what they were looking for)
• It happened in api_call inside fetch_weather
• The whole request took 30 seconds
```

(table/list, cells in reading order)
- Key Span Concepts
- Concept
- Description
- Example
- Parent span
- The outer operation containing this work
- handle_request  is parent of  fetch_weather
- Child span
- A sub-operation within a parent
- api_call  is child of  fetch_weather
- Span context
- Data attached to a span (and inherited by children)
- request_id ,  user_id
- Span duration
- Time from span start to end
- Helps find slow operations
- Spans create hierarchical context that flows through async calls:

```rust
use tracing::{instrument, span, Level, Instrument};

// Automatic span creation with #[instrument]
#[instrument(
    name = "get_weather",
    skip(input),
    fields(
        tool = "get-current-weather",
        city = %input.city,
        request_id = %Uuid::new_v4()
    )
)]
async fn handler(input: WeatherInput) -> Result<Weather> {
    // All logs inside here include the span context
    info!("Starting weather lookup");

    // Nested span for sub-operation
    let api_result = fetch_from_api(&input.city)
        .instrument(tracing::info_span!("api_call", endpoint = "weather"))
        .await?;

    info!(temp = %api_result.temperature, "Weather retrieved");
    Ok(api_result)
}

// Manual span creation
async fn process_batch(items: Vec<Item>) {
    let span = span!(Level::INFO, "batch_process", count = items.len());
    let _guard = span.enter();

    for (i, item) in items.iter().enumerate() {
        let item_span = span!(Level::DEBUG, "item", index = i, id = %item.id);
        let _item_guard = item_span.enter();

        process_item(item).await;
    }
}
```


### Span Output

```rust
{
  "timestamp": "2024-12-30T10:15:30.123Z",
  "level": "INFO",
  "message": "Weather retrieved",
  "target": "weather_server::tools::weather",
  "span": {
    "name": "get_weather",
    "tool": "get-current-weather",
    "city": "London",
    "request_id": "abc-123-def-456"
  },
  "spans": [
    { "name": "handle_request", "session_id": "session-789" },
    { "name": "get_weather", "city": "London" },
    { "name": "api_call", "endpoint": "weather" }
  ],
  "fields": {
    "temp": "22.5"
  }
}
```

```rust
use pmcp::types::protocol::LogLevel;

async fn handler(input: DatabaseInput) -> Result<QueryResult> {
    // Log to MCP client (visible in AI interface)
    pmcp::log(
        LogLevel::Info,
        "Starting database query",
        Some(serde_json::json!({
            "query_type": "select",
            "table": input.table
        }))
    ).await;

    // Simulate work
    for step in 1..=3 {
        pmcp::log(
            LogLevel::Info,
            &format!("Processing step {}/3", step),
            Some(serde_json::json!({
                "step": step,
                "progress": format!("{}%", step * 33)
            }))
        ).await;
    }

    // Warn about high resource usage
    pmcp::log(
        LogLevel::Warning,
        "Query returned large result set",
        Some(serde_json::json!({
            "row_count": 15000,
            "recommendation": "Consider pagination"
        }))
    ).await;

    Ok(result)
}
```


### Server Lifecycle Logging

```rust
async fn run_server() -> Result<()> {
    // Log startup with structured metadata
    pmcp::log(
        LogLevel::Info,
        "Server initialized and ready",
        Some(serde_json::json!({
            "name": "weather-server",
            "version": "1.0.0",
            "pid": std::process::id(),
            "transport": "http",
            "port": 8080
        }))
    ).await;

    let server = Server::builder()
        .name("weather-server")
        .version("1.0.0")
        .build()?;

    // Log shutdown
    pmcp::log(LogLevel::Info, "Server shutting down", None).await;

    Ok(())
}
```

- Never log sensitive data in production:

```text
                  Sensitive Data Categories
[BAD] NEVER LOG:
• API keys, tokens, secrets
• Passwords, password hashes
• Personal identifiable information (PII)
• Credit card numbers
• Session tokens, JWTs
• OAuth access/refresh tokens
• Database credentials
[GOOD] SAFE TO LOG:
• Request IDs, correlation IDs
• User IDs (if not considered PII)
• Timestamps, durations
• Error codes (not messages with user data)
• Operation types, method names
• Aggregate counts, statistics
```


### Redaction Patterns

```rust
use std::fmt;


/// Wrapper that redacts value in Display/Debug
pub struct Redacted<T>(pub T);

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

// Usage
async fn authenticate(token: &str) -> Result<User> {
    tracing::info!(
        token = %Redacted(token),  // Logs as "[REDACTED]"
        "Authentication attempt"
    );

    // Actual auth logic
    Ok(user)
}
```


### Automatic Redaction Middleware

```rust
use pmcp::server::http_middleware::ServerHttpLoggingMiddleware;

// HTTP middleware with automatic redaction
let logging = ServerHttpLoggingMiddleware::new()
    .with_level(tracing::Level::INFO)
    .with_redact_query(true);       // Strips ?token=xxx from URLs

// Automatically redacted headers:
// - Authorization
// - Cookie
// - X-Api-Key
```


### Field-Level Redaction

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
struct UserCredentials {
    username: String,
    #[serde(skip_serializing)]  // Never serialize password
    password: String,
}

// Custom Debug that redacts
#[derive(Serialize)]
struct ApiConfig {
    base_url: String,
    api_key: String,
}

impl fmt::Debug for ApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}
```

```rust
// Pretty, colored output for local development
tracing_subscriber::fmt()
    .pretty()
    .with_target(true)
    .with_level(true)
    .with_env_filter("debug")
    .init();

// Output:
// 2024-12-30T10:15:30.123Z DEBUG weather_server::tools
//   in get_weather{city="London"}
//   Weather retrieved
//     temp: 22.5
```


### Production: JSON

```rust
// Structured JSON for log aggregation
tracing_subscriber::fmt()
    .json()
    .with_current_span(true)
    .with_env_filter("info")
    .init();

// Output (single line):
// {"timestamp":"2024-12-30T10:15:30.123Z","level":"INFO",...}
```


### Multi-Output Configuration

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

fn init_logging() {
    // JSON logs to stdout for production systems
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_filter(tracing_subscriber::EnvFilter::new("info"));

    // Pretty logs to stderr for local debugging
    let pretty_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stderr)
        .with_filter(tracing_subscriber::EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(json_layer)
        .with(pretty_layer)
        .init();
}
```


### Cloud Platform Integration

```rust
// AWS CloudWatch format (JSON with specific fields)
use tracing_subscriber::fmt::format::JsonFields;

tracing_subscriber::fmt()
    .json()
    .flatten_event(true)
    .with_current_span(true)
    .init();

// Output compatible with CloudWatch Insights:
// {"level":"INFO","target":"weather_server","city":"London","message":"Weather retrieved"}
```

```rust
use anyhow::{Context, Result};
use tracing::error;

async fn fetch_weather(city: &str) -> Result<Weather> {
    let response = client
        .get(&format!("{}/weather/{}", base_url, city))
        .send()
        .await
        .context("Failed to send request to weather API")?;

    if !response.status().is_success() {
        error!(
            city = %city,
            status = %response.status(),
            "Weather API returned error"
        );
        return Err(anyhow::anyhow!("Weather API error: {}", response.status()));
    }

    response
        .json::<Weather>()
        .await
        .context("Failed to parse weather response")
}
```


### Error Chain Logging

```rust
fn log_error_chain(error: &anyhow::Error) {
    error!(error = %error, "Operation failed");

    // Log each cause in the chain
    for (i, cause) in error.chain().enumerate().skip(1) {
        error!(cause = %cause, depth = i, "Caused by");
    }
}

// Usage
if let Err(e) = process_request().await {
    log_error_chain(&e);
}

// Output:
// ERROR Operation failed: Failed to fetch weather
// ERROR Caused by: HTTP request failed | depth=1
// ERROR Caused by: connection refused | depth=2
```

```rust
// Set via environment variable:
// RUST_LOG=warn,pmcp=info,my_server=debug

tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```


### Per-Module Filtering

```rust
use tracing_subscriber::EnvFilter;

let filter = EnvFilter::new("")
    .add_directive("warn".parse().unwrap())           // Default: warn
    .add_directive("pmcp=info".parse().unwrap())      // PMCP: info
    .add_directive("my_server=debug".parse().unwrap()) // Our code: debug
    .add_directive("hyper=warn".parse().unwrap())     // HTTP: warn only
    .add_directive("sqlx=info".parse().unwrap());     // Database: info

tracing_subscriber::fmt()
    .with_env_filter(filter)
    .init();
```

```rust
use rand::Rng;

struct SamplingMiddleware {
    sample_rate: f64,  // 0.01 = 1% of requests
}

#[async_trait]
impl AdvancedMiddleware for SamplingMiddleware {
    async fn on_request_with_context(
        &self,
        request: &mut JSONRPCRequest,
        context: &MiddlewareContext,
    ) -> Result<()> {
        let should_sample = rand::thread_rng().gen::<f64>() < self.sample_rate;
        context.set_metadata(
            "sample".to_string(),
            should_sample.to_string()
        );

        if should_sample {
            tracing::debug!(
                method = %request.method,
                "Request sampled for detailed logging"
            );
        }

        Ok(())
    }
}
```

(table/list, cells in reading order)
- Summary
- Practice
- Implementation
- Structured logging
- Use  tracing  with JSON output
- Practice
- Implementation
- Contextual spans
- Use  #[instrument]  on handlers
- Log levels
- ERROR for failures, INFO for operations, DEBUG for diagnostics
- Sensitive data
- Use  Redacted<T>  wrapper,  #[serde(skip)]
- Error context
- Use  anyhow::Context , log error chains
- Cloud integration
- JSON format with CloudWatch/Datadog fields
- High traffic
- Sample logs, filter by module

The combination of  tracing  for Rust-side logging and PMCP's protocol logging provides comprehensive visibility into both server internals and client-facing operations.


### Metrics Collection

Metrics transform operations from reactive firefighting to proactive monitoring.

```text
                  Metrics vs Logs: When to Use Each
LOGS answer: "What happened?"
METRICS answer: "How much/how fast/how many?"
Scenario: Your MCP server is "slow"
Logs tell you:                    Metrics tell you:
"Request abc-123 took 5000ms"     Requests/second: 150
"Request def-456 took 3200ms"     P50 latency: 45ms
"Request ghi-789 took 4800ms"     P95 latency: 250ms
"Request jkl-012 took 50ms"       P99 latency: 4,800ms  ← Problem!
... (thousands more)              Error rate: 0.5%
To find the problem in logs:      To find the problem in metrics:
• Search through thousands        • Glance at dashboard
• Calculate averages manually     • See P99 spike immediately
• Hard to spot patterns           • Correlate with time
Use LOGS when you need:           Use METRICS when you need:
• Full context of an event        • Trends over time
• Debugging specific issues       • Alerting on thresholds
• Audit trails                    • Capacity planning
• Error messages                  • SLA monitoring
```

(table/list, cells in reading order)
- Why Metrics Matter
- Without Metrics
- With Metrics
- "Users say it's slow"
- "P95 latency increased from 100ms to 500ms at 2:30 PM"
- "Something is wrong"
- "Error rate jumped from 0.1% to 5% after the last deployment"
- "We need more capacity"
- "At current growth rate, we'll hit capacity limits in 3 weeks"
- "Is the fix working?"
- "Error rate dropped from 5% to 0.2% after the hotfix"
- The Three Types of Metrics

```text
                  The Three Metric Types
COUNTER
"How many times did X happen?"
• Only goes UP (or resets to 0)
• Like an odometer in a car
Examples:
• Total requests served              requests_total
• Total errors                       ████████████████ 1,523
• Total bytes transferred
                                     errors_total
Use when: You want to count          ██ 47
events that accumulate
GAUGE
"What is the current value of X?"
• Can go UP and DOWN
• Like a thermometer or fuel gauge
Examples:
• Active connections                 connections_active
• Queue depth                        ████████░░░░ 42
• Memory usage
• Temperature                        (can increase/decrease)
Use when: You want to track
current state that fluctuates
```

```text
HISTOGRAM
"What is the distribution of X?"
• Records many values, calculates percentiles
• Like tracking all marathon finish times, not just the average
Examples:
• Request latency                    request_duration_ms
• Response size
• Query execution time                ▂▅█▇▄▂▁
                                      10 50 100 200 500 ms
Use when: You need percentiles
(P50, P95, P99) not just averages     P50: 45ms  P99: 450ms
```


### Understanding Percentiles

Percentiles are crucial for understanding real user experience.

```text
                  Why Percentiles Matter
Scenario: 100 requests with these latencies:
• 90 requests: 50ms each
• 9 requests: 100ms each
• 1 request: 5,000ms (timeout!)
Average = (90×50 + 9×100 + 1×5000) / 100 = 104ms  ← "Looks fine!"
But look at percentiles:
• P50 (median) = 50ms    ← Half of users see 50ms or less
• P90 = 50ms             ← 90% of users see 50ms or less
• P95 = 100ms            ← 95% of users see 100ms or less
• P99 = 5,000ms          ← 1% of users wait 5 SECONDS! 🚨
Which percentile to monitor?
• P50 (median): Typical user experience
• P95: Most users' worst-case experience
• P99: Your "long tail" - affects 1 in 100 users
• P99.9: For high-traffic sites (1 in 1000 users)
If you have 1 million requests/day:
• P99 = 10,000 users having a bad experience daily
• P99.9 = 1,000 users having a bad experience daily
Rule of thumb: Alert on P95 or P99, not averages
```


### The Metrics Ecosystem

Rust's  metrics  crate provides a facade pattern similar to  log  for logging—you write metrics once and choose the backend at runtime:

```text
                  Metrics Architecture
Application Code
   counter!("requests_total").increment(1);
   histogram!("request_duration_ms").record(45.5);
   gauge!("active_connections").set(12);
                     metrics (facade crate)
  Prometheus          Datadog          CloudWatch
   Exporter            Agent             Agent
   Prometheus         Datadog             AWS
     Server            Cloud           CloudWatch
```

(table/list, cells in reading order)
- Metric Types
- Type
- Purpose
- Example
- Counter
- Monotonically increasing count
- Total requests, errors
- Gauge
- Value that can go up or down
- Active connections, queue depth
- Histogram
- Distribution of values
- Request duration, response size

```rust
use metrics::{counter, gauge, histogram};

async fn handler(input: Input) -> Result<Output> {
    let start = Instant::now();

    // Count the request
    counter!("mcp.requests_total", "tool" => "get-weather").increment(1);

    // Track active requests
    gauge!("mcp.requests_active").increment(1.0);

    let result = process(input).await;

    // Record duration
    histogram!("mcp.request_duration_ms", "tool" => "get-weather")
        .record(start.elapsed().as_millis() as f64);

    // Track active requests
    gauge!("mcp.requests_active").decrement(1.0);

    // Count success/failure
    match &result {
        Ok(_) => counter!("mcp.requests_success").increment(1),
        Err(_) => counter!("mcp.requests_error").increment(1),
    }

    result
}
```


### PMCP's Built-in Observability Metrics

PMCP v1.9.2+ includes a built-in observability module that automatically collects metrics without requiring manual middleware setup:

```rust
use pmcp::server::builder::ServerCoreBuilder;
use pmcp::server::observability::ObservabilityConfig;

// One line enables automatic metrics collection
let server = ServerCoreBuilder::new()
    .name("my-server")
    .version("1.0.0")
    .tool("weather", WeatherTool)
    .with_observability(ObservabilityConfig::development())
    .build()?;
```

(table/list, cells in reading order)
- Standard Metrics (Built-in)
- The built-in observability automatically emits these metrics:
- Metric
- Type
- Description
- mcp.request.duration
- Histogram (ms)
- Request latency per tool
- mcp.request.count
- mcp.request.errors
- Counter
- Counter
- Total requests processed
- Error count by type
- mcp.response.size
- Histogram (bytes)
- Response payload sizes
- mcp.composition.depth
- Gauge
- Nesting depth for composed servers


### Custom MetricsMiddleware (Advanced)

For custom metric backends (Prometheus, Datadog, etc.), you can still use the  MetricsMiddleware  directly:

```rust
use pmcp::shared::MetricsMiddleware;
use pmcp::shared::EnhancedMiddlewareChain;
use std::sync::Arc;

fn build_instrumented_chain() -> EnhancedMiddlewareChain {
    let mut chain = EnhancedMiddlewareChain::new();

    // Add metrics collection
    chain.add(Arc::new(MetricsMiddleware::new("my-server".to_string())));

    chain
}
```

(table/list, cells in reading order)
- Recorded Metrics (Custom MetricsMiddleware)
- The  MetricsMiddleware  automatically records:
- Metric
- Type
- Labels
- Description
- mcp.requests.total
- Counter
- service, method
- Total requests processed
- mcp.requests.duration_ms
- Histogram
- service, method
- Request latency
- mcp.requests.errors
- Counter
- service, error_type
- Error count by type
- mcp.requests.active
- Gauge
- service
- In-flight requests
- Custom Metrics in Handlers
- Add tool-specific metrics directly in handlers:

```rust
use metrics::{counter, histogram};
use std::time::Instant;

async fn handler(input: WeatherInput) -> Result<Weather> {
    let start = Instant::now();

    // Business metrics
    counter!(
        "weather.lookups_total",
        "city" => input.city.clone(),
        "units" => input.units.as_str()
    ).increment(1);

    let weather = match cache.get(&input.city) {
        Some(cached) => {
            counter!("weather.cache_hits").increment(1);
            cached
        }
        None => {
            counter!("weather.cache_misses").increment(1);
            let result = fetch_weather(&input.city).await?;

            histogram!("weather.api_latency_ms")
                .record(start.elapsed().as_millis() as f64);

            result
        }
    };

    // Track temperature extremes
    if weather.temperature > 40.0 {
        counter!("weather.extreme_heat_events").increment(1);
    }

    Ok(weather)
}
```

```rust
// Cargo.toml
[dependencies]
metrics = "0.23"
metrics-exporter-prometheus = "0.15"

// main.rs
use metrics_exporter_prometheus::PrometheusBuilder;

fn init_metrics() {
    // Start Prometheus exporter on port 9090
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()
        .expect("Failed to install Prometheus exporter");
}

#[tokio::main]
async fn main() {
    init_metrics();

    // Metrics now available at http://localhost:9090/metrics
    run_server().await;
}
```


### Prometheus output format:

```text
# HELP mcp_requests_total Total MCP requests
# TYPE mcp_requests_total counter
```


### mcp_requests_total{service="weather-server",method="get-weather"} 1523

```text
# HELP mcp_request_duration_ms Request latency in milliseconds
# TYPE mcp_request_duration_ms histogram
```

mcp_request_duration_ms_bucket{service="weather-server",le="10"} 450 mcp_request_duration_ms_bucket{service="weather-server",le="50"} 1200 mcp_request_duration_ms_bucket{service="weather-server",le="100"} 1500 mcp_request_duration_ms_bucket{service="weather-server",le="+Inf"} 1523 mcp_request_durati

```rust
// Cargo.toml
[dependencies]
metrics = "0.23"
metrics-exporter-statsd = "0.7"

// Using StatsD (Datadog agent listens on port 8125)
use metrics_exporter_statsd::StatsdBuilder;

fn init_metrics() {
    StatsdBuilder::from("127.0.0.1", 8125)
        .with_queue_size(5000)
        .with_buffer_size(1024)
        .install()
        .expect("Failed to install StatsD exporter");
}
```

```rust
    "mcp.requests",
    "service" => "weather-server",
    "tool" => "get-weather",
    "env" => "production"
).increment(1);

// Becomes: mcp.requests:1|c|#service:weather-server,tool:get-weather,env:production
```

```rust
// Cargo.toml
[dependencies]
metrics = "0.23"
aws-sdk-cloudwatch = "1.0"
tokio = { version = "1", features = ["full"] }

// Custom CloudWatch recorder
use aws_sdk_cloudwatch::{Client, types::MetricDatum, types::StandardUnit};
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, Unit};
use std::sync::Arc;

struct CloudWatchRecorder {
    client: Client,
    namespace: String,
}

impl CloudWatchRecorder {
    async fn new(namespace: &str) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        Self {
            client: Client::new(&config),
            namespace: namespace.to_string(),
        }
    }

    async fn publish_metrics(&self, metrics: Vec<MetricDatum>) {
        self.client
            .put_metric_data()
            .namespace(&self.namespace)
            .set_metric_data(Some(metrics))
            .send()
            .await
            .expect("Failed to publish metrics");
    }
}
```

```rust
// Cargo.toml
[dependencies]
opentelemetry = "0.24"
opentelemetry_sdk = "0.24"
opentelemetry-otlp = "0.17"
tracing-opentelemetry = "0.25"

use opentelemetry::global;
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_otlp::WithExportConfig;

fn init_otel_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("https://otlp.grafana.net:4317");

    let provider = MeterProvider::builder()
        .with_reader(
            opentelemetry_sdk::metrics::PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio)
                .with_interval(std::time::Duration::from_secs(30))
                .build()
        )
        .build();

    global::set_meter_provider(provider);
    Ok(())
}
```

```text
               Multi-Platform Metrics Design
                  Application Layer
Use metrics crate with consistent naming:
• mcp.requests.total
• mcp.requests.duration_ms
• mcp.requests.errors
                 Platform Adapter
Choose at deployment time via environment/config:
METRICS_BACKEND=prometheus  →  PrometheusBuilder
METRICS_BACKEND=datadog     →  StatsdBuilder
METRICS_BACKEND=cloudwatch  →  CloudWatchRecorder
METRICS_BACKEND=otlp        →  OpenTelemetry
```


### Platform Selection at Runtime

```rust
use std::env;

fn init_metrics_backend() {
    let backend = env::var("METRICS_BACKEND")
        .unwrap_or_else(|_| "prometheus".to_string());

    match backend.as_str() {
        "prometheus" => {
            metrics_exporter_prometheus::PrometheusBuilder::new()
                .with_http_listener(([0, 0, 0, 0], 9090))
                .install()
                .expect("Prometheus exporter failed");
        }
        "statsd" | "datadog" => {
            let host = env::var("STATSD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = env::var("STATSD_PORT")
                .unwrap_or_else(|_| "8125".to_string())
                .parse()
                .expect("Invalid STATSD_PORT");

            metrics_exporter_statsd::StatsdBuilder::from(&host, port)
                .install()
                .expect("StatsD exporter failed");
        }
        "none" | "disabled" => {
            // No-op for local development
            tracing::info!("Metrics collection disabled");
        }
        other => {
            panic!("Unknown metrics backend: {}", other);
        }
    }
}
```

```rust
// GOOD: Hierarchical, consistent naming
counter!("mcp.tool.requests_total", "tool" => "weather").increment(1);
histogram!("mcp.tool.duration_ms", "tool" => "weather").record(45.0);
counter!("mcp.tool.errors_total", "tool" => "weather", "error" => "timeout").increment(1);

// BAD: Inconsistent, flat naming
counter!("weather_requests").increment(1);
counter!("weatherToolDurationMs").increment(1);
counter!("errors").increment(1);
```


### Cardinality Control

Cardinality refers to the number of unique combinations of label values for a metric.

```text
                  The Cardinality Problem
What happens with high cardinality labels?
Each unique label combination creates a NEW time series in memory:
counter!("requests", "user_id" => user_id)
With 1 million users, this creates 1 MILLION time series:
requests{user_id="user-000001"} = 5
requests{user_id="user-000002"} = 12
requests{user_id="user-000003"} = 3
... (999,997 more) ...
requests{user_id="user-999999"} = 7
requests{user_id="user-1000000"} = 1
Each time series consumes memory in:
• Your application
• Prometheus/metrics backend
• Grafana/dashboard queries
Result: Memory exhaustion, slow queries, crashed monitoring
Good labels (bounded):              Bad labels (unbounded):
• tool: 10-50 tools max             • user_id: millions of users
• status: success/error             • request_id: infinite
• tier: free/pro/enterprise         • city: thousands of cities
• environment: dev/staging/prod     • email: unbounded
• http_method: GET/POST/PUT/DELETE  • timestamp: infinite
Rule of thumb: Labels should have fewer than 100 possible values
```

If you need per-user or per-request data, use logs instead of metrics.

```rust
// BAD: Unbounded cardinality (user_id could be millions)
counter!("requests", "user_id" => user_id).increment(1);

// BAD: High cardinality (city names - thousands of values)
counter!("weather_requests", "city" => &input.city).increment(1);

// GOOD: Bounded cardinality (only 3 possible values)
counter!(
    "requests",
    "user_tier" => user.tier.as_str()  // "free", "pro", "enterprise"
).increment(1);

// GOOD: Use histogram for distribution instead of labels
histogram!("request_duration_ms").record(duration);

// GOOD: Log high-cardinality data instead of metrics
tracing::info!(user_id = %user_id, city = %city, "Request processed");
```

```rust
use std::sync::OnceLock;

struct MetricsContext {
    service: String,
    version: String,
    environment: String,
}

static CONTEXT: OnceLock<MetricsContext> = OnceLock::new();

fn init_context() {
    CONTEXT.get_or_init(|| MetricsContext {
        service: env::var("SERVICE_NAME").unwrap_or_else(|_| "mcp-server".to_string()),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: env::var("ENV").unwrap_or_else(|_| "development".to_string()),
    });
}

// Helper for consistent labeling
macro_rules! labeled_counter {
    ($name:expr, $($key:expr => $value:expr),*) => {{
        let ctx = CONTEXT.get().expect("Metrics context not initialized");
        counter!(
            $name,
            "service" => ctx.service.clone(),
            "version" => ctx.version.clone(),
            "env" => ctx.environment.clone(),
            $($key => $value),*
        )
    }};
}

// Usage
labeled_counter!("mcp.requests", "tool" => "weather").increment(1);
```

```yaml
# Grafana dashboard panels (pseudo-config)
panels:
  - title: "Request Rate"
    query: rate(mcp_requests_total[5m])
    type: graph

  - title: "P95 Latency"
    query: histogram_quantile(0.95, rate(mcp_request_duration_ms_bucket[5m]))
    type: graph

  - title: "Error Rate"
    query: rate(mcp_requests_errors_total[5m]) / rate(mcp_requests_total[5m])
    type: gauge
    thresholds:
      - value: 0.01
        color: yellow
      - value: 0.05
        color: red

  - title: "Active Connections"
    query: mcp_connections_active
    type: stat
```


### Alert Rules

```yaml
# Prometheus alerting rules
groups:
  - name: mcp-server
    rules:
      - alert: HighErrorRate
        expr: rate(mcp_requests_errors_total[5m]) / rate(mcp_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "MCP server error rate above 5%"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(mcp_request_duration_ms_bucket[5m])) > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "MCP server P95 latency above 1 second"

      - alert: ServiceDown
        expr: up{job="mcp-server"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "MCP server is down"
```

```yaml
# scenarios/smoke.yaml
name: "Smoke Test with Metrics Verification"
steps:
  - name: "Call weather tool"
    operation:
      type: tool_call
      tool: "get-weather"
      arguments:
        city: "London"
    assertions:
      - type: success
      - type: duration_ms
        max: 1000

  # Verify metrics endpoint
  - name: "Check metrics"
    operation:
      type: http_get
      url: "http://localhost:9090/metrics"
    assertions:
      - type: contains
        value: "mcp_requests_total"
      - type: contains
        value: 'tool="get-weather"'
```


### Metrics in CI/CD

```bash
# .github/workflows/test.yml
jobs:
  test:
    steps:
      - name: Start server
        run: cargo run --release &
        env:
          METRICS_BACKEND: prometheus

      - name: Wait for startup
        run: sleep 5

      - name: Run tests
        run: cargo pmcp test --server weather

      - name: Verify metrics
        run: |
          curl -s http://localhost:9090/metrics | grep mcp_requests_total
          curl -s http://localhost:9090/metrics | grep mcp_request_duration_ms
```

(table/list, cells in reading order)
- Summary
- Aspect
- Crate
- Types
- Naming
- Labels
- Recommendation
- Use  metrics  facade for portability
- Counter (totals), Histogram (durations), Gauge (current state)
- Hierarchical:  mcp.component.metric_name
- Service, tool, environment; avoid high cardinality
- Platform
- Configure at runtime via environment variables
- Aspect
- Recommendation
- Prometheus
- Default for cloud-native, excellent Grafana support
- Datadog
- StatsD exporter, good for existing Datadog users
- CloudWatch
- Custom recorder for AWS-native deployments
- Alerting
- Error rate > 5%, P95 latency > 1s, service down

Metrics provide the quantitative foundation for understanding system behavior.
