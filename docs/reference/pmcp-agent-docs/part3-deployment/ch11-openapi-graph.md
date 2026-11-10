---
chapter: 11
title: "OpenAPI over Microsoft Graph"
part: "Part III: Deployment"
use_when: "Worked example: OpenAPI server over Microsoft Graph / M365."
suggested_prerequisites: [ch10]
source_kb: 9
extracted_kb: 4
code_blocks: 7
quality: auto-extracted
---

# Ch 11: OpenAPI over Microsoft Graph

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### OpenAPI over Microsoft Graph (Contoso M365)

The Config-Driven OpenAPI Servers chapter built an HTTP MCP server from config with an  api_key  backend. The backend is Microsoft Graph, and every read must happen as the signed-in user.


### What You'll Learn

How to point a config-driven OpenAPI server at Microsoft Graph to read an existing Excel workbook — no data migration The  oauth_passthrough  double lock: admin-consent ceiling + per-user forwarded token, with no standing credential on the server Two read-only  customer_id -keyed script tools ( get_


### The Problem (Keep Your Excel Files, Connect Them to AI)

Contoso already has the data.

```toml
[server]
name = "contoso-m365"
version = "1.0.0"
description = "Contoso M365: per-user delegated OAuth (passthrough) reading a Customers/Orders Excel workbook over
```


### Microsoft Graph."

```toml
[backend]
base_url = "https://graph.microsoft.com/v1.0"
```


### Authenticate to the Backend — The Double Lock

This is the headline. The workbook holds sensitive data, so the server must never read more than the asker is allowed to, and it must hold no standing credential to steal.  oauth_passthrough  gives both with one config block, quoted verbatim from the shipped crates/pmcp-openapi-server/examples/contoso-m365.toml :

```toml
[backend.auth]
type = "oauth_passthrough"
target_header = "Authorization"
required = true
```


### Two locks, both engaging on every request:

1.

consent is the ceiling — the most the server could ever request on a user's behalf, and it cannot be exceeded.

The admin ceiling never widens an individual user's reach.

Passthrough is the right choice when the backend's per-user authorization should govern access.


### Two Kinds of Tools (get_customer / get_customer_orders)

Contoso curates two read-only script tools, each keyed by  customer_id .

```toml
[[tools]]
name = "get_customer"
description = "Fetch one customer row (customer_id, name, segment, region) from the Contoso Customers sheet."
script = """
```

```text
return matches;
```


### """

```toml
[[tools]]
name = "get_customer_orders"
description = "Fetch the Orders rows (order_id, customer_id, order_date, amount) for one customer from the Contoso
Orders sheet."
script = """
```

const resp = await api.get("/drives/CONTOSO_DRIVE/items/ORDERS_ITEM/workbook/worksheets/Orders/range(address='A2:D7')? $select=values"); const rows = resp.values; const matches = rows.filter(row => row[1] === args.customer_id);


### """

Both tools read the same whole-sheet block ( A2:D7 ) and differ only in the filter column:  customer_id  is column 0 in Customers, column 1 in Orders.


### Resources & Prompts (Code Mode Context)

The server ships three inert static-markdown resources plus a  start_code_mode  prompt bundling them.

```toml
[[resources]]
uri = "docs://contoso-m365/schema"
name = "Contoso Workbook Schema"

[[resources]]
uri = "docs://contoso-m365/examples"
name = "Contoso Example Scripts"

[[resources]]
uri = "code-mode://learnings"
name = "Contoso Code Mode Learnings"

[[prompts]]
name = "start_code_mode"
description = "Load all context needed for Code Mode script generation against the Contoso workbook"
include_resources = [
    "docs://contoso-m365/schema",
    "docs://contoso-m365/examples",
    "code-mode://learnings",
]
```

Every  include_resources  URI must match a  [[resources]] uri  exactly.


### The Headline Query

The curated tools answer simple questions; the interesting one is uncurated: "customers who bought more than 100 in the last 3 months." That headline Code Mode query reads the whole Customers and Orders blocks over the same Graph range-read API the tools expose (still as the calling user, still forw


### Two properties make it trustworthy:

Deterministic. With no  Date  builtin, the "last 3 months" window is computed from a pinned reference date ( 2026-05-28  → inclusive window  [2026-02-28, 2026-05-28] ), never a wall-clock  now . An offline Code Mode test (Plan 03) asserts the returned set equals the canonical expected set, so it cannot rot as the calendar advances.


### Exercise: Add a Segment Filter

Goal: extend the Contoso surface with an analyst-curated read.


### Key Takeaways

A config-driven OpenAPI server can read an existing Microsoft 365 Excel workbook over Graph with no data migration — the business analyst curates the slice in  config.toml . oauth_passthrough  is a double lock: admin-consent ceiling and per-user forwarded token that further restricts, with no standi
