---
chapter: 5
title: "Input Validation and Output Schemas"
part: "Part II: Thoughtful Design"
use_when: "Writing schemas, enums, validation, output schemas, tool annotations, AI-friendly error messages."
suggested_prerequisites: [ch02, ch04]
source_kb: 77
extracted_kb: 63
code_blocks: 88
quality: auto-extracted
---

# Ch 5: Input Validation and Output Schemas

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Input Validation and Output Schemas

When an AI client calls your tool, it constructs the parameters based on your schema and description.

```text
// What the AI sends
{
    "tool": "sales_query",
    "parameters": {
        "query": "SELECT * FROM orders",
        "date_range": "last month"  // Natural language, not ISO dates
    }
}
```


### Without proper validation, this might:

Crash with a parse error Silently ignore the date_range Return all orders (no filtering)


### With proper validation, the AI gets useful feedback:

```json
{
    "error": {
        "code": "INVALID_DATE_RANGE",
        "message": "date_range must be an object with 'start' and 'end' ISO 8601 dates",
        "expected": {
            "start": "2024-11-01",
            "end": "2024-11-30"
        },
        "received": "last month"
    }
}
```

The AI can now self-correct and retry with the proper format.


### Why Schemas Matter

MCP tools declare their parameters using JSON Schema.

```rust
Tool::new("sales_query")
    .input_schema(json!({
        "type": "object",
        "required": ["query"],
        "properties": {
            "query": {
                "type": "string",
                "description": "SQL SELECT query"
            },
            "date_range": {
                "type": "object",
                "description": "Filter results to this date range",
                "properties": {
                    "start": {
                        "type": "string",
                        "format": "date",
                        "description": "Start date (ISO 8601)"
                    },
                    "end": {
                        "type": "string",
                        "format": "date",
                        "description": "End date (ISO 8601)"
                    }
                }
            },
            "limit": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10000,
                "default": 100
            }
        }
    }))
```


### 2. Pre-Call Validation

Many MCP clients validate parameters against the schema before sending the request.

- Your server should also validate, because:

Not all clients validate Schemas can't express all constraints Defense in depth is good practice

(table/list, cells in reading order)
- The Validation Spectrum
- Different levels of validation serve different purposes:
- Level
- What It Catches
- Example
- Schema
- Type mismatches
- String instead of number
- Format
- Structural errors
- Invalid date format
- Business
- Domain violations
- Future dates for historical query
- Security
- Dangerous inputs
- SQL injection attempts
- The PMCP SDK Approach: TypedTool

```rust
use pmcp::{TypedTool, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Input parameters for sales queries
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SalesQueryInput {
    /// SQL SELECT query to execute against PostgreSQL 15.
    /// Supports CTEs, WINDOW functions, and JSON operators.
    query: String,

    /// Optional date range filter for the query
    date_range: Option<DateRange>,

    /// Maximum rows to return (1-10000, default: 100)
    #[serde(default = "default_limit")]
    limit: u32,

    /// Query timeout in milliseconds (100-30000, default: 5000)
    #[serde(default = "default_timeout")]
    timeout_ms: u32,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DateRange {
    /// Start date in ISO 8601 format (YYYY-MM-DD)
    start: NaiveDate,
    /// End date in ISO 8601 format (YYYY-MM-DD)
    end: NaiveDate,
}

fn default_limit() -> u32 { 100 }
fn default_timeout() -> u32 { 5000 }
```

```json
{
  "properties": {
    "query": {
      "type": "string",
      "description": "SQL SELECT query to execute against PostgreSQL 15. Supports CTEs, WINDOW functions, and JSON
operators."
    },
    "date_range": {
      "type": "object",
      "description": "Optional date range filter for the query",
      "properties": {
        "start": { "type": "string", "format": "date", "description": "Start date in ISO 8601 format (YYYY-MM-DD)" },
        "end": { "type": "string", "format": "date", "description": "End date in ISO 8601 format (YYYY-MM-DD)" }
      }
    },
    "limit": {
      "type": "integer",
      "description": "Maximum rows to return (1-10000, default: 100)"
    }
  },
  "required": ["query"]
}
```


### Type-Safe Validation in the Handler

With  TypedTool , schema validation happens automatically.

```rust
let sales_query_tool = TypedTool::new(
    "sales_query",
    |args: SalesQueryInput, _extra| {
        Box::pin(async move {
            // 1. Schema validation: ALREADY DONE by TypedTool!
            //    - args.query is guaranteed to be a String
            //    - args.date_range, if present, has valid NaiveDate fields
            //    - Invalid JSON is rejected before this code runs

            // 2. Format validation: Partially handled by types
            //    - NaiveDate parsing validates ISO 8601 format
            //    - Add additional format checks as needed
            if args.query.trim().is_empty() {
                return Err(Error::Validation(
                    "Query cannot be empty".to_string()
                ));
            }

            // 3. Business validation
            if let Some(ref dr) = args.date_range {
                if dr.end < dr.start {
                    return Err(Error::Validation(
                        "date_range.end must be after date_range.start".to_string()
                    ));
                }
                if dr.end > chrono::Utc::now().date_naive() {
                    return Err(Error::Validation(
                        "Cannot query future dates".to_string()
                    ));
                }
            }

            // Enforce bounds even if client ignores schema hints
            let limit = args.limit.min(10000).max(1);
            let timeout = args.timeout_ms.min(30000).max(100);

            // 4. Security validation
            validate_sql_security(&args.query)?;

            // Execute with validated parameters
            execute_query(&args.query, args.date_range, limit, timeout).await
        })
    }
)
.with_description(
    "Execute read-only SQL queries against the sales PostgreSQL 15 database. \
    Returns results as JSON array. Use PostgreSQL date functions like DATE_TRUNC."
);

fn validate_sql_security(sql: &str) -> Result<(), Error> {
    let sql_upper = sql.to_uppercase();

    // Only allow SELECT queries
    if !sql_upper.trim_start().starts_with("SELECT") {
        return Err(Error::Validation(
            "Only SELECT queries are allowed".to_string()
        ));
    }

    // Block dangerous constructs
// ... truncated, 11 more lines in source book ...
```


### Using Enums for Constrained Values

When parameters have a fixed set of valid values, use Rust enums instead of validating strings:

```rust
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Csv,
    Markdown,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SalesRegion {
    NorthAmerica,
    Europe,
    AsiaPacific,
    LatinAmerica,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SalesReportInput {
    /// Sales region to report on
    region: SalesRegion,

    /// Output format for the report
    #[serde(default)]
    format: OutputFormat,

    /// Include year-over-year comparison
    #[serde(default)]
    include_yoy: bool,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}
```


### The generated schema includes the valid enum values:

```json
{
  "properties": {
    "region": {
      "type": "string",
      "enum": ["north_america", "europe", "asia_pacific", "latin_america"],
      "description": "Sales region to report on"
    },
    "format": {
      "type": "string",
      "enum": ["json", "csv", "markdown"],
      "description": "Output format for the report"
    }
  }
}
```

The AI knows exactly which values are valid and won't try "JSON", "Json", or "application/json".

(table/list, cells in reading order)
- Why TypedTool is Better
- Manual JSON Schema
- TypedTool with Structs
- Schema and code can drift apart
- Schema generated from code—always in sync
- Validation logic duplicated
- Type system enforces validation
- Easy to miss edge cases
- Compiler catches type mismatches
- String comparisons everywhere
- Pattern matching on enums
- Runtime type errors
- Compile-time type safety
- Verbose error handling
- Automatic deserialization errors

```rust
// [BAD] Manual approach: error-prone, verbose
let format = params.get("format")
    .and_then(|v| v.as_str())
    .ok_or(ValidationError::missing_field("format"))?;
if !["json", "csv", "markdown"].contains(&format) {
    return Err(ValidationError::invalid_value("format", "..."));
}

// [GOOD] TypedTool approach: type-safe, concise
// format is already OutputFormat enum—invalid values rejected automatically
match args.format {
    OutputFormat::Json => generate_json_report(&data),
    OutputFormat::Csv => generate_csv_report(&data),
    OutputFormat::Markdown => generate_markdown_report(&data),
}
```

- Error messages should help the AI self-correct. Include:

```rust
// Poor error message
Err(Error::new("Invalid input"))

// Good error message
Err(ValidationError {
    code: "INVALID_DATE_FORMAT",
    field: "date_range.start",
    message: "Date must be in ISO 8601 format (YYYY-MM-DD)",
    expected: "2024-11-01",
    received: "November 1st, 2024",
    suggestion: "Convert 'November 1st, 2024' to '2024-11-01'"
})
```

When an AI client makes a mistake, your validation should teach it the right way.


### Schema-Driven Validation

JSON Schema is your first line of defense—and your first opportunity to communicate with AI clients.


### Schema as Documentation

When an AI client encounters your tool, it reads the schema to understand what parameters are valid.

```rust
purposes:

Tool::new("sales_query")
    .description("Execute read-only SQL queries against the sales database")
    .input_schema(json!({
        "type": "object",
        "required": ["query"],
        "properties": {
            "query": {
                "type": "string",
                "description": "SQL SELECT statement to execute",
                "minLength": 1,
                "maxLength": 10000
            },
            "limit": {
                "type": "integer",
                "description": "Maximum rows to return (default: 100, max: 10000)",
                "minimum": 1,
                "maximum": 10000,
                "default": 100
            },
            "timeout_ms": {
                "type": "integer",
                "description": "Query timeout in milliseconds",
                "minimum": 100,
                "maximum": 30000,
                "default": 5000
            }
        }
    }))
```


### This schema tells the AI:

query  is required, must be a non-empty string limit  is optional with sensible bounds timeout_ms  has reasonable defaults

(table/list, cells in reading order)
- Essential Schema Patterns
- Required vs Optional Fields
- Use  required  to distinguish mandatory from optional parameters:
- json!({

```text
    "type": "object",
    "required": ["customer_id"],  // Must provide customer_id
    "properties": {
        "customer_id": {
            "type": "string",
            "description": "Unique customer identifier (required)"
        },
        "include_history": {
            "type": "boolean",
            "description": "Include order history (optional, default: false)",
            "default": false
        }
    }
})
```

```text
    "type": "object",
    "properties": {
        "region": {
            "type": "string",
            "enum": ["north", "south", "east", "west"],
            "description": "Sales region to query"
        },
        "format": {
            "type": "string",
            "enum": ["json", "csv", "markdown"],
            "description": "Output format for results",
            "default": "json"
        }
    }
})
```

```text
    "type": "object",
    "required": ["date_range"],
    "properties": {
        "date_range": {
            "type": "object",
            "description": "Date range for the query",
            "required": ["start", "end"],
            "properties": {
                "start": {
                    "type": "string",
                    "format": "date",
                    "description": "Start date (ISO 8601: YYYY-MM-DD)"
                },
                "end": {
                    "type": "string",
                    "format": "date",
                    "description": "End date (ISO 8601: YYYY-MM-DD)"
                }
            }
        }
    }
})
```

```text
    "type": "object",
    "properties": {
        "product_ids": {
            "type": "array",
            "description": "List of product IDs to query",
            "items": {
                "type": "string",
                "pattern": "^PRD-[0-9]{6}$"
            },
            "minItems": 1,
            "maxItems": 100
        },
        "metrics": {
            "type": "array",
            "description": "Metrics to include in report",
            "items": {
                "type": "string",
                "enum": ["revenue", "units", "margin", "growth"]
            },
            "uniqueItems": true
        }
    }
})
```


### Format Specifications

JSON Schema supports format hints that help AI clients construct correct values:

(table/list, cells in reading order)
- Format
- Description
- Example
- date
- ISO 8601 date
- 2024-11-15
- date-time
- ISO 8601 datetime
- 2024-11-15T14:30:00Z
- Format
- Description
- Example
- time
- email
- uri
- uuid
- ISO 8601 time
- Email address
- URI/URL
- UUID
- 14:30:00
- user@example.com
- https://example.com/path
- 550e8400-e29b-41d4-a716-446655440000
- json!({

```text
    "type": "object",
    "properties": {
        "email": {
            "type": "string",
            "format": "email",
            "description": "Customer email address"
        },
        "created_after": {
            "type": "string",
            "format": "date-time",
            "description": "Filter to records created after this timestamp"
        },
        "callback_url": {
            "type": "string",
            "format": "uri",
            "description": "Webhook URL for async notifications"
        }
    }
})
```

```text
    "type": "object",
    "properties": {
        "order_id": {
            "type": "string",
            "pattern": "^ORD-[0-9]{4}-[A-Z]{2}-[0-9]{6}$",
            "description": "Order ID (format: ORD-YYYY-RR-NNNNNN, e.g., ORD-2024-NA-000123)"
        },
        "phone": {
            "type": "string",
            "pattern": "^\\+[1-9]\\d{1,14}$",
            "description": "Phone number in E.164 format (e.g., +14155551234)"
        }
    }
})
```

Important: Include an example in the description.


### Implementing Validation in Rust

Schema validation happens at two levels: the MCP client may validate before sending, and your server should validate on receipt.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SalesQueryParams {
    pub query: String,

    #[serde(default = "default_limit")]
    pub limit: u32,

    #[serde(default = "default_timeout")]
    pub timeout_ms: u32,
}

fn default_limit() -> u32 { 100 }
fn default_timeout() -> u32 { 5000 }

pub async fn handle_sales_query(params: Value) -> Result<Value> {
    // Parse with serde - handles type validation
    let params: SalesQueryParams = serde_json::from_value(params)
        .map_err(|e| ValidationError::parse_error(e))?;

    // Additional validation beyond what schema can express
    if params.limit > 10000 {
        return Err(ValidationError::invalid_value(
            "limit",
            "Maximum limit is 10000",
            params.limit.to_string()
        ));
    }

    // Proceed with validated params
    execute_query(&params).await
}
```

```rust
pub struct ValidationError {
    pub code: String,
    pub field: String,
    pub message: String,
    pub expected: Option<String>,
    pub received: Option<String>,
}

impl ValidationError {
    pub fn missing_field(field: &str) -> Self {
        Self {
            code: "MISSING_REQUIRED_FIELD".into(),
            field: field.into(),
            message: format!("Required field '{}' is missing", field),
            expected: Some(format!("A value for '{}'", field)),
            received: None,
        }
    }

    pub fn invalid_type(field: &str, expected: &str, received: &str) -> Self {
        Self {
            code: "INVALID_TYPE".into(),
            field: field.into(),
            message: format!("Field '{}' has wrong type", field),
            expected: Some(expected.into()),
            received: Some(received.into()),
        }
    }

    pub fn invalid_value(field: &str, message: &str, received: String) -> Self {
        Self {
            code: "INVALID_VALUE".into(),
            field: field.into(),
            message: message.into(),
            expected: None,
            received: Some(received),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "error": {
                "code": self.code,
                "field": self.field,
                "message": self.message,
                "expected": self.expected,
                "received": self.received
            }
        })
    }
}
```


### Comprehensive Validation Function

```rust
pub fn validate_sales_query(params: &Value) -> Result<SalesQueryParams, ValidationError> {
    // 1. Check required fields
    let query = params.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ValidationError::missing_field("query"))?;

    if query.is_empty() {
        return Err(ValidationError::invalid_value(
            "query",
            "Query cannot be empty",
            "".into()
        ));
    }

    // 2. Validate optional fields with defaults
    let limit = match params.get("limit") {
        Some(Value::Number(n)) => {
            n.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| ValidationError::invalid_type(
                    "limit",
                    "positive integer",
                    &n.to_string()
                ))?
        }
        Some(v) => {
            return Err(ValidationError::invalid_type(
                "limit",
                "integer",
                &format!("{:?}", v)
            ));
        }
        None => 100,  // default
    };

    if limit > 10000 {

        return Err(ValidationError::invalid_value(
            "limit",
            "Maximum allowed value is 10000",
            limit.to_string()
        ));
    }

    // 3. Return validated struct
    Ok(SalesQueryParams {
        query: query.to_string(),
        limit,
        timeout_ms: extract_timeout(params)?,
    })
}
```

- Don't: Silent Coercion

```rust
// BAD: Silently converts or ignores invalid values
let limit = params.get("limit")
    .and_then(|v| v.as_u64())
    .unwrap_or(100);  // AI never learns its mistake
```


### Do: Explicit Errors

```rust
// GOOD: Tell the AI what went wrong
let limit = match params.get("limit") {
    Some(Value::Number(n)) if n.as_u64().is_some() => {
        n.as_u64().unwrap() as u32
    }
    Some(v) => {
        return Err(ValidationError::invalid_type(
            "limit",
            "positive integer",
            &format!("{}", v)
        ));
    }
    None => 100,
};
```


### Don't: Vague Error Messages

```rust
// BAD: AI can't learn from this
Err(Error::new("Invalid input"))
```


### Do: Specific, Actionable Errors

```yaml
// GOOD: AI knows exactly what to fix
Err(ValidationError {
    code: "INVALID_DATE_FORMAT".into(),
    field: "date_range.start".into(),
    message: "Date must be in ISO 8601 format".into(),
    expected: Some("2024-11-15".into()),
    received: Some("November 15, 2024".into()),
})
```


### How AI Clients Use Error Messages

When your tool returns an error, the AI client sees it as part of the tool's output.

```yaml
correction:
```

```text
AI Client Reasoning After Error
 Tool call failed with:
 {
   "error": {
     "code": "INVALID_DATE_FORMAT",
     "field": "date_range.start",
     "expected": "2024-11-15",
     "received": "November 15, 2024"
   }
 }
 AI reasoning:
 - The date format was wrong
 - I sent "November 15, 2024"
 - It expects "2024-11-15" (ISO 8601)
 - I'll retry with the correct format
```


### Retry with Corrected Parameters

Clear error messages enable the AI to immediately retry with fixed values:


### Attempt 1: sales_query(date_range: {start: "November 15, 2024", ...})

```text
           → Error: INVALID_DATE_FORMAT
```


### Attempt 2: sales_query(date_range: {start: "2024-11-15", ...})

```text
           → Success!
```

The AI learned from the error and self-corrected without user intervention.


### Try a Different Approach

Sometimes an error indicates the AI should try a completely different strategy:


### Attempt 1: customer_lookup(email: "john@...")

```text
           → Error: CUSTOMER_NOT_FOUND
```

AI reasoning: - Customer doesn't exist with this email - Maybe I should search by name instead - Or ask the user for more information


### Attempt 2: customer_search(name: "John Smith")

```text
           → Success: Found 3 matching customers
```

```text
// Your error response
{
    "error": {
        "code": "RATE_LIMITED",
        "message": "Too many requests",
        "retry_after_seconds": 30
    }
}

// AI can reason:
// - RATE_LIMITED means I should wait and retry
// - NOT_FOUND means I should try a different query
// - PERMISSION_DENIED means I should inform the user
// - INVALID_FORMAT means I should fix my parameters
```

1.


### Security: MCP as an Attack Vector

MCP servers expose your backend systems to a new attack surface.


### The Threat Model

```text
                  THREAT LANDSCAPE
Malicious User
    Prompt        ▶   AI Client      ▶  MCP Server
   Injection          (Claude)           (Your Code)
                            Backend Systems
                      • Databases (SQL injection)
                      • File systems (path traversal)
                      • APIs (credential theft)
                      • Internal networks (SSRF)
```


### The First Line of Defense: Authentication

Before discussing input validation, it's critical to understand that authentication is your first barrier. Every request to your MCP server should require a valid OAuth access token that:

```text
                  DEFENSE IN DEPTH
Request   ▶ [Layer 1: OAuth] Token invalid?   ▶ REJECT
                  ▼ (token valid)
       [Layer 2: Authorization] No permission?   ▶ REJECT
                  ▼ (authorized)
       [Layer 3: Input Validation] Invalid?   ▶ REJECT
                  ▼ (validated)
            [Execute Tool]
```


### With proper OAuth integration:

A sales analyst can only query sales data they have access to in the underlying system An attacker without valid credentials gets rejected at the gate Even if prompt injection convinces the AI to try accessing admin tables, the user's token doesn't have those permissions


### Best Practice: Pass-Through Authentication

The backend data system is the source of truth for permissions—not your MCP server.

Your MCP server should pass the user's access token through to backend systems and let them enforce permissions:

```rust
pub async fn execute_query(
    sql: &str,
    user_token: &AccessToken,  // Pass through, don't interpret
    pool: &DbPool,
) -> Result<Value, Error> {
    // Backend database enforces row-level security based on token
    let conn = pool.get_connection_with_token(user_token).await?;

    // The database sees the user's identity and applies its own permissions
    // If user can't access certain rows/tables, the DB rejects the query
    let results = conn.query(sql).await?;

    Ok(results)
}
```


### Don't duplicate permission logic in your MCP server:

```rust
// [BAD] BAD: Duplicating permission checks in MCP server
if user.role != "admin" && table_name == "salaries" {
    return Err(Error::Forbidden("Only admins can query salaries"));
}
// This duplicates logic that already exists in your HR database!

// [GOOD] GOOD: Let the backend enforce its own permissions
// Pass the token through; the HR database already knows who can see salaries
let results = hr_database.query_with_token(sql, &user_token).await?;
```


### What the MCP server SHOULD restrict:

Only add restrictions that are inherent to the MCP server's design—things the backend systems don't know about:

```rust
// [GOOD] GOOD: Block internal/system tables not meant for MCP exposure
let mcp_forbidden_tables = [
    "mcp_audit_log",      // MCP server's internal logging
    "mcp_rate_limits",    // MCP server's rate limit tracking
    "pg_catalog",         // Database system tables
    "information_schema", // Database metadata (if not explicitly exposed)
];
```


### if mcp_forbidden_tables.iter().any(|t| sql_lower.contains(t)) {

```rust
    return Err(Error::Validation(
        "This table is not accessible through the MCP interface".into()
    ));
}

// But DON'T block business tables—let the backend decide based on the token
// whether this user can access "salaries", "customer_pii", etc.
```

(table/list, cells in reading order)
- This approach has several benefits:
- Benefit
- Why It Matters
- Single source of truth
- Permissions are managed in one place (the data system)
- No sync issues

When permissions change in the backend, MCP automatically reflects them

(table/list, cells in reading order)
- Reduced attack surface
- Less permission logic = fewer bugs to exploit
- Audit compliance
- Backend systems have mature audit logging for access control
- Simpler MCP code
- Your server focuses on protocol, not authorization


### Attack Type 1: Prompt Injection for Data Theft

Malicious users can manipulate AI clients to extract data they shouldn't access:


### Defense: Validate query intent, not just syntax:

```rust
pub fn validate_query_security(sql: &str) -> Result<(), SecurityError> {
    let sql_lower = sql.to_lowercase();

    // Block access to sensitive tables
    let forbidden_tables = ["users", "credentials", "api_keys", "sessions", "audit_log"];
    for table in forbidden_tables {
        if sql_lower.contains(table) {
            return Err(SecurityError::ForbiddenTable {
                table: table.to_string(),
                message: format!(
                    "Access to '{}' table is not permitted through this tool. \
                    Contact your administrator for access.",
                    table
                ),
            });
        }
    }

    // Block sensitive columns even in allowed tables
    let forbidden_columns = ["password", "secret", "token", "private_key", "ssn"];
    for column in forbidden_columns {
        if sql_lower.contains(column) {
            return Err(SecurityError::ForbiddenColumn {
                column: column.to_string(),
                message: format!(
                    "Column '{}' contains sensitive data and cannot be queried.",
                    column
                ),
            });
        }
    }

    Ok(())
}
```


### Attack Type 2: SQL Injection Through AI

Even when the AI constructs queries, malicious input can embed SQL injection:


### Defense: Never allow raw SQL construction—use parameterized queries:

```rust
// DANGEROUS: AI-constructed SQL with string interpolation
Tool::new("unsafe_query")
    .description("Query customers by criteria")
    // AI might construct: WHERE name = '{user_input}'

// SAFE: Parameterized queries only
Tool::new("customer_search")
    .description("Search customers by specific fields")
    .input_schema(json!({
        "properties": {
            "name": { "type": "string", "maxLength": 100 },
            "email": { "type": "string", "format": "email" },
            "region": { "type": "string", "enum": ["NA", "EU", "APAC"] }
        }
    }))

pub async fn handle_customer_search(params: Value) -> Result<Value> {
    let validated = validate_customer_search(&params)?;

    // Use parameterized query—input is NEVER interpolated into SQL
    let rows = sqlx::query(
        "SELECT id, name, email, region FROM customers
         WHERE ($1::text IS NULL OR name ILIKE $1)
         AND ($2::text IS NULL OR email = $2)
         AND ($3::text IS NULL OR region = $3)"
    )
    .bind(validated.name.map(|n| format!("%{}%", n)))
    .bind(validated.email)
    .bind(validated.region)
    .fetch_all(&pool)
    .await?;

    Ok(json!({ "customers": rows }))
}
```

User: "Get ALL historical data from the transactions table for the past 10 years" AI: db_query(sql: "SELECT * FROM transactions WHERE date > '2014-01-01'")

```text
// Returns 500 million rows, crashes the server
```


### Defense: Enforce resource limits at every level:

```rust
Tool::new("db_query")
    .input_schema(json!({
        "properties": {
            "sql": { "type": "string", "maxLength": 4000 },  // Limit query size
            "limit": {
                "type": "integer",
                "minimum": 1,
                "maximum": 1000,  // Hard cap on rows
                "default": 100
            },
            "timeout_ms": {
                "type": "integer",
                "minimum": 100,
                "maximum": 10000,  // 10 second max
                "default": 5000
            }
        }
    }))

pub async fn handle_query(params: Value) -> Result<Value> {
    let validated = validate_query(&params)?;

    // Enforce limits even if not specified
    let limit = validated.limit.min(1000);

    // Wrap query with timeout
    let result = tokio::time::timeout(
        Duration::from_millis(validated.timeout_ms as u64),
        execute_query(&validated.sql, limit)
    ).await
    .map_err(|_| SecurityError::QueryTimeout {
        message: "Query exceeded time limit. Try a more specific query.".into()
    })?;

    result
}
```

User: "Read the config file at ../../../../etc/passwd" AI: file_read(path: "../../../../etc/passwd")


### Defense: Validate and sanitize all paths:

```rust
use std::path::{Path, PathBuf};

pub fn validate_file_path(
    requested_path: &str,
    allowed_root: &Path,
) -> Result<PathBuf, SecurityError> {
    // Resolve to absolute path
    let requested = Path::new(requested_path);
    let absolute = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        allowed_root.join(requested)
    };

    // Canonicalize to resolve .. and symlinks
    let canonical = absolute.canonicalize()
        .map_err(|_| SecurityError::InvalidPath {
            path: requested_path.to_string(),
            message: "Path does not exist or cannot be accessed".into(),
        })?;

    // Verify it's within allowed directory
    if !canonical.starts_with(allowed_root) {
        return Err(SecurityError::PathTraversal {
            path: requested_path.to_string(),
            message: format!(
                "Access denied. Files must be within: {}",
                allowed_root.display()
            ),
        });
    }

    Ok(canonical)
}
```

User: "What environment variables are set?


### Defense: Never expose secrets through tools:

```rust
pub fn sanitize_environment_output(vars: HashMap<String, String>) -> HashMap<String, String> {
    let secret_patterns = [
        "KEY", "SECRET", "PASSWORD", "TOKEN", "CREDENTIAL",
        "PRIVATE", "AUTH", "API_KEY", "CONNECTION_STRING"
    ];

    vars.into_iter()
        .map(|(key, value)| {
            let is_secret = secret_patterns.iter()
                .any(|pattern| key.to_uppercase().contains(pattern));

            if is_secret {
                (key, "[REDACTED]".to_string())
            } else {
                (key, value)
            }
        })
        .collect()
}

// Don't provide tools that read arbitrary config files
// Instead, expose only specific, safe configuration
Tool::new("get_app_config")
    .description("Get application configuration (non-sensitive settings only)")
```

```rust
pub async fn handle_tool_call(tool: &str, params: Value) -> Result<Value> {
    // Layer 1: Schema validation (type safety)
    let schema_result = validate_schema(tool, &params)?;

    // Layer 2: Business validation (logical constraints)
    let business_result = validate_business_rules(tool, &params)?;

    // Layer 3: Security validation (threat prevention)
    let security_result = validate_security(tool, &params)?;

    // Layer 4: Rate limiting (abuse prevention)
    check_rate_limit(&caller_id, tool).await?;

    // Layer 5: Audit logging (forensics)
    log_tool_invocation(tool, &params, &caller_id).await;

    // Execute only after all validations pass
    execute_tool(tool, params).await
}
```

- Security errors should be informative but not leak sensitive details:

```rust
pub enum SecurityError {
    ForbiddenTable { table: String, message: String },
    ForbiddenColumn { column: String, message: String },
    PathTraversal { path: String, message: String },
    QueryTimeout { message: String },
    RateLimited { retry_after: u32 },
}

impl SecurityError {
    pub fn to_safe_response(&self) -> Value {
        match self {
            // Tell AI what's blocked without revealing system details
            SecurityError::ForbiddenTable { message, .. } => json!({
                "error": {
                    "code": "ACCESS_DENIED",
                    "message": message,
                    "suggestion": "Query a different table or contact administrator"
                }
            }),
            SecurityError::PathTraversal { message, .. } => json!({
                "error": {
                    "code": "ACCESS_DENIED",
                    "message": message,
                    "suggestion": "Request a file within the allowed directory"
                }
            }),
            SecurityError::RateLimited { retry_after } => json!({
                "error": {
                    "code": "RATE_LIMITED",
                    "message": "Too many requests",
                    "retry_after_seconds": retry_after
                }
            }),
            _ => json!({
                "error": {
                    "code": "SECURITY_VIOLATION",
                    "message": "Request was blocked for security reasons"
                }
            })

        }
    }
}
```


### The First Line of Defense

Input validation isn't just about correctness—it's about security.

```yaml
thoroughly:
```

Fail fast with clear errors (don't let partial attacks proceed) 3.

Remember: malicious users don't care that an AI is between them and your systems.


### Schema Validation Libraries

For more sophisticated validation, consider schema validation libraries:

```rust
use jsonschema::{JSONSchema, Draft};

pub struct ValidatedTool {
    schema: JSONSchema,
}

impl ValidatedTool {
    pub fn new(schema: Value) -> Self {
        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .expect("Invalid schema");

        Self { schema: compiled }
    }

    pub fn validate(&self, params: &Value) -> Result<(), Vec<ValidationError>> {
        let result = self.schema.validate(params);

        if let Err(errors) = result {
            let validation_errors: Vec<ValidationError> = errors
                .map(|e| ValidationError {
                    code: "SCHEMA_VIOLATION".into(),
                    field: e.instance_path.to_string(),
                    message: e.to_string(),
                    expected: None,
                    received: Some(format!("{}", e.instance)),
                })
                .collect();

            return Err(validation_errors);
        }

        Ok(())
    }
}
```

(table/list, cells in reading order)
- Summary
- Schema-driven validation is about communication:
- Aspect
- Purpose
- Required fields
- Tell AI what it must provide
- Types and formats
- Guide AI to correct data shapes
- Enums
- Constrain to valid choices
- Patterns with examples
- Show exact expected format
- Clear error messages
- Help AI self-correct


### Output Schemas for Composition

Input validation prevents errors.

Without knowing the output structure, the AI must guess—or execute the first tool and inspect results before continuing.


### The PMCP SDK Approach: TypedToolWithOutput

Just as  TypedTool  auto-generates input schemas from Rust structs, PMCP provides  TypedToolWithOutput  that generates both input AND output schemas automatically:

```rust
use pmcp::TypedToolWithOutput;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input: Query parameters for top customers
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopCustomersInput {
    /// Time period for revenue calculation
    period: Period,

    /// Maximum number of customers to return (1-100)
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Month,
    Quarter,
    Year,
}

fn default_limit() -> u32 { 10 }

/// Output: List of top customers with revenue data
#[derive(Debug, Serialize, JsonSchema)]
pub struct TopCustomersOutput {
    /// List of customers sorted by revenue (highest first)
    pub customers: Vec<CustomerSummary>,

    /// The period that was queried
    pub period: String,

    /// When this report was generated (ISO 8601)
    pub generated_at: String,
}

#[derive(Debug, Serialize, JsonSchema)]

pub struct CustomerSummary {
    /// Unique customer identifier - use with order_history, customer_details
    pub customer_id: String,

    /// Customer display name
    pub name: String,

    /// Total revenue in USD cents (divide by 100 for dollars)
    pub total_revenue: i64,

    /// Number of orders in the period
    pub order_count: u32,

    /// Most recent order date (ISO 8601)
    pub last_order_date: String,
}
```


### Now create the tool with both schemas auto-generated:

v2.0 Tip: The  #[mcp_tool]  macro eliminates  Box::pin(async move { ... })  boilerplate.

```rust
let top_customers_tool = TypedToolWithOutput::new(
    "sales_top_customers",
    |args: TopCustomersInput, _extra| {
        Box::pin(async move {
            let customers = fetch_top_customers(&args.period, args.limit).await?;

            Ok(TopCustomersOutput {
                customers,
                period: format!("{:?}", args.period).to_lowercase(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            })
        })
    }
)
.with_description(
    "Get top customers by revenue for a time period. \
    Returns customer_id values that work with order_history and customer_details tools."
);
```


### The PMCP SDK automatically:

1.

```rust
{
  "type": "object",
  "properties": {
    "customers": {
      "type": "array",
      "description": "List of customers sorted by revenue (highest first)",
      "items": {
        "type": "object",
        "properties": {
          "customer_id": {
            "type": "string",
            "description": "Unique customer identifier - use with order_history, customer_details"
          },
          "total_revenue": {
            "type": "integer",
            "description": "Total revenue in USD cents (divide by 100 for dollars)"
          }
        }
      }
    }
  }
}
```


### The AI now knows:

Results are in a  customers  array Each customer has  customer_id  (not  id  or  customer ) Revenue is in cents (needs division for dollars) customer_id  works with other tools


### MCP Structured Content

MCP supports returning both human-readable text and structured data in tool responses.

```rust
use serde_json::json;

// Inside your tool handler
Ok(json!({
    "content": [{
        "type": "text",
        "text": format!("Found {} top customers for {}",
                       output.customers.len(), output.period)
    }],
    "structuredContent": output,  // The typed TopCustomersOutput
    "isError": false
}))
```


### AI clients see:

content: Human-readable summary for display structuredContent: Typed data matching your output schema


### The Structured Response Pattern

```rust
use pmcp::Error;
use serde::Serialize;

/// Helper to create MCP-compliant responses with structured content
pub fn structured_response<T: Serialize>(
    summary: &str,
    data: T,
) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary
        }],
        "structuredContent": data,
        "isError": false
    }))
}

/// Helper for structured error responses
pub fn structured_error<T: Serialize>(
    message: &str,
    error_data: T,
) -> Result<serde_json::Value, Error> {
    Ok(json!({
        "content": [{
            "type": "text",
            "text": message
        }],
        "structuredContent": error_data,
        "isError": true
    }))
}

// Usage in tool handler
let output = fetch_top_customers(&args).await?;
```


### structured_response(

```text
    &format!("Found {} top customers", output.customers.len()),
    output
)
```

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct ToolResponse<T> {
    /// Whether the operation succeeded
    pub success: bool,

    /// Tool-specific response data (present when success=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error details (present when success=false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,

    /// Execution metadata
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ResponseMetadata {
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,

    /// Data source identifier
    pub source: String,

    /// Whether results came from cache
    pub cached: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorDetail {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

With a consistent envelope, the AI learns one pattern for all your tools:


### if response.success {

```text
    process(response.data)
} else {
    handle_error(response.error)
}
```


### Implementation

```rust
impl<T: Serialize> ToolResponse<T> {
    pub fn success(data: T, metadata: ResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }

    pub fn error(error: ErrorDetail, metadata: ResponseMetadata) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata,
        }
    }
}
```

(table/list, cells in reading order)
- Designing for Chaining
- Structure outputs to support common chaining patterns:
- IDs for Follow-up Operations
- When a tool returns entities, include IDs that work with other tools:

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct CustomerSummary {
    /// Unique customer identifier - use with order_history, customer_details tools
    pub customer_id: String,

    /// Customer display name
    pub name: String,
    // ...
}

// Document the relationship in the tool receiving the ID
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrderHistoryInput {
    /// Customer ID from sales_top_customers or customer_search
    pub customer_id: String,

    /// Maximum orders to return
    #[serde(default = "default_order_limit")]
    pub limit: u32,
}
```

The AI sees  customer_id  in both schemas and understands how to chain them.

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    /// The result items for this page
    pub results: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PaginationInfo {
    /// Total number of results available
    pub total_count: u64,

    /// Number of results per page
    pub page_size: u32,

    /// Whether more results are available
    pub has_more: bool,

    /// Pass to 'cursor' parameter to get next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
```

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct SalesMetrics {
    /// Revenue in USD cents (divide by 100 for dollars)
    pub revenue_cents: i64,

    /// Number of units sold
    pub quantity: u32,

    /// Percentage as decimal (0.15 = 15%)
    pub growth_rate: f64,
}
```


### Type-Safe Server Composition

Output schemas become even more powerful when servers call other servers.

```rust
// Without output schemas - what shape does result have?
let result: Value = composition_client
    .call_tool("sqlite-explorer", "query", json!({"sql": "SELECT * FROM orders"}))
    .await?;

// Must guess or parse manually - error prone!
let rows = result["rows"].as_array().ok_or("expected rows")?;
```

```bash
# Export schema from running server
cargo pmcp schema export --endpoint https://my-server.pmcp.run/mcp \
    --output my-server-schema.json

# Generate typed Rust client
cargo pmcp generate --schema my-server-schema.json \
    --output src/clients/my_server.rs
```


### The generated code includes both input AND output types:

```rust
//! Auto-generated typed client for sqlite-explorer

/// Arguments for query tool
#[derive(Debug, Serialize)]
pub struct QueryArgs {
    /// SQL query to execute
    pub sql: String,
}

/// Result from query tool (from top-level outputSchema)
#[derive(Debug, Deserialize)]
pub struct QueryResult {
    /// Column names from the result set
    pub columns: Vec<String>,
    /// Row data as arrays of values
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Total number of rows returned
    pub row_count: i64,
}

/// Typed client for sqlite-explorer server
impl SqliteExplorerClient {
    /// Execute SQL query and return results
    pub async fn query(&self, args: QueryArgs) -> Result<QueryResult, Error> {
        // Type-safe call with automatic serialization/deserialization
    }
}
```


### Now your domain server has full type safety:

```rust
// In your domain server composing sqlite-explorer
let result: QueryResult = sqlite_client
    .query(QueryArgs { sql: "SELECT * FROM orders".into() })
    .await?;

// Compiler-checked field access!
println!("Found {} rows with {} columns",
         result.row_count, result.columns.len());

for row in &result.rows {
    // Process typed data
}
```


### Output Schema on ToolInfo

Per MCP spec 2025-06-18, output schemas are top-level fields on  ToolInfo :

```rust
use pmcp::types::{ToolInfo, ToolAnnotations};

let tool = ToolInfo::new("query", Some("Execute SQL".into()), input_schema)
    .with_output_schema(schemars::schema_for!(QueryResult))
    .with_annotations(
        ToolAnnotations::new()
            .with_read_only(true)
            .with_output_type_name("QueryResult")  // PMCP codegen extension
    );
```


### The exported tool metadata includes:

```json
{
  "name": "query",
  "inputSchema": { ... },
  "outputSchema": { ... },
  "annotations": {
    "readOnlyHint": true,
    "pmcp:outputTypeName": "QueryResult"
  }
}
```

The  pmcp:outputTypeName  annotation is a PMCP codegen extension.

```rust
pub async fn generate_report(params: ReportInput) -> Result<ReportOutput, Error> {
    let report = build_report(&params).await?;

    // Validate output matches business rules
    if report.total_revenue < 0 {
        return Err(Error::Internal(
            "Generated report has negative revenue - data integrity issue".into()
        ));
    }

    if report.customer_id.is_empty() {
        return Err(Error::Internal(
            "Generated report missing customer_id".into()
        ));
    }

    Ok(report)
}
```

```rust
// Compiler error if you return wrong type!
TypedToolWithOutput::new("my_tool", |args: Input, _| {
    Box::pin(async move {
        Ok(Output { ... })  // Must match Output type exactly
    })
})
```


### 3. Document Field Relationships in Comments

```rust
#[derive(Debug, Serialize, JsonSchema)]
pub struct CustomerSummary {
    /// Unique customer ID. Use with:
    /// - order_history: Get customer's order history
    /// - customer_details: Get full customer profile
    /// - customer_contacts: Get customer contact list
    pub customer_id: String,
}
```

```rust
// Good: Clear, descriptive name
#[derive(Debug, Serialize, JsonSchema)]
pub struct OrderQueryResult { ... }

// Bad: Generic name causes conflicts
pub struct Result { ... }
```

(table/list, cells in reading order)
- Summary
- Output schemas enable composition by telling AI clients:
- What to Document
- Why It Matters
- Field names and types
- AI constructs follow-up operations correctly
- ID relationships
- AI knows how to chain tools together
- What to Document
- Why It Matters
- Consistent envelopes
- AI learns one pattern for all your tools
- Error structures
- AI can handle failures gracefully
- Units and formats
- AI interprets values correctly
- Pagination patterns
- AI knows how to get more results
- PMCP SDK Benefits
- Manual JSON Schema
- TypedToolWithOutput
- Schema and code can drift
- Schema generated from code—always in sync
- Manual JSON construction
- Rust types with derive macros
- No code generation
- Generate typed clients for composition
- Runtime type errors
- Compile-time type safety
- Verbose documentation
- Doc comments become schema descriptions

Remember: output schemas are a contract.


### Type-Safe Tool Annotations

MCP tool annotations provide metadata beyond schemas—hints about behavior, safety, and usage that help AI clients make better decisions.

Annotations are structured metadata attached to tools that describe characteristics the AI should consider:

```rust
use pmcp::types::ToolAnnotations;

let annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(true)
    .with_idempotent(false)
    .with_open_world(false);
```


### These annotations tell the AI:

This tool modifies data (not read-only) It can be destructive (data loss possible) It's not idempotent (calling twice has different effects) It operates on a closed world (internal database)

(table/list, cells in reading order)
- Standard MCP Annotations
- The MCP specification defines several standard annotation hints:
- readOnlyHint
- Indicates whether the tool only reads data or can modify state:

```rust
// Read-only tool - safe to call speculatively
let annotations = ToolAnnotations::new()
    .with_read_only(true);

// Modifying tool - AI should confirm before calling
let annotations = ToolAnnotations::new()
    .with_read_only(false);
```

```rust
// Non-destructive: data can be recovered
let annotations = ToolAnnotations::new()
    .with_destructive(false);

// Destructive: data is permanently lost
let annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(true);
```


### idempotentHint

Indicates whether calling the tool multiple times has the same effect as calling once:

```rust
// Idempotent: safe to retry
let annotations = ToolAnnotations::new()
    .with_idempotent(true);

// Not idempotent: each call has cumulative effect
let annotations = ToolAnnotations::new()
    .with_idempotent(false);
```

```rust
// Closed world: internal database only
let annotations = ToolAnnotations::new()
    .with_open_world(false);

// Open world: calls external APIs
let annotations = ToolAnnotations::new()
    .with_open_world(true);
```


### PMCP SDK: ToolAnnotations Builder

The PMCP SDK provides a fluent builder for creating type-safe annotations:

```rust
use pmcp::types::ToolAnnotations;
use serde_json::json;

// Build annotations with the fluent API
let annotations = ToolAnnotations::new()
    .with_read_only(true)
    .with_idempotent(true)
    .with_open_world(false);

// Create a tool with annotations
use pmcp::types::ToolInfo;

let tool = ToolInfo::with_annotations(
    "sales_query",
    Some("Query sales data from PostgreSQL 15".to_string()),
    json!({
        "type": "object",
        "properties": {
            "sql": { "type": "string" }
        }
    }),
    annotations,
);
```


### Combining with Output Schema

For tools that need both behavioral hints and output schemas, set the output schema on  ToolInfo  (top-level per MCP spec 2025-06-18) and the type name in annotations:

```rust
let tool = ToolInfo::new("query", Some("Execute SQL".into()), input_schema)
    .with_output_schema(json!({
        "type": "object",
        "properties": {
            "rows": { "type": "array" },
            "count": { "type": "integer" }
        }
    }))
    .with_annotations(
        ToolAnnotations::new()
            .with_read_only(true)
            .with_output_type_name("QueryResult")
    );
```


### TypedTool and Annotations

The PMCP SDK provides full annotation support directly on  TypedTool ,  TypedSyncTool , and  TypedToolWithOutput .


### TypedTool with Annotations

v2.0 Tip: The  #[mcp_tool]  macro eliminates  Box::pin(async move { ... })  boilerplate.

```rust
use pmcp::server::typed_tool::TypedTool;
use pmcp::types::ToolAnnotations;
use schemars::JsonSchema;
use serde::Deserialize;

/// Input parameters for the delete tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteCustomerInput {
    /// Customer ID to permanently delete
    pub customer_id: String,

    /// Reason for deletion (required for audit log)
    pub reason: String,
}

// Full annotation support with TypedTool
let tool = TypedTool::new("delete_customer", |args: DeleteCustomerInput, _extra| {
    Box::pin(async move {
        if args.reason.len() < 10 {
            return Err(pmcp::Error::Validation(
                "Deletion reason must be at least 10 characters".into()
            ));
        }
        // Execute deletion...
        Ok(serde_json::json!({ "deleted": true, "customer_id": args.customer_id }))
    })
})
.with_description("Permanently delete a customer and all associated data")
.with_annotations(
    ToolAnnotations::new()
        .with_read_only(false)
        .with_destructive(true)     // Permanent deletion
        .with_idempotent(true)      // Deleting twice = same result
        .with_open_world(false)     // Internal database
);
```


### Convenience Methods

For common annotation patterns, use the convenience methods directly on the tool:

```rust
// Read-only query tool
let query_tool = TypedTool::new("sales_query", |args: QueryInput, _| {
    Box::pin(async move {
        // Execute read-only query...
        Ok(serde_json::json!({ "rows": [] }))
    })
})
.with_description("Query sales data from PostgreSQL 15")
.read_only()      // Sets readOnlyHint: true
.idempotent();    // Sets idempotentHint: true

// Destructive delete tool
let delete_tool = TypedTool::new("delete_record", |args: DeleteInput, _| {
    Box::pin(async move {
        // Execute deletion...
        Ok(serde_json::json!({ "deleted": true }))
    })
})
.with_description("Permanently delete a record")
.destructive()    // Sets readOnlyHint: false, destructiveHint: true
.idempotent();    // Safe to retry

// External API tool
let api_tool = TypedTool::new("fetch_stock_price", |args: StockInput, _| {
    Box::pin(async move {
        // Call external API...
        Ok(serde_json::json!({ "price": 150.25 }))
    })
})
.with_description("Fetch current stock price from market data API")
.read_only()
.open_world();    // Sets openWorldHint: true (external system)
```


### TypedToolWithOutput: Merged Annotations

When using  TypedToolWithOutput , user-provided annotations are automatically merged with the auto-generated output schema

```rust
annotation:

use pmcp::server::typed_tool::TypedToolWithOutput;
use pmcp::types::ToolAnnotations;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryInput {
    pub sql: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct QueryOutput {
    pub rows: Vec<serde_json::Value>,
    pub count: i64,
}

let tool = TypedToolWithOutput::new("query", |args: QueryInput, _| {
    Box::pin(async move {
        // Execute query...
        Ok(QueryOutput { rows: vec![], count: 0 })
    })
})
.with_description("Execute SQL query")
.read_only()      // User-provided: readOnlyHint: true
.idempotent();    // User-provided: idempotentHint: true

// The tool now has BOTH:
// - User annotations: readOnlyHint, idempotentHint, pmcp:outputTypeName
// - Auto-generated top-level: outputSchema on ToolInfo
```


### TypedSyncTool for Synchronous Handlers

For tools that don't need async, use  TypedSyncTool  with the same annotation support:

```rust
use pmcp::server::typed_tool::TypedSyncTool;

let tool = TypedSyncTool::new("calculate", |args: CalcInput, _extra| {
    // Synchronous computation
    Ok(serde_json::json!({ "result": args.a + args.b }))
})
.with_description("Perform calculation")
.read_only()
.idempotent();
```

```rust
let annotations = ToolAnnotations::new()
    .with_read_only(true)
    .with_idempotent(true)    // Same query = same results
    .with_open_world(false);  // Internal database
```


### External API Tools

```rust
let annotations = ToolAnnotations::new()
    .with_read_only(true)     // Just fetching data
    .with_open_world(true)    // Calls external API
    .with_idempotent(false);  // External state may change
```


### Update Tools (Modifying)

```rust
let annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(false)  // Updates are recoverable
    .with_idempotent(true);   // SET status='active' is idempotent
```


### Delete Tools (Destructive)

```rust
let annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(true)   // Permanent deletion
    .with_idempotent(true);   // Deleting twice = same result
```


### Insert Tools (Non-Idempotent)

```rust
let annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(false)
    .with_idempotent(false);  // Each insert creates new record
```


### Custom Annotations

Beyond standard hints, define custom annotations for your domain using the raw JSON approach:


### Using ToolInfo with Custom Fields

```rust
use pmcp::types::ToolInfo;

// Start with standard annotations
let mut annotations = ToolAnnotations::new()
    .with_read_only(false)
    .with_destructive(true);

// Create tool info
let mut tool = ToolInfo::with_annotations(
    "admin_reset",
    Some("Reset user password".into()),
    input_schema,
    annotations,
);

// Access the underlying _meta for custom fields if needed
// (Custom annotations beyond MCP standard hints)
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainAnnotations {
    // MCP standard hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    // Custom domain annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_role: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_log: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

impl DomainAnnotations {
    pub fn admin_only() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(true),
            idempotent_hint: None,
            requires_role: Some("admin".into()),

            audit_log: Some(true),
            rate_limit: None,
        }
    }

    pub fn external_api(rpm: u32) -> Self {
        Self {
            read_only_hint: Some(true),
            destructive_hint: Some(false),
            idempotent_hint: Some(false),
            requires_role: None,
            audit_log: None,
            rate_limit: Some(RateLimitConfig {
                requests_per_minute: rpm,
                requests_per_hour: rpm * 60,
            }),
        }
    }
}
```

```rust
pub async fn execute_tool(
    tool: &RegisteredTool,
    params: Value,
    context: &ExecutionContext,
) -> Result<Value> {
    if let Some(annotations) = &tool.annotations {
        // Check role requirements (custom annotation)
        if let Some(required_role) = annotations.requires_role.as_ref() {
            if !context.user_roles.contains(required_role) {
                return Err(Error::AccessDenied(format!(
                    "Tool '{}' requires role '{}'",
                    tool.name, required_role
                )));
            }
        }

        // Require confirmation for destructive operations
        if annotations.destructive_hint == Some(true) && !context.confirmed {
            return Err(Error::ConfirmationRequired(
                "This operation is destructive. Please confirm.".into()
            ));
        }

        // Log audit trail
        if annotations.audit_log == Some(true) {
            audit_log(&tool.name, &params, &context.user_id).await;
        }
    }

    // Execute the tool
    (tool.handler)(params, context).await
}
```

```rust
pub fn generate_safety_docs(tool: &ToolInfo) -> String {
    let mut doc = String::new();

    if let Some(ann) = &tool.annotations {
        doc.push_str("### Safety Characteristics\n\n");

        if ann.read_only_hint == Some(true) {
            doc.push_str("- [GOOD] **Read-only**: Safe to call without modifying data\n");
        } else if ann.read_only_hint == Some(false) {
            doc.push_str("- [warn] **Modifies data**: This tool changes system state\n");
        }

        if ann.destructive_hint == Some(true) {
            doc.push_str("- [BAD] **Destructive**: May cause irreversible changes\n");
        }

        if ann.idempotent_hint == Some(true) {
            doc.push_str("- 🔄 **Idempotent**: Safe to retry on failure\n");
        }

        if ann.open_world_hint == Some(true) {
            doc.push_str("- 🌐 **External**: Interacts with external systems\n");
        }
    }

    doc
}
```

(table/list, cells in reading order)
- Summary
- Tool annotations provide behavioral metadata that:
- Annotation
- Purpose
- AI Behavior
- readOnlyHint
- Read vs write
- Controls speculation
- destructiveHint
- Irreversible changes
- Requires confirmation
- idempotentHint
- openWorldHint
- Safe to retry
- Retry on failure
- External systems
- Expects latency/limits
- outputSchema  (top-level)
- Output type
- Enables composition
- PMCP SDK Annotation Support
- Tool Type
- Annotation Support
- TypedTool

Full:  .with_annotations() ,  .read_only() ,  .destructive() ,  .idempotent() ,

```text
.open_world()
```

(table/list, cells in reading order)
- TypedSyncTool
- Full: Same methods as  TypedTool
- TypedToolWithOutput
- Full: Same methods + auto-merges with output schema

```rust
ToolInfo::with_annotations()
```

(table/list, cells in reading order)
- Full: Direct construction with  ToolAnnotations  builder
- Custom  ToolHandler
- Full control via  metadata()  method
- Best Practices

Always annotate destructive tools - AI clients need this for user safety 2.
