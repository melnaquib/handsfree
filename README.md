# handsfree

Eyes and hands for any model. Cross-platform, multi-vendor UI introspection and control.
Desktop (Linux, Windows, macOS), browsers, mobile, embedded, emulators.
Delivered as: MCP server (core), CLI, skill, host plugin wrappers.

Built on the [PMCP SDK](https://github.com/paiml/rust-mcp-sdk).
Status: **spec phase**. Specs live in `kitty-specs/`.

## Goals and non-goals

Goals:
1. One server. Any model. Any vendor. Observe and act on UIs.
2. Same tool surface on every platform. Drivers differ underneath.
3. Safe by default: read-only, allowlisted, audited, human approvals the model cannot self-approve.

Non-goals:
1. Not an agent. No LLM inside. The client model decides.
2. Not a test framework. Reuse Appium/Playwright ideas where good.
3. No cloud dependency. Runs local. Remote only by opt-in.
4. No bypass of OS security (UAC/UIPI, TCC, Wayland portals, FLAG_SECURE). Report limits instead.

## Docs

- Index: [docs/README.md](docs/README.md)
- Tech stack: [docs/techstack.md](docs/techstack.md)
- Decisions: [docs/adr/](docs/adr/)
- Security: [docs/security/](docs/security/)
- MCP tools: [docs/mcp/tools.md](docs/mcp/tools.md)
- Testing: [docs/testing.md](docs/testing.md)
- Runbook: [docs/runbook.md](docs/runbook.md)
- PMCP/Rust workflow: [docs/tech/pmcp-workflow.md](docs/tech/pmcp-workflow.md)
- Constitution: [.kittify/constitution/constitution.md](.kittify/constitution/constitution.md)
- Brand assets: [brand/](brand/)

## Quick Start

### Prerequisites

Install mcp-tester for automated testing:
```bash
cargo install mcp-tester
```

### Development

```bash
# Add your first server
cargo-pmcp add server myserver --template minimal

# Generate and run tests
cargo-pmcp test --server myserver --generate-scenarios

# Start development server
cargo run --bin handeye-server

# Run quality checks
make quality-gate
```

## Project Structure

```
handsfree/
├── crates/
│   ├── server-common/     # Shared HTTP bootstrap (80 LOC)
│   ├── mcp-handeye-core/    # Server business logic (tools, resources)
│   └── handeye-server/      # Server binary (6 LOC)
├── scenarios/             # Test scenarios (YAML)
├── lambda/                # Lambda deployment configs
├── Cargo.toml             # Workspace manifest
└── Makefile               # Build/test/deploy commands
```

## Development Workflow

### 1. Add a new server
```bash
cargo-pmcp add server myserver --template minimal
```

### 2. Generate and run tests
```bash
# Generate test scenarios using mcp-tester (requires: cargo install mcp-tester)
cargo-pmcp test --server myserver --generate-scenarios

# Or just run existing scenarios
cargo-pmcp test --server myserver
```

### 3. Run quality checks
```bash
make quality-gate  # fmt + clippy + tests
```

### 4. Build and run
```bash
# Development
cargo run --bin myserver-server

# Production
cargo build --release --bin myserver-server
```

## Server Pattern

Each server has two crates:

- **mcp-handeye-core** (library): Business logic, tools, resources, workflows
- **handeye-server** (binary): Just 6 lines calling `server_common::run_http()`

This pattern:
- Shares HTTP bootstrap across all servers (DRY)
- Makes binaries trivial (easy to audit)
- Enables unit testing without HTTP complexity
- Scales from 1 to 100 servers

## Configuration

Servers use environment variables:

```bash
RUST_LOG=info              # Logging level
MCP_HTTP_PORT=3000         # Port (or PORT)
MCP_ALLOWED_ORIGINS=*      # CORS origins
```

## Testing

### Automated Testing with mcp-tester

Install mcp-tester:
```bash
cargo install mcp-tester
```

Generate test scenarios automatically:
```bash
cargo-pmcp test --server handeye --generate-scenarios
```

This will:
1. Build your server
2. Start it temporarily
3. Use mcp-tester to discover all tools and capabilities
4. Generate comprehensive test scenarios in `scenarios/handeye/generated.yaml`
5. Run the scenarios and show results

### Manual Testing

Run unit tests:
```bash
cargo test -p mcp-handeye-core
```

Start server and test with curl:
```bash
cargo run --bin handeye-server &
curl -X POST http://0.0.0.0:3000 \\
  -H "Content-Type: application/json" \\
  -H "Accept: application/json" \\
  -d '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}'
```

## Quality Standards

- **Zero tolerance for defects**: All commits pass quality gates
- **80%+ test coverage**: Property tests + unit tests + integration tests
- **Type safety**: schemars JsonSchema with validation
- **Production middleware**: Client tracking, redaction, request IDs

## Resources

- [PMCP SDK Documentation](https://github.com/paiml/rust-mcp-sdk)
- [MCP Specification](https://spec.modelcontextprotocol.io)
- [Example Servers](https://github.com/paiml/rust-mcp-sdk/tree/main/examples)

## License

Apache-2.0 (see `LICENSE`)
