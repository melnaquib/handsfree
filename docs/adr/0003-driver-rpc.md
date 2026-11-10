# 0003. Driver RPC: JSON-RPC 2.0

* Status: accepted for protocol; encoding open
* Date: 2026-11-04
* Deciders: Mustafa

## Context

Drivers are heterogeneous (Rust a11y desktop, VT100, Blender, C++ Qt probe, API-based). Attach-mode drivers cannot be stdio children, so the link needs more than pipes. In v1 the driver is linked into the server process; this RPC applies to out-of-process drivers. MCP 2026-07-28 uses JSON-RPC text, stdio NDJSON and Streamable HTTP, with no standard WebSocket.

## Options considered

1. JSON-RPC 2.0 over NDJSON on stdio, WebSocket adapter later.
2. JSON-RPC + CBOR + own mux: more efficient, but a new spec in four languages. Conformance suite is the real cost.
3. ConnectRPC, Cap'n Proto, yamux: rejected or blocked (non-standard over pipes, no official C++).
4. Rejected: SSE, gRPC, RSocket, ZeroMQ, pickle, MQTT/NATS, D-Bus, shared memory as base link, REST over loopback, MCP as driver protocol.

## Decision

Use JSON-RPC 2.0. Default encoding NDJSON on stdio. CBOR stays optional until measured. Media goes out of band (content-addressed files, shm ring buffer).

Streams are tags on notifications (`op`, `event`, `system`) with run id and per-run sequence number. No subscriptions.

## Consequences

* Good: debuggable, matches MCP, simple to implement in any language.
* Bad: JSON overhead for bulk data. Mitigated by out-of-band media.
* Security: the MCP server enforces max frame size, concurrent stream cap and per-stream buffer limits before allocating. The MCP server re-validates all out-of-process driver input.
* Follow-ups: decide CBOR + mux only after a measured spike. If adopted, framing is `u32 len | u8 type | u32 stream_id | payload`.

## Verification

Spike: driver streaming screenshots while answering control calls. Measure control latency.
