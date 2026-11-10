---
chapter: 3
title: "Database MCP Servers"
part: "Part I: Foundations"
use_when: "Building a database MCP server: list_tables/execute_query tools, SQL injection safety, resources, pagination."
suggested_prerequisites: [ch02]
source_kb: 83
extracted_kb: 53
code_blocks: 70
quality: auto-extracted
---

# Ch 3: Database MCP Servers

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Database MCP Servers

Database access is the killer app for enterprise MCP.

(table/list, cells in reading order)
- What You'll Learn
- Section
- Topics
- The Enterprise Data Access Problem
- Why database access is MCP's killer app, the friction it eliminates
- Building db-explorer
- SQL Safety and Injection Prevention
- Step-by-step server creation, query tools, schema introspection
- Security patterns, parameterized queries, allowlisting
- Resource-Based Data Patterns
- When to use resources vs tools, structured access patterns
- Handling Large Results
- Pagination, streaming, cursor-based navigation
- Quick Preview


### User: "Show me our top 10 customers by revenue"

Claude: I'll query the sales database for you.


### The Architecture

```text
Claude / AI Client
      MCP Protocol
```

```text
               Database MCP Server
list_tables         query          Resources
   Tool             Tool          (optional)
              Connection Pool
              (sqlx + Arc)
```

```text
 Database
(SQLite,
PostgreSQL,
MySQL)
```

- Before starting this chapter, you should have:

Completed Chapter 2: Your First Production Server Basic familiarity with SQL A sample database to work with (we'll provide one)


### Sample Database

We'll use the Chinook database—a sample database representing a digital media store with customers, invoices, tracks, and artists.

```bash
# Download the sample database
curl -L -o chinook.db https://github.com/lerocha/chinook-
```

(table/list, cells in reading order)
- database/raw/master/ChinookDatabase/DataSources/Chinook_Sqlite.sqlite
- Chapter Sections
- 1. The Enterprise Data Access Problem
- Understand why database access is MCP's killer app for enterprises:

(table/list, cells in reading order)
- Testing with MCP Inspector and Claude
- 3. SQL Safety and Injection Prevention
- Master security patterns for database access:
- SQL injection attacks and prevention Parameterized queries with sqlx
- Allowlisting vs blocklisting approaches Defense in depth strategies
- 4. Resource-Based Data Patterns
- Learn when to use MCP resources instead of SQL tools:

Connection pooling configured Query timeout set Audit logging enabled Authentication required (OAuth in production)


### The Current Workflow

When an employee wants to use AI to analyze company data, here's what typically happens:

```text
                  The Data Access Gauntlet
1. REQUEST ACCESS
     → Submit IT ticket
         → Wait for approval (days/weeks)
             → Get credentials
2. LEARN THE TOOLS
     → Figure out which database has the data
         → Learn SQL or the reporting tool
             → Understand the schema
3. EXTRACT THE DATA
     → Write the query
         → Export to CSV
             → Maybe clean it up in Excel
4. USE WITH AI
     → Copy-paste into ChatGPT
         → Hope it's not too large
             → Repeat for every new question
```

(table/list, cells in reading order)
- This workflow has serious problems:
- Problem
- Impact
- Slow
- Days or weeks to get access, minutes per query
- Error-prone
- Manual copy-paste introduces mistakes
- Limited
- Stale
- Large datasets don't fit in chat contexts
- Exported data is immediately out of date
- Insecure
- Data copied to external AI services
- Problem
- Impact
- Inefficient
- Every question requires the full workflow
- The MCP Solution
- With a database MCP server, the workflow becomes:

```text
                  MCP Database Access
User: "What were our top products last quarter?"
Claude: [Calls list_tables to understand schema]
        [Calls query with appropriate SQL]
        "Here are your top 10 products by revenue..."
Time: ~2 seconds
```

(table/list, cells in reading order)
- The key differences:
- Aspect
- Before MCP
- With MCP
- Access time
- Days/weeks
- Instant (pre-authorized)
- Data freshness
- Stale exports
- Query complexity
- User writes SQL
- Live queries
- AI writes SQL
- Data size
- Security
- Limited by copy-paste
- Paginated, unlimited
- Data leaves enterprise
- Stays within boundary
- Aspect
- Before MCP
- With MCP
- Repeatability
- Manual each time
- Automatic
- Why This Matters for Enterprises
- 1. Democratized Data Access
- Not everyone knows SQL. With an MCP server, a salesperson can ask:

Claude translates this to SQL, queries the database, and presents the results—no SQL knowledge required.


### 2. Real-Time Insights

Traditional BI dashboards show pre-defined reports.

```text
             Enterprise Network
Database    ◄    ►   Database MCP Server
(Private)            - SELECT only
                     - Row limits
                     - Column filtering
                     - Audit logging
                     - OAuth authentication
                                   HTTPS + OAuth
```

```text
Claude / AI
(Authorized)
```


### Data never leaves your network as raw exports. The MCP server:

Enforces read-only access Limits result sizes Filters sensitive columns Logs all queries for audit Requires authentication

```text
       offering them our current promotion"
```


### Claude:

```text
  1. [Calls database server] → Gets inactive customer list
  2. [Calls promotions server] → Gets current offer details
  3. [Calls email server] → Drafts personalized emails
```

The database becomes one component in larger AI-powered workflows.


### Finance & Operations

"What's our current inventory status?" "Show me outstanding invoices over 60 days" "Compare expenses by department"


### HR & People

"What's our headcount by location?" "Show me open positions and time-to-fill" "Analyze training completion rates"


### Product & Analytics

"What features are most used?" "Show me user retention by cohort" "Compare performance across regions"

- Database access requires careful security design:

- What the MCP Server Should Enforce


### What the Database Should Enforce

1.


### What the Infrastructure Should Enforce

1.

(table/list, cells in reading order)
- 4. Backup - Regular database backups
- The Business Case
- Metric
- Traditional Approach
- With MCP
- Time to first insight
- Hours to days
- Seconds
- Queries per day (per user)
- SQL knowledge required
- 2-5
- Yes
- 20-50
- No
- Data freshness
- Security risk
- IT ticket volume
- Hours/days old
- Real-time
- High (data exports)
- Low (controlled access)
- High
- Low
- For a 1,000-person organization where 200 people regularly need data:


### After: 200 people × 30 queries/day × 5 sec/query = 8 hours/day saved

That's 92 hours per day returned to productive work.


### Getting Started

In the next section, we'll build a database MCP server from scratch.


### Building db-explorer

Let's build a database MCP server.

```bash
cargo pmcp add server db-explorer --template db-explorer
```


### Step 2: Get a Sample Database

We'll use the Chinook database—a sample music store with customers, invoices, artists, and tracks:

```bash
curl -L -o chinook.db https://github.com/lerocha/chinook-
```

(table/list, cells in reading order)
- database/raw/master/ChinookDatabase/DataSources/Chinook_Sqlite.sqlite
- Step 3: Run the Server
- DATABASE_URL=sqlite:./chinook.db cargo pmcp dev db-explorer
- You should see:

```bash
claude mcp add db-explorer -t http http://localhost:3000
```


### You gave Claude direct access to a database. It can:

1.

```bash
npx @modelcontextprotocol/inspector http://localhost:3000/mcp
```

(table/list, cells in reading order)
- This opens a web UI where you can:
- Action
- How
- Browse tools
- See  list_tables  and  query  with their schemas
- Call list_tables
- Click the tool, then "Execute" (no parameters needed)
- Run a query
- Enter  {"query": "SELECT * FROM artists LIMIT 5"}
- See raw JSON
- View the exact MCP protocol messages
- Try these queries in the inspector:
- {"query": "SELECT * FROM customers LIMIT 5"}

```yaml
structure:
```


### servers/db-explorer/

```text
Cargo.toml
src/
    main.rs           # Entry point, server setup
    database.rs       # Connection pool management
    tools/
        mod.rs        # Tool exports
        list_tables.rs # Schema introspection
        query.rs      # SQL execution
```


### The Database Connection

```rust
// src/database.rs
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::sync::Arc;

pub type DbPool = Arc<Pool<Sqlite>>;

pub async fn create_pool(database_url: &str) -> Result<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(Arc::new(pool))
}
```

max_connections(5)  - Limits concurrent database connections Pool is shared between all tool handlers


### The list_tables Tool

```rust
// src/tools/list_tables.rs (simplified)

#[derive(Debug, Serialize, JsonSchema)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
}

async fn list_tables_impl(pool: &DbPool) -> Result<Vec<TableInfo>> {
    // Get table names from SQLite's system catalog
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master
         WHERE type = 'table'
         AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool.as_ref())
    .await?;

    // For each table, get columns and row count
    let mut result = Vec::new();
    for (table_name,) in tables {
        let columns = get_columns(pool, &table_name).await?;
        let row_count = get_row_count(pool, &table_name).await?;

        result.push(TableInfo { name: table_name, columns, row_count });
    }

    Ok(result)
}
```


### This tool:

Queries SQLite's  sqlite_master  for table names Uses  PRAGMA table_info()  to get column details


### The query Tool

```rust
// src/tools/query.rs (simplified)

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryInput {
    /// SQL query to execute (SELECT only)
    pub query: String,

    /// Maximum rows to return
    #[serde(default = "default_limit")]
    pub limit: i32,
}

async fn query_impl(pool: &DbPool, input: QueryInput) -> Result<QueryOutput> {
    // Security: Only allow SELECT
    if !input.query.trim().to_uppercase().starts_with("SELECT") {
        return Err(anyhow!("Only SELECT queries are allowed"));
    }

    // Security: Block dangerous keywords
    let blocked = ["INSERT", "UPDATE", "DELETE", "DROP"];
    for keyword in blocked {
        if input.query.to_uppercase().contains(keyword) {
            return Err(anyhow!("{} is not allowed", keyword));
        }
    }

    // Add LIMIT if not present
    let limited_query = if !input.query.to_uppercase().contains("LIMIT") {
        format!("{} LIMIT {}", input.query, input.limit + 1)
    } else {
        input.query.clone()
    };

    // Execute and return results
    let rows = sqlx::query(&limited_query)
        .fetch_all(pool.as_ref())

        .await?;

    Ok(format_results(rows, input.limit))
}
```


### Security measures:

1.


### The Main Entry Point

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./chinook.db".to_string());

    // Create connection pool
    let pool = create_pool(&database_url).await?;

    // Build MCP server with both tools
    let server = ServerBuilder::new("db-explorer", "1.0.0")
        .capabilities(ServerCapabilities {
            tools: Some(ToolCapabilities::default()),
            ..Default::default()
        })
        .tool(ListTables::new(pool.clone()).into_tool())
        .tool(Query::new(pool.clone()).into_tool())
        .build()?;

    // Start HTTP server
    server_common::create_http_server(server)
        .serve("0.0.0.0:3000")
        .await
}
```


### Building from Scratch

Want to build it yourself instead of using the template?


### 1. Create Minimal Server

```bash
cargo pmcp add server my-db-server --template minimal
```

```toml
[dependencies]
pmcp = { path = "../../pmcp" }
server-common = { path = "../../server-common" }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

database.rs  - Connection pool tools/list_tables.rs  - Schema discovery

(table/list, cells in reading order)
- What We Built
- Component
- Purpose
- DbPool
- Shared, pooled database connections
- list_tables
- Schema discovery for Claude
- query
- Flexible SQL execution with safety checks
- Connection pooling
- Efficient resource usage
- Query validation
- Basic SQL injection protection
- Result limiting
- Memory safety
- Limitations of This Basic Server
- This server works, but has security limitations:
- Issue
- Risk
- Solution
- String-based validation
- Can be bypassed
- Proper parsing
- No parameterized queries
- SQL injection
- Use  .bind()
- No authentication
- Anyone can query
- Add OAuth
- No audit logging
- No accountability
- Log all queries
- No column filtering
- May expose PII
- Allowlist columns
- The next sections address these:

In production deployments, you should:

Require OAuth authentication for all MCP requests 2.


### permissions

3.


### SQL Safety and Injection Prevention

SQL injection is consistently in the OWASP Top 10 vulnerabilities. When you build a database MCP server, you're creating an interface between AI-generated queries and your production data.


### Understanding SQL Injection

SQL injection occurs when untrusted input is concatenated into SQL queries:

```rust
// DANGEROUS: SQL Injection vulnerability
let query = format!(
    "SELECT * FROM users WHERE name = '{}'",
    user_input  // What if user_input is: ' OR '1'='1
);
```

(table/list, cells in reading order)
- Attack Examples
- Attack
- Payload
- Result
- Data exfiltration
- Bypass authentication
- Delete data
- ' UNION SELECT password FROM users-
- -
- ' OR '1'='1
- '; DROP TABLE users;--
- Read files

(table/list, cells in reading order)
- Leaks passwords
- Returns all rows
- Destroys table
- System compromise
- Defense Layer 1: Parameterized Queries
- Always use parameterized queries for any user-controlled values:

```rust
// SAFE: Parameterized query
let users = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE name = ?"
)
.bind(&user_input)  // Value is escaped/handled by the driver
.fetch_all(&pool)
```


### .await?;

The database driver handles escaping—the user input can never become SQL code.


### When to Use Parameters

```rust
// [GOOD] SAFE: Values as parameters
sqlx::query("SELECT * FROM users WHERE id = ?")
    .bind(user_id)

sqlx::query("SELECT * FROM orders WHERE date > ? AND status = ?")
    .bind(start_date)
    .bind(status)

// [BAD] UNSAFE: String formatting
format!("SELECT * FROM users WHERE id = {}", user_id)
```

format!("SELECT * FROM {} WHERE id = ?", table_name)  // Table names can't be parameterized!

- You cannot parameterize table or column names:

```rust
// This WON'T work - table names can't be parameters
sqlx::query("SELECT * FROM ? WHERE id = ?")
    .bind(table_name)  // Error!
    .bind(id)
```


### Defense Layer 2: Allowlisting

When you can't use parameters (table names, column names, ORDER BY), use strict allowlists:

```rust
/// Tables that users are allowed to query
const ALLOWED_TABLES: &[&str] = &[
    "customers",
    "orders",
    "products",
    "invoices",
];

/// Validate a table name against the allowlist
fn validate_table(table: &str) -> Result<&str> {
    let table_lower = table.to_lowercase();

    ALLOWED_TABLES
        .iter()
        .find(|&&t| t == table_lower)
        .map(|&t| t)
        .ok_or_else(|| anyhow!("Table '{}' is not accessible", table))
}

// Usage
let table = validate_table(&input.table)?;
let query = format!("SELECT * FROM {} WHERE id = ?", table);
```


### Column Name Allowlisting

```rust
fn validate_order_column(table: &str, column: &str) -> Result<&'static str> {
    let allowed = match table {
        "customers" => &["id", "name", "email", "created_at"][..],
        "orders" => &["id", "customer_id", "total", "order_date"][..],
        "products" => &["id", "name", "price", "category"][..],
        _ => return Err(anyhow!("Unknown table")),
    };

    allowed
        .iter()
        .find(|&&c| c == column.to_lowercase())
        .copied()
        .ok_or_else(|| anyhow!("Cannot sort by '{}'", column))
}

// Usage in ORDER BY
let order_col = validate_order_column("customers", &input.sort_by)?;
let query = format!(
    "SELECT * FROM customers ORDER BY {} {}",
    order_col,
    if input.ascending { "ASC" } else { "DESC" }
);
```


### Defense Layer 3: Query Validation

For MCP servers that accept raw SQL (like our  query  tool), validate the query structure:

```rust
/// Validate that a query is safe to execute
fn validate_query(sql: &str) -> Result<()> {
    let sql_upper = sql.trim().to_uppercase();

    // Must start with SELECT
    if !sql_upper.starts_with("SELECT") {
        return Err(anyhow!("Only SELECT queries are allowed"));
    }

    // Block dangerous keywords
    let blocked = [
        "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER",
        "TRUNCATE", "EXEC", "EXECUTE", "GRANT", "REVOKE",
        "INTO OUTFILE", "INTO DUMPFILE", "LOAD_FILE",
    ];

    for keyword in blocked {
        if sql_upper.contains(keyword) {
            return Err(anyhow!("'{}' is not allowed in queries", keyword));
        }
    }

    // Block multiple statements
    if sql.contains(';') {
        let parts: Vec<_> = sql.split(';').filter(|s| !s.trim().is_empty()).collect();
        if parts.len() > 1 {
            return Err(anyhow!("Multiple statements are not allowed"));
        }
    }

    // Block comments (often used in injection attacks)
    if sql.contains("--") || sql.contains("/*") {
        return Err(anyhow!("SQL comments are not allowed"));
    }

    Ok(())
}
```

```text
// These attacks might bypass simple validation:

// Unicode tricks
```

```text
// Case variations
```


### "sElEcT * fRoM users" // Mixed case

```text
// Encoded characters
```


### "SELECT%20*%20FROM%20users" // URL encoded

```text
// Comments
```

- Never rely on query validation alone. Use it alongside:

1.

- The MCP server's database user should have minimal privileges:


### For SQLite, use a read-only connection:

```rust
let pool = SqlitePoolOptions::new()
    .connect("sqlite:./data.db?mode=ro")  // Read-only mode
    .await?;
```

```rust
use tokio::time::{timeout, Duration};

async fn execute_with_timeout(
    pool: &DbPool,
    query: &str,
    max_duration: Duration,
) -> Result<Vec<SqliteRow>> {
    timeout(max_duration, async {
        sqlx::query(query)
            .fetch_all(pool.as_ref())
            .await
    })
    .await
    .map_err(|_| anyhow!("Query timed out after {:?}", max_duration))?
    .map_err(|e| anyhow!("Query failed: {}", e))
}

// Usage
let rows = execute_with_timeout(
    &pool,
    &query,
    Duration::from_secs(30)
).await?;
```

- Always limit result sizes to prevent memory exhaustion:

```rust
const MAX_ROWS: i32 = 10_000;
const DEFAULT_ROWS: i32 = 100;

fn apply_limit(query: &str, requested_limit: Option<i32>) -> String {
    let limit = requested_limit
        .unwrap_or(DEFAULT_ROWS)
        .min(MAX_ROWS);

    let query_upper = query.to_uppercase();

    if query_upper.contains("LIMIT") {
        // Already has LIMIT - don't add another
        // But we should validate the existing limit isn't too high
        query.to_string()
    } else {
        format!("{} LIMIT {}", query.trim_end_matches(';'), limit)
    }
}
```

```rust
use tracing::{info, warn};

async fn execute_query(
    pool: &DbPool,
    query: &str,
    user_id: &str,
) -> Result<QueryOutput> {
    let start = std::time::Instant::now();

    // Log the query attempt
    info!(
        user_id = %user_id,
        query_preview = %query.chars().take(100).collect::<String>(),
        "Query execution started"
    );

    let result = sqlx::query(query)
        .fetch_all(pool.as_ref())
        .await;

    let duration = start.elapsed();

    match &result {
        Ok(rows) => {
            info!(
                user_id = %user_id,
                row_count = rows.len(),
                duration_ms = duration.as_millis(),
                "Query completed successfully"
            );
        }
        Err(e) => {
            warn!(
                user_id = %user_id,
                error = %e,
                duration_ms = duration.as_millis(),
                "Query failed"
            );
        }

    }

    // Convert result...
    Ok(result?)
}
```

```rust
use anyhow::{Result, anyhow};
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

const MAX_ROWS: i32 = 10_000;
const DEFAULT_ROWS: i32 = 100;
const QUERY_TIMEOUT: Duration = Duration::from_secs(30);

const BLOCKED_KEYWORDS: &[&str] = &[
    "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER",
    "TRUNCATE", "EXEC", "EXECUTE", "GRANT", "REVOKE",
    "INTO OUTFILE", "INTO DUMPFILE", "LOAD_FILE",
];

pub async fn secure_query(
    pool: &DbPool,
    input: QueryInput,
    user_context: &UserContext,
) -> Result<QueryOutput> {
    // Layer 3: Query validation
    validate_query(&input.query)?;

    // Layer 6: Apply row limit
    let limit = input.limit.unwrap_or(DEFAULT_ROWS).min(MAX_ROWS);
    let limited_query = apply_limit(&input.query, limit);

    // Layer 7: Audit logging
    info!(
        user_id = %user_context.user_id,
        query = %limited_query,
        "Executing query"
    );

    // Layer 5: Timeout
    let result = timeout(QUERY_TIMEOUT, async {
        sqlx::query(&limited_query)
            .fetch_all(pool.as_ref())
            .await
    })

    .await
    .map_err(|_| anyhow!("Query timed out"))?
    .map_err(|e| anyhow!("Query failed: {}", e))?;

    // Check truncation
    let truncated = result.len() > limit as usize;
    let rows: Vec<_> = result.into_iter().take(limit as usize).collect();

    info!(
        user_id = %user_context.user_id,
        row_count = rows.len(),
        truncated = truncated,
        "Query completed"
    );

    Ok(format_output(rows, truncated))
}

fn validate_query(sql: &str) -> Result<()> {
    let sql_upper = sql.trim().to_uppercase();
// ... truncated, 26 more lines in source book ...
```


### User Context and Token Pass-Through

The  user_context  parameter in the examples above is more than just a logging convenience—in

production, it represents the authenticated user and should flow through to your backend systems.

```rust
/// User context extracted from OAuth access token
pub struct UserContext {
    /// User ID from the identity provider
    pub user_id: String,

    /// The raw access token - pass this to backend systems
    pub access_token: String,

    /// User's roles/groups from token claims
    pub roles: Vec<String>,
}

impl UserContext {
    /// Extract from MCP request metadata (simplified)
    pub fn from_request(extra: &RequestExtra) -> Result<Self> {
        let token = extra.headers
            .get("authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| anyhow!("Missing authorization header"))?;

        // Validate token and extract claims
        let claims = validate_jwt(token)?;

        Ok(Self {
            user_id: claims.sub,
            access_token: token.to_string(),
            roles: claims.groups,
        })
    }
}
```


### Pass Tokens to Backend Systems

The MCP server should not be the source of truth for permissions.

```rust
pub async fn secure_query_with_passthrough(
    pool: &DbPool,
    input: QueryInput,
    user_context: &UserContext,
) -> Result<QueryOutput> {
    // For databases that support session context (PostgreSQL, Oracle):
    // Pass the user identity so row-level security policies apply
    sqlx::query("SELECT set_config('app.current_user', $1, true)")
        .bind(&user_context.user_id)
        .execute(pool.as_ref())
        .await?;

    // Now queries are filtered by database RLS policies
    let result = sqlx::query(&input.query)
        .fetch_all(pool.as_ref())
        .await?;

    // ...
}
```


### For external APIs, pass the token in the request:

```rust
pub async fn call_backend_api(
    client: &reqwest::Client,
    user_context: &UserContext,
    endpoint: &str,
) -> Result<serde_json::Value> {
    // Pass the user's token - let the backend validate permissions
    let response = client.get(endpoint)
        .header("Authorization", format!("Bearer {}", user_context.access_token))
        .send()
        .await?;

    // Backend enforces what this user can access
    Ok(response.json().await?)
}
```

Learn More: See Part 5: Security for complete OAuth integration patterns, including extracting tokens from MCP requests and configuring row-level security in PostgreSQL.

(table/list, cells in reading order)
- Security Checklist
- Before deploying your database MCP server:
- Layer
- Check
- Status
- Authentication
- OAuth required for all requests
- Token Pass-Through
- Access tokens passed to backend systems
- Parameterization
- All user values use  .bind()
- Allowlisting
- Table/column names validated against lists
- Query Validation
- Dangerous keywords blocked
- Permissions
- Database user has SELECT only
- Timeouts
- Limits
- Logging
- Queries timeout after reasonable duration
- Result size is bounded
- All queries are logged with user context
- Sensitive Data
- PII/secrets columns are filtered
- ☐
- ☐
- ☐
- ☐
- ☐
- ☐
- ☐
- ☐
- ☐
- ☐
- Common Mistakes to Avoid
- [BAD] Blocklisting Instead of Allowlisting

```text
// BAD: Trying to block known bad things
if !input.contains("DROP") && !input.contains("DELETE") {
    // Still vulnerable to: DrOp, DEL/**/ETE, etc.
}

// GOOD: Only allow known good things
if ALLOWED_TABLES.contains(&table) {
    // Secure - we control the list
}
```


### [BAD] Trusting Client-Side Validation

```rust
// BAD: Assuming the schema validation caught everything
// JsonSchema regex can be bypassed by determined attackers
#[schemars(regex(pattern = r"^SELECT"))]
```

query: String,  // Don't rely on this alone!

```rust
// GOOD: Always validate server-side
fn validate_query(sql: &str) -> Result<()> {
    // Server-side validation that can't be bypassed
}
```


### [BAD] Logging Sensitive Data

```text
// BAD: Logging full query might expose sensitive filters
```

info!("Query: {}", query);  // Might contain: WHERE ssn = '123-45-6789'

```text
// GOOD: Log query structure, not values
info!(
    query_type = "SELECT",
    tables = ?extract_tables(&query),
    "Query executed"
);
```


### Resource-Based Data Patterns

MCP offers two ways to expose data: tools and resources. Understanding when to use each is key to building intuitive database servers.

(table/list, cells in reading order)
- Tools vs Resources: When to Use Each
- Aspect
- Nature
- Data
- Tools
- Resources
- Actions, operations, queries
- Documentation, reference data, metadata
- Dynamic, user-specific
- Static or slowly-changing
- Parameters
- Flexible input parameters
- URI-based, limited parameters
- Use case
- Caching
- "Do something"
- "Read about something"
- Usually not cached
- Often cached
- Use Resources For:


### Resource approach:

```yaml
  Claude: "I need customer 12345"
  → Read db://customers/12345
  → Returns one customer
  → Claude: "Now I need their orders"
  → Read db://customers/12345/orders
  → Returns orders
  → Claude: "What's their total spend?"
  → ??? No resource for aggregations
```


### Tool approach:

```yaml
  Claude: "I need customer 12345 with their order history and total spend"
  → query("SELECT c.*, SUM(o.total) as total_spend
           FROM customers c
           JOIN orders o ON c.id = o.customer_id
           WHERE c.id = 12345
           GROUP BY c.id")
  → Returns everything in one call
```

Tools are more flexible for data access.

```rust
use pmcp::resource::{Resource, ResourceContent, ResourceInfo};

/// Database schema documentation as a resource
pub struct SchemaResource {
    pool: DbPool,
}

impl Resource for SchemaResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://schema", "Database Schema")
            .with_description(
                "Complete database schema with tables, columns, types, and
relationships. \
                 Use this to understand the database structure before writing
queries."
            )
            .with_mime_type("application/json")
    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let schema = self.build_schema_documentation().await?;
        Ok(ResourceContent::json(&schema)?)
    }
}

#[derive(Serialize)]
struct SchemaDocumentation {
    database_name: String,
    tables: Vec<TableDocumentation>,
    relationships: Vec<Relationship>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct TableDocumentation {
    name: String,
    description: String,
    columns: Vec<ColumnDocumentation>,
    primary_key: Vec<String>,

    row_count: i64,
    example_query: String,
}

#[derive(Serialize)]
struct ColumnDocumentation {
    name: String,
    data_type: String,
    nullable: bool,
    description: String,  // Can be populated from comments or a separate config
}

#[derive(Serialize)]
struct Relationship {
    from_table: String,
    from_column: String,
    to_table: String,
    to_column: String,
    relationship_type: String,  // "one-to-many", "many-to-many", etc.
}
// ... truncated, 48 more lines in source book ...
```

```rust
/// Individual table documentation
pub struct TableSchemaResource {
    pool: DbPool,
}

impl Resource for TableSchemaResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://schema/{table_name}", "Table Schema")
            .with_description(
                "Detailed schema for a specific table including columns, \
                 types, constraints, and example queries."
            )
            .with_mime_type("application/json")
    }

    async fn read(&self, uri: &str) -> Result<ResourceContent> {
        let table_name = uri.strip_prefix("db://schema/")
            .ok_or_else(|| anyhow!("Invalid URI"))?;

        // Validate table exists
        let valid_tables = self.get_table_names().await?;
        if !valid_tables.contains(&table_name.to_string()) {
            return Err(anyhow!("Table '{}' not found", table_name));
        }

        let doc = self.build_table_documentation(table_name).await?;
        Ok(ResourceContent::json(&doc)?)
    }
}

impl TableSchemaResource {
    async fn build_table_documentation(&self, table: &str) ->
Result<TableDocumentation> {
        let columns = self.get_columns(table).await?;
        let pk = self.get_primary_key(table).await?;
        let row_count = self.get_row_count(table).await?;

        Ok(TableDocumentation {
            name: table.to_string(),

            description: self.get_table_description(table),
            columns,
            primary_key: pk,
            row_count,
            example_query: format!(
                "SELECT * FROM {} LIMIT 10",
                table
            ),
        })
    }

    fn get_table_description(&self, table: &str) -> String {
        // In production, this might come from a config file or database comments
        match table {
            "customers" => "Customer information including contact details and
location",
            "invoices" => "Sales transactions with date, customer, and billing info",
            "invoice_items" => "Line items for each invoice, linking to tracks",
            "tracks" => "Music tracks with duration, genre, and pricing",
            "albums" => "Music albums with artist reference",
// ... truncated, 8 more lines in source book ...
```

```rust
/// Reference data: All available genres
pub struct GenresResource {
    pool: DbPool,
}

impl Resource for GenresResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://reference/genres", "Music Genres")
            .with_description(
                "List of all music genres in the database. \
                 Use these values when filtering tracks by genre."
            )
            .with_mime_type("application/json")
    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let genres: Vec<Genre> = sqlx::query_as(
            "SELECT GenreId, Name FROM genres ORDER BY Name"
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(ResourceContent::json(&genres)?)
    }
}

/// Reference data: All media types
pub struct MediaTypesResource {
    pool: DbPool,
}

impl Resource for MediaTypesResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://reference/media-types", "Media Types")
            .with_description(
                "Available media formats (MP3, AAC, etc.). \
                 Use when filtering or understanding track formats."
            )
            .with_mime_type("application/json")

    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let types: Vec<MediaType> = sqlx::query_as(
            "SELECT MediaTypeId, Name FROM media_types ORDER BY Name"
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(ResourceContent::json(&types)?)
    }
}
```

```rust
/// Example queries for common operations
pub struct QueryExamplesResource;

impl Resource for QueryExamplesResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://help/query-examples", "Query Examples")
            .with_description(
                "Example SQL queries for common operations. \
                 Reference these patterns when writing queries."
            )
            .with_mime_type("application/json")
    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let examples = vec![
            QueryExample {
                name: "Customer with orders",
                description: "Get a customer and their order history",
                sql: r#"
                    SELECT c.FirstName, c.LastName, c.Email,
                           i.InvoiceId, i.InvoiceDate, i.Total
                    FROM customers c
                    JOIN invoices i ON c.CustomerId = i.CustomerId
                    WHERE c.CustomerId = ?
                    ORDER BY i.InvoiceDate DESC
                "#.to_string(),
            },
            QueryExample {
                name: "Top selling tracks",
                description: "Tracks ordered by number of sales",
                sql: r#"
                    SELECT t.Name as Track, ar.Name as Artist,
                           COUNT(*) as TimesSold
                    FROM tracks t
                    JOIN invoice_items ii ON t.TrackId = ii.TrackId
                    JOIN albums al ON t.AlbumId = al.AlbumId
                    JOIN artists ar ON al.ArtistId = ar.ArtistId
                    GROUP BY t.TrackId
                    ORDER BY TimesSold DESC

                    LIMIT 10
                "#.to_string(),
            },
            QueryExample {
                name: "Revenue by country",
                description: "Total sales grouped by customer country",
                sql: r#"
                    SELECT c.Country,
                           COUNT(DISTINCT c.CustomerId) as Customers,
                           SUM(i.Total) as Revenue
                    FROM customers c
                    JOIN invoices i ON c.CustomerId = i.CustomerId
                    GROUP BY c.Country
                    ORDER BY Revenue DESC
                "#.to_string(),
            },
            QueryExample {
                name: "Genre popularity",
                description: "Number of tracks per genre",
                sql: r#"
// ... truncated, 19 more lines in source book ...
```


### Example 5: Loading Resources from Files

Not all documentation comes from developers.

```text
src/
    main.rs
docs/                          # Maintained by DBAs/analysts
    database-guide.md
    tables/
        customers.md
        invoices.md
        tracks.md
    query-patterns.md
Cargo.toml
```


### Example markdown file ( docs/tables/customers.md ):

```text
# Customers Table
```

The customers table stores contact information for all registered customers.


### ## Columns

| Column | Type | Description | |--------|------|-------------| | CustomerId | INTEGER | Primary key, auto-increment | | FirstName | TEXT | Customer's first name (required) | | LastName | TEXT | Customer's last name (required) | | Email | TEXT | Unique email address (required) | | Company | TEXT | C


### ## Common Queries

Find customers by country: ```sql SELECT * FROM customers WHERE Country = 'USA' ORDER BY LastName;


### Find customers with their total spend:

SELECT c.FirstName, c.LastName, SUM(i.Total) as TotalSpend FROM customers c JOIN invoices i ON c.CustomerId = i.CustomerId GROUP BY c.CustomerId ORDER BY TotalSpend DESC;

- Email must be unique across all customers

```rust
use std::path::{Path, PathBuf};
use tokio::fs;

/// Documentation loaded from markdown files
pub struct FileDocumentationResource {
    docs_dir: PathBuf,
}

impl FileDocumentationResource {
    pub fn new(docs_dir: impl AsRef<Path>) -> Self {
        Self {
            docs_dir: docs_dir.as_ref().to_path_buf(),
        }
    }
}

impl Resource for FileDocumentationResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://docs/tables/{table_name}", "Table Documentation")
            .with_description(
                "Human-written documentation for database tables. \
                 Includes column descriptions, business rules, and example queries. \
                 Maintained by DBAs and data analysts."
            )
            .with_mime_type("text/markdown")
    }

    async fn read(&self, uri: &str) -> Result<ResourceContent> {
        let table_name = uri.strip_prefix("db://docs/tables/")
            .ok_or_else(|| anyhow!("Invalid URI format"))?;

        // Prevent path traversal attacks
        if table_name.contains("..") || table_name.contains('/') {
            return Err(anyhow!("Invalid table name"));
        }

        let file_path = self.docs_dir
            .join("tables")
            .join(format!("{}.md", table_name));

        // Check file exists within docs directory
        let canonical = file_path.canonicalize()
            .map_err(|_| anyhow!("Documentation not found for table '{}'",
```


### table_name))?;

```rust
        if !canonical.starts_with(self.docs_dir.canonicalize()?) {
            return Err(anyhow!("Invalid path"));
        }

        let content = fs::read_to_string(&file_path).await
            .map_err(|_| anyhow!("Documentation not found for table '{}'",
```


### table_name))?;

```rust
        Ok(ResourceContent::text(content))
    }
}

/// Database guide - single file resource
pub struct DatabaseGuideResource {
    docs_dir: PathBuf,
}

impl Resource for DatabaseGuideResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://docs/guide", "Database Guide")
            .with_description(
                "Comprehensive database guide written by the DBA team. \
                 Includes naming conventions, relationships, and best practices."
            )
            .with_mime_type("text/markdown")
    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let file_path = self.docs_dir.join("database-guide.md");

        let content = fs::read_to_string(&file_path).await
            .map_err(|_| anyhow!("Database guide not found"))?;

        Ok(ResourceContent::text(content))
    }
}
```


### Listing available documentation files:

```rust
/// List all available table documentation
pub struct TableDocsListResource {
    docs_dir: PathBuf,
}

impl Resource for TableDocsListResource {
    fn info(&self) -> ResourceInfo {
        ResourceInfo::new("db://docs/tables", "Available Table Documentation")
            .with_description(
                "Lists all tables that have documentation available."
            )
            .with_mime_type("application/json")
    }

    async fn read(&self, _uri: &str) -> Result<ResourceContent> {
        let tables_dir = self.docs_dir.join("tables");
        let mut entries = fs::read_dir(&tables_dir).await?;

        let mut tables = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(stem) = path.file_stem() {
                    tables.push(stem.to_string_lossy().to_string());
                }
            }
        }

        tables.sort();

        Ok(ResourceContent::json(&serde_json::json!({
            "tables": tables,
            "note": "Use db://docs/tables/{name} to read specific documentation"
        }))?)
    }
}
```

Why file-based resources?

(table/list, cells in reading order)
- Approach
- Best For
- Rust code (hardcoded)
- Static strings, compile-time constants
- Database queries
- Dynamic data, schema introspection
- File system
- Human-maintained docs, external contributions
- Benefits of file-based documentation:


### Hot reloading pattern:

For development, you might want to reload documentation without restarting:

```rust
impl Resource for FileDocumentationResource {
    fn cache_hint(&self) -> Option<Duration> {
        // In development: no caching, always fresh
        #[cfg(debug_assertions)]
        return None;

        // In production: cache for 5 minutes
        #[cfg(not(debug_assertions))]
        return Some(Duration::from_secs(300));
    }
}
```

```rust
let docs_dir = PathBuf::from("./docs");

let server = ServerBuilder::new("db-explorer", "1.0.0")
    .capabilities(ServerCapabilities {
        tools: Some(ToolCapabilities::default()),
        resources: Some(ResourceCapabilities::default()),
        ..Default::default()
    })
    // Tools for dynamic queries
    .tool(ListTables::new(pool.clone()).into_tool())
    .tool(Query::new(pool.clone()).into_tool())
    // Resources from database introspection
    .resource(SchemaResource::new(pool.clone()))
    .resource(TableSchemaResource::new(pool.clone()))
    .resource(GenresResource::new(pool.clone()))
    .resource(MediaTypesResource::new(pool.clone()))
    // Resources from code
    .resource(QueryExamplesResource)
    // Resources from filesystem (maintained by DBAs)
    .resource(DatabaseGuideResource::new(docs_dir.clone()))
    .resource(TableDocsListResource::new(docs_dir.clone()))
    .resource(FileDocumentationResource::new(docs_dir))
    .build()?;
```

```text
  - This is asking about reference data
  - I can read db://reference/genres
  - No need to write a query
```


### Claude: [Reads db://reference/genres]

The database contains 25 music genres: Alternative, Blues, Classical, Comedy, Country...

```text
  - I need to write a query
  - Let me check db://schema for table structure
  - And db://help/query-examples for patterns
```


### Claude: [Reads db://schema]

```json
        [Reads db://help/query-examples]
        [Uses query tool with adapted SQL]
```

(table/list, cells in reading order)
- Benefits of This Pattern
- 1. Better AI Understanding
- Resources give Claude context without requiring queries:
- Without resources:

```text
  Claude must guess table/column names or call list_tables first
```


### With resources:

```text
  Claude reads schema once, understands the entire database
```

```text
  1. list_tables() - What tables exist?
  2. query("PRAGMA table_info(customers)") - What columns?
  3. query("PRAGMA foreign_key_list(customers)") - Relationships?
  4. query("SELECT...") - Finally, the actual query
```


### With resources:

```text
  1. Read db://schema - Understand everything
  2. query("SELECT...") - Execute the query
```

```rust
impl Resource for SchemaResource {
    fn cache_hint(&self) -> Option<Duration> {
        Some(Duration::from_secs(300))  // Cache for 5 minutes
    }
}
```

(table/list, cells in reading order)
- 4. Clear Separation of Concerns
- Resource
- Purpose
- db://schema
- Understand the database
- db://reference/*
- Lookup valid values
- db://help/*
- Learn query patterns
- Tool
- Purpose
- query
- Execute any SELECT
- list_tables
- Quick table overview
- Summary
- When to Use Each Approach
- Data Type
- Approach
- Example
- Table structures
- Resource
- db://schema
- Column descriptions
- Resource
- db://schema/customers
- Lookup tables (genres, countries)
- Resource
- db://reference/genres
- Data Type
- Approach
- Example
- Foreign key relationships
- Resource
- Part of  db://schema
- Query patterns/examples
- Human-written docs
- Resource
- Resource
- db://help/query-examples

```text
query("SELECT...")

query("SELECT SUM...")

query("SELECT...WHERE...")
```

(table/list, cells in reading order)
- Three Ways to Populate Resources
- Source
- Best For
- Example
- Database queries
- Rust code
- Filesystem
- Dynamic schema, reference tables
- db://schema , db://reference/genres
- Static content, computed examples
- Human-maintained docs, external tools
- db://help/query-examples
- db://docs/tables/{name}

Resources = Documentation.


### Handling Large Results

Enterprise databases contain millions of rows. When Claude asks "Show me all customers," you can't return everything at once.

(table/list, cells in reading order)
- The Problem with Large Results
- Returning too much data causes multiple problems:
- Problem
- Impact
- Memory exhaustion
- Server crashes with OOM
- Slow responses
- Users wait forever
- Context overflow
- AI can't process millions of rows
- Network costs
- Unnecessary data transfer
- Poor UX
- Information overload
- Pagination Strategies
- Strategy 1: Offset Pagination (Simple but Limited)


### Implementation:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OffsetPaginatedInput {
    pub query: String,

    #[serde(default = "default_page")]
    pub page: i32,

    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 { 0 }
fn default_page_size() -> i32 { 50 }

#[derive(Debug, Serialize, JsonSchema)]
pub struct OffsetPaginatedOutput {
    pub rows: Vec<Vec<serde_json::Value>>,
    pub columns: Vec<String>,
    pub page: i32,
    pub page_size: i32,
    pub has_more: bool,
}

async fn paginated_query(pool: &DbPool, input: OffsetPaginatedInput) ->
Result<OffsetPaginatedOutput> {
    let page_size = input.page_size.min(100);  // Cap at 100
    let offset = input.page * page_size;

    // Fetch one extra to detect if there are more
    let query = format!(
        "{} LIMIT {} OFFSET {}",
        input.query.trim_end_matches(';'),
        page_size + 1,
        offset
    );

    let rows = execute_query(pool, &query).await?;
    let has_more = rows.len() > page_size as usize;
    let rows: Vec<_> = rows.into_iter().take(page_size as usize).collect();

    Ok(OffsetPaginatedOutput {
        rows,
        columns: vec![],  // Extract from first row
        page: input.page,
        page_size,
        has_more,
    })
}
```


### Problems with offset pagination:

Page 1:     OFFSET 0    → Scans 100 rows      [ok] Fast Page 100:   OFFSET 9900 → Scans 10,000 rows   [warn] Slow Page 10000: OFFSET 999900 → Scans 1M rows    [no] Very slow

The database must skip all offset rows before returning results.

- Strategy 2: Cursor Pagination (Recommended)


### Implementation:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Opaque cursor containing pagination state
#[derive(Debug, Serialize, Deserialize)]
struct Cursor {
    /// The last seen ID
    last_id: i64,
    /// Table name (for validation)
    table: String,
    /// Sort column
    sort_column: String,
    /// Sort direction
    ascending: bool,
}

impl Cursor {
    /// Encode cursor to opaque string
    fn encode(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        BASE64.encode(json.as_bytes())
    }

    /// Decode cursor from opaque string
    fn decode(encoded: &str) -> Result<Self> {
        let bytes = BASE64.decode(encoded)
            .map_err(|_| anyhow!("Invalid cursor"))?;
        let json = String::from_utf8(bytes)
            .map_err(|_| anyhow!("Invalid cursor encoding"))?;
        serde_json::from_str(&json)
            .map_err(|_| anyhow!("Invalid cursor format"))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CursorPaginatedInput {
    /// Table to query
    pub table: String,

    /// Number of results per page (max 100)

    #[serde(default = "default_page_size")]
    pub page_size: i32,

    /// Cursor from previous response (omit for first page)
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CursorPaginatedOutput {
    pub rows: Vec<serde_json::Value>,
    pub columns: Vec<String>,
    pub count: usize,

    /// Cursor to fetch next page (null if no more data)
    pub next_cursor: Option<String>,

    /// Human-readable pagination status
    pub status: String,
}

// ... truncated, 85 more lines in source book ...
```

Why Include Table in Cursor?


### The cursor includes the table name for security:

```rust
// Attacker tries to use a customers cursor on the users table
cursor = { last_id: 12345, table: "customers", ... }
```


### input.table = "users"  // Trying to access different table

```text
// Validation catches this:
if cursor.table != input.table {
    return Err(anyhow!("Cursor is for different table"));
}
```


### Without this check, an attacker could:

1.

```rust
use futures::StreamExt;

async fn stream_query(
    pool: &DbPool,
    query: &str,
) -> impl futures::Stream<Item = Result<serde_json::Value>> {
    sqlx::query(query)
        .fetch(pool.as_ref())
        .map(|row_result| {
            row_result
                .map(|row| row_to_json(&row))
                .map_err(|e| anyhow!("Row error: {}", e))
        })
}

// Usage for large exports
async fn export_table(pool: &DbPool, table: &str, output: &mut File) -> Result<()> {
    let query = format!("SELECT * FROM {}", table);
    let mut stream = stream_query(pool, &query);

    while let Some(row_result) = stream.next().await {
        let row = row_result?;
        writeln!(output, "{}", serde_json::to_string(&row)?)?;
    }

    Ok(())
}
```

(table/list, cells in reading order)
- File exports Background processing
- Chunked responses (if your transport supports it)
- Memory-Safe Patterns
- Pattern 1: Always Limit

```rust
const MAX_ROWS: i32 = 10_000;
const DEFAULT_ROWS: i32 = 100;

fn safe_limit(requested: Option<i32>) -> i32 {
    requested
        .unwrap_or(DEFAULT_ROWS)
        .min(MAX_ROWS)
        .max(1)  // At least 1
}
```


### Pattern 2: Early Termination

```rust
async fn fetch_limited(pool: &DbPool, query: &str, max: usize) -> Result<Vec<Row>> {
    let mut rows = Vec::with_capacity(max);
    let mut stream = sqlx::query(query).fetch(pool.as_ref());

    while let Some(row) = stream.next().await {
        rows.push(row?);
        if rows.len() >= max {
            break;  // Stop fetching, even if more exist
        }
    }

    Ok(rows)
}
```


### Pattern 3: Result Size Estimation

```rust
async fn check_result_size(pool: &DbPool, query: &str) -> Result<i64> {
    // Wrap query in COUNT to check size first
    let count_query = format!(
        "SELECT COUNT(*) FROM ({}) as subquery",
        query.trim_end_matches(';')
    );

    let count: (i64,) = sqlx::query_as(&count_query)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(count.0)
}

async fn safe_query(pool: &DbPool, query: &str, limit: i32) -> Result<QueryOutput> {
    let estimated_size = check_result_size(pool, query).await?;

    if estimated_size > 100_000 {
        return Err(anyhow!(
            "Query would return {} rows. Please add filters or use pagination.",
            estimated_size
        ));
    }

    // Proceed with actual query
    execute_query(pool, query, limit).await
}
```

```rust
fn pagination_message(count: usize, total: Option<i64>, has_more: bool) -> String {
    match (total, has_more) {
        (Some(t), true) => format!(
            "Showing {} of {} total results. Use the next_cursor to fetch more.",
            count, t
        ),
        (Some(t), false) => format!(
            "Showing all {} results.",
            t
        ),
        (None, true) => format!(
            "Showing {} results. More are available - use next_cursor to continue.",
            count
        ),
        (None, false) => format!(
            "Showing {} results. This is the complete result set.",
            count
        ),
    }
}
```


### Claude can then naturally say:

"I found 50 customers matching your criteria.

(table/list, cells in reading order)
- Performance Comparison
- Approach
- Page 1
- Page 100
- Page 10,000
- No pagination
- [no] OOM
- [no] OOM
- [no] OOM
- Approach
- Page 1
- Page 100
- Page 10,000
- OFFSET
- Cursor
- 10ms
- 10ms
- 100ms
- 10ms
- 5000ms
- 10ms

(table/list, cells in reading order)
- When to Use Each Strategy
- Scenario
- Recommended Strategy
- Simple UI pagination
- Offset (if depth < 100 pages)
- API pagination
- Search results
- Infinite scroll
- Cursor
- Cursor
- Cursor
- Admin data export
- Streaming
- Real-time feeds
- Cursor + polling
- Complete Pagination Implementation

```rust
/// Paginated query tool with cursor-based pagination
pub struct PaginatedQuery {
    pool: DbPool,
}

impl PaginatedQuery {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn into_tool(self) -> TypedTool<CursorPaginatedInput, CursorPaginatedOutput> {
        let pool = self.pool.clone();

        TypedTool::new(
            "paginated_query",
            "Query a table with cursor-based pagination. Returns a cursor for fetching
additional pages.",
            move |input: CursorPaginatedInput| {
                let pool = pool.clone();
                Box::pin(async move {
                    cursor_paginated_query(&pool, input).await
                })
            },
        )
    }
}
```

(table/list, cells in reading order)
- Summary
- Problem
- Solution
- Too many rows
- Always use LIMIT
- Problem
- Solution
- Deep pagination slow
- Use cursor pagination
- Memory exhaustion
- Stream or chunk
- AI can't process all data
- Provide clear pagination status
- Cursor tampering
- Include table in cursor, validate
