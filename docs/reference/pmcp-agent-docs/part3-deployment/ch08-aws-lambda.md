---
chapter: 8
title: "AWS Lambda Deployment"
part: "Part III: Deployment"
use_when: "Deploying to AWS Lambda: CDK, Lambda Web Adapter, VPC, secrets, cold starts, troubleshooting."
suggested_prerequisites: [ch07]
source_kb: 41
extracted_kb: 24
code_blocks: 52
quality: auto-extracted
---

# Ch 8: AWS Lambda Deployment

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### AWS Lambda Deployment

This chapter provides a comprehensive, hands-on guide to deploying MCP servers on AWS Lambda.

```bash
# AWS CLI configured with credentials
aws sts get-caller-identity

# Node.js for CDK (18+ recommended)
```


### node --version

```bash
# Cargo Lambda for cross-compilation
cargo install cargo-lambda

# AWS CDK CLI
npm install -g aws-cdk
```

```text
                      AWS LAMBDA MCP ARCHITECTURE
Internet
                       API Gateway (HTTP API)
       HTTPS           CORS            Lambda Authorizer
     Termination      Headers          (JWT validation)
                       Lambda Function
      Lambda Web Adapter
      (HTTP → Lambda event translation)
      Your MCP Server (StreamableHttpServer)
      - Tool handlers
      - Resource providers
      - Prompt workflows
    ▼ (VPC)
   Private Resources
       RDS         DynamoDB         S3          Secrets Manager
```


### Lambda Web Adapter

PMCP uses the Lambda Web Adapter to run standard HTTP servers on Lambda.

```rust
unchanged:

// The same code runs locally AND on Lambda
#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::builder()
        .name("my-mcp-server")
        .version("1.0.0")
        .tool("query", TypedTool::new(...))
        .build()?;

    // Lambda Web Adapter translates Lambda events to HTTP
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    StreamableHttpServer::new(server)
        .run(addr)
        .await
}
```


### The Lambda Web Adapter:

Receives Lambda invocation events from API Gateway Translates them to HTTP requests to localhost:8080 Forwards your HTTP response back as Lambda response Handles connection keep-alive for warm invocations

```bash
# From your MCP server project directory
cargo pmcp deploy init --target aws-lambda
```

```text
deploy.toml           # Deployment configuration
cdk/                  # CDK infrastructure
    bin/
        app.ts        # CDK app entry point
    lib/
        stack.ts      # Infrastructure stack
    package.json
    tsconfig.json
    cdk.json
```

```toml
[target]
target_type = "aws-lambda"

[server]
name = "my-mcp-server"
description = "Production MCP server for data queries"

[aws]
region = "us-east-1"
profile = "default"  # AWS CLI profile to use

[lambda]
memory_size = 256          # MB (128-10240)
timeout_seconds = 30       # seconds (1-900)
architecture = "arm64"     # arm64 (recommended) or x86_64
reserved_concurrency = 100 # Optional: limit concurrent executions

[lambda.environment]
RUST_LOG = "info"
# Add your environment variables here
# DATABASE_URL comes from Secrets Manager, not here

[api_gateway]
type = "http"              # "http" (recommended) or "rest"
stage_name = "prod"
throttling_rate = 1000     # requests per second
throttling_burst = 2000    # burst capacity

[auth]
enabled = true
provider = "cognito"       # or "custom" for bring-your-own

[vpc]
enabled = true             # Enable for RDS/private resource access
# VPC settings auto-discovered or specify:
# vpc_id = "vpc-12345"
# subnet_ids = ["subnet-a", "subnet-b"]
# security_group_ids = ["sg-12345"]
```


### Step 3: Build and Deploy

```bash
# Build for Lambda (cross-compiles to ARM64 Linux)
cargo pmcp deploy build

# Deploy infrastructure and function
cargo pmcp deploy

# View outputs (API URL, etc.)
cargo pmcp deploy outputs
```


### First deployment creates all AWS resources (~3-5 minutes):

Lambda function with Web Adapter layer API Gateway HTTP API with routes IAM roles and policies CloudWatch log groups (Optional) Cognito user pool (Optional) VPC configuration


### Step 4: Verify Deployment

```bash
# Get the API endpoint
cargo pmcp deploy outputs

# Output:
# ApiEndpoint: https://abc123.execute-api.us-east-1.amazonaws.com/prod
# McpEndpoint: https://abc123.execute-api.us-east-1.amazonaws.com/prod/mcp

# Test the endpoint
curl -X POST https://abc123.execute-api.us-east-1.amazonaws.com/prod/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":
```

(table/list, cells in reading order)
- {"name":"test","version":"1.0"}},"id":1}'
- CDK Stack Details
- The generated CDK stack ( .pmcp/cdk/lib/stack.ts ) creates:
- Lambda Function
- const mcpFunction = new lambda.Function(this, 'McpFunction', {

```yaml
  runtime: lambda.Runtime.PROVIDED_AL2023,
  handler: 'bootstrap',
  code: lambda.Code.fromAsset('../target/lambda/release'),
  architecture: lambda.Architecture.ARM_64,
  memorySize: 256,
  timeout: Duration.seconds(30),
  environment: {
    RUST_LOG: 'info',
    AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH: 'true',
  },
  // Lambda Web Adapter layer
  layers: [
    lambda.LayerVersion.fromLayerVersionArn(
      this, 'WebAdapter',
      `arn:aws:lambda:${this.region}:753240598075:layer:LambdaAdapterLayerArm64:22`
    ),
  ],
});
```

```yaml
  apiName: 'my-mcp-server-api',
  corsPreflight: {
    allowOrigins: ['*'],
    allowMethods: [apigatewayv2.CorsHttpMethod.POST],
    allowHeaders: ['Content-Type', 'Authorization'],
  },
});

// Route all /mcp requests to Lambda
api.addRoutes({
  path: '/mcp',
  methods: [apigatewayv2.HttpMethod.POST],
  integration: new HttpLambdaIntegration('McpIntegration', mcpFunction),
});

// SSE endpoint for streaming (if needed)
api.addRoutes({
  path: '/mcp/sse',
  methods: [apigatewayv2.HttpMethod.GET],
  integration: new HttpLambdaIntegration('SseIntegration', mcpFunction),
});
```


### VPC Configuration (Optional)

```yaml
// For private database access
const vpc = ec2.Vpc.fromLookup(this, 'Vpc', {
  vpcId: props.vpcId,
});
```


### mcpFunction.connections.allowTo(

```text
  ec2.Peer.ipv4(vpc.vpcCidrBlock),
  ec2.Port.tcp(5432),
  'PostgreSQL'
);
```

(table/list, cells in reading order)
- API Gateway Configuration
- HTTP API vs REST API
- Feature
- HTTP API
- REST API
- Latency
- Lower (~10ms)
- Higher (~30ms)
- Cost
- $1.00/million
- $3.50/million
- Features
- Basic
- Full (caching, WAF, etc.)
- WebSocket
- No
- Yes

Recommendation: Use HTTP API unless you need REST API-specific features.

```text
// In stack.ts
const certificate = acm.Certificate.fromCertificateArn(
  this, 'Cert',
  'arn:aws:acm:us-east-1:123456789:certificate/abc-123'
);
```


### const domainName = new apigatewayv2.DomainName(this, 'Domain', {

```yaml
  domainName: 'mcp.example.com',
  certificate,
});
```


### api.addStage('prod', {

```yaml
  stageName: 'prod',
  autoDeploy: true,
  domainMapping: { domainName },
});
```

Then add a Route53 record pointing to the API Gateway domain.

```yaml
  corsPreflight: {
    allowOrigins: [
      'https://claude.ai',
      'https://your-app.com',
    ],
    allowMethods: [
      apigatewayv2.CorsHttpMethod.POST,
      apigatewayv2.CorsHttpMethod.OPTIONS,
    ],
    allowHeaders: [
      'Content-Type',
      'Authorization',
      'X-Request-Id',
    ],
    allowCredentials: true,
    maxAge: Duration.hours(1),
  },
});
```

```yaml
  stageName: 'prod',
  autoDeploy: true,
  throttle: {
    rateLimit: 1000,    // requests per second
    burstLimit: 2000,   // burst capacity
  },
});
```

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit
panic = "abort"        # No unwinding
strip = true           # Strip symbols
```


### [profile.release.package."*"]

```toml
opt-level = "z"
```

Typical Rust MCP server binary: 5-15MB (vs 50-100MB for Node.js with dependencies).

```rust
use once_cell::sync::OnceCell;
use sqlx::{Pool, Postgres};

// Global pool - initialized once per Lambda instance
static DB_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

async fn get_pool() -> &'static Pool<Postgres> {
    DB_POOL.get_or_init(|| {
        tokio::runtime::Handle::current().block_on(async {
            let database_url = get_secret("DATABASE_URL").await.unwrap();
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .unwrap()
        })
    })
}

// In your tool handler
async fn query_handler(input: QueryInput) -> Result<Value> {
    let pool = get_pool().await;  // Returns cached pool on warm start
    let rows = sqlx::query("SELECT * FROM users")
        .fetch_all(pool)
        .await?;
    // ...
}
```

```toml
# .pmcp/deploy.toml
[lambda]
provisioned_concurrency = 5  # Keep 5 instances warm

// In stack.ts
const alias = new lambda.Alias(this, 'ProdAlias', {
  aliasName: 'prod',
  version: mcpFunction.currentVersion,
  provisionedConcurrentExecutions: 5,
});
```


### SnapStart (Java-like Fast Starts)

While SnapStart is Java-only, Rust achieves similar performance naturally:

(table/list, cells in reading order)
- Runtime
- Cold Start
- With Optimization
- Rust
- Java
- Python
- Node.js
- 50-100ms
- 30-50ms
- 3-5s
- 200-500ms (SnapStart)
- 500-1500ms
- 300-500ms
- 200-500ms
- 100-200ms

Rust's compiled binaries don't need SnapStart - they're already fast.

```bash
# Stream logs
cargo pmcp deploy logs --tail

# Or use AWS CLI
aws logs tail /aws/lambda/my-mcp-server --follow
```

```rust
use tracing::{info, warn, instrument};

#[instrument(skip(pool))]
async fn query_handler(pool: &Pool<Postgres>, input: QueryInput) -> Result<Value> {
    info!(table = %input.table, "Executing query");

    let start = Instant::now();
    let result = sqlx::query(&input.query)
        .fetch_all(pool)
        .await;

    match &result {
        Ok(rows) => info!(
            rows = rows.len(),
            duration_ms = start.elapsed().as_millis(),
            "Query completed"
        ),
        Err(e) => warn!(error = %e, "Query failed"),
    }

    result
}
```

(table/list, cells in reading order)
- CloudWatch Metrics
- Key metrics to monitor:
- Metric
- Description
- Alert Threshold
- Invocations
- Total requests
- Anomaly detection
- Errors
- Duration
- Failed invocations
- > 1% error rate
- Execution time
- > 80% of timeout
- ConcurrentExecutions
- Active instances
- > 80% of limit
- Throttles
- Rate-limited requests
- > 0
- X-Ray Tracing
- Enable distributed tracing:

```rust
# .pmcp/deploy.toml
[lambda]
tracing = "active"  # Enable X-Ray

// In your code
use aws_xray_sdk::trace;

#[trace]
async fn query_handler(input: QueryInput) -> Result<Value> {
    // Automatically traced
}
```

```bash
# Create a secret
aws secretsmanager create-secret \
  --name my-mcp-server/database \
  --secret-string '{"host":"db.example.com","password":"secret123"}'
```


### Retrieve in your Lambda:

```rust
use aws_sdk_secretsmanager::Client;

async fn get_secret(name: &str) -> Result<String> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let response = client
        .get_secret_value()
        .secret_id(name)
        .send()
        .await?;

    Ok(response.secret_string().unwrap().to_string())
}
```

```text
  this, 'DbSecret', 'my-mcp-server/database'
);
```

Cause: Lambda timeout too short for your operation.


### Solution:

```toml
[lambda]
timeout_seconds = 60  # Increase timeout
```


### Issue: "Unable to connect to database"

Cause: Lambda not in VPC or security group misconfigured.


### Solution:

```toml
[vpc]
enabled = true
security_group_ids = ["sg-xxx"]  # Must allow outbound to DB
```


### Issue: High cold start latency

Cause: Large binary or slow initialization.


### Solution:

1.


### Issue: "AccessDenied" on Secrets Manager

Cause: Lambda IAM role missing permissions.

(table/list, cells in reading order)
- Solution: Ensure CDK grants access:
- secret.grantRead(mcpFunction);
- Cleanup
- Remove all deployed resources:

```bash
# Destroy Lambda, API Gateway, and all resources
cargo pmcp deploy destroy --clean

# This removes:
# - Lambda function
# - API Gateway
# - IAM roles
# - CloudWatch logs
# - (Optional) Cognito user pool
```

Zero server management - AWS handles scaling, patching, availability Pay-per-use - No cost when idle Fast deployment -  cargo pmcp deploy  handles everything Production-ready - VPC, OAuth, monitoring built-in


### Key commands:

```bash
cargo pmcp deploy init --target aws-lambda  # Initialize
cargo pmcp deploy                           # Deploy
cargo pmcp deploy outputs                   # Get API URL
cargo pmcp deploy logs --tail               # View logs
cargo pmcp deploy destroy --clean           # Cleanup
```


### Connecting MCP Clients

After deploying your MCP server to AWS Lambda, you need to connect clients to it.

- Remote MCP servers use HTTP transport instead of stdio:

```text
                  MCP CLIENT CONNECTION FLOW
LOCAL SERVER (stdio)              REMOTE SERVER (HTTP)
  MCP Client                        MCP Client
         stdin/stdout                      HTTPS
  Local                             API Gateway
  Process                           + Lambda
Config:                           Config:
{                                 {
  "command": "my-server"            "url": "https://...",
}                                   "transport": "streamable-http"
                                  }
```

```bash
cargo pmcp deploy outputs

# Output:
```

```text
#
#                       Deployment Outputs
#
#   ApiEndpoint:  https://abc123.execute-api.us-east-1.amazonaws.com
#   McpEndpoint:  https://abc123.execute-api.us-east-1.amazonaws.com/mcp
#   OAuthUrl:     https://auth.abc123.amazoncognito.com
#   ClientId:     1234567890abcdef
#
```

- For internal servers without OAuth (not recommended for production):

```json
{
  "mcpServers": {
    "my-remote-server": {
      "transport": "streamable-http",
      "url": "https://abc123.execute-api.us-east-1.amazonaws.com/mcp"
    }
  }
}
```

```json
{
  "mcpServers": {
    "my-remote-server": {
      "transport": "streamable-http",
      "url": "https://abc123.execute-api.us-east-1.amazonaws.com/mcp",
      "oauth": {
        "client_id": "1234567890abcdef",
        "authorization_url": "https://auth.abc123.amazoncognito.com/oauth2/authorize",
        "token_url": "https://auth.abc123.amazoncognito.com/oauth2/token",
        "scopes": ["openid", "mcp:read", "mcp:write"]
      }
    }
  }
}
```


### When you start Claude Desktop:

1.

```json
{
  "mcpServers": {
    "my-remote-server": {
      "transport": "streamable-http",
      "url": "https://abc123.execute-api.us-east-1.amazonaws.com/mcp",
      "headers": {
        "Authorization": "Bearer your-api-key-here"
      }
    }
  }
}
```

```json
{
  "mcpServers": {
    "my-remote-server": {
      "transport": "streamable-http",
      "url": "https://abc123.execute-api.us-east-1.amazonaws.com/mcp",
      "headers": {
        "Authorization": "Bearer ${MCP_API_KEY}"
      }
    }
  }
}
```


### Then set the environment variable before starting Claude Desktop:

```bash
export MCP_API_KEY="your-api-key-here"
```

Claude.ai supports connecting to remote MCP servers through the Integrations settings.


### OAuth Settings:

```yaml
  Client ID:         1234567890abcdef
  Authorization URL: https://auth.abc123.amazoncognito.com/oauth2/authorize
  Token URL:         https://auth.abc123.amazoncognito.com/oauth2/token
  Scopes:            openid mcp:read mcp:write
```

1.

```text
                       OAUTH 2.0 FLOW
1. USER INITIATES CONNECTION
   Claude Desktop/Claude.ai detects OAuth config
2. AUTHORIZATION REQUEST
   Browser opens:
   https://auth.abc123.amazoncognito.com/oauth2/authorize
     ?client_id=1234567890abcdef
     &response_type=code
     &redirect_uri=http://localhost:8765/callback
     &scope=openid%20mcp:read%20mcp:write
     &state=random_state_value
3. USER AUTHENTICATES
   - Username/password
   - Or federated SSO (Google, SAML, etc.)
4. AUTHORIZATION CODE RETURNED
   Browser redirects to:
   http://localhost:8765/callback?code=AUTH_CODE&state=random_state
5. TOKEN EXCHANGE
   Client POSTs to token endpoint:
   POST https://auth.abc123.amazoncognito.com/oauth2/token
     grant_type=authorization_code
     &code=AUTH_CODE
     &client_id=1234567890abcdef
     &redirect_uri=http://localhost:8765/callback
   Response:
   {
     "access_token": "eyJhbGciOi...",
     "refresh_token": "eyJjdHki...",
     "expires_in": 3600
   }
```

```text
6. MCP REQUESTS WITH TOKEN
   POST https://abc123.execute-api.us-east-1.amazonaws.com/mcp
   Authorization: Bearer eyJhbGciOi...
```


### Token Refresh

Access tokens expire (typically after 1 hour).

```text
                      TOKEN REFRESH FLOW
1. Access token expires (401 Unauthorized)
2. Client uses refresh token:
   POST https://auth.abc123.amazoncognito.com/oauth2/token
     grant_type=refresh_token
     &refresh_token=eyJjdHki...
     &client_id=1234567890abcdef
3. New tokens returned
4. Retry original request with new access token
```

```bash
# Create a user
aws cognito-idp admin-create-user \
  --user-pool-id us-east-1_ABC123 \
  --username alice@company.com \
  --user-attributes Name=email,Value=alice@company.com \
  --temporary-password "TempPass123!"

# Set permanent password (skip temporary)
aws cognito-idp admin-set-user-password \
  --user-pool-id us-east-1_ABC123 \
  --username alice@company.com \
  --password "SecurePass456!" \
  --permanent
```

```bash
# Create resource server with scopes
aws cognito-idp create-resource-server \
  --user-pool-id us-east-1_ABC123 \
  --identifier "mcp" \
  --name "MCP API" \
  --scopes ScopeName=read,ScopeDescription="Read access" \
          ScopeName=write,ScopeDescription="Write access" \
          ScopeName=admin,ScopeDescription="Admin access"
```


### Update your app client to include scopes:

```bash
aws cognito-idp update-user-pool-client \
  --user-pool-id us-east-1_ABC123 \
  --client-id 1234567890abcdef \
  --allowed-oauth-scopes openid mcp/read mcp/write
```

```yaml
// In CDK stack
const userPool = new cognito.UserPool(this, 'McpUserPool', {
  // ...
});

// Add Google SSO
const googleProvider = new cognito.UserPoolIdentityProviderGoogle(
  this, 'Google',
  {
    userPool,
    clientId: 'google-client-id',
    clientSecretValue: SecretValue.secretsManager('google-client-secret'),
    scopes: ['email', 'profile'],
    attributeMapping: {
      email: cognito.ProviderAttribute.GOOGLE_EMAIL,
      fullname: cognito.ProviderAttribute.GOOGLE_NAME,
    },
  }
);

// Add SAML provider for enterprise SSO
const samlProvider = new cognito.UserPoolIdentityProviderSaml(
  this, 'Okta',
  {
    userPool,
    metadata: cognito.UserPoolIdentityProviderSamlMetadata.url(
      'https://company.okta.com/app/metadata'
    ),
    attributeMapping: {
      email: cognito.ProviderAttribute.other('email'),
    },
  }
);
```

```rust
use pmcp::client::{Client, HttpTransport};
use pmcp::types::CallToolParams;

#[tokio::main]
async fn main() -> Result<()> {
    // Create HTTP transport with OAuth token
    let transport = HttpTransport::new("https://abc123.execute-api.us-east-1.amazonaws.com/mcp")
        .with_bearer_token("eyJhbGciOi...")
        .build()?;

    // Connect to server
    let client = Client::connect(transport).await?;

    // Initialize
    let server_info = client.initialize().await?;
    println!("Connected to: {}", server_info.name);

    // List available tools
    let tools = client.list_tools().await?;
    for tool in &tools {
        println!("Tool: {} - {}", tool.name, tool.description.as_deref().unwrap_or(""));
    }

    // Call a tool
    let result = client.call_tool(CallToolParams {
        name: "query_users".to_string(),
        arguments: serde_json::json!({
            "department": "Engineering"
        }),
    }).await?;

    println!("Result: {}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
```


### TypeScript/JavaScript Client

```text
import { Client, HttpTransport } from '@anthropic/mcp-sdk';
```


### async function main() {

```rust
  // Create transport with authentication
  const transport = new HttpTransport({
    url: 'https://abc123.execute-api.us-east-1.amazonaws.com/mcp',
    headers: {
      'Authorization': `Bearer ${process.env.MCP_TOKEN}`,
    },
  });

  // Connect
  const client = new Client({ transport });
  await client.connect();

  // Initialize
  const serverInfo = await client.initialize({
    protocolVersion: '2025-11-25',
    capabilities: {},
    clientInfo: { name: 'my-app', version: '1.0.0' },
  });

  console.log(`Connected to: ${serverInfo.serverInfo.name}`);

  // List tools
  const tools = await client.listTools();
  console.log('Available tools:', tools.tools.map(t => t.name));

  // Call a tool
  const result = await client.callTool({
    name: 'query_users',
    arguments: { department: 'Engineering' },
  });

  console.log('Result:', result);
}

main().catch(console.error);
```


### Python Client

```text
import asyncio
from mcp import Client, HttpTransport
```


### async def main():

```text
    # Create transport with authentication
    transport = HttpTransport(
        url="https://abc123.execute-api.us-east-1.amazonaws.com/mcp",
        headers={"Authorization": f"Bearer {os.environ['MCP_TOKEN']}"}
    )

    # Connect
    async with Client(transport) as client:
        # Initialize
        server_info = await client.initialize()
        print(f"Connected to: {server_info.name}")

        # List tools
        tools = await client.list_tools()
        print(f"Available tools: {[t.name for t in tools]}")

        # Call a tool
        result = await client.call_tool(
            name="query_users",
            arguments={"department": "Engineering"}
        )
        print(f"Result: {result}")
```

(table/list, cells in reading order)
- asyncio.run(main())
- Testing the Connection
- Using curl
- Test your endpoint directly:

```bash
# Initialize
curl -X POST https://abc123.execute-api.us-east-1.amazonaws.com/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-11-25",
      "capabilities": {},
      "clientInfo": {"name": "curl", "version": "1.0"}
    },
    "id": 1
  }'

# List tools
curl -X POST https://abc123.execute-api.us-east-1.amazonaws.com/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 2
  }'

# Call a tool
curl -X POST https://abc123.execute-api.us-east-1.amazonaws.com/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "query_users",
      "arguments": {"department": "Engineering"}
    },

    "id": 3
  }'
```

```bash
# Run integration tests against deployed server
cargo pmcp deploy test

# Output:
# Testing connection to https://abc123.execute-api.us-east-1.amazonaws.com/mcp
# [ok] Initialize: 45ms
# [ok] List Tools: 23ms (found 5 tools)
# [ok] Call 'query_users': 156ms
# [ok] Call 'get_user_details': 89ms
```


### #

```text
# All tests passed!
```

Cause: Invalid or expired token.


### Solution:

1.


### "403 Forbidden"

Cause: Token valid but missing required scopes.


### Solution:

1. Check Cognito app client has required scopes 2.


### "CORS Error" (Browser)

Cause: API Gateway CORS not configured for your origin.

```yaml
  allowOrigins: ['https://your-app.com'],
  // ...
}
```


### "Connection Timeout"

Cause: Lambda in VPC without NAT gateway, or cold start too slow.


### Solution:

1.


### "Invalid Redirect URI"

Cause: Callback URL doesn't match Cognito configuration.


### Solution: Add the redirect URI to Cognito app client:

```bash
aws cognito-idp update-user-pool-client \
  --user-pool-id us-east-1_ABC123 \
  --client-id 1234567890abcdef \
  --callback-urls "http://localhost:8765/callback" "https://claude.ai/callback"
```

1. Configure authentication: OAuth (recommended) or API keys 3.


### Key configuration patterns:

```text
// Claude Desktop with OAuth
{
  "mcpServers": {
    "my-server": {
      "transport": "streamable-http",
      "url": "https://abc123.execute-api.us-east-1.amazonaws.com/mcp",
      "oauth": {
        "client_id": "...",
        "authorization_url": "...",
        "token_url": "...",
        "scopes": ["openid", "mcp:read"]
      }
    }
  }
}
```

Your MCP server is now accessible to anyone with proper credentials, from anywhere in the world.
