---
chapter: 4
title: "Beyond Tool Sprawl"
part: "Part II: Thoughtful Design"
use_when: "Designing a tool set: avoiding sprawl and name collisions, cohesive single-purpose tools."
suggested_prerequisites: [ch02]
source_kb: 41
extracted_kb: 26
code_blocks: 45
quality: auto-extracted
---

# Ch 4: Beyond Tool Sprawl

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Beyond Tool Sprawl

You've built your first MCP servers.

When you build an MCP server, you control almost none of this.


### You Don't Control Other Servers

Your MCP server isn't alone.

```text
                    MCP Client
                 (Claude Desktop)
  Your Server       Google Drive      Asana
  (db-explorer)     Server            Server
  • query_db        • get_doc         • get_task
  • list_tables     • create_doc      • create_task
  • get_schema      • list_docs       • list_tasks
The AI sees ALL tools from ALL servers simultaneously
```

The AI must choose between them based on descriptions alone.

- You Don't Control the Client

You cannot force the AI to call your tools in a specific order. You cannot prevent it from calling tools you didn't intend for a particular workflow. You cannot guarantee it will use the "right" tool for a task.


### User: "Show me the sales data"

AI's internal reasoning (you don't see this): - Found 3 potential tools: query_db, get_report, fetch_data - query_db description mentions "SQL queries" - get_report description mentions "sales reports" - fetch_data description is vague: "fetches data" - Choosing: get_report (best match for "sales")


### The User Has Some Control (But Not You)

Modern MCP clients like Claude Desktop and ChatGPT provide users with control mechanisms:

"Use only the database server for this task" "Don't use the Asana server right now"

/analyze-schema  - A prompt that structures how schema analysis should proceed


### /generate-report  - A prompt that defines report generation workflow

But notice: the user has this control, not you as the developer.

```rust
correctly:

// Poor design: vague, overlapping with common names
Tool::new("get")
    .description("Gets data")

// Better design: specific, clear purpose
Tool::new("query_sales_database")
    .description("Execute read-only SQL queries against the sales PostgreSQL database.
```


### 2. Resource Design

How you expose data as resources affects discoverability and appropriate usage:

```rust
// Resources are for stable, addressable data
Resource::new("sales://schema/customers")
    .description("Customer table schema including all columns and constraints")
    .mime_type("application/json")
```

```rust
// Prompts give users control over multi-step operations
Prompt::new("analyze-sales-trend")
    .description("Analyze sales trends over a specified period")
    .arguments(vec![
        PromptArgument::new("period").description("Time period: daily, weekly,
monthly"),
        PromptArgument::new("metric").description("Metric to analyze: revenue, units,
customers"),
    ])
```

Avoid Anti-Patterns: Why "50 confusing tools" fails and what to do instead 2. Single Responsibility: Why each tool should do one thing well


### The Anti-Pattern: 50 Confusing Tools

The most common mistake when building MCP servers is treating them like REST APIs.

```text
# ... 35 more endpoints
```


### The naive approach converts each endpoint to a tool:

```rust
// DON'T DO THIS
let tools = vec![
    Tool::new("create_product"),
    Tool::new("list_products"),
    Tool::new("get_product"),
    Tool::new("update_product"),
    Tool::new("delete_product"),
    Tool::new("add_product_image"),
    Tool::new("remove_product_image"),
    Tool::new("get_product_reviews"),
    Tool::new("add_product_review"),
    Tool::new("update_inventory"),
    Tool::new("list_categories"),
    Tool::new("create_category"),
    // ... 35 more tools
];
```

This creates a nightmare for AI clients.

When an AI sees 47 tools, it must evaluate each one against the user's request.


### User: "Add a new laptop to the store"

AI must consider: - create_product? (should set initial stock?) - list_categories?

With 47 tools, the AI might: - Choose the wrong tool - Call tools in a suboptimal order - Miss required steps - Get confused and ask for clarification


### Problem 2: Name Collisions

Your 47 tools don't exist in isolation.


### Problem 3: Implicit Workflows Hidden

APIs encode workflows implicitly through endpoint sequences.

MCP reality: - AI sees 3 independent tools - No indication they should be called together - User must know to request all three steps - Or AI must infer the workflow (unreliable)


### Problem 4: Description Burden

Each of your 47 tools needs a description good enough for an AI to understand when to use it. Most API endpoints don't have descriptions written for this purpose:

```rust
// Typical API-converted tool (inadequate)
Tool::new("update_inventory")
    .description("Updates inventory")  // Useless for AI decision-making

// What the AI actually needs
Tool::new("update_product_stock_level")
    .description(
        "Set the available quantity for a product in the inventory system. \
        Use this after creating a new product or when restocking. \
        Requires product_id and quantity. Quantity must be non-negative. \
        Returns the updated inventory record with last_modified timestamp."
    )
```


### Case Study: The Naming Collision

A database tool server used  query  as a tool name.

Users reported "sometimes it queries the database, sometimes it searches logs, I can't predict which."


### The Better Approach: Purposeful Design

Instead of converting APIs to tools 1:1, design for how AI clients actually work:


### 1. Focus on User Tasks, Not API Operations

```rust
// Instead of 7 product CRUD tools, one task-focused tool:
Tool::new("manage_product_catalog")
    .description(
        "Create, update, or manage products in the catalog. \
        Handles product details, images, categories, and initial inventory. \
        Provide the operation type and relevant product data."
    )
    .input_schema(json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["create", "update", "add_image", "set_category",
"discontinue"]
            },
            "product": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "price": { "type": "number" },
                    "category": { "type": "string" },
                    "initial_stock": { "type": "integer" }
                }
            }
        }
    }))
```


### 2. Use Prompts for Workflows

Instead of hoping the AI calls tools in the right order, define workflows as prompts:

```rust
Prompt::new("add-new-product")
    .description("Complete workflow to add a new product with images and inventory")
    .template(
        "I'll help you add a new product to the catalog. \
        This will:\n\
        1. Create the product with basic details\n\
        2. Upload any product images\n\
        3. Set initial inventory levels\n\
        4. Assign to appropriate categories\n\n\
        Please provide the product details..."
    )
```


### 3. Use Resources for Reference Data

Instead of  list_categories  and  get_category  tools, expose categories as resources:

```rust
Resource::new("catalog://categories")
    .description("All product categories with IDs and hierarchy")
    .mime_type("application/json")
```

(table/list, cells in reading order)
- Summary: From API to MCP
- API Thinking
- MCP Thinking
- One endpoint = one tool
- One user task = one tool
- CRUD operations
- High-level actions
- Client controls workflow
- Prompts guide workflow
- API Thinking
- MCP Thinking
- Endpoints are independent
- Tools designed for multi-server environment
- Minimal descriptions
- AI-decision-quality descriptions
- 47 endpoints → 47 tools
- 47 endpoints → 8-12 focused tools + prompts + resources


### Cohesive API Design

Cohesion in MCP server design means your tools, resources, and prompts form a unified, understandable whole—both for AI clients that must choose between them and for users who need predictable behavior.


### The Multi-Server Reality

Your MCP server operates in an environment you don't control.

```text
   - create_document, update_document, delete_document,
   - list_documents, search_documents, share_document
```


### 2. asana-server (task management)

```text
   - create_task, update_task, delete_task, list_tasks,
   - create_project, assign_task, set_due_date
```


### 3. salesforce-server (CRM)

```text
   - query_accounts, update_opportunity, list_contacts, log_activity
```


### 4. your-server (you're building this)


### Total tools visible to AI: 20+ (and growing)

Your server's tools must be instantly distinguishable in this crowded environment.

- Prefix tool names with your domain to avoid collisions:

```rust
// Collision risk: generic names
Tool::new("query")           // Collides with postgres-server
Tool::new("search")          // Collides with Google Drive search_documents
Tool::new("list")            // Collides with everything

// Cohesive: domain-specific names
Tool::new("sales_query")     // Clearly your sales system
Tool::new("sales_report")    // Consistent prefix
Tool::new("sales_forecast")  // AI understands these are related
```

```rust
// Inconsistent verbs (confusing)
Tool::new("get_customer")       // "get"
Tool::new("fetch_orders")       // "fetch" - same meaning, different word
Tool::new("retrieve_products")  // "retrieve" - yet another synonym
Tool::new("load_inventory")     // "load" - and another

// Consistent verbs (cohesive)
Tool::new("get_customer")
Tool::new("get_orders")
Tool::new("get_products")
Tool::new("get_inventory")
```

```text
// Flat structure (hard to understand relationships)
```


### vec![

```rust
    Tool::new("create_order"),
    Tool::new("add_item"),
    Tool::new("remove_item"),
    Tool::new("apply_discount"),
    Tool::new("calculate_total"),
    Tool::new("submit_order"),
    Tool::new("cancel_order"),
]

// Hierarchical structure (clear relationships)
// Order lifecycle tools
Tool::new("order_create")
    .description("Create a new order. Returns order_id for subsequent operations.")

Tool::new("order_modify")
    .description("Add items, remove items, or apply discounts to an existing order.")
    .input_schema(json!({
        "properties": {
            "order_id": { "type": "string" },
            "action": {
                "type": "string",
                "enum": ["add_item", "remove_item", "apply_discount"]
            }
        }
    }))

Tool::new("order_finalize")
    .description("Calculate totals and submit the order, or cancel it.")
    .input_schema(json!({
        "properties": {
            "order_id": { "type": "string" },
            "action": {
                "type": "string",
                "enum": ["submit", "cancel"]
            }

        }
    }))
```

Three tools instead of seven, with clear lifecycle stages.

```rust
// Template: What it does | When to use it | What it returns

Tool::new("sales_query")
    .description(
        "Execute SQL queries against the sales database. \
        Use for retrieving sales records, revenue data, and transaction history. \
        Returns query results as JSON array of records."
    )

Tool::new("sales_report")
    .description(
        "Generate formatted sales reports for a date range. \
        Use when the user needs summaries, trends, or printable reports. \
        Returns report data with totals, averages, and visualizable metrics."
    )

Tool::new("sales_forecast")
    .description(
        "Predict future sales based on historical data. \
        Use when the user asks about projections, predictions, or planning. \
        Returns forecast data with confidence intervals."
    )
```


### The AI can now distinguish:

Raw data needs →  sales_query Summaries/reports →  sales_report Future predictions →  sales_forecast

```rust
Tool::new("sales_query")
    .description(
        "Execute read-only SQL queries against the sales database. \
        Use for retrieving sales records and transaction history. \
        \
        NOTE: This tool CANNOT modify data. For updates, use sales_admin. \
        NOTE: For reports and summaries, use sales_report instead (faster)."
    )
```

- Tools in the same domain should return consistent structures:

```text
// All sales tools return a consistent envelope
{
    "success": true,
    "data": { /* tool-specific data */ },
    "metadata": {
        "query_time_ms": 45,
        "source": "sales_db_replica",
        "cached": false
    }
}
```

```rust
// TOOLS: Actions on the sales domain
Tool::new("sales_query")
Tool::new("sales_report")
Tool::new("sales_forecast")

// RESOURCES: Reference data for sales operations
Resource::new("sales://schema")
    .description("Sales database schema - tables, columns, relationships")
Resource::new("sales://regions")
    .description("List of sales regions with IDs and territories")
Resource::new("sales://products")
    .description("Product catalog with IDs, names, and categories")

// PROMPTS: Guided workflows combining tools and resources
Prompt::new("quarterly-sales-analysis")
    .description("Comprehensive quarterly sales analysis with trends and forecasts")
Prompt::new("sales-territory-review")
    .description("Review sales performance by territory with recommendations")
```


### The AI sees a complete, cohesive sales domain:

Resources provide context (what data exists) Tools provide actions (what can be done)


### The "Explain It" Test

Describe your server to a colleague in one sentence. If you can't, your tools aren't cohesive.

```text
       and does some customer stuff"
```

```text
       reports, and forecasting future sales"
```


### The "New Tool" Test

When you add a new tool, does its name and description obviously fit with existing tools?


### Existing: sales_query, sales_report, sales_forecast

Adding customer support?


### Advanced: Foundation and Domain Servers

As your organization scales MCP adoption, cohesion becomes even more critical.


### The Pattern

Instead of building monolithic servers or having every team create their own database tools, you create a layered architecture:

```text
                Business Users
 Sales Manager         Finance Manager    Domain Servers
 Domain Server         Domain Server      (department-
                                           specific)
• pipeline_view       • budget_check
• territory_perf      • expense_report
• forecast_q4         • revenue_audit
           Foundation Server    Foundation Server
           (db-explorer)        (general-purpose)
           • db_query
           • db_schema
           • db_export
```


### Why This Matters for Cohesion

Foundation Servers are general-purpose, reusable across the organization:

```rust
// Sales Manager Domain Server
// Wraps db-explorer but exposes only sales-relevant operations

Tool::new("pipeline_view")
    .description("View sales pipeline with deal stages and probabilities")
    // Internally calls db_query with pre-built sales pipeline query

Tool::new("territory_performance")
    .description("Compare territory performance against targets")
    // Internally calls db_query + db_export for territory reports

Prompt::new("weekly-forecast")
    .description("Generate weekly sales forecast for your territories")
    // Guides the manager through a structured forecasting workflow
```


### Benefits

1.


### When to Use This Pattern

Multiple departments need different views of the same data You want to control what each role can access

Business users shouldn't need to understand database schemas You're scaling from one team to organization-wide MCP adoption

Your tools don't exist in isolation—they compete for the AI's attention alongside dozens of other tools.

Next, we'll examine the single responsibility principle—why each tool should do one thing well.


### Single Responsibility for Tools

The single responsibility principle for MCP tools isn't about code organization—it's about AI comprehension.

```rust
Tool::new("data_operation")
    .description("Perform data operations - query, insert, update, delete, export,
import, validate, transform")
    .input_schema(json!({
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["query", "insert", "update", "delete", "export", "import",
"validate", "transform"]
            },
            "table": { "type": "string" },
            "data": { "type": "object" },
            "format": { "type": "string" },
            "options": { "type": "object" }
        }
    }))
```


### 1. AI Decision Paralysis

The AI must understand 8 different behaviors from one tool. When a user says "get me the sales data," the AI must reason:


### User: "get me the sales data"

AI reasoning about data_operation: - Is this a "query" operation? - Or should I "export" to get the data? - What's the difference between query and export here? - The description doesn't clarify... - Maybe I should ask the user?


### 2. Parameter Confusion

Different operations need different parameters, but they share one schema:

```text
// For "query": table and maybe some filter options
// For "insert": table and data object
// For "export": table and format
// For "transform": data and transformation options

// All crammed into one ambiguous schema
{
    "table": "???",     // Required for some, ignored by others
    "data": "???",      // Sometimes input, sometimes not
    "format": "???",    // Only for export
    "options": "???"    // Means different things per operation
}
```


### 3. Error Messages Are Vague

When something goes wrong, what failed?

```json
{
    "error": "Invalid parameters for data_operation"
}
```

```rust
// READ operations
Tool::new("db_query")
    .description(
        "Execute read-only SQL queries. \
        Use for retrieving data from any table. \
        Returns results as JSON array."
    )
    .input_schema(json!({
        "required": ["sql"],
        "properties": {
            "sql": { "type": "string" },
            "limit": { "type": "integer", "default": 100 }
        }
    }))

// WRITE operations (separate from read for safety)
Tool::new("db_modify")
    .description(
        "Insert, update, or delete records. \
        Use when the user explicitly requests data changes. \
        Returns affected row count."
    )
    .input_schema(json!({
        "required": ["operation", "table"],
        "properties": {
            "operation": { "enum": ["insert", "update", "delete"] },
            "table": { "type": "string" },
            "data": { "type": "object" },
            "where": { "type": "string" }
        }
    }))

// EXPORT operations
Tool::new("db_export")
    .description(
        "Export table data to file formats (CSV, JSON, Parquet). \
        Use when user needs to download or share data. \
        Returns file path or download URL."
    )

    .input_schema(json!({
        "required": ["table", "format"],
        "properties": {
            "table": { "type": "string" },
            "format": { "enum": ["csv", "json", "parquet"] },
            "filter": { "type": "string" }
        }
    }))

// VALIDATION operations
Tool::new("db_validate")
    .description(
        "Check data integrity and validate against schemas. \
        Use before imports or to diagnose data issues. \
        Returns validation report."
    )
```


### Now the AI's job is clear:

User wants data? →  db_query User wants to change data? →  db_modify User wants a file? →  db_export User wants to check data? →  db_validate


### Helping AI Generate Correct SQL

When your tool accepts SQL queries, the AI must generate syntactically correct SQL for your specific database.

(table/list, cells in reading order)
- Database
- Date Literal
- PostgreSQL
- '2024-01-
- 15'::date
- String Concat
- Window Functions
- JSON Access
- `
- `
- MySQL
- 01-15', '%Y-%m-

```text
CONCAT()
```

(table/list, cells in reading order)
- STR_TO_DATE('2024-
- %d')
- TO_DATE('2024-01-
- 15', 'YYYY-MM-DD')
- `
- MySQL 8+ only

```text
JSON_EXTRACT()
```

(table/list, cells in reading order)
- Full support
- json_extract_scalar(
- '2024-01-15'
- `
- `
- Oracle
- Amazon Athena
- SQLite
- Always specify the database flavor in your tool description:

```rust
// POOR: AI doesn't know which SQL dialect to use
Tool::new("db_query")
    .description(
        "Execute read-only SQL queries. \
        Returns results as JSON array."
    )

// BETTER: AI knows the exact database engine
Tool::new("db_query")
    .description(
        "Execute read-only SQL queries against PostgreSQL 15. \
        Supports all PostgreSQL features including WINDOW functions, \
        CTEs, LATERAL joins, and JSON operators (->>, @>). \
        Use PostgreSQL-specific date functions (DATE_TRUNC, EXTRACT). \
        Returns results as JSON array."
    )

// FOR ATHENA: Specify Presto/Trino SQL dialect
Tool::new("athena_query")
    .description(
        "Execute read-only queries against Amazon Athena (Trino SQL). \
        Use Presto SQL syntax: CONCAT() for strings, DATE '2024-01-15' \
        for date literals, json_extract_scalar() for JSON. \
        Supports WINDOW functions and CTEs. \
        Returns results as JSON array with max 1000 rows."
    )
```


### Why This Matters

When a user asks "show me sales by month for 2024," the AI must generate SQL:


### Without dialect information:

-- AI might generate generic SQL that fails SELECT MONTH(sale_date), SUM(amount) FROM sales WHERE YEAR(sale_date) = 2024 GROUP BY MONTH(sale_date) -- Fails on PostgreSQL: MONTH() doesn't exist


### With PostgreSQL specified:

-- AI generates PostgreSQL-correct SQL SELECT DATE_TRUNC('month', sale_date) AS month, SUM(amount) FROM sales WHERE sale_date >= '2024-01-01' AND sale_date < '2025-01-01' GROUP BY DATE_TRUNC('month', sale_date) ORDER BY month


### With Amazon Athena specified:

-- AI generates Athena/Presto-correct SQL SELECT DATE_TRUNC('month', sale_date) AS month, SUM(amount) FROM sales WHERE sale_date >= DATE '2024-01-01' AND sale_date < DATE '2025-01-01' GROUP BY DATE_TRUNC('month', sale_date) ORDER BY month

```rust
Tool::new("analytics_query")
    .description(
        "Execute analytical queries against ClickHouse. \
        Optimized for aggregations over large datasets. \
        Supports: WINDOW functions, Array functions (arrayJoin, groupArray), \
        approximate functions (uniq, quantile), sampling (SAMPLE 0.1). \
        Use ClickHouse date functions: toStartOfMonth(), toYear(). \
        Column-oriented: SELECT only columns you need for best performance."
    )
```

```text
    toStartOfMonth(sale_date) AS month,
    uniq(customer_id) AS unique_customers,  -- Approximate count, very fast
    quantile(0.95)(amount) AS p95_amount    -- 95th percentile
```

```rust
// MySQL 5.7 - limited window function support
Tool::new("legacy_query")
    .description(
        "Query against MySQL 5.7. \
        Note: WINDOW functions not supported. \
        Use subqueries or temporary tables for ranking/running totals."
    )

// MySQL 8.0 - full modern SQL support
Tool::new("modern_query")
    .description(
        "Query against MySQL 8.0. \
        Full WINDOW function support (ROW_NUMBER, RANK, LAG/LEAD). \
        Supports CTEs (WITH clause) and JSON_TABLE()."
    )
```


### The "One Sentence" Rule

If you can't describe what a tool does in one clear sentence, it's doing too much:

```text
// FAIL: Multiple responsibilities
```

```text
// PASS: Single responsibility
```


### Balancing Granularity

Single responsibility doesn't mean creating hundreds of micro-tools.

```yaml
abstraction:
```


### Too Granular (tool explosion)

```rust
Tool::new("select_from_customers")
Tool::new("select_from_orders")
Tool::new("select_from_products")
Tool::new("select_with_where")
Tool::new("select_with_join")
Tool::new("select_with_group_by")
// 50 more query variations...
```


### Too Coarse (swiss army knife)

```rust
Tool::new("database")  // Does everything database-related
```


### Just Right (task-oriented)

```rust
Tool::new("db_query")      // Read data with SQL
Tool::new("db_schema")     // Explore table structures
Tool::new("db_export")     // Export to files
Tool::new("db_admin")      // Administrative operations (with appropriate guards)
```

```rust
// Read operations: safe, can be used freely
Tool::new("db_query")
    .description("Read-only queries - safe for exploration")

// Write operations: need confirmation
Tool::new("db_modify")
    .description("Modifies data - AI should confirm with user before destructive
```


### operations")

```rust
// Admin operations: restricted
Tool::new("db_admin")
    .description("Administrative operations - requires explicit user authorization")
    .annotations(json!({
        "requires_confirmation": true,
        "risk_level": "high"
    }))
```

With separate tools, you can apply different security policies to each.

```rust
// Multi-purpose tools can't be combined
Tool::new("analyze_and_report")  // Does analysis AND reporting
// What if user wants analysis without report? Too bad.

// Single-purpose tools compose flexibly
Tool::new("db_query")           // Get the data
Tool::new("data_analyze")       // Analyze it
Tool::new("report_generate")    // Create report

// AI can now:
// - Query without analysis
// - Analyze without report
// - Query, analyze, AND report
// - Any combination the user needs
```

```rust
// FAIL: Can't partially use
Tool::new("fetch_and_format_data")
// What if user wants raw data without formatting?

// PASS: Separable concerns
Tool::new("fetch_data")
Tool::new("format_data")
```


### The "Who Cares" Test

For each operation in a tool, ask: "Would a different user care about just this operation?"

```text
// In "data_operation":
// - query: Data analysts care about this
// - insert: Application developers care about this
// - export: Business users care about this
// - validate: Data engineers care about this

// Different audiences = different tools
```


### The "Change Impact" Test

If the tool's behavior needs to change, how much else breaks?

```rust
// Multi-purpose: changing export format affects everything
Tool::new("data_operation")  // Export format change touches all code paths

// Single-purpose: changes are isolated
Tool::new("db_export")  // Only export code needs to change
```

(table/list, cells in reading order)
- Summary
- Single responsibility for MCP tools means:
- Principle
- Benefit
- One clear purpose per tool
- AI selects correctly
- Focused parameter schemas
- Less confusion, better errors
- Principle
- Benefit
- Separable concerns
- Users get exactly what they need
- Composable operations
- Flexible workflows
- Isolated safety controls
- Appropriate permissions per operation

You're writing tools for AI clients that must choose correctly from dozens of options.
