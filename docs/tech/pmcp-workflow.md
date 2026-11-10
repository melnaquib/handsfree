# PMCP workflow (Rust)

Status: active. Follows ADR [0001](../adr/0001-pmcp-server.md) (PMCP server, accepted). This file keeps the PMCP workflow from the template README so it is not lost.

Source: template README, moved from the repo root. Commands and crate names are from the template and are unverified for handfree.

## Prerequisites

Install mcp-tester for automated testing:

```bash
cargo install mcp-tester
```

## Development

```bash
# Add your first server
cargo-pmcp add server calculator --template minimal

# Generate and run tests
cargo-pmcp test --server calculator --generate-scenarios

# Start development server
cargo run --bin calculator-server

# Run quality checks
make quality-gate
```

## Project structure (template)

```
handsfree/
├── crates/
│   ├── server-common/        # Shared HTTP bootstrap (80 LOC)
│   ├── mcp-calculator-core/  # Calculator business logic
│   └── calculator-server/    # Calculator binary (6 LOC)
├── scenarios/                # Test scenarios (YAML)
├── lambda/                   # Lambda deployment configs
├── Cargo.toml                # Workspace manifest
└── Makefile                  # Build/test/deploy commands
```

`calculator-*` names are template placeholders. Rename to handfree crates when created.

## Adding a server

```bash
cargo-pmcp add server myserver --template minimal
```

## Generate and run tests

```bash
# Generate test scenarios using mcp-tester
cargo-pmcp test --server myserver --generate-scenarios

# Or just run existing scenarios
cargo-pmcp test --server myserver
```

## Build and run

```bash
# Development
cargo run --bin myserver-server

# Production
cargo build --release --bin myserver-server
```

## Server pattern

Each server has two crates:

- **core** (library): business logic, tools, resources, workflows
- **server binary**: about 6 lines calling `server_common::run_http()`

Why:
- Shared HTTP bootstrap across servers
- Trivial binaries, easy to audit
- Unit tests without HTTP
- Scales from 1 to 100 servers

## Manual testing with curl

Unit tests:

```bash
cargo test -p mcp-calculator-core
```

Start the server, then list tools:

```bash
cargo run --bin calculator-server &
curl -X POST http://0.0.0.0:3000 \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

## mcp-tester generated scenarios

`cargo-pmcp test --server calculator --generate-scenarios` is expected to:
1. Build the server
2. Start it temporarily
3. Use mcp-tester to discover tools and capabilities
4. Write scenarios to `scenarios/calculator/generated.yaml`
5. Run them and show results
