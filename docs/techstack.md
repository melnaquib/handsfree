# Tech stack

Status: decided unless marked open. New crates need the owner's approval (see the constitution). Approved crates are pinned in `[workspace.dependencies]` in `Cargo.toml`; a member crate adds one only when code uses it.

| Area | Choice | Why / ADR |
|---|---|---|
| Language | Rust | single toolchain, ADR [0001](adr/0001-pmcp-server.md) |
| MCP server | pmcp | [0001](adr/0001-pmcp-server.md) |
| Tool schemas | schemars | typed tool schemas |
| Async runtime | tokio | pmcp is built on it |
| Expressions | CEL, `cel` 0.15.0 (exact pin) | [0002](adr/0002-cel.md) |
| Driver RPC | JSON-RPC 2.0, NDJSON | [0003](adr/0003-driver-rpc.md) |
| GUI | egui, eframe | |
| System tray | tray-icon (egui example) | |
| GUI testing | egui_kittest (kittest, AccessKit) | |
| Screen capture | gstreamer-rs, plugins added as needed | video, KVM capture |
| Linux a11y | xa11y (exact pin) | other a11y crates only if needed |
| Config | confy only (figment parked) | [tech/config.md](tech/config.md) |
| CLI | clap | |
| Errors | thiserror 2 (libraries), anyhow (binaries) | constitution |
| MCP testing | mcp-tester, cargo-pmcp scenarios | [pmcp workflow](tech/pmcp-workflow.md) |
| Toolchain | `rust-toolchain.toml` + `rust-version` | pinned to stable 1.99.1 |
| Process | Spec Kitty | `kitty-specs/` |
| Brand data | `brand/brand.toml` | colors, tagline, repo |
| Logging, tracing | `tracing` + `tracing-subscriber`, pmcp `ObservabilityConfig` (console) | [0005](adr/0005-observability-and-audit.md) |
| Audit | `tracing` events, target `audit`, own layer, JSONL via `tracing-appender` (optional, audit target only); analysis tools out of scope | [0005](adr/0005-observability-and-audit.md), proposed |
| Metrics | pmcp built-in collection; Prometheus exporter parked | [0005](adr/0005-observability-and-audit.md) |
