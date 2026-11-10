# Architecture decision records

Format: `NNNN-short-title.md`, copied from [0000-template.md](0000-template.md).

## Index
| ID | Title | Status | Supersedes |
|---|---|---|---|
| [0001](0001-pmcp-server.md) | PMCP server | accepted | - |
| [0002](0002-cel.md) | CEL expressions | accepted (library: `cel` 0.15.0) | - |
| [0003](0003-driver-rpc.md) | Driver RPC (JSON-RPC 2.0) | accepted for protocol; encoding open | - |
| [0004](0004-record-decisions.md) | Record architecture decisions | accepted | - |
| [0005](0005-observability-and-audit.md) | Logging, tracing and audit | accepted | - |

## Rules
- Never edit an accepted ADR's decision. Write a new ADR that supersedes it.
- Constitution changes that come from a decision cite the ADR.
