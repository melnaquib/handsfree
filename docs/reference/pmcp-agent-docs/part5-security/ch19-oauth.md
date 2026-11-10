---
chapter: 19
title: "OAuth for MCP"
part: "Part V: Enterprise Security"
use_when: "Adding OAuth 2.0: middleware, JWT validation, scopes, multi-tenant, auth errors, auth tests."
suggested_prerequisites: [ch02]
source_kb: 110
extracted_kb: 57
code_blocks: 61
quality: auto-extracted
---

# Ch 19: OAuth for MCP

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### OAuth for MCP

Enterprise MCP servers must authenticate users properly.


### What you'll learn:

Why API keys are insufficient for production How OAuth 2.0 flow works with MCP Implementing JWT validation in your server Scope-based authorization for tools Testing authenticated endpoints


### Why OAuth, Not API Keys

API keys seem simple—generate a secret, include it in requests, check it on the server.


### Many tutorials show API key authentication:

```bash
# DON'T DO THIS in production
curl -H "X-API-Key: sk_live_abc123" http://mcp-server/tools
```

(table/list, cells in reading order)
- Problems with API keys:
- Issue
- Impact
- No user identity
- Can't audit who did what
- OAuth 2.0 solves these:
- Issue
- Impact
- Hard to rotate
- Changing keys breaks all clients
- No granular permissions
- Key has full access or none
- Easy to leak
- No federation
- Shows up in logs, git history
- Can't integrate with enterprise IdP
- Feature
- Benefit
- User identity
- JWT contains user info
- Token expiration
- Automatic rotation
- Scopes
- Fine-grained permissions
- Standard protocol
- Works with existing IdPs
- Audit trail
- Every request tied to a user
- OAuth 2.0 for MCP: Quick Overview

OAuth 2.0 separates authentication (verifying identity) from your application. This means you never handle passwords—a significant security advantage.

```text
  MCP                 MCP                Identity
  Client         ▶    Server         ▶   Provider
(ChatGPT)    ◀       (Your App)  ◀      (Cognito)
        1. Connect
                        ▶
                            2. Redirect to
      ◀                        IdP login
        3. User logs in
                                            ▶
        4. Auth code
      ◀
        5. Exchange for
           access token
                        ▶   6. Validate
                                            ▶
                          ◀
        7. Tool calls
           with token
                        ▶   8. Verify JWT
      ◀
```


### Adding OAuth to Your Server

Adding OAuth involves two parts: configuring your Identity Provider (outside your code) and adding validation middleware to your server (in your code).


### Using cargo pmcp

The easiest way to add OAuth—this generates the boilerplate configuration and middleware:

```bash
# Initialize OAuth with Cognito
cargo pmcp deploy init --target pmcp-run --oauth cognito

# Or with Auth0
cargo pmcp deploy init --target pmcp-run --oauth auth0
```


### Manual Setup

For more control or custom IdP configurations, add OAuth manually.

```rust
// src/main.rs
use pmcp::prelude::*;
use pmcp::server::auth::{JwtValidator, ValidationConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure JWT validation
    let jwt_config = ValidationConfig::cognito(
        "us-east-1",           // AWS region
        "us-east-1_xxxxxx",    // User pool ID
        "your-client-id",      // App client ID
    );

    let validator = JwtValidator::new()
        .with_config(jwt_config);

    // Build server with OAuth middleware
    let server = ServerBuilder::new("secure-server", "1.0.0")
        .with_auth(validator)
        .with_tool(tools::SecureTool)
        .build()?;

    server_common::create_http_server(server)
        .serve("0.0.0.0:3000")
        .await
}
```


### The OAuth Middleware

The middleware runs before every request handler. If validation fails, the request is rejected immediately—your tool code never runs.

```rust
use pmcp::server::auth::{AuthContext, ServerHttpMiddleware};

pub struct OAuthMiddleware {
    validator: JwtValidator,
}

#[async_trait]
impl ServerHttpMiddleware for OAuthMiddleware {
    async fn on_request(
        &self,
        request: &HttpRequest,
        context: &mut ServerHttpContext,
    ) -> Result<Option<HttpResponse>> {
        // Extract Bearer token
        let token = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| PmcpError::unauthorized("Missing authorization header"))?;

        // Validate JWT
        let claims = self.validator
            .validate(token)
            .await
            .map_err(|e| PmcpError::unauthorized(format!("Invalid token: {}", e)))?;

        // Store user info in context for tools to access
        context.set_auth(AuthContext::from_claims(claims));

        Ok(None)  // Continue to handler
    }
}
```


### Accessing User Info in Tools

Once authentication succeeds, your tools can access user information through the context.

```rust
#[derive(TypedTool)]
#[mcp_tool(name = "get_my_data", description = "Get data for the authenticated user")]
pub struct GetMyData;

impl GetMyData {
    pub async fn run(
        &self,
        _input: (),
        context: &ToolContext,
    ) -> Result<UserData> {
        // Get authenticated user from context
        let auth = context.auth()
            .ok_or_else(|| PmcpError::unauthorized("Not authenticated"))?;

        let user_id = auth.user_id();
        let email = auth.email();
        let scopes = auth.scopes();

        // Check for required scope
        if !scopes.contains(&"read:data".to_string()) {
            return Err(PmcpError::forbidden("Missing read:data scope"));
        }

        // Fetch user's data
        let data = self.database.get_user_data(user_id).await?;

        Ok(data)
    }
}
```


### Token Validation

JWT (JSON Web Token) validation is the core of OAuth security.


### JWT Structure

A JWT has three parts (header, payload, signature), each Base64-encoded and separated by dots.

```json
{
  "header": {
    "alg": "RS256",
    "kid": "key-id-123"
  },
  "payload": {
    "sub": "user-123",
    "email": "user@example.com",
    "scope": "read:tools write:tools",
    "iss": "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_xxx",
    "aud": "client-id",
    "exp": 1699999999,
    "iat": 1699996399
  },
  "signature": "..."
}
```


### Validation Steps

Each validation step catches a different type of attack or misconfiguration.

```rust
impl JwtValidator {
    pub async fn validate(&self, token: &str) -> Result<Claims> {
        // 1. Decode header to get key ID
        let header = decode_header(token)?;
        let kid = header.kid.ok_or("Missing key ID")?;

        // 2. Fetch JWKS from IdP (cached)
        let jwks = self.get_jwks().await?;
        let key = jwks.find(&kid).ok_or("Key not found")?;

        // 3. Verify signature
        let claims: Claims = decode(token, &key, &self.validation)?;

        // 4. Check expiration
        if claims.exp < current_time() {
            return Err("Token expired");
        }

        // 5. Check issuer
        if claims.iss != self.config.issuer {
            return Err("Invalid issuer");
        }

        // 6. Check audience
        if claims.aud != self.config.audience {
            return Err("Invalid audience");
        }

        Ok(claims)
    }
}
```


### Scope-Based Authorization

Scopes are permission labels attached to tokens.


### Common scope patterns:

read:resource  /  write:resource  — Read/write separation admin:resource  — Administrative operations resource:action  — Fine-grained actions (e.g.,  customers:delete )

```rust
#[derive(TypedTool)]
#[mcp_tool(
    name = "delete_customer",
    description = "Delete a customer record",
    annotations(destructive = true)
)]
pub struct DeleteCustomer;

impl DeleteCustomer {
    pub async fn run(&self, input: DeleteInput, context: &ToolContext) -> Result<()> {
        let auth = context.auth().ok_or(PmcpError::unauthorized("Not authenticated"))?;

        // Require admin scope for destructive operations
        if !auth.has_scope("admin:customers") {
            return Err(PmcpError::forbidden(
                "This operation requires admin:customers scope"
            ));
        }

        // Log the action for audit
        tracing::info!(
            user_id = %auth.user_id(),
            customer_id = %input.customer_id,
            "Deleting customer"
        );

        self.database.delete_customer(&input.customer_id).await?;

        Ok(())
    }
}
```


### Multi-Tenant Configuration

Multi-tenant MCP servers serve multiple organizations, each with their own IdP. The server must validate tokens from any of these issuers while ensuring users from one tenant can't access another tenant's data.

```rust
pub struct MultiTenantValidator {
    validators: HashMap<String, JwtValidator>,
}

impl MultiTenantValidator {
    pub async fn validate(&self, token: &str) -> Result<Claims> {
        // Decode without verification to get issuer
        let unverified = decode_unverified(token)?;
        let issuer = &unverified.iss;

        // Find validator for this tenant
        let validator = self.validators
            .get(issuer)
            .ok_or_else(|| PmcpError::unauthorized("Unknown issuer"))?;

        // Validate with tenant-specific config
        validator.validate(token).await
    }
}
```


### Error Handling

OAuth errors must be precise—clients need to know whether to retry with a new token (401) or inform the user they lack permissions (403).


### 401 Unauthorized — "I don't know who you are"

Missing token, expired token, invalid signature Client should re-authenticate


### 403 Forbidden — "I know who you are, but you can't do this"

Valid token but insufficient scopes Client should inform user, not retry

```rust
// 401 Unauthorized - missing or invalid credentials
PmcpError::unauthorized("Invalid or expired token")

// 403 Forbidden - valid credentials but insufficient permissions
PmcpError::forbidden("Insufficient scope for this operation")

// Include WWW-Authenticate header for 401
HttpResponse::unauthorized()
    .header("WWW-Authenticate", "Bearer realm=\"mcp\", error=\"invalid_token\"")
```


### Testing OAuth

Testing authenticated endpoints is tricky—you don't want tests depending on a real IdP.


### Testing strategies:

Unit tests: Mock validator with configurable users/scopes Integration tests: Test against a local IdP (like Keycloak in Docker) E2E tests: Test against your staging IdP with test accounts


### Mock Validator for Tests

The mock validator lets you test any authentication scenario without real tokens:

```rust
#[cfg(test)]
mod tests {
    use pmcp::server::auth::MockValidator;

    #[tokio::test]
    async fn test_requires_authentication() {
        let server = build_test_server().await;

        // Without token - should fail
        let response = server.call_tool("get_my_data", json!({})).await;
        assert_eq!(response.error.code, -32001);  // Unauthorized

        // With valid token - should succeed
        let response = server
            .with_auth(MockValidator::user("test-user"))
            .call_tool("get_my_data", json!({}))
            .await;
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_requires_admin_scope() {
        let server = build_test_server().await;

        // With regular user - should fail
        let response = server
            .with_auth(MockValidator::user("regular-user"))
            .call_tool("delete_customer", json!({"id": "123"}))
            .await;
        assert_eq!(response.error.code, -32003);  // Forbidden

        // With admin - should succeed
        let response = server
            .with_auth(MockValidator::admin("admin-user"))
            .call_tool("delete_customer", json!({"id": "123"}))
            .await;
        assert!(response.error.is_none());

    }
}
```


### Security Best Practices

These practices come from real-world OAuth incidents.

Always validate tokens server-side - Don't trust client claims. Implement token refresh - Don't force users to re-authenticate 4. Use HTTPS only - Never send tokens over HTTP 6.


### Practice Ideas

These informal exercises help reinforce the concepts.

Implement scope checking: Create tools that require different scopes

When you add MCP servers to the mix, you have two choices:

```bash
# Developer creates an API key in a dashboard
# Key: sk_live_abc123def456...

# Client includes it in every request
curl -H "X-API-Key: sk_live_abc123def456" \
  https://mcp-server.example.com/mcp
```


### Problem 1: No User Identity

```text
                  API Key Authentication
Request 1:
   X-API-Key: sk_live_abc123
   Tool: delete_customer
   Args: { "id": "cust_789" }
Who made this request?
   ❓ Could be Alice from accounting
   ❓ Could be Bob from engineering
   ❓ Could be an attacker who found the key
   ❓ Could be an automated system
   Answer: We have no idea
Audit log:
"Customer cust_789 deleted by... someone with API key abc123"
```

When something goes wrong, you can't answer "who did it?" The API key identifies the application, not the person.


### Problem 2: No Granular Permissions

```rust
// With API keys, you typically have two options:

// Option 1: Full access
if request.api_key == valid_key {
    // User can do EVERYTHING
    allow_all_operations();
}

// Option 2: Separate keys per feature (unmanageable)
let read_key = "sk_read_abc123";
let write_key = "sk_write_def456";
let admin_key = "sk_admin_ghi789";
// Now you need to manage 3x the keys...
// And what about per-resource permissions?
```


### Real enterprise scenarios require:

User A can read customer data but not modify it User B can modify their own team's data User C has admin access but only during business hours User D can access everything except financial records

API keys can't express these nuances.


### Problem 3: Key Rotation is Painful

```text
                  API Key Rotation Nightmare
Day 0: Key potentially compromised
Day 1-7: Security team investigates
Day 8: Decision to rotate key
Day 9-14: Find all places using the key
  • Production server configs
  • CI/CD pipelines
  • Developer machines
  • Third-party integrations
  • Mobile apps (oh no, need app store update)
  • Partner systems (need to coordinate)
Day 15-30: Coordinate the change
  • Update all systems simultaneously
  • Some systems break anyway
  • Rollback, fix, retry
Day 31: Finally rotated
  • Attacker had access for a full month
```


### Problem 4: Keys Leak Easily

```bash
# Leakage vectors for API keys:

# 1. Git history (most common)
git log --all -p | grep "sk_live_"

# 2. Error logs
```


### [ERROR] Failed to connect: auth failed with key sk_live_abc123

```bash
# 3. Browser developer tools
fetch('/api/data', { headers: { 'X-API-Key': 'sk_live_abc123' }})

# 4. Shared documentation
curl -H "X-API-Key: sk_live_abc123" https://...  # "Replace with your key"

# 5. Environment variable dumps
```


### env | grep API  # Often logged during debugging

```text
# 6. Configuration backups
```


### cat /backup/2024/config.json | grep key

GitHub continuously scans for leaked API keys.


### Problem 5: No Federation (The Biggest Problem)

This is the deal-breaker for enterprises.

You end up with shadow permissions that don't appear in your corporate access reviews.

```text
             Enterprise Identity Architecture
What enterprises have:
   Active Directory / Entra ID / Okta / etc.
   • Single source of truth for users
   • Group memberships
   • Role assignments
   • Automatic deprovisioning when employees leave
   • Compliance and audit requirements
What API keys need:
   Separate key management
   • Manual provisioning
   • Manual deprovisioning (often forgotten!)
   • No connection to corporate identity
   • Separate audit trail
   • Yet another credential to manage
Result: Former employees still have valid API keys
```


### OAuth 2.0: The Enterprise Solution

OAuth 2.0 addresses every API key problem while integrating seamlessly with your existing infrastructure:


### User Identity

```text
// JWT token payload
{
  "sub": "auth0|user123",
  "email": "alice@company.com",
  "name": "Alice Smith",
  "groups": ["engineering", "data-team"],
  "roles": ["developer", "data-analyst"],
  "iat": 1699996399,
  "exp": 1700000000
}
```

Every request is tied to a specific user.


### Granular Permissions (Scopes)

```json
{
  "scope": "read:customers write:own-data admin:reports"
}
```

Scopes define exactly what operations a user can perform.


### Automatic Token Rotation

```text
                  OAuth Token Lifecycle
Access Token
   Lifetime: 1 hour (typical)
   Used for API requests
   Automatically expires
Refresh Token
   Lifetime: 30 days (typical)
   Used to get new access tokens
   Can be revoked immediately
Key rotation happens automatically:
• Signing keys rotate on the IdP
• Clients get new tokens transparently
• No coordinated deployment needed
```

```text
   - Access tokens expire in ~1 hour
   - Even if leaked, damage is limited
```


### 2. Bound to specific client

```text
   - Tokens include client_id
   - Can't be used from other applications
```


### 3. Revocable

```text
   - Revoke user's refresh token
   - All their sessions end immediately
```


### 4. Not stored in code

```text
   - Tokens are obtained at runtime
   - Never committed to git
```


### 5. Automatic refresh

```text
   - No reason to store long-lived credentials
```


### Full Federation: One Source of Truth

This is the key advantage for enterprises.

```text
                  Federated Identity Flow
Corporate IdP (Entra ID)
       SAML/OIDC Federation
OAuth Provider (Auth0/Cognito)
       JWT with corporate identity
MCP Server
       User identity preserved
Audit Log:
"alice@company.com (Engineering) called delete_customer"
When Alice leaves the company:
1. IT disables her in Entra ID
2. Her OAuth tokens stop working immediately
3. No manual key revocation needed
```

(table/list, cells in reading order)
- Comparison Summary
- Aspect
- API Keys
- OAuth 2.0
- User identity
- Permissions
- Rotation
- Application only
- Full user info
- All or nothing
- Granular scopes
- Manual, painful
- Automatic
- Aspect
- API Keys
- OAuth 2.0
- Leak impact
- Revocation
- Long-term access
- 1 hour max
- Find and delete
- Instant, central
- Enterprise IdP
- No integration
- Full federation
- Compliance
- Standard
- Difficult
- Proprietary
- Built-in audit trail
- Industry standard
- Provider options
- Build your own

(table/list, cells in reading order)
- Permission management
- Duplicate in every app
- Single source of truth
- When API Keys Are Still Okay
- API keys aren't always wrong. They're acceptable for:


### But for MCP servers that:

Handle sensitive enterprise data Need user-level audit trails Must integrate with corporate identity Require granular permissions Face compliance requirements

No identity - Can't audit who did what 2.


### OAuth 2.0 solves all of these with:

1.


### OAuth 2.0 Fundamentals

This chapter covers the OAuth 2.0 concepts essential for implementing authentication in MCP servers.

Good news for MCP developers: You don't need to build token management from scratch.

```text
                   OAuth 2.0 Roles for MCP
Resource Owner (User)
   The person using the AI assistant
   Grants permission for AI to access MCP tools
Client (MCP Client)
   Claude Code, ChatGPT, Cursor, or custom application
   Securely stores tokens (locally or server-side)
   Automatically refreshes tokens before expiration
   Sends access token with every MCP request
Authorization Server (Identity Provider)
   Cognito, Auth0, Entra ID, Okta
   Authenticates users
   Issues access tokens
Resource Server (Your MCP Server)
   Validates access tokens
   Enforces scopes
   Provides tools and resources
```


### Grant Types

OAuth 2.0 defines several grant types.

- Authorization Code Grant (Recommended)

```text
               Authorization Code Flow
1. User clicks "Connect to MCP Server"
2. Client redirects to Authorization Server
   GET /authorize?
     response_type=code&
     client_id=abc&
     redirect_uri=https://client/callback&
     scope=read:tools write:tools&
     state=random123
3. User logs in and consents
4. Authorization Server redirects back with code
   GET https://client/callback?
     code=AUTH_CODE_HERE&
     state=random123
5. Client exchanges code for tokens (server-side)
   POST /token
     grant_type=authorization_code&
     code=AUTH_CODE_HERE&
     client_id=abc&
     client_secret=xyz&
     redirect_uri=https://client/callback
6. Authorization Server returns tokens
   {
     "access_token": "eyJhbGc...",
     "refresh_token": "def456...",
     "token_type": "Bearer",
```

```text
     "expires_in": 3600
   }
7. Client calls MCP Server with access token
   POST /mcp
     Authorization: Bearer eyJhbGc...
```

```rust
// Server-to-server authentication
let token_response = reqwest::Client::new()
    .post("https://auth.example.com/oauth/token")
    .form(&[
        ("grant_type", "client_credentials"),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("scope", "read:tools"),
    ])
    .send()
    .await?
    .json::<TokenResponse>()
    .await?;

// Use the token
let mcp_response = client
    .post("https://mcp.example.com/mcp")
    .bearer_auth(&token_response.access_token)
    .json(&mcp_request)
    .send()
    .await?;
```


### JSON Web Tokens (JWT)

OAuth 2.0 access tokens are typically JWTs.


### JWT Structure

```text
                      JWT Structure
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.
                       HEADER (Base64)
eyJzdWIiOiJ1c2VyMTIzIiwiZW1haWwiOiJhbGljZUBjby5jb20iLCJzY29w...
                       PAYLOAD (Base64)
SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
                       SIGNATURE
```


### Header

```json
{
  "alg": "RS256",      // Signing algorithm
  "typ": "JWT",        // Token type
  "kid": "key-123"     // Key ID for signature verification
}
```


### Common algorithms:

RS256 - RSA signature with SHA-256 (recommended) RS384 - RSA with SHA-384 ES256 - ECDSA with P-256 curve HS256 - HMAC with SHA-256 (avoid for access tokens)


### Payload (Claims)

```json
{
  // Standard claims
  "iss": "https://auth.example.com/",           // Issuer
  "sub": "auth0|user123",                        // Subject (user ID)
  "aud": "https://mcp.example.com",             // Audience
  "exp": 1700000000,                            // Expiration time
  "iat": 1699996400,                            // Issued at
  "nbf": 1699996400,                            // Not before

  // Common custom claims
  "email": "alice@company.com",
  "name": "Alice Smith",
  "scope": "read:tools write:tools",
  "permissions": ["read:customers", "write:orders"],
  "org_id": "org_abc123",
  "roles": ["developer", "data-analyst"]
}
```

(table/list, cells in reading order)
- Essential Claims for MCP
- Claim
- sub
- Purpose
- Example
- User identifier
- `auth0
- Claim
- Purpose
- Example
- iss
- aud
- exp
- scope
- email
- Token issuer (IdP)

https://cognito...

(table/list, cells in reading order)
- Intended audience
- mcp-server-prod
- Expiration time
- 1700000000
- Granted permissions
- read:tools write:data
- User email (optional)
- alice@co.com
- Scopes and Permissions
- Defining Scopes

```rust
// Scope definitions for an MCP server
pub enum Scope {
    // Tool access
    ReadTools,      // "read:tools" - List and describe tools
    ExecuteTools,   // "execute:tools" - Call tools

    // Resource access
    ReadResources,  // "read:resources" - Read resources
    WriteResources, // "write:resources" - Modify resources

    // Admin operations
    AdminAudit,     // "admin:audit" - View audit logs
    AdminUsers,     // "admin:users" - Manage users
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadTools => "read:tools",
            Self::ExecuteTools => "execute:tools",
            Self::ReadResources => "read:resources",
            Self::WriteResources => "write:resources",
            Self::AdminAudit => "admin:audit",
            Self::AdminUsers => "admin:users",
        }
    }
}
```


### Checking Scopes in Tools

```rust
#[derive(TypedTool)]
#[mcp_tool(name = "execute_query", description = "Run a database query")]
pub struct ExecuteQuery;

impl ExecuteQuery {
    pub async fn run(
        &self,
        input: QueryInput,
        context: &ToolContext,
    ) -> Result<QueryResult> {
        let auth = context.auth()?;

        // Check for required scope
        auth.require_scope("execute:tools")?;

        // For write operations, check additional scope
        if is_write_query(&input.sql) {
            auth.require_scope("write:resources")?;
        }

        // Execute query...
        self.database.execute(&input.sql).await
    }
}
```

```text
admin:audit  → Read audit logs
admin:users  → Manage users
admin:config → Modify configuration
```


### write:*          → Full write access

```text
write:tools  → Execute modifying tools
write:data   → Modify resources
```


### read:*           → Full read access

```text
read:tools   → List and describe tools
read:data    → Read resources
```

```rust
impl AuthContext {
    pub fn has_scope(&self, required: &str) -> bool {
        self.scopes.iter().any(|s| {
            s == required ||
            // Check wildcard: "write:*" matches "write:tools"
            (s.ends_with(":*") && required.starts_with(&s[..s.len()-1]))
        })
    }
}
```


### Understanding Token Lifetimes

If you're new to OAuth, token lifetimes can be confusing.


### Think of it like a building security system:

Access token = Day pass.

```text
                  Token Lifetime Strategy
ACCESS TOKEN (Short-lived: 15-60 minutes)
   [ok] Sent with every API request
   [ok] If leaked, damage limited to minutes/hours
   [ok] Contains user claims (who, what permissions)
   [no] Cannot be revoked (must wait for expiration)
REFRESH TOKEN (Long-lived: 30-90 days)
   [ok] Only sent to the IdP, never to your MCP server
   [ok] Used to get new access tokens silently
   [ok] Can be revoked immediately by administrators
   [ok] Enables "login once, work for weeks" experience
The combination: Security of short-lived tokens +
                 Convenience of long sessions
```

(table/list, cells in reading order)
- What Users Experience
- Day
- Day 1
- What Happens
- User Action Required
- User connects MCP server to Claude Code
- Login once via SSO
- Day 2-89
- Access tokens refresh automatically every hour
- None - seamless
- Day 90
- Refresh token expires
- Login again via SSO

MCP clients like Claude Code, ChatGPT, and Cursor handle all the token refresh logic automatically—users never see it happening.


### Administrator Control: Immediate Revocation

Even though refresh tokens last 90 days, administrators can revoke them instantly:

```text
                  Token Revocation Scenario
Monday 9am:  Employee leaves company
Monday 9:05am: IT disables account in IdP
Monday 9:06am: Employee tries to use MCP server
What happens:
1. Claude Code tries to refresh the access token
2. IdP rejects: "Refresh token revoked"
3. Claude Code prompts for re-authentication
4. Employee can't login (account disabled)
5. Access denied [ok]
Maximum exposure time: Until current access token expires
(typically 15-60 minutes, not 90 days)
```

This is why access tokens are kept short-lived: even if you can't revoke them directly, you limit the damage window to minutes, not days.


### Token Refresh Flow

Access tokens are short-lived by design.

```text
                   Token Refresh Flow
Initial State:
   access_token: eyJhbGc... (expires in 1 hour)
   refresh_token: def456... (expires in 30 days)
When access token expires:
Client → Authorization Server
POST /token
  grant_type=refresh_token&
  refresh_token=def456...&
  client_id=abc&
  client_secret=xyz
Authorization Server → Client
{
  "access_token": "NEW_TOKEN...",
  "refresh_token": "NEW_REFRESH...",  // May be rotated
  "expires_in": 3600
}
Note: Some IdPs rotate refresh tokens on each use
```


### How MCP Clients Store Tokens Securely

You might wonder: "If refresh tokens last 90 days, where are they stored?" MCP clients handle this differently depending on their

```yaml
architecture:
```

(table/list, cells in reading order)
- Client
- Token Storage
- Security Model
- Claude Code
- OS keychain (macOS Keychain, Windows Credential Manager)
- Encrypted, per-user, survives restarts
- Server-side (OpenAI infrastructure)
- User never sees tokens, encrypted at rest
- ChatGPT
- Cursor
- OS keychain
- Custom apps
- Your responsibility
- Same as Claude Code
- Use OS keychain or secure enclave

The important point: Users never need to handle tokens directly.


### Client-Side Token Management (For Custom Implementations)

If you're building a custom MCP client, here's the pattern for automatic token refresh:

```rust
pub struct TokenManager {
    access_token: RwLock<String>,
    refresh_token: RwLock<String>,
    expires_at: RwLock<Instant>,
    client: reqwest::Client,
}

impl TokenManager {
    pub async fn get_valid_token(&self) -> Result<String> {
        // Check if current token is still valid (with buffer)
        let expires_at = *self.expires_at.read().await;
        if Instant::now() + Duration::from_secs(60) < expires_at {
            return Ok(self.access_token.read().await.clone());
        }

        // Token expired or expiring soon, refresh it
        self.refresh().await
    }

    async fn refresh(&self) -> Result<String> {
        let refresh_token = self.refresh_token.read().await.clone();

        let response = self.client
            .post("https://auth.example.com/oauth/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &refresh_token),
                ("client_id", &self.client_id),
            ])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        // Update stored tokens
        *self.access_token.write().await = response.access_token.clone();
        if let Some(new_refresh) = response.refresh_token {
            *self.refresh_token.write().await = new_refresh;
        }

        *self.expires_at.write().await =
            Instant::now() + Duration::from_secs(response.expires_in);

        Ok(response.access_token)
    }
}
```


### PKCE: Proof Key for Code Exchange

For public clients (mobile apps, SPAs), use PKCE to prevent authorization code interception:

```text
                  PKCE Flow
1. Client generates code_verifier (random string)
   code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
2. Client creates code_challenge (SHA256 hash)
   code_challenge = BASE64URL(SHA256(code_verifier))
   = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
3. Authorization request includes challenge
   GET /authorize?
     response_type=code&
     code_challenge=E9Melhoa...&
     code_challenge_method=S256&
     ...
4. Token request includes verifier
   POST /token
     grant_type=authorization_code&
     code=AUTH_CODE&
     code_verifier=dBjftJeZ...&
     ...
5. Server verifies SHA256(verifier) == challenge
   [ok] Only the original client can exchange the code
```

```rust
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;

pub fn generate_pkce() -> (String, String) {
    // Generate random verifier (43-128 characters)
    let verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    // Create challenge (SHA256 + Base64URL)
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}
```

```json
{
  "iss": "https://auth.example.com/",
  "sub": "user123",
  "aud": "client-id",
  "exp": 1700000000,
  "iat": 1699996400,
  "nonce": "random-nonce",

  // OIDC standard claims
  "email": "alice@company.com",
  "email_verified": true,
  "name": "Alice Smith",
  "given_name": "Alice",
  "family_name": "Smith",
  "picture": "https://...",
  "locale": "en-US"
}
```

```text
# Cognito
```

```text
# Auth0
```


### https://your-tenant.auth0.com/.well-known/openid-configuration

```text
# Entra ID
```

https://login.microsoftonline.com/{tenant}/v2.0/.well-known/openid-configuration


### Response includes endpoints and supported features:

```json
{
  "issuer": "https://auth.example.com/",
  "authorization_endpoint": "https://auth.example.com/authorize",
  "token_endpoint": "https://auth.example.com/oauth/token",
  "userinfo_endpoint": "https://auth.example.com/userinfo",
  "jwks_uri": "https://auth.example.com/.well-known/jwks.json",
  "scopes_supported": ["openid", "profile", "email"],
  "response_types_supported": ["code", "token"],
  "token_endpoint_auth_methods_supported": ["client_secret_post", "client_secret_basic"]
}
```

1. Always validate tokens - Never trust client claims 2.

(table/list, cells in reading order)
- Token Lifetimes: What They Mean in Practice
- Token Type
- Recommended Lifetime
- What This Means
- Access Token
- 15-60 minutes

(table/list, cells in reading order)
- Token Type
- Recommended Lifetime
- What This Means
- ID Token
- 5-15 minutes

You don't handle refresh tokens—that's between the MCP client and the IdP.


### Security Checklist

Use HTTPS everywhere Validate token signature Check issuer matches expected Check audience matches your server Check expiration (with clock skew) Use PKCE for public clients Implement token refresh Log authentication events

PKCE - Required for public clients 7.


### Key takeaways for the user experience:

Users authenticate once and work uninterrupted for 30-90 days (refresh token lifetime) MCP clients handle complexity: Claude Code, ChatGPT, Cursor store tokens securely and refresh them automatically Administrators stay in control: Tokens can be revoked instantly, regardless of expiration date Secur


### Key takeaways for MCP server developers:

You only validate access tokens - Refresh handling is the client's job Check every request - Validate signature, issuer, audience, and expiration Use scopes for authorization - They define what each user can do Return proper errors - 401 for invalid tokens, 403 for insufficient permissions


### Token Validation

This chapter covers the practical implementation of JWT token validation in Rust MCP servers.


### Multi-Layer Security: Understanding Where to Validate

Before diving into implementation, understand that security happens at multiple layers. You don't have to implement everything in your MCP server—you can leverage existing security in your backend systems.

```text
                  Security Layers in MCP
LAYER 1: MCP Server Access
   Question: Can this user reach the MCP server at all?
   Validated: Token signature, expiration, issuer, audience
   Claims used: sub, iss, aud, exp
   Result: 401 Unauthorized if invalid
LAYER 2: Tool-Level Authorization
   Question: Can this user call this specific tool?
   Validated: Scopes match tool requirements
   Claims used: scope, permissions, roles, groups
   Result: 403 Forbidden if insufficient permissions
LAYER 3: Data-Level Security (Backend Systems)
   Question: What data can this user see/modify?
   Validated by: Database, API, or data platform
   Examples:
   • PostgreSQL Row-Level Security (RLS)
   • GraphQL field-level authorization
   • API gateway per-resource policies
   • Data warehouse column masking
   Result: Filtered/masked data or 403 from backend
```

(table/list, cells in reading order)
- Each layer catches different security concerns:
- Layer
- What It Catches
- Example
- Layer 1: Server Access
- Invalid/expired tokens, wrong IdP, attacks
- Stolen token from different app
- Layer 2: Tool Authorization
- Users calling tools they shouldn't
- Analyst trying to use admin tools
- Layer 3: Data Security
- Users accessing data they shouldn't
- User A reading User B's records
- What You Control vs. What You Delegate
- Your MCP server handles Layers 1 & 2:


### Backend systems handle Layer 3:

Databases enforce row-level security using the user ID you provide APIs check permissions on each resource Data platforms apply column masking based on user roles

The advantage: You don't reinvent data security.


### Practical Example: The Three Layers in Action

```rust
// LAYER 1: Happens in middleware before your tool code runs
// The request already has a validated token at this point

#[derive(TypedTool)]
#[mcp_tool(name = "query_sales", description = "Query sales data")]
pub struct QuerySales;

impl QuerySales {
    pub async fn run(
        &self,
        input: QueryInput,
        context: &ToolContext,
    ) -> Result<SalesData> {
        let auth = context.auth()?;

        // LAYER 2: Check tool-level scope
        // "Can this user call this tool at all?"
        auth.require_scope("read:sales")?;

        // LAYER 3: Pass identity to database, let RLS handle row filtering
        // "What sales records can this user see?"
        let results = self.database
            .query(&input.sql)
            .with_user_context(&auth.user_id, &auth.org_id)  // Database uses this for RLS
            .await?;

        // The database only returns rows this user is allowed to see
        // We didn't write that logic—the database handles it

        Ok(results)
    }
}
```

-- Policy defined once in database, enforced automatically CREATE POLICY sales_team_only ON sales

```rust
    FOR SELECT
    USING (team_id = current_setting('app.team_id')::uuid);
```

```yaml
  id: ID!
  name: String!
  email: String! @auth(requires: "read:pii")      # Only users with PII scope
  ssn: String @auth(requires: "admin:sensitive")  # Only admins
}
```


### API Gateway policies:

```yaml
# AWS API Gateway resource policy
/customers/{customerId}:
  GET:
    auth:
      # User can only access customers in their organization
      condition: $context.authorizer.org_id == $resource.org_id
```

(table/list, cells in reading order)
- Choosing Where to Implement Security
- Security Concern
- Best Layer
- Reasoning
- "Is this token valid?"
- Layer 1 (MCP Server)
- Must happen first
- "Can user call this tool?"
- Layer 2 (MCP Server)
- Scope-based, defined in IdP
- "Can user see this row?"
- Layer 3 (Database)
- Database knows data relationships
- "Can user see this field?"
- Layer 3 (API/GraphQL)
- Field sensitivity is data concern
- "What columns should be masked?"
- Layer 3 (Data Platform)
- Masking rules are data governance

The principle: Implement security as close to the data as possible.


### The Validation Pipeline

```text
                  JWT Validation Pipeline
Incoming Request
  1. EXTRACT TOKEN
     Authorization: Bearer eyJhbGciOiJS...
  2. DECODE HEADER (without verification)
     { "alg": "RS256", "kid": "key-123" }
  3. FETCH PUBLIC KEY (from JWKS, cached)
     Match key by "kid" from header
  4. VERIFY SIGNATURE
     RSA/ECDSA verification using public key
  5. VALIDATE CLAIMS
     • exp (expiration)
     • nbf (not before)
     • iss (issuer)
```

```text
   • aud (audience)
6. EXTRACT USER INFO
   sub, email, scopes → AuthContext
```


### Core Dependencies

```toml
# Cargo.toml
[dependencies]
jsonwebtoken = "9"           # JWT encoding/decoding
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync"] }
thiserror = "1"
tracing = "0.1"
```

```rust
    extract::FromRequestParts,
    http::{request::Parts, StatusCode, header},
    response::{IntoResponse, Response},
};

pub struct BearerToken(pub String);

#[async_trait]
```


### impl<S> FromRequestParts<S> for BearerToken where

```rust
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or(AuthError::MissingToken)?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AuthError::InvalidHeader)?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AuthError::InvalidScheme);
        }

        let token = auth_str[7..].trim().to_string();

        if token.is_empty() {
            return Err(AuthError::MissingToken);
        }

        Ok(BearerToken(token))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization token")]
    MissingToken,

    #[error("Invalid authorization header")]
    InvalidHeader,

    #[error("Invalid authorization scheme, expected Bearer")]
    InvalidScheme,

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("Insufficient permissions")]
    InsufficientScope,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::MissingToken | AuthError::InvalidHeader | AuthError::InvalidScheme => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AuthError::ValidationFailed(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AuthError::InsufficientScope => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
        };

// ... truncated, 9 more lines in source book ...
```

```rust
use jsonwebtoken::jwk::{JwkSet, Jwk};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct JwksClient {
    jwks_uri: String,
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    cache_duration: Duration,
}

struct CachedJwks {
    jwks: JwkSet,
    fetched_at: Instant,
}

impl JwksClient {
    pub fn new(jwks_uri: String) -> Self {
        Self {
            jwks_uri,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_secs(3600), // 1 hour
        }
    }

    pub async fn get_key(&self, kid: &str) -> Result<Jwk, AuthError> {
        let jwks = self.get_jwks().await?;

        jwks.keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(kid))
            .cloned()
            .ok_or_else(|| AuthError::ValidationFailed(
                format!("Key not found: {}", kid)
            ))
    }

    async fn get_jwks(&self) -> Result<JwkSet, AuthError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = &*cache {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Fetch fresh JWKS
        let jwks = self.fetch_jwks().await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedJwks {
                jwks: jwks.clone(),
                fetched_at: Instant::now(),
            });
// ... truncated, 29 more lines in source book ...
```

```rust
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct JwtValidatorConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
    pub algorithms: Vec<Algorithm>,
    pub leeway_seconds: u64,
}

impl JwtValidatorConfig {
    /// Create config for AWS Cognito
    pub fn cognito(region: &str, user_pool_id: &str, client_id: &str) -> Self {
        let issuer = format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            region, user_pool_id
        );
        let jwks_uri = format!("{}/.well-known/jwks.json", issuer);

        Self {
            issuer,
            audience: client_id.to_string(),
            jwks_uri,
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 60,
        }
    }

    /// Create config for Auth0
    pub fn auth0(domain: &str, audience: &str) -> Self {
        Self {
            issuer: format!("https://{}/", domain),
            audience: audience.to_string(),
            jwks_uri: format!("https://{}/.well-known/jwks.json", domain),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 60,
        }

    }


    /// Create config for Microsoft Entra ID
    pub fn entra(tenant_id: &str, client_id: &str) -> Self {
        Self {
            issuer: format!("https://login.microsoftonline.com/{}/v2.0", tenant_id),
            audience: client_id.to_string(),
            jwks_uri: format!(
                "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
                tenant_id
            ),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 60,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
// ... truncated, 94 more lines in source book ...
```


### Auth Context for Tools

Make authentication available to tools.

```rust
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub scopes: HashSet<String>,
}

impl AuthContext {
    pub fn from_claims(claims: &Claims) -> Self {
        let scopes = claims.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .or_else(|| {
                claims.permissions.as_ref().map(|p| p.iter().cloned().collect())
            })
            .unwrap_or_default();

        Self {
            user_id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.name.clone(),
            scopes,
        }
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope) ||
        // Check wildcards: "write:*" matches "write:data"
        self.scopes.iter().any(|s| {
            s.ends_with(":*") && scope.starts_with(&s[..s.len()-1])
        })
    }

    pub fn require_scope(&self, scope: &str) -> Result<(), AuthError> {
        if self.has_scope(scope) {
            Ok(())

        } else {
            Err(AuthError::InsufficientScope)
        }
    }

    pub fn require_any_scope(&self, scopes: &[&str]) -> Result<(), AuthError> {
        if scopes.iter().any(|s| self.has_scope(s)) {
            Ok(())
        } else {
            Err(AuthError::InsufficientScope)
        }
    }
}
```


### Extended Auth Context for Backend Passthrough

For Layer 3 security, you often need to pass additional claims to backend systems.

```rust
information:

#[derive(Debug, Clone)]
pub struct AuthContext {
    // Identity (Layer 1)
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,

    // Scopes for tool authorization (Layer 2)
    pub scopes: HashSet<String>,

    // Organization context for backend systems (Layer 3)
    pub org_id: Option<String>,
    pub team_id: Option<String>,
    pub roles: Vec<String>,
    pub groups: Vec<String>,

    // Raw claims for custom backend needs
    pub custom_claims: serde_json::Value,
}

impl AuthContext {
    /// Get the context as headers for HTTP backend calls
    pub fn as_headers(&self) -> Vec<(&'static str, String)> {
        let mut headers = vec![
            ("X-User-ID", self.user_id.clone()),
        ];

        if let Some(ref org) = self.org_id {
            headers.push(("X-Org-ID", org.clone()));
        }
        if let Some(ref team) = self.team_id {
            headers.push(("X-Team-ID", team.clone()));
        }
        if let Some(ref email) = self.email {
            headers.push(("X-User-Email", email.clone()));
        }

        headers
    }

    /// Get context for database session variables (PostgreSQL RLS)
    pub fn as_db_session_vars(&self) -> Vec<(&'static str, String)> {
        let mut vars = vec![
            ("app.user_id", self.user_id.clone()),
        ];

        if let Some(ref org) = self.org_id {
            vars.push(("app.org_id", org.clone()));
        }
        if let Some(ref team) = self.team_id {
            vars.push(("app.team_id", team.clone()));
        }

        vars
    }
}
```

```rust
    middleware::{self, Next},
    extract::State,
    http::Request,
    body::Body,
};
use std::sync::Arc;

pub type SharedValidator = Arc<JwtValidator>;

pub async fn auth_middleware(
    State(validator): State<SharedValidator>,
    BearerToken(token): BearerToken,
    mut request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, AuthError> {
    // Validate the token
    let claims = validator.validate(&token).await?;

    // Create auth context
    let auth_context = AuthContext::from_claims(&claims);

    // Add to request extensions for handlers to access
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

// Usage in router
pub fn create_router(validator: SharedValidator) -> Router {
    Router::new()
        .route("/mcp", post(mcp_handler))
        .layer(middleware::from_fn_with_state(
            validator.clone(),
            auth_middleware
        ))
        .with_state(validator)
}

// Access in handler
async fn mcp_handler(
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    tracing::info!(user = %auth.user_id, "Processing MCP request");
    // ...
}
```

```rust
impl JwtValidator {
    pub async fn validate_with_retry(&self, token: &str) -> Result<Claims, AuthError> {
        match self.validate(token).await {
            Ok(claims) => Ok(claims),
            Err(AuthError::ValidationFailed(msg)) if msg.contains("Key not found") => {
                // Key might have rotated, refresh JWKS and retry
                tracing::info!("Key not found, refreshing JWKS");
                self.jwks_client.refresh().await?;
                self.validate(token).await
            }
            Err(e) => Err(e),
        }
    }
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn create_test_token(claims: &Claims, key: &str) -> String {
        let encoding_key = EncodingKey::from_secret(key.as_bytes());
        encode(
            &Header::new(Algorithm::HS256),
            claims,
            &encoding_key
        ).unwrap()
    }

    #[test]
    fn test_auth_context_scope_matching() {
        let context = AuthContext {
            user_id: "user123".into(),
            email: Some("user@example.com".into()),
            name: None,
            scopes: ["read:data", "write:*"].iter().map(|s| s.to_string()).collect(),
        };

        assert!(context.has_scope("read:data"));
        assert!(context.has_scope("write:data"));
        assert!(context.has_scope("write:users"));
        assert!(!context.has_scope("admin:users"));
    }

    #[test]
    fn test_bearer_token_extraction() {
        // Test valid header

        let valid = "Bearer
```

```rust
        assert!(valid.starts_with("Bearer "));

        // Test invalid scheme
        let invalid = "Basic dXNlcjpwYXNz";
        assert!(!invalid.starts_with("Bearer "));
    }
}
```


### Integration Tests with Real IdP

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run manually with: cargo test -- --ignored
    async fn test_cognito_validation() {
        let config = JwtValidatorConfig::cognito(
            "us-east-1",
            &std::env::var("COGNITO_USER_POOL_ID").unwrap(),
            &std::env::var("COGNITO_CLIENT_ID").unwrap(),
        );

        let validator = JwtValidator::new(config);

        // Get a real token from Cognito (e.g., via test user)
        let token = get_test_token().await;

        let claims = validator.validate(&token).await.unwrap();

        assert!(!claims.sub.is_empty());
        assert!(claims.email.is_some());
    }
}
```

```rust
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_code, description) = match &self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "missing_token",
                "No authorization token provided"
            ),
            AuthError::InvalidHeader | AuthError::InvalidScheme => (
                StatusCode::UNAUTHORIZED,
                "invalid_request",
                "Invalid authorization header format"
            ),
            AuthError::ValidationFailed(msg) => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                msg.as_str()
            ),
            AuthError::InsufficientScope => (
                StatusCode::FORBIDDEN,
                "insufficient_scope",
                "Token does not have required scope"
            ),
        };

        // Add WWW-Authenticate header for 401 responses
        let mut response = (
            status,
            axum::Json(serde_json::json!({
                "error": error_code,
                "error_description": description
            }))
        ).into_response();

        if status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                format!("Bearer realm=\"mcp\", error=\"{}\"", error_code)
                    .parse()

                    .unwrap()
            );
        }

        response
    }
}
```

```text
// In JwtValidatorConfig
```


### pub leeway_seconds: u64,  // Typically 60 seconds

```text
// In validation
```

- Never accept the  alg  from the token without verification:

```rust
// GOOD: Explicitly allow specific algorithms
let mut validation = Validation::new(Algorithm::RS256);

// BAD: Would allow attacker to switch to "none"
// let validation = Validation::default();
```

- Always verify the audience matches your server:

```rust
// The token might be valid but intended for a different service
if !claims.aud.contains(&self.config.audience) {
    return Err(AuthError::ValidationFailed("Invalid audience".into()));
}
```


### Passing Identity to Backend Systems

Now that you have the auth context, here's how to pass it to different backend systems for Layer 3 security:


### Database with Row-Level Security

```rust
impl QueryTool {
    pub async fn run(&self, input: QueryInput, context: &ToolContext) -> Result<QueryResult> {
        let auth = context.auth()?;
        auth.require_scope("read:data")?;  // Layer 2

        // Layer 3: Set session variables for PostgreSQL RLS
        let pool = &self.database;
        let conn = pool.acquire().await?;

        // Set user context that RLS policies will use
        for (key, value) in auth.as_db_session_vars() {
            sqlx::query(&format!("SET LOCAL {} = $1", key))
                .bind(&value)
                .execute(&mut *conn)
                .await?;
        }

        // Query executes with RLS automatically filtering rows
        let results = sqlx::query_as::<_, Record>(&input.sql)
            .fetch_all(&mut *conn)
            .await?;

        Ok(QueryResult { records: results })
    }
}
```


### Downstream API Calls

```rust
impl ApiTool {
    pub async fn run(&self, input: ApiInput, context: &ToolContext) -> Result<ApiResult> {
        let auth = context.auth()?;
        auth.require_scope("read:api")?;  // Layer 2

        // Layer 3: Forward identity headers to downstream API
        let mut request = self.client
            .get(&format!("{}/resource/{}", self.api_base, input.resource_id));

        for (name, value) in auth.as_headers() {
            request = request.header(name, value);
        }

        // Downstream API uses these headers for its own authorization
        let response = request.send().await?;

        if response.status() == StatusCode::FORBIDDEN {
            // Backend denied access - this is Layer 3 rejection
            return Err(McpError::forbidden(
                "You don't have access to this resource"
            ));
        }

        Ok(response.json().await?)
    }
}
```


### GraphQL with Field-Level Security

```rust
impl GraphQLTool {
    pub async fn run(&self, input: GraphQLInput, context: &ToolContext) -> Result<GraphQLResult> {
        let auth = context.auth()?;
        auth.require_scope("read:graphql")?;  // Layer 2

        // Layer 3: GraphQL server handles field-level authorization
        // using the identity we pass in the context
        let response = self.graphql_client
            .query(&input.query)
            .variables(input.variables)
            .header("X-User-ID", &auth.user_id)
            .header("X-User-Scopes", auth.scopes.iter().collect::<Vec<_>>().join(" "))
            .execute()
            .await?;

        // Fields the user can't access come back as null or are omitted
        // based on the GraphQL schema's @auth directives

        Ok(response)
    }
}
```


### The Security Division of Labor

```text
            What Each System Is Responsible For
YOUR MCP SERVER
   [ok] Validate JWT signature and claims (Layer 1)
   [ok] Check scopes for each tool (Layer 2)
   [ok] Extract and forward user identity
   [no] NOT: Per-row or per-field authorization
YOUR DATABASE
   [ok] Row-Level Security policies
   [ok] Column-level permissions
   [ok] Data filtering based on user context
   [no] NOT: Token validation (trusts MCP server)
YOUR API LAYER
   [ok] Resource-level authorization
   [ok] Field masking (PII, sensitive data)
   [ok] Rate limiting per user/org
   [no] NOT: Token validation (trusts MCP server)
RESULT: Each system does what it's best at
MCP validates identity → Backend enforces data policies
```

1.

(table/list, cells in reading order)
- Multi-Layer Security Model
- Layer
- What
- Where
- Your Responsibility
- Layer 1
- Token validation
- MCP Server
- Implement (this chapter)
- Layer 2
- Tool authorization
- MCP Server
- Check scopes in tools
- Layer 3
- Data authorization
- Backend systems
- Pass identity, delegate to existing systems

The key insight: You don't have to build all security in your MCP server.


### Common Pitfalls to Avoid

Not caching JWKS (DoS risk) Not handling key rotation Accepting any algorithm Skipping audience verification

Ignoring clock skew Trying to implement row-level security in MCP instead of the database
