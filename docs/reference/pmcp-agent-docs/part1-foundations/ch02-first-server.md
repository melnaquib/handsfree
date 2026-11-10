---
chapter: 2
title: "Your First Production Server"
part: "Part I: Foundations"
use_when: "Scaffolding a first server, running it locally, connecting Claude Code, testing with MCP Inspector."
suggested_prerequisites: [ch00]
source_kb: 44
extracted_kb: 28
code_blocks: 93
quality: auto-extracted
---

# Ch 2: Your First Production Server

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

## Agent notes (checked against extracted text)

- Quickstart (verified in source):
  1. `cargo install cargo-pmcp`
  2. `cargo pmcp new my-mcp-servers && cd my-mcp-servers`
  3. `cargo pmcp add server calculator --template calculator`
  4. `cargo pmcp dev calculator` (serves HTTP on port 3000)
  5. `claude mcp add calculator -t http http://0.0.0.0:3000`
  6. `npx @modelcontextprotocol/inspector http://localhost:3000/mcp`
- Generated layout: `Cargo.toml`, `pmcp.toml`, `server-common/`, `servers/<name>/src/{main.rs,tools/}`.
- CLI subcommands listed: `new`, `add`, `dev`, `test`, `deploy`.

## Extracted content


### Your First Production Server

Prerequisites: Make sure you've completed the Development Environment Setup before continuing.

```bash
cargo pmcp new my-mcp-servers
```


### cd my-mcp-servers

This creates a workspace structure for building MCP servers.


### Step 2: Add a Calculator Server

```bash
cargo pmcp add server calculator --template calculator
```

This generates a complete, working MCP server with example tools.


### Step 3: Build and Run

```bash
cargo pmcp dev calculator
```


### You should see:

INFO Starting MCP server "calculator" v1.0.0 INFO Listening on http://0.0.0.0:3000

```bash
claude mcp add calculator -t http http://0.0.0.0:3000
```


### Try a few more:

"Calculate 100 divided by 7" "What's 15 times 23?" "Divide 10 by 0" (watch the error handling)

(table/list, cells in reading order)
- In those 5 steps, you created a production-ready MCP server that:
- Feature
- What It Does
- Type-safe inputs
- Invalid inputs are rejected automatically
- Structured outputs
- Results include both values and descriptions
- Error handling
- Division by zero returns a proper error, not a crash
- JSON Schema
- Claude knows exactly what parameters each tool accepts
- HTTP transport
- Ready for cloud deployment


### Testing with MCP Inspector

Before connecting to Claude, you can test your server interactively using MCP Inspector:

```bash
npx @modelcontextprotocol/inspector http://localhost:3000/mcp
```

```text
Cargo.toml              # Workspace manifest
pmcp.toml               # PMCP configuration
server-common/          # Shared HTTP bootstrap code
    Cargo.toml
    src/lib.rs
servers/
    calculator/         # Your calculator server
        Cargo.toml
        src/
            main.rs     # Entry point
            tools/
                mod.rs
                calculator.rs
```


### Your Turn: Build Your First Server

You've seen the calculator server in action.


### Development Environment Setup

Before building your first MCP server, let's set up your development environment.

```yaml
things:
```

- If you don't have Rust installed, run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
# Should output: rustc 1.82.0 or later

cargo --version
# Should output: cargo 1.82.0 or later
```

```bash
cargo install cargo-pmcp
```

```bash
cargo pmcp new
```


### Create a new MCP workspace

```bash
cargo pmcp add
```


### Add servers and tools to your workspace

```bash
cargo pmcp dev
```


### Run a server in development mode

```bash
cargo pmcp test
```


### Run MCP-specific tests

```bash
cargo pmcp deploy
```

```bash
cargo pmcp --version
```


### Choosing an MCP Client

MCP servers need a client to connect to.

(table/list, cells in reading order)
- Client
- Best For
- MCP Support
- Claude Code
- Terminal-based development, CLI workflows
- Excellent
- Cursor
- AI-assisted coding in VS Code fork
- Good
- Client
- Best For
- MCP Support
- Gemini Code Assist
- Google Cloud integrated development
- Cline
- Kiro
- VS Code extension for AI coding
- AWS-focused agentic IDE
- Codex CLI
- OpenAI's terminal assistant
- Good
- Good
- Good
- Basic

For this course, we recommend Claude Code.

```bash
curl -fsSL https://claude.ai/install.sh | bash
```

```bash
claude --version
```


### Adding MCP Servers to Claude Code

Once your MCP server is running, you can add it to Claude Code with a single command:

```bash
claude mcp add <server-name> -t http <server-url>
```


### For example:

```bash
claude mcp add calculator -t http http://localhost:3000
```


### You can list your configured servers:

```bash
claude mcp list
```


### And remove servers you no longer need:

```bash
claude mcp remove calculator
```


### MCP Inspector (Optional)

MCP Inspector is a debugging tool that lets you interact with MCP servers directly, without going through an AI client.


### No installation needed—just run with npx:

This opens a web UI where you can browse tools, call them with test inputs, and see the raw JSON- RPC messages.

(table/list, cells in reading order)
- Configuring Your IDE
- For writing Rust code, configure your preferred IDE:
- VS Code
- Install these extensions:


### Cursor

Cursor includes rust-analyzer support.


### RustRover

JetBrains RustRover works out of the box with Rust projects—no additional configuration needed.


### Zed

Zed has built-in Rust support with excellent performance.

```text
# Check Rust
```


### rustc --version && cargo --version

```bash
# Check cargo-pmcp
cargo pmcp --version

# Check Claude Code
claude --version
```

If all three commands succeed, you're ready to build your first MCP server!


### Building and Running

Now that you've seen the quick start, let's understand what  cargo pmcp  created and how to work with it effectively.


### The Workspace Structure

When you ran  cargo pmcp new my-mcp-servers , it created a Cargo workspace:


### my-mcp-servers/

```text
Cargo.toml              # Workspace manifest
pmcp.toml               # PMCP configuration
server-common/          # Shared infrastructure code
    Cargo.toml
    src/
        lib.rs
servers/                # Your MCP servers live here
    calculator/
        Cargo.toml
        src/
            main.rs
            tools/
                mod.rs
                calculator.rs
```

Why a Workspace?

```yaml
provides:
```

```bash
cargo build  compiles everything together
```

```toml
[workspace]
resolver = "2"
members = [
    "server-common",
    "servers/*",
]

[workspace.dependencies]
pmcp = "1.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
async-trait = "0.1"
```

(table/list, cells in reading order)
- Key points:
- members  includes  server-common  and all packages under  servers/
- [workspace.dependencies]  defines shared dependency versions
- Individual packages inherit these with  dependency.workspace = true
- The PMCP Configuration
- The  pmcp.toml  file configures cargo-pmcp behavior:

```toml
[workspace]
name = "my-mcp-servers"
default_server = "calculator"

[servers.calculator]
package = "calculator"
port = 3000

[deploy]
default_target = "lambda"
```


### Server-Common: Shared Infrastructure

The  server-common  crate provides HTTP server bootstrap code that all your MCP servers share:

```rust
// server-common/src/lib.rs
use pmcp::server::streamable_http_server::{
    StreamableHttpServer,
    StreamableHttpServerConfig
};
use pmcp::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Start an HTTP server for the given MCP server
pub async fn serve_http(
    server: Server,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(Mutex::new(server));

    let config = StreamableHttpServerConfig {
        session_id_generator: None,   // Stateless mode
        enable_json_response: true,
        event_store: None,
        on_session_initialized: None,
        on_session_closed: None,
        http_middleware: None,
    };

    let http_server = StreamableHttpServer::with_config(addr, server, config);
    let (bound_addr, handle) = http_server.start().await?;

    tracing::info!("MCP server listening on http://{}/mcp", bound_addr);

    handle.await?;
    Ok(())
}
```

```bash
# Run the default server (from pmcp.toml)
cargo pmcp dev

# Run a specific server
cargo pmcp dev calculator

# Run on a different port
cargo pmcp dev calculator --port 8080
```

(table/list, cells in reading order)
- Development mode includes:
- Hot reloading (rebuilds on file changes)
- Verbose logging Pretty-printed output
- Production Build
- For production, build a release binary:

```bash
cargo build --release --package calculator
```

(table/list, cells in reading order)
- Run it directly:
- ./target/release/calculator
- Or with environment configuration:
- RUST_LOG=info PORT=3000 ./target/release/calculator
- Adding More Servers
- Add a new server to your workspace:

```bash
cargo pmcp add server inventory --template basic
```


### servers/

```text
calculator/
inventory/
```

Both servers share  server-common  and workspace dependencies.

(table/list, cells in reading order)
- Available Templates
- cargo pmcp add server  supports several templates:
- Template
- Description
- basic
- Minimal server with one example tool
- calculator
- Math operations with typed inputs/outputs
- database
- crud
- Database query patterns with connection pooling
- Create/Read/Update/Delete operations
- authenticated
- OAuth-protected server template
- Use  --template  to specify:

```bash
cargo pmcp add server users --template crud
cargo pmcp add server reports --template database
```

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building (faster)
cargo check
```

```bash
# All tests
cargo test

# Tests for a specific server
cargo test --package calculator

# With output
cargo test -- --nocapture
```

```bash
# Format all code
cargo fmt

# Lint all code
cargo clippy

# Both (recommended before commits)
cargo fmt && cargo clippy
```

```bash
cargo pmcp new <name>
```


### Create a new workspace

```bash
cargo pmcp add server <name>
```


### Add a server to the workspace

```bash
cargo pmcp dev [server]
```


### Run in development mode

```bash
cargo build --release
```


### Build for production

```bash
cargo test
```


### Run all tests

```bash
cargo fmt && cargo clippy
```


### Code quality checks

Next, let's look inside the calculator server to understand how tools are defined.


### The Calculator Server

Let's examine the calculator server in detail.

```rust
// servers/calculator/src/main.rs
use pmcp::prelude::*;
use server_common::serve_http;
use std::net::{Ipv4Addr, SocketAddr};

mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Build the MCP server
    let server = Server::builder()
        .name("calculator")
        .version("1.0.0")
        .capabilities(ServerCapabilities::tools_only())
        .tool("add", tools::AddTool)
        .tool("subtract", tools::SubtractTool)
        .tool("multiply", tools::MultiplyTool)
        .tool("divide", tools::DivideTool)
        .build()?;

    // Start HTTP server
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 3000);
    tracing::info!("Starting calculator server");

    serve_http(server, addr).await
}
```

(table/list, cells in reading order)
- Key elements:
- Line
- Purpose
- use pmcp::prelude::*
- Imports common types (Server, ServerCapabilities, etc.)
- Line
- Purpose

```text
mod tools
```


### Includes the tools module

```rust
#[tokio::main]
```


### Enables async main function

```rust
Server::builder()
```


### Fluent API for server configuration

```text
.tool("name", Handler)
```


### Registers each tool

```text
serve_http(server, addr)
```

(table/list, cells in reading order)
- Starts the HTTP transport
- Tool Module Structure
- Tools are organized in the  tools/  directory:
- src/tools/

```text
mod.rs          # Module exports
calculator.rs   # Tool implementations
```


### The  mod.rs  file exports the tool handlers:

```rust
// src/tools/mod.rs
mod calculator;

pub use calculator::{AddTool, SubtractTool, MultiplyTool, DivideTool};
```

```rust
// src/tools/calculator.rs
use async_trait::async_trait;
use pmcp::{ToolHandler, RequestHandlerExtra, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Input arguments for the add operation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddArgs {
    /// First number to add
    pub a: f64,
    /// Second number to add
    pub b: f64,
}

/// Result of the add operation
#[derive(Debug, Serialize, JsonSchema)]
pub struct AddResult {
    /// The sum of a and b
    pub result: f64,
    /// Human-readable expression
    pub expression: String,
}

/// Tool that adds two numbers
pub struct AddTool;

#[async_trait]
impl ToolHandler for AddTool {
    async fn handle(
        &self,
        args: Value,
        _extra: RequestHandlerExtra
    ) -> Result<Value, Error> {
        // Parse and validate arguments
        let input: AddArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        // Perform the calculation
        let sum = input.a + input.b;

        // Return structured result
        let result = AddResult {
            result: sum,
            expression: format!("{} + {} = {}", input.a, input.b, sum),
        };

        Ok(serde_json::to_value(result)?)
    }

    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        let schema = schemars::schema_for!(AddArgs);
        Some(pmcp::types::ToolInfo::new(
            "add",
            Some("Add two numbers together".to_string()),
            serde_json::to_value(&schema).unwrap_or_default(),
        ))
    }
}
```

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddArgs {
    /// First number to add
    pub a: f64,
    /// Second number to add
    pub b: f64,
}
```


### Deserialize  - Parses JSON into this struct

JsonSchema  - Generates JSON Schema for validation Doc comments ( /// ) become field descriptions in the schema

```json
{
  "type": "object",
  "properties": {
    "a": { "type": "number", "description": "First number to add" },
    "b": { "type": "number", "description": "Second number to add" }
  },
  "required": ["a", "b"]
}
```


### 2. Output Type

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct AddResult {
    pub result: f64,
    pub expression: String,
}
```

```rust
#[async_trait]
impl ToolHandler for AddTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value,
Error> {
        // Implementation
    }
}
```

_extra: RequestHandlerExtra  - Additional context (we'll use this later)

```rust
fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
    let schema = schemars::schema_for!(AddArgs);
    Some(pmcp::types::ToolInfo::new(
        "add",
        Some("Add two numbers together".to_string()),
        serde_json::to_value(&schema).unwrap_or_default(),
    ))
}
```

(table/list, cells in reading order)
- This tells MCP clients:
- Tool name:  "add"
- Description:  "Add two numbers together"
- Input schema: Generated from  AddArgs
- Error Handling: The Divide Tool
- The divide tool shows proper error handling:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DivideArgs {
    /// The dividend (number to be divided)
    pub dividend: f64,
    /// The divisor (number to divide by)
    pub divisor: f64,
}

pub struct DivideTool;

#[async_trait]
impl ToolHandler for DivideTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value,
Error> {
        let input: DivideArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        // Validate: prevent division by zero
        if input.divisor == 0.0 {
            return Err(Error::validation("Cannot divide by zero"));
        }

        let quotient = input.dividend / input.divisor;

        Ok(json!({
            "result": quotient,
            "expression": format!("{} ÷ {} = {}", input.dividend, input.divisor,
quotient)
        }))
    }

    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        let schema = schemars::schema_for!(DivideArgs);
        Some(pmcp::types::ToolInfo::new(
            "divide",
            Some("Divide two numbers. Returns an error if divisor is
zero.".to_string()),
            serde_json::to_value(&schema).unwrap_or_default(),
        ))

    }
}
```

(table/list, cells in reading order)
- Error Types
- PMCP provides error types that map to MCP error codes:
- Error Type
- When to Use
- MCP Code

```rust
Error::validation(msg)
```

```rust
Error::internal(msg)
```


### Server-side failures

```rust
Error::not_found(msg)
```


### Resource doesn't exist

```rust
Error::permission_denied(msg)
```

(table/list, cells in reading order)
- Authorization failure
- -32603
- -32001
- -32002

When Claude sees a validation error, it understands the request was malformed and can try again with corrected input.

```rust
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, ToolHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// === Shared Types ===

#[derive(Debug, Serialize, JsonSchema)]
pub struct CalculationResult {
    pub result: f64,
    pub expression: String,
}

// === Add Tool ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddArgs {
    /// First number
    pub a: f64,
    /// Second number
    pub b: f64,
}

pub struct AddTool;

#[async_trait]
impl ToolHandler for AddTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value,
Error> {
        let input: AddArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;

        let result = input.a + input.b;
        Ok(serde_json::to_value(CalculationResult {
            result,
            expression: format!("{} + {} = {}", input.a, input.b, result),
        })?)
    }

    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        let schema = schemars::schema_for!(AddArgs);
        Some(pmcp::types::ToolInfo::new(
            "add",
            Some("Add two numbers".to_string()),
            serde_json::to_value(&schema).unwrap_or_default(),
        ))
    }
}

// === Subtract Tool ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubtractArgs {
    /// Number to subtract from
    pub a: f64,
    /// Number to subtract
    pub b: f64,
}

// ... truncated, 105 more lines in source book ...
```

```json
{
  "tools": [
    {
      "name": "add",
      "description": "Add two numbers",
      "inputSchema": {
        "type": "object",
        "properties": {
          "a": { "type": "number", "description": "First number" },
          "b": { "type": "number", "description": "Second number" }
        },
        "required": ["a", "b"]
      }
    },
    {
      "name": "divide",
      "description": "Divide two numbers (divisor cannot be zero)",
      "inputSchema": {
        "type": "object",
        "properties": {
          "dividend": { "type": "number", "description": "The dividend" },
          "divisor": { "type": "number", "description": "The divisor (cannot be zero)"
}
        },
        "required": ["dividend", "divisor"]
      }
    }
  ]
}
```


### Claude uses this information to:

1. Know what arguments each tool requires


### Understanding the Generated Code

Now that you've seen the calculator server, let's understand the patterns and conventions that make PMCP code production-ready.

```rust
use pmcp::prelude::*;
```

(table/list, cells in reading order)
- This imports commonly used types:
- Type
- Purpose
- Server
- The MCP server instance
- ServerBuilder
- Fluent API for building servers
- ServerCapabilities
- Declares what the server supports
- ToolHandler
- Trait for implementing tools
- RequestHandlerExtra
- Additional context for handlers
- Error
- PMCP error types
- You can also import types explicitly:

```rust
use pmcp::{Server, ServerBuilder, ServerCapabilities, ToolHandler, Error};
```


### Server Builder Pattern

The  ServerBuilder  uses the builder pattern for flexible configuration:

```rust
let server = Server::builder()
    .name("my-server")           // Required: server name
    .version("1.0.0")            // Required: semantic version
    .capabilities(caps)          // Required: what the server supports
    .tool("tool_name", handler)  // Add tools
    .resource("uri", provider)   // Add resources
    .prompt("name", template)    // Add prompts
    .build()?;                   // Finalize and validate
```

```rust
// Only tools
let caps = ServerCapabilities::tools_only();

// Only resources
let caps = ServerCapabilities::resources_only();

// Tools and resources
let caps = ServerCapabilities {
    tools: Some(pmcp::types::ToolCapabilities::default()),
    resources: Some(pmcp::types::ResourceCapabilities::default()),
    ..Default::default()
};

// Everything
let caps = ServerCapabilities::all();
```

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Handle a tool invocation
    async fn handle(
        &self,
        args: Value,
        extra: RequestHandlerExtra,
    ) -> Result<Value, Error>;

    /// Return tool metadata (name, description, schema)
    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        None  // Default: no metadata
    }
}
```

```rust
gap:

use async_trait::async_trait;

#[async_trait]
impl ToolHandler for MyTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value,
Error> {
        // Can use .await here
        let data = fetch_data().await?;
        Ok(json!({ "data": data }))
    }
}
```

```rust
async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value,
Error> {
    // Access request metadata
    if let Some(meta) = &extra.meta {
        tracing::info!("Request ID: {:?}", meta.progress_token);
    }

    // ... handle request
}
```

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolArgs {
    pub required_field: String,

    #[serde(default)]
    pub optional_field: Option<i32>,

    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 { 10 }
```

```rust
#[serde(default)]
```


### Use  Default::default()  if missing

```rust
#[serde(default = "fn")]
```


### Use custom default function

```rust
#[serde(rename = "name")]
```


### Use different JSON field name

```rust
#[serde(skip)]
```


### Don't serialize/deserialize

```rust
#[serde(flatten)]
```

- Always parse with proper error handling:

```rust
let input: MyToolArgs = serde_json::from_value(args)
    .map_err(|e| Error::validation(format!("Invalid arguments: {}", e)))?;
```

This converts parsing errors into MCP validation errors that clients understand.

```rust
use schemars::JsonSchema;

#[derive(JsonSchema)]
pub struct SearchArgs {
    /// The search query string
    pub query: String,

    /// Maximum results to return (1-100)
    #[schemars(range(min = 1, max = 100))]
    pub limit: u32,

    /// Filter by status
    pub status: Option<Status>,
}

#[derive(JsonSchema)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```


### Generated schema:

```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "The search query string"
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 100,
      "description": "Maximum results to return (1-100)"
    },
    "status": {
      "type": "string",
      "enum": ["Active", "Inactive", "Pending"],
      "description": "Filter by status"
    }
  },
  "required": ["query", "limit"]
}
```

```text
/// comment
```


### Becomes  description

```rust
#[schemars(range(min, max))]
```


### Adds numeric bounds

```rust
#[schemars(length(min, max))]
```


### Adds string length bounds

```rust
#[schemars(regex(pattern))]
```

```rust
// Missing required field
if input.query.is_empty() {
    return Err(Error::validation("Query cannot be empty"));
}

// Invalid value
if input.limit > 100 {
    return Err(Error::validation("Limit cannot exceed 100"));
}

// Invalid format
if !input.email.contains('@') {
    return Err(Error::validation("Invalid email format"));
}
```


### Internal Errors (Server's Fault)

```rust
// Database failure
let result = db.query(&sql).await
    .map_err(|e| Error::internal(format!("Database error: {}", e)))?;

// External service failure
let response = client.get(url).await
    .map_err(|e| Error::internal(format!("API error: {}", e)))?;
```


### Resource Errors

```rust
// Not found
let user = db.find_user(id).await?
    .ok_or_else(|| Error::not_found(format!("User {} not found", id)))?;

// Permission denied
if !user.can_access(resource) {
    return Err(Error::permission_denied("Access denied"));
}
```

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self, extra))]
async fn handle(&self, args: Value, extra: RequestHandlerExtra) -> Result<Value,
Error> {
    info!(tool = "my_tool", "Processing request");

    let input: MyArgs = serde_json::from_value(args)?;
    debug!(query = %input.query, "Parsed arguments");

    match do_work(&input).await {
        Ok(result) => {
            info!(result_count = result.len(), "Request completed");
            Ok(serde_json::to_value(result)?)
        }
        Err(e) => {
            error!(error = %e, "Request failed");
            Err(Error::internal(e.to_string()))
        }
    }
}
```

error!

(table/list, cells in reading order)
- Use For
- Failures that need attention
- Unexpected but handled situations
- Normal operational messages
- Detailed debugging info
- Very verbose debugging
- The #[instrument] Macro
- Automatically creates a span with function arguments:

```rust
#[instrument(skip(db), fields(user_id = %user_id))]
async fn get_user(db: &Database, user_id: i64) -> Result<User, Error> {
    // Logs: get_user{user_id=123}
    db.find(user_id).await
}
```

```rust
let user = db.get_user(user_id).await?;
let orders = db.get_orders(user_id).await?;
let total = calculate_total(&orders);
```


### Parallel Operations

```rust
use tokio::try_join;
```


### let (user, orders, preferences) = try_join!(

```text
    db.get_user(user_id),
    db.get_orders(user_id),
    db.get_preferences(user_id),
)?;
```


### Timeout Handling

```rust
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(5), slow_operation())
    .await
    .map_err(|_| Error::internal("Operation timed out"))??;
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_tool() {
        let tool = AddTool;
        let args = json!({ "a": 10.0, "b": 5.0 });
        let extra = RequestHandlerExtra::default();

        let result = tool.handle(args, extra).await.unwrap();

        assert_eq!(result["result"], 15.0);
        assert_eq!(result["expression"], "10 + 5 = 15");
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let tool = DivideTool;
        let args = json!({ "dividend": 10.0, "divisor": 0.0 });
        let extra = RequestHandlerExtra::default();

        let result = tool.handle(args, extra).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("divide by zero"));
    }
}
```


### Testing Schema Generation

```rust
#[test]
fn test_args_schema() {
    let schema = schemars::schema_for!(AddArgs);
    let json = serde_json::to_value(&schema).unwrap();

    assert!(json["properties"]["a"].is_object());
    assert!(json["properties"]["b"].is_object());
    assert!(json["required"].as_array().unwrap().contains(&json!("a")));
}
```

2.

```rust
// 1. Input type
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolArgs { /* ... */ }

// 2. Output type
#[derive(Debug, Serialize, JsonSchema)]
pub struct MyToolResult { /* ... */ }

// 3. Handler implementation
pub struct MyTool;

#[async_trait]
impl ToolHandler for MyTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value,
Error> {
        let input: MyToolArgs = serde_json::from_value(args)
            .map_err(|e| Error::validation(e.to_string()))?;

        // Business logic here

        Ok(serde_json::to_value(result)?)
    }

    // 4. Metadata
    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        let schema = schemars::schema_for!(MyToolArgs);
        Some(pmcp::types::ToolInfo::new(
            "my_tool",
            Some("Description here".to_string()),
            serde_json::to_value(&schema).unwrap_or_default(),
        ))
    }
}

// 5. Registration
let server = Server::builder()

    .tool("my_tool", MyTool)
    .build()?;
```


### Hands-On Exercise: Code Review

Now that you understand the patterns, practice your code review skills with a hands-on exercise.
