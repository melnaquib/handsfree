# 0001. Use pmcp (Rust) for the MCP server

* Status: accepted (spike still to confirm the checklist below)
* Date: 2026-11-04
* Deciders: Mustafa

## Context

handsfree needs an MCP server in front of drivers. In v1 the server and the driver are one process. The design moved to a Rust server using the [pmcp](https://crates.io/crates/pmcp) crate.

Unverified (check in spike): crate version, license, maturity, MCP revision 2026-07-28 support, MRTR (`input_required`) and Tasks, tool annotations, toolset filtering, stdio + Streamable HTTP.

## Options considered

1. pmcp (Rust): one language with the Rust drivers. Maturity unverified.
2. Hand-written JSON-RPC server: more work, no SDK risk.

## Decision

Use pmcp for the MCP server. Keep it a thin adapter: handlers parse args, call plain library crates, map errors. Pin exact versions.

Coding conventions: [pmcp agent docs](../reference/pmcp-agent-docs/README.md).

## Consequences

* Good: single toolchain, typed tool schemas via `schemars`.
* Bad: SDK maturity risk.
* Follow-ups: spike with 3 tools, stdio, mock driver over JSON-RPC (see `docs/baseline/tasks.md`). If the spike fails, mark this ADR superseded.

## Verification

Spike checklist: toolsets, annotations, MRTR, Tasks, stdio. Review after spike.
