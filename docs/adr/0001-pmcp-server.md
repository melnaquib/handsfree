# 0001. Use pmcp (Rust) for the MCP server

* Status: accepted
* Date: 2026-11-04
* Deciders: Mustafa

## Context

handsfree needs an MCP server in front of drivers. In v1 the server and the driver are one process.

## Decision

Use the [pmcp](https://crates.io/crates/pmcp) crate for the MCP server. Keep it a thin adapter: handlers parse args, call plain library crates, map errors. Pin exact versions.

Coding conventions: [pmcp agent docs](../reference/pmcp-agent-docs/README.md).

## Consequences

* Good: single toolchain with the Rust drivers, typed tool schemas via `schemars`.
* Bad: depends on one SDK's maturity.
