---
chapter: 25
title: "Server Composition"
part: "Part VIII: Advanced Patterns"
use_when: "Organising 5-10+ servers: foundation/domain/orchestration tiers; server-side workflows."
suggested_prerequisites: [ch05, ch06, ch23]
source_kb: 75
extracted_kb: 44
code_blocks: 29
quality: auto-extracted
---

# Ch 25: Server Composition

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

## Agent notes (checked against extracted text)

- Use only with many servers. Source says skip with fewer than 5 servers; read with 10+.
- Tiers: Foundation (stable, generic: auth, DB, files, logging) -> Domain (business tools) -> Orchestration (cross-domain workflows).
- Bad foundation candidates: business rules, domain calculations, department logic.
- Example code uses `Server::builder().with_foundation(...)`. Verify this API in the repo before using.

## Extracted content


### Server Composition

This chapter covers advanced patterns for building hierarchies of MCP servers in large organizations. These techniques become valuable when you have many domain-specific servers that share common functionality.


### When to Use Server Composition

```text
                  Is This Chapter For You?
[warn]  ADVANCED TOPIC - This chapter is OPTIONAL
Skip this chapter if:
• You have fewer than 5 MCP servers
• Your servers don't share common functionality
• You're still learning MCP basics
• Your organization hasn't standardized on MCP yet
Read this chapter when:
• You have 10+ MCP servers across teams
• You see duplicated code in multiple servers
• Teams are building similar tools independently
• Discovery of available tools has become difficult
• You need domain-specific server hierarchies
The techniques here help large organizations:
[ok] Reduce duplication with foundation servers
[ok] Organize servers by business domain
[ok] Enable tool discovery across the organization
[ok] Build complex workflows from simple components
```

(table/list, cells in reading order)
- The Problem at Scale
- As organizations adopt MCP, they often encounter these challenges:
- Problem
- Example
- Impact
- Code Duplication
- Every team implements their own "get-database- connection" tool
- Inconsistent behavior, maintenance burden
- Discovery Difficulty
- "Does anyone have a tool that does X?"
- Lost productivity, duplicate work
- Inconsistent Patterns
- Different error handling, naming, authentication
- Hard to compose servers
- Domain Isolation
- Finance tools mixed with HR tools in one server
- Hard to manage access control
- The Three-Tier Solution

Server composition addresses these challenges with a hierarchical approach:

```text
                  Server Composition Hierarchy
                          Orchestration        ← Complex workflows
                             Servers             spanning domains
    Finance Domain         HR Domain           Engineering
       Server               Server             Domain Server
                           Foundation          ← Shared capabilities:
                             Servers             auth, database, files
Layer Responsibilities:
Foundation: Core building blocks used by all domains
• Authentication tools (validate_token, get_user_info)
• Database access (query, insert, update)
• File operations (read, write, list)
• Logging and metrics infrastructure
Domain: Business-specific tools built on foundation
• Finance: expense_report, invoice, budget_forecast
• HR: employee_lookup, time_off_request, org_chart
• Engineering: deploy, rollback, service_status
```

```text
Orchestration: Cross-domain workflows
• Onboarding workflow (HR + Engineering + Finance)
• Quarterly review (HR + Finance)
• Incident response (Engineering + all affected domains)
```

- Don't Repeat Yourself applies to MCP server development:

```rust
// [BAD] WITHOUT composition: Every domain server duplicates auth
// finance_server.rs
async fn validate_token(token: &str) -> Result<User> {
    // 50 lines of auth code
}

// hr_server.rs
async fn validate_token(token: &str) -> Result<User> {
    // Same 50 lines copied
}

// engineering_server.rs
async fn validate_token(token: &str) -> Result<User> {
    // Same 50 lines copied again
}

// [GOOD] WITH composition: Foundation server provides auth
// foundation_auth_server.rs
pub struct AuthFoundation { /* ... */ }
impl AuthFoundation {
    pub async fn validate_token(&self, token: &str) -> Result<User> {
        // Auth logic written ONCE
    }
}

// Domain servers compose foundation
let finance_server = Server::builder()
    .name("finance-server")
    .with_foundation(auth_foundation.clone())  // Reuse!
    .tool("expense_report", expense_tool)      // Domain-specific
    .build()?;
```


### Discovery Benefits

With organized server hierarchies, AI clients can discover tools effectively:

```text
                  Tool Discovery with Composition
AI Client: "I need to check an employee's expense status"
Without Composition:                  With Composition:
Client must search 50+ servers        Client queries domains:
for relevant tools                    1. HR → employee_lookup
                                      2. Finance → expense_status
Hard to know which server             3. Orchestration → combines them
has what capability
                                      Clear hierarchy makes
Tools may have conflicting            discovery straightforward
names across servers
```

(table/list, cells in reading order)
- Key Concepts Preview
- Concept
- What It Means
- When to Use
- Foundation Server
- Provides core capabilities other servers build on
- Domain Server
- Business-specific server composing foundation components
- Orchestration
- Workflows spanning multiple servers/domains
- When multiple servers need the same functionality
- When a department needs specialized tools
- When tasks require coordination across boundaries
- Concept
- What It Means
- When to Use
- Dynamic Resources
- URI-template-based resource providers
- When resources follow patterns (users/{id}, files/{path})
- Server-Side Execution
- Tools executed by server, not client
- When workflows need deterministic execution
- Foundation Servers


### Foundation servers should be:

```text
                  Foundation Server Characteristics
[ok] STABLE: APIs rarely change (breaking changes affect all domains)
[ok] GENERIC: No business-specific logic
[ok] COMPOSABLE: Easy to combine with other foundations
[ok] WELL-TESTED: Heavily tested since bugs affect everyone
[ok] DOCUMENTED: Clear contracts for domain developers
Good Foundation Candidates:       Bad Foundation Candidates:
• Authentication/Authorization    • Business rules
• Database connectivity           • Domain calculations
• File system access              • Department-specific logic
• HTTP client operations          • UI/presentation code
• Caching infrastructure          • Company-specific policies
• Logging and metrics
```

```rust
use pmcp::{Result, Server};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// User information returned by authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub department: String,
}

/// Authentication foundation providing user validation and info retrieval
pub struct AuthFoundation {
    // In production: connection to identity provider (Okta, Auth0, etc.)
    user_cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, AuthenticatedUser>>>,
}

impl AuthFoundation {
    pub fn new() -> Self {
        Self {
            user_cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Validate a token and return user information
    pub async fn validate_token(&self, token: &str) -> Result<AuthenticatedUser> {
        // In production: validate JWT, check with IdP, etc.
        // This is the SINGLE place where token validation logic lives

        if token.starts_with("valid_") {
            let user_id = token.strip_prefix("valid_").unwrap_or("unknown");
            Ok(AuthenticatedUser {
                id: user_id.to_string(),
                email: format!("{}@company.com", user_id),
                roles: vec!["employee".to_string()],
                department: "engineering".to_string(),

            })
        } else {
            Err(pmcp::Error::protocol(
                pmcp::ErrorCode::INVALID_PARAMS,
                "Invalid authentication token",
            ))
        }
    }

    /// Check if user has required role
    pub fn has_role(&self, user: &AuthenticatedUser, required_role: &str) -> bool {
        user.roles.iter().any(|r| r == required_role || r == "admin")
    }

    /// Create middleware that validates tokens on every request
    pub fn create_middleware(&self) -> AuthMiddleware {
        AuthMiddleware {
            foundation: self.clone(),
        }
    }
// ... truncated, 14 more lines in source book ...
```

```rust
use async_trait::async_trait;
use pmcp::server::dynamic_resources::{DynamicResourceProvider, RequestContext, UriParams};
use pmcp::types::{Content, ReadResourceResult, ResourceTemplate};

/// Database foundation providing query capabilities
pub struct DatabaseFoundation {
    connection_string: String,
    // In production: connection pool (sqlx, diesel, etc.)
}

impl DatabaseFoundation {
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }

    /// Execute a read-only query
    pub async fn query(&self, sql: &str, params: &[&str]) -> Result<Vec<serde_json::Value>> {
        // Single place for:
        // - Query validation
        // - SQL injection prevention
        // - Connection pooling
        // - Query logging
        // - Performance metrics

        tracing::info!(sql = %sql, "Executing query");

        // Simulated response
        Ok(vec![serde_json::json!({
            "id": 1,
            "result": "sample data"
        })])
    }

    /// Create a dynamic resource provider for database tables
    pub fn create_table_provider(&self, allowed_tables: Vec<String>) -> TableResourceProvider {
        TableResourceProvider {
            foundation: self.clone(),

            allowed_tables,
        }
    }
}

impl Clone for DatabaseFoundation {
    fn clone(&self) -> Self {
        Self {
            connection_string: self.connection_string.clone(),
        }
    }
}

/// Dynamic resource provider for database tables
///
/// Provides resources like:
/// - tables://{table}/schema - Table schema
/// - tables://{table}/sample - Sample rows
/// - tables://{table}/count - Row count
pub struct TableResourceProvider {
// ... truncated, 81 more lines in source book ...
```

```rust
use std::path::{Path, PathBuf};

/// File system foundation with security controls
pub struct FileSystemFoundation {
    base_path: PathBuf,
    allowed_extensions: Vec<String>,
    max_file_size: usize,
}

impl FileSystemFoundation {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            allowed_extensions: vec![
                "txt".to_string(),
                "json".to_string(),
                "csv".to_string(),
                "md".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10 MB
        }
    }

    /// Safely resolve a path, preventing directory traversal attacks
    fn safe_path(&self, relative_path: &str) -> Result<PathBuf> {
        let path = self.base_path.join(relative_path);
        let canonical = path.canonicalize().map_err(|_| {
            pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "Path not found")
        })?;

        // Prevent directory traversal (../../../etc/passwd)
        if !canonical.starts_with(&self.base_path) {
            return Err(pmcp::Error::protocol(
                pmcp::ErrorCode::INVALID_PARAMS,
                "Path traversal not allowed",
            ));
        }

        // Check extension

        if let Some(ext) = canonical.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if !self.allowed_extensions.contains(&ext_str) {
                return Err(pmcp::Error::protocol(
                    pmcp::ErrorCode::INVALID_PARAMS,
                    format!("File extension '{}' not allowed", ext_str),
                ));
            }
        }

        Ok(canonical)
    }

    /// Read a file with security checks
    pub async fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.safe_path(relative_path)?;

        // Check file size
        let metadata = tokio::fs::metadata(&path).await.map_err(|e| {
            pmcp::Error::protocol(pmcp::ErrorCode::INTERNAL_ERROR, e.to_string())
// ... truncated, 36 more lines in source book ...
```

v2.0 Tip: The  #[mcp_tool]  macro eliminates  Box::pin(async move { ... })  boilerplate.

```rust
use pmcp::Server;
use std::sync::Arc;

/// A domain server that composes multiple foundations
pub async fn create_finance_server(
    auth: Arc<AuthFoundation>,
    db: Arc<DatabaseFoundation>,
    fs: Arc<FileSystemFoundation>,
) -> Result<Server> {
    // Create typed tools that use foundations
    let auth_clone = auth.clone();
    let db_clone = db.clone();

    Server::builder()
        .name("finance-server")
        .version("1.0.0")
        // Tool using auth + database foundations
        .tool_typed("get_expense_report", move |input: ExpenseReportInput, extra| {
            let auth = auth_clone.clone();
            let db = db_clone.clone();
            Box::pin(async move {
                // Use auth foundation
                let user = auth.validate_token(&input.token).await?;

                // Check permissions
                if !auth.has_role(&user, "finance_viewer") {
                    return Err(pmcp::Error::protocol(
                        pmcp::ErrorCode::INVALID_PARAMS,
                        "Insufficient permissions",
                    ));
                }

                // Use database foundation
                let expenses = db.query(
                    "SELECT * FROM expenses WHERE user_id = $1 AND month = $2",
                    &[&user.id, &input.month],
                ).await?;

                Ok(serde_json::json!({

                    "user": user.email,
                    "month": input.month,
                    "expenses": expenses
                }))
            })
        })
        // Add file resources using filesystem foundation
        .resources(
            pmcp::server::simple_resources::ResourceCollection::new()
                .add_dynamic_provider(Arc::new(
                    fs.create_resource_provider("reports://")
                ))
        )
        .build()
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ExpenseReportInput {
    token: String,
    month: String,
// ... truncated, 1 more lines in source book ...
```

```rust
/// Foundation trait with version information
pub trait Foundation: Send + Sync {
    /// Foundation version for compatibility checking
    fn version(&self) -> &str;

    /// Minimum compatible version
    fn min_compatible_version(&self) -> &str;
}

impl Foundation for AuthFoundation {
    fn version(&self) -> &str {
        "2.0.0"
    }

    fn min_compatible_version(&self) -> &str {
        "1.5.0"  // Backwards compatible with 1.5+
    }
}

/// Check foundation compatibility before composing
fn check_compatibility(foundation: &dyn Foundation, required_version: &str) -> Result<()> {
    let version = semver::Version::parse(foundation.version())?;
    let required = semver::Version::parse(required_version)?;

    if version < required {
        return Err(pmcp::Error::protocol(
            pmcp::ErrorCode::INTERNAL_ERROR,
            format!(
                "Foundation version {} is below required version {}",
                version, required
            ),
        ));
    }

    Ok(())
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn auth_foundation_validates_tokens() {
        let auth = AuthFoundation::new();

        // Valid token
        let user = auth.validate_token("valid_user123").await.unwrap();
        assert_eq!(user.id, "user123");

        // Invalid token
        let result = auth.validate_token("invalid_token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn auth_foundation_checks_roles() {
        let auth = AuthFoundation::new();
        let user = auth.validate_token("valid_admin").await.unwrap();

        assert!(auth.has_role(&user, "employee"));
        assert!(!auth.has_role(&user, "super_admin"));
    }

    #[tokio::test]
    async fn filesystem_prevents_traversal() {
        let fs = FileSystemFoundation::new("/data");

        // Attempting path traversal should fail
        let result = fs.read_file("../../../etc/passwd").await;
        assert!(result.is_err());

        // Valid path should work
        let result = fs.read_file("reports/q1.txt").await;
        // Depends on actual file existence
    }

    #[tokio::test]
    async fn database_validates_tables() {
        let db = DatabaseFoundation::new("postgres://localhost/test");
        let provider = db.create_table_provider(vec!["users".to_string(), "orders".to_string()]);

        // Allowed table
        let templates = provider.templates();
        assert_eq!(templates.len(), 3);

        // Verify URI template format
        assert!(templates[0].uri_template.contains("{table}"));
    }
}
```

(table/list, cells in reading order)
- Summary
- Foundation Type
- Provides
- Authentication
- Token validation, user info, roles
- Used By
- All domain servers
- Database
- File System
- HTTP Client
- Cache
- Connection pooling, query execution, resource providers
- Servers needing data access
- Safe file access, directory listing
- External API calls, retry logic
- Servers handling documents
- Integration servers
- In-memory and distributed caching
- Performance-critical servers


### Domain Servers

Domain servers provide business-specific tools organized by functional area.


### Domain Organization

```text
                  Domain Server Organization
Organization Structure
company/
    mcp-servers/
        foundations/          # Shared components
            auth/
            database/
            filesystem/
        domains/              # Business domains
            finance/          # Finance team owns
                expense-server/
                invoice-server/
                budget-server/
            hr/               # HR team owns
                employee-server/
                recruiting-server/
                benefits-server/
            engineering/      # Engineering team owns
                deploy-server/
                monitoring-server/
                incident-server/
        orchestration/        # Cross-domain workflows
            onboarding/
            offboarding/
```

(table/list, cells in reading order)
- Domain Ownership
- Each domain should have clear ownership:
- Domain
- Owner
- Scope
- Dependencies
- Finance
- Finance team
- Expenses, invoices, budgets
- Auth, Database
- HR
- HR team
- Employees, recruiting, benefits
- Auth, Database, Filesystem
- Engineering
- Platform team
- Deployments, monitoring, incidents
- Auth, Database, HTTP
- Sales
- Sales ops
- CRM, quotes, contracts
- Auth, Database, HTTP
- Building a Domain Server
- Step 1: Define Domain Boundaries
- Before writing code, define what belongs in the domain:

```text
/// Finance domain boundaries
///
/// INCLUDES:
/// - Expense reports (create, view, approve)
/// - Invoices (generate, send, track)
/// - Budget tracking and forecasting
/// - Financial reporting
///
/// EXCLUDES:
/// - Employee management (HR domain)
/// - Customer management (Sales domain)
/// - Authentication (Foundation)
/// - Database access (Foundation)

// This documentation becomes the contract for the domain
```

```rust
use pmcp::{Result, Server};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Import foundations
use crate::foundations::{AuthFoundation, DatabaseFoundation};

/// Finance domain server
pub struct FinanceDomainServer {
    auth: Arc<AuthFoundation>,
    db: Arc<DatabaseFoundation>,
}

impl FinanceDomainServer {
    pub fn new(auth: Arc<AuthFoundation>, db: Arc<DatabaseFoundation>) -> Self {
        Self { auth, db }
    }

    /// Build the MCP server with all finance domain tools
    pub fn build(&self) -> Result<Server> {
        let auth = self.auth.clone();
        let db = self.db.clone();

        Server::builder()
            .name("finance-domain-server")
            .version("1.0.0")
            // Expense tools
            .tool_typed("create_expense", self.create_expense_handler())
            .tool_typed("get_expenses", self.get_expenses_handler())
            .tool_typed("approve_expense", self.approve_expense_handler())
            // Invoice tools
            .tool_typed("generate_invoice", self.generate_invoice_handler())
            .tool_typed("track_invoice", self.track_invoice_handler())
            // Budget tools
            .tool_typed("get_budget_summary", self.budget_summary_handler())
            // Resources
            .resources(self.create_resources())
            .build()

    }

    // Tool handlers defined below...
}
```

```rust
/// Input for creating an expense report
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateExpenseInput {
    /// Authentication token
    pub token: String,
    /// Expense description
    pub description: String,
    /// Amount in cents (to avoid floating point issues)
    pub amount_cents: i64,
    /// Expense category
    pub category: ExpenseCategory,
    /// Optional receipt URL
    pub receipt_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategory {
    Travel,
    Meals,
    Supplies,
    Equipment,
    Software,
    Other,
}

/// Output for expense operations
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExpenseResult {
    pub expense_id: String,
    pub status: ExpenseStatus,
    pub submitted_by: String,
    pub submitted_at: String,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseStatus {
    Pending,

    Approved,
    Rejected,
    Reimbursed,
}
```

```rust
impl FinanceDomainServer {
    fn create_expense_handler(&self) -> impl Fn(CreateExpenseInput, RequestHandlerExtra) -> BoxFuture<'static,
Result<Value>> {
        let auth = self.auth.clone();
        let db = self.db.clone();

        move |input: CreateExpenseInput, _extra| {
            let auth = auth.clone();
            let db = db.clone();

            Box::pin(async move {
                // 1. Authenticate using foundation
                let user = auth.validate_token(&input.token).await?;

                // 2. Apply business rules (domain logic)
                validate_expense_amount(input.amount_cents)?;
                validate_category_for_user(&user, &input.category)?;

                // 3. Store using foundation
                let expense_id = generate_expense_id();
                db.query(
                    "INSERT INTO expenses (id, user_id, description, amount_cents, category, status)
                     VALUES ($1, $2, $3, $4, $5, 'pending')",
                    &[&expense_id, &user.id, &input.description,
                      &input.amount_cents.to_string(), &format!("{:?}", input.category)],
                ).await?;

                // 4. Return domain-specific result
                Ok(serde_json::to_value(ExpenseResult {
                    expense_id,
                    status: ExpenseStatus::Pending,
                    submitted_by: user.email,
                    submitted_at: chrono::Utc::now().to_rfc3339(),
                })?)
            })
        }
    }
}

/// Domain-specific business rule: expense limits
fn validate_expense_amount(amount_cents: i64) -> Result<()> {
    const MAX_EXPENSE_CENTS: i64 = 1_000_000; // $10,000

    if amount_cents <= 0 {
        return Err(pmcp::Error::Validation(
            "Expense amount must be positive".to_string()
        ));
    }

    if amount_cents > MAX_EXPENSE_CENTS {
        return Err(pmcp::Error::Validation(
            format!("Expense amount exceeds limit of ${:.2}", MAX_EXPENSE_CENTS as f64 / 100.0)
        ));
    }

    Ok(())
}

/// Domain-specific business rule: category restrictions
fn validate_category_for_user(user: &AuthenticatedUser, category: &ExpenseCategory) -> Result<()> {
// ... truncated, 11 more lines in source book ...
```


### Dynamic Resources for Domains

Domain servers often expose resources with patterns.

```rust
use async_trait::async_trait;
use pmcp::server::dynamic_resources::{DynamicResourceProvider, RequestContext, UriParams};
use pmcp::types::{Content, ReadResourceResult, ResourceTemplate};

/// Finance domain resource provider
///
/// Provides resources like:
/// - finance://expenses/{user_id}/summary
/// - finance://budgets/{department}/current
/// - finance://invoices/{invoice_id}
pub struct FinanceResourceProvider {
    db: Arc<DatabaseFoundation>,
}

#[async_trait]
impl DynamicResourceProvider for FinanceResourceProvider {
    fn templates(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                uri_template: "finance://expenses/{user_id}/summary".to_string(),
                name: "Expense Summary".to_string(),
                Some("Monthly expense summary for a user".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceTemplate {
                uri_template: "finance://budgets/{department}/current".to_string(),
                name: "Department Budget".to_string(),
                Some("Current budget status for a department".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            ResourceTemplate {
                uri_template: "finance://invoices/{invoice_id}".to_string(),
                name: "Invoice Details".to_string(),
                Some("Detailed invoice information".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    async fn fetch(
        &self,
        uri: &str,
        params: UriParams,
        _context: RequestContext,
    ) -> Result<ReadResourceResult> {
        let content = if uri.contains("/expenses/") && uri.contains("/summary") {
            let user_id = params.get("user_id").ok_or_else(|| {
                pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "Missing user_id")
            })?;
            self.get_expense_summary(user_id).await?
        } else if uri.contains("/budgets/") {
            let department = params.get("department").ok_or_else(|| {
                pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "Missing department")
            })?;
            self.get_budget_status(department).await?
        } else if uri.contains("/invoices/") {
            let invoice_id = params.get("invoice_id").ok_or_else(|| {
                pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "Missing invoice_id")
            })?;
            self.get_invoice_details(invoice_id).await?
// ... truncated, 47 more lines in source book ...
```

```rust
/// Pattern 1: Orchestration layer handles cross-domain communication
/// (Preferred - see ch19-03-orchestration.md)

/// Pattern 2: Domain exposes limited interface for other domains
pub struct FinanceDomainPublicApi {
    server: Arc<FinanceDomainServer>,
}

impl FinanceDomainPublicApi {
    /// Check if user has any pending expense approvals
    /// Called by HR domain during offboarding
    pub async fn has_pending_expenses(&self, user_id: &str) -> Result<bool> {
        // Minimal interface - just yes/no, no details
        let result = self.server.db.query(
            "SELECT COUNT(*) as count FROM expenses WHERE user_id = $1 AND status = 'pending'",
            &[user_id],
        ).await?;

        let count: i64 = result.first()
            .and_then(|r| r.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(count > 0)
    }
}

/// Pattern 3: Event-based communication (advanced)
/// Domain publishes events, other domains subscribe
pub enum FinanceDomainEvent {
    ExpenseApproved { expense_id: String, user_id: String, amount_cents: i64 },
    BudgetExceeded { department: String, overage_cents: i64 },
    InvoicePaid { invoice_id: String, amount_cents: i64 },
}
```

```rust
impl FinanceDomainServer {
    /// Create a discovery resource that describes domain capabilities
    fn create_discovery_resource(&self) -> StaticResource {
        let capabilities = serde_json::json!({
            "domain": "finance",
            "version": "1.0.0",
            "description": "Finance domain tools for expense management, invoicing, and budgets",
            "tools": [
                {
                    "name": "create_expense",
                    "description": "Submit a new expense report",
                    "requires_role": "employee"
                },
                {
                    "name": "approve_expense",
                    "description": "Approve or reject an expense report",
                    "requires_role": "manager"
                },
                {
                    "name": "generate_invoice",
                    "description": "Generate an invoice for a customer",
                    "requires_role": "finance_admin"
                }
            ],
            "resources": [
                "finance://expenses/{user_id}/summary",
                "finance://budgets/{department}/current",
                "finance://invoices/{invoice_id}"
            ],
            "contact": "finance-platform@company.com"
        });

        StaticResource::new_json(
            "finance://discovery",
            capabilities,
        ).with_description("Finance domain capabilities and available tools")
    }
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Mock foundation for testing
    struct MockAuthFoundation;

    impl MockAuthFoundation {
        async fn validate_token(&self, token: &str) -> Result<AuthenticatedUser> {
            match token {
                "employee_token" => Ok(AuthenticatedUser {
                    id: "emp123".to_string(),
                    email: "employee@company.com".to_string(),
                    roles: vec!["employee".to_string()],
                    department: "engineering".to_string(),
                }),
                "manager_token" => Ok(AuthenticatedUser {
                    id: "mgr456".to_string(),
                    email: "manager@company.com".to_string(),
                    roles: vec!["employee".to_string(), "manager".to_string()],
                    department: "engineering".to_string(),
                }),
                _ => Err(pmcp::Error::protocol(
                    pmcp::ErrorCode::INVALID_PARAMS,
                    "Invalid token",
                )),
            }
        }
    }

    #[test]
    fn expense_amount_validation() {
        // Valid amounts
        assert!(validate_expense_amount(100).is_ok());
        assert!(validate_expense_amount(1_000_000).is_ok());

        // Invalid amounts
        assert!(validate_expense_amount(0).is_err());
        assert!(validate_expense_amount(-100).is_err());

        assert!(validate_expense_amount(1_000_001).is_err());
    }

    #[test]
    fn category_restrictions() {
        let employee = AuthenticatedUser {
            id: "emp".to_string(),
            email: "emp@co.com".to_string(),
            roles: vec!["employee".to_string()],
            department: "eng".to_string(),
        };

        let manager = AuthenticatedUser {
            id: "mgr".to_string(),
            email: "mgr@co.com".to_string(),
            roles: vec!["employee".to_string(), "manager".to_string()],
            department: "eng".to_string(),
        };

        // Employees can create travel expenses
// ... truncated, 7 more lines in source book ...
```

(table/list, cells in reading order)
- Summary
- Aspect
- Best Practice
- Ownership
- One team owns each domain server
- Boundaries
- Clear documentation of what's in/out of scope
- Aspect
- Best Practice
- Foundations
- Compose, don't duplicate foundation logic
- Types
- Strongly-typed domain-specific inputs/outputs
- Business Rules
- Domain logic separate from infrastructure
- Resources
- Discovery
- Testing
- Dynamic providers for parameterized resources
- Expose capabilities for AI client discovery
- Mock foundations, test domain logic in isolation


### Orchestration Patterns

Orchestration enables complex workflows that span multiple domains. When a task requires coordination across HR, Finance, and Engineering (like employee onboarding), orchestration servers tie everything together.


### When to Use Orchestration

```text
                  Orchestration vs Direct Calls
Direct AI-to-Tools (without orchestration):
AI Client
      ▶ HR Server: create_employee()                  ▶ Step 1
      ▶ Finance Server: create_payroll_account()      ▶ Step 2
      ▶ Engineering Server: create_github_access()    ▶ Step 3
      ▶ IT Server: provision_laptop()                 ▶ Step 4
Problems:
• AI must know correct order
• No rollback if step 3 fails
• Multiple round trips (slow)
• AI might skip steps or call in wrong order
With Orchestration:
AI Client
      ▶ Orchestration Server: onboard_employee()
              ▶ HR Server: create_employee()
                      ▶ Store employee_id for later steps
```

```text
              ▶ Finance Server: create_payroll_account()
                      ▶ Uses employee_id from step 1
              ▶ Engineering Server: create_github_access()
              ▶ IT Server: provision_laptop()
Benefits:
[ok] Single tool call for AI
[ok] Guaranteed execution order
[ok] Data flows between steps automatically
[ok] Single round trip
[ok] Deterministic, testable
```


### PMCP Workflow System

PMCP provides a workflow system for building multi-step orchestrations with automatic data binding between steps.

```rust
    dsl::{constant, field, from_step, prompt_arg},
    InternalPromptMessage, SequentialWorkflow, ToolHandle, WorkflowStep,
};
use serde_json::json;

/// Create an employee onboarding workflow
fn create_onboarding_workflow() -> SequentialWorkflow {
    SequentialWorkflow::new(
        "onboard_employee",
        "Complete employee onboarding across all systems",
    )
    // Define required inputs
    .argument("employee_name", "Full name of the employee", true)
    .argument("department", "Department to join", true)
    .argument("role", "Job role/title", true)
    .argument("manager_id", "ID of the reporting manager", true)
    .argument("start_date", "Start date (YYYY-MM-DD)", true)

    // Add system instructions for the AI
    .instruction(InternalPromptMessage::system(
        "Execute employee onboarding workflow. All steps are mandatory."
    ))

    // Step 1: Create employee record in HR system
    .step(
        WorkflowStep::new("create_employee", ToolHandle::new("hr_create_employee"))
            .arg("name", prompt_arg("employee_name"))
            .arg("department", prompt_arg("department"))
            .arg("role", prompt_arg("role"))
            .arg("manager_id", prompt_arg("manager_id"))
            .arg("start_date", prompt_arg("start_date"))
            .bind("employee_record")  // Store output for later steps
    )

    // Step 2: Create payroll account using employee_id from step 1

    .step(
        WorkflowStep::new("setup_payroll", ToolHandle::new("finance_create_payroll"))
            .arg("employee_id", field("employee_record", "employee_id"))  // Extract from step 1
            .arg("department", prompt_arg("department"))
            .bind("payroll_record")
    )

    // Step 3: Create GitHub access
    .step(
        WorkflowStep::new("github_access", ToolHandle::new("eng_create_github_user"))
            .arg("employee_id", field("employee_record", "employee_id"))
            .arg("email", field("employee_record", "email"))
            .arg("team", prompt_arg("department"))
            .bind("github_record")
    )

    // Step 4: Provision laptop
    .step(
        WorkflowStep::new("provision_laptop", ToolHandle::new("it_provision_laptop"))
            .arg("employee_id", field("employee_record", "employee_id"))
            .arg("department", prompt_arg("department"))
            .arg("start_date", prompt_arg("start_date"))
            .bind("laptop_record")
    )
// ... truncated, 12 more lines in source book ...
```

(table/list, cells in reading order)
- DSL Helpers
- The workflow DSL provides helpers for binding data between steps:
- Helper
- Purpose
- Example

```text
prompt_arg("name")
```


### Reference workflow input argument

```text
arg("email", prompt_arg("employee_email"))

from_step("binding")
```


### Reference entire output of a step

```text
arg("data", from_step("employee_record"))
```

(table/list, cells in reading order)
- field("binding",
- "field")
- Extract specific field from step output
- arg("id", field("employee_record",
- "employee_id"))

```text
constant(value)
```


### Provide a constant value

```text
arg("template", constant(json!("welcome")))
```


### Server-Side Execution

Workflows execute server-side, not client-side.

```rust
use pmcp::{Result, Server};

/// Create orchestration server with workflows
fn create_orchestration_server() -> Result<Server> {
    Server::builder()
        .name("orchestration-server")
        .version("1.0.0")
        // Register the tools that workflows use
        .tool_typed("hr_create_employee", hr_create_employee_handler)
        .tool_typed("finance_create_payroll", finance_create_payroll_handler)
        .tool_typed("eng_create_github_user", eng_create_github_handler)
        .tool_typed("it_provision_laptop", it_provision_laptop_handler)
        .tool_typed("comms_send_email", comms_send_email_handler)
        // Register workflows as prompts
        .prompt_workflow(create_onboarding_workflow())?
        .prompt_workflow(create_offboarding_workflow())?
        .build()
}
```


### Execution Flow

```text
                  Workflow Execution Flow
Client Request:
{
  "method": "prompts/get",
  "params": {
    "name": "onboard_employee",
    "arguments": {
      "employee_name": "Alice Smith",
      "department": "engineering",
      "role": "Software Engineer",
      "manager_id": "mgr-123",
      "start_date": "2024-02-01"
    }
  }
}
Server-Side Execution:
Step 1: hr_create_employee
        Input: {name: "Alice Smith", department: "engineering", ...}
        Output: {employee_id: "emp-456", email: "alice@company.com"}
        → Stored as "employee_record"
Step 2: finance_create_payroll
        Input: {employee_id: "emp-456", department: "engineering"}
        Output: {payroll_id: "pay-789", status: "active"}
        → Stored as "payroll_record"
Step 3: eng_create_github_user
```

```text
        Input: {employee_id: "emp-456", email: "alice@company.com"}
        Output: {username: "asmith", access_level: "developer"}
        → Stored as "github_record"
Step 4: it_provision_laptop
        Input: {employee_id: "emp-456", start_date: "2024-02-01"}
        Output: {tracking_number: "FX123456", eta: "2024-01-30"}
        → Stored as "laptop_record"
Step 5: comms_send_email
        Input: {to: "alice@company.com", github: "asmith", ...}
        Output: {sent: true, message_id: "msg-abc"}
        → Stored as "email_result"
Server Response (conversation trace):
[
  {role: "user", content: "Onboard Alice Smith to engineering..."},
  {role: "assistant", content: "Executing 5-step onboarding..."},
  {role: "assistant", content: "Calling hr_create_employee..."},
  {role: "user", content: "Tool result: {employee_id: 'emp-456'...}"},
  ... (more messages for each step)
]
```


### Real-World Workflow Example

Here's a complete code review workflow from  examples/53_typed_tools_workflow_integration.rs :

```rust
use pmcp::server::workflow::dsl::*;
use pmcp::server::workflow::{SequentialWorkflow, ToolHandle, WorkflowStep};
use pmcp::{Result, Server};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// Tool Definitions
// ============================================================================

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct AnalyzeCodeInput {
    code: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_depth")]
    depth: u8,
}

fn default_language() -> String { "rust".to_string() }
fn default_depth() -> u8 { 2 }

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct ReviewCodeInput {
    analysis: String,
    focus: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct FormatCodeInput {
    code: String,
    issues: Vec<String>,
}

// Tool implementations
async fn analyze_code(input: AnalyzeCodeInput, _extra: RequestHandlerExtra) -> Result<Value> {
    Ok(json!({
        "language": input.language,

        "depth": input.depth,
        "lines_of_code": input.code.lines().count(),
        "issues_found": 3,
        "complexity_score": 7.5,
        "analysis_summary": format!(
            "Analyzed {} lines of {} code. Found 3 potential issues.",
            input.code.lines().count(),
            input.language
        ),
        "issue_details": [
            "Function has high cyclomatic complexity",
            "Missing error handling",
            "Consider using Result<T> instead of panicking"
        ]
    }))
}

async fn review_code(input: ReviewCodeInput, _extra: RequestHandlerExtra) -> Result<Value> {
    Ok(json!({
        "review_summary": format!("Reviewed with focus on: {}", input.focus.join(", ")),
// ... truncated, 87 more lines in source book ...
```

```rust
fn create_workflow() -> SequentialWorkflow {
    let workflow = SequentialWorkflow::new("my_workflow", "Description")
        .argument("input", "Required input", true)
        .step(
            WorkflowStep::new("step1", ToolHandle::new("tool1"))
                .arg("data", prompt_arg("input"))
                .bind("result1")
        )
        .step(
            WorkflowStep::new("step2", ToolHandle::new("tool2"))
                .arg("prev", field("result1", "output"))  // References step1 output
                .bind("result2")
        );

    // Validate before registering
    workflow.validate().expect("Workflow should be valid");

    workflow
}
```

(table/list, cells in reading order)
- Validation Checks
- Check
- Error Example
- Undefined binding
- field("nonexistent", "field")  - binding doesn't exist
- Missing argument
- prompt_arg("missing")  - argument not declared
- Duplicate binding
- Two steps with same  .bind("name")
- Empty workflow
- No steps defined
- Error Handling in Workflows
- If a step fails, the workflow stops and returns the error:

```rust
// Step that might fail
.step(
    WorkflowStep::new("risky_operation", ToolHandle::new("external_api"))
        .arg("data", from_step("previous_result"))
        .bind("api_result")
        // If external_api fails, workflow stops here
        // Client receives error with context about which step failed
)
```


### For advanced error handling, implement retry logic in the tool itself:

```rust
async fn external_api_with_retry(input: ApiInput, _extra: RequestHandlerExtra) -> Result<Value> {
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;
        match call_external_api(&input).await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_attempts => {
                tracing::warn!(attempt = attempts, error = %e, "Retrying...");
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

(table/list, cells in reading order)
- When NOT to Use Orchestration
- Orchestration adds complexity. Avoid it when:
- Scenario
- Better Approach
- Single tool call
- Direct tool call
- Steps are independent
- Parallel direct calls
- AI needs to make decisions
- Let AI orchestrate
- Dynamic step order
- AI-driven workflow
- User interaction between steps
- Multiple client requests
- Summary
- Concept
- Purpose
- SequentialWorkflow
- Define multi-step workflows
- WorkflowStep
- Individual step with tool and arguments

```text
bind()
```


### Store step output for later steps

```text
prompt_arg()
```


### Reference workflow input

```text
field()

from_step()
```

(table/list, cells in reading order)
- Extract field from previous step output
- Reference entire step output
- Server-side execution
- Single request, deterministic execution

Orchestration is powerful for complex, multi-domain workflows. Use it when you need guaranteed execution order, data flow between steps, and single-request completion of multi-step processes.
