# Part III: Deployment

Load one chapter file, not the whole part.

| Ch | Title | Use when | Needs | KB |
|---|---|---|---|---|
| 7 | [Deployment Overview](ch07-deployment-overview.md) | Choosing a deployment target (Lambda, Cloudflare Workers, Cloud Run), cost model, security boundaries. | ch02 | 40 |
| 8 | [AWS Lambda Deployment](ch08-aws-lambda.md) | Deploying to AWS Lambda: CDK, Lambda Web Adapter, VPC, secrets, cold starts, troubleshooting. | ch07 | 24 |
| 9 | [Config-Driven SQL Servers](ch09-config-sql.md) | Shipping a SQL MCP server from config only (no Rust), deploying to Lambda. | ch03, ch08 | 6 |
| 10 | [Config-Driven OpenAPI Servers](ch10-config-openapi.md) | Wrapping a REST API via OpenAPI spec; backend auth (oauth2_client_credentials, oauth_passthrough); inline resources/prompts. | ch09 | 8 |
| 11 | [OpenAPI over Microsoft Graph](ch11-openapi-graph.md) | Worked example: OpenAPI server over Microsoft Graph / M365. | ch10 | 5 |
| 12 | [Config-Driven Workbook Servers](ch12-config-workbook.md) | Serving a governed Excel workbook as an MCP server: compile, serve, govern, deploy. | ch08 | 7 |
| 13 | [Authoring Workbooks as MCP Tools](ch13-workbook-authoring.md) | Authoring workbook tables as MCP tools: the table contract, tiers, governance. | ch12 | 4 |
| 14 | [Cloudflare Workers (WASM)](ch14-cloudflare.md) | Deploying to Cloudflare Workers: wrangler, bindings, WASM limits, what does not work. | ch07 | 35 |
| 15 | [Google Cloud Run](ch15-cloud-run.md) | Deploying to Google Cloud Run: containers, config, secrets, scaling. | ch07 | 59 |
| 16 | [Azure Container Apps](ch16-azure.md) | Deploying to Azure Container Apps: [azure] config, ingress requirements, lifecycle, securing. | ch07 | 8 |
