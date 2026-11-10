---
chapter: 24
title: "Operations and Monitoring"
part: "Part VII: Observability"
use_when: "Load testing config and scenarios; production monitoring."
suggested_prerequisites: [ch23]
source_kb: 38
extracted_kb: 23
code_blocks: 57
quality: auto-extracted
---

# Ch 24: Operations and Monitoring

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

(table/list, cells in reading order)
- Operations and Monitoring
- pmcp.run Dashboard
- Alerting and Incidents
- Performance Optimization

Because load testing answers questions that functional tests can't: How many concurrent clients can your server handle?

```text
                  Testing Pyramid
```

```text
    Load Testing           <-- You are here
(capacity & limits)
 Integration Testing
(end-to-end flows)
   Unit Testing
(individual functions)
```

```text
  Load testing sits at the top of the pyramid.
  It tests what happens when many clients
  hit your server at the same time.
```

You don't need to manually construct JSON-RPC payloads -- the tool does it for you.


### Your First Load Test

Let's run your first load test in three steps.


### Step 1: Start Your Server

You'll need a running MCP server to test against.


### Open a terminal and start the calculator server:

```bash
# Build and run in release mode for accurate performance numbers
cargo run --release --bin calculator
```

Make sure it's running on  http://localhost:3000/mcp . You should see a log line like:


### INFO  Starting calculator server

Leave this terminal open -- the server needs to keep running while we test it.


### Step 2: Generate a Config

Open a second terminal in the same project directory.

```bash
# Connect to the server and generate a config
cargo pmcp loadtest init http://localhost:3000/mcp
```


### You'll see output like:

Discovering server schema at http://localhost:3000/mcp...

```toml
# Generated from server: http://localhost:3000/mcp

[settings]
# Number of concurrent virtual users
virtual_users = 10

# Test duration in seconds
duration_secs = 60

# Per-request timeout in milliseconds
timeout_ms = 5000

# Scenario steps discovered from server capabilities.
# Adjust weights to control the mix of operations.

[[scenario]]
type = "tools/call"
weight = 25
tool = "add"
# arguments = {}

[[scenario]]
type = "tools/call"
weight = 25
tool = "subtract"
# arguments = {}

[[scenario]]
type = "tools/call"
weight = 25
tool = "multiply"
# arguments = {}

[[scenario]]
type = "tools/call"
weight = 25
tool = "divide"
# arguments = {}
```

```toml
[[scenario]]
type = "tools/call"
weight = 25
tool = "add"
arguments = { a = 42, b = 17 }

[[scenario]]
type = "tools/call"
weight = 25
tool = "subtract"
arguments = { a = 100, b = 37 }

[[scenario]]
type = "tools/call"
weight = 25
tool = "multiply"
arguments = { a = 6, b = 7 }

[[scenario]]
type = "tools/call"
weight = 25
tool = "divide"
arguments = { dividend = 100, divisor = 4 }
```

```bash
cargo pmcp loadtest run http://localhost:3000/mcp
```

```text
          /\      |  cargo-pmcp loadtest
         /  \     |
    /\  /    \    |  target:    http://localhost:3000/mcp
   /  \/      \   |  vus:       10
  /    \       \  |  duration:  60s
```


### /      \       \ |  scenarios: 4 steps

```text
  mcp_req_duration............: p50=12ms  p95=45ms  p99=120ms
  mcp_req_success_count.......: 4820
  mcp_req_error_count.........: 0
  mcp_req_error_rate..........: 0.0%
  mcp_req_throughput..........: 80.3 req/s
  mcp_req_total...............: 4820
  mcp_req_elapsed.............: 60.0s
```

```text
p50=12ms

p95=45ms

p99=120ms
```

(table/list, cells in reading order)
- Meaning
- Half your requests completed in under 12ms (the "typical" experience)
- 95% of requests finished in under 45ms (most users' worst case)
- 99% of requests finished in under 120ms (the unlucky 1%)
- mcp_req_throughput
- Your server handled ~80 requests per second
- mcp_req_error_rate
- No errors -- the calculator is rock solid

Congratulations -- you've run your first MCP load test!

The important thing is that you have a baseline to compare against.


### Understanding the Config File

Now let's understand what that generated config actually means.

```toml
[settings]
virtual_users = 10          # Simulated clients hitting your server simultaneously
duration_secs = 60          # How long the test runs (in seconds)
timeout_ms = 5000           # When to give up on a single request (5 seconds)
# expected_interval_ms = 100  # For coordinated omission correction (we'll cover this later)
```

(table/list, cells in reading order)
- virtual_users
- Number of concurrent simulated clients
- "10 AI clients using my server at the same time"
- duration_secs
- Total test duration
- "Hammer the server for 60 seconds"
- timeout_ms
- Per-request timeout
- "If a request takes more than 5 seconds, count it as failed"
- expected_interval_ms
- Coordinated omission correction interval


### The [[scenario]] Blocks

Each  [[scenario]]  block defines an MCP operation the load tester will execute.

Weights do not need to sum to 100.


### The tool supports three MCP operation types:

```toml
# Call a tool (like tools/call in the MCP protocol)
[[scenario]]
type = "tools/call"
weight = 60
tool = "calculate"
arguments = { expression = "2+2" }

# Read a resource (like resources/read)
[[scenario]]
type = "resources/read"
weight = 30
uri = "file:///data/config.json"

# Get a prompt (like prompts/get)
[[scenario]]
type = "prompts/get"
weight = 10
prompt = "summarize"
arguments = { text = "Hello world" }
```

Try this: Go back to your  .pmcp/loadtest.toml  and change the weights so that  add  gets 80% of the traffic while the other tools split the remaining 20%.

```toml
[[scenario]]
type = "tools/call"
weight = 80
tool = "add"
arguments = { a = 42, b = 17 }

[[scenario]]
type = "tools/call"
weight = 7
tool = "subtract"
arguments = { a = 100, b = 37 }

[[scenario]]
type = "tools/call"
weight = 7
tool = "multiply"
arguments = { a = 6, b = 7 }

[[scenario]]
type = "tools/call"
weight = 6
tool = "divide"
arguments = { dividend = 100, divisor = 4 }
```


### Writing Your Own Config from Scratch

You don't have to use  loadtest init  -- you can write a config file from scratch.


### 2. Add the required  [settings]  block with all three required fields:

```toml
[settings]
virtual_users = 5        # Start small -- you can always increase
duration_secs = 30       # 30 seconds is enough for a quick check
timeout_ms = 5000        # 5 second timeout
```


### 3. Add at least one  [[scenario]]  block:

```toml
[[scenario]]
type = "tools/call"
weight = 100             # Only one scenario, so weight doesn't matter
tool = "add"
arguments = { a = 1, b = 2 }
```


### 4. Run with your custom config:

```bash
cargo pmcp loadtest run http://localhost:3000/mcp --config .pmcp/loadtest.toml
```


### Validation rules the tool enforces:

At least one  [[scenario]]  must be defined Total weight across all scenarios must be greater than zero If  [[stage]]  blocks are present, each stage must have  duration_secs > 0


### Staged Load Profiles

So far, you've been running flat load tests -- all virtual users start immediately and hammer the server at a constant rate for the entire duration.

(table/list, cells in reading order)
- Flat Load vs Staged Load
- Here's the difference visually:
- Flat load -- constant pressure:
- VUs 10 |##################################################

```text
    |##################################################
    |##################################################
```

```text
0
```

```text
    0s                                              60s
```

```text
50 |
```

```text
    |                /                        \
    |              /                            \
```


### 25 |            /                                \

```text
    |          /                                    \
```


### 10 |        /                                        \

```text
0
```

```text
    0s   10s        30s                 90s        120s
         ramp-up        hold             ramp-down
```

VUs start at 0, ramp up to 50 over 30 seconds, hold at 50 for 60 seconds, then ramp back down.

(table/list, cells in reading order)
- When to use each:
- Goal
- Use
- Measure baseline throughput at a known load level
- Flat load
- Find your server's breaking point
- Staged load (ramp up)
- Simulate realistic traffic patterns (morning ramp-up)
- Staged load
- Run a quick smoke test
- Flat load with low VUs
- Stress test graceful degradation
- Staged load (ramp to extreme)
- Soak test for memory leaks
- Flat load over a long duration
- Building Your First Staged Profile


### Here's a classic ramp-up, hold, ramp-down pattern:

```toml
[settings]
virtual_users = 10       # Ignored when stages are present
duration_secs = 60       # Ignored when stages are present
timeout_ms = 5000

[[scenario]]
type = "tools/call"
weight = 100
tool = "add"
arguments = { a = 42, b = 17 }

# Stage 1: Ramp up from 0 to 20 VUs over 30 seconds
[[stage]]
target_vus = 20
duration_secs = 30

# Stage 2: Hold at 20 VUs for 60 seconds
[[stage]]
target_vus = 20
duration_secs = 60

# Stage 3: Ramp down from 20 to 0 VUs over 30 seconds
[[stage]]
target_vus = 0
duration_secs = 30
```

Important: When  [[stage]]  blocks are present,  settings.virtual_users  and  settings.duration_secs  are ignored -- the stages control everything. A warning is printed if you have both settings and stages.


### How staged ramp works internally:

Ramp up (target > current): New VUs are spawned with linear stagger over the stage duration.

(table/list, cells in reading order)
- Designing Profiles for Different Goals
- Different testing goals call for different stage shapes:
- Goal
- Profile Shape
- Example Stages
- Capacity planning
- Gradual ramp to find limits
- 0->10 (30s), 10->50 (60s), 50->100 (60s), 100->0 (30s)
- Soak testing
- Quick ramp to steady state, long hold
- 0->20 (10s), 20->20 (300s), 20->0 (10s)
- Stress testing
- Aggressive ramp to extreme levels
- 0->50 (10s), 50->200 (30s), 200->0 (10s)
- Realistic traffic
- Morning ramp, peak hours, evening decline
- 0->10 (30s), 10->30 (60s), 30->30 (120s), 30->5 (60s)


### Reading Your Results

You've run load tests and seen the summary output.


### Percentiles: P50, P95, P99

Percentiles tell you how many of your requests finished within a given time.

(table/list, cells in reading order)
- Percentile
- What it means
- Analogy
- P50 (median)
- Half of all requests are faster than this
- "The typical user experience"
- P95
- P99
- 95% of requests are faster than this
- "Almost the worst case -- what most users will see at worst"
- 99% of requests are faster than this
- "The unlucky 1% -- the tail that bites you in production"

When load testing MCP servers, always focus on P95 and P99 -- they reveal the worst-case experience for your users' AI clients.

```text
          /\      |  cargo-pmcp loadtest          <-- Tool name
         /  \     |
    /\  /    \    |  target:    http://localhost:3000/mcp   <-- Server URL
   /  \/      \   |  vus:       10                 <-- Virtual users
  /    \       \  |  duration:  60s                <-- Test duration
```


### /      \       \ |  scenarios: 3 steps            <-- Scenario count

```yaml
  mcp_req_duration............: p50=42ms  p95=120ms  p99=350ms
  mcp_req_success_count.......: 950
  mcp_req_error_count.........: 50
  mcp_req_error_rate..........: 5.0%
  mcp_req_throughput..........: 16.7 req/s
  mcp_req_total...............: 1000
  mcp_req_elapsed.............: 60.0s

  errors:
    timeout...................: 30
    jsonrpc...................: 15
    http......................: 5
```

Success latency percentiles (P50, P95, P99) in milliseconds.


### mcp_req_error_count

Total number of requests that failed.


### mcp_req_error_rate

Percentage of requests that failed (error_count / total).


### mcp_req_throughput

Requests per second (total_requests / elapsed_time).


### mcp_req_total

Total requests sent (success + error).


### mcp_req_elapsed

Wall-clock time the test ran.

(table/list, cells in reading order)
- Category
- timeout
- jsonrpc
- http
- What it means
- Common cause
- Request exceeded the configured  timeout_ms
- Server overloaded, resource exhaustion
- JSON-RPC error returned by the server
- Method not found, invalid params, internal error
- HTTP transport error (4xx or 5xx)
- Server bug, bad gateway, rate limiting
- connection
- Connection-level failure
- Server down, DNS failure, TLS error

```text
  per-tool metrics:

  tool                             reqs      rate   err%     p50     p95     p99

  calculate                         680    11.3/s   2.1%    42ms   120ms   350ms
  search                            120     2.0/s   8.3%    85ms   250ms   800ms
```

In the example above,  search  has a much higher error rate and P99 than calculate  -- that's where you should focus optimization efforts.


### HdrHistogram and Coordinated Omission

The loadtest engine uses HdrHistogram (High Dynamic Range Histogram) for latency measurement.

During a 10-second stall, the tool records just ONE slow request instead of the ~100 requests that would have been sent during those 10 seconds.


### Without Correction:

```yaml
  Time:     0s    1s    2s    3s    4s    5s    6s    7s    8s    9s   10s
  Events:   .     .     .     .     .     .     .     .     .     .    X
                                                                       ^
                                                     Only 1 sample recorded!
                                                     P99 looks great.
```


### With Correction:

```yaml
  Time:     0s    1s    2s    3s    4s    5s    6s    7s    8s    9s   10s
  Events:   x     x     x     x     x     x     x     x     x     x    X
            ^     ^     ^     ^     ^     ^     ^     ^     ^     ^
            Synthetic samples filled in to represent missed requests.
            P99 reflects reality -- many users saw high latency.
```

How correction works in the engine: The engine uses HdrHistogram's  record_correct()  method instead of plain  record() .

- Recommended setting

```toml
expected_interval_ms = 50
```


### Standard APIs (50-200ms)

```toml
expected_interval_ms = 100  (default)
```


### Slow operations (> 500ms)

```toml
expected_interval_ms = 500
```

This is why the numbers from  cargo pmcp loadtest  are more accurate than simple averages or basic histograms.


### Finding Your Server's Breaking Point

Every server has a limit.


### What the Breaking Point Detector Does

The loadtest engine includes a self-calibrating breaking point detector that watches your server's behavior over time and flags when things start going wrong. Unlike threshold-based alerting (which requires you to know "good" values in advance), the detector learns what "normal" looks like for YOUR server and then detects when behavior deviates.


### Here's how it works, step by step:

```text
  Rolling Window (20 seconds of history)
```

```text
Sample 1  Sample 2  ...  Sample 10
    Baseline        Recent
(older half)     (newer half)
```

```text
  Every 2 seconds, a new sample is added.
  The detector splits the window in half:
  - Older half = baseline (what "normal" looks like)
  - Newer half = recent (what's happening now)
  Then it compares recent vs baseline.
```


### Two detection conditions (either one triggers):

1. Both conditions must be

(table/list, cells in reading order)
- Detection constant
- Value
- Purpose
- DEFAULT_WINDOW_SIZE
- 10 samples
- Number of 2-second snapshots in the rolling window
- ERROR_RATE_ABSOLUTE_THRESHOLD
- ERROR_RATE_RELATIVE_MULTIPLIER
- P99_RELATIVE_MULTIPLIER
- 10%
- 2.0x
- 3.0x
- Minimum error rate before detection can trigger
- Error rate must exceed this multiple of baseline
- P99 must exceed this multiple of baseline
- Hands-On: Find Your Calculator Server's Breaking Point

```toml
[settings]
virtual_users = 1
duration_secs = 10
timeout_ms = 5000

[[scenario]]
type = "tools/call"
weight = 100
tool = "add"
arguments = { a = 42, b = 17 }

# Ramp from 0 to 10 VUs over 30 seconds (warm up)
[[stage]]
target_vus = 10
duration_secs = 30

# Ramp from 10 to 50 VUs over 60 seconds
[[stage]]
target_vus = 50
duration_secs = 60

# Ramp from 50 to 100 VUs over 60 seconds
[[stage]]
target_vus = 100
duration_secs = 60

# Ramp down to 0
[[stage]]
target_vus = 0
duration_secs = 30
```


### 2. Make sure your calculator server is still running, then execute:

3. At some point, you may see a warning like:


### Or it might trigger on latency instead:

Breaking point detected at 65 VUs (latency_degradation: P99 450ms exceeds 3.0x baseline 120ms)


### Reading the JSON Report

Every test run saves a JSON report to  .pmcp/reports/ .


### .pmcp/reports/loadtest-2026-02-27T14-30-45.json

The report contains everything from the terminal output plus additional detail.

```json
{
  "breaking_point": {
    "detected": true,
    "vus": 42,
    "reason": "error_rate_spike",
    "detail": "Error rate 15.0% exceeds threshold (>10% and >2.0x baseline 3.2%)",
    "timestamp": "2026-02-27T14:31:15.456789Z"
  }
}
```

```text
# Get the most recent report file
```


### REPORT=$(ls -t .pmcp/reports/loadtest-*.json | head -1)

```text
# Extract the breaking point VU count
```


### jq '.breaking_point.vus' "$REPORT"

```text
# View the full breaking point details
```


### jq '.breaking_point' "$REPORT"

```text
# Get P99 latency
```


### jq '.metrics.latency.p99_ms' "$REPORT"

The JSON report uses schema version  "1.1" . External tools should check the  schema_version  field for parser compatibility.


### Load Testing a Deployed Server

Local testing tells you the floor -- the best your server can do with no network latency, no load balancers, and dedicated resources.

```bash
# Generate a config from the deployed server
cargo pmcp loadtest init https://staging.example.com/mcp

# Run the test
cargo pmcp loadtest run https://staging.example.com/mcp
```


### There are a few considerations for remote testing:

Network latency adds to every request.

```toml
[settings]
virtual_users = 10
duration_secs = 60
timeout_ms = 15000       # 15 seconds for cloud deployments
expected_interval_ms = 200  # Deployed servers are slower than localhost
```

Don't hit a staging server with 200 VUs unless you've coordinated with your team.


### A Practical Example: Staging Environment Load Test

Let's walk through a complete load test of a deployed MCP server.


### Step 1: Generate the config from the staging server

```bash
cargo pmcp loadtest init https://staging.example.com/mcp
```

This connects to the staging server, discovers its tools, and generates  .pmcp/loadtest.toml .

```toml
[settings]
virtual_users = 20
duration_secs = 120
timeout_ms = 10000        # 10s timeout for deployed server
expected_interval_ms = 200  # Match the server's typical response time

[[scenario]]
type = "tools/call"
weight = 70               # Most traffic is tool calls
tool = "search"
arguments = { query = "test query" }

[[scenario]]
type = "resources/read"
weight = 20               # Some resource reads
uri = "config://settings"

[[scenario]]
type = "prompts/get"
weight = 10               # Occasional prompt fetches
prompt = "summarize"
arguments = { text = "sample text for testing" }
```


### Step 3: Run the test

```bash
cargo pmcp loadtest run https://staging.example.com/mcp
```

For a quick test, you can override VUs and duration from the command line without editing the config:

```bash
# Quick smoke test: 5 VUs for 30 seconds
cargo pmcp loadtest run https://staging.example.com/mcp --vus 5 --duration 30

# Full load test with the config file settings
cargo pmcp loadtest run https://staging.example.com/mcp
```


### Step 4: Interpret the results

```text
  mcp_req_duration............: p50=85ms  p95=250ms  p99=480ms
  mcp_req_success_count.......: 2340
  mcp_req_error_count.........: 12
  mcp_req_error_rate..........: 0.5%
  mcp_req_throughput..........: 19.5 req/s
  mcp_req_total...............: 2352
  mcp_req_elapsed.............: 120.0s
```

What this tells you: "At 20 concurrent clients, the staging server handles requests with P99 under 500ms and a 0.5% error rate.


### Capacity Planning from Results

Here's how to turn load test numbers into deployment decisions.

(table/list, cells in reading order)
- VU Count
- P99
- Error Rate
- Throughput
- Conclusion
- 5
- 10
- 20
- 30
- 50
- 120ms
- 180ms
- 350ms
- 800ms
- 2500ms
- 0.0%
- 0.1%
- 0.5%
- 2.1%
- 12%
- 10 req/s
- 18 req/s
- 32 req/s
- 38 req/s
- 35 req/s
- Comfortable -- server barely notices
- Good -- scaling linearly
- Acceptable -- latency rising
- Warning -- P99 approaching 1s
- Breaking -- throughput plateaued, errors spiking


### Planning rules of thumb:

Target headroom: Plan for 2x your expected peak concurrent users.


### Decision framework:

```text
  IF breaking point VUs > 2x expected peak users:
      You have sufficient capacity. Ship it.

  IF breaking point VUs is 1-2x expected peak:
      You're close to the edge. Consider:
      - Optimizing the slowest tools (check per-tool metrics)
      - Increasing server resources (CPU, memory, connection pool)
      - Adding horizontal scaling (load balancer + multiple instances)

  IF breaking point VUs < expected peak:
      You need to scale before deploying. Either:
      - Optimize the server code
      - Increase resources significantly
      - Add more instances behind a load balancer
```

(table/list, cells in reading order)
- Quick Reference
- CLI Commands
- Command
- Description

```bash
cargo pmcp loadtest run <URL>
```


### Run a load test against an MCP server

```bash
cargo pmcp loadtest run <URL> --vus 20
```


### Override virtual user count

```bash
cargo pmcp loadtest run <URL> --duration 120
```

```bash
cargo pmcp loadtest run <URL> --config path.toml
```


### Use a specific config file

```bash
cargo pmcp loadtest run <URL> --iterations 1000
```


### Stop after N iterations

```bash
cargo pmcp loadtest run <URL> --no-report
```


### Skip JSON report output

```bash
cargo pmcp loadtest run <URL> --no-color
```


### Disable colored output (CI-friendly)

```bash
cargo pmcp loadtest init
```


### Generate a default config template

```bash
cargo pmcp loadtest init <URL>
```


### Generate config with schema discovery

```bash
cargo pmcp loadtest init <URL> --force
```

```toml
# --- Required settings ---
[settings]
virtual_users = 10        # Concurrent simulated clients
duration_secs = 60        # Test duration
timeout_ms = 5000         # Per-request timeout

# --- Optional settings ---
# expected_interval_ms = 100   # Coordinated omission interval
# request_interval_ms = 15000  # Delay between requests per VU

# --- Scenario types ---
[[scenario]]
type = "tools/call"       # Call an MCP tool
weight = 60
tool = "tool-name"
arguments = { key = "value" }

[[scenario]]
type = "resources/read"   # Read an MCP resource
weight = 30
uri = "file:///path"

[[scenario]]
type = "prompts/get"      # Get an MCP prompt
weight = 10
prompt = "prompt-name"
arguments = { key = "value" }

# --- Optional staged load ---
[[stage]]
target_vus = 20           # Ramp to this many VUs
duration_secs = 30        # Over this many seconds
```

(table/list, cells in reading order)
- Metric Names Reference
- Metric
- mcp_req_duration
- mcp_req_success_count
- mcp_req_error_count
- Type
- Latency
- Counter
- Counter
- Description
- P50/P95/P99 success latency (ms)
- Total successful requests
- Total failed requests
- mcp_req_error_rate
- Percentage
- Error rate (errors / total)
- mcp_req_throughput
- Rate
- Requests per second
- mcp_req_total
- Counter
- Total requests (success + error)
- mcp_req_elapsed
- Duration
- Wall-clock test time

For the complete reference covering JSON report schema, per-tool metrics, and CI/CD integration patterns, see Chapter 14 of the PMCP Book.


### Practice Ideas

These informal exercises reinforce the concepts from this chapter.
