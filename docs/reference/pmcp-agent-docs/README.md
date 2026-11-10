# PMCP Rust MCP: agent docs

Audience: coding agents (Claude Code, OpenCode, pi, etc.).
Topic: building production MCP (Model Context Protocol) servers in Rust with the PMCP SDK (`pmcp` crate, `cargo-pmcp` CLI).
Source: [Advanced MCP book](https://paiml.github.io/rust-mcp-sdk/course/print.html) print export. Repo: [rust-mcp-sdk](https://github.com/paiml/rust-mcp-sdk).

## How to use
1. Find the task in the routing table.
2. Open that chapter file only. Each has frontmatter: `use_when`, `suggested_prerequisites`.
3. Open the part `INDEX.md` if the task spans chapters.
4. Check code against the repo examples before copying. Content is auto-extracted from a PDF export.

## Routing table

| Task | File |
|---|---|
| Setting up toolchain: Rust, cargo-pmcp, an MCP client. | [part1-foundations/ch00-prerequisites.md](part1-foundations/ch00-prerequisites.md) |
| Deciding if MCP fits vs RAG, fine-tuning, hand-written agents; justifying Rust. | [part1-foundations/ch01-enterprise-case.md](part1-foundations/ch01-enterprise-case.md) |
| Scaffolding a first server, running it locally, connecting Claude Code, testing with MCP Inspector. | [part1-foundations/ch02-first-server.md](part1-foundations/ch02-first-server.md) |
| Building a database MCP server: list_tables/execute_query tools, SQL injection safety, resources, pagination. | [part1-foundations/ch03-database-servers.md](part1-foundations/ch03-database-servers.md) |
| Designing a tool set: avoiding sprawl and name collisions, cohesive single-purpose tools. | [part2-design/ch04-tool-sprawl.md](part2-design/ch04-tool-sprawl.md) |
| Writing schemas, enums, validation, output schemas, tool annotations, AI-friendly error messages. | [part2-design/ch05-validation.md](part2-design/ch05-validation.md) |
| Choosing resources vs tools vs prompts; soft/hybrid/hard workflows; resource templates and caching. | [part2-design/ch06-resources-prompts.md](part2-design/ch06-resources-prompts.md) |
| Choosing a deployment target (Lambda, Cloudflare Workers, Cloud Run), cost model, security boundaries. | [part3-deployment/ch07-deployment-overview.md](part3-deployment/ch07-deployment-overview.md) |
| Deploying to AWS Lambda: CDK, Lambda Web Adapter, VPC, secrets, cold starts, troubleshooting. | [part3-deployment/ch08-aws-lambda.md](part3-deployment/ch08-aws-lambda.md) |
| Shipping a SQL MCP server from config only (no Rust), deploying to Lambda. | [part3-deployment/ch09-config-sql.md](part3-deployment/ch09-config-sql.md) |
| Wrapping a REST API via OpenAPI spec; backend auth (oauth2_client_credentials, oauth_passthrough); inline resources/prompts. | [part3-deployment/ch10-config-openapi.md](part3-deployment/ch10-config-openapi.md) |
| Worked example: OpenAPI server over Microsoft Graph / M365. | [part3-deployment/ch11-openapi-graph.md](part3-deployment/ch11-openapi-graph.md) |
| Serving a governed Excel workbook as an MCP server: compile, serve, govern, deploy. | [part3-deployment/ch12-config-workbook.md](part3-deployment/ch12-config-workbook.md) |
| Authoring workbook tables as MCP tools: the table contract, tiers, governance. | [part3-deployment/ch13-workbook-authoring.md](part3-deployment/ch13-workbook-authoring.md) |
| Deploying to Cloudflare Workers: wrangler, bindings, WASM limits, what does not work. | [part3-deployment/ch14-cloudflare.md](part3-deployment/ch14-cloudflare.md) |
| Deploying to Google Cloud Run: containers, config, secrets, scaling. | [part3-deployment/ch15-cloud-run.md](part3-deployment/ch15-cloud-run.md) |
| Deploying to Azure Container Apps: [azure] config, ingress requirements, lifecycle, securing. | [part3-deployment/ch16-azure.md](part3-deployment/ch16-azure.md) |
| Unit/property tests, MCP Inspector, mcp-tester, schema-driven test generation. | [part4-testing/ch17-local-testing.md](part4-testing/ch17-local-testing.md) |
| Testing deployed servers, load tests, CI/CD with GitHub Actions, regression tests. | [part4-testing/ch18-remote-testing.md](part4-testing/ch18-remote-testing.md) |
| Adding OAuth 2.0: middleware, JWT validation, scopes, multi-tenant, auth errors, auth tests. | [part5-security/ch19-oauth.md](part5-security/ch19-oauth.md) |
| Configuring Cognito, Auth0 or Entra ID; comparing providers. | [part5-security/ch20-idp.md](part5-security/ch20-idp.md) |
| Setting up an AI coding assistant for MCP work; compiler feedback loop; clippy gates. | [part6-ai-dev/ch21-ai-dev.md](part6-ai-dev/ch21-ai-dev.md) |
| Workflow and prompting patterns for AI-built MCP tools; QA with scenarios; alternative assistants. | [part6-ai-dev/ch22-ai-collab.md](part6-ai-dev/ch22-ai-collab.md) |
| Adding logging, metrics, tracing, health checks, resilience via built-in observability or custom middleware. | [part7-observability/ch23-middleware.md](part7-observability/ch23-middleware.md) |
| Load testing config and scenarios; production monitoring. | [part7-observability/ch24-ops.md](part7-observability/ch24-ops.md) |
| Organising 5-10+ servers: foundation/domain/orchestration tiers; server-side workflows. | [part8-advanced/ch25-composition.md](part8-advanced/ch25-composition.md) |
| Shipping interactive widget UIs from tools (UI resources, MIME type, CSP, bridge). | [part8-advanced/ch26-mcp-apps.md](part8-advanced/ch26-mcp-apps.md) |
| Operations too slow for one request: TaskStore, TaskSupport, polling, get_task_result fallback. | [part8-advanced/ch27-tasks.md](part8-advanced/ch27-tasks.md) |
| Letting LLMs submit validated code (GraphQL/OpenAPI/SQL) via validate_code/execute_code with policy checks. | [part8-advanced/ch28-code-mode.md](part8-advanced/ch28-code-mode.md) |
| Publishing agent workflow instructions from the server (SEP-2640), dual skill/prompt surface. | [part8-advanced/ch29-skills.md](part8-advanced/ch29-skills.md) |
| Running agent loops and teams against MCP servers; deploy-anywhere agents. | [part8-advanced/ch30-agents.md](part8-advanced/ch30-agents.md) |

## Verified quickstart (from ch02)
```bash
cargo install cargo-pmcp
cargo pmcp new my-mcp-servers && cd my-mcp-servers
cargo pmcp add server calculator --template calculator
cargo pmcp dev calculator   # HTTP on port 3000
claude mcp add calculator -t http http://0.0.0.0:3000
npx @modelcontextprotocol/inspector http://localhost:3000/mcp
```

## Caveats
1. Auto-extracted. Code can be truncated (blocks over 60 lines are cut) or missing where the PDF dropped it. Tables are flattened.
2. Exercise sections are excluded.
3. Hand-checked "Agent notes" exist only for ch02, ch25, ch27, ch29. Other chapters are not hand-verified.
4. Crate versions seen on docs.rs: 2.19.2 to 2.22.5. Version pins in the book are unverified. Check [crates.io](https://crates.io/crates/pmcp) for the current version.
5. Performance claims (microsecond latency, "16x faster") come from marketing text. Not benchmarked here.
6. Appendices (cargo pmcp reference, templates, troubleshooting, security checklist) are not included.
