# Runbook

## Install
TODO

## Configure
See [.kittify/config.yaml](../.kittify/config.yaml) if used.

## Troubleshoot
- TODO symptom: cause, fix

## Kill switch
TODO

## Logs
TODO location

## HTTP transport environment (PMCP template)

```bash
RUST_LOG=info              # Logging level
MCP_HTTP_PORT=3000         # Port (or PORT)
MCP_ALLOWED_ORIGINS=*      # CORS origins. Do not use * in production.
```

Note: the handfree spec uses stdio by default. HTTP is for remote opt-in only (see README non-goals).
