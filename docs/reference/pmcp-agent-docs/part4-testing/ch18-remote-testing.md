---
chapter: 18
title: "Remote Testing"
part: "Part IV: Testing"
use_when: "Testing deployed servers, load tests, CI/CD with GitHub Actions, regression tests."
suggested_prerequisites: [ch17]
source_kb: 72
extracted_kb: 42
code_blocks: 87
quality: auto-extracted
---

# Ch 18: Remote Testing

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Remote Testing

Testing MCP servers in production environments requires different strategies than local development.

Local testing catches most bugs, but production environments introduce variables you can't simulate:

```text
               Local vs Production Differences
LOCAL DEVELOPMENT                  PRODUCTION
  • Localhost                        • Load balancers
  • No latency                       • Network latency
  • Fast database                    • Database pools
  • Full resources                   • Resource limits
  • No TLS                           • TLS termination
  • Single instance                  • Multiple replicas
  • Test data                        • Real data
  • No auth                          • Auth required
Production-only issues:
• Cold starts under real traffic
• Connection pool exhaustion
• SSL/TLS certificate problems
• DNS resolution failures
• Cross-region latency
• Concurrent request handling
```


### Remote Testing Architecture

```text
                 Remote Testing Pipeline
    CI/CD          ▶    Deploy         ▶    Test
   Trigger              Server              Remote
   Test Environments
        Staging              Preview            Production
      (Pre-prod)            (Per-PR)            (Canary)
     Full test suite      Smoke tests          Health checks
     Integration          Critical paths       Monitoring
     Performance
Results:
  • Pass: Promote to next environment
  • Fail: Rollback, alert team, block deployment
```

```bash
# Test a deployed server
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --scenario tests/scenarios/

# With authentication
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --header "Authorization: Bearer ${MCP_API_KEY}" \
  --scenario tests/scenarios/

# With timeout for cold starts
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --timeout 30000 \
  --scenario tests/scenarios/smoke/
```


### Environment-Specific Configuration

```bash
# tests/config/staging.yaml
server:
  url: https://staging.mcp.example.com/mcp
  headers:
    Authorization: "Bearer ${STAGING_API_KEY}"
  timeout_ms: 30000
  retry_count: 3

scenarios:
  - tests/scenarios/smoke/
  - tests/scenarios/integration/

options:
  parallel: 4
  fail_fast: false
  junit_output: test-results/staging.xml

# tests/config/production.yaml
server:
  url: https://mcp.example.com/mcp
  headers:
    Authorization: "Bearer ${PROD_API_KEY}"
  timeout_ms: 10000
  retry_count: 1

scenarios:
  - tests/scenarios/smoke/

options:
  parallel: 2
  fail_fast: true
  junit_output: test-results/production.xml

# Run with environment config
cargo pmcp test run --config tests/config/staging.yaml
cargo pmcp test run --config tests/config/production.yaml
```

```text
# tests/scenarios/smoke/health_check.yaml
```

name: "Smoke - Basic health check" description: "Verify server responds to basic requests"

```yaml
tags:
  - smoke
  - critical

steps:
  - name: "Server responds"
    tool: list_tables
    input: {}
    expect:
      success: true
      response_time_ms:
        less_than: 5000  # Cold start allowance

  - name: "Execute simple query"
    tool: execute_query
    input:
      sql: "SELECT 1 as health_check"
    expect:
      success: true
      content:
        contains: "health_check"

# tests/scenarios/smoke/critical_paths.yaml
```

name: "Smoke - Critical user paths" description: "Test the most important user workflows"

```yaml
tags:
  - smoke
  - critical

steps:
  - name: "List available tables"
    tool: list_tables
    input: {}
    expect:
      success: true

  - name: "Query user data"
    tool: execute_query
    input:
      sql: "SELECT id, name FROM users LIMIT 1"
    expect:
      success: true
      content:
        type: text

  - name: "Sample rows work"
    tool: get_sample_rows
    input:
      table: "users"
      limit: 1
    expect:
      success: true
```

```text
# .github/workflows/mcp-testing.yml
```


### name: MCP Server Testing

```bash
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Run unit tests
        run: cargo test --all-features

      - name: Upload coverage
        uses: codecov/codecov-action@v3

  integration-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      postgres:

        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: mcp_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Build server
        run: cargo build --release

      - name: Start MCP server
        run: |
          cargo run --release &
          sleep 5  # Wait for server to start
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/mcp_test

      - name: Run mcp-tester
        run: |
// ... truncated, 77 more lines in source book ...
```


### GitLab CI Pipeline

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - deploy-staging
  - test-staging
  - deploy-production
  - test-production

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUSTUP_HOME: $CI_PROJECT_DIR/.rustup

cache:
  paths:
    - .cargo/
    - target/

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/mcp-server

unit-tests:
  stage: test
  image: rust:latest
  script:
    - cargo test --all-features
  coverage: '/^\d+.\d+% coverage/'

integration-tests:
  stage: test

  image: rust:latest
  services:
    - postgres:15
  variables:
    DATABASE_URL: postgres://postgres:password@postgres:5432/test
    POSTGRES_PASSWORD: password
    POSTGRES_DB: test
  script:
    - cargo run --release &
    - sleep 5
    - cargo pmcp test run --server http://localhost:3000/mcp --format junit --output integration-results.xml
  artifacts:
    reports:
      junit: integration-results.xml

deploy-staging:
  stage: deploy-staging
  environment:
    name: staging
    url: https://staging.mcp.example.com
  script:
    - ./deploy.sh staging
  only:
// ... truncated, 42 more lines in source book ...
```


### Makefile Integration

```text
# Makefile for MCP server testing
```


### .PHONY: test test-unit test-integration test-staging test-prod

```text
# Local testing
```


### test: test-unit test-integration

```bash
test-unit:
    cargo test --all-features

test-integration:
    @echo "Starting server..."
    cargo run --release &
    sleep 5
    cargo pmcp test run --server http://localhost:3000/mcp --scenario tests/scenarios/
    @pkill -f "target/release/mcp-server" || true

# Remote testing
test-staging:
    cargo pmcp test run \
        --server https://staging.mcp.example.com/mcp \
        --header "Authorization: Bearer $(STAGING_API_KEY)" \
        --scenario tests/scenarios/ \
        --format junit \
        --output test-results/staging.xml

test-prod-smoke:
    cargo pmcp test run \
        --server https://mcp.example.com/mcp \
        --header "Authorization: Bearer $(PROD_API_KEY)" \
        --scenario tests/scenarios/smoke/ \
        --fail-fast

# Generate tests from schema
generate-tests:
    cargo run --release &

    sleep 5
    cargo pmcp test generate --server http://localhost:3000/mcp --output tests/scenarios/generated/
    @pkill -f "target/release/mcp-server" || true

# Quality gate (run before commit)
quality-gate: test-unit
    cargo fmt --check
    cargo clippy -- -D warnings
```

```text
# tests/scenarios/regression/issue-123-null-handling.yaml
```


### name: "Regression #123 - Null value handling" description: |

```yaml
  Fixed in v1.2.3: Server crashed when query returned NULL values.
  This test ensures the fix remains in place.
tags:
  - regression
  - critical
  - issue-123

steps:
  - name: "Query with NULL values doesn't crash"
    tool: execute_query
    input:
      sql: "SELECT NULL as null_col, 1 as int_col"
    expect:
      success: true
      content:
        type: text

# tests/scenarios/regression/issue-456-unicode.yaml
```


### name: "Regression #456 - Unicode in table names" description: |

```yaml
  Fixed in v1.3.0: Unicode characters in table names caused errors.
tags:
  - regression
  - unicode
  - issue-456

steps:
  - name: "Query table with unicode name"
    tool: execute_query
    input:
      sql: "SELECT * FROM \"datos_españoles\" LIMIT 1"
    expect:
      success: true
```


### Automated Regression Detection

```text
# .github/workflows/regression-check.yml
```


### name: Regression Check

```bash
on:
  pull_request:
    branches: [main]

jobs:
  regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need history for comparison

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Build current version
        run: cargo build --release

      - name: Start server
        run: |
          cargo run --release &
          sleep 5

      - name: Run regression suite
        run: |
          cargo pmcp test run \
            --server http://localhost:3000/mcp \
            --scenario tests/scenarios/regression/ \
            --fail-fast \
            --format junit \
            --output regression-results.xml

      - name: Compare with baseline

        run: |
          # Download baseline results from previous release
          gh release download --pattern 'baseline-results.json' --dir /tmp || true

          # Compare response times
          cargo pmcp test compare \
            --current regression-results.xml \
            --baseline /tmp/baseline-results.json \
            --threshold 20%  # Fail if >20% slower
```


### Load Testing Your Deployed Servers

Beyond functional correctness, you'll also want to know how your deployed server performs under realistic load.

```bash
cargo pmcp loadtest run --server https://mcp.example.com/mcp --vus 10 --duration 30s
```


### With load testing, you can discover:

Concurrent client capacity - How many simultaneous clients your server handles before response times degrade Tail latency - What P99 latency looks like at different load levels (the metric that matters most for user experience) Breaking points - Where your server starts dropping requests or returnin


### Here's a quick taste of what a load test run looks like:

```bash
cargo pmcp loadtest run \
  --server https://staging.mcp.example.com/mcp \
  --vus 10 \
  --duration 30s

# Output (k6-style):
#   scenarios: 1, max VUs: 10, duration: 30s
#   requests .......... 1,247  (41.6/s)
#   latency p50 ...... 12ms
#   latency p95 ...... 34ms
#   latency p99 ...... 87ms
#   errors ........... 0     (0.00%)
```

Hands-on tutorial: This section gives you a taste of what's possible.


### Chapter Summary

Remote testing validates that your MCP server works in production conditions.


### Practice Ideas

These informal exercises help reinforce the concepts.

```bash
# Simple remote test
cargo pmcp test run --server https://mcp.example.com/mcp

# With HTTPS verification disabled (for self-signed certs in staging)
cargo pmcp test run \
  --server https://staging.mcp.example.com/mcp \
  --insecure
```

- Most production servers require authentication:

```bash
# Bearer token authentication
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --header "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."

# API key authentication
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --header "X-API-Key: your-api-key-here"

# Multiple headers
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --header "Authorization: Bearer ${TOKEN}" \
  --header "X-Request-ID: test-run-$(date +%s)" \
  --header "X-Environment: staging"
```

```bash
# Set credentials
export MCP_SERVER_URL="https://mcp.example.com/mcp"
export MCP_API_KEY="your-secret-key"

# Run tests (configuration file references env vars)
cargo pmcp test run --config tests/config/remote.yaml

# tests/config/remote.yaml
server:
  url: "${MCP_SERVER_URL}"
  headers:
    Authorization: "Bearer ${MCP_API_KEY}"
```

```bash
# tests/config/lambda.yaml
server:
  url: https://abc123.lambda-url.us-east-1.on.aws/mcp
  timeout_ms: 30000    # 30 seconds for cold start
  retry_count: 3       # Retry on timeout
  retry_delay_ms: 1000 # Wait 1s between retries

# First request allows extra time
first_request:
  timeout_ms: 60000    # 60 seconds for initial cold start

# CLI equivalent
cargo pmcp test run \
  --server https://abc123.lambda-url.us-east-1.on.aws/mcp \
  --timeout 30000 \
  --retry 3 \
  --retry-delay 1000
```

(table/list, cells in reading order)
- Platform-Specific Timeouts
- Different platforms have different characteristics:
- Platform
- First Request
- Subsequent
- Notes
- Lambda
- 30-60s
- 1-5s
- Cold starts
- Platform
- First Request
- Subsequent
- Notes
- Cloud Run
- Workers
- ECS/Kubernetes
- 15-30s
- <1s
- 1-5s
- 1-3s
- Cold starts with min-instances=0
- <100ms
- No cold starts
- 100-500ms
- Always warm

```yaml
# tests/config/cloudflare-workers.yaml
server:
  url: https://mcp-server.yourname.workers.dev
  timeout_ms: 5000     # Workers are fast
  retry_count: 1       # Rarely need retries

# tests/config/aws-lambda.yaml
server:
  url: https://abc123.lambda-url.us-east-1.on.aws/mcp
  timeout_ms: 45000    # Allow for cold starts
  retry_count: 3
```

```text
# tests/scenarios/performance/latency_requirements.yaml
```

```yaml
tags:
  - performance
  - sla

steps:
  - name: "Health check under 1s"
    tool: list_tables
    input: {}
    expect:
      success: true
      response_time_ms:
        less_than: 1000

  - name: "Simple query under 2s"
    tool: execute_query
    input:
      sql: "SELECT 1"
    expect:
      success: true
      response_time_ms:
        less_than: 2000

  - name: "Complex query under 5s"
    tool: execute_query
    input:
      sql: "SELECT COUNT(*) FROM large_table"
    expect:
      success: true
      response_time_ms:
        less_than: 5000
        greater_than: 0  # Ensure it's not cached
```

```text
# tests/scenarios/load/concurrent_requests.yaml
```

```yaml
tags:
  - load
  - performance

config:
  parallel: 10    # Run 10 concurrent tests
  iterations: 5   # Each runs 5 times

steps:
  - name: "Concurrent queries"
    tool: execute_query
    input:
      sql: "SELECT * FROM users LIMIT 10"
    expect:
      success: true
      response_time_ms:
        less_than: 3000  # Even under load
```


### Burst Traffic Simulation

```bash
# Simulate burst traffic
for i in {1..100}; do
  cargo pmcp test run \
    --server https://mcp.example.com/mcp \
    --scenario tests/scenarios/smoke/ \
    --quiet &
```


### done wait

```text
# Check results
```

(table/list, cells in reading order)
- grep -r "FAIL" test-results/
- Testing Different Environments
- Environment Configuration Files
- tests/

```text
config/
    local.yaml       # Local development
    staging.yaml     # Staging environment
    production.yaml  # Production (smoke only)
    preview.yaml     # PR preview environments
scenarios/
    smoke/           # Quick validation
    integration/     # Full integration tests
    performance/     # Performance tests
```

```yaml
# tests/config/local.yaml
server:
  url: http://localhost:3000/mcp
  timeout_ms: 5000
scenarios:
  - tests/scenarios/

# tests/config/staging.yaml
server:
  url: https://staging.mcp.example.com/mcp
  headers:
    Authorization: "Bearer ${STAGING_TOKEN}"
  timeout_ms: 30000
scenarios:
  - tests/scenarios/smoke/
  - tests/scenarios/integration/

# tests/config/production.yaml
server:
  url: https://mcp.example.com/mcp
  headers:
    Authorization: "Bearer ${PROD_TOKEN}"
  timeout_ms: 10000
scenarios:
  - tests/scenarios/smoke/  # Only smoke tests in prod
options:
  fail_fast: true
  parallel: 2  # Light load on production
```

```bash
# .github/workflows/pr-preview.yml
on:
  pull_request:
    types: [opened, synchronize]

jobs:
  deploy-preview:
    runs-on: ubuntu-latest
    outputs:
      preview_url: ${{ steps.deploy.outputs.url }}
    steps:
      - uses: actions/checkout@v4
      - id: deploy
        run: |
          # Deploy to preview environment
          URL=$(./deploy.sh preview --pr ${{ github.event.pull_request.number }})
          echo "url=$URL" >> $GITHUB_OUTPUT

  test-preview:
    needs: deploy-preview
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test preview environment
        run: |
          cargo pmcp test run \
            --server "${{ needs.deploy-preview.outputs.preview_url }}/mcp" \
            --scenario tests/scenarios/smoke/ \
            --format junit \
            --output preview-results.xml
```

```bash
# Maximum verbosity for debugging
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --scenario tests/scenarios/failing_test.yaml \
  --verbose \
  --show-requests \
  --show-responses
```


### Output includes:

[10:23:45.123] Connecting to https://mcp.example.com/mcp [10:23:45.234] → Request: tools/call

```json
{
  "name": "execute_query",
  "arguments": {
    "sql": "SELECT * FROM users"
  }
}
[10:23:45.567] ← Response (333ms):
{
  "content": [...],
  "isError": false
}
```

```yaml
              Actual: 333ms, Expected: < 200ms
```


### Saving Request/Response Logs

```bash
# Save all requests and responses
cargo pmcp test run \
  --server https://mcp.example.com/mcp \
  --scenario tests/scenarios/ \
  --log-requests test-logs/requests.json \
  --log-responses test-logs/responses.json

# Analyze failures
```

```bash
# Test with curl first
curl -X POST https://mcp.example.com/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
  }' \
  -v  # Verbose output shows headers, timing

# Check DNS resolution
```


### nslookup mcp.example.com

```text
# Check SSL certificate
```

openssl s_client -connect mcp.example.com:443 -servername mcp.example.com

```text
# Check connectivity
```

```text
# tests/scenarios/health/pre_check.yaml
```

```bash
tags:
  - health
  - prerequisite

steps:
  - name: "Server responds"
    tool: list_tables
    input: {}
    expect:
      success: true
      response_time_ms:
        less_than: 10000

  - name: "Database connected"
    tool: execute_query
    input:
      sql: "SELECT 1 as health"
    expect:
      success: true

# Run health check first, then full suite
cargo pmcp test run --scenario tests/scenarios/health/ --fail-fast && \
cargo pmcp test run --scenario tests/scenarios/
```

```text
# health_monitor.sh - Run periodic health checks
```


### while true; do

```bash
  if ! cargo pmcp test run \
    --server https://mcp.example.com/mcp \
    --scenario tests/scenarios/health/ \
    --quiet; then

    # Alert on failure
    curl -X POST https://hooks.slack.com/services/xxx \
      -d '{"text":"MCP Server health check failed!"}'
  fi

  sleep 60  # Check every minute
```

- Testing deployed servers requires:

1.


### Practice Ideas

These informal exercises help reinforce the concepts.


### CI/CD Integration

Integrating MCP server testing into CI/CD pipelines ensures every change is tested before reaching production.


### Why CI/CD matters for MCP servers:

Catches bugs before they reach production Ensures consistent quality across all changes Provides confidence for rapid iteration Documents the expected behavior through passing tests Enables safe, automated deployments


### Pipeline Architecture

A well-designed pipeline progresses through stages, with each stage adding more confidence.

```text
               MCP Server CI/CD Pipeline
                       COMMIT
STAGE 1: Build & Unit Tests
• cargo build --release
• cargo test --all-features
• cargo clippy
⏱ ~3-5 minutes
STAGE 2: Integration Tests
• Start local server with test database
• cargo pmcp test run (full suite)
• Generate coverage report
⏱ ~5-10 minutes
STAGE 3: Deploy to Staging
• Build container/package
• Deploy to staging environment
• Wait for deployment to stabilize
⏱ ~5-10 minutes
STAGE 4: Staging Tests
• Smoke tests (critical paths)
```

```text
• Full integration suite
• Performance validation
⏱ ~5-15 minutes
STAGE 5: Production Deployment
• Canary deployment (10%)
• Smoke tests on canary
• Gradual rollout (25%, 50%, 100%)
• Monitor for errors
⏱ ~15-30 minutes
```


### GitHub Actions Configuration

GitHub Actions is the most common CI/CD platform for Rust projects.


### Complete Workflow

This workflow demonstrates a full pipeline from commit to production.

```text
# .github/workflows/ci.yml
```


### name: CI/CD Pipeline

```bash
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ============================================
  # Stage 1: Build and Lint
  # ============================================
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Build
        run: cargo build --release

      - name: Upload binary
        uses: actions/upload-artifact@v4
        with:
          name: mcp-server
          path: target/release/mcp-server

  # ============================================
  # Stage 1b: Unit Tests (parallel with build)
  # ============================================
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run unit tests
        run: cargo test --all-features --lib

      - name: Generate coverage
// ... truncated, 229 more lines in source book ...
```


### Reusable Workflow

```text
# .github/workflows/mcp-test.yml
```


### name: MCP Test Workflow

```bash
on:
  workflow_call:
    inputs:
      server_url:
        required: true
        type: string
      scenarios:
        required: false
        type: string
        default: "tests/scenarios/"
      fail_fast:
        required: false
        type: boolean
        default: false
    secrets:
      api_key:
        required: false

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install pmcp
        run: cargo install cargo-pmcp

      - name: Run tests
        run: |
          AUTH_HEADER=""
          if [ -n "${{ secrets.api_key }}" ]; then
            AUTH_HEADER="--header \"Authorization: Bearer ${{ secrets.api_key }}\""
          fi

          FAIL_FAST=""
          if [ "${{ inputs.fail_fast }}" == "true" ]; then
            FAIL_FAST="--fail-fast"
          fi

          cargo pmcp test run \
            --server "${{ inputs.server_url }}" \
            $AUTH_HEADER \
            --scenario "${{ inputs.scenarios }}" \
            $FAIL_FAST \
            --format junit \
            --output test-results/results.xml

      - name: Upload results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: test-results
          path: test-results/
```


### Using the reusable workflow:

```text
# .github/workflows/test-all-environments.yml
```


### name: Test All Environments

```yaml
on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours

jobs:
  test-staging:
    uses: ./.github/workflows/mcp-test.yml
    with:
      server_url: https://staging.mcp.example.com/mcp
      scenarios: tests/scenarios/
    secrets:
      api_key: ${{ secrets.STAGING_API_KEY }}

  test-production:
    uses: ./.github/workflows/mcp-test.yml
    with:
      server_url: https://mcp.example.com/mcp
      scenarios: tests/scenarios/smoke/
      fail_fast: true
    secrets:
      api_key: ${{ secrets.PROD_API_KEY }}
```


### Test Result Reporting

Good reporting makes the difference between "tests failed" and "tests failed and here's exactly what broke." CI systems can parse standardized formats like JUnit XML to display results inline with pull requests.


### JUnit Format for CI Systems

JUnit XML is the universal format for test results.

```bash
# Generate JUnit XML for CI parsing
cargo pmcp test run \
  --server http://localhost:3000/mcp \
  --format junit \
  --output test-results/results.xml
```

```text
<testsuites name="mcp-tests" tests="15" failures="1" time="5.234">
  <testsuite name="smoke/health_check.yaml" tests="3" failures="0" time="1.234">
    <testcase name="Server responds" time="0.456"/>
    <testcase name="Execute simple query" time="0.789"/>
    <testcase name="Sample rows work" time="0.234"/>
  </testsuite>
  <testsuite name="integration/crud.yaml" tests="5" failures="1" time="2.567">
    <testcase name="Create record" time="0.234"/>
    <testcase name="Read record" time="0.123"/>
    <testcase name="Update record" time="0.345">
      <failure message="Assertion failed: content.contains('updated')">
Expected content to contain 'updated', got: '{"status":"unchanged"}'
      </failure>
    </testcase>
    <testcase name="Delete record" time="0.234"/>
    <testcase name="Verify deletion" time="0.123"/>
  </testsuite>
```

```rust
- name: Annotate failures
  if: failure()
  run: |
    # Parse JUnit and create annotations
    python3 << 'EOF'
    import xml.etree.ElementTree as ET

    tree = ET.parse('test-results/results.xml')
    for testsuite in tree.findall('.//testsuite'):
        for testcase in testsuite.findall('testcase'):
            failure = testcase.find('failure')
            if failure is not None:
                name = testcase.get('name')
                message = failure.get('message')
                print(f"::error title=Test Failed: {name}::{message}")
    EOF
```


### Slack Notifications

```yaml
- name: Notify on failure
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    payload: |
      {
        "text": "MCP Tests Failed",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "*MCP Server Tests Failed* :x:\n\n*Branch:* ${{ github.ref_name }}\n*Commit:* ${{ github.sha
}}\n*Author:* ${{ github.actor }}"
            }
          },
          {
            "type": "actions",
            "elements": [
              {
                "type": "button",
                "text": {"type": "plain_text", "text": "View Run"},
                "url": "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}"
              }
            ]
          }
        ]
      }
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```


### Parallel Test Execution

Large test suites can take a long time to run. The trade-off: more complex configuration and potential for resource contention.


### Matrix Strategy

GitHub Actions' matrix feature runs the same job with different parameters.

```bash
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        scenario-dir:
          - tests/scenarios/smoke
          - tests/scenarios/integration
          - tests/scenarios/performance
          - tests/scenarios/security
      fail-fast: false

    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: |
          cargo pmcp test run \
            --server http://localhost:3000/mcp \
            --scenario ${{ matrix.scenario-dir }}/ \
            --format junit \
            --output test-results/${{ matrix.scenario-dir }}.xml

  aggregate:
    needs: test
    runs-on: ubuntu-latest
    if: always()
    steps:
      - name: Download all results
        uses: actions/download-artifact@v4
        with:
          path: all-results

      - name: Merge results
        run: |
          # Combine all JUnit files
          npx junit-merge -d all-results -o final-results.xml
```


### Parallel Within mcp-tester

```bash
# Run scenarios in parallel
cargo pmcp test run \
  --server http://localhost:3000/mcp \
  --scenario tests/scenarios/ \
  --parallel 4  # Run 4 scenarios concurrently
```


### Caching Strategies

Rust builds are notoriously slow because of the compilation model.


### Rust Build Cache

The  rust-cache  action intelligently caches compiled dependencies while invalidating when Cargo.lock or Cargo.toml changes.

```yaml
- name: Cache Rust
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: "mcp-server"
    cache-targets: true
```


### Docker Layer Cache

```yaml
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3

- name: Build and push
  uses: docker/build-push-action@v5
  with:
    context: .
    push: true
    tags: ghcr.io/${{ github.repository }}:${{ github.sha }}
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

- Effective CI/CD integration requires:

1.


### Practice Ideas

These informal exercises help reinforce the concepts.


### Regression Testing

Regression testing ensures that bug fixes stay fixed and new features don't break existing functionality.

```text
                 Regression Testing Purpose
Without Regression Tests:
   v1.0: Bug found → Bug fixed [ok]
   v1.1: New feature added
   v1.2: Bug reappears! [no]
   v1.3: Same bug fixed again...
   v1.4: Bug reappears again...
With Regression Tests:
   v1.0: Bug found → Bug fixed + Regression test added [ok]
   v1.1: New feature added, regression test passes [ok]
   v1.2: Code change would reintroduce bug...
         → Regression test FAILS [no]
         → Developer catches issue before release
         → Bug never reaches production again!
```

```text
# tests/scenarios/regression/issue-42-empty-result.yaml
```


### name: "Regression #42 - Empty query result handling" description: |

```yaml
  Bug: Server returned 500 when query returned empty results.
  Fixed in: v1.2.1 (commit abc123)
  Root cause: Missing null check in result formatting.

  This test ensures empty results are handled gracefully.

tags:
  - regression
  - issue-42
  - critical

# Link to original issue
metadata:
  issue_url: https://github.com/example/mcp-server/issues/42
  fixed_in: v1.2.1
  fixed_by: commit abc123

steps:
  - name: "Query returning empty results should succeed"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE id = -999999"
    expect:
      success: true
      content:
        type: text
        contains: "0 rows"

  - name: "Empty table query should succeed"
    tool: execute_query
    input:
      sql: "SELECT * FROM empty_table"
    expect:
      success: true
```


### From Production Incidents

After a production incident, capture the exact sequence that caused the problem:

```text
# tests/scenarios/regression/incident-2024-01-15.yaml
```


### name: "Regression - Production incident 2024-01-15" description: |

```yaml
  Incident: Server crashed under specific query pattern.
  Impact: 15 minutes of downtime.
  Root cause: Memory exhaustion when joining large tables without LIMIT.

  This test reproduces the exact conditions that triggered the crash.

tags:
  - regression
  - incident
  - performance
  - critical

metadata:
  incident_date: "2024-01-15"
  postmortem_url: https://wiki.example.com/postmortems/2024-01-15

steps:
  - name: "Large join query with limit doesn't crash"
    tool: execute_query
    input:
      sql: "SELECT u.*, o.* FROM users u JOIN orders o ON u.id = o.user_id LIMIT 100"
    expect:
      success: true
      response_time_ms:
        less_than: 30000  # Should complete, not timeout

  - name: "Query without limit is rejected"
    tool: execute_query
    input:
      sql: "SELECT u.*, o.* FROM users u JOIN orders o ON u.id = o.user_id"
    expect:
      error:
        message_contains: "LIMIT required"
```

```text
# tests/scenarios/regression/unicode-handling.yaml
```


### name: "Regression - Unicode edge cases" description: |

```yaml
  Various Unicode handling edge cases that have caused issues.

tags:
  - regression
  - unicode
  - i18n

steps:
  - name: "Emoji in query values"
    tool: execute_query
    input:
      sql: "SELECT * FROM messages WHERE content LIKE '%🎉%'"
    expect:
      success: true

  - name: "Chinese characters in table names"
    tool: execute_query
    input:
      sql: "SELECT * FROM \"用户表\" LIMIT 1"
    expect:
      success: true

  - name: "RTL text handling"
    tool: execute_query
    input:
      sql: "SELECT * FROM messages WHERE content = 'ابحرم'"
    expect:
      success: true

  - name: "Zero-width characters"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE name = 'test\u200B'"
    expect:
      success: true
```

```text
README.md              # Overview and organization guide
by-issue/              # Organized by issue number
    issue-42.yaml
    issue-87.yaml
    issue-123.yaml
by-component/          # Organized by affected component
    auth/
        oauth-token-refresh.yaml
        session-expiry.yaml
    query/
        null-handling.yaml
        unicode.yaml
    transport/
        sse-reconnect.yaml
        timeout-handling.yaml
by-severity/           # Organized by severity
    critical/
        data-loss-prevention.yaml
        security-bypass.yaml
    medium/
        display-issues.yaml
        performance.yaml
incidents/             # Production incidents
    2024-01-15.yaml
    2024-02-20.yaml
```


### Naming Conventions

```text
# Good: Descriptive, includes issue reference
```


### name: "Regression #42 - Empty result set handling"

```text
# Good: Includes component and behavior
```


### name: "Regression - Query: NULL value comparison"

```text
# Bad: Too vague
```


### name: "Bug fix test"

```text
# Bad: No context
```

```yaml
tags:
  - regression          # All regression tests
  - issue-42           # Specific issue number
  - query              # Affected component
  - critical           # Severity level
  - fixed-v1.2.1       # Version where fixed
  - database           # Related system
```


### Query tests by tags:

```bash
# Run all critical regressions
cargo pmcp test run --scenario tests/scenarios/regression/ --tag critical

# Run regressions for a specific component
cargo pmcp test run --scenario tests/scenarios/regression/ --tag query

# Run regressions fixed in a specific version
cargo pmcp test run --scenario tests/scenarios/regression/ --tag fixed-v1.2.1
```

```text
# .github/workflows/regression-review.yml
```


### name: Monthly Regression Review

```yaml
on:
  schedule:
    - cron: '0 9 1 * *'  # First of each month

jobs:
  generate-report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Count regression tests
        run: |
          echo "## Regression Test Report" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "Total regression tests: $(find tests/scenarios/regression -name '*.yaml' | wc -l)" >>
```


### $GITHUB_STEP_SUMMARY

```text
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "### By Severity" >> $GITHUB_STEP_SUMMARY
          echo "- Critical: $(grep -r 'critical' tests/scenarios/regression --include='*.yaml' | wc -l)" >>
```


### $GITHUB_STEP_SUMMARY

```text
          echo "- Medium: $(grep -r 'medium' tests/scenarios/regression --include='*.yaml' | wc -l)" >>
```


### $GITHUB_STEP_SUMMARY

```yaml
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "### Recent additions (last 30 days)" >> $GITHUB_STEP_SUMMARY
          find tests/scenarios/regression -name '*.yaml' -mtime -30 >> $GITHUB_STEP_SUMMARY

      - name: Check for stale tests
        run: |
          echo "### Tests without recent validation" >> $GITHUB_STEP_SUMMARY
          # Find tests not modified in 6 months
          find tests/scenarios/regression -name '*.yaml' -mtime +180 >> $GITHUB_STEP_SUMMARY
```

```text
# tests/scenarios/regression/deprecated/issue-15.yaml
```


### name: "DEPRECATED - Issue #15" description: |

```yaml
  This regression test is deprecated as of v2.0.0.

  Reason: The affected component (legacy auth) was completely replaced
  in v2.0.0 with a new OAuth implementation.

  Original issue: #15
  Deprecated in: v2.0.0
  Safe to remove after: v3.0.0

tags:
  - regression
  - deprecated
  - issue-15

# Skip this test but keep for documentation
```


### skip: true skip_reason: "Component replaced in v2.0.0"

```yaml
steps:
  # Original test preserved for reference
  - name: "Legacy auth token refresh"
    tool: refresh_token
    input:
      token: "expired_token"
    expect:
      success: true
```

```text
# Before: Multiple similar files
# - issue-45-null-string.yaml
# - issue-67-empty-string.yaml
# - issue-89-whitespace-string.yaml

# After: Consolidated test
# tests/scenarios/regression/string-edge-cases.yaml
```


### name: "Regression - String edge cases" description: |

```yaml
  Consolidated test for string handling edge cases.
  Covers issues: #45, #67, #89

tags:
  - regression
  - issue-45
  - issue-67
  - issue-89
  - strings

steps:
  - name: "NULL string handling (#45)"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE name IS NULL"
    expect:
      success: true

  - name: "Empty string handling (#67)"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE name = ''"
    expect:
      success: true

  - name: "Whitespace-only string handling (#89)"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE name = '   '"

    expect:
      success: true
```

```text
# .github/workflows/schema-check.yml
```


### name: Schema Change Detection

```bash
on:
  pull_request:
    paths:
      - 'src/tools/**'
      - 'src/schema/**'

jobs:
  check-schema:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Compare schemas
        run: |
          # Get schema from main branch
          git show origin/main:schema.json > /tmp/old-schema.json

          # Get current schema
          cargo run --release &
          sleep 5
          curl http://localhost:3000/schema > /tmp/new-schema.json

          # Compare
          if ! diff /tmp/old-schema.json /tmp/new-schema.json > /dev/null; then
            echo "::warning::Schema has changed. Review regression tests."
            diff /tmp/old-schema.json /tmp/new-schema.json
          fi

      - name: Run affected regression tests
        run: |
          # Identify changed tools
          CHANGED_TOOLS=$(diff /tmp/old-schema.json /tmp/new-schema.json | grep -oP '"name":\s*"\K[^"]+')

          # Run regression tests for those tools

          for tool in $CHANGED_TOOLS; do
            cargo pmcp test run \
              --scenario tests/scenarios/regression/ \
              --tag "$tool"
          done
```

```text
# tests/scenarios/regression/performance/baseline.yaml
```

name: "Performance - Baseline measurements" description: "Track performance to detect regressions"

```bash
tags:
  - regression
  - performance
  - baseline

steps:
  - name: "Simple query baseline"
    tool: execute_query
    input:
      sql: "SELECT 1"
    expect:
      success: true
      response_time_ms:
        less_than: 100

  - name: "Table listing baseline"
    tool: list_tables
    input: {}
    expect:
      success: true
      response_time_ms:
        less_than: 500

  - name: "Complex query baseline"
    tool: execute_query
    input:
      sql: "SELECT * FROM users JOIN orders ON users.id = orders.user_id LIMIT 100"
    expect:
      success: true
      response_time_ms:
        less_than: 2000

# Compare performance with baseline
cargo pmcp test run \
  --scenario tests/scenarios/regression/performance/ \
  --format json \
  --output current-perf.json

# Historical comparison
cargo pmcp test compare \
  --current current-perf.json \
  --baseline baseline-perf.json \
  --threshold 20%  # Fail if >20% slower
```

```text
# Workflow for bug fixes
```


### 2. Include Context

```text
# Good: Full context for future maintainers
```

name: "Regression #123 - SQL injection in table parameter" description: |

```yaml
  Bug: The `table` parameter in get_sample_rows was passed directly
  to SQL without sanitization, allowing SQL injection attacks.

  Example attack vector:
    table: "users; DROP TABLE secrets; --"

  Fix: Added input validation using allowed table list.

  Security impact: HIGH - Could leak or destroy data.
  Fixed by: @developer in PR #456

tags:
  - regression
  - security
  - critical
  - issue-123

steps:
  - name: "SQL injection attempt is blocked"
    tool: get_sample_rows
    input:
      table: "users; DROP TABLE secrets; --"
    expect:
      error:
        message_contains: "Invalid table name"
```


### 3. Keep Tests Fast

```yaml
# Good: Focused test
steps:
  - name: "Specific edge case"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE id = NULL"
    expect:
      success: true

# Bad: Slow, broad test
steps:
  - name: "Test everything"
    tool: execute_query
    input:
      sql: "SELECT * FROM large_table"  # Slow!
    expect:
      success: true
```


### 4. Make Tests Independent

```yaml
# Good: Self-contained test
steps:
  - name: "Create test data"
    tool: insert_record
    input:
      table: users
      data: { id: 99999, name: "test" }

  - name: "Test specific behavior"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE id = 99999"
    expect:
      success: true

  - name: "Clean up"
    tool: delete_record
    input:
      table: users
      id: 99999

# Bad: Depends on external state
steps:
  - name: "Assumes data exists"
    tool: execute_query
    input:
      sql: "SELECT * FROM users WHERE id = 1"  # Might not exist
```

1. Create tests with every bug fix - Never fix a bug without a test 2.


### Practice Ideas

These informal exercises help reinforce the concepts.
