---
chapter: 22
title: "Effective AI Collaboration"
part: "Part VI: AI-Assisted Development"
use_when: "Workflow and prompting patterns for AI-built MCP tools; QA with scenarios; alternative assistants."
suggested_prerequisites: [ch21]
source_kb: 58
extracted_kb: 32
code_blocks: 68
quality: auto-extracted
---

# Ch 22: Effective AI Collaboration

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Effective AI Collaboration

With your AI assistant configured (Chapter 15), this chapter focuses on making your collaboration productive.


### The Collaboration Model

```text
               Effective AI Collaboration
                      You (Developer)
• Define WHAT to build (business requirements)
• Provide domain knowledge (API constraints, data models)
• Make architectural decisions (transport, security)
• Review generated code (ownership and understanding)
               Clear Communication
                   AI Assistant
• Generates HOW to build (code implementation)
• Applies cargo-pmcp patterns (scaffolding, testing)
• Handles boilerplate (types, error handling, serialization)
• Iterates on compiler feedback (until quality gates pass)
                 Quality Validation
                  Rust Compiler + Tooling
• Type checking (catches errors at compile time)
• Borrow checking (memory safety guarantees)
• Clippy linting (code quality enforcement)
• Test runner (behavior verification)
```

(table/list, cells in reading order)
- The Three Pillars
- 1. Structured Workflow
- A predictable workflow reduces ambiguity:
- Request → Scaffold → Implement → Validate → Deploy

```text
         AI handles these steps
         You provide direction
```

(table/list, cells in reading order)
- 2. Effective Communication
- Good prompts lead to good code:
- Poor Prompt
- Better Prompt
- "Make an API server"
- "Create an MCP server that queries the GitHub API"
- "Add database stuff"
- "Add a  list_tables  tool that returns table names and row counts"
- "Fix the bug"

"The  get_user  tool returns 500 when the user doesn't exist. It should return a validation error."

```bash
# AI iterates until all pass
cargo fmt --check     # [ok] Formatting
cargo clippy          # [ok] Linting
cargo test            # [ok] Unit tests
cargo pmcp test       # [ok] Integration tests
```

```text
                              ^                        |
                              |________________________|
                                    Slow feedback
```

```text
            ^          |          |        |

            |              Fast iteration
```

```text
cargo-pmcp handles structure
```


### Key differences:

1.

What tools should the server provide? What data should be accessible?


### 2. Domain Knowledge

API authentication methods Data validation rules Business logic constraints

(table/list, cells in reading order)
- 3. Architectural Decisions
- Transport mode (stdio vs HTTP) Security requirements Deployment target
- 4. Code Review
- Understanding what was generated Catching logical errors
- Ensuring maintainability
- AI Is Responsible For
- 1. Code Generation
- Type definitions Handler implementations Error handling boilerplate
- 2. Pattern Application
- TypedTool structure JsonSchema derives cargo-pmcp conventions
- 3. Iteration

(table/list, cells in reading order)
- 4. Documentation
- Inline comments API documentation Usage examples
- Working Sessions
- Short Sessions (15-30 minutes)
- Good for:


### Pattern:

"Add a search_users tool to the GitHub server that takes a query string and returns matching usernames"

(table/list, cells in reading order)
- Medium Sessions (1-2 hours)
- Good for:
- Creating a new server Implementing a feature set Major refactoring
- Pattern:


### Pattern:

"Build a complete CI/CD MCP server that: 1.

- Anti-Patterns to Avoid


### Good:

"Create a weather tool that fetches current temperature for a city.

(table/list, cells in reading order)
- 2. Vague Requirements
- Bad:
- "Make a database thing"
- Good:

(table/list, cells in reading order)
- 3. Ignoring Compiler Feedback
- Bad:
- User: "That doesn't work" AI: "Let me try something else entirely"
- Good:

```text
     Let me fix that specific problem."
```

(table/list, cells in reading order)
- 4. Skipping Quality Gates
- Bad:
- User: "Just make it compile, we'll fix warnings later"
- Good:
- User: "Run cargo clippy and fix all warnings before

```text
      we consider this done"
```

(table/list, cells in reading order)
- Chapter Overview
- This chapter covers three key topics:
- The Development Workflow
- The step-by-step cargo-pmcp workflow:

(table/list, cells in reading order)
- Quality Assurance with AI
- Ensuring production-quality output:
- Automated quality gates Test generation
- Code review patterns Common issue resolution
- Summary
- Effective AI collaboration requires:


### The Development Workflow

This chapter walks through the complete cargo-pmcp workflow for AI-assisted MCP server development.


### The Standard Workflow

```text
              cargo-pmcp Development Workflow
1. Scaffold         cargo pmcp new <workspace>
   Workspace        cargo pmcp add server <name> --template minimal
2. Implement        Edit crates/mcp-<name>-core/src/tools/*.rs
   Tools            Register tools in lib.rs
3. Development      cargo pmcp dev --server <name>
   Server           Hot-reload on http://0.0.0.0:3000
4. Generate         cargo pmcp test --server <name>
   Tests                --generate-scenarios
5. Run Tests        cargo pmcp test --server <name>
                    cargo test
6. Quality          cargo fmt --check
```

```text
   Gates            cargo clippy -- -D warnings
7. Production       cargo build --release
   Build            Configure MCP client
```

(table/list, cells in reading order)
- Step 1: Scaffold Workspace
- Create New Workspace
- Request to AI:
- Create a new MCP workspace called "weather-mcp-workspace"
- AI executes:

```bash
cargo pmcp new weather-mcp-workspace
```

```text
Cargo.toml              # Workspace manifest
Cargo.lock
.gitignore
README.md
crates/
    server-common/      # Shared transport helpers
        Cargo.toml
        src/
            lib.rs
            http.rs
scenarios/              # Test scenarios directory
```

(table/list, cells in reading order)
- Add Server to Workspace
- Request to AI:
- Add a weather server using the minimal template
- AI executes:

```bash
cargo pmcp add server weather --template minimal
```

```text
server-common/
mcp-weather-core/       # NEW - Library crate
    Cargo.toml
    src/
        lib.rs          # Server builder
        tools/          # Tool implementations
            mod.rs
        resources/      # Resource handlers
        workflows/      # Workflow definitions
weather-server/         # NEW - Binary crate
    Cargo.toml
    src/
        main.rs         # Entry point
```


### scenarios/

```text
weather/               # NEW - Test scenarios
```

(table/list, cells in reading order)
- Template Options
- Template
- Use Case
- minimal
- calculator
- Custom servers
- Learning MCP
- Includes
- Empty structure
- Single add tool
- complete_calculator
- Reference implementation
- 5 tools, tests, prompts
- sqlite_explorer
- Database servers
- DB connection, queries

Rule: Always use templates. Never create this structure manually.

(table/list, cells in reading order)
- Step 2: Implement Tools
- Tool File Structure
- Each tool lives in its own file:
- crates/mcp-weather-core/src/tools/

```text
mod.rs              # Module exports
get_current.rs      # Current weather tool
get_forecast.rs     # Forecast tool
```


### AI creates  tools/get_current.rs :

```rust
use pmcp::{Error, RequestHandlerExtra, Result, TypedTool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input for current weather lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GetCurrentInput {
    /// City name (e.g., "London", "New York")
    #[schemars(description = "City name to get weather for")]
    pub city: String,
}

/// Current weather data
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetCurrentOutput {
    /// City name
    pub city: String,
    /// Temperature in Celsius
    pub temperature_celsius: f64,
    /// Relative humidity percentage
    pub humidity: u8,
    /// Weather conditions (e.g., "Sunny", "Cloudy")
    pub conditions: String,
}

async fn handler(
    input: GetCurrentInput,
    _extra: RequestHandlerExtra,
) -> Result<GetCurrentOutput> {
    // Validate input
    if input.city.trim().is_empty() {
        return Err(Error::validation("City name cannot be empty"));
    }

    // In production, call weather API here
    // For now, return mock data
    Ok(GetCurrentOutput {
        city: input.city,

        temperature_celsius: 18.5,
        humidity: 65,
        conditions: "Partly Cloudy".to_string(),
    })
}

pub fn build_tool() -> TypedTool<GetCurrentInput, GetCurrentOutput> {
    TypedTool::new("get-current-weather", |input, extra| {
        Box::pin(handler(input, extra))
    })
    .with_description("Get current weather conditions for a city")
}
```

```rust
pub mod tools;

use pmcp::{McpServer, Result};

pub fn build_weather_server() -> Result<McpServer> {
    McpServer::builder("weather-server", "1.0.0")
        .tool("get-current-weather", tools::get_current::build_tool())
        .build()
}
```

```text
pub mod get_current;
```

(table/list, cells in reading order)
- Step 3: Development Server
- Start Hot-Reload Server
- Request to AI:
- Start the development server for weather
- AI executes:

```bash
cargo pmcp dev --server weather
```


### Output:

Building weather-server...

```text
   Compiling mcp-weather-core v1.0.0
   Compiling weather-server v1.0.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.34s
```

```text
  - tools: get-current-weather
```

(table/list, cells in reading order)
- Watching for changes... [INFO] Server ready to accept connections
- Iterating with Hot-Reload
- When you request changes:
- Add validation for city name length (max 100 characters)
- AI edits the tool, hot-reload automatically rebuilds:

```text
   Compiling mcp-weather-core v1.0.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.89s
```

```bash
cargo pmcp dev --server weather --port 8080
```

(table/list, cells in reading order)
- Step 4: Generate Test Scenarios
- Auto-Generate from Server
- Request to AI:
- Generate test scenarios for the weather server
- AI executes (in another terminal):

```bash
cargo pmcp test --server weather --generate-scenarios
```


### Generated  scenarios/weather/generated.yaml :

name: "Weather Server Tests" description: "Auto-generated tests for weather server" timeout: 60 stop_on_failure: false

```yaml
steps:
  - name: "Test get-current-weather with valid city"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "London"
    assertions:
      - type: success
      - type: field_exists
        path: "content.0.text"

  - name: "Test get-current-weather with empty city"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: ""
    assertions:
      - type: error
      - type: contains
        path: "error.message"
        value: "cannot be empty"
```

```yaml
  - name: "Test Unicode city name"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "東京"  # Tokyo in Japanese
    assertions:
      - type: success

  - name: "Test very long city name"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "A very long city name that exceeds the maximum..."
    assertions:
      - type: error
```

```bash
# Dev server must be running
cargo pmcp test --server weather
```


### Output:

Running scenarios for weather server...


### Scenario: Weather Server Tests

```json
  [ok] Test get-current-weather with valid city (15ms)
  [ok] Test get-current-weather with empty city (8ms)
  [ok] Test Unicode city name (12ms)
  [ok] Test very long city name (7ms)
```

(table/list, cells in reading order)
- Results: 4 passed, 0 failed
- Unit Tests
- Request to AI:
- Add unit tests for the get_current handler
- AI adds to  tools/get_current.rs :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_city() {
        let input = GetCurrentInput {
            city: "London".to_string(),
        };
        let result = handler(input, RequestHandlerExtra::default()).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.city, "London");
    }

    #[tokio::test]
    async fn test_empty_city() {
        let input = GetCurrentInput {
            city: "".to_string(),
        };
        let result = handler(input, RequestHandlerExtra::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitespace_only_city() {
        let input = GetCurrentInput {
            city: "   ".to_string(),
        };
        let result = handler(input, RequestHandlerExtra::default()).await;
        assert!(result.is_err());
    }
}
```


### Run unit tests:

```bash
cargo test
```

```bash
cargo fmt --check
```


### If formatting issues exist:

```bash
cargo fmt  # Auto-fix
```


### Clippy Linting

```bash
cargo clippy -- -D warnings
```


### Common issues AI fixes:

```rust
// Before: Clippy warning about redundant clone
let city = input.city.clone();

// After: AI fix
let city = input.city;
```

(table/list, cells in reading order)
- All Quality Gates
- Request to AI:
- Run all quality gates and fix any issues
- AI executes:

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

If any fail, AI iterates until all pass.

(table/list, cells in reading order)
- Makefile Integration
- For projects with Makefile:
- make quality-gate
- Step 7: Production Build
- Release Build

```bash
cargo build --release
```

(table/list, cells in reading order)
- Binary location:
- target/release/weather-server
- Configure MCP Client
- Claude Code ( ~/.claude/mcp_servers.json ):

```json
{
  "weather": {
    "command": "/path/to/weather-mcp-workspace/target/release/weather-server",
    "args": [],
    "env": {
      "WEATHER_API_KEY": "${env:WEATHER_API_KEY}"
    }
  }
}
```


### Kiro ( .kiro/settings.json ):

```json
{
  "mcpServers": {
    "weather": {
      "command": "/path/to/weather-server",
      "args": []
    }
  }
}
```

Create an MCP server for managing Kubernetes pods.


### AI Workflow

```bash
# Step 1: Scaffold
$ cargo pmcp new k8s-mcp-workspace
$ cd k8s-mcp-workspace
$ cargo pmcp add server k8s --template minimal

# Step 2: Implement (AI edits files)
# Creates: list_pods.rs, get_pod.rs, get_logs.rs

# Step 3: Dev server
$ cargo pmcp dev --server k8s

# Step 4: Generate tests (in another terminal)
$ cargo pmcp test --server k8s --generate-scenarios

# Step 5: Run tests
$ cargo pmcp test --server k8s
$ cargo test

# Step 6: Quality gates
$ cargo fmt --check
$ cargo clippy -- -D warnings

# Step 7: Build
$ cargo build --release
```

- User: The list_pods tool should filter by namespace

AI: I'll update the input type to accept an optional namespace parameter.

```bash
$ cargo build  # Check compilation
$ cargo test   # Verify behavior
$ cargo clippy -- -D warnings  # Quality check
```

```text
New project?
   Yes → cargo pmcp new <workspace>
           cargo pmcp add server <name> --template minimal
Add server to existing workspace?
   Yes → cargo pmcp add server <name> --template <template>
Add tool to existing server?
   Yes → cargo pmcp add tool <tool> --server <server>
         (or manually create in tools/)
Test changes?
   Yes → cargo pmcp dev --server <name>  (terminal 1)
         cargo pmcp test --server <name>  (terminal 2)
Ready for production?
```

```bash
        Yes → cargo fmt --check
              cargo clippy -- -D warnings
              cargo test
              cargo build --release
```

Scaffold - Never create files manually 2.


### Prompting for MCP Tools

Effective prompts lead to better code faster.


### The Anatomy of a Good Prompt

```text
                Effective MCP Tool Prompt
1. CONTEXT
   "Create an MCP server for [domain]..."
   Sets the problem space and technology
2. CAPABILITY
   "...with tools that [action] and [action]..."
   Describes what the server should do
3. CONSTRAINTS
   "...limit [X], require [Y], return [Z]..."
   Sets boundaries and requirements
4. EXAMPLES (Optional)
   "For example, when given [input], return [output]"
   Clarifies expected behavior
```

Create a tool that converts temperatures between Celsius and Fahrenheit.


### What AI generates:

Input type with temperature (f64) and unit (enum) Output type with both conversions Validation for reasonable temperature range Unit tests

Input: - query: String (required) - search pattern (regex supported) - start_time: DateTime (optional) - earliest log timestamp - end_time: DateTime (optional) - latest log timestamp - level: enum [DEBUG, INFO, WARN, ERROR] (optional) - filter by level - limit: u32 (optional, default 100, max 1000) - result count


### What AI generates:

Complete input/output types with all fields Regex validation with helpful error messages Time range validation Pagination handling Comprehensive test coverage


### Tools: 1. list_workflows - Get all workflows for a repo

```text
   - Input: owner, repo
   - Output: workflow id, name, state, path
```


### 2. get_workflow_runs - Get recent runs for a workflow

```text
   - Input: owner, repo, workflow_id, status filter (optional)
   - Output: run id, status, conclusion, started_at, duration
```


### 3. trigger_workflow - Start a workflow run

```text
   - Input: owner, repo, workflow_id, ref (branch), inputs (map)
   - Output: run id, url
   - IMPORTANT: Require confirmation in description
```


### 4. cancel_run - Cancel an in-progress run

```text
   - Input: owner, repo, run_id
   - Output: success boolean
```

Architecture: - Use octocrab crate for GitHub API - Token from GITHUB_TOKEN env var - Rate limiting: implement retry with backoff - All times in UTC

Security: - trigger_workflow must log the action - No workflow deletion capabilities

- Required vs Optional Fields

Required inputs: - recipient: String (email or phone) - message: String (1-1000 chars)

Optional inputs: - subject: String (for email only) - priority: enum [LOW, NORMAL, HIGH] (default: NORMAL) - schedule_at: DateTime (send later, must be future)


### AI understands:

Option<T>  for optional fields Default values via  unwrap_or Conditional validation (subject only for email)

Constraint: Cannot convert to same format (validation error) Constraint: PDF output only from DOCX, HTML, MARKDOWN

```rust
    return Err(Error::validation("Cannot convert to same format"));
}
```


### if matches!(input.target_format, OutputFormat::Pdf) {

```rust
    if !matches!(input.source_format, SourceFormat::Docx | SourceFormat::Html | SourceFormat::Markdown) {
        return Err(Error::validation("PDF output requires DOCX, HTML, or Markdown input"));
    }
}
```

```text
  - product_id: String
  - quantity: u32 (min 1, max 100)
  - options: optional map of String → String
- shipping_address:
  - street: String
  - city: String
  - state: String (2 letters for US)
  - zip: String
  - country: String (ISO 3166-1 alpha-2)
```


### - payment_method: enum [CREDIT_CARD, PAYPAL, INVOICE]

AI generates proper nested types with validation.

(table/list, cells in reading order)
- Specifying Output Types
- Simple Output
- Return temperature in both Celsius and Fahrenheit
- Structured Output

```text
  - display_name: String
  - avatar_url: Option<String>
  - bio: Option<String>
- settings:
  - theme: enum [LIGHT, DARK, SYSTEM]
  - notifications: boolean
```


### Pagination Output

Output for list_items tool: - items: array of Item objects - pagination:

```text
  - total: u64 (total matching items)
  - page: u32 (current page, 1-indexed)
  - per_page: u32
  - has_next: boolean
  - next_cursor: Option<String>
```

Empty query → Error::validation("Query cannot be empty") 2.

(table/list, cells in reading order)
- Bad: .context("error")?  // Too vague Bad: ? alone  // No context
- Iterating on Generated Code
- Refinement Prompts
- After initial generation:

Add cursor-based pagination instead of offset 2.

(table/list, cells in reading order)
- Anti-Patterns in Prompting
- Too Vague
- Bad:
- Make a tool that does stuff with data
- Good:
- Create a tool that parses CSV files and returns rows as JSON
- Too Prescriptive
- Bad:

The function should first check if data.len() > 0...


### Good:

Create a data processing tool that accepts binary data, validates it's not empty, and returns the parsed result.

(table/list, cells in reading order)
- Missing Error Cases
- Bad:
- Create a tool that divides two numbers
- Good:

Create a division tool: - Input: numerator and denominator (both f64) - Output: result - Error: Division by zero should return validation error - Edge cases: Handle infinity and NaN appropriately

(table/list, cells in reading order)
- Ambiguous Requirements
- Bad:
- Create a search tool with good performance
- Good:

Input: - [field]: [type] ([required/optional]) - [description] - ...

Current behavior: [what it does now] Desired behavior: [what it should do]

(table/list, cells in reading order)
- Changes needed: 1. [Specific change] 2. [Specific change]
- Preserve: [what should stay the same]
- Bug Fix Template
- Fix issue in `[tool-name]`:
- Steps to reproduce: 1. [Action] 2. [Action]
- Expected: [result] Actual: [result]
- Additional context: [relevant details]
- Server Design Template


### Purpose: [what problem it solves]

Tools (list with brief descriptions): 1.

(table/list, cells in reading order)
- Quality requirements: - [Coverage, testing, etc.]
- Summary
- Effective prompting for MCP tools:
- Aspect
- Approach
- Context
- Set domain and technology
- Capability
- Describe what, not how
- Constraints
- Set clear boundaries
- Error cases
- Enumerate explicitly
- Output
- Specify structure clearly
- Iteration
- Refine with focused requests

What the tool should accomplish What inputs it needs What outputs it produces What errors it handles What constraints apply


### Quality Assurance with AI

AI assistants generate code quickly, but speed without quality creates technical debt.


### The Quality Assurance Stack

```text
                Quality Assurance Layers
Layer 1: Compile-Time Safety
 cargo build   →  Type errors, borrow issues, missing imports
                  AI iterates until compilation succeeds
Layer 2: Static Analysis
 cargo clippy  →  Code smells, inefficiencies, patterns
                  AI fixes warnings to meet zero-warning goal
Layer 3: Unit Testing
 cargo test    →  Handler logic, edge cases, error paths
                  AI writes tests covering success and failure
Layer 4: Integration Testing
 cargo pmcp test →  Full server behavior, MCP protocol
                     AI generates and runs scenarios
Layer 5: Code Review
 Human review  →  Logic correctness, security, maintainability
```

```text
You verify AI's work before deployment
```

(table/list, cells in reading order)
- Layer 1: Compile-Time Safety
- The Compilation Loop
- AI generates code → Compiler checks → Errors found → AI fixes → Repeat
- User: Create a weather tool
- AI: [Generates code]

```bash
$ cargo build
```


### error[E0308]: mismatched types

```rust
  --> src/tools/weather.rs:25:12
   |
25 |     return temperature;
   |            ^^^^^^^^^^^ expected `WeatherOutput`, found `f64`
```


### AI: I see the issue - I need to wrap the result. [Fixes code]

```bash
$ cargo build
   Compiling mcp-weather-core v1.0.0
    Finished dev [unoptimized + debuginfo]
```

(table/list, cells in reading order)
- Requesting Compilation Checks
- After any code change:
- Run cargo build and fix any errors
- AI will:

1.

(table/list, cells in reading order)
- Common Compilation Errors AI Handles
- Error Type
- AI Fix
- Type mismatch
- Wrap in correct type, add conversion
- Missing import
- Add  use  statement
- Borrow issue
- Clone, use reference, restructure
- Lifetime error
- Add annotation, restructure ownership
- Missing trait
- Add derive macro, implement trait
- Layer 2: Static Analysis (Clippy)
- Running Clippy
- Run cargo clippy and fix all warnings
- AI executes:

```rust
// Before: warning: redundant clone
let city = input.city.clone();
process(city);

// After: AI fix
let city = input.city;
process(city);
```


### Unnecessary Collect:

```rust
// Before: warning: avoid using `collect()` followed by `into_iter()`
let items: Vec<_> = data.iter().collect();
```


### for item in items.into_iter() { ... }

```text
// After: AI fix
```

```rust
// Before: warning: this match could be replaced
match result {
    Some(x) => x,
    None => return Err(Error::validation("not found")),
}

// After: AI fix
```

result.ok_or_else(|| Error::validation("not found"))?

```toml
# Cargo.toml
[lints.clippy]
all = "warn"
pedantic = "warn"
```


### Generated Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input(city: &str) -> WeatherInput {
        WeatherInput {
            city: city.to_string(),
            days: None,
        }
    }

    #[tokio::test]
    async fn test_valid_city() {
        let input = create_test_input("London");
        let result = handler(input, RequestHandlerExtra::default()).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.city, "London");
        assert!(output.temperature_celsius > -100.0);
        assert!(output.temperature_celsius < 100.0);
    }

    #[tokio::test]
    async fn test_empty_city() {
        let input = create_test_input("");
        let result = handler(input, RequestHandlerExtra::default()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_whitespace_city() {
        let input = create_test_input("   ");

        let result = handler(input, RequestHandlerExtra::default()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_very_long_city() {
        let long_city = "a".repeat(1000);
        let input = create_test_input(&long_city);
        let result = handler(input, RequestHandlerExtra::default()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("too long"));
    }
}
```

Ensure the weather tool has: - At least one test per error case - Tests for boundary conditions (0, max, edge values) - Tests with Unicode input - Tests for optional parameter handling


### Running Tests


### With output:

running 4 tests test tools::weather::tests::test_valid_city ... ok test tools::weather::tests::test_empty_city ... ok test tools::weather::tests::test_whitespace_city ... ok test tools::weather::tests::test_very_long_city ... ok

(table/list, cells in reading order)
- test result: ok. 4 passed; 0 failed; 0 ignored
- Layer 4: Integration Testing
- Generating Scenarios
- Generate integration test scenarios for the weather server
- AI executes:

```yaml
steps:
  - name: "List available tools"
    operation:
      type: list_tools
    assertions:
      - type: success
      - type: contains
        path: "tools"
        value: "get-current-weather"

  - name: "Get weather for valid city"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "London"
    assertions:
      - type: success
      - type: field_exists
        path: "content.0.text"

  - name: "Get weather for invalid city"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: ""
    assertions:
      - type: error
      - type: contains
        path: "error.message"
        value: "cannot be empty"
```


### Running Integration Tests

```bash
# Terminal 1: Start dev server
cargo pmcp dev --server weather

# Terminal 2: Run tests
cargo pmcp test --server weather
```


### Output:

Running scenarios for weather server...


### Scenario: Weather Server Integration Tests

```json
  [ok] List available tools (12ms)
  [ok] Get weather for valid city (45ms)
  [ok] Get weather for invalid city (8ms)
```

```yaml
  - name: "Test with Unicode city name"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "東京"
    assertions:
      - type: success

  - name: "Test with special characters"
    operation:
      type: tool_call
      tool: "get-current-weather"
      arguments:
        city: "São Paulo"
    assertions:
      - type: success

  - name: "Test concurrent requests"
    parallel:
      - operation:
          type: tool_call
          tool: "get-current-weather"
          arguments:
            city: "London"
      - operation:
          type: tool_call
          tool: "get-current-weather"
          arguments:
            city: "Paris"
    assertions:
      - type: all_success
```

(table/list, cells in reading order)
- Layer 5: Code Review
- Review Checklist
- When reviewing AI-generated code:
- 1. Logic Correctness

Does the code do what was requested?


### 2. Security

Input validation present?


### 3. Performance

No unnecessary allocations?


### 4. Maintainability

Clear variable names?

```text
   - Empty city name (user should provide a valid city)
   - City name too long (user should shorten input)
```


### 2. Error::internal() for system issues:

```text
   - API connection failure (not user's fault)
   - Parse failure (indicates API change)
```

This distinction helps AI clients understand whether to retry with different input or report a server problem.


### Security Review Prompts

Review the database query tool for SQL injection vulnerabilities.


### AI responds:

I've reviewed the query handling.

```rust
let results = sqlx::query_as!(
    Record,
    "SELECT * FROM users WHERE name = $1",
    input.name  // Parameterized, not interpolated
)
```

(table/list, cells in reading order)
- Quality Gate Automation
- Pre-Commit Hook
- Create  .git/hooks/pre-commit :
- #!/bin/bash set -e
- echo "Running quality gates..."

```bash
# Format check
cargo fmt --check || {
    echo "[BAD] Run 'cargo fmt' to fix formatting"
    exit 1
}

# Clippy
cargo clippy -- -D warnings || {
    echo "[BAD] Fix clippy warnings"
    exit 1
}

# Unit tests
cargo test || {
    echo "[BAD] Fix failing tests"
    exit 1
}
```

(table/list, cells in reading order)
- echo "[GOOD] All quality gates passed"
- Makefile Integration
- .PHONY: quality-gate test lint fmt
- quality-gate: fmt lint test

```bash
    @echo "[GOOD] All quality gates passed"

fmt:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

test:
    cargo test

integration-test:
    cargo pmcp test --server $(SERVER)
```

```text
# .github/workflows/quality.yml
```

```yaml
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Format Check
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Unit Tests
        run: cargo test

      - name: Build Release
        run: cargo build --release
```

(table/list, cells in reading order)
- Common Quality Issues and Fixes
- Issue: Unwrap in Production Code
- Detection: Clippy warning or code review
- Request:


### Before:

```rust
let value = map.get("key").unwrap();
```


### After:

```rust
let value = map.get("key")
    .ok_or_else(|| Error::internal("Missing required key"))?;
```

Add input validation for the create_user tool: - username: 3-50 chars, alphanumeric only - email: valid email format - age: 13-120 (if provided)

Each error should explain: 1.

Deprecated patterns 2.


### Dependency Updates

Check for outdated dependencies and suggest updates.


### Documentation Verification

Verify all public APIs have: 1.

(table/list, cells in reading order)
- Summary
- Quality assurance with AI follows a layered approach:
- Layer
- Tool
- What It Catches
- AI Role
- 1
- 2
- 3
- 4
- 5

```bash
cargo build
```

```bash
cargo clippy
```

```bash
cargo pmcp test
```

(table/list, cells in reading order)
- Integration issues
- Generates scenarios
- Human review
- Design flaws
- Explains, justifies
- Key practices:

Run all gates after every change - Don't accumulate issues 2.
