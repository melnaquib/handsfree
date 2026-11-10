---
chapter: 17
title: "Local Testing"
part: "Part IV: Testing"
use_when: "Unit/property tests, MCP Inspector, mcp-tester, schema-driven test generation."
suggested_prerequisites: [ch02]
source_kb: 125
extracted_kb: 76
code_blocks: 117
quality: auto-extracted
---

# Ch 17: Local Testing

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Local Testing

Testing is what separates professional MCP servers from demos.


### The Testing Pyramid for MCP Servers

The testing pyramid is a mental model for balancing different types of tests.


### For MCP servers, this translates to:

Unit tests (base): Test your tool logic in isolation—fast, reliable, catch logic bugs Integration tests (middle): Test MCP protocol interactions—catch format and schema bugs E2E tests (top): Test with real clients—catch deployment and configuration bugs

```text
                  MCP Testing Pyramid
                       /  E2E     \         MCP Inspector
                      /  Testing   \        Claude Desktop
                     /              \
                    /   mcp-tester   \       Scenario files
                   /   Integration    \      API testing
                  /                    \
                 /    Rust Unit Tests   \    Tool logic
                /   Property Tests       \   Input validation
               /                          \
More tests at base, fewer at top
Base runs fastest, top runs slowest
```


### Rust Unit Tests

Before testing MCP protocol interactions, test your core tool logic with standard Rust tests. Unit tests are your first line of defense—they run in milliseconds, don't require a running server, and catch bugs at the source.


### Why unit test first:

Fast feedback loop (run in <1 second) Precise error location (the failing test points to the broken function) Easy to test edge cases (no network or database setup) Serve as documentation (tests show how functions should be used)


### Testing Tool Logic

Start by testing the pure functions that implement your tool's business logic. These functions should be independent of the MCP protocol.

```rust
// src/tools/calculator.rs
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, CalculatorError> {
    if b == 0.0 {
        return Err(CalculatorError::DivisionByZero);
    }
    Ok(a / b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive_numbers() {
        assert_eq!(add(2.0, 3.0), 5.0);
    }

    #[test]
    fn test_add_negative_numbers() {
        assert_eq!(add(-2.0, -3.0), -5.0);
    }

    #[test]
    fn test_add_mixed_signs() {
        assert_eq!(add(-2.0, 3.0), 1.0);
    }

    #[test]
    fn test_divide_normal() {
        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(matches!(

            divide(10.0, 0.0),
            Err(CalculatorError::DivisionByZero)
        ));
    }

    #[test]
    fn test_divide_zero_numerator() {
        assert_eq!(divide(0.0, 5.0).unwrap(), 0.0);
    }
}
```


### Testing Input Validation

Input validation is critical for MCP servers—bad input can cause crashes, security vulnerabilities, or confusing errors. Test your validation logic thoroughly: valid inputs should pass, invalid inputs should fail with helpful messages.

```rust
// src/tools/query.rs
use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Only SELECT queries are allowed")]
    NonSelectQuery,
    #[error("Limit must be between 1 and 1000, got {0}")]
    InvalidLimit(i32),
    #[error("Query cannot be empty")]
    EmptyQuery,
}

pub fn validate_query(query: &str, limit: Option<i32>) -> Result<(), QueryError> {
    if query.trim().is_empty() {
        return Err(QueryError::EmptyQuery);
    }

    let select_pattern = Regex::new(r"(?i)^\s*SELECT\b").unwrap();
    if !select_pattern.is_match(query) {
        return Err(QueryError::NonSelectQuery);
    }

    if let Some(l) = limit {
        if l < 1 || l > 1000 {
            return Err(QueryError::InvalidLimit(l));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select_query() {
        assert!(validate_query("SELECT * FROM users", Some(100)).is_ok());

    }


    #[test]
    fn test_select_case_insensitive() {
        assert!(validate_query("select * from users", None).is_ok());
        assert!(validate_query("Select id From users", None).is_ok());
    }

    #[test]
    fn test_rejects_insert() {
        assert!(matches!(
            validate_query("INSERT INTO users VALUES (1)", None),
            Err(QueryError::NonSelectQuery)
        ));
    }

    #[test]
    fn test_rejects_drop() {
        assert!(matches!(
// ... truncated, 32 more lines in source book ...
```


### Testing MCP Response Formatting

MCP has specific requirements for response format.

```rust
// src/mcp/response.rs
use serde_json::{json, Value};

pub fn format_tool_result(data: impl serde::Serialize) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&data).unwrap_or_default()
        }]
    })
}

pub fn format_error_result(message: &str) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": format!("Error: {}", message)
        }],
        "isError": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tool_result_with_struct() {
        #[derive(serde::Serialize)]
        struct QueryResult {
            rows: Vec<String>,
            count: usize,
        }

        let result = QueryResult {
            rows: vec!["row1".to_string()],
            count: 1,
        };

        let formatted = format_tool_result(result);

        assert_eq!(formatted["content"][0]["type"], "text");
        let text = formatted["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("\"count\": 1"));
    }

    #[test]
    fn test_format_error_result() {
        let formatted = format_error_result("Division by zero");

        assert_eq!(formatted["isError"], true);
        let text = formatted["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Division by zero"));
    }
}
```


### Property-Based Testing with proptest

Property-based testing takes a different approach: instead of testing specific inputs, you define properties that should hold for all inputs, and the framework generates thousands of random inputs to try to break those properties.


### Why property-based testing matters:

Catches edge cases you didn't think of Tests with inputs you'd never manually write (extreme values, unicode, etc.) Forces you to think about invariants, not just examples Often finds bugs that manual tests miss

```rust
// src/tools/calculator.rs
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn add_is_commutative(a in -1e10..1e10f64, b in -1e10..1e10f64) {
            prop_assert!((add(a, b) - add(b, a)).abs() < 1e-10);
        }

        #[test]
        fn add_zero_is_identity(a in -1e10..1e10f64) {
            prop_assert_eq!(add(a, 0.0), a);
        }

        #[test]
        fn divide_then_multiply_returns_original(
            a in -1e10..1e10f64,
            b in prop::num::f64::NORMAL.prop_filter("non-zero", |x| x.abs() > 1e-10)
        ) {
            let result = divide(a, b).unwrap();
            prop_assert!((result * b - a).abs() < 1e-6);
        }

        #[test]
        fn limit_validation_respects_bounds(limit in -100..2000i32) {
            let result = validate_query("SELECT 1", Some(limit));
            if limit >= 1 && limit <= 1000 {
                prop_assert!(result.is_ok());
            } else {
                prop_assert!(result.is_err());
            }
        }
    }
}
```


### Async Test Patterns

Most MCP tools perform async operations (database queries, HTTP calls, file I/O). Testing async code requires some extra setup, but the patterns are well-established.


### Key considerations:

Use  #[tokio::test]  instead of  #[test]  for async tests Set up and tear down test data to avoid test pollution Use test databases or mocks to avoid affecting production data

```rust
// src/tools/database.rs
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Use test fixtures
    async fn setup_test_db() -> PgPool {
        let pool = PgPool::connect("postgres://test:test@localhost/test_db")
            .await
            .expect("Failed to connect to test database");

        sqlx::query("CREATE TABLE IF NOT EXISTS test_users (id SERIAL, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    async fn teardown_test_db(pool: &PgPool) {
        sqlx::query("DROP TABLE IF EXISTS test_users")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_query_returns_results() {
        let pool = setup_test_db().await;

        // Insert test data
        sqlx::query("INSERT INTO test_users (name) VALUES ('Alice'), ('Bob')")
            .execute(&pool)
            .await
            .unwrap();

        // Test the query function
        let result = execute_query(&pool, "SELECT * FROM test_users", 10).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);

        teardown_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_query_respects_limit() {
        let pool = setup_test_db().await;

        // Insert more data than limit
        for i in 0..20 {
            sqlx::query(&format!("INSERT INTO test_users (name) VALUES ('User{}')", i))
                .execute(&pool)
                .await
                .unwrap();
        }

        let result = execute_query(&pool, "SELECT * FROM test_users", 5).await;
        assert!(result.is_ok());
// ... truncated, 5 more lines in source book ...
```


### Running Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test tools::calculator

# Run tests matching a pattern
cargo test divide

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```


### MCP Inspector: Interactive Testing

MCP Inspector is essential for development but not for automation.


### Quick Start

```bash
# Install Inspector
npm install -g @anthropic/mcp-inspector

# Start your server
cargo run --release

# Connect Inspector (HTTP transport)
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# Connect with SSE transport
npx @anthropic/mcp-inspector --transport sse http://localhost:3000/sse
```

(table/list, cells in reading order)
- When to Use Inspector vs mcp-tester
- Task
- Inspector
- mcp-tester
- Debugging new tool
- Exploring server capabilities
- One-off manual testing
- Automated test suites

```toml
[ok]

[ok]

[ok]
```

(table/list, cells in reading order)
- CI/CD pipelines
- Regression testing
- Edge case coverage
- Performance testing

```toml
[ok]

[ok]

[ok]

[ok]

[ok]
```


### mcp-tester: Automated Testing

mcp-tester is the core of PMCP's testing strategy.


### Core Workflow

```text
                  mcp-tester Workflow
1. GENERATE
   cargo pmcp test generate
     MCP Server       ▶    Schema
     (running)            Introspect
                  Generated Scenario Files
      tests/scenarios/
          tool_name_valid.yaml      (happy paths)
          tool_name_invalid.yaml    (error cases)
          tool_name_edge.yaml       (boundary values)
          tool_name_types.yaml      (type validation)
2. EDIT (optional)
   Add custom scenarios, assertions, edge cases
3. RUN
   cargo pmcp test run
      Scenario        ▶  MCP Server       ▶    Assert
       Files              Execute              Results
```


### Basic Commands

```bash
# Generate test scenarios from running server
cargo pmcp test generate --server http://localhost:3000

# Run all generated tests
cargo pmcp test run --server http://localhost:3000

# Run specific scenario file
cargo pmcp test run --scenario tests/scenarios/query_valid.yaml

# Verbose output with timing
cargo pmcp test run --verbose

# JSON output for CI integration
cargo pmcp test run --format json --output results.json
```

See mcp-tester Introduction for comprehensive documentation.


### Schema-Driven Test Generation

The most powerful mcp-tester feature is automatic test generation from JSON Schema.

```bash
# Generate tests for all tools
cargo pmcp test generate --output tests/scenarios/

# Generate with edge case depth
cargo pmcp test generate --edge-cases deep

# Generate only for specific tools
cargo pmcp test generate --tools query,insert
```


### See Schema-Driven Test Generation for the complete guide including:

How schema analysis works Generated test categories Customizing generated tests CI/CD integration

```text
src/
    tools/
        mod.rs
        calculator.rs      # Tool implementation
        query.rs
    lib.rs
tests/
    unit/                   # Rust unit tests
        calculator_test.rs
        query_test.rs
    scenarios/              # mcp-tester scenarios
        generated/          # Auto-generated (gitignore)
            add_valid.yaml
            add_invalid.yaml
        custom/             # Hand-written tests
            complex_workflow.yaml
            regression_123.yaml
    integration/            # Full integration tests
        client_test.rs
Cargo.toml
```


### Naming Conventions

```text
# tests/scenarios/custom/query_sql_injection_prevention.yaml
```


### name: "Query - SQL injection prevention" description: |

```yaml
  Verify that the query tool properly rejects SQL injection attempts.
  This is a critical security test.
tags:
  - security
  - regression
  - critical

steps:
  - tool: query
    input:
      sql: "SELECT * FROM users WHERE id = '1; DROP TABLE users; --'"
    expect:
      error:
        message_contains: "Invalid SQL"
```


### Continuous Testing Workflow

```bash
# Development workflow with watch mode
cargo watch -x test -x "pmcp test run"

# Pre-commit testing
cargo test && cargo pmcp test run --fail-fast

# Full test suite before PR
cargo test --all-features && \
cargo pmcp test generate && \
cargo pmcp test run --format junit --output test-results.xml
```

1.


### Practice Ideas

These informal exercises help reinforce the concepts.


### MCP Inspector Deep Dive

MCP Inspector is an interactive debugging and exploration tool for MCP servers.


### When to reach for Inspector:

You're developing a new tool and want to see if it works Something is broken and you need to see the actual requests/responses You want to understand an unfamiliar server's capabilities You're reproducing a bug report from a user

```text
                MCP Inspector Architecture
                     MCP Protocol
                                      ▶
MCP Inspector      JSON-RPC over:          MCP Server
  (Browser)        - HTTP POST            (Your Server)
                ◀
                   - SSE
                   - stdio
Developer Features:
• Tool browser with schema display
• Input form generation from JSON Schema
• Response viewer with pretty-printing
• Request/response history
• Error inspection and debugging
• Session lifecycle management
• Session management
```

```bash
# Install globally
npm install -g @anthropic/mcp-inspector

# Or run without installing
npx @anthropic/mcp-inspector

# Verify installation
```

```bash
# HTTP transport (recommended for development)
cargo run --release
# Server listening on http://localhost:3000

# With verbose logging for debugging
```


### RUST_LOG=debug cargo run --release

```bash
# With specific configuration
cargo run --release -- --port 3001 --host 0.0.0.0
```


### Connecting Inspector

```bash
# Connect to HTTP transport
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# Connect with SSE transport
npx @anthropic/mcp-inspector --transport sse http://localhost:3000/sse

# Connect to stdio-based server
npx @anthropic/mcp-inspector --transport stdio "cargo run --release"

# Connect with authentication
npx @anthropic/mcp-inspector \
  --header "Authorization: Bearer your-token" \
  http://localhost:3000/mcp

# Connect with custom timeout
npx @anthropic/mcp-inspector --timeout 30000 http://localhost:3000/mcp
```

```text
                      MCP Inspector
Connection: ● Connected to http://localhost:3000/mcp
Server: db-explorer v1.0.0
Protocol: MCP 2025-11-25
   CAPABILITIES
      Tools: 3 available
           list_tables
           get_sample_rows
           execute_query
      Resources: 0
      Prompts: 0
[Tools] [Resources] [Prompts] [Messages] [Settings]
```

```text
Tool: execute_query
Description: Execute a SELECT query on the database (read-only)
INPUT SCHEMA:
   {
     "type": "object",
     "properties": {
       "sql": {
         "type": "string",
         "description": "SQL SELECT query to execute"
       }
     },
     "required": ["sql"]
   }
INPUT FORM:
   sql*: [SELECT * FROM users LIMIT 5                       ]
                                            [Execute Tool]
```

```text
Response: execute_query                           Duration: 23ms
STATUS: Success
CONTENT:
   [
     {
       "type": "text",
       "text": "| id | name  | email           |\n..."
     }
   ]
RAW JSON:
   {
     "jsonrpc": "2.0",
     "id": 3,
     "result": {
       "content": [...]
     }
   }
[Copy Response] [Add to History] [Export]
```

```text
Message History                                   [Clear] [Export]
#1 [10:23:45] → initialize
    Client info, capabilities request
#2 [10:23:45] ← initialize (success)
    Server: db-explorer v1.0.0, Protocol: 2025-11-25
#3 [10:23:46] → tools/list
    List available tools
#4 [10:23:46] ← tools/list (success)
    3 tools: list_tables, get_sample_rows, execute_query
#5 [10:24:12] → tools/call (execute_query)
    sql: "SELECT * FROM users LIMIT 5"
#6 [10:24:12] ← tools/call (success, 23ms)
    5 rows returned
Click any message to see full JSON
```


### Common Debugging Workflows

These workflows represent the most common debugging scenarios you'll encounter.


### Workflow 1: Debugging a New Tool

When developing a new tool, use Inspector to validate behavior before writing automated tests.

```text
# 1. Start server with debug logging
```


### RUST_LOG=debug cargo run --release

```bash
# 2. Connect Inspector
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# 3. In Inspector:
#    a. Go to Tools tab
#    b. Find your new tool
#    c. Verify the schema looks correct
#    d. Test with valid inputs
#    e. Test with invalid inputs
#    f. Check error messages are helpful
```

Are all required fields marked as required?


### 2. Happy path testing

Does valid input produce expected output?


### 3. Error handling

What happens with missing required fields?


### Workflow 2: Diagnosing Connection Issues

Connection problems are frustrating because the error messages are often generic ("connection refused", "timeout").

```bash
# Check server is running
curl http://localhost:3000/health

# Check MCP endpoint responds
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":
```


### {"name":"test","version":"1.0"}},"id":1}'

```bash
# Expected: JSON response with server info

# Check with Inspector verbose mode
npx @anthropic/mcp-inspector --verbose http://localhost:3000/mcp
```

(table/list, cells in reading order)
- Common connection issues:
- Symptom
- Cause
- Solution
- Connection refused
- Server not running
- Start server first
- 404 on /mcp
- CORS error
- Timeout
- Wrong endpoint
- Check server route configuration
- Missing headers
- Add CORS middleware
- Server not responding
- Check for blocking code
- Symptom
- Cause
- Solution
- Parse error
- Invalid JSON
- Check response format
- Workflow 3: Testing Authentication

Authentication bugs are common and often subtle.

```bash
# Test without auth (should fail)
npx @anthropic/mcp-inspector http://localhost:3000/mcp
# Expected: 401 Unauthorized

# Test with auth header
npx @anthropic/mcp-inspector \
  --header "Authorization: Bearer your-api-key" \
  http://localhost:3000/mcp

# Test with multiple headers
npx @anthropic/mcp-inspector \
  --header "Authorization: Bearer your-api-key" \
  --header "X-Request-ID: test-123" \
  http://localhost:3000/mcp
```


### Workflow 4: Reproducing Bug Reports

The first step in fixing any bug is reproducing it.

```bash
# 1. Start server with exact configuration
cargo run --release

# 2. Connect Inspector
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# 3. Manually execute the reported sequence
#    - Use exact inputs from bug report
#    - Copy responses for analysis
#    - Export message history

# 4. Check Messages tab for:
#    - Request format
#    - Response format
#    - Error details
#    - Timing information
```


### Advanced Inspector Features

Beyond basic tool testing, Inspector provides advanced capabilities for edge case testing, security verification, and deep protocol debugging.


### Custom Request Builder

Sometimes you need to send requests that the normal UI can't construct—malformed JSON, missing fields, or injection attempts.

```text
Custom Request
METHOD: [tools/call                                    ▼]
PARAMS:
   {
     "name": "execute_query",
     "arguments": {
       "sql": "SELECT * FROM users; DROP TABLE users; --"
     }
   }
[Send Request]
```


### This allows testing:

Malformed requests Invalid method names Missing required fields Injection attempts Boundary values

```bash
# Start Inspector with session tracing
npx @anthropic/mcp-inspector --trace-session http://localhost:3000/mcp
```


### Watch for:

1.

```text
# Export message history
# In Inspector: Messages tab → Export → JSON

# The export includes:
{
  "session": {
    "server": "db-explorer",
    "version": "1.0.0",
    "connected_at": "2024-01-15T10:23:45Z"
  },
  "messages": [
    {
      "direction": "outgoing",
      "timestamp": "2024-01-15T10:23:45.123Z",
      "message": {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {...},
        "id": 1
      }
    },
    ...
  ]
}
```


### Testing Different Transports

MCP supports multiple transport mechanisms, and Inspector can test all of them.


### HTTP POST Transport

The simplest and most common transport.

```rust
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# Server implementation
async fn mcp_handler(
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    // Handle request and return response
}
```


### SSE Transport

Server-Sent Events enable the server to push updates to the client—useful for long-running operations or real-time notifications.

```bash
npx @anthropic/mcp-inspector --transport sse http://localhost:3000/sse

# Server sends events like:
# event: message
# data: {"jsonrpc":"2.0","result":...}
```


### Inspector will:

Send requests via POST Receive responses via SSE stream Handle connection keep-alive Reconnect on disconnect


### Streamable HTTP Transport

The newest transport option, combining the simplicity of HTTP with streaming capabilities.

```bash
npx @anthropic/mcp-inspector --transport streamable http://localhost:3000/mcp

# This transport supports:
# - HTTP POST for requests
# - Streaming responses
# - Server-initiated notifications
```


### stdio Transport

For servers that run as local processes (like CLI tools), stdio transport communicates via standard input/output.

```bash
npx @anthropic/mcp-inspector --transport stdio "cargo run --release"

# Inspector will:
# - Spawn your server as a subprocess
# - Send JSON-RPC over stdin
# - Read responses from stdout
# - Display stderr as debug output
```

(table/list, cells in reading order)
- Comparing Tools
- Inspector vs mcp-tester
- Feature
- Inspector
- mcp-tester
- Purpose
- Interface
- Automation
- Interactive debugging
- Automated testing
- Visual/GUI
- Manual only
- CLI/YAML files
- Full CI/CD support
- Schema exploration
- Excellent
- Basic
- Error debugging
- Detailed view
- Regression testing
- Not suitable
- Performance testing
- Basic timing
- Edge case discovery
- Manual
- Pass/fail results
- Designed for it
- Detailed metrics
- Auto-generated
- Inspector vs Claude Desktop
- Feature
- Inspector
- Claude Desktop
- Purpose
- Development/debugging
- End-user experience
- Protocol view
- Full visibility
- Custom requests
- Supported

```text
                  Testing Tool Selection
Development Phase:
   Writing new tool → Inspector
   Debugging issue  → Inspector
   Learning MCP     → Inspector
Testing Phase:
   Unit tests      → cargo test
   Integration     → mcp-tester
   Edge cases      → mcp-tester (generated)
   Regression      → mcp-tester (CI/CD)
Production Phase:
   Smoke tests     → mcp-tester (subset)
   User acceptance → Claude Desktop
   Bug reproduction→ Inspector
```


### When you see slow responses:

1.

```text
# Test SQL injection
```


### Input: "SELECT * FROM users WHERE id = '1' OR '1'='1'"

```text
# Test path traversal
```


### Input: "../../../etc/passwd"

```text
# Test command injection
```


### Input: "test; rm -rf /"

```text
# Test XSS (if output is HTML)
```

Rejects or sanitizes malicious input Returns appropriate error messages Doesn't expose sensitive data in errors


### Common Pitfalls

1. Forgetting to restart server - Code changes require restart 2.

```text
# 1. Write code
```


### vim src/tools/new_feature.rs

```bash
# 2. Build and run
cargo run --release &

# 3. Test with Inspector
npx @anthropic/mcp-inspector http://localhost:3000/mcp
# - Explore schema
# - Test happy paths
# - Test error cases

# 4. If issues found, check logs
# Server window shows RUST_LOG output

# 5. Fix and repeat
```


### Watch Mode Development

```bash
# Terminal 1: Watch for changes and rebuild
cargo watch -x run --release

# Terminal 2: Keep Inspector connected
npx @anthropic/mcp-inspector http://localhost:3000/mcp

# Workflow:
# 1. Edit code
# 2. cargo watch rebuilds automatically
# 3. Inspector reconnects (may need manual refresh)
# 4. Test immediately
```

Understanding how your server responds to requests Debugging issues during development Exploring server capabilities and schemas Reproducing reported bugs Testing authentication and security


### Practice Ideas

These informal exercises help reinforce the concepts.


### mcp-tester: Automated MCP Testing

mcp-tester is the automated testing component of cargo-pmcp, designed to make MCP server testing as natural as unit testing in Rust.


### The Problem with Manual MCP Testing

```text
                  Manual MCP Testing Pain
1. Craft JSON-RPC request manually
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": { "name": "query", "arguments": { ... } }
   }
2. Send via curl or Inspector
   curl -X POST ... -d '...'
3. Manually verify response
   - Check JSON structure
   - Verify expected values
   - Test error cases... repeat for each
4. Repeat for every tool × every input combination
   🔁 Tedious, error-prone, not repeatable
```


### The mcp-tester Solution

```text
                  mcp-tester Automation
1. Generate scenarios from schema
   cargo pmcp test generate
   → Creates YAML test files automatically
2. Edit scenarios (optional)
   → Add custom edge cases
   → Tune assertions
3. Run tests automatically
   cargo pmcp test run
   → Executes all scenarios
   → Reports pass/fail with details
4. Integrate in CI/CD
   → JUnit output for CI systems
   → Fail builds on test failures
```

```bash
# Install cargo-pmcp (includes mcp-tester)
cargo install cargo-pmcp

# Verify installation
cargo pmcp test --help
```

```bash
# Generate from a running server
cargo pmcp test generate --server http://localhost:3000

# Generate to specific directory
cargo pmcp test generate --server http://localhost:3000 --output tests/scenarios

# Generate with deep edge cases
cargo pmcp test generate --server http://localhost:3000 --edge-cases deep

# Generate for specific tools only
cargo pmcp test generate --server http://localhost:3000 --tools query,insert,delete

# Generate with custom naming
cargo pmcp test generate --server http://localhost:3000 --prefix db_explorer
```


### Running Tests

```bash
# Run all scenarios in default directory
cargo pmcp test run --server http://localhost:3000

# Run specific scenario file
cargo pmcp test run --server http://localhost:3000 \
  --scenario tests/scenarios/query_valid.yaml

# Run all scenarios matching a pattern
cargo pmcp test run --server http://localhost:3000 \
  --pattern "*_security_*.yaml"

# Run with verbose output
cargo pmcp test run --server http://localhost:3000 --verbose

# Stop on first failure
cargo pmcp test run --server http://localhost:3000 --fail-fast

# Output in different formats
cargo pmcp test run --server http://localhost:3000 --format json
cargo pmcp test run --server http://localhost:3000 --format junit --output results.xml
cargo pmcp test run --server http://localhost:3000 --format tap
```


### Scenario File Format

Scenarios are YAML files that describe test steps and expected outcomes.


### Basic Structure

```text
# tests/scenarios/calculator_add.yaml

# Metadata
```

name: "Calculator Add Tool" description: "Verify the add tool performs correct arithmetic" version: "1.0"

```yaml
tags:
  - calculator
  - arithmetic
  - regression

# Server configuration (optional, can be overridden by CLI)
server:
  url: http://localhost:3000
  transport: http
  timeout: 30s

# Setup steps (run before test steps)
setup:
  - tool: reset_calculator
    input: {}

# Test steps
steps:
  - name: "Add two positive numbers"
    tool: add
    input:
      a: 10
      b: 5
    expect:
      result: 15

  - name: "Add negative numbers"
    tool: add
    input:
      a: -10

      b: -5
    expect:
      result: -15

  - name: "Add with zero"
    tool: add
    input:
      a: 42
      b: 0
    expect:
      result: 42

# Teardown steps (run after test steps, even on failure)
teardown:
  - tool: cleanup
    input: {}
```


### Complete Step Options

```yaml
steps:
  - name: "Descriptive step name"           # Required
    description: "Longer description"       # Optional

    # Tool invocation
    tool: tool_name                         # Required
    input:                                  # Tool arguments
      param1: "value1"
      param2: 123
      nested:
        key: "value"

    # Timing
    timeout: 10s                            # Step-specific timeout
    delay_before: 500ms                     # Wait before execution
    delay_after: 100ms                      # Wait after execution

    # Retry configuration
    retry:
      count: 3                              # Number of retries
      delay: 1s                             # Delay between retries
      on_error: true                        # Retry on any error

    # Expectations (assertions)
    expect:
      # Success assertions
      success: true                         # Expect success (default)
      result: <exact_value>                 # Exact match
      contains:                             # Partial match
        key: "expected_value"
      type:                                 # Type checking
        result: number
        items: array
      matches:                              # Regex matching
        message: "Created item \\d+"
      comparison:                           # Numeric comparisons

        count:
          gte: 1
          lte: 100

      # Error assertions
      error:                                # Expect an error
        code: -32602                        # JSON-RPC error code
        message: "exact message"            # Exact message match
        message_contains: "partial"         # Partial message match

    # Capture values for later steps
    capture:
      item_id: "$.result.id"                # JSONPath expression
      all_items: "$.result.items[*]"        # Array capture
```

```yaml
steps:
  - name: "Create a customer"
    tool: create_customer
    input:
      name: "Test Corp"
      email: "test@example.com"
    capture:
      customer_id: "$.result.id"
      created_at: "$.result.created_at"

  - name: "Retrieve the customer"
    tool: get_customer
    input:
      id: "${customer_id}"                  # Use captured value
    expect:
      contains:
        id: "${customer_id}"
        name: "Test Corp"

  - name: "Update the customer"
    tool: update_customer
    input:
      id: "${customer_id}"
      name: "Updated Corp"
    expect:
      success: true

  - name: "Delete the customer"
    tool: delete_customer
    input:
      id: "${customer_id}"
    expect:
      contains:
        deleted: true
```


### Environment Variables

```yaml
# Reference environment variables
server:
  url: "${MCP_SERVER_URL:-http://localhost:3000}"

steps:
  - name: "Query with credentials"
    tool: authenticated_query
    input:
      api_key: "${API_KEY}"                 # From environment
      query: "SELECT * FROM users"
```


### Assertion Types

Assertions are how you tell mcp-tester what to verify about the response.


### Choosing the right assertion:

Exact match when you need to verify the complete response (simple values, critical fields) Partial match when you only care about specific fields (response may include extra data) Type checking when the structure matters but values vary (IDs, timestamps) Regex matching when values follow a pattern (


### Exact Match

Use exact match when you need to verify the complete response or when specific values are critical.

```yaml
expect:
  result: 42                                # Number
  message: "Success"                        # String
  items: [1, 2, 3]                          # Array
  user:                                     # Object
    name: "Alice"
    age: 30
```


### Partial Match (contains)

The most commonly used assertion. Use it when you want to verify specific fields exist with correct values, but you don't care about other fields in the response. This makes tests more resilient to API evolution—adding new fields won't break existing tests.

```yaml
expect:
  contains:
    status: "success"                       # Object must contain this
    # Other fields are ignored
```


### Type Checking

Use type checking when the structure matters more than specific values. This is ideal for fields that vary by call (like auto-generated IDs or timestamps) where you can't predict the exact value but know it should be a string, number, etc.

```yaml
expect:
  type:
    id: string
    count: number
    items: array
    metadata: object
    active: boolean
    optional_field: "null|string"           # Nullable
```


### Regex Matching

Use regex when values follow a predictable pattern but aren't exact.

```yaml
expect:
  matches:
    id: "^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$"  # UUID v4
    timestamp: "\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}"  # ISO datetime
    message: "Created (user|customer) \\d+"
```


### Numeric Comparisons

Use comparisons when you need to verify values fall within acceptable ranges rather than matching exact numbers. This is essential for counts (should be at least 1), scores (should be between 0-100), or any value where the exact number varies but should stay within bounds.

```yaml
expect:
  comparison:
    count:
      gt: 0                                 # Greater than
      gte: 1                                # Greater than or equal
      lt: 100                               # Less than
      lte: 100                              # Less than or equal
      eq: 50                                # Equal
      ne: 0                                 # Not equal
    response_time_ms:
      lt: 1000                              # Performance assertion
```


### Array Assertions

Use array assertions when working with collections. You often can't predict exact array contents, but you can verify: length constraints (pagination working?), presence of specific elements (admin user exists?), or that all elements meet certain criteria (all users have required fields?).

```yaml
expect:
  array:
    items:
      length: 5                             # Exact length
      min_length: 1                         # Minimum length
      max_length: 100                       # Maximum length
      contains: "admin"                     # Contains element
      all_match:                            # All elements match
        type: object
        contains:
          active: true
      any_match:                            # At least one matches
        contains:
          role: "admin"
```


### Error Assertions

Error assertions verify that your server fails correctly. This is just as important as success testing—you need to prove that invalid input produces helpful errors, not crashes or security vulnerabilities.


### Levels of strictness:

error: true  — just verify it fails (any error is acceptable) error.code  — verify the JSON-RPC error code (for programmatic handling) error.message  — verify the exact message (for user-facing errors) error.message_contains  — verify the message includes key information

```yaml
# Expect specific error
expect:
  error:
    code: -32602                            # Invalid params
    message: "Missing required field: query"

# Expect any error
expect:
  error: true

# Expect error containing text
expect:
  error:
    message_contains: "not found"

# Expect error matching pattern
expect:
  error:
    message_matches: "Item \\d+ not found"
```


### Test Categories

Testing isn't just about verifying your code works—it's about systematically proving your server handles all the situations it will encounter in production. Think of them as layers of protection: happy path tests prove your server does what it should, error tests prove it fails gracefully, edge case tests prove it handles unusual inputs, and security tests prove it can't be exploited.


### Happy Path Tests

What they test: The normal, expected usage patterns—what happens when users use your tool correctly.


### What to include:

The most common use case (the one 80% of users will hit) Variations with different valid input combinations Empty results (a valid query that returns nothing is still a success)

```text
# tests/scenarios/query_happy_path.yaml
```

name: "Query Tool - Happy Path" description: "Normal usage patterns that should succeed"

```yaml
steps:
  - name: "Simple SELECT query"
    tool: query
    input:
      sql: "SELECT * FROM users LIMIT 5"
    expect:
      type:
        rows: array
      array:
        rows:
          max_length: 5

  - name: "Query with parameters"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE status = $1"
      params: ["active"]
    expect:
      success: true

  - name: "Empty result set"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE 1=0"
    expect:
      contains:
        rows: []
        row_count: 0
```


### Error Handling Tests

What they test: How your server responds when given bad input or when something goes wrong.


### What to include:

Missing required fields Invalid field values (wrong type, out of range) Forbidden operations (like DROP TABLE in a read-only query tool) Malformed input that might cause parsing errors

"Query cannot be empty"  is actionable;  "Internal server error"  is not.

```text
# tests/scenarios/query_errors.yaml
```

```yaml
steps:
  - name: "Reject non-SELECT query"
    tool: query
    input:
      sql: "DROP TABLE users"
    expect:
      error:
        code: -32602
        message_contains: "Only SELECT queries allowed"

  - name: "Reject empty query"
    tool: query
    input:
      sql: ""
    expect:
      error:
        message_contains: "Query cannot be empty"

  - name: "Reject SQL injection attempt"
    tool: query
    input:
      sql: "SELECT * FROM users; DROP TABLE users; --"
    expect:
      error:
        message_contains: "Invalid SQL"

  - name: "Handle invalid table"
    tool: query
    input:
      sql: "SELECT * FROM nonexistent_table"
    expect:
      error:
        message_contains: "does not exist"
```


### Edge Case Tests

What they test: The boundary conditions and unusual-but-valid inputs at the extremes of what your tool accepts.


### What to include:

Boundary values (minimum, maximum, just above/below limits) Empty inputs (empty string, empty array, null where allowed) Unicode and special characters Very large or very small values Unusual but valid combinations

```text
# tests/scenarios/query_edge_cases.yaml
```

```yaml
steps:
  - name: "Maximum limit value"
    tool: query
    input:
      sql: "SELECT * FROM users"
      limit: 1000
    expect:
      success: true

  - name: "Limit at boundary (1001 should fail)"
    tool: query
    input:
      sql: "SELECT * FROM users"
      limit: 1001
    expect:
      error:
        message_contains: "Limit must be between 1 and 1000"

  - name: "Unicode in query"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE name = '日本語'"
    expect:
      success: true

  - name: "Very long query"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE name IN ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z')"
    expect:
      success: true
```


### Security Tests

What they test: Whether your server can be tricked into doing something dangerous through malicious input.


### Common attack patterns to test:

SQL Injection: Can an attacker embed SQL commands in input fields? Authorization Bypass: Can users access data they shouldn't?

Important: Security tests should be tagged (see  tags:  below) so you can run them separately and ensure they never regress.

```text
# tests/scenarios/query_security.yaml
```

```yaml
tags:
  - security
  - critical

steps:
  - name: "SQL injection - comment"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE id = '1' --"
    expect:
      error:
        message_contains: "Invalid SQL"

  - name: "SQL injection - UNION"
    tool: query
    input:
      sql: "SELECT * FROM users UNION SELECT * FROM passwords"
    expect:
      error:
        message_contains: "UNION not allowed"

  - name: "SQL injection - subquery"
    tool: query
    input:
      sql: "SELECT * FROM users WHERE id = (SELECT password FROM users WHERE id = 1)"
    expect:
      # Either success (if subquery allowed) or specific error
      success: true

  - name: "Path traversal in table name"
    tool: query
    input:
      sql: "SELECT * FROM '../../../etc/passwd'"
    expect:
      error: true
```


### Performance Tests

What they test: Whether your server responds within acceptable time limits.


### What to include:

Simple operations (should be fast—under 100ms) Complex operations (acceptable latency—1-5 seconds) Timeout boundaries (verify the server doesn't hang indefinitely)


### Key considerations:

Set realistic thresholds based on what your users expect Performance can vary by environment (CI machines are often slower) Consider running performance tests separately from functional tests Track performance trends over time, not just pass/fail

The timeout assertion: Using  timeout: 100ms  doesn't just test speed—it proves your server will fail fast rather than hang when something goes wrong.

```text
# tests/scenarios/query_performance.yaml
```

```yaml
tags:
  - performance

steps:
  - name: "Simple query under 100ms"
    tool: query
    input:
      sql: "SELECT 1"
    timeout: 100ms
    expect:
      success: true

  - name: "Complex query under 5s"
    tool: query
    input:
      sql: "SELECT * FROM large_table LIMIT 1000"
    timeout: 5s
    expect:
      success: true
```


### Multi-Step Workflows

Single-tool tests verify individual operations work correctly.


### Why workflows matter:

They test the actual user journeys, not just isolated operations They catch state-related bugs (e.g., created record has wrong ID format) They verify that your API is coherent (create returns what get expects)


### They document real-world usage patterns

Variable capture is the key feature:  capture  extracts values from one step's response so you can use them in later steps.


### CRUD Workflow

The most common workflow pattern tests the full lifecycle of a resource: Create, Read, Update, Delete.

```text
# tests/scenarios/customer_crud_workflow.yaml
```

```yaml
steps:
  - name: "Create customer"
    tool: create_customer
    input:
      name: "Acme Corp"
      email: "contact@acme.com"
      tier: "enterprise"
    capture:
      customer_id: "$.result.id"
    expect:
      contains:
        name: "Acme Corp"
        tier: "enterprise"

  - name: "Read customer"
    tool: get_customer
    input:
      id: "${customer_id}"
    expect:
      contains:
        id: "${customer_id}"
        name: "Acme Corp"

  - name: "Update customer"
    tool: update_customer
    input:
      id: "${customer_id}"
      name: "Acme Corporation"
      tier: "premium"
    expect:
      contains:
        name: "Acme Corporation"
        tier: "premium"

  - name: "Verify update"

    tool: get_customer
    input:
      id: "${customer_id}"
    expect:
      contains:
        name: "Acme Corporation"

  - name: "Delete customer"
    tool: delete_customer
    input:
      id: "${customer_id}"
    expect:
      contains:
        deleted: true

  - name: "Verify deletion"
    tool: get_customer
    input:
      id: "${customer_id}"
    expect:
      error:
        message_contains: "not found"
```


### Conditional Workflows

Sometimes workflows need to branch based on runtime conditions—testing different paths depending on server state or configuration. Conditional steps let you write tests that adapt to the actual server response rather than assuming a fixed state.


### Use cases:

Testing feature flag behavior (if flag enabled, test new behavior; otherwise, test legacy) Handling optional features (if server supports X, test X) Testing different authorization levels

```text
# tests/scenarios/conditional_workflow.yaml
```

```yaml
steps:
  - name: "Check feature flag"
    tool: get_feature_flag
    input:
      flag: "new_pricing"
    capture:
      flag_enabled: "$.result.enabled"

  - name: "Apply new pricing (if enabled)"
    condition: "${flag_enabled} == true"
    tool: calculate_price
    input:
      product_id: "prod_123"
      pricing_version: "v2"
    expect:
      success: true

  - name: "Apply legacy pricing (if disabled)"
    condition: "${flag_enabled} == false"
    tool: calculate_price
    input:
      product_id: "prod_123"
      pricing_version: "v1"
    expect:
      success: true
```


### CI/CD Integration

Tests are only valuable if they run consistently.


### Key integration patterns:

1. Fail the build — don't allow merging if tests fail 4.


### GitHub Actions

```text
# .github/workflows/test.yml
```


### name: MCP Server Tests

```bash
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cargo-pmcp
        run: cargo install cargo-pmcp

      - name: Build server

        run: cargo build --release

      - name: Start server
        run: |
          ./target/release/my-mcp-server &
          sleep 5  # Wait for startup
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost/test

      - name: Run mcp-tester
        run: |
          cargo pmcp test run \
            --server http://localhost:3000 \
            --format junit \
            --output test-results.xml

      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: test-results.xml

      - name: Publish test results
        uses: dorny/test-reporter@v1
        if: always()
// ... truncated, 4 more lines in source book ...
```


### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

build:
  stage: build
  image: rust:1.75
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/my-mcp-server

test:
  stage: test
  image: rust:1.75
  services:
    - postgres:15
  variables:
    DATABASE_URL: postgres://postgres:postgres@postgres/test
  script:
    - cargo install cargo-pmcp
    - ./target/release/my-mcp-server &
    - sleep 5
    - cargo pmcp test run --server http://localhost:3000 --format junit --output results.xml
  artifacts:
    reports:
      junit: results.xml
```


### Makefile Integration

```text
# Makefile
```


### .PHONY: test test-unit test-mcp test-all

```bash
# Rust unit tests
test-unit:
    cargo test

# Start server and run mcp-tester
test-mcp: build
    @echo "Starting server..."
    @./target/release/my-mcp-server &
    @sleep 3
    @echo "Running mcp-tester..."
    @cargo pmcp test run --server http://localhost:3000 || (pkill my-mcp-server; exit 1)
    @pkill my-mcp-server

# Generate new test scenarios
test-generate:
    @./target/release/my-mcp-server &
    @sleep 3
    @cargo pmcp test generate --server http://localhost:3000 --output tests/scenarios/generated/
    @pkill my-mcp-server

# Run all tests
```


### test-all: test-unit test-mcp

```bash
# CI target
ci: build
    cargo test --all-features
    ./target/release/my-mcp-server &
    sleep 3
    cargo pmcp test run --server http://localhost:3000 --format junit --output test-results.xml
    pkill my-mcp-server
```

```bash
# See detailed request/response
cargo pmcp test run --verbose

# Output:
```

```text
#
```

```text
# Step: Add two positive numbers
```

```text
#
```

```text
# Request:
#   Tool: add
#   Input: {"a": 10, "b": 5}
```


### #

```text
# Response:
#   Status: Success
#   Result: {"content": [{"type": "text", "text": "15"}]}
#   Duration: 12ms
```


### #

```text
# Assertions:
#   [ok] result equals 15
```

```text
#
```


### Debug Mode

```bash
# Maximum verbosity with JSON-RPC traces
cargo pmcp test run --debug

# Save raw responses for analysis
cargo pmcp test run --save-responses ./debug/
```


### Common Failure Patterns

```text
                  Common Test Failures
"Connection refused"
→ Server not running or wrong port
→ Check: curl http://localhost:3000/health
"Expected X but got Y"
→ Response format changed
→ Check: cargo pmcp test run --verbose
"Timeout exceeded"
→ Server too slow or hung
→ Increase timeout or check server logs
"Invalid JSON-RPC response"
→ Server returning non-JSON or malformed response
→ Check server implementation
"Capture failed: path not found"
→ JSONPath doesn't match response structure
→ Use --verbose to see actual response
```


### Best Practices

Good test suites are maintainable, reliable, and trustworthy. These practices help you avoid common pitfalls that make tests fragile, slow, or confusing.


### Scenario Organization

Keep your test files organized so you can find what you need.


### tests/scenarios/

```text
generated/              # Auto-generated (add to .gitignore)
    query_valid.yaml
    query_invalid.yaml
custom/                 # Hand-written tests (commit these)
    query_security.yaml
    query_edge_cases.yaml
    workflow_crud.yaml
regression/             # Bug fix verification tests
    issue_123.yaml
    issue_456.yaml
```


### Test Independence

Tests should be self-contained—each scenario should set up its own data and clean up after itself.

The rule: A test that passes when run alone should pass when run with other tests. A test that fails should fail for one reason: the code under test is broken.

```yaml
# BAD: Tests depend on each other
steps:
  - name: "Create user"
    tool: create_user
    # Later tests assume this user exists

# GOOD: Each test is self-contained
setup:
  - tool: create_test_user
    input:
      id: "test_user_1"

steps:
  - name: "Get user"
    tool: get_user
    input:
      id: "test_user_1"

teardown:
  - tool: delete_user
    input:
      id: "test_user_1"
```


### Meaningful Assertions

A test that only checks  success: true  proves very little—the server could return completely wrong data and the test would still pass.

```yaml
# BAD: Only checks success
expect:
  success: true

# GOOD: Verifies actual behavior
expect:
  contains:
    id: "${created_id}"
    status: "active"
  type:
    created_at: string
  comparison:
    items:
      gte: 1
```


### Key workflow:

```bash
# Generate initial tests
cargo pmcp test generate --server http://localhost:3000

# Add custom edge cases and security tests
```


### vim tests/scenarios/custom/security.yaml

```bash
# Run all tests
cargo pmcp test run --server http://localhost:3000

# Integrate in CI
cargo pmcp test run --format junit --output results.xml
```


### App Metadata Validation

Learning objective: After this section, you will be able to validate MCP App metadata conformance using  mcp-tester apps  and  cargo pmcp test apps .


### Why App Metadata Validation Matters

MCP Apps servers must emit correct  _meta , MIME types, and resource cross-references for widgets to render across different hosts (Claude Desktop, ChatGPT, VS Code).

Claude Desktop requires  text/html;profile=mcp-app  MIME type and  _meta.ui.resourceUri  on resources ChatGPT requires additional  openai/*  descriptor keys in tool and resource metadata All hosts require that  _meta.ui.resourceUri  on tools points to a resource that actually exists in  resources/list


### CLI Usage

```text
# Standard validation -- checks all App-capable tools
```


### mcp-tester apps http://localhost:3000

```text
# ChatGPT-specific validation -- also checks openai/* keys
```


### mcp-tester apps http://localhost:3000 --mode chatgpt

```text
# Claude Desktop validation
```


### mcp-tester apps http://localhost:3000 --mode claude-desktop

```text
# Strict mode -- warnings become failures (ideal for CI)
```


### mcp-tester apps http://localhost:3000 --strict

```bash
# Via cargo pmcp wrapper
cargo pmcp test apps --url http://localhost:3000
cargo pmcp test apps --url http://localhost:3000 --mode chatgpt --strict
```

(table/list, cells in reading order)
- What Gets Validated
- The validator checks each App-capable tool in the server:
- Check
- What It Verifies
- _meta  field present
- Tool has metadata indicating it's App-capable
- ui.resourceUri  present
- Tool links to a widget resource
- MIME type correct
- Resource uses  text/html;profile=mcp-app  (not legacy variants)
- Resource cross-reference
- Tool's URI matches an entry in  resources/list
- [ChatGPT mode]  openai/*  keys
- ChatGPT descriptor keys present in tool metadata
- [if present]  outputSchema  valid
- Output schema structure is well-formed

Try this: Run  mcp-tester apps  against the Open Images example server and examine the validation output.


### Practice Ideas

These informal exercises help reinforce the concepts.


### Schema-Driven Test Generation

The most powerful feature of mcp-tester is automatic test generation from your MCP server's JSON Schema definitions.


### How Schema Analysis Works

Schema-driven testing leverages the fact that MCP tools already define their input requirements via JSON Schema. Instead of manually writing tests for every field and constraint, mcp-tester reads your schema and automatically generates tests that verify your server correctly enforces those constraints.

If you declare a field as required, you're promising to reject requests without it.


### The Generation Process

```text
                  Schema Analysis Pipeline
1. INTROSPECT
      mcp-tester connects to server
      Calls: initialize → tools/list
      Retrieves: tool names, descriptions, inputSchemas
2. ANALYZE SCHEMA
      For each tool's inputSchema:
      • Parse JSON Schema structure
      • Identify required vs optional properties
      • Extract type constraints (string, number, etc.)
      • Find validation rules (min, max, pattern, enum)
      • Detect nested objects and arrays
3. GENERATE TEST CASES
      For each property and constraint:
      • Valid value tests (within constraints)
      • Boundary value tests (min, max, at limits)
      • Invalid value tests (violate constraints)
      • Type violation tests (wrong types)
      • Required field tests (missing required)
4. OUTPUT YAML FILES
```

```text
tests/scenarios/generated/
    toolname_valid.yaml
    toolname_invalid.yaml
    toolname_edge.yaml
    toolname_types.yaml
```


### Schema Elements Analyzed

Each JSON Schema constraint maps to specific test cases.

(table/list, cells in reading order)
- Schema Element
- Generated Tests
- type: string
- type: number
- type: integer
- type: boolean
- type: array
- type: object
- Valid string, empty string, null
- Valid number, zero, negative, float
- Valid int, float (should fail), boundaries
- true, false, truthy strings (should fail)
- Empty array, single item, multiple items
- Valid object, empty object, nested
- required: [...]
- Missing each required field
- minimum/maximum
- Below min, at min, at max, above max
- minLength/maxLength
- Empty, at min, at max, over max
- pattern
- enum
- Matching, non-matching
- Each valid value, invalid value
- format  (email, uri, etc.)
- Valid format, invalid format
- Running the Generator
- Basic Generation

```bash
# Start your server
cargo run --release &

# Generate tests
cargo pmcp test generate --server http://localhost:3000

# Output:
# Connecting to server...
# Found 5 tools: query, insert, update, delete, get_schema
# Generating tests...
# [ok] query_valid.yaml (8 test steps)
# [ok] query_invalid.yaml (12 test steps)
# [ok] query_edge.yaml (6 test steps)
# [ok] query_types.yaml (4 test steps)
# ... (repeated for each tool)
# Generated 80 test scenarios in tests/scenarios/generated/
```


### Generation Options

```bash
# Specify output directory
cargo pmcp test generate \
  --server http://localhost:3000 \
  --output tests/scenarios/generated/

# Generate only for specific tools
cargo pmcp test generate \
  --server http://localhost:3000 \
  --tools query,insert

# Control edge case depth
cargo pmcp test generate \
  --server http://localhost:3000 \
  --edge-cases minimal    # Fewer edge cases
cargo pmcp test generate \
  --server http://localhost:3000 \
  --edge-cases deep       # More comprehensive

# Add prefix to generated files
cargo pmcp test generate \
  --server http://localhost:3000 \
  --prefix db_explorer

# Generate with descriptions from tool metadata
cargo pmcp test generate \
  --server http://localhost:3000 \
  --include-descriptions

# Dry run - show what would be generated
cargo pmcp test generate \
  --server http://localhost:3000 \
  --dry-run
```


### Generated Test Categories

mcp-tester organizes generated tests into four categories, each serving a distinct purpose.


### 1. Valid Input Tests (_valid.yaml)

Purpose: Prove that your tool accepts inputs that conform to the schema.

If valid input tests fail, your tool is rejecting requests it should accept—a critical bug that would frustrate users.


### What's generated:

One test with all required fields (the minimal valid request) Tests with optional fields included Tests for each enum value (if applicable) Tests with different valid combinations

```text
# Generated: query_valid.yaml
```

```yaml
steps:
  # Test with all required fields
  - name: "All required fields provided"
    tool: query
    input:
      sql: "SELECT * FROM users"
    expect:
      success: true

  # Test with optional fields
  - name: "With optional limit"
    tool: query
    input:
      sql: "SELECT * FROM users"
      limit: 100
    expect:
      success: true

  # Test each enum value
  - name: "Format: json"
    tool: query
    input:
      sql: "SELECT 1"
      format: "json"
    expect:
      success: true

  - name: "Format: csv"
    tool: query
    input:
      sql: "SELECT 1"
      format: "csv"

    expect:
      success: true
```


### 2. Invalid Input Tests (_invalid.yaml)

Purpose: Prove that your tool rejects inputs that violate the schema.


### What's generated:

One test for each required field (missing that field) Tests that violate each constraint (below minimum, above maximum, wrong pattern) Tests with invalid enum values Tests with null for non-nullable fields

```text
# Generated: query_invalid.yaml
```

```yaml
steps:
  # Missing required field
  - name: "Missing required: sql"
    tool: query
    input:
      limit: 100
      # sql is missing
    expect:
      error:
        code: -32602
        message_contains: "sql"

  # Pattern violation
  - name: "Pattern violation: sql must start with SELECT"
    tool: query
    input:
      sql: "DROP TABLE users"
    expect:
      error:
        code: -32602

  # Enum violation
  - name: "Invalid enum value: format"
    tool: query
    input:
      sql: "SELECT 1"
      format: "invalid_format"
    expect:
      error:
        code: -32602
        message_contains: "format"

  # Below minimum
  - name: "Below minimum: limit"

    tool: query
    input:
      sql: "SELECT 1"
      limit: 0
    expect:
      error:
        code: -32602
        message_contains: "limit"

  # Above maximum
  - name: "Above maximum: limit"
    tool: query
    input:
      sql: "SELECT 1"
      limit: 10001
    expect:
      error:
        code: -32602
```


### 3. Edge Case Tests (_edge.yaml)

Purpose: Test the boundary conditions—values that are valid but at the extreme edges of what's allowed.


### What's generated:

Values exactly at minimum and maximum boundaries Strings exactly at minLength and maxLength Arrays at minItems and maxItems First and last enum values

```text
# Generated: query_edge.yaml
```

```yaml
steps:
  # Boundary: at minimum
  - name: "Boundary: limit at minimum (1)"
    tool: query
    input:
      sql: "SELECT 1"
      limit: 1
    expect:
      success: true

  # Boundary: at maximum
  - name: "Boundary: limit at maximum (1000)"
    tool: query
    input:
      sql: "SELECT 1"
      limit: 1000
    expect:
      success: true

  # String length: at minLength
  - name: "String at minLength"
    tool: query
    input:
      sql: "S"  # If minLength: 1
    expect:
      success: true

  # String length: at maxLength
  - name: "String at maxLength"
    tool: query
    input:
      sql: "SELECT ... (very long)"  # At maxLength
    expect:
      success: true

  # Empty array (if minItems: 0)
  - name: "Empty array for columns"
    tool: query
    input:
      sql: "SELECT 1"
      columns: []
    expect:
      success: true

  # Array at minItems
  - name: "Array at minItems"
    tool: query
    input:
      sql: "SELECT 1"
      columns: ["id"]  # minItems: 1
    expect:
      success: true
```


### 4. Type Validation Tests (_types.yaml)

Purpose: Verify that your tool rejects values of the wrong type.

A number field might receive "42"  (string) instead of  42  (number). Type validation tests ensure your server catches these mistakes rather than causing cryptic errors or incorrect behavior downstream.


### What's generated:

String fields receiving numbers Number fields receiving strings Boolean fields receiving truthy strings like  "true" Array fields receiving comma-separated strings Object fields receiving primitives

```text
# Generated: query_types.yaml
```

```yaml
steps:
  # Wrong type for string field
  - name: "Type error: sql should be string, got number"
    tool: query
    input:
      sql: 12345
    expect:
      error:
        code: -32602

  # Wrong type for number field
  - name: "Type error: limit should be integer, got string"
    tool: query
    input:
      sql: "SELECT 1"
      limit: "one hundred"
    expect:
      error:
        code: -32602

  # Wrong type for boolean field
  - name: "Type error: verbose should be boolean, got string"
    tool: query
    input:
      sql: "SELECT 1"
      verbose: "true"  # String, not boolean
    expect:
      error:
        code: -32602

  # Wrong type for array field
  - name: "Type error: columns should be array, got string"
    tool: query
    input:

      sql: "SELECT 1"
      columns: "id,name"  # String, not array
    expect:
      error:
        code: -32602

  # Null for non-nullable field
  - name: "Type error: sql cannot be null"
    tool: query
    input:
      sql: null
    expect:
      error:
        code: -32602
```


### Customizing Generated Tests

Generated tests cover schema constraints, but they can't know your business logic. A query tool's schema might allow any SELECT statement, but your business rules might require specific table access patterns.


### The workflow:

1. Create custom test files for scenarios the generator can't know about 4. Use override files to replace generated tests when needed


### Editing Generated Files

Generated tests are a starting point—they verify schema compliance but not business correctness.

```text
# tests/scenarios/generated/query_valid.yaml (edited)
```

```text
# Add: edited marker to prevent regeneration overwrite
```


### edited: true

```yaml
steps:
  # Keep generated steps...

  # ADD: Custom test for specific business logic
  - name: "Query with JOIN (business requirement)"
    tool: query
    input:
      sql: "SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id"
    expect:
      success: true
      type:
        rows: array

  # ADD: Test for specific column selection
  - name: "Query specific columns"
    tool: query
    input:
      sql: "SELECT id, name, email FROM users"
      columns: ["id", "name", "email"]
    expect:
      contains:
        column_count: 3
```


### Override Files

When you need to significantly customize generated tests, use override files instead of editing the generated files directly. This keeps your customizations safe when you regenerate tests after schema changes.


### tests/scenarios/

```text
generated/              # Auto-generated
    query_valid.yaml
    query_invalid.yaml
overrides/              # Manual overrides (higher priority)
    query_valid.yaml    # Replaces generated version
custom/                 # Additional custom tests
    query_security.yaml
```

```text
# tests/scenarios/overrides/query_valid.yaml
```

name: "query - Valid Inputs (Custom)" description: "Customized valid input tests with business-specific cases"

```yaml
# Include steps from generated file
include:
  - ../generated/query_valid.yaml

# Add additional steps
steps:
  - name: "Complex business query"
    tool: query
    input:
      sql: "SELECT * FROM quarterly_reports WHERE year = 2024"
    expect:
      success: true
```


### Regeneration Strategy

```bash
# Regenerate only, don't overwrite edited files
cargo pmcp test generate \
  --server http://localhost:3000 \
  --skip-edited

# Force regenerate everything
cargo pmcp test generate \
  --server http://localhost:3000 \
  --force

# Regenerate and show diff
cargo pmcp test generate \
  --server http://localhost:3000 \
  --diff

# Merge new tests with existing
cargo pmcp test generate \
  --server http://localhost:3000 \
  --merge
```


### Advanced Schema Patterns

Real-world schemas are rarely flat.


### Understanding these patterns helps you:

1.


### Nested Object Schemas

Nested objects require testing at each level: the parent object, child objects, and the relationship between them.

```json
{
  "type": "object",
  "properties": {
    "user": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "address": {
          "type": "object",
          "properties": {
            "city": { "type": "string" },
            "zip": { "type": "string", "pattern": "^\\d{5}$" }
          },
          "required": ["city"]
        }
      },
      "required": ["name"]
    }
  },
  "required": ["user"]
}
```


### Generated tests:

```yaml
steps:
  # Valid nested object
  - name: "Valid nested object"
    tool: create_user
    input:
      user:
        name: "Alice"
        address:
          city: "New York"
          zip: "10001"
    expect:
      success: true

  # Missing nested required field
  - name: "Missing nested required: user.name"
    tool: create_user
    input:
      user:
        address:
          city: "New York"
    expect:
      error:
        code: -32602

  # Missing deeply nested required
  - name: "Missing deeply nested required: user.address.city"
    tool: create_user
    input:
      user:
        name: "Alice"
        address:
          zip: "10001"
    expect:
      error:
        code: -32602

  # Pattern violation in nested field
  - name: "Pattern violation: user.address.zip"
    tool: create_user

    input:
      user:
        name: "Alice"
        address:
          city: "New York"
          zip: "invalid"
    expect:
      error:
        code: -32602
```


### Array Item Schemas

Arrays of objects are common (order items, user roles, configuration entries). Tests must verify: the array itself (length constraints), and each item within the array (item-level constraints). A single invalid item should cause the entire request to fail.

```json
{
  "type": "object",
  "properties": {
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "quantity": { "type": "integer", "minimum": 1 }
        },
        "required": ["id", "quantity"]
      },
      "minItems": 1,
      "maxItems": 100
    }
  },
  "required": ["items"]
}
```


### Generated tests:

```yaml
steps:
  # Valid array
  - name: "Valid array with items"
    tool: process_order
    input:
      items:
        - id: 1
          quantity: 5
        - id: 2
          quantity: 3
    expect:
      success: true

  # Empty array (violates minItems)
  - name: "Empty array violates minItems"
    tool: process_order
    input:
      items: []
    expect:
      error:
        code: -32602

  # Array item missing required field
  - name: "Array item missing required: quantity"
    tool: process_order
    input:
      items:
        - id: 1
          # quantity missing
    expect:
      error:
        code: -32602

  # Array item constraint violation
  - name: "Array item constraint: quantity below minimum"
    tool: process_order
    input:
      items:
        - id: 1

          quantity: 0  # minimum is 1
    expect:
      error:
        code: -32602

  # Array exceeds maxItems
  - name: "Array exceeds maxItems (100)"
    tool: process_order
    input:
      items: [/* 101 items */]
    expect:
      error:
        code: -32602
```


### oneOf/anyOf/allOf Schemas

Polymorphic schemas allow different structures for the same field. A payment might be a credit card OR a bank transfer—each with different required fields. These are powerful but tricky: tests must verify each variant works, that invalid variants are rejected, and that each variant's constraints are enforced.

oneOf: Exactly one subschema must match (use for mutually exclusive options) anyOf: At least one subschema must match (use for flexible alternatives) allOf: All subschemas must match (use for combining constraints)

```json
{
  "type": "object",
  "properties": {
    "payment": {
      "oneOf": [
        {
          "type": "object",
          "properties": {
            "type": { "const": "credit_card" },
            "card_number": { "type": "string" }
          },
          "required": ["type", "card_number"]
        },
        {
          "type": "object",
          "properties": {
            "type": { "const": "bank_transfer" },
            "account_number": { "type": "string" }
          },
          "required": ["type", "account_number"]
        }
      ]
    }
  }
}
```


### Generated tests:

```yaml
steps:
  # Valid: first oneOf option
  - name: "Valid oneOf: credit_card"
    tool: process_payment
    input:
      payment:
        type: "credit_card"
        card_number: "4111111111111111"
    expect:
      success: true

  # Valid: second oneOf option
  - name: "Valid oneOf: bank_transfer"
    tool: process_payment
    input:
      payment:
        type: "bank_transfer"
        account_number: "123456789"
    expect:
      success: true

  # Invalid: matches neither oneOf
  - name: "Invalid oneOf: unknown type"
    tool: process_payment
    input:
      payment:
        type: "cash"
    expect:
      error:
        code: -32602

  # Invalid: missing field for matched oneOf
  - name: "Invalid oneOf: credit_card missing card_number"
    tool: process_payment
    input:
      payment:
        type: "credit_card"
        # card_number missing
    expect:

      error:
        code: -32602
```


### CI/CD Pipeline Integration

Schema-driven testing shines in CI/CD pipelines.


### Complete GitHub Actions Workflow

This workflow demonstrates a complete setup: build the server, generate tests from the current schema, check for unexpected schema changes, run all tests, and report results.

```text
# .github/workflows/mcp-tests.yml
```


### name: MCP Server Tests

```bash
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM

jobs:
  generate-and-test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |

            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-pmcp
        run: cargo install cargo-pmcp

      - name: Build server
        run: cargo build --release

      - name: Start server
        run: |
          ./target/release/my-mcp-server &
          echo $! > server.pid
          sleep 5
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost/test

      - name: Generate tests from schema
        run: |
          cargo pmcp test generate \
            --server http://localhost:3000 \
// ... truncated, 43 more lines in source book ...
```


### Schema Change Detection

This specialized workflow catches unintentional schema changes.

```text
# .github/workflows/schema-check.yml
```


### name: Schema Change Detection

```bash
on:
  pull_request:
    paths:
      - 'src/**'

jobs:
  check-schema:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install tools
        run: cargo install cargo-pmcp

      - name: Build and start server
        run: |
          cargo build --release
          ./target/release/my-mcp-server &
          sleep 5

      - name: Generate current schema tests
        run: |
          cargo pmcp test generate \
            --server http://localhost:3000 \
            --output tests/scenarios/current/

      - name: Compare with committed tests
        run: |
          if ! diff -r tests/scenarios/generated/ tests/scenarios/current/; then
            echo "::error::Schema has changed! Update tests with: cargo pmcp test generate"
            exit 1
          fi
```


### Best Practices

These practices help you maintain a healthy balance between automated generation and manual customization.


### 1. Version Control Strategy

A key decision: should generated tests be committed to version control?


### tests/scenarios/

```text
generated/              # Add to .gitignore OR commit baseline
    .gitkeep
custom/                 # Always commit
    security/
    performance/
    workflows/
regression/             # Always commit
    issue_fixes/
```

```text
# Commit generated tests, regenerate on schema changes
# Use PR checks to detect drift
```


### 2. Test Organization

Tags help you run subsets of tests for different purposes.

```bash
# Use tags for filtering
tags:
  - smoke         # Quick sanity tests
  - regression    # Bug fix verification
  - security      # Security-focused
  - performance   # Performance requirements
  - integration   # Multi-step workflows

# Run subsets
cargo pmcp test run --tags smoke
cargo pmcp test run --tags security,regression
```


### 3. Maintenance Workflow

Schema-driven tests require periodic maintenance: regenerating after schema changes, adding regression tests for bugs, and reviewing generated tests for relevance.

```bash
# Weekly: regenerate and review
cargo pmcp test generate --diff

# On schema change: update baseline
cargo pmcp test generate --force
git add tests/scenarios/generated/
git commit -m "Update generated tests for schema change"

# On bug fix: add regression test
vim tests/scenarios/regression/issue_123.yaml
git add tests/scenarios/regression/
git commit -m "Add regression test for issue #123"
```


### Key commands:

```bash
# Generate tests
cargo pmcp test generate --server http://localhost:3000

# Generate with deep edge cases
cargo pmcp test generate --server http://localhost:3000 --edge-cases deep

# Check for changes
cargo pmcp test generate --diff

# Run generated tests
cargo pmcp test run --server http://localhost:3000
```


### Practice Ideas

These informal exercises help reinforce the concepts.
