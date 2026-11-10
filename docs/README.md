# Docs index

Single entry point for all project documentation. Keep this file an index only: link, do not duplicate.

## Read order (new humans and agents)
1. [Vision v1](vision/v1.md)
2. [Constitution](../.kittify/constitution/constitution.md)
3. [ADR index](adr/)
4. [Tech stack](techstack.md)
5. [Current features](../kitty-specs/)

## Map
| Path | Purpose | Version |
|---|---|---|
| [help/](help/) | User-facing documentation | all |
| [vision/](vision/) | Product vision per version | v1, v2 |
| [adr/](adr/) | Architecture decision records | all |
| [tech/](tech/) | Stack, surfaces, references | all |
| [security/](security/) | Threat model, permissions | all |
| [mcp/tools.md](mcp/tools.md) | Tool catalog (public contract) | all |
| [privacy.md](privacy.md) | Data captured, retention, residency | all |
| [platforms.md](platforms.md) | OS support matrix | all |
| [testing.md](testing.md) | Test strategy and how to run | all |
| [runbook.md](runbook.md) | Install, config, troubleshoot | all |
| [release.md](release.md) | Versioning and release flow | all |
| [glossary.md](glossary.md) | Shared terms | all |
| [limitations.md](limitations.md) | Known gaps, deferred items | v1 |
| [backlog-v2.md](backlog-v2.md) | v2 backlog | v2 |
| [brainstorm/](brainstorm/) | Unvalidated ideas (not decisions) | all |

## Status legend
Docs: `draft` | `accepted` | `superseded`. ADRs: `proposed` | `accepted` | `superseded by ADR-NNNN`.

## Rules
- Update this index when a doc is added, moved, or superseded.
- Register key docs in `.kittify/constitution/references.yaml`.
- Use relative links.
