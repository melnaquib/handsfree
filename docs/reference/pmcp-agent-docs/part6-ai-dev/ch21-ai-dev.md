---
chapter: 21
title: "AI-Assisted MCP Development"
part: "Part VI: AI-Assisted Development"
use_when: "Setting up an AI coding assistant for MCP work; compiler feedback loop; clippy gates."
suggested_prerequisites: [ch02]
source_kb: 41
extracted_kb: 26
code_blocks: 84
quality: auto-extracted
---

# Ch 21: AI-Assisted MCP Development

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### AI-Assisted MCP Development

Building MCP servers with AI assistance transforms the development experience.


### The Perfect Storm for AI Development

```text
         AI-Assisted MCP Development Stack
                  AI Coding Assistant
(Claude Code, Kiro, Cursor, Copilot)
• Understands requirements
• Generates type-safe code
• Interprets compiler feedback
• Iterates until quality gates pass
                  cargo-pmcp Toolkit
• Scaffolds complete server structure
• Enforces proven patterns
• Hot-reload development server
• Automated test generation
                  Rust Compiler
• Catches errors at compile time
• Provides actionable error messages
• Enforces memory safety
• Type system prevents runtime bugs
```

```text
              Production MCP Server
• Type-safe tools with JSON Schema
• Comprehensive error handling
• 80%+ test coverage
• Zero clippy warnings
```

Unlike dynamically-typed languages where bugs appear at runtime, Rust's compiler provides immediate, detailed feedback:


### error[E0308]: mismatched types

```rust
  --> src/tools/weather.rs:45:12
   |
45 |     return temperature;
   |            ^^^^^^^^^^^ expected `WeatherOutput`, found `f64`
   |
help: try wrapping the expression in `WeatherOutput`
   |
45 |     return WeatherOutput { temperature, conditions: todo!() };
   |            ++++++++++++++++++++++++++++++++++++++++++++++++++
```

AI assistants can read these errors and fix them automatically.


### 2. Type Safety Prevents Entire Classes of Bugs

```rust
// The type system catches errors before runtime
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeatherInput {
    pub city: String,           // Must be provided
    pub days: Option<u8>,       // Optional with type constraints
}

// AI generates this - compiler ensures correctness
async fn handler(input: WeatherInput, _: RequestHandlerExtra) -> Result<WeatherOutput> {
    let days = input.days.unwrap_or(1);  // Safe default
    // ...
}
```


### 3. cargo-pmcp Eliminates Boilerplate Decisions

AI assistants don't need to decide project structure - cargo-pmcp enforces it:

```bash
# AI runs this - complete structure in seconds
cargo pmcp new weather-workspace
cargo pmcp add server weather --template minimal

# AI only writes business logic in:
# crates/mcp-weather-core/src/tools/*.rs
```


### 4. Quality Gates Catch What AI Misses

```bash
# Every change validated automatically
cargo fmt --check    # Consistent formatting
cargo clippy         # Lint checks
cargo test           # Unit tests pass
cargo pmcp test      # Integration tests pass
```

1.

```bash
$ cargo pmcp new github-mcp-workspace
$ cd github-mcp-workspace
$ cargo pmcp add server github --template minimal
```


### Now implementing the tools:

1. get_repo - Fetch repository details 2. list_repos - List user's repositories 3. get_readme - Fetch repository README


### [Implements tools with proper types, validation, error handling]

```bash
$ cargo fmt && cargo clippy && cargo test
```

All quality gates passing!


### To configure in Claude Code:

```json
{
  "mcpServers": {
    "github": {
      "command": "./target/release/github-server",
      "env": { "GITHUB_TOKEN": "${env:GITHUB_TOKEN}" }
    }
  }
}
```

Lower barrier to entry: Describe what you want, AI handles Rust complexity Learn by example: See idiomatic Rust code generated Immediate feedback: Compiler and AI explain errors Best practices built-in: cargo-pmcp enforces patterns


### For Experienced Developers

Faster iteration: Focus on business logic, not boilerplate Consistent quality: Same patterns across all servers Reduced cognitive load: AI handles routine code More ambitious projects: Build more in less time


### For Teams

Onboarding: New developers productive immediately Standardization: All servers follow same structure Code review: AI-generated code follows conventions Documentation: AI generates docs from types


### Then effective collaboration:

4.

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# 2. Install cargo-pmcp
cargo install cargo-pmcp

# 3. Verify
cargo pmcp --version
```


### AI assistants armed with MCP knowledge can:

Scaffold complete server structures Implement type-safe tools Handle error cases properly Generate comprehensive tests Pass all quality gates


### The AI-Compiler Feedback Loop

The combination of Rust's compiler and AI coding assistants creates a powerful feedback loop that accelerates development while maintaining quality.


### The Feedback Loop

```text
         AI-Compiler Feedback Loop
Developer
Request      "Create a weather tool"
   AI        Generates initial code
Assistant
 cargo          ▶    Errors?        ▶    AI reads
 build                           Yes     errors
                          No
                     clippy
                     check
                    Warnings?
                                 Yes
                          No
                     Tests
                     pass?
```

```text
                             No
                      Yes
                Complete!
◀
```

error[E0599]: no method named `fetch` found for struct `Client` in the current scope

```rust
  --> src/tools/api.rs:23:10
   |
23 |     client.fetch(&url).await?;
   |            ^^^^^ method not found in `Client`
   |
help: there is a method with a similar name
   |
23 |     client.get(&url).await?;
   |            ~~~
```


### Key elements:

Error code:  E0599  (searchable, documented) Location: File, line, column

```rust
// AI writes this
let result = fetch_weather(city);

// Compiler responds
error[E0308]: mismatched types
  --> src/tools/weather.rs:15:16
   |
15 |     let result = fetch_weather(city);
   |                  ^^^^^^^^^^^^^^^^^^^ expected `&str`, found `String`
   |
help: consider borrowing here
   |
15 |     let result = fetch_weather(&city);
   |                                +

// AI fixes automatically
let result = fetch_weather(&city);
```

- warning: this `if let` can be collapsed into the outer `if let`

```rust
  --> src/tools/parser.rs:42:9
   |
```

```yaml
   | |_________^
   |
help: try
   |
```

```rust
// AI generates initial attempt
async fn get_weather(input: WeatherInput) -> WeatherOutput {
    let response = reqwest::get(&format!("https://api.weather.com/{}", input.city))
        .await
        .json()
        .await;

    WeatherOutput {
        temperature: response.temp,
        conditions: response.weather,
    }
}
```

```rust
  --> src/tools/weather.rs:8:14
   |
8  |         .await?;
   |              ^ the trait `From<reqwest::Error>` is not implemented
```


### AI fixes:

```rust
async fn get_weather(
    input: WeatherInput,
    _: RequestHandlerExtra
) -> Result<WeatherOutput> {
    let response = reqwest::get(&format!("https://api.weather.com/{}", input.city))
        .await
        .context("Failed to fetch weather")?  // Added context
        .json::<ApiResponse>()
        .await
        .context("Failed to parse response")?;  // Added context

    Ok(WeatherOutput {  // Wrapped in Ok
        temperature: response.temp,
        conditions: response.weather,
    })
}
```

- warning: this function has too many lines (52/50)

```rust
  --> src/tools/weather.rs:10:1
   |
   = help: for further information visit rust-lang.github.io/rust-clippy
```


### AI refactors:

```rust
// Extract helper function
async fn fetch_weather_data(city: &str) -> Result<ApiResponse> {
    reqwest::get(&format!("https://api.weather.com/{}", city))
        .await
        .context("Failed to fetch weather")?
        .json()
        .await
        .context("Failed to parse response")
}

// Main handler now smaller
async fn get_weather(input: WeatherInput, _: RequestHandlerExtra) -> Result<WeatherOutput> {
    let response = fetch_weather_data(&input.city).await?;
    Ok(WeatherOutput::from(response))
}
```

- warning: function `get_weather` is never used

```rust
  --> src/tools/weather.rs:10:8
```


### AI adds tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weather_valid_city() {
        let input = WeatherInput { city: "London".to_string(), days: None };
        let result = get_weather(input, RequestHandlerExtra::default()).await;
        // ... assertions
    }
}
```

```text
def get_weather(city):
    response = requests.get(f"https://api.weather.com/{city}")
    return {
        "temperature": response["temp"],      # KeyError at runtime!
        "conditions": response["wheather"],   # Typo not caught!
    }
```

```yaml
    const response = await fetch(`https://api.weather.com/${city}`);
    const data = await response.json();
    return {
        temperature: data.temp,
        conditions: data.wheather,  // Typo not caught!
    };
}
```

```rust
async fn get_weather(input: WeatherInput) -> Result<WeatherOutput> {
    let response: ApiResponse = reqwest::get(/*...*/)
        .await?  // Must handle error
        .json()
        .await?;  // Must handle error

    Ok(WeatherOutput {
        temperature: response.temp,    // Verified at compile time
        conditions: response.weather,  // Typo would be caught
    })
}
```

(table/list, cells in reading order)
- Problems discovered: Before code even runs
- Error Categories AI Handles
- 1. Type Errors (Most Common)
- error[E0308]: mismatched types

AI understands: needs type conversion, wrapping, or different return type.

```toml
# Cargo.toml
[lints.rust]
warnings = "deny"

[lints.clippy]
all = "warn"
pedantic = "warn"
```


### 2. Use Strict Clippy

```bash
cargo clippy -- -D warnings
```


### 3. Run Tests Early

```bash
# After each significant change
cargo test --lib
```


### 4. Continuous Feedback with cargo-watch

```bash
cargo watch -x check -x clippy -x "test --lib"
```

1. Each fix is progress: Errors don't multiply 3.


### Typical convergence:

Simple tools: 1-2 iterations Complex tools: 3-5 iterations Integration issues: 5-10 iterations

(table/list, cells in reading order)
- Summary
- Rust Provides
- AI Provides
- Detailed error messages
- Pattern recognition
- Type-driven suggestions
- Code generation
- Compile-time safety
- Rapid iteration
- Actionable warnings
- Intent understanding
- Together, they create a development experience where:


### Setting Up Claude Code

Claude Code is a command-line AI assistant that integrates deeply with your development workflow.

```text
# macOS
```


### brew install claude-code

```bash
# Or via npm
npm install -g @anthropic-ai/claude-code

# Verify installation
claude --version
```


### Install Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# cargo-pmcp toolkit
cargo install cargo-pmcp

# Verify
cargo pmcp --version
```

The mcp-developer agent teaches Claude Code how to build MCP servers using cargo-pmcp and pmcp SDK best practices.

- User-Level Installation (Recommended)

```text
# Create agents directory
```


### mkdir -p ~/.claude/agents

```bash
# Download the agent
curl -fsSL https://raw.githubusercontent.com/paiml/rust-mcp-sdk/main/ai-agents/claude-code/mcp-developer.md \
  -o ~/.claude/agents/mcp-developer.md
```

```text
# Create project agents directory
```


### mkdir -p .claude/agents

```bash
# Download the agent
curl -fsSL https://raw.githubusercontent.com/paiml/rust-mcp-sdk/main/ai-agents/claude-code/mcp-developer.md \
  -o .claude/agents/mcp-developer.md
```


### Verify Installation

```text
# Start Claude Code
```


### claude

```text
# List agents
```


### /agents

```text
# Should show:
# - mcp-developer: Expert MCP server developer using pmcp Rust SDK
```

(table/list, cells in reading order)
- What the Agent Provides
- Knowledge Encoded
- The mcp-developer agent knows:
- 1. cargo-pmcp workflow - Never create files manually

2. pmcp SDK patterns - TypedTool, resources, workflows 3.

```text
# In a new directory
```


### First, let me scaffold the workspace:

```bash
$ cargo pmcp new github-mcp-workspace
$ cd github-mcp-workspace
```


### Now adding the server:

```bash
$ cargo pmcp add server github --template minimal
```

This creates: - crates/mcp-github-core/ - Library with tools - crates/github-server/ - Binary executable - scenarios/github/ - Test directory

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRepoInput {
    #[schemars(description = "Repository owner (user or org)")]
    pub owner: String,

    #[schemars(description = "Repository name")]
    pub repo: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetRepoOutput {
    pub full_name: String,
    pub description: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub language: Option<String>,
}
```


### 2. Implement handlers with proper error handling

```rust
async fn get_repo_handler(
    input: GetRepoInput,
    extra: RequestHandlerExtra,
) -> Result<GetRepoOutput> {
    // Validation
    if input.owner.is_empty() || input.repo.is_empty() {
        return Err(Error::validation("Owner and repo required"));
    }

    // Get token from metadata
    let token = extra.metadata
        .get("github_token")
        .ok_or_else(|| Error::internal("GitHub token not configured"))?;

    // API call with error handling
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("https://api.github.com/repos/{}/{}", input.owner, input.repo))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "mcp-github-server")
        .send()
        .await
        .context("Failed to connect to GitHub")?;

    // Handle HTTP errors
    if response.status() == 404 {
        return Err(Error::validation(format!(
            "Repository {}/{} not found", input.owner, input.repo
        )));
    }

    if !response.status().is_success() {
        return Err(Error::internal(format!(
            "GitHub API error: {}", response.status()
        )));
    }

    // Parse response
    let repo: GitHubRepo = response

        .json()
        .await
        .context("Failed to parse GitHub response")?;

    Ok(GetRepoOutput {
        full_name: repo.full_name,
        description: repo.description,
        stars: repo.stargazers_count,
        forks: repo.forks_count,
        language: repo.language,
    })
}
```


### 3. Register tools in lib.rs

```rust
pub fn build_github_server() -> Result<McpServer> {
    McpServer::builder("github-server", "1.0.0")
        .tool("get-repo", tools::get_repo::build_tool())
        .tool("list-repos", tools::list_repos::build_tool())
        .build()
}
```


### Step 4: Test and Validate

```bash
# Claude Code runs automatically:
$ cargo fmt --check
$ cargo clippy -- -D warnings
$ cargo test

# Start dev server
$ cargo pmcp dev --server github

# Generate test scenarios
$ cargo pmcp test --server github --generate-scenarios
```

```json
{
  "agents": {
    "mcp-developer": {
      "autoInvoke": true,
      "keywords": ["mcp", "server", "tool", "pmcp"]
    }
  },
  "rust": {
    "formatOnSave": true,
    "clippyOnSave": true
  }
}
```


### Environment Variables

```text
# .env file for MCP development
```


### RUST_LOG=debug GITHUB_TOKEN=ghp_your_token WEATHER_API_KEY=your_key

```text
# cargo-pmcp settings
```

(table/list, cells in reading order)
- CARGO_PMCP_TEMPLATE_DIR=~/.cargo-pmcp/templates
- Working with the Agent
- Effective Requests
- Good:

Create an MCP server that queries a PostgreSQL database with list_tables and execute_query tools.


### Better:

Create a PostgreSQL MCP server with: 1. list_tables - returns table names and row counts 2. describe_table - returns column info for a table 3. execute_query - runs SELECT queries with 100 row limit

```bash
$ cargo test
```


### error[E0597]: `response` does not live long enough

I see the issue.


### [Applies fix]

```bash
$ cargo build
```

Build successful.


### Connecting Your MCP Server to Claude Code

After building your server, connect it as an MCP server for Claude Code itself:


### Configure in Claude Code

```text
// ~/.claude/mcp_servers.json
{
  "github": {
    "command": "/path/to/github-mcp-workspace/target/release/github-server",
    "args": [],
    "env": {
      "GITHUB_TOKEN": "${env:GITHUB_TOKEN}"
    }
  }
}
```

```text
# Claude Code now has access to your GitHub tools
```


### You: Use the GitHub MCP server to get info about rust-lang/rust

Claude Code: I'll use the get-repo tool from the GitHub server.


### [Calls get-repo with owner="rust-lang", repo="rust"]

The rust-lang/rust repository has: - 95,000+ stars - Language: Rust - Description: Empowering everyone to build reliable software

```text
# Check current version
```


### head -5 ~/.claude/agents/mcp-developer.md

```bash
# Update to latest
curl -fsSL https://raw.githubusercontent.com/paiml/rust-mcp-sdk/main/ai-agents/claude-code/mcp-developer.md \
  -o ~/.claude/agents/mcp-developer.md
```

```text
# Verify file exists
```


### ls -la ~/.claude/agents/mcp-developer.md

```text
# Check file has correct frontmatter
```


### head -10 ~/.claude/agents/mcp-developer.md

```text
# Should show:
# ---
# name: mcp-developer
# description: Expert MCP server developer...
# ---
```

Or mention keywords: "MCP server", "build", "create", "scaffold", "pmcp"


### cargo-pmcp Not Found

```bash
# Reinstall
cargo install cargo-pmcp --force

# Verify
```


### which cargo-pmcp

```bash
cargo pmcp --version
```

1.

The agent handles the cargo-pmcp workflow, letting you focus on what you want to build rather than how to build it.


### Alternative AI Assistants

While Claude Code is our primary recommendation, the MCP developer knowledge can be adapted to other AI coding assistants.


### Kiro

Kiro uses "steering files" - always-active knowledge that persists across conversations.


### Installation

```text
# Create powers directory
```


### mkdir -p ~/.kiro/powers

```bash
# Clone the MCP developer power
cd ~/.kiro/powers
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/paiml/rust-mcp-sdk.git temp
```


### cd temp

```bash
git sparse-checkout set ai-agents/kiro/mcp-developer-power
```

mv ai-agents/kiro/mcp-developer-power ../mcp-developer cd .. && rm -rf temp

```text
# Restart Kiro
```


### Verify Installation

```text
# Check files
```


### ls ~/.kiro/powers/mcp-developer/steering/

```text
# Should show:
# mcp-product.md
# mcp-tech.md
# mcp-structure.md
# mcp-workflow.md
# mcp-tool-patterns.md
```


### How It Works

Kiro's steering files are always active - Kiro reads them for every conversation:


### steering/

```text
mcp-product.md      # MCP concepts, use cases
mcp-tech.md         # Technology stack, patterns
mcp-structure.md    # Project organization
mcp-workflow.md     # CRITICAL: cargo-pmcp workflow
mcp-tool-patterns.md # Tool implementation patterns
```


### Usage

Simply ask Kiro to build an MCP server - it automatically knows the workflow:


### You: Create a weather MCP server

Kiro: I'll create a weather server using cargo-pmcp.

```bash
$ cargo pmcp new weather-workspace
$ cd weather-workspace
$ cargo pmcp add server weather --template minimal
```

(table/list, cells in reading order)
- [Implements tools following patterns from steering files]
- Kiro vs Claude Code
- Aspect
- Kiro
- Claude Code
- Knowledge type
- Always-active steering
- On-demand agent
- Context size
- 10,000+ lines persistent
- ~600 lines per invocation
- Best for
- Deep learning + building
- Quick scaffolding
- MCP integration
- Native MCP client
- Native MCP client
- Cursor

```text
# MCP Server Development Rules
```


### ## CRITICAL: Always Use cargo-pmcp

NEVER create Cargo.toml, lib.rs, main.rs, or directories manually. ALWAYS use cargo-pmcp commands:


### ```bash

```bash
# Create workspace (one-time)
cargo pmcp new <workspace-name>

# Add server
cargo pmcp add server <name> --template <template>

# Add tool to existing server
cargo pmcp add tool <tool-name> --server <server-name>
```


### Templates

minimal  - Empty structure for custom servers calculator  - Simple arithmetic example sqlite_explorer  - Database browser pattern


### Tool Implementation Pattern

```rust
use pmcp::{Result, TypedTool, RequestHandlerExtra, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct MyInput {
    #[schemars(description = "Parameter description")]
    pub param: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MyOutput {
    pub result: String,
}

async fn handler(input: MyInput, _: RequestHandlerExtra) -> Result<MyOutput> {
    // Validate
    if input.param.is_empty() {
        return Err(Error::validation("Param required"));
    }

    // Process
    Ok(MyOutput { result: input.param })
}

pub fn build_tool() -> TypedTool<MyInput, MyOutput> {
    TypedTool::new("my-tool", |input, extra| {
        Box::pin(handler(input, extra))
    })
    .with_description("Tool description")
}
```


### Quality Standards

Run  cargo fmt --check  before committing Zero clippy warnings:  cargo clippy -- -D warnings Minimum 80% test coverage Never use unwrap() in production code


### ### Usage

With `.cursorrules` in place, Cursor follows these rules automatically when editing Rust MCP code.


### ## GitHub Copilot

Copilot uses `.github/copilot-instructions.md` for repository-level guidance.

```text
# MCP Server Development Instructions
```

- 1. **Scaffolding**: Always use `cargo pmcp` commands

```text
   - `cargo pmcp new` for workspaces
   - `cargo pmcp add server` for new servers
   - Never create files manually
```


### 2. **Tool Pattern**: Use TypedTool with JsonSchema

```text
   - Input types derive: Debug, Deserialize, JsonSchema
   - Output types derive: Debug, Serialize, JsonSchema
   - Handlers return Result<Output>
```


### 3. **Error Handling**: Use pmcp::Error types

```rust
   - Error::validation() for user errors
   - Error::internal() for server errors
   - Always add context with .context()
```


### 4. **Testing**: Use mcp-tester scenarios

```text
   - `cargo pmcp test --generate-scenarios` to generate
   - `cargo pmcp test` to run

   - Minimum 80% coverage
```


### ## Code Style

- Format with `cargo fmt` - Lint with `cargo clippy -- -D warnings` - No unwrap() in production code - Comprehensive error messages


### Aider

Aider uses  .aider.conf.yml  for configuration.

```text
# Aider configuration for MCP development
```

- ## Convention files to always include

```yaml
read:
  - .github/copilot-instructions.md
  - CONVENTIONS.md
```


### ## Auto-commit settings auto-commits: false dirty-commits: false

## Lint command (runs after edits) lint-cmd: cargo fmt --check && cargo clippy -- -D warnings

```text
# MCP Development Conventions
```

## Scaffolding ALWAYS use cargo-pmcp for project structure: - `cargo pmcp new <workspace>` - Create workspace - `cargo pmcp add server <name> --template minimal` - Add server

## Error Handling - Validation errors: `Error::validation("message")` - Internal errors: `Error::internal("message")` - Context: `.context("Failed to...")?` - Never: `unwrap()`, `expect()`, `panic!()`


### Continue.dev

Continue uses  .continuerc.json  for configuration.

```json
{
  "customCommands": [
    {
      "name": "mcp-new",
      "description": "Create new MCP workspace",
      "prompt": "Create a new MCP server workspace using cargo-pmcp. Follow these steps:\n1. cargo pmcp new
```

```rust
    },
    {
      "name": "mcp-tool",
      "description": "Add MCP tool",
      "prompt": "Add a new tool to the MCP server. Use TypedTool with proper JsonSchema types. Include validation and
error handling. Add unit tests."
    }
  ],
  "contextProviders": [
    {
      "name": "pmcp-docs",
      "type": "url",
      "url": "https://docs.rs/pmcp/latest/pmcp/"
    }
  ],
  "systemPrompt": "When working on MCP servers:\n- Always use cargo-pmcp commands for scaffolding\n- Follow TypedTool
```


### Windsurf

Windsurf uses agent configurations similar to Claude Code.

```yaml
triggers:
  - mcp
  - server
  - pmcp
  - tool
```


### ---

```text
# MCP Development Agent
```

You are an expert MCP server developer using the pmcp Rust SDK.

- 1. **ALWAYS** use cargo-pmcp for scaffolding:

```text
   - `cargo pmcp new <workspace>` for new projects
   - `cargo pmcp add server <name> --template minimal` for servers
   - NEVER create Cargo.toml or directory structure manually
```


### 2. **Tool Pattern**:

```rust
   - Input types: `#[derive(Debug, Deserialize, JsonSchema)]`
   - Output types: `#[derive(Debug, Serialize, JsonSchema)]`
   - Handlers: `async fn(Input, RequestHandlerExtra) -> Result<Output>`
```


### 3. **Quality Gates**:

```text
   - `cargo fmt --check` - formatting
   - `cargo clippy -- -D warnings` - linting
   - `cargo test` - tests pass
   - 80%+ test coverage
```


### 4. **Error Handling**:

```rust
   - Never use unwrap() or expect()
   - Use Error::validation() for user errors
   - Use Error::internal() for server errors
   - Add .context() to error paths
```

(table/list, cells in reading order)
- Creating Custom Configurations
- Core Knowledge to Include
- Any AI assistant configuration should include:
- 1. Workflow (most critical):

Use cargo-pmcp commands Never create files manually Follow scaffold → implement → test → validate flow


### 2. Type Patterns:

JsonSchema derives for auto-schema generation Proper input/output type definitions TypedTool builder pattern


### 3. Error Handling:

Error types and when to use each Context addition with anyhow No unwrap/panic rules


### 4. Quality Standards:

Format, lint, test commands Coverage requirements Toyota Way principles


### Template

```text
# MCP Developer Configuration for [AI Tool]
```

## Workflow - Scaffold: `cargo pmcp new` and `cargo pmcp add server` - Implement: Edit `crates/mcp-*-core/src/tools/*.rs` - Test: `cargo pmcp test --generate-scenarios && cargo pmcp test` - Validate: `cargo fmt && cargo clippy && cargo test`

(table/list, cells in reading order)
- ## Tool Pattern [Include TypedTool example]
- ## Error Handling [Include Error types and .context() usage]
- ## Quality Gates [Include specific commands and thresholds]
- Summary
- AI Assistant
- Configuration
- Location
- Claude Code
- Agent markdown
- ~/.claude/agents/
- Kiro
- Cursor
- Copilot
- Aider
- Continue
- Windsurf
- Steering files
- ~/.kiro/powers/
- Rules file
- Instructions
- YAML config
- JSON config
- .cursorrules
- .github/copilot-instructions.md
- .aider.conf.yml
- .continuerc.json
- Agent markdown
- .windsurf/agents/
- All configurations encode the same core knowledge:
