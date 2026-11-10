---
chapter: 16
title: "Azure Container Apps"
part: "Part III: Deployment"
use_when: "Deploying to Azure Container Apps: [azure] config, ingress requirements, lifecycle, securing."
suggested_prerequisites: [ch07]
source_kb: 12
extracted_kb: 7
code_blocks: 13
quality: auto-extracted
---

# Ch 16: Azure Container Apps

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Azure Container Apps Deployment

Azure Container Apps (ACA) runs your MCP server as a managed container behind automatic HTTPS ingress, with scale-to-zero, revision- based rollouts, and a Container Apps environment that gives you Log Analytics and Dapr if you want them.


### The cloud-build advantage

```text
            Container Deploy Path Comparison
Google Cloud Run                 Azure Container Apps
docker buildx (LOCAL)            az containerapp up --source
→ push to gcr.io                 → ACR cloud-build (NO local Docker)
gcloud run deploy                managed environment + ingress
needs Docker installed           needs only the `az` CLI
Best for:                        Best for:
GCP ecosystem                    Azure ecosystem
local image control              zero-Docker dev machines / CI
```

Because the build happens in ACR,  cargo pmcp 's Azure target only requires the  az  CLI to be installed and authenticated — its is_available()  check does not look for Docker.

(table/list, cells in reading order)
- When to choose Azure Container Apps
- Requirement
- Why Azure Container Apps
- No local Docker
- ACR cloud-builds from  --source
- Azure ecosystem
- Native to the subscription, Log Analytics, managed identity
- Requirement
- Why Azure Container Apps
- Container flexibility
- Full Linux image, any dependency
- Auto HTTPS ingress
- TLS-terminated FQDN issued on deploy
- Scale to zero
- min-replicas 0  (we set  1  by default to avoid cold starts)
- Revision rollback
- ingress traffic set --revision-weight
- The CLI Workflow (Start Here)

This is the framing you should teach and reach for first.

```bash
# 1. Scaffold a server — the template ships the two ingress-safe defaults (see below)
cargo pmcp new my-mcp-server
```


### cd my-mcp-server

```bash
cargo pmcp add server calculator --template complete

# 2. Generate the Azure deploy artifacts
cargo pmcp deploy init --target azure-container-apps
#   writes: Dockerfile, .dockerignore, .pmcp/deploy.toml (type = "azure-container-apps")
#   skips:  AWS credentials + CDK entirely

# 3. Authenticate — management-plane calls need a fresh interactive token
az login            # pick the target subscription if prompted

# 4. Deploy — ACR cloud-builds the Dockerfile and provisions the Container App
cargo pmcp deploy --target-type azure-container-apps
```

After a successful deploy your MCP endpoint is served at the root of the issued FQDN:

```text
# Builder
```

```text
# Runtime
```


### Configuration: the [azure] section

init  leaves  [azure]  at defaults.

```toml
[target]
type = "azure-container-apps"

[server]
name = "calculator-server"

[azure]
resource_group = "calculator-server-rg"   # default: <server-name>-rg
location        = "eastus"                 # default: eastus
environment     = "calculator-server-env"  # default: <server-name>-env
target_port     = 8080                     # default: 8080
min_replicas    = 1                        # default: 1
```

(table/list, cells in reading order)
- Precedence: ENV > section > defaults
- Every setting resolves in this order:
- Setting
- Env override
- [azure]  key
- Default
- Resource group
- AZURE_RESOURCE_GROUP
- resource_group
- <server-name>-rg
- Environment
- Location
- AZURE_CONTAINERAPP_ENV
- environment
- <server-name>-env
- AZURE_LOCATION
- location
- eastus
- Target port
- AZURE_TARGET_PORT  (u16)
- target_port
- Min replicas
- AZURE_MIN_REPLICAS  (u32)
- min_replicas
- 8080
- 1

The numeric env overrides are validated — an unparseable  AZURE_TARGET_PORT  is a clear named error, never a raw string handed to

```bash
az . An absent/empty  [azure]  section serialises byte-identically to a pre-Azure  deploy.toml , so adding the target never disturbs an
```


### The Two Ingress Requirements (Don't Skip This)

Spike 007 deployed a hand-built server and got  403  on every call. Two settings are mandatory for any container MCP server behind ingress.  cargo pmcp new  now bakes both into the generated template (a global default across all targets), but if you hand-roll your server you must set them yourself:

```rust
use pmcp::{AllowedOrigins, StreamableHttpServerConfig};

let config = StreamableHttpServerConfig {
    allowed_origins: Some(AllowedOrigins::any()), // proxy-safe default
    // ...
};
```

Bind  0.0.0.0:$PORT  ( Ipv4Addr::UNSPECIFIED ), honoring  $PORT  (default 8080) — binding  127.0.0.1  means ingress cannot reach

```bash
cargo pmcp deploy init  prints a warning surfacing both requirements, so you find out before you spend the cloud-build time, not after
```

(table/list, cells in reading order)
- Comparison with Google Cloud Run
- Aspect
- Google Cloud Run
- Azure Container Apps
- Build
- Local Docker required
- CORS
- local  docker buildx  → push

```bash
az containerapp up --source  (ACR cloud-build)
```

(table/list, cells in reading order)
- Yes
- No
- application-level
- ingress-level ( az containerapp ingress cors enable )
- Aspect
- Google Cloud Run
- Azure Container Apps
- Provider setup
- enable APIs ( gcloud services enable )
- Poll reliability
- stable
- register providers with  --wait  (cold-sub one-time)

```bash
az  long-polls drop ( RemoteDisconnected ) — re-check
```

(table/list, cells in reading order)
- state
- Endpoint
- *.run.app/mcp
- *.azurecontainerapps.io/  (MCP at root)


### Lifecycle Commands

All share the  cargo pmcp deploy --target-type azure-container-apps  surface:

```bash
# Stream logs
cargo pmcp deploy --target-type azure-container-apps logs --follow --tail 100

# Show outputs (probes the FQDN)
cargo pmcp deploy --target-type azure-container-apps outputs

# Roll back to a prior revision (ingress traffic set --revision-weight <rev>=100)
cargo pmcp deploy --target-type azure-container-apps rollback --version <revision>

# Tear down (group delete --no-wait; clean also removes Dockerfile/.dockerignore)
cargo pmcp deploy --target-type azure-container-apps destroy
```


### Securing the Deployment

A freshly deployed server is reachable — and broadly open.

A server that 403s behind every reverse proxy is the worse failure mode, and at deploy time you do not yet know your FQDN.


### 1. Restrict origins to the FQDN

Replace  any()  with the actual deployment host, and tighten ingress CORS to match:

```rust
let config = StreamableHttpServerConfig {
    allowed_origins: Some(AllowedOrigins::new([
        "https://my-mcp-server.delightfulpebble.eastus.azurecontainerapps.io",
    ])),
    // ...
};

az containerapp ingress cors enable \
  --name my-mcp-server --resource-group my-mcp-server-rg \
  --allowed-origins 'https://my-mcp-server.delightfulpebble.eastus.azurecontainerapps.io' \
  --allowed-methods 'GET,POST,DELETE,OPTIONS' --allowed-headers '*'
```


### 2. Add authentication

Public ingress with broad CORS and no auth means anyone who learns the URL can call your tools.


### Summary

Azure Container Apps gives MCP servers a managed, auto-HTTPS container runtime with a zero-local-Docker deploy path:

```bash
cargo pmcp deploy init --target azure-container-apps  →  az login  →  cargo pmcp deploy --target-type azure-container-
```

The scaffold ships the two ingress requirements ( allowed_origins::any()  +  0.0.0.0 ); hand-rolled servers must set them or every request 403s.

Why does the Azure target not require local Docker, and what builds the image instead? What two settings must every container MCP server have to survive ingress, and what symptom appears if  allowed_origins  is wrong?
