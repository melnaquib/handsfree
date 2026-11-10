---
chapter: 10
title: "Config-Driven OpenAPI Servers"
part: "Part III: Deployment"
use_when: "Wrapping a REST API via OpenAPI spec; backend auth (oauth2_client_credentials, oauth_passthrough); inline resources/prompts."
suggested_prerequisites: [ch09]
source_kb: 14
extracted_kb: 7
code_blocks: 12
quality: auto-extracted
---

# Ch 10: Config-Driven OpenAPI Servers

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Config-Driven OpenAPI Servers: From Scaffold to Deploy

Earlier you hand-built MCP servers in Rust — writing a tool handler per operation, managing the client, and recompiling on every change. For those, PMCP offers a config-driven path where the server is described in a  config.toml  (plus an optional OpenAPI spec) instead of written in Rust — and  cargo pmcp  scaffolds, runs, and deploys it for you.


### What You'll Learn

When a config-driven OpenAPI server beats a hand-coded one (and when it doesn't) How  cargo pmcp new --kind openapi-server  scaffolds a deployable crate The two kinds of tools — single-call and script — and the Code Mode long tail The six outgoing-auth variants and the  ${ENV}  secret discipline How


### Prerequisites

```bash
# The PMCP CLI
cargo install cargo-pmcp
```


### The Pareto Split

An API-backed MCP server faces two kinds of demand. A much larger long tail of ad-hoc questions cannot be enumerated in advance.


### Config-driven servers split the work cleanly:

```text
                CONFIG-DRIVEN OPENAPI MCP SERVER
config.toml                                          api.yaml (--spec)
   [[tools]]                                     OpenAPI contract
    get-tube-status                              (served as the Code
    disrupted-lines (script)                      Mode api_schema)
      the curated ~20%
   Named MCP tools                         Code Mode (validate_code /
   (single-call + script)                  execute_code) — the long-tail
                                           ~80%, scripts under policy
                one HTTP engine (reqwest::Client + outgoing auth)
```

You curate the 20% as  [[tools]] ; Code Mode covers the 80%.


### Step 1: Scaffold

```bash
cargo pmcp new tube-api --kind openapi-server
```

```text
Cargo.toml          # pins pmcp-server-toolkit (openapi-code-mode) + pmcp-openapi-server
src/main.rs         # generated wiring — the only Rust, and you rarely touch it
config.toml         # [server] / [backend] / [code_mode] + a single-call + a script tool
api.yaml            # minimal OpenAPI spec (optional at runtime)
deploy.toml         # deploy descriptor (human-visible)
.pmcp/deploy.toml   # the copy cargo pmcp deploy reads
```

The generated  src/main.rs  loads the config (and  api.yaml  if present), calls  dispatch  to build the connector + Code Mode executor pair, calls  build_server  to assemble the tools, and serves over streamable HTTP.


### Step 2: Run It Locally

```bash
cargo run
# serves over streamable HTTP on 127.0.0.1:8080 by default
```

Connect your MCP client (or  cargo pmcp test check <addr> ) to the address.


### Step 3: Two Kinds of Tools

The most important new idea over the SQL server is that  [[tools]]  comes in two kinds (D-01).

```toml
# single-call: one endpoint, no parameters.
[[tools]]
name = "get-tube-status"
description = "Get the current status of all London Underground lines."
path = "/Line/Mode/tube/Status"
method = "GET"

# script: fetch statuses, filter to disrupted lines, fan out for per-line detail.
[[tools]]
name = "disrupted-lines-with-detail"
description = "List currently-disrupted tube lines with per-line disruption detail."
script = """
```

```text
  const detail = await api.get(`/Line/${line.id}/Disruption`);
  out.push({ line: line.name, detail: detail, max: args.maxLines });
}
return { count: out.length, lines: out };
```


### """

```toml
[[tools.parameters]]
name = "maxLines"
type = "integer"
required = false
default = 5
```

The script subset is deliberately small:  await api.get(...)  with string or template-literal paths, normal JS filtering/looping,  args. <name>  for declared parameters, and a final  return .


### Step 4: Authenticate to the Backend

The  [backend.auth] type  value picks the outgoing-auth variant.

(table/list, cells in reading order)
- type
- Mechanism
- none
- api_key
- bearer
- basic


### oauth2_client_credentials

OAuth2 client-credentials grant.


### oauth_passthrough

Forwards the incoming MCP client token (SSO).


### Always reference secrets via  ${ENV_VAR}  — never inline a real token:

```toml
[backend]
base_url = "https://api.tfl.gov.uk"

[backend.auth]
type = "api_key"
query_params = { app_key = "${TFL_APP_KEY}" }
required = false
```

An unset  required = false  reference is omitted rather than sent as the literal  ${...}  placeholder.


### The Spec Is Optional (D-03)

--spec  (the scaffold's  api.yaml ) is optional.

contract rather than failing.


### Resources & Prompts (Code Mode Context)

[[resources]]  and  [[prompts]]  blocks ship the context an agent needs to write good Code Mode scripts — straight from config, no Rust.


### A resource is inline markdown addressed by a URI:

```toml
[[resources]]
uri = "docs://london-tube/schema"
name = "TfL Line API Schema"
description = "Endpoints, response shapes, and line ids for the TfL Line API"
mime_type = "text/markdown"
content = """
# TfL Line API (subset)
```

- `GET /Line/Mode/tube/Status` — status for every tube line - `GET /Line/{lineId}/Disruption` — disruption detail for one line

```toml
[[resources]]
uri = "code-mode://learnings"
name = "TfL Code Mode Learnings"
description = "Tips and gotchas for authoring TfL Code Mode scripts"
mime_type = "text/markdown"
content = """
# TfL Code Mode Learnings
```

```toml
[[prompts]]
name = "start_code_mode"
description = "Load all context needed for Code Mode script generation"
include_resources = [
    "docs://london-tube/schema",
    "code-mode://learnings",
]
```

These resources are inert static markdown context the model reads — not executable scripts; the server never runs resource content. Every URI in a prompt's  include_resources  must exactly match a defined  [[resources]]   uri , or the prompt loads against a missing resource.


### Step 5: Deploy

The scaffold's  deploy.toml  targets  pmcp-run  by default.

```bash
cargo pmcp validate deploy        # pre-flight checks, no cloud calls yet
cargo pmcp deploy
cargo pmcp deploy outputs         # deployed endpoint URL
```

The deploy bundles  config.toml  +  api.yaml  and applies the same secret posture as the SQL server: any inline DEV  token_secret  in the bundled config is rewritten to a  ${...}  reference so the deployed artifact never ships a dev literal.

(table/list, cells in reading order)
- When to Use This (and When Not To)
- Use config-driven OpenAPI
- Hand-code instead
- Mostly proxying / orchestrating REST endpoints
- Complex bespoke logic per tool
- You want non-Rust teammates to own tools
- Custom transports / middleware
- Fast iteration on tools
- Non-HTTP backends
- Curated 20% + agent-driven long tail
- Every operation must be explicitly coded
- Exercise: Ship a Two-Tool OpenAPI Server

Success criteria: the single-call tool returns live conditions; the script tool aggregates across cities; the secret is referenced via  ${...} , never inlined; and (stretch) the deployed endpoint authenticates with the env-supplied key.


### Key Takeaways

Config-driven OpenAPI servers describe an HTTP MCP server in  config.toml  (+ optional OpenAPI spec) instead of hand-written Rust — curated tools for the common 20%, Code Mode for the long-tail 80%. Outgoing auth has six variants ( none  +  api_key / bearer / basic /  oauth2_client_credentials / oauth_passthrough ); always reference secrets via  ${ENV} .

```bash
cargo pmcp new --kind openapi-server  scaffolds a deployable crate;  cargo pmcp deploy  bundles assets and swaps dev secrets
```
