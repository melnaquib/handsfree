---
chapter: 28
title: "Code Mode: Validated LLM Code Execution"
part: "Part VIII: Advanced Patterns"
use_when: "Letting LLMs submit validated code (GraphQL/OpenAPI/SQL) via validate_code/execute_code with policy checks."
suggested_prerequisites: [ch19]
source_kb: 29
extracted_kb: 12
code_blocks: 17
quality: auto-extracted
---

# Ch 28: Code Mode: Validated LLM Code Execution

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Code Mode: Validated LLM Code Execution

Code Mode is PMCP's framework for safely executing LLM-generated code (GraphQL queries, JavaScript plans, SQL statements, MCP tool compositions) with cryptographic approval tokens.

Add Code Mode to any MCP server using  #[derive(CodeMode)]  with the four required field names ( code_mode_config , token_secret ,  policy_evaluator ,  code_executor ). Identify when  NoopPolicyEvaluator  is appropriate (local development only) and when production must replace it.


### Why Code Mode Matters for Enterprise MCP

When an LLM generates code to run against your backend, three questions arise: Is it safe?

Consistent security boundaries across many servers — a fleet of GraphQL, SQL, JS, and MCP-composition servers should share

man-in-the-middle) cannot quietly substitute a different one.

```text
validate_code(code)       ► ValidationPipeline
                                  Parse (language-specific)
                                  Security scan
                                  Policy evaluation (Cedar/AVP/custom)
                                  Explain (human-readable)
                                  HMAC-sign
```

```text
approval_token ◄             (code hash + user + session + expiry)
```


### User reviews explanation, approves

```text
execute_code(code, token)  ► Token verification
                                  Signature check
                                  Code hash match
                                  Expiry check
                                  Context match
```

```rust
        ▼
CodeExecutor::execute(code) → Result
```

Each pipeline stage is a separate, observable step. The HMAC-SHA256 token binding is the load-bearing security property: any change to the code after validation produces a hash mismatch at execute time, so the token cannot be reused for a tampered query.


### Adding Code Mode with #[derive(CodeMode)]

These are the five mechanical steps that take an existing PMCP server and add Code Mode to it. The canonical path uses the derive macro — manual handler registration exists as an escape hatch for advanced cases, but you should not start there.


### Step 1: Add Dependencies

```toml
[dependencies]
pmcp = { version = "2.7.0", features = ["full"] }
pmcp-code-mode = "0.5.1"
pmcp-code-mode-derive = "0.3.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```


### Step 2: Derive and Configure

Define a server struct with the four required field names and annotate it with  #[derive(CodeMode)] :


### use pmcp_code_mode::{

```rust
    CodeExecutor, CodeModeConfig, ExecutionError, NoopPolicyEvaluator, TokenSecret,
    ValidationContext, ValidationPipeline,
};
use pmcp_code_mode_derive::CodeMode;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(CodeMode)]
struct MyGraphQLServer {
    code_mode_config: CodeModeConfig,
    token_secret: TokenSecret,
    policy_evaluator: Arc<NoopPolicyEvaluator>,
    code_executor: Arc<GraphQLExecutor>,
}
```

The four field names are a convention (v0.1.0 of the derive crate): renaming any of them breaks compilation.

```rust
#[derive(CodeMode)]
#[code_mode(language = "javascript")]
```

(table/list, cells in reading order)
- struct MyJavaScriptServer { /* same four fields */ }
- Step 3: Pick a Language
- The  language  attribute selects the validation path at compile time:
- Language
- Attribute value
- Feature flag required
- GraphQL
- "graphql"  (default)
- (none)
- JavaScript/OpenAPI
- "javascript"
- openapi-code-mode
- Language
- Attribute value
- Feature flag required
- SQL
- MCP composition
- "sql"
- "mcp"
- sql-code-mode
- mcp-code-mode

The other three are opt-in to keep compile times sharp for servers that don't need them.


### Step 4: Choose Your Executor

The  code_executor  field is  Arc<impl CodeExecutor> .

```rust
struct GraphQLExecutor;

#[pmcp_code_mode::async_trait]
impl CodeExecutor for GraphQLExecutor {
    async fn execute(
        &self,
        code: &str,
        _variables: Option<&Value>,
    ) -> Result<Value, ExecutionError> {
        // In production, this would execute the GraphQL query against a real backend.
        // For this example, return mock data based on the query.
        Ok(json!({
            "data": {
                "query": code,
                "result": [
                    {"id": "1", "name": "Alice"},
                    {"id": "2", "name": "Bob"},
                ]
            }
        }))
    }
}
```

```rust
// JS + HTTP — Cost Coach calling REST APIs
let executor = Arc::new(JsCodeExecutor::new(http_client, ExecutionConfig::default()));

// JS + SDK — direct AWS SDK calls
let executor = Arc::new(SdkCodeExecutor::new(sdk_client, ExecutionConfig::default()));

// MCP tool composition — routing to other MCP servers
let executor = Arc::new(McpCodeExecutor::new(mcp_router, ExecutionConfig::default()));
```

Choose a direct impl only when none of the three adapters fit.


### Step 5: Register on the Builder and Build

```rust
#[tokio::main]
async fn main() {
    let server = MyGraphQLServer {
        code_mode_config: CodeModeConfig::enabled(),
        token_secret: TokenSecret::new(b"example-secret-key-32-bytes!!!!".to_vec()),
        policy_evaluator: Arc::new(NoopPolicyEvaluator::new()),
        code_executor: Arc::new(GraphQLExecutor),
    };

    let builder = pmcp::Server::builder();
    let _builder = server
        .register_code_mode_tools(builder)
        .expect("Failed to register code mode tools");
    println!("Registered validate_code and execute_code tools on builder.");
    // In a real server, add more tools, then `.build()` and `.run_stdio()`.
}
```

After  register_code_mode_tools(builder) , the builder exposes  validate_code  and  execute_code  as standard MCP tools.


### Worked Example: GraphQL Round-Trip (s41_code_mode_graphql.rs)

The worked example in  examples/s41_code_mode_graphql.rs  runs both the success path and the rejection path in a single binary so you can see both behaviors in one run.


### Success path — a valid GraphQL query

```rust
// --- SUCCESS PATH ---
println!("--- Success Path: Valid GraphQL Query ---");
let query = "query { users { id name } }";
println!("Query: {query}");

match pipeline.validate_graphql_query(query, &context) {
    Ok(result) => {
        println!("Validation: PASSED (is_valid={})", result.is_valid);
        println!("Risk level: {}", result.risk_level);
        println!(
            "Approval token: {}...",
            result
                .approval_token
                .as_ref()
                .map(|t| &t[..t.len().min(20)])
                .unwrap_or("none")
        );
        // Execute with the approval token
        if result.approval_token.is_some() {
            let exec_result = server.code_executor.execute(query, None).await;
            // ...
        }
    },
    Err(e) => { println!("Validation: FAILED - {e:?}"); },
}
```

The query passes parsing, security scan, and policy evaluation; the pipeline returns a  ValidationResult  whose  approval_token  is

```text
Some(...) . The server then calls  code_executor.execute(query, None)  with the validated query and observes a mock JSON result. In a
```


### Rejection path — a mutation under default config

```rust
// --- REJECTION PATH ---
println!("\n--- Rejection Path: Mutation Blocked by Config ---");
let mutation = "mutation { createUser(name: \"evil\") { id } }";
println!("Query: {mutation}");

match pipeline.validate_graphql_query(mutation, &context) {
    Ok(result) => {
        if result.is_valid {
            println!("Validation: PASSED (unexpected for mutation with default config)");
        } else {
            println!("Validation: REJECTED (expected)");
            for violation in &result.violations {
                println!("  Violation: {} - {}", violation.rule, violation.message);
            }
            println!("This demonstrates that mutations do NOT receive an approval token.");
            println!(
                "Approval token present: {}",
                result.approval_token.is_some()
            );
        }
    },
    Err(e) => {
        println!("Validation: REJECTED (expected) - {e:?}");
    },
}
```

This is the load-bearing security property in action. To allow mutations, you must explicitly opt in by constructing  CodeModeConfig { allow_mutations: true, ..CodeModeConfig::enabled() } ; doing so is a deliberate, auditable policy decision rather than the default.


### Configuration in config.toml (Platform-Level Policy)

The  config.toml  file separates "what the runtime allows" ( CodeModeConfig , set in Rust at server-start time) from "what the platform permits" (declared in  config.toml , surfaced in the pmcp.run admin UI). When you deploy with  cargo pmcp deploy , the  config.toml  is included in the deploy ZIP and read by the platform to populate the Code Mode policy page.


### OpenAPI example

```toml
[server]
name = "cost-coach"
type = "openapi-api"

[code_mode]
allow_writes = false
allow_deletes = false

[[code_mode.operations]]
name = "getCostAndUsage"
description = "Retrieve AWS cost and usage data"
path = "/ce/GetCostAndUsage"
method = "POST"

[[code_mode.operations]]
name = "deleteBudget"
description = "Delete a budget"
path = "/budgets/DeleteBudget"
method = "POST"
destructive_hint = true
```


### GraphQL example

```toml
[server]
name = "open-images"
type = "graphql-api"

[code_mode]
allow_writes = false

[[code_mode.operations]]
name = "searchImages"
operation_type = "query"

[[code_mode.operations]]
name = "deleteImage"
operation_type = "mutation"
destructive_hint = true
```


### SQL example

```toml
[server]
name = "analytics"
type = "sql"

[code_mode]
allow_writes = true
allow_deletes = false
blocked_tables = ["audit_log", "credentials"]

[database]
[[database.tables]]
name = "orders"
description = "Customer order history"

[[database.tables]]
name = "products"
description = "Product catalog"
```


### Categorization

Operations are sorted into read, write, delete, and admin categories based on:

(table/list, cells in reading order)
- Server type
- Read
- Write
- Delete
- OpenAPI
- GraphQL
- SQL
- GET
- query
- SELECT
- POST  /  PUT
- mutation  (write)
- DELETE
- mutation  (delete via  destructive_hint )
- INSERT  /  UPDATE  (gated by  allow_writes )
- DELETE  (gated by  allow_deletes )
- MCP-API
- by annotation
- by annotation
- by annotation

The  operation_category  field overrides automatic categorization when you need explicit control.


### Policy Evaluation (Cedar / AVP / Custom)

Code Mode supports pluggable per-request authorization through the  PolicyEvaluator  trait. A denied operation never receives a token; therefore it never reaches execute_code .


### Cedar (local, no network)

```rust
// Cedar feature flag: pmcp-code-mode = { version = "0.5.1", features = ["cedar"] }
let evaluator = Arc::new(CedarPolicyEvaluator::new(policy_set));
```

Use Cedar when you want fast, in-process policy decisions and your policies are static or refreshed periodically.


### AWS Verified Permissions (AVP, remote)

```rust
// Provided by the pmcp-code-mode-avp companion crate.
let evaluator = Arc::new(AvpPolicyEvaluator::new(avp_client, policy_store_id));
```

Use AVP when you want centralized policy management across many services and per-request audit trails in CloudTrail.


### Custom

```rust
let evaluator = Arc::new(MyCustomEvaluator::new(authz_client));
```

Implement the  PolicyEvaluator  trait yourself when you have an existing authorization backend (Open Policy Agent, Spicedb, internal RPC, etc.).


### Security Properties Reference

Tokens are stateless — verified via HMAC signature, not looked up in a server-side store. Production deployments should use 32+ bytes.


### Hands-On (Run the Example)

Three follow-on exercises in Chapter 22 Exercises give you hands-on practice with the concepts above.

```bash
cargo run --example s41_code_mode_graphql --features full
```


### What to observe in the output:

The success path prints  Validation: PASSED , a risk level, an approval token prefix (first 20 characters), and a mock GraphQL result with two users.


### Modify-and-experiment tips:

Change  code_mode_config: CodeModeConfig::enabled()  to  code_mode_config: CodeModeConfig { allow_mutations: true, ..CodeModeConfig::enabled() }  and re-run.

Required field names.

NoopPolicyEvaluator caveat. Why does the s41 example use  NoopPolicyEvaluator , and what must change for production?

When you can answer all five from memory, head to Chapter 22 Exercises.
