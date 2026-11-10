---
chapter: 12
title: "Config-Driven Workbook Servers"
part: "Part III: Deployment"
use_when: "Serving a governed Excel workbook as an MCP server: compile, serve, govern, deploy."
suggested_prerequisites: [ch08]
source_kb: 16
extracted_kb: 6
code_blocks: 9
quality: auto-extracted
---

# Ch 12: Config-Driven Workbook Servers

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Config-Driven Workbook Servers: From Spreadsheet to Served MCP

Earlier you hand-built calculation MCP servers in Rust — wiring a  ServerBuilder , writing a tool handler for every formula, loading and versioning your model, and recompiling on every change.

Instead of a  config.toml  you author Rust-free, the input here is a governed Excel workbook that you compile into a served bundle.


### What You'll Learn

When a workbook-driven server beats a hand-coded one (and when it doesn't) How  cargo pmcp workbook compile  turns a governed Excel file into a deterministic  bundle@version  directory The two ways to serve a bundle — the prebuilt  pmcp-workbook-server  binary (no Rust) and a  cargo pmcp new --kind 


### Prerequisites

```bash
# The PMCP CLI (provides `cargo pmcp workbook compile` and the scaffold)
cargo install cargo-pmcp

# The prebuilt no-Rust binary (Shape A) — optional, only if you want to
# serve a bundle without scaffolding a crate:
cargo install pmcp-workbook-server
```

You also need a governed Excel workbook that conforms to the workbook dialect.


### The Single-Source-of-Truth Model

A SQL or OpenAPI built-in server is described in a  config.toml  you write by hand.

```text
                 WORKBOOK-DRIVEN MCP SERVER
governed Excel workbook            cargo pmcp workbook compile
   inputs / outputs              ►   ingest → lint → synth → parse →
   formulas                          compile → reconcile → GATE → write
   render pointers
   (the source of truth)
                                    bundles/<name>@<version>/
                                    (deterministic, BUNDLE.lock-sealed)
                                pmcp-workbook-server  (Shape A binary)
                                or  --kind workbook-server  (Shape B)
                                → 5 tools + workbook:// resource
```


### Step 1: Compile the Workbook

The compiler is a  cargo pmcp  subcommand.

```bash
cargo pmcp workbook compile tax-calc.xlsx --workflow tax-calc --approver alice
```


### bundles/

```text
tax-calc@1.1.0/        # <name>@<version>; version is read FROM the workbook
    BUNDLE.lock        # integrity hashes re-verified at boot
    ...                # manifest, compiled formulas, render pointers
```


### A few properties worth internalizing:

The bundle is deterministic and reproducible — the same workbook compiles to the same bundle.


### You can lint a workbook on its own first if you just want feedback:

```bash
cargo pmcp workbook lint tax-calc.xlsx
```


### Step 2: Serve the Bundle — Two Ways

The only input to serving is the compiled bundle directory.


### Shape A — the prebuilt binary (no Rust)

If you installed  pmcp-workbook-server , point it at the bundle directory:


### pmcp-workbook-server --bundle-dir bundles/tax-calc@1.1.0

```text
# serves over streamable HTTP on 127.0.0.1:8080 by default
```

(table/list, cells in reading order)
- --bundle-dir  is the only required flag. Two optional flags matter:
- Flag
- Required
- Default
- Purpose
- --bundle-
- dir
- --bundle-
- id
- --http
- Yes
- No
- No
- —
- —

Path to the compiled  bundle@version  directory.

Fail-closed identity assertion: the loaded bundle's id must equal this value, or the binary exits non-zero before registering any tool.


### 127.0.0.1:8080

Streamable-HTTP bind address ( host:port ).

```text
# Assert identity (fail-closed) and bind a public address:
pmcp-workbook-server --bundle-dir bundles/tax-calc@1.1.0 \
  --bundle-id tax-calc \
  --http 0.0.0.0:9000
```


### Shape B — scaffold an extendable crate

If you want a crate you can extend — add your own transports, middleware, or additional handlers around the workbook surface — scaffold one over the  pmcp-server-toolkit  library:

```bash
cargo pmcp new tax-server --kind workbook-server
```


### cd tax-server

```bash
cargo run
# prints: PMCP_WORKBOOK_SERVER_ADDR=http://... — connect your MCP client there
```

The scaffold is an ordinary Rust crate with its own  main.rs  that uses the toolkit. It embeds a workbook ( workbook/tax-calc.xlsx ); when you edit that workbook, recompile the embedded bundle and rerun:

```bash
cargo pmcp workbook compile
cargo run
```


### Step 3: What You Get — Per-Table Tools and a Render Resource

Either way you serve it, a compiled bundle is exposed as one calculation tool per output Table plus four fixed infrastructure tools and one resource — no per-tool Rust.

(table/list, cells in reading order)
- Tool
- Purpose
- (one per output Table, e.g.  calculate_tax )
- Run that Table's calculation against supplied inputs
- explain
- get_manifest
- diff_version
- render_workbook
- Explain how a result was derived
- Return the bundle manifest (identity, inputs, outputs)
- Compare results or definitions across versions
- Produce a render of the workbook


### Step 4: Customize Through the Workbook

This is the key mental shift from the SQL and OpenAPI servers.


### Step 5: Govern and Boot Safely

Two governance properties are worth teaching explicitly, because they are what make it safe to hand a compiled bundle to a server you did not hand-write.

A tampered, incomplete, or missing bundle fails boot with a non-zero exit — the server never comes up serving partial or wrong tools. When you pass --bundle-id , the binary additionally asserts the loaded bundle's identity before registering anything, and exits non-zero on a mismatch.


### Step 6: Deploy

Because the Shape B scaffold is a normal crate over the  pmcp-server-toolkit  library, the same deploy path you used for the SQL and OpenAPI built-in servers applies.

```bash
cargo pmcp validate deploy        # pre-flight checks, no cloud calls yet
cargo pmcp deploy
cargo pmcp deploy outputs         # deployed endpoint URL
```

For the per-target mechanics, lean on the existing chapters rather than re-learning them here:

(table/list, cells in reading order)
- When to Use This (and When Not To)
- Use workbook-driven
- Hand-code instead (Chapter 3)
- The model already lives in a governed spreadsheet
- Logic is bespoke Rust, not a calc model
- Domain owners maintain it in Excel
- Engineers own every formula in code
- You need deterministic, versioned, gated bundles
- You need custom transports / middleware
- Pure calculation (per-table calculate tools /  explain  /  diff )
- The backend isn't a calculation workbook


### Exercise: Compile, Serve, and Version a Workbook

Goal: take a governed workbook through the full compile → serve → version lifecycle.


### Key Takeaways

Workbook-driven servers make the governed Excel workbook the source of truth.
