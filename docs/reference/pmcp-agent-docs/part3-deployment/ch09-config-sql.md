---
chapter: 9
title: "Config-Driven SQL Servers"
part: "Part III: Deployment"
use_when: "Shipping a SQL MCP server from config only (no Rust), deploying to Lambda."
suggested_prerequisites: [ch03, ch08]
source_kb: 11
extracted_kb: 6
code_blocks: 7
quality: auto-extracted
---

# Ch 9: Config-Driven SQL Servers

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Config-Driven SQL Servers: From Scaffold to Lambda

In Chapter 3 you hand-built a database MCP server in Rust — writing a tool handler per query, managing the connector, and recompiling on every change. For those, PMCP offers a config-driven path where the server is described in a  config.toml  instead of written in Rust — and  cargo pmcp  scaffolds, runs, and deploys it to AWS Lambda for you.


### What You'll Learn

When a config-driven server beats a hand-coded one (and when it doesn't) How  cargo pmcp new --kind sql-server  scaffolds a deployable crate How the curated-tools / Code Mode "Pareto split" covers 100% of queries How  cargo pmcp deploy  ships the server to Lambda, including the secret posture that k


### Prerequisites

```bash
# The PMCP CLI
cargo install cargo-pmcp

# For the Lambda deploy at the end of this chapter:
aws sts get-caller-identity        # AWS credentials configured
cargo install cargo-lambda         # cross-compile to the Lambda runtime
npm install -g aws-cdk             # infrastructure provisioning
```


### The Pareto Split

A production SQL server faces two kinds of demand. A much larger long tail of ad-hoc questions cannot be enumerated in advance.


### Config-driven servers split the work cleanly:

```text
                  CONFIG-DRIVEN SQL MCP SERVER
config.toml                                          schema.sql
   [[tools]]                                     CREATE TABLE ...
    list_books                                   (served as the Code
    books_by_author                               Mode schema resource)
      the curated ~20%
   Named MCP tools                         Code Mode (validate_code /
   (typed parameters)                      execute_code) — the long-tail
                                           ~80%, generated SQL under
                                           policy + approval tokens
                       pmcp-server-toolkit
                       (SQL connector + builder)
```

You curate the 20% as  [[tools]] ; Code Mode (Chapter 12.9 concepts) covers the 80%.


### Step 1: Scaffold

```bash
cargo pmcp new acme-sql --kind sql-server
```

```text
Cargo.toml          # pins pmcp-server-toolkit ["code-mode","sqlite","http"]
src/main.rs         # generated wiring — the only Rust, and you rarely touch it
config.toml         # [server] / [database] / [code_mode] + a list_books tool
schema.sql          # idempotent demo DDL + seed
deploy.toml         # deploy descriptor (human-visible)
.pmcp/deploy.toml   # the copy cargo pmcp deploy reads
```

The generated  src/main.rs  loads  config.toml  and  schema.sql  through  pmcp::assets::load_string  (cwd locally, /var/task/assets/  on Lambda) and opens SQLite at  demo_db_path()  ( /tmp/demo.db  on Lambda).


### Step 2: Run It Locally

```bash
cargo run
# prints: PMCP_SQL_SERVER_ADDR=http://127.0.0.1:<port>
```

Connect your MCP client (or  cargo pmcp test check <addr> ) to the printed address.


### Step 3: Customize Through Config

The config uses  #[serde(deny_unknown_fields)] , so typos fail loudly at startup rather than silently disabling a feature — a property you want in regulated environments.

```toml
[code_mode]
enabled = true
allow_writes = false        # default-deny: read-only
require_limit = true        # every generated query must bound its rows
max_limit = 1000
token_secret = "dev-only-insecure-secret-min-16-bytes"   # DEV ONLY
allow_inline_token_secret_for_dev = true

[[tools]]
name = "books_by_author"
description = "Books written by a given author"
sql = "SELECT id, title FROM books WHERE author = :author ORDER BY title LIMIT :limit"

[[tools.parameters]]
name = "author"
type = "string"
required = true
```


### Step 4: Deploy to AWS Lambda

The scaffold's  deploy.toml  targets  pmcp-run  by default.

```bash
[target]
type = "aws-lambda"
version = "1.0.0"

[aws]
region = "us-east-1"

[server]
name = "acme-sql"
memory_mb = 512
timeout_seconds = 30

[assets]
include = ["config.toml", "schema.sql"]

cargo pmcp validate deploy                  # pre-flight: IAM footgun detection, no AWS calls yet
cargo pmcp deploy --target-type aws-lambda
cargo pmcp deploy outputs                    # deployed endpoint URL
```


### What the deploy does for you

```text
config.toml                               ► /var/task/assets/config.toml  (read-only)
                bundled into the zip
schema.sql                                ► /var/task/assets/schema.sql    (read-only)
runtime opens SQLite                    ► /tmp/demo.db                     (writable)
```

The critical security step is automatic: the deploy path rewrites the bundled config's inline DEV  token_secret  to ${CODE_MODE_SECRET} , so the deployed artifact never ships the dev literal.

```bash
cargo pmcp test conformance <deployed-url>
```

(table/list, cells in reading order)
- When to Use This (and When Not To)
- Use config-driven
- Hand-code instead (Chapter 3)
- Mostly CRUD / reporting over SQL
- Complex business logic per tool
- You want non-Rust teammates to own tools
- You need custom transports/middleware
- Fast iteration on tools & schema
- Non-SQL backends or bespoke connectors
- Curated 20% + agent-driven long tail
- Every operation must be explicitly coded


### Exercise: Ship a Two-Table Server

Goal: scaffold, extend, and deploy a config-driven SQL server.

Edit  deploy.toml  to target  aws-lambda , run  cargo pmcp validate deploy , and resolve any IAM warning it reports. 5.


### Key Takeaways

Config-driven servers describe a SQL MCP server in  config.toml  +  schema.sql  instead of hand-written Rust — curated tools for the common 20%, Code Mode for the long-tail 80%.

```bash
cargo pmcp new --kind sql-server  scaffolds a deployable crate; the same binary runs locally and on Lambda because assets and
the DB resolve to runtime-appropriate paths.
cargo pmcp deploy --target-type aws-lambda  bundles your assets and swaps the dev secret for  ${CODE_MODE_SECRET}
```
