# Driver protocol options (brainstorm, not requirements)

Raw notes. Decision so far is in [0003](../adr/0003-driver-rpc.md). Claims from README and memory, unverified.

## Still open

1. Encoding: NDJSON vs CBOR.
2. Own mux framing vs none.
3. Cap'n Proto as alternative path.

## Compare


| Option                    | For                                                                                                                                                                         | Against                                                                     |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| JSON-RPC + NDJSON         | simple, debuggable, matches MCP                                                                                                                                             | text overhead                                                               |
| JSON-RPC + CBOR + own mux | compact, binary payloads,[ciborium](https://docs.rs/ciborium)in Rust,[nlohmann/json](https://github.com/nlohmann/json)in C++ | own stream ids, new spec in four languages                                  |
| ConnectRPC                | typed                                                                                                                                                                       | non-standard over pipes, no official C++                                    |
| Cap'n Proto               | handles fit well                                                                                                                                                            | large-payload bug history in bindings |
| yamux                     | proven mux                                                                                                                                                                  | [yamux crate](https://docs.rs/yamux)is Rust only       |

## If CBOR + mux is adopted

* Framing: `u32 len | u8 type | u32 stream_id | payload`.
* Types: `CTRL`, `DATA`, `END`, `RESET`. `CREDIT` later.
* Sender priority: ctrl > audio > video notices > bulk. Chunk bulk about 64 KiB.
* Outside the mux: stderr, hard kill (process group / Job Object).
* One muxed channel, not fixed fds. Fits attach mode, WebSocket, native messaging, Windows.

## Media

* Screenshots: encode once, downscale first, content-addressed files in a private run dir, lazy pull.
* Audio: raw PCM 10-20 ms chunks locally, Opus on network.
* Video: shm ring buffer locally (latest wins), H.264 or AV1 for recording and network.
* Bursts: dedup by hash, lazy pull, dirty rects, or encode as video.

## Spikes to run before deciding

1. Driver streaming screenshots while answering ctrl calls. Measure ctrl latency.
2. shm ring 1080p30, Rust driver to MCP server, vs H.264 over mux.
3. Cap'n Proto over socketpair with 5 MB payloads.
4. Verify C++ codegen from JSON Schema.
