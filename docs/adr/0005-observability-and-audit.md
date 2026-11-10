# 0005. Logging, tracing and audit through `tracing`

* Status: accepted (details verified when needed)
* Date: 2026-11-10
* Deciders: Mustafa

## Context

handsfree needs debug logs, request traces and an audit trail of what the model did. pmcp already emits through `tracing` (62 source files, `tracing-subscriber` behind its default `logging` feature) and has an `ObservabilityConfig` (disabled, development, production) with console and CloudWatch backends, `TraceContext`, a middleware chain, and built-in metrics (`mcp.request.duration`, `.count`, `.errors`, `mcp.response.size`). It has no audit concept, no Prometheus exporter and no OTLP exporter.

Audit differs from logs: it must not be lost, must never contain secrets, is kept longer, and later may need tamper evidence. See ADR [0001](0001-pmcp-server.md) (pmcp) and the constitution ("log every action, never log secrets").

## Options considered

1. `tracing` for everything: logs, traces, and audit as a dedicated target with its own layer and sink.
2. A separate audit system outside `tracing` (own writer, own API).
3. `log` + `env_logger` or `slog`: rejected, pmcp already uses `tracing`.
4. OpenTelemetry as the primary path: rejected for v1. Exporters are batched and best-effort, so they can drop audit data, and the goals say no cloud dependency.

## Decision

* **Logs and traces:** `tracing` + `tracing-subscriber`, with pmcp's `ObservabilityConfig` and the console backend. No CloudWatch.
* **Audit:** `tracing` events with `target: "audit"`, written by their own layer to their own JSONL file.
  * The audit layer has a fixed per-layer filter, independent of `RUST_LOG`.
  * The writer must not drop events (non-blocking writer with lossy off, guard kept alive until exit).
  * Fixed fields: `actor`, `action`, `target_id`, `risk`, `outcome`, `approval`, and the trace id shared with the logs. No secrets.
  * Two levels of events: the MCP boundary (pmcp middleware: request and outcome, no domain knowledge) and the driver (what was done, risk class, approval). Drivers own risk classification (ADR [0003](0003-driver-rpc.md) and the vision).
* **Storage:** JSONL file, written with `tracing-appender` (optional crate, used for the audit target only). Audit analysis and external tools are out of scope.
* **Later, off by default, not in scope now:** hash chain (`sha2`), syslog (RFC 5424) or OTLP sinks, `tracing-opentelemetry`, and a Prometheus exporter (`metrics-exporter-prometheus`) if dashboards are wanted. Each new crate needs the owner's approval.
* **Metrics:** pmcp's built-in collection only. No exporter in v1.

## Consequences

* Good: one API, pmcp events and ours land in the same stream, nothing to bridge.
* Good: audit stays local and works offline. Exporters can be added without touching the call sites.
* Bad: `tracing` has no custom "audit" level, so the separation depends on the target name and per-layer filters. A wrong filter silently loses audit events.
* Bad: a hash chain (later) needs state, so it will need a custom layer, not a plain `fmt` layer.
* Follow-ups: define the audit event schema (field names borrowed from OCSF or ECS, no strict dependency).

## Verification

No spike. Audit requirements are refined later; verify each detail (lossless writer, filter independence, redaction, middleware visibility) when it is needed.
