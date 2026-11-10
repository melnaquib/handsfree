---
chapter: 20
title: "Identity Provider Integration"
part: "Part V: Enterprise Security"
use_when: "Configuring Cognito, Auth0 or Entra ID; comparing providers."
suggested_prerequisites: [ch19]
source_kb: 104
extracted_kb: 77
code_blocks: 86
quality: auto-extracted
---

# Ch 20: Identity Provider Integration

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Identity Provider Integration

This chapter covers integrating MCP servers with enterprise identity providers.


### The Most Important Advice: Use What You Already Have

The providers in this course are examples, not recommendations.

```text
               Provider Selection Decision Tree
Does your organization already have an identity provider?
YES                                                               ▶
    USE THAT PROVIDER.
    • Users already know how to log in
    • IT already knows how to manage it
    • Security policies already exist
    • Compliance is already handled
    • No new vendor relationships needed
NO (starting fresh)                                               ▶
    Consider your existing infrastructure:
    • Heavy AWS user? → Cognito
    • Microsoft 365? → Entra ID
    • Need flexibility? → Auth0 or Okta
    • Self-hosted? → Keycloak
```

(table/list, cells in reading order)
- Reason
- Impact
- Users have to learn new login
- Friction, support tickets
- IT has to manage two systems
- Operational burden
- Permissions need duplication
- Security gaps
- Reason
- Impact
- Compliance scope expands
- More audit work
- More vendors to manage
- Procurement complexity

(table/list, cells in reading order)
- Understanding Provider Examples
- The providers covered in this chapter:
- Provider
- Why We Cover It
- Your Situation
- AWS Cognito
- Common in AWS shops
- Use if you're already AWS-native
- Auth0
- Developer-friendly, good docs
- Use if you need rapid prototyping
- Microsoft Entra
- Enterprise Microsoft environments
- Use if you have Microsoft 365
- Okta
- Keycloak
- Enterprise workforce identity
- Use if already deployed
- Self-hosted, open source
- Use if you need on-premises


### Provider Feature Comparison

```text
                Identity Provider Comparison
AWS Cognito
   Best for: AWS-native applications
   Pros: Deep AWS integration, pay-per-use pricing
   Cons: Limited customization, complex federation
   Use when: Already invested in AWS ecosystem
Auth0
   Best for: Developer-friendly, custom requirements
   Pros: Extensive customization, excellent docs
   Cons: Can get expensive at scale
   Use when: Need flexibility and rapid development
Microsoft Entra ID (formerly Azure AD)
   Best for: Microsoft/O365 enterprises
   Pros: SSO with Microsoft apps, enterprise features
   Cons: Complex setup, Microsoft-centric
   Use when: Enterprise already uses Microsoft 365
Okta
   Best for: Large enterprises, workforce identity
   Pros: Enterprise features, SSO across apps
   Cons: Expensive, complex
   Use when: Enterprise-grade requirements
```

```rust
use crate::auth::{JwtValidator, JwtValidatorConfig};

pub enum IdentityProvider {
    Cognito {
        region: String,
        user_pool_id: String,
        client_id: String,
    },
    Auth0 {
        domain: String,
        audience: String,
    },
    Entra {
        tenant_id: String,
        client_id: String,
    },
}

impl IdentityProvider {
    pub fn into_validator(self) -> JwtValidator {
        let config = match self {
            IdentityProvider::Cognito { region, user_pool_id, client_id } => {
                JwtValidatorConfig::cognito(&region, &user_pool_id, &client_id)
            }
            IdentityProvider::Auth0 { domain, audience } => {
                JwtValidatorConfig::auth0(&domain, &audience)
            }
            IdentityProvider::Entra { tenant_id, client_id } => {
                JwtValidatorConfig::entra(&tenant_id, &client_id)
            }
        };

        JwtValidator::new(config)
    }
}
```

```rust
use std::env;

#[derive(Debug, Clone)]
pub struct IdpConfig {
    pub provider: IdentityProvider,
}


impl IdpConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let provider_type = env::var("IDP_PROVIDER")
            .unwrap_or_else(|_| "cognito".to_string());

        let provider = match provider_type.as_str() {
            "cognito" => IdentityProvider::Cognito {
                region: env::var("AWS_REGION")
                    .map_err(|_| ConfigError::Missing("AWS_REGION"))?,
                user_pool_id: env::var("COGNITO_USER_POOL_ID")
                    .map_err(|_| ConfigError::Missing("COGNITO_USER_POOL_ID"))?,
                client_id: env::var("COGNITO_CLIENT_ID")
                    .map_err(|_| ConfigError::Missing("COGNITO_CLIENT_ID"))?,
            },
            "auth0" => IdentityProvider::Auth0 {
                domain: env::var("AUTH0_DOMAIN")
                    .map_err(|_| ConfigError::Missing("AUTH0_DOMAIN"))?,
                audience: env::var("AUTH0_AUDIENCE")
                    .map_err(|_| ConfigError::Missing("AUTH0_AUDIENCE"))?,
            },
            "entra" | "azure" => IdentityProvider::Entra {
                tenant_id: env::var("ENTRA_TENANT_ID")
                    .map_err(|_| ConfigError::Missing("ENTRA_TENANT_ID"))?,
                client_id: env::var("ENTRA_CLIENT_ID")
                    .map_err(|_| ConfigError::Missing("ENTRA_CLIENT_ID"))?,
            },
            _ => return Err(ConfigError::InvalidProvider(provider_type)),
        };

        Ok(Self { provider })

    }
}
```

```rust
#[derive(Debug)]
pub struct UserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub groups: Vec<String>,
    pub scopes: Vec<String>,
}

impl UserInfo {
    /// Parse claims based on IdP format
    pub fn from_claims(claims: &Claims, provider: &IdentityProvider) -> Self {
        match provider {
            IdentityProvider::Cognito { .. } => Self::from_cognito(claims),
            IdentityProvider::Auth0 { .. } => Self::from_auth0(claims),
            IdentityProvider::Entra { .. } => Self::from_entra(claims),
        }
    }

    fn from_cognito(claims: &Claims) -> Self {
        // Cognito uses:
        // - sub: user ID (UUID)
        // - email: user email
        // - cognito:username: username
        // - cognito:groups: array of group names
        Self {
            id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.get("cognito:username").cloned(),
            groups: claims.get_array("cognito:groups").unwrap_or_default(),
            scopes: claims.scope_list(),
        }
    }

    fn from_auth0(claims: &Claims) -> Self {
        // Auth0 uses:
        // - sub: provider|user_id (e.g., "auth0|123" or "google-oauth2|456")
        // - email: user email
        // - name: display name

        // - permissions: array of permission strings
        Self {
            id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.name.clone(),
            groups: claims.get_array("https://yourapp/groups").unwrap_or_default(),
            scopes: claims.permissions.clone().unwrap_or_else(|| claims.scope_list()),
        }
    }

    fn from_entra(claims: &Claims) -> Self {
        // Entra ID uses:
        // - oid: object ID (GUID)
        // - preferred_username: UPN (user@domain.com)
        // - name: display name
        // - groups: array of group GUIDs
        // - roles: array of app role names
        Self {
            id: claims.get("oid").unwrap_or(&claims.sub).clone(),
            email: claims.get("preferred_username").cloned(),
// ... truncated, 6 more lines in source book ...
```

```text
                Corporate Federation
User → Corporate IdP (Okta/Entra) → OAuth Provider → MCP Server
1. User clicks "Login with Corporate SSO"
2. Redirected to corporate IdP (Okta, Entra, etc.)
3. User authenticates with corporate credentials
4. Corporate IdP issues SAML assertion to OAuth provider
5. OAuth provider (Cognito/Auth0) issues JWT
6. MCP server validates JWT
Benefits:
• Single sign-on across all apps
• Central user management
• Automatic deprovisioning when employees leave
```

```text
                  Social Login Federation
User → Social Provider (Google/GitHub) → OAuth Provider → MCP
Cognito: Social identity pools
Auth0: Social connections
Entra: External identities
User identity format varies:
• Cognito: "us-east-1:abc123-def456"
• Auth0: "google-oauth2|1234567890"
• Entra: "external_identity_guid"
```

```rust
# Cognito uses OAuth scopes + custom scopes from resource servers
# Define custom scopes in Cognito resource server:

aws cognito-idp create-resource-server \
  --user-pool-id us-east-1_xxxx \
  --identifier mcp-server \
  --name "MCP Server" \
  --scopes ScopeName=read:tools,ScopeDescription="Read tools" \
          ScopeName=execute:tools,ScopeDescription="Execute tools"

// Cognito scope format: "resource-server/scope"
fn cognito_scopes(claims: &Claims) -> Vec<String> {
    claims.scope
        .as_ref()
        .map(|s| {
            s.split_whitespace()
                .filter_map(|scope| {
                    // Strip resource server prefix if present
                    scope.split('/').last().map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}
```


### Auth0 Permissions

```rust
// Auth0 uses permissions array (from RBAC)
fn auth0_permissions(claims: &Claims) -> Vec<String> {
    // Prefer permissions if available (RBAC)
    if let Some(perms) = &claims.permissions {
        return perms.clone();
    }
    // Fall back to scope string
    claims.scope_list()
}
```


### Entra App Roles

```rust
// Entra ID uses app roles (defined in app registration)
fn entra_roles(claims: &Claims) -> Vec<String> {
    claims.get_array("roles").unwrap_or_default()
}
```

```bash
# Cognito: Use AWS CLI
aws cognito-idp admin-initiate-auth \
  --user-pool-id us-east-1_xxxx \
  --client-id your-client-id \
  --auth-flow ADMIN_USER_PASSWORD_AUTH \
  --auth-parameters USERNAME=testuser,PASSWORD=TestPass123!

# Auth0: Use Management API or test application
curl --request POST \
  --url 'https://your-tenant.auth0.com/oauth/token' \
  --header 'content-type: application/x-www-form-urlencoded' \
  --data grant_type=password \
  --data username=testuser@example.com \
  --data 'password=TestPass123!' \
  --data client_id=your-client-id \
  --data client_secret=your-client-secret

# Entra: Use Azure CLI
az account get-access-token --resource your-client-id
```


### Mock Validator for Tests

```rust
#[cfg(test)]
pub struct MockValidator {
    user_id: String,
    scopes: Vec<String>,
}

#[cfg(test)]
impl MockValidator {
    pub fn user(id: &str) -> Self {
        Self {
            user_id: id.to_string(),
            scopes: vec!["read:tools".into()],
        }
    }

    pub fn admin(id: &str) -> Self {
        Self {
            user_id: id.to_string(),
            scopes: vec!["admin:*".into()],
        }
    }

    pub fn with_scopes(mut self, scopes: &[&str]) -> Self {
        self.scopes = scopes.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn into_context(self) -> AuthContext {
        AuthContext {
            user_id: self.user_id,
            email: Some("test@example.com".into()),
            name: Some("Test User".into()),
            scopes: self.scopes.into_iter().collect(),
        }
    }
}
```

(table/list, cells in reading order)
- Security Considerations
- Token Audience Validation
- Each provider sets audience differently:
- Provider
- Audience Value
- Cognito
- Client ID
- Auth0
- Entra
- API identifier (custom URL)
- Client ID or Application ID URI

```rust
// Always validate audience matches your configuration
if !claims.aud.contains(&self.config.audience) {
    return Err(AuthError::ValidationFailed("Invalid audience"));
}
```


### Issuer Validation

```rust
// Expected issuers
let cognito_iss = "https://cognito-idp.us-east-1.amazonaws.com/us-east-1_xxxx";
let auth0_iss = "https://your-tenant.auth0.com/";
let entra_iss = "https://login.microsoftonline.com/tenant-id/v2.0";

// Validate issuer exactly matches
if claims.iss != expected_issuer {
    return Err(AuthError::ValidationFailed("Invalid issuer"));
}
```


### Chapter Overview

The following sections provide detailed setup guides for each provider:


### Summary

The most important takeaway: Use what you already have. Don't introduce a new identity provider just for MCP servers.

The providers in this chapter are examples, not recommendations.


### AWS Cognito

AWS Cognito is Amazon's identity service, providing user pools for authentication and identity pools for AWS resource access.


### The Easy Way: cargo pmcp + CDK

The fastest path to production: Use  cargo pmcp  to configure OAuth, then let the generated CDK handle Cognito setup. You don't need to manually create user pools, configure clients, or set up resource servers—the CDK does it all.


### Step 1: Initialize OAuth Configuration

```bash
# Initialize deployment with Cognito OAuth
cargo pmcp deploy init --target pmcp-run --oauth cognito

# This creates/updates .pmcp/deploy.toml with:

# .pmcp/deploy.toml
[auth]
enabled = true
provider = "cognito"
callback_urls = [
    "http://localhost:3000/callback",  # For development
]

[auth.dcr]
# Dynamic Client Registration for MCP clients
enabled = true
public_client_patterns = [
    "claude",
    "cursor",
    "chatgpt",
    "mcp-inspector",
]
default_scopes = [
    "openid",
    "email",
    "mcp/read",
]
allowed_scopes = [
    "openid",
    "email",
    "profile",
    "mcp/read",
    "mcp/write",
    "mcp/admin",
]
```


### Step 2: Deploy with CDK

The deployment generates a CDK stack that creates all Cognito resources:

```bash
# Build and deploy
cargo pmcp deploy

# The CDK stack creates:
# - Cognito User Pool with password policies
# - App client with OAuth flows configured
# - Resource server with MCP scopes
# - Optional: Federation with corporate IdP
```

```yaml
// Example: What cargo pmcp deploy generates in CDK
// You don't write this - it's generated from deploy.toml

// User Pool with enterprise settings
const userPool = new cognito.UserPool(this, 'McpUserPool', {
  userPoolName: `${serverId}-users`,
  selfSignUpEnabled: false,  // Admin-only provisioning
  passwordPolicy: {
    minLength: 12,
    requireLowercase: true,
    requireUppercase: true,
    requireDigits: true,
    requireSymbols: true,
  },
  mfa: cognito.Mfa.OPTIONAL,
});

// Resource server with MCP scopes
const resourceServer = userPool.addResourceServer('McpApi', {
  identifier: 'mcp',
  scopes: [
    { scopeName: 'read', scopeDescription: 'Read MCP resources' },
    { scopeName: 'write', scopeDescription: 'Modify MCP resources' },
    { scopeName: 'admin', scopeDescription: 'Admin operations' },
  ],
});

// App client with OAuth configuration
const appClient = userPool.addClient('McpClient', {
  generateSecret: true,
  oAuth: {
    flows: { authorizationCodeGrant: true },
    scopes: [/* from deploy.toml */],
    callbackUrls: [/* from deploy.toml */],
  },
});
```

You don't need to understand Cognito's complex configuration options.


### Step 3: Your Server Code

Your Rust code just uses the OAuth middleware—it doesn't know or care that Cognito is the provider:

```rust
use pmcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // OAuth configuration is loaded from environment
    // (set by CDK stack outputs)
    let server = ServerBuilder::new("my-server", "1.0.0")
        .with_oauth_from_env()  // Reads COGNITO_* env vars
        .with_tool(MyTool)
        .build()?;

    server.serve().await
}
```


### Manual Setup (When You Need Control)

If you need more control, or your organization has specific Cognito requirements, you can configure Cognito manually.


### Cognito Architecture

```text
                  Cognito for MCP Servers
    User Pool        Authentication
                     • User sign-up/sign-in
   • Users           • Password policies
   • Groups          • MFA
   • App clients     • Custom attributes
                     • Federation (SAML/OIDC)
           Issues JWT
    MCP Server       Validates JWT, extracts user info
(Optional for AWS access)
   Identity Pool     AWS credentials for resources
                     • S3, DynamoDB, etc.
```

(table/list, cells in reading order)
- Creating a User Pool
- AWS Console
- 1. Go to Cognito → User Pools → Create user pool
- 2. Configure sign-in:
- Email as username (recommended) Enable MFA (optional but recommended)
- 3. Configure sign-up:
- Self-registration or admin-only Required attributes (email)
- 4. Configure app client:

Create app client for your MCP server Enable ALLOW_USER_SRP_AUTH Generate client secret (for server-side apps)


### AWS CLI / CloudFormation

```text
# cloudformation/cognito.yaml
```

AWSTemplateFormatVersion: '2010-09-09' Description: Cognito User Pool for MCP Server


### Resources:

```rust
  UserPool:
    Type: AWS::Cognito::UserPool
    Properties:
      UserPoolName: mcp-server-users
      UsernameAttributes:
        - email
      AutoVerifiedAttributes:
        - email
      MfaConfiguration: OPTIONAL
      Policies:
        PasswordPolicy:
          MinimumLength: 12
          RequireLowercase: true
          RequireNumbers: true
          RequireSymbols: true
          RequireUppercase: true
      Schema:
        - Name: email
          Required: true
          Mutable: true
        - Name: department
          AttributeDataType: String
          Mutable: true

  UserPoolClient:
    Type: AWS::Cognito::UserPoolClient
    Properties:
      UserPoolId: !Ref UserPool
      ClientName: mcp-server-client
      GenerateSecret: true
      ExplicitAuthFlows:

        - ALLOW_USER_SRP_AUTH
        - ALLOW_REFRESH_TOKEN_AUTH
        - ALLOW_USER_PASSWORD_AUTH  # For testing only
      SupportedIdentityProviders:
        - COGNITO
      AllowedOAuthFlows:
        - code
      AllowedOAuthScopes:
        - openid
        - email
        - profile
        - mcp-server/read:tools
        - mcp-server/execute:tools
      AllowedOAuthFlowsUserPoolClient: true
      CallbackURLs:
        - https://your-app.com/callback
        - http://localhost:3000/callback

  ResourceServer:
    Type: AWS::Cognito::UserPoolResourceServer
    Properties:
      UserPoolId: !Ref UserPool
      Identifier: mcp-server
      Name: MCP Server API
      Scopes:
        - ScopeName: read:tools
          ScopeDescription: Read MCP tools
        - ScopeName: execute:tools
// ... truncated, 3 more lines in source book ...
```


### Outputs:

```rust
  UserPoolId:
    Value: !Ref UserPool
  UserPoolClientId:
    Value: !Ref UserPoolClient
  UserPoolDomain:
    Value: !Sub "https://cognito-idp.${AWS::Region}.amazonaws.com/${UserPool}"
```

```rust
use std::env;

#[derive(Debug, Clone)]
pub struct CognitoConfig {
    pub region: String,
    pub user_pool_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl CognitoConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            region: env::var("AWS_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            user_pool_id: env::var("COGNITO_USER_POOL_ID")?,
            client_id: env::var("COGNITO_CLIENT_ID")?,
            client_secret: env::var("COGNITO_CLIENT_SECRET").ok(),
        })
    }

    pub fn issuer(&self) -> String {
        format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            self.region, self.user_pool_id
        )
    }

    pub fn jwks_uri(&self) -> String {
        format!("{}/.well-known/jwks.json", self.issuer())

    }
}
```


### Validator Setup

```rust
impl JwtValidatorConfig {
    pub fn from_cognito(config: &CognitoConfig) -> Self {
        Self {
            issuer: config.issuer(),
            audience: config.client_id.clone(),
            jwks_uri: config.jwks_uri(),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 60,
        }
    }
}

// Main setup
pub async fn setup_cognito_auth() -> Result<JwtValidator> {
    let config = CognitoConfig::from_env()?;
    let validator_config = JwtValidatorConfig::from_cognito(&config);
    Ok(JwtValidator::new(validator_config))
}
```


### Cognito-Specific Claims

```rust
#[derive(Debug, Deserialize)]
pub struct CognitoClaims {
    // Standard claims
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,

    // Cognito-specific
    pub token_use: String,           // "access" or "id"
    pub auth_time: Option<u64>,
    pub client_id: Option<String>,

    // User attributes (from ID token)
    pub email: Option<String>,
    pub email_verified: Option<bool>,

    // Groups (custom claim)
    #[serde(rename = "cognito:groups")]
    pub groups: Option<Vec<String>>,

    #[serde(rename = "cognito:username")]
    pub username: Option<String>,

    // Custom attributes (prefixed with "custom:")
    #[serde(flatten)]
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

impl CognitoClaims {
    pub fn get_custom(&self, name: &str) -> Option<&serde_json::Value> {
        self.custom_attributes.get(&format!("custom:{}", name))
    }

    pub fn is_access_token(&self) -> bool {

        self.token_use == "access"
    }

    pub fn is_id_token(&self) -> bool {
        self.token_use == "id"
    }
}
```

```bash
# Create groups in Cognito
aws cognito-idp create-group \
  --user-pool-id us-east-1_xxxx \
  --group-name Admins \
  --description "Administrator access"

aws cognito-idp create-group \
  --user-pool-id us-east-1_xxxx \
  --group-name Developers \
  --description "Developer access"

# Add user to group
aws cognito-idp admin-add-user-to-group \
  --user-pool-id us-east-1_xxxx \
  --username user@example.com \
  --group-name Developers
```


### Group-Based Authorization

```rust
impl AuthContext {
    pub fn from_cognito_claims(claims: &CognitoClaims) -> Self {
        let groups = claims.groups.clone().unwrap_or_default();

        // Map groups to scopes
        let mut scopes: HashSet<String> = HashSet::new();

        for group in &groups {
            match group.as_str() {
                "Admins" => {
                    scopes.insert("admin:*".into());
                    scopes.insert("execute:tools".into());
                    scopes.insert("read:tools".into());
                }
                "Developers" => {
                    scopes.insert("execute:tools".into());
                    scopes.insert("read:tools".into());
                }
                "ReadOnly" => {
                    scopes.insert("read:tools".into());
                }
                _ => {}
            }
        }

        Self {
            user_id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.username.clone(),
            scopes,
        }
    }
}
```

```rust
# CloudFormation for SAML IdP
SAMLIdentityProvider:
  Type: AWS::Cognito::UserPoolIdentityProvider
  Properties:
    UserPoolId: !Ref UserPool
    ProviderName: CorporateSSO
    ProviderType: SAML
    ProviderDetails:
      MetadataURL: https://idp.company.com/metadata.xml
    AttributeMapping:
      email: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress
      given_name: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname
      family_name: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname
      custom:department: Department
```

```rust
  Type: AWS::Cognito::UserPoolIdentityProvider
  Properties:
    UserPoolId: !Ref UserPool
    ProviderName: Okta
    ProviderType: OIDC
    ProviderDetails:
      client_id: okta-client-id
      client_secret: okta-client-secret
      authorize_scopes: openid email profile
      oidc_issuer: https://company.okta.com
    AttributeMapping:
      email: email
      given_name: given_name
      family_name: family_name
```

```rust
// Pre-token generation trigger - customize JWT claims
use aws_lambda_events::event::cognito::CognitoEventUserPoolsPreTokenGen;
use lambda_runtime::{service_fn, Error, LambdaEvent};

async fn pre_token_gen(
    event: LambdaEvent<CognitoEventUserPoolsPreTokenGen>,
) -> Result<CognitoEventUserPoolsPreTokenGen, Error> {
    let mut response = event.payload;

    // Add custom claims based on user attributes
    let user_attributes = &response.request.user_attributes;

    if let Some(department) = user_attributes.get("custom:department") {
        // Add department to claims
        response.response.claims_override_details
            .get_or_insert_default()
            .claims_to_add_or_override
            .get_or_insert_default()
            .insert("department".into(), department.clone());
    }

    // Add permissions based on groups
    if let Some(groups) = user_attributes.get("cognito:groups") {
        let permissions = groups_to_permissions(groups);
        response.response.claims_override_details
            .get_or_insert_default()
            .claims_to_add_or_override
            .get_or_insert_default()
            .insert("permissions".into(), permissions.join(" "));
    }

    Ok(response)
}
```

```bash
# Create test user
aws cognito-idp admin-create-user \
  --user-pool-id us-east-1_xxxx \
  --username testuser@example.com \
  --user-attributes Name=email,Value=testuser@example.com \
  --temporary-password TempPass123!

# Set permanent password
aws cognito-idp admin-set-user-password \
  --user-pool-id us-east-1_xxxx \
  --username testuser@example.com \
  --password SecurePass123! \
  --permanent

# Get tokens
aws cognito-idp admin-initiate-auth \
  --user-pool-id us-east-1_xxxx \
  --client-id your-client-id \
  --auth-flow ADMIN_USER_PASSWORD_AUTH \
  --auth-parameters USERNAME=testuser@example.com,PASSWORD=SecurePass123!
```


### Integration Test

```rust
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_cognito_auth() {
    let config = CognitoConfig::from_env().unwrap();
    let validator = JwtValidator::new(JwtValidatorConfig::from_cognito(&config));

    // Get token via AWS SDK
    let token = get_cognito_token(&config).await.unwrap();

    let claims = validator.validate(&token).await.unwrap();

    assert!(!claims.sub.is_empty());
    println!("User ID: {}", claims.sub);
    println!("Email: {:?}", claims.email);
}

async fn get_cognito_token(config: &CognitoConfig) -> Result<String> {
    let client = aws_sdk_cognitoidentityprovider::Client::new(&aws_config::load_from_env().await);

    let response = client
        .admin_initiate_auth()
        .user_pool_id(&config.user_pool_id)
        .client_id(&config.client_id)
        .auth_flow(AuthFlowType::AdminUserPasswordAuth)
        .auth_parameters("USERNAME", "testuser@example.com")
        .auth_parameters("PASSWORD", "SecurePass123!")
        .send()
        .await?;

    Ok(response.authentication_result()
        .unwrap()
        .access_token()
        .unwrap()
        .to_string())
}
```


### Summary

Recommended approach: Use  cargo pmcp deploy init --oauth cognito  to generate the CDK stack that handles all Cognito complexity.


### If you need manual setup, AWS Cognito integration requires:

1.


### Key Cognito-specific considerations:

Token types: Access vs ID tokens Groups appear in  cognito:groups  claim Custom attributes prefixed with  custom: Lambda triggers for claim customization


### Auth0

Auth0 is a flexible identity platform known for developer-friendly APIs and extensive customization.


### The Easy Way: cargo pmcp + CDK

The fastest path to production: Use  cargo pmcp  to configure OAuth with Auth0.


### Step 1: Initialize OAuth Configuration

```bash
# Initialize deployment with Auth0 OAuth
cargo pmcp deploy init --target pmcp-run --oauth auth0

# This creates/updates .pmcp/deploy.toml with:

# .pmcp/deploy.toml
[auth]
enabled = true
provider = "auth0"
domain = "your-tenant.auth0.com"  # Set this to your Auth0 domain
audience = "https://mcp.example.com"  # Your API identifier

[auth.dcr]
# Dynamic Client Registration for MCP clients
enabled = true
public_client_patterns = [
    "claude",
    "cursor",
    "chatgpt",
    "mcp-inspector",
]
default_scopes = [
    "openid",
    "email",
    "read:tools",
]
allowed_scopes = [
    "openid",
    "email",
    "profile",
    "read:tools",
    "execute:tools",
    "read:resources",
    "write:resources",
    "admin",
]
```


### Step 2: Configure Auth0 (One-Time Setup)

Unlike Cognito, Auth0 isn't created by CDK—you configure it in Auth0 Dashboard.

```text
# After running deploy init, it outputs:
```


### #

```text
# Auth0 Setup Required:
# 1. Create API in Auth0 Dashboard
#    - Identifier: https://mcp.example.com
#    - Permissions: read:tools, execute:tools, read:resources, write:resources, admin
```


### #

```text
# 2. Create Application (Regular Web Application)
#    - Callback URLs: https://your-deployment.pmcp.run/callback
```


### #

```text
# 3. Set environment variables or update deploy.toml:
#    AUTH0_DOMAIN=your-tenant.auth0.com
#    AUTH0_AUDIENCE=https://mcp.example.com
```


### Step 3: Deploy

```bash
# Build and deploy
cargo pmcp deploy

# The deployment:
# - Configures Lambda with Auth0 environment variables
# - Sets up JWT validation middleware
# - Your server validates Auth0 tokens automatically
```

```rust
use pmcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // OAuth configuration loaded from environment
    // (AUTH0_DOMAIN, AUTH0_AUDIENCE set by deployment)
    let server = ServerBuilder::new("my-server", "1.0.0")
        .with_oauth_from_env()  // Works with any provider
        .with_tool(MyTool)
        .build()?;

    server.serve().await
}
```


### Manual Setup (When You Need Control)

If you need more control over Auth0 configuration, or your organization has specific Auth0 requirements, you can configure it manually.


### Auth0 Architecture

```text
                 Auth0 for MCP Servers
 Application               API
(MCP Client)           (MCP Server)
        Auth Code Flow         Validates JWT
                       Auth0 Tenant
   Connections          Rules/              RBAC
   (Database,          Actions          (Roles &
   Social,            (Customize         Permissions)
   Enterprise)         tokens)
```

(table/list, cells in reading order)
- Setting Up Auth0
- Create Tenant and API
- 1. Create Auth0 account at auth0.com
- 2. Create API (represents your MCP server):

Go to Applications → APIs → Create API Name: "MCP Server" Identifier:  https://mcp.example.com  (your audience) Signing Algorithm: RS256


### 3. Create Application (represents MCP clients):

Go to Applications → Create Application Type: Regular Web Application (for server-side) Configure callback URLs


### Define Permissions

```text
// API Settings → Permissions
{
  "permissions": [
    { "value": "read:tools", "description": "List and describe tools" },
    { "value": "execute:tools", "description": "Execute MCP tools" },
    { "value": "read:resources", "description": "Read resources" },
    { "value": "write:resources", "description": "Modify resources" },
    { "value": "admin", "description": "Administrative access" }
  ]
}
```


### Create Roles

```text
// User Management → Roles
{
  "roles": [
    {
      "name": "MCP User",
      "permissions": ["read:tools", "execute:tools", "read:resources"]
    },
    {
      "name": "MCP Admin",
      "permissions": ["read:tools", "execute:tools", "read:resources", "write:resources", "admin"]
    }
  ]
}
```

```rust
#[derive(Debug, Clone)]
pub struct Auth0Config {
    pub domain: String,
    pub audience: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl Auth0Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            domain: env::var("AUTH0_DOMAIN")?,
            audience: env::var("AUTH0_AUDIENCE")?,
            client_id: env::var("AUTH0_CLIENT_ID")?,
            client_secret: env::var("AUTH0_CLIENT_SECRET").ok(),
        })
    }

    pub fn issuer(&self) -> String {
        format!("https://{}/", self.domain)
    }

    pub fn jwks_uri(&self) -> String {
        format!("https://{}/.well-known/jwks.json", self.domain)
    }

    pub fn token_endpoint(&self) -> String {
        format!("https://{}/oauth/token", self.domain)
    }
}
```


### Validator Setup

```rust
impl JwtValidatorConfig {
    pub fn from_auth0(config: &Auth0Config) -> Self {
        Self {
            issuer: config.issuer(),
            audience: config.audience.clone(),
            jwks_uri: config.jwks_uri(),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 60,
        }
    }
}
```


### Auth0 Claims

```rust
#[derive(Debug, Deserialize)]
pub struct Auth0Claims {
    // Standard OIDC claims
    pub sub: String,              // "auth0|123" or "google-oauth2|456"
    pub iss: String,
    pub aud: ClaimAudience,
    pub exp: u64,
    pub iat: u64,
    pub azp: Option<String>,      // Authorized party (client_id)

    // User info
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub picture: Option<String>,

    // RBAC permissions (requires API setting)
    pub permissions: Option<Vec<String>>,

    // Scope string
    pub scope: Option<String>,

    // Custom claims (namespaced)
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Auth0Claims {
    /// Get namespaced custom claim
    pub fn get_custom(&self, namespace: &str, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(&format!("{}/{}", namespace, key))
    }

    /// Get permissions (from RBAC or scope)
    pub fn permissions_list(&self) -> Vec<String> {

        self.permissions.clone().unwrap_or_else(|| {
            self.scope
                .as_ref()
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default()
        })
    }

    /// Parse identity provider from sub claim
    pub fn identity_provider(&self) -> &str {
        self.sub.split('|').next().unwrap_or("unknown")
    }
}
```

(table/list, cells in reading order)
- Role-Based Access Control (RBAC)
- Enable RBAC
- In Auth0 Dashboard → APIs → Your API → Settings:
- Enable RBAC: ON Add Permissions in the Access Token: ON
- Permissions in Token
- With RBAC enabled, permissions appear in the access token:

```json
{
  "iss": "https://your-tenant.auth0.com/",
  "sub": "auth0|123456",
  "aud": "https://mcp.example.com",
  "permissions": [
    "read:tools",
    "execute:tools",
    "read:resources"
  ]
}
```


### Authorization in Rust

```rust
impl AuthContext {
    pub fn from_auth0_claims(claims: &Auth0Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            email: claims.email.clone(),
            name: claims.name.clone(),
            scopes: claims.permissions_list().into_iter().collect(),
        }
    }
}

// Use in tools
pub async fn run(&self, input: Input, context: &ToolContext) -> Result<Output> {
    let auth = context.auth()?;

    // Check for specific permission
    auth.require_scope("execute:tools")?;

    // Or check any of multiple permissions
    auth.require_any_scope(&["admin", "write:resources"])?;

    // Proceed with operation
}
```

```text
// Actions → Flows → Login → Add Action
```


### exports.onExecutePostLogin = async (event, api) => {

```text
  // Add custom claims (must be namespaced)
  const namespace = 'https://mcp.example.com';

  // Add user metadata
  if (event.user.app_metadata.department) {
    api.accessToken.setCustomClaim(
      `${namespace}/department`,
      event.user.app_metadata.department
    );
  }

  // Add organization info
  if (event.organization) {
    api.accessToken.setCustomClaim(
      `${namespace}/org_id`,
      event.organization.id
    );
    api.accessToken.setCustomClaim(
      `${namespace}/org_name`,
      event.organization.name
    );
  }

  // Add custom permissions based on conditions
  if (event.user.email.endsWith('@admin.example.com')) {
    // Get existing permissions
    const permissions = event.authorization?.permissions || [];
    permissions.push('admin:*');
    api.accessToken.setCustomClaim('permissions', permissions);

  }
};
```


### Handle Custom Claims in Rust

```rust
impl Auth0Claims {
    pub fn department(&self) -> Option<String> {
        self.get_custom("https://mcp.example.com", "department")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn org_id(&self) -> Option<String> {
        self.get_custom("https://mcp.example.com", "org_id")
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}
```

1.

```json
{
  "mappings": {
    "email": "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress",
    "given_name": "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname",
    "family_name": "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname",
    "department": "Department",
    "groups": "Groups"
  }
}
```

```rust
// Enable organizations in Auth0 Dashboard
// Applications → Your App → Organizations → Enable

// Token will include organization claim
{
  "org_id": "org_abc123",
  "org_name": "Acme Corp"
}

impl Auth0Claims {
    pub fn organization(&self) -> Option<(String, Option<String>)> {
        let org_id = self.custom.get("org_id")
            .and_then(|v| v.as_str())
            .map(String::from)?;

        let org_name = self.custom.get("org_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        Some((org_id, org_name))
    }
}
```

```bash
# Enable Password grant in Application settings first
curl --request POST \
  --url 'https://your-tenant.auth0.com/oauth/token' \
  --header 'content-type: application/x-www-form-urlencoded' \
  --data grant_type=password \
  --data 'username=test@example.com' \
  --data 'password=TestPass123!' \
  --data 'client_id=YOUR_CLIENT_ID' \
  --data 'client_secret=YOUR_CLIENT_SECRET' \
  --data 'audience=https://mcp.example.com' \
  --data 'scope=openid email profile'
```


### Get Test Token (Client Credentials)

```bash
# For machine-to-machine testing
curl --request POST \
  --url 'https://your-tenant.auth0.com/oauth/token' \
  --header 'content-type: application/x-www-form-urlencoded' \
  --data grant_type=client_credentials \
  --data 'client_id=YOUR_CLIENT_ID' \
  --data 'client_secret=YOUR_CLIENT_SECRET' \
  --data 'audience=https://mcp.example.com'
```


### Integration Test

```rust
#[tokio::test]
#[ignore]
async fn test_auth0_validation() {
    let config = Auth0Config::from_env().unwrap();
    let validator = JwtValidator::new(JwtValidatorConfig::from_auth0(&config));

    // Get token
    let token = get_auth0_token(&config).await.unwrap();

    // Validate
    let claims = validator.validate(&token).await.unwrap();

    assert!(!claims.sub.is_empty());
    println!("User: {}", claims.sub);
    println!("Permissions: {:?}", claims.permissions);
}

async fn get_auth0_token(config: &Auth0Config) -> Result<String> {
    let client = reqwest::Client::new();

    let response: serde_json::Value = client
        .post(&config.token_endpoint())
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", &config.client_id),
            ("client_secret", config.client_secret.as_ref().unwrap()),
            ("audience", &config.audience),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(response["access_token"].as_str().unwrap().to_string())
}
```


### Summary

Recommended approach: Use  cargo pmcp deploy init --oauth auth0  to generate deployment configuration.


### If you need manual setup, Auth0 integration provides:

1.


### Key Auth0-specific considerations:

Permissions via RBAC appear in  permissions  array Custom claims require namespacing (e.g.,  https://mcp.example.com/claim ) sub  format:  provider|id  (e.g.,  auth0|123 ,  google-oauth2|456 ) Actions for advanced token customization


### Microsoft Entra ID

Microsoft Entra ID (formerly Azure Active Directory) is the identity platform for Microsoft 365 enterprises.


### The Easy Way: cargo pmcp + CDK

The fastest path to production: Use  cargo pmcp  to configure OAuth with Entra ID.


### Step 1: Initialize OAuth Configuration

```bash
# Initialize deployment with Entra ID OAuth
cargo pmcp deploy init --target pmcp-run --oauth entra

# This creates/updates .pmcp/deploy.toml with:

# .pmcp/deploy.toml
[auth]
enabled = true
provider = "entra"
tenant_id = "your-tenant-id"  # From Azure Portal
client_id = "your-client-id"  # From App Registration

[auth.dcr]
# Dynamic Client Registration for MCP clients
enabled = true
public_client_patterns = [
    "claude",
    "cursor",
    "chatgpt",
    "mcp-inspector",
]
default_scopes = [
    "openid",
    "email",
    "profile",
]
```


### Step 2: Configure Entra ID (One-Time Setup)

Entra ID resources are managed in Azure Portal.  cargo pmcp  tells you what to create:


### #

```text
# Entra ID Setup Required:
# 1. Create App Registration in Azure Portal
#    - Go to: Entra ID → App registrations → New registration
#    - Name: "MCP Server - Production"
#    - Redirect URI: https://your-deployment.pmcp.run/callback
```


### #

```text
# 2. Configure App Roles (App registration → App roles):
#    - MCP.User: Can read and execute tools
#    - MCP.Admin: Full administrative access
```


### #

```text
# 3. Configure Token (App registration → Token configuration):
#    - Add optional claims: email, groups
```


### #

```text
# 4. Record these values for deploy.toml:
#    - Application (client) ID
#    - Directory (tenant) ID
```


### #

```text
# 5. Set environment variables or update deploy.toml:
#    ENTRA_TENANT_ID=your-tenant-id
#    ENTRA_CLIENT_ID=your-client-id
```


### Step 3: Deploy

```bash
# Build and deploy
cargo pmcp deploy

# The deployment:
# - Configures Lambda with Entra ID environment variables
# - Sets up JWT validation middleware with correct issuer/JWKS
# - Your server validates Entra ID tokens automatically
```

```rust
use pmcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // OAuth configuration loaded from environment
    // (ENTRA_TENANT_ID, ENTRA_CLIENT_ID set by deployment)
    let server = ServerBuilder::new("my-server", "1.0.0")
        .with_oauth_from_env()  // Works with any provider
        .with_tool(MyTool)
        .build()?;

    server.serve().await
}
```


### Why Entra ID for Microsoft Shops

If your organization uses Microsoft 365, Entra ID is the natural choice:

(table/list, cells in reading order)
- Benefit
- Same login
- AD groups
- Description
- Employees use their Microsoft 365 credentials
- Existing Active Directory groups work for MCP permissions
- SSO everywhere
- MCP access works like Teams, Outlook, SharePoint
- IT familiarity
- Your IT team already knows Entra ID
- Conditional Access
- Apply existing security policies to MCP
- Manual Setup (When You Need Control)


### Entra ID Architecture

```text
                  Entra ID for MCP Servers
Microsoft 365 Tenant
        Users              Groups           App Roles
      (Employees)       (AD Groups)         (Defined in
                                             App Reg)
   App Registration (MCP Server)
      Client ID
      API permissions
      App roles
Token Flow:
User → Entra ID → JWT with oid, groups, roles → MCP Server
```

(table/list, cells in reading order)
- Setting Up App Registration
- Azure Portal
- 1. Go to Entra ID → App registrations → New registration
- 2. Configure:

Name: "MCP Server" Supported account types: Single tenant or multi-tenant Redirect URI: Web,  https://your-app/callback

(table/list, cells in reading order)
- 3. Record:
- Application (client) ID Directory (tenant) ID
- Define App Roles
- In App registration → App roles → Create app role:

```json
{
  "appRoles": [
    {
      "displayName": "MCP User",
      "value": "MCP.User",
      "description": "Can read and execute MCP tools",
      "allowedMemberTypes": ["User"]
    },
    {
      "displayName": "MCP Admin",
      "value": "MCP.Admin",
      "description": "Full administrative access",
      "allowedMemberTypes": ["User"]
    },
    {
      "displayName": "MCP Service",
      "value": "MCP.Service",
      "description": "Machine-to-machine access",
      "allowedMemberTypes": ["Application"]
    }
  ]
}
```

(table/list, cells in reading order)
- Configure Token
- In App registration → Token configuration:
- 1. Add optional claims:
- email
- given_name
- family_name groups  (group membership)
- 2. For groups claim, configure:
- Emit groups as role claims: Security groups
- Rust Integration
- Configuration

```rust
#[derive(Debug, Clone)]
pub struct EntraConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl EntraConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            tenant_id: env::var("ENTRA_TENANT_ID")
                .or_else(|_| env::var("AZURE_TENANT_ID"))?,
            client_id: env::var("ENTRA_CLIENT_ID")
                .or_else(|_| env::var("AZURE_CLIENT_ID"))?,
            client_secret: env::var("ENTRA_CLIENT_SECRET")
                .or_else(|_| env::var("AZURE_CLIENT_SECRET"))
                .ok(),
        })
    }

    pub fn issuer(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/v2.0",
            self.tenant_id
        )
    }

    pub fn jwks_uri(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
            self.tenant_id
        )

    }


    pub fn token_endpoint(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant_id
        )
    }
}
```


### Validator Setup

```rust
impl JwtValidatorConfig {
    pub fn from_entra(config: &EntraConfig) -> Self {
        Self {
            issuer: config.issuer(),
            audience: config.client_id.clone(),
            jwks_uri: config.jwks_uri(),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 300, // Entra recommends 5 minutes
        }
    }
}
```


### Entra ID Claims

```rust
#[derive(Debug, Deserialize)]
pub struct EntraClaims {
    // Standard claims
    pub sub: String,
    pub iss: String,
    pub aud: ClaimAudience,
    pub exp: u64,
    pub iat: u64,
    pub nbf: u64,

    // Entra-specific identifiers
    pub oid: String,                      // Object ID (user GUID)
    pub tid: String,                      // Tenant ID
    pub azp: Option<String>,              // Authorized party (client_id)

    // User info
    pub preferred_username: Option<String>, // UPN (user@domain.com)
    pub email: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,

    // Groups (GUIDs)
    pub groups: Option<Vec<String>>,

    // App roles (from app registration)
    pub roles: Option<Vec<String>>,

    // For multi-tenant apps
    pub idp: Option<String>,              // Identity provider
}

impl EntraClaims {
    /// User's UPN (email-like identifier)
    pub fn upn(&self) -> Option<&str> {
        self.preferred_username.as_deref()

            .or(self.email.as_deref())
    }

    /// Primary identifier - use oid for consistency
    pub fn user_id(&self) -> &str {
        &self.oid
    }

    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.as_ref()
            .map(|r| r.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    /// Check if user is in group (by GUID)
    pub fn in_group(&self, group_id: &str) -> bool {
        self.groups.as_ref()
            .map(|g| g.iter().any(|g| g == group_id))
            .unwrap_or(false)
    }
}
```

```rust
pub struct GroupPermissionMapper {
    // Map group GUIDs to permission sets
    group_permissions: HashMap<String, Vec<String>>,
}

impl GroupPermissionMapper {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Configure your group mappings
        map.insert(
            "12345678-1234-1234-1234-123456789abc".into(), // Admins group GUID
            vec!["admin:*".into(), "execute:tools".into(), "read:tools".into()],
        );
        map.insert(
            "87654321-4321-4321-4321-cba987654321".into(), // Developers group
            vec!["execute:tools".into(), "read:tools".into()],
        );

        Self { group_permissions: map }
    }

    pub fn permissions_for_groups(&self, groups: &[String]) -> HashSet<String> {
        groups.iter()
            .filter_map(|g| self.group_permissions.get(g))
            .flatten()
            .cloned()
            .collect()
    }
}

impl AuthContext {

    pub fn from_entra_claims(claims: &EntraClaims, mapper: &GroupPermissionMapper) -> Self {
        let mut scopes: HashSet<String> = HashSet::new();

        // Add role-based permissions
        if let Some(roles) = &claims.roles {
            for role in roles {
                match role.as_str() {
                    "MCP.Admin" => {
                        scopes.insert("admin:*".into());
                    }
                    "MCP.User" => {
                        scopes.insert("execute:tools".into());
                        scopes.insert("read:tools".into());
                    }
                    _ => {}
                }
            }
        }

        // Add group-based permissions
        if let Some(groups) = &claims.groups {
            scopes.extend(mapper.permissions_for_groups(groups));
        }

        Self {
            user_id: claims.oid.clone(),
            email: claims.upn().map(String::from),
// ... truncated, 5 more lines in source book ...
```

```rust
pub async fn get_obo_token(
    config: &EntraConfig,
    user_token: &str,
    target_scope: &str,
) -> Result<String> {
    let client = reqwest::Client::new();

    let response: serde_json::Value = client
        .post(&config.token_endpoint())
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("client_id", &config.client_id),
            ("client_secret", config.client_secret.as_ref().unwrap()),
            ("assertion", user_token),
            ("scope", target_scope),
            ("requested_token_use", "on_behalf_of"),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(response["access_token"].as_str().unwrap().to_string())
}
```

(table/list, cells in reading order)
- Multi-Tenant Applications
- For apps serving multiple organizations:
- Configure Multi-Tenant
- In App registration:
- Supported account types: "Accounts in any organizational directory"
- Validate Any Tenant

```rust
impl JwtValidatorConfig {
    pub fn from_entra_multitenant(client_id: &str) -> Self {
        Self {
            // Use 'common' endpoint for multi-tenant
            issuer: "https://login.microsoftonline.com/{tenantid}/v2.0".into(),
            audience: client_id.to_string(),
            jwks_uri: "https://login.microsoftonline.com/common/discovery/v2.0/keys".into(),
            algorithms: vec![Algorithm::RS256],
            leeway_seconds: 300,
        }
    }
}

// Custom validation for multi-tenant
impl JwtValidator {
    pub async fn validate_multitenant(&self, token: &str) -> Result<EntraClaims> {
        let claims: EntraClaims = self.decode_without_validation(token)?;

        // Verify issuer matches tenant in token
        let expected_issuer = format!(
            "https://login.microsoftonline.com/{}/v2.0",
            claims.tid
        );
        if claims.iss != expected_issuer {
            return Err(AuthError::ValidationFailed("Invalid issuer".into()));
        }

        // Continue with normal validation
        self.validate(token).await
    }
}
```

```bash
# Login
az login

# Get token for your app
az account get-access-token \
  --resource api://your-client-id \
  --query accessToken -o tsv
```


### Client Credentials (Service Principal)

```bash
curl -X POST \
  "https://login.microsoftonline.com/${TENANT_ID}/oauth2/v2.0/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=${CLIENT_ID}" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "scope=api://${CLIENT_ID}/.default" \
  -d "grant_type=client_credentials"
```


### Integration Test

```rust
#[tokio::test]
#[ignore]
async fn test_entra_validation() {
    let config = EntraConfig::from_env().unwrap();
    let validator = JwtValidator::new(JwtValidatorConfig::from_entra(&config));

    // Get token via Azure SDK or CLI
    let token = get_entra_token(&config).await.unwrap();

    let claims = validator.validate(&token).await.unwrap();

    assert!(!claims.sub.is_empty());
    println!("User OID: {}", claims.oid);
    println!("UPN: {:?}", claims.preferred_username);
    println!("Roles: {:?}", claims.roles);
}
```


### Summary

Recommended approach: Use  cargo pmcp deploy init --oauth entra  to generate deployment configuration.


### If you need manual setup, Microsoft Entra ID integration requires:

1.


### Key Entra-specific considerations:

Use  oid  (Object ID) as the stable user identifier, not  sub Roles appear in  roles  array (from app roles you define) Groups are GUIDs—you need to map them to human-readable permissions Multi-tenant apps require special issuer validation (tenant ID varies) 5-minute clock skew recommended (Entra's 


### Multi-Tenant Considerations

Multi-tenant MCP servers serve multiple organizations from a single deployment.

Most organizations don't.


### Scenario

Multi-Tenant?

(table/list, cells in reading order)
- Why
- Internal MCP server for your company
- Department-specific servers
- SaaS product serving multiple customers
- Partner integrations with isolated data
- No
- No
- Yes
- Yes
- Single organization, use your IdP directly
- Deploy separate servers per department
- Multiple organizations, shared infrastructure
- Multiple external organizations
- Enterprise platform with subsidiaries
- Maybe
- Could use separate deployments or multi-tenant

The rule of thumb: If all your users come from the same organization (even with different teams or roles), you don't need multi- tenancy.


### Multi-tenancy adds significant complexity:

Tenant isolation at every layer (code, data, rate limits) Cross-tenant attack surface to protect Tenant provisioning and lifecycle management Complex debugging (which tenant had the issue?)


### The Easy Way: cargo pmcp Multi-Tenant Mode

If you do need multi-tenancy,  cargo pmcp  provides configuration support:

```bash
# Initialize with multi-tenant support
cargo pmcp deploy init --target pmcp-run --oauth auth0 --multi-tenant

# This creates/updates .pmcp/deploy.toml with:

# .pmcp/deploy.toml
[auth]
enabled = true
provider = "auth0"  # Or cognito, entra—any provider works
domain = "your-tenant.auth0.com"

[auth.multi_tenant]
enabled = true
# How to identify the tenant from the JWT
tenant_claim = "org_id"  # Auth0 Organizations
# Or: "tid" for Entra ID
# Or: "custom:tenant_id" for Cognito

# Tenant isolation strategy
isolation = "row_level_security"  # Or "schema_per_tenant", "prefix"

# Default rate limit per tenant (requests per minute)
default_rate_limit = 100
```


### Your tools receive the tenant automatically:

```rust
pub async fn run(
    &self,
    input: Input,
    context: &ToolContext,
) -> Result<Output> {
    // Tenant is extracted from JWT by middleware
    let tenant = context.tenant()?;  // TenantContext

    // All database operations automatically scoped
    let data = self.db.query(&tenant, "SELECT * FROM resources").await?;

    Ok(Output { data })
}
```


### Manual Setup (For Complex Requirements)

If you need custom tenant resolution, complex isolation patterns, or cross-tenant admin operations, configure multi-tenancy manually.


### Multi-Tenant Architecture

```text
              Multi-Tenant MCP Architecture
Organization A          Organization B          Organization C
  MCP Client             MCP Client             MCP Client
         JWT (tenant_a)         JWT (tenant_b)         JWT (c)
                       MCP Server
                       • Extract tenant
                       • Validate access
                       • Isolate data
  Tenant A               Tenant B               Tenant C
  Data/Config            Data/Config            Data/Config
```

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: Option<String>,
    pub user_id: String,
    pub scopes: Vec<String>,
}

impl TenantContext {
    /// Extract tenant from Cognito claims
    pub fn from_cognito(claims: &CognitoClaims) -> Result<Self, TenantError> {
        // Cognito: Use custom attribute or user pool ID
        let tenant_id = claims
            .get_custom("tenant_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or(TenantError::MissingTenant)?;

        Ok(Self {
            tenant_id,
            tenant_name: claims.get_custom("tenant_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            user_id: claims.sub.clone(),
            scopes: claims.scope_list(),
        })
    }

    /// Extract tenant from Auth0 claims
    pub fn from_auth0(claims: &Auth0Claims) -> Result<Self, TenantError> {
        // Auth0: Use organization claim
        let tenant_id = claims
            .custom.get("org_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or(TenantError::MissingTenant)?;

        Ok(Self {
            tenant_id,
            tenant_name: claims.custom.get("org_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            user_id: claims.sub.clone(),
            scopes: claims.permissions_list(),
        })
    }

    /// Extract tenant from Entra ID claims
    pub fn from_entra(claims: &EntraClaims) -> Result<Self, TenantError> {
        // Entra: Use tid (tenant ID) claim
        Ok(Self {
            tenant_id: claims.tid.clone(),
            tenant_name: None, // Can be fetched from Graph API
            user_id: claims.oid.clone(),
            scopes: claims.roles.clone().unwrap_or_default(),
        })
    }
// ... truncated, 13 more lines in source book ...
```

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub config: TenantConfig,
    pub status: TenantStatus,
}

#[derive(Debug, Clone)]
pub struct TenantConfig {
    pub database_schema: String,
    pub storage_prefix: String,
    pub rate_limit: u32,
    pub allowed_tools: Vec<String>,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Trial { expires_at: u64 },
}

pub struct TenantRegistry {
    tenants: Arc<RwLock<HashMap<String, TenantInfo>>>,
}

impl TenantRegistry {
    pub async fn get(&self, tenant_id: &str) -> Result<TenantInfo, TenantError> {
        let tenants = self.tenants.read().await;

        let tenant = tenants.get(tenant_id)
            .ok_or_else(|| TenantError::UnknownTenant(tenant_id.to_string()))?;

        // Check tenant status

        match &tenant.status {
            TenantStatus::Active => Ok(tenant.clone()),
            TenantStatus::Suspended => {
                Err(TenantError::AccessDenied("Tenant suspended".into()))
            }
            TenantStatus::Trial { expires_at } => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > *expires_at {
                    Err(TenantError::AccessDenied("Trial expired".into()))
                } else {
                    Ok(tenant.clone())
                }
            }
        }
    }

// ... truncated, 6 more lines in source book ...
```

```rust
use sqlx::{Pool, Postgres};

pub struct SchemaIsolatedDb {
    pool: Pool<Postgres>,
}

impl SchemaIsolatedDb {
    /// Execute query in tenant's schema
    pub async fn query_tenant<T>(
        &self,
        tenant: &TenantContext,
        query: &str,
    ) -> Result<Vec<T>, DbError>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        // Set search_path to tenant's schema
        let schema = self.tenant_schema(&tenant.tenant_id);

        sqlx::query(&format!("SET search_path TO {}", schema))
            .execute(&self.pool)
            .await?;

        let results = sqlx::query_as::<_, T>(query)
            .fetch_all(&self.pool)
            .await?;

        // Reset to public schema
        sqlx::query("SET search_path TO public")
            .execute(&self.pool)
            .await?;

        Ok(results)
    }

    fn tenant_schema(&self, tenant_id: &str) -> String {
        // Sanitize tenant_id to prevent SQL injection
        let safe_id = tenant_id
            .chars()

            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        format!("tenant_{}", safe_id)
    }

    /// Create schema for new tenant
    pub async fn provision_tenant(&self, tenant_id: &str) -> Result<(), DbError> {
        let schema = self.tenant_schema(tenant_id);

        // Create schema
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema))
            .execute(&self.pool)
            .await?;

        // Run migrations in tenant schema
        sqlx::query(&format!("SET search_path TO {}", schema))
            .execute(&self.pool)
            .await?;

// ... truncated, 15 more lines in source book ...
```

```text
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

-- Create index for tenant filtering CREATE INDEX idx_resources_tenant ON resources(tenant_id);


### -- Enable RLS ALTER TABLE resources ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only see their tenant's data CREATE POLICY tenant_isolation ON resources

```text
    FOR ALL
    USING (tenant_id = current_setting('app.tenant_id'));
```

```rust
pub struct RlsIsolatedDb {
    pool: Pool<Postgres>,
}

impl RlsIsolatedDb {
    /// Execute query with tenant context
    pub async fn with_tenant<F, T>(
        &self,
        tenant: &TenantContext,
        operation: F,
    ) -> Result<T, DbError>
    where
        F: FnOnce(&Pool<Postgres>) -> futures::future::BoxFuture<'_, Result<T, DbError>>,
    {
        // Set tenant context for RLS
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
            .bind(&tenant.tenant_id)
            .execute(&self.pool)
            .await?;

        // Execute the operation
        operation(&self.pool).await
    }

    /// Query resources (automatically filtered by RLS)
    pub async fn list_resources(
        &self,
        tenant: &TenantContext,
    ) -> Result<Vec<Resource>, DbError> {
        self.with_tenant(tenant, |pool| {
            Box::pin(async move {
                sqlx::query_as::<_, Resource>("SELECT * FROM resources")
                    .fetch_all(pool)
                    .await
                    .map_err(DbError::from)
            })
        }).await

    }
}
```

```rust
pub struct PrefixIsolatedStorage {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl PrefixIsolatedStorage {
    /// Get object with tenant prefix
    pub async fn get(
        &self,
        tenant: &TenantContext,
        key: &str,
    ) -> Result<Vec<u8>, StorageError> {
        let prefixed_key = self.tenant_key(tenant, key);

        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&prefixed_key)
            .send()
            .await?;

        let bytes = response.body.collect().await?.into_bytes();
        Ok(bytes.to_vec())
    }

    /// Put object with tenant prefix
    pub async fn put(
        &self,
        tenant: &TenantContext,
        key: &str,
        data: Vec<u8>,
    ) -> Result<(), StorageError> {
        let prefixed_key = self.tenant_key(tenant, key);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&prefixed_key)
            .body(data.into())

            .send()
            .await?;

        Ok(())
    }

    /// List objects for tenant
    pub async fn list(
        &self,
        tenant: &TenantContext,
        prefix: &str,
    ) -> Result<Vec<String>, StorageError> {
        let full_prefix = self.tenant_key(tenant, prefix);

        let response = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&full_prefix)
            .send()
            .await?;
// ... truncated, 16 more lines in source book ...
```

```rust
use mcp_server::{Tool, ToolContext, ToolError};

pub struct ListDocumentsTool {
    storage: Arc<PrefixIsolatedStorage>,
}

impl Tool for ListDocumentsTool {
    type Input = ListDocumentsInput;
    type Output = ListDocumentsOutput;

    fn name(&self) -> &str {
        "list_documents"
    }

    async fn run(
        &self,
        input: Self::Input,
        context: &ToolContext,
    ) -> Result<Self::Output, ToolError> {
        // Get tenant from context (extracted from JWT by middleware)
        let tenant = context.tenant()
            .ok_or_else(|| ToolError::Unauthorized("Missing tenant context"))?;

        // Operation is automatically scoped to tenant
        let documents = self.storage
            .list(&tenant, &input.prefix.unwrap_or_default())
            .await
            .map_err(|e| ToolError::Internal(e.to_string()))?;

        Ok(ListDocumentsOutput { documents })

    }
}
```


### Tenant-Specific Tool Configuration

```rust
pub struct TenantAwareToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    tenant_registry: Arc<TenantRegistry>,
}

impl TenantAwareToolRegistry {
    /// List tools available to tenant
    pub async fn list_for_tenant(
        &self,
        tenant: &TenantContext,
    ) -> Result<Vec<ToolInfo>, ToolError> {
        let tenant_info = self.tenant_registry
            .get(&tenant.tenant_id)
            .await
            .map_err(|e| ToolError::Unauthorized(e.to_string()))?;

        // Filter tools based on tenant's allowed list
        let available: Vec<_> = self.tools
            .iter()
            .filter(|(name, _)| {
                tenant_info.config.allowed_tools.is_empty() ||
                tenant_info.config.allowed_tools.contains(*name)
            })
            .map(|(name, tool)| ToolInfo {
                name: name.clone(),
                description: tool.description().to_string(),
            })
            .collect();

        Ok(available)
    }

    /// Check if tenant can use tool
    pub async fn can_use(
        &self,
        tenant: &TenantContext,

        tool_name: &str,
    ) -> Result<bool, ToolError> {
        let tenant_info = self.tenant_registry
            .get(&tenant.tenant_id)
            .await
            .map_err(|e| ToolError::Unauthorized(e.to_string()))?;

        // Empty allowed_tools means all tools are allowed
        if tenant_info.config.allowed_tools.is_empty() {
            return Ok(true);
        }

        Ok(tenant_info.config.allowed_tools.contains(&tool_name.to_string()))
    }
}
```


### Rate Limiting Per Tenant

```rust
use std::time::{Duration, Instant};
use dashmap::DashMap;

pub struct TenantRateLimiter {
    limits: DashMap<String, RateLimitState>,
    default_limit: u32,
}

struct RateLimitState {
    tokens: u32,
    last_refill: Instant,
    limit: u32,
}

impl TenantRateLimiter {
    pub fn new(default_limit: u32) -> Self {
        Self {
            limits: DashMap::new(),
            default_limit,
        }
    }

    /// Check and consume rate limit
    pub async fn check(
        &self,
        tenant: &TenantContext,
        tenant_info: &TenantInfo,
    ) -> Result<(), RateLimitError> {
        let limit = if tenant_info.config.rate_limit > 0 {
            tenant_info.config.rate_limit
        } else {
            self.default_limit
        };

        let mut state = self.limits
            .entry(tenant.tenant_id.clone())

            .or_insert_with(|| RateLimitState {
                tokens: limit,
                last_refill: Instant::now(),
                limit,
            });

        // Refill tokens (1 per second)
        let elapsed = state.last_refill.elapsed();
        let refill = elapsed.as_secs() as u32;
        if refill > 0 {
            state.tokens = (state.tokens + refill).min(state.limit);
            state.last_refill = Instant::now();
        }

        // Consume token
        if state.tokens > 0 {
            state.tokens -= 1;
            Ok(())
        } else {
            Err(RateLimitError::Exceeded {
                tenant_id: tenant.tenant_id.clone(),
                retry_after: Duration::from_secs(1),
            })
// ... truncated, 12 more lines in source book ...
```

```rust
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract auth context (set by auth middleware)
    let auth = request.extensions()
        .get::<AuthContext>()
        .ok_or(ApiError::Unauthorized)?;

    // Extract tenant from claims
    let tenant_context = match &state.idp_type {
        IdpType::Cognito => TenantContext::from_cognito(&auth.claims)?,
        IdpType::Auth0 => TenantContext::from_auth0(&auth.claims)?,
        IdpType::Entra => TenantContext::from_entra(&auth.claims)?,
    };

    // Validate tenant
    let tenant_info = state.tenant_registry
        .get(&tenant_context.tenant_id)
        .await?;

    // Check rate limit
    state.rate_limiter
        .check(&tenant_context, &tenant_info)
        .await?;

    // Add tenant context to request
    request.extensions_mut().insert(tenant_context);
    request.extensions_mut().insert(tenant_info);

    Ok(next.run(request).await)
}
```


### Request Context Extraction

```rust
use axum::extract::FromRequestParts;

pub struct Tenant(pub TenantContext);

#[async_trait]
```


### impl<S> FromRequestParts<S> for Tenant where

```rust
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions
            .get::<TenantContext>()
            .cloned()
            .map(Tenant)
            .ok_or(ApiError::MissingTenantContext)
    }
}

// Use in handlers
async fn list_resources(
    Tenant(tenant): Tenant,
    State(state): State<AppState>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    let resources = state.db.list_resources(&tenant).await?;
    Ok(Json(resources))
}
```

```rust
#[derive(Debug, Clone)]
pub enum TenantScope {
    /// Operations scoped to a single tenant
    Single(TenantContext),

    /// Admin operations across all tenants
    Global { admin_id: String },
}

impl TenantScope {
    pub fn from_auth(auth: &AuthContext) -> Result<Self, TenantError> {
        // Check for global admin role
        if auth.has_scope("admin:global") {
            return Ok(TenantScope::Global {
                admin_id: auth.user_id.clone(),
            });
        }

        // Extract tenant for normal users
        let tenant = TenantContext::from_claims(&auth.claims)?;
        Ok(TenantScope::Single(tenant))
    }
}

pub struct AdminTool {
    db: Arc<RlsIsolatedDb>,
}

impl AdminTool {
    pub async fn list_all_tenants(
        &self,
        scope: &TenantScope,

    ) -> Result<Vec<TenantInfo>, ToolError> {
        match scope {
            TenantScope::Global { admin_id } => {
                tracing::info!(admin = %admin_id, "Listing all tenants");
                // Query without tenant filter
                self.db.list_all_tenants().await
            }
            TenantScope::Single(_) => {
                Err(ToolError::Forbidden("Global admin access required"))
            }
        }
    }

    pub async fn impersonate_tenant(
        &self,
        scope: &TenantScope,
        target_tenant_id: &str,
    ) -> Result<TenantContext, ToolError> {
        match scope {
            TenantScope::Global { admin_id } => {
                tracing::warn!(
                    admin = %admin_id,
                    tenant = %target_tenant_id,
                    "Admin impersonating tenant"
                );

                // Create impersonated context
// ... truncated, 14 more lines in source book ...
```

```rust
pub struct TenantProvisioner {
    db: Arc<SchemaIsolatedDb>,
    storage: Arc<PrefixIsolatedStorage>,
    registry: Arc<TenantRegistry>,
}

impl TenantProvisioner {
    pub async fn provision(&self, request: ProvisionRequest) -> Result<TenantInfo, ProvisionError> {
        let tenant_id = uuid::Uuid::new_v4().to_string();

        tracing::info!(tenant_id = %tenant_id, "Provisioning new tenant");

        // 1. Create database schema
        self.db.provision_tenant(&tenant_id).await?;

        // 2. Create storage prefix (just needs first write)
        self.storage.put(
            &TenantContext {
                tenant_id: tenant_id.clone(),
                tenant_name: Some(request.name.clone()),
                user_id: "system".into(),
                scopes: vec![],
            },
            ".tenant-marker",
            b"initialized".to_vec(),
        ).await?;

        // 3. Create tenant record
        let tenant_info = TenantInfo {
            id: tenant_id.clone(),
            name: request.name,
            config: TenantConfig {

                database_schema: format!("tenant_{}", tenant_id),
                storage_prefix: tenant_id.clone(),
                rate_limit: request.rate_limit.unwrap_or(100),
                allowed_tools: request.allowed_tools,
                feature_flags: request.feature_flags,
            },
            status: if request.trial_days > 0 {
                let expires_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + (request.trial_days as u64 * 86400);
                TenantStatus::Trial { expires_at }
            } else {
                TenantStatus::Active
            },
        };

        // 4. Register tenant
        self.registry.add(tenant_info.clone()).await?;

        tracing::info!(tenant_id = %tenant_id, "Tenant provisioned successfully");

        Ok(tenant_info)
    }

    pub async fn deprovision(&self, tenant_id: &str) -> Result<(), ProvisionError> {
        tracing::warn!(tenant_id = %tenant_id, "Deprovisioning tenant");
// ... truncated, 23 more lines in source book ...
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_tenant(id: &str) -> TenantContext {
        TenantContext {
            tenant_id: id.to_string(),
            tenant_name: Some(format!("Test Tenant {}", id)),
            user_id: format!("user-{}", id),
            scopes: vec!["read:tools".into(), "execute:tools".into()],
        }
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let storage = setup_test_storage().await;

        let tenant_a = test_tenant("tenant-a");
        let tenant_b = test_tenant("tenant-b");

        // Write data for tenant A
        storage.put(&tenant_a, "secret.txt", b"tenant-a-secret".to_vec())
            .await.unwrap();

        // Write data for tenant B
        storage.put(&tenant_b, "secret.txt", b"tenant-b-secret".to_vec())
            .await.unwrap();

        // Tenant A can only see their data
        let data_a = storage.get(&tenant_a, "secret.txt").await.unwrap();
        assert_eq!(data_a, b"tenant-a-secret");

        // Tenant B can only see their data
        let data_b = storage.get(&tenant_b, "secret.txt").await.unwrap();
        assert_eq!(data_b, b"tenant-b-secret");

        // Tenant A cannot access tenant B's data
        let list_a = storage.list(&tenant_a, "").await.unwrap();
        assert!(!list_a.iter().any(|k| k.contains("tenant-b")));
    }

    #[tokio::test]
    async fn test_cross_tenant_blocked() {
        let db = setup_test_db().await;

        let tenant_a = test_tenant("tenant-a");
        let tenant_b = test_tenant("tenant-b");

        // Create resource for tenant A
        db.with_tenant(&tenant_a, |pool| {
            Box::pin(async move {
                sqlx::query("INSERT INTO resources (name) VALUES ('secret')")
                    .execute(pool)
                    .await
                    .map_err(DbError::from)
            })
        }).await.unwrap();

        // Tenant B should not see tenant A's resource
        let resources = db.list_resources(&tenant_b).await.unwrap();
// ... truncated, 3 more lines in source book ...
```


### Integration Test Setup

```rust
#[cfg(test)]
pub struct MultiTenantTestHarness {
    server: TestServer,
    tenants: Vec<(TenantContext, String)>, // (context, token)
}

#[cfg(test)]
impl MultiTenantTestHarness {
    pub async fn setup(num_tenants: usize) -> Self {
        let server = TestServer::new().await;

        let mut tenants = Vec::new();
        for i in 0..num_tenants {
            let tenant_id = format!("test-tenant-{}", i);
            let context = TenantContext {
                tenant_id: tenant_id.clone(),
                tenant_name: Some(format!("Test Tenant {}", i)),
                user_id: format!("user-{}", i),
                scopes: vec!["read:tools".into(), "execute:tools".into()],
            };

            // Generate test token for tenant
            let token = generate_test_token(&context);
            tenants.push((context, token));
        }

        Self { server, tenants }
    }

    pub fn tenant(&self, index: usize) -> (&TenantContext, &str) {
        let (ctx, token) = &self.tenants[index];
        (ctx, token)
    }
}
```


### Summary

First, ask: Do you need multi-tenancy? Most organizations don't.


### For advanced requirements, multi-tenant MCP servers require:

1.


### Key security principles:

Defense in depth - Multiple isolation layers (middleware + database + storage) Fail secure - Default deny cross-tenant access; explicit allow only Audit everything - Log all operations with tenant ID Test isolation - Verify data cannot leak between tenants (write tests!) Minimize cross-tenant - Admi
