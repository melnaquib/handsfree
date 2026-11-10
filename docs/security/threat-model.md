# Threat model

Status: draft

## Assets
- User desktop input and output
- Screenshots and on-screen data
- Credentials visible on screen

## Trust boundaries
- MCP client to server
- Server to OS automation APIs
- Server to network (if any)

## Threats
| Threat | Impact | Mitigation |
|---|---|---|
| Prompt injection triggers destructive action | data loss | permission gate, confirm |
| Screenshot leaks secrets | credential exposure | redaction, retention limit |
| Malicious tool input | arbitrary input injection | schema validation, allowlist |

## Out of scope
TODO
