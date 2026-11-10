# Constitution

Version: 1.0.0
Changelog: 1.0.0, initial for handsfree. Rules drawn from the repo's existing quality gate and ADRs.

## Code
- Rust. `make quality-gate` must pass: `cargo fmt --check`, `clippy -D warnings`, tests.
- No `unwrap` in prod paths. Errors: `thiserror` (lib), `anyhow` (bin).
- Guard clauses over nested ifs. One responsibility per module.
- Spec first: spec, types, tests, then impl.

## Desktop automation safety
- Default deny for input and screen reads. Gated actions need user approval.
- Kill switch must stop all actions.
- Log every action. Never log secrets.

## Protocol
- MCP server built on PMCP. See [ADR-0001](../../docs/adr/0001-pmcp-server.md).
- Expressions use CEL. See [ADR-0002](../../docs/adr/0002-cel.md).
- Driver RPC is JSON-RPC 2.0. See [ADR-0003](../../docs/adr/0003-driver-rpc.md).

## Data
- Minimise captured data. No secrets in code or config.

## Process
- Do not add a crate (dependency) without telling the owner first. Present the top 2-4 options in a short comparison (features, maintenance, license, size, fit) and wait for the owner's choice.
- Architecture changes need an ADR in `docs/adr/`.
- Docs index: [docs/README.md](../../docs/README.md).
