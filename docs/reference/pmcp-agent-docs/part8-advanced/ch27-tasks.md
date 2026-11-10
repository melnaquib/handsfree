---
chapter: 27
title: "MCP Tasks: Long-Running Operations"
part: "Part VIII: Advanced Patterns"
use_when: "Operations too slow for one request: TaskStore, TaskSupport, polling, get_task_result fallback."
suggested_prerequisites: [ch02]
source_kb: 48
extracted_kb: 26
code_blocks: 34
quality: auto-extracted
---

# Ch 27: MCP Tasks: Long-Running Operations

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

## Agent notes (checked against extracted text)

- `TaskStore` trait methods: `create`, `get`, `update_status`, `list`, `cancel`, `cleanup_expired`, `config`. Lookups are scoped to owner id.
- `InMemoryTaskStore` is dev/test only: tasks do not survive restart and are not shared across instances.
- Serverless needs a shared external store (DynamoDB, Redis, other DB).
- `.task_store(store)` on the builder auto-advertises the `tasks` capability. Do not set it by hand.
- `TaskSupport::Required` on a tool with no store is a build-time error.
- Polling (`tasks/get`) works on any transport and in serverless. SSE needs persistent streamable HTTP.
- Add a `get_task_result` fallback tool for clients that do not support tasks.

## Extracted content


### MCP Tasks: Long-Running Operations

Some operations take too long to fit inside a single request-response cycle.


### Why Tasks Matter for Enterprise MCP

Enterprise MCP deployments face a fundamental tension: many valuable operations are inherently slow, but the MCP protocol's default behavior assumes synchronous request-response.

(table/list, cells in reading order)
- The Polling Model vs SSE Notifications
- MCP defines two mechanisms for tracking long-running work:
- Mechanism
- Transport
- Serverless Compatible
- Client Complexity
- Polling ( tasks/get )
- SSE notifications
- Any (HTTP, stdio, WS)
- Yes
- Low -- just retry on interval
- Streamable HTTP only
- No -- requires persistent connection
- Higher -- must hold open connection

```text
   tools/call           >
                             create task           >
<   CreateTaskResult
                            (Lambda exits)
   tasks/get            >   (new Lambda)
                             read task             >
<   {status: working}
                            (Lambda exits)
   tasks/get            >   (new Lambda)
                             read task             >
<   {status: completed}
   tasks/result         >   (new Lambda)
                             read result           >
<   CallToolResult
```

The  TaskStore , backed by DynamoDB, Redis or another external store, provides the continuity that the stateless compute layer cannot. That only works if the store is shared: an in-memory store on one instance is invisible to the next.


### TaskStore: The External State Bridge

The  TaskStore  trait is the abstraction that makes tasks work across stateless invocations.

```rust
use pmcp::server::task_store::{InMemoryTaskStore, TaskStore, StoreConfig};
use std::sync::Arc;

// For development and testing: in-memory store
let store = Arc::new(InMemoryTaskStore::new());

// For production: your own `impl TaskStore` over a shared database
// let store = Arc::new(MyDynamoDbTaskStore::new(client, table_name));
```

It is the only  TaskStore  implementation in the SDK itself, and its tasks don't survive a restart or reach other instances. The required methods are  create ,  get , update_status ,  list ,  cancel ,  cleanup_expired  and  config , and every lookup is scoped to the owner id.

Recommended pattern (read the next section with this in mind). The SDK then mints the task id, auto-advertises the  tasks  capability, and serves  tasks/get | result | list | cancel  typed from the store -- you never hand-write  tasks/*  wire JSON.

(table/list, cells in reading order)
- Backend
- Plugs into
- Use Case
- InMemoryTaskStore  ( pmcp )

```text
task_store(...)
```

(table/list, cells in reading order)
- Development, testing, single- instance servers
- Backend
- Plugs into
- Use Case


### Your own  impl TaskStore  over another database

GenericTaskStore<DynamoDbBackend>  /  <RedisBackend>  via TaskRouterImpl  ( pmcp-tasks )


### legacy

```text
with_task_store(...)
```


### Existing servers on the router path

Owner isolation needs authentication.

This means you cannot accidentally corrupt task state regardless of how your handler logic is structured.


### get_task_result  fallback tool pattern

2.


### is needed for task negotiation

3.

What problem does the two-phase task model solve that synchronous tool calls cannot?


### Task Lifecycle and Polling

In this section, you will build a complete task-enabled tool from scratch. You will configure the server's TaskStore, declare task support on a tool, write a handler that branches between sync and async execution, and implement a fallback tool for clients that do not understand tasks.


### Setting Up TaskStore in the Server Builder

The builder's  .task_store()  method does two things: it records the backend, and it makes the server auto-advertise the  tasks capability so clients can discover task support during initialization. The capability follows the backend — you never set it by hand. (We will see in Capability Negotiation that a  TaskSupport::Required  tool with no store is a build-time error, precisely so the advertised capability is never hollow.)

```rust
use pmcp::server::builder::ServerCoreBuilder;
use pmcp::server::task_store::{InMemoryTaskStore, TaskStore};
use std::sync::Arc;

let store = Arc::new(InMemoryTaskStore::new()) as Arc<dyn TaskStore>;

let server = ServerCoreBuilder::new()
    .name("satellite-analysis")
    .version("1.0.0")
    .task_store(store.clone())
    // ... register a task tool with .with_task_support(...) ...
    .build()?;
```

```json
{
  "capabilities": {
    "tasks": {
      "list": {},
      "cancel": {},
      "requests": {
        "tools": {
          "call": {}
        }
      }
    }
  }
}
```

You should see the  tasks  field in the server capabilities.


### Declaring Task Support on Tools

Each tool declares its own task support level through the  execution  field. This tells clients whether they should, may, or must not use tasks with this specific tool.

```rust
use pmcp::types::{ToolExecution, TaskSupport};

let tool = TypedTool::new("analyze_imagery", |args: AnalyzeArgs, extra| {
    Box::pin(async move {
        // handler implementation (shown below)
        Ok(json!({}))
    })
})
.with_description("Analyze satellite imagery for terrain classification")
.with_execution(
    ToolExecution::new().with_task_support(TaskSupport::Optional)
);
```

(table/list, cells in reading order)
- The three support levels control client behavior:
- Level
- Wire Value
- Meaning
- When to Use

```rust
TaskSupport::Required
```

- "required"

- Client MUST send  task  parameter

- Operations that always exceed timeout limits

```rust
TaskSupport::Optional
```

```rust
TaskSupport::Forbidden
```

- Client MUST NOT send  task parameter


### The  execution  field appears in the  tools/list  response:

```json
{
  "name": "analyze_imagery",
  "description": "Analyze satellite imagery for terrain classification",
  "inputSchema": { "type": "object", "properties": { ... } },
  "execution": {
    "taskSupport": "optional"
  }
}
```


### Recommended Pattern: Let the SDK Own the Wire Protocol

The cleanest way to expose a tool as a Task is to declare  with_task_support  and register a  TaskStore  — then let the SDK mint the task id, advertise the capability, and serialize every  tasks/*  response from typed structs.

```rust
use pmcp::server::typed_tool::TypedTool;
use pmcp::types::{ToolExecution, TaskSupport};
use serde_json::json;

let task_tool = TypedTool::new_with_schema(
    "analyze_imagery",
    json!({ "type": "object", "properties": { "image_uri": { "type": "string" } } }),
    |_args, _extra| Box::pin(async {
        // Do the work; return a CallToolResult-shaped value.
        Ok(json!({ "content": [{ "type": "text", "text": "terrain classified" }] }))
    }),
)
.with_description("Analyze satellite imagery for terrain classification")
.with_execution(ToolExecution::new().with_task_support(TaskSupport::Required));
```

```rust
use pmcp::ToolCallResponse;

let task_id = match client
    .call_tool_with_task("analyze_imagery".to_string(), serde_json::json!({ "image_uri": "s3://..." }))
    .await?
{
    ToolCallResponse::Task(task) => task.task_id, // store-minted id
    ToolCallResponse::Result(_) => unreachable!("Required task tool always creates a task"),
};

let mut task = client.tasks_get(&task_id).await?;
while !task.status.is_terminal() {
    tokio::time::sleep(std::time::Duration::from_millis(task.poll_interval.unwrap_or(2000))).await;
    task = client.tasks_get(&task_id).await?;
}
let result = client.tasks_result(&task_id).await?; // typed CallToolResult
```

You never assemble  tasks/*  wire JSON by hand, so the server and client agree by construction — eliminating an entire class of silent wire-shape bugs (mismatched ids, malformed  tasks/result  payloads).


### Advanced: Hand-Rolled Dual-Path Handler (is_task_request)

The pattern below predates the recommended path above. Study it to understand what the SDK now does for you — and prefer the recommended pattern in new code. It remains useful when you need full manual control over background scheduling (for example, enqueueing to SQS instead of

```rust
tokio::spawn ).
```


### Here is the full pattern, using a satellite imagery analysis scenario:

```rust
use pmcp::prelude::*;
use pmcp::server::task_store::TaskStore;
use pmcp::types::tasks::{Task, TaskStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
struct AnalyzeArgs {
    /// S3 URI of the satellite image to analyze
    image_uri: String,
    /// Classification model to use
    #[serde(default = "default_model")]
    model: String,
}

fn default_model() -> String {
    "terrain-v3".to_string()
}

fn build_analyze_tool(
    store: Arc<dyn TaskStore>,
) -> TypedTool<AnalyzeArgs, impl Fn(AnalyzeArgs, RequestHandlerExtra) -> Pin<Box<dyn Future<Output = Result<Value>> +
Send>> + Send + Sync> {
    let store_clone = store.clone();

    TypedTool::new("analyze_imagery", move |args: AnalyzeArgs, extra| {
        let store = store_clone.clone();
        Box::pin(async move {
            if extra.is_task_request() {
```

```text
//    TASK PATH
```

```rust
                // Create a task record, spawn background work, return immediately
                let owner = extra.auth_context()
                    .map(|ctx| ctx.subject.clone())
                    .unwrap_or_else(|| "anonymous".to_string());

                let task = store.create(&owner, Some(300_000)).await?; // 5 min TTL

                // Spawn the actual analysis in the background
                let task_id = task.task_id.clone();

                let store_bg = store.clone();
                tokio::spawn(async move {
                    // Simulate long-running analysis
                    let result = run_analysis(&args.image_uri, &args.model).await;
                    match result {
                        Ok(_) => {
                            let _ = store_bg.update_status(
                                &task_id, &owner,
                                TaskStatus::Completed,
                                Some("Analysis complete".into()),
                            ).await;
                        }
                        Err(e) => {
                            let _ = store_bg.update_status(
                                &task_id, &owner,
                                TaskStatus::Failed,
                                Some(format!("Analysis failed: {e}")),
                            ).await;
                        }
                    }
                });

                // Return CreateTaskResult immediately
                Ok(json!({
                    "task": {
                        "taskId": task.task_id,
                        "status": "working",
                        "createdAt": task.created_at,
                        "lastUpdatedAt": task.last_updated_at,
                        "pollInterval": task.poll_interval,
                        "ttl": task.ttl
                    }
                }))
            } else {
```

```text
//    SYNC PATH
```

```rust
                // Run the analysis inline and return the result directly
                let result = run_analysis(&args.image_uri, &args.model).await
                    .map_err(|e| Error::internal(format!("Analysis failed: {e}")))?;

                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result)?
                    }]
                }))
            }
        })
    })
    .with_description("Analyze satellite imagery for terrain classification")
    .with_execution(
        ToolExecution::new().with_task_support(TaskSupport::Optional)
    )
}
```


### Let's walk through the key decisions:

1.

aware clients get it faster (immediate acknowledgment instead of waiting).


### The Task Status State Machine

The  TaskStore  enforces valid status transitions.

```rust
                +--------------+
                |   working    |  <-- initial state from store.create()
                +------+-------+
                       |
          +------------+------------+
          v            |            v
  +----------------+   |   +-----------------+
  | input_required |---+   |    terminal     |
  +----------------+       | +-------------+ |
          |                | | completed   | |
          +--------------->  | failed      | |
                           | | cancelled   | |
                           | +-------------+ |
                           +-----------------+
```


### Valid transitions:

working  ->  input_required ,  completed ,  failed ,  cancelled input_required  ->  working ,  completed ,  failed ,  cancelled Terminal states ( completed ,  failed ,  cancelled ) -> nothing (any attempt returns  InvalidTransition )


### The get_task_result Fallback Tool Pattern

Not every MCP client understands tasks.

```rust
use pmcp::prelude::*;
use pmcp::server::task_store::TaskStore;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
struct GetTaskResultArgs {
    /// The task ID returned by the original tool call
    task_id: String,
}

fn build_get_task_result_tool(
    store: Arc<dyn TaskStore>,
) -> TypedTool<GetTaskResultArgs, impl Fn(GetTaskResultArgs, RequestHandlerExtra) -> Pin<Box<dyn Future<Output =
Result<Value>> + Send>> + Send + Sync> {
    let store_clone = store.clone();

    TypedTool::new("get_task_result", move |args: GetTaskResultArgs, extra| {
        let store = store_clone.clone();
        Box::pin(async move {
            let owner = extra.auth_context()
                .map(|ctx| ctx.subject.clone())
                .unwrap_or_else(|| "anonymous".to_string());

            let task = store.get(&args.task_id, &owner).await
                .map_err(|e| Error::not_found(format!("Task not found: {e}")))?;

            match task.status {
                TaskStatus::Completed => {
                    // Return the stored result
                    // (In practice, you would read from a result store)
                    Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "Task {} completed: {}",
                                task.task_id,
                                task.status_message.unwrap_or_default()
                            )
                        }]

                    }))
                }
                TaskStatus::Failed => {
                    Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "Task {} failed: {}",
                                task.task_id,
                                task.status_message.unwrap_or_else(|| "Unknown error".into())
                            )
                        }],
                        "isError": true
                    }))
                }
                _ => {
                    // Still running -- tell the client to wait
                    Ok(json!({
                        "content": [{
                            "type": "text",
// ... truncated, 17 more lines in source book ...
```

When to use this pattern: Always include  get_task_result  if any of your tools use  TaskSupport::Optional  or

```rust
TaskSupport::Required . It costs nothing to register and provides a universal fallback.
```


### Putting It All Together

Here is how the pieces connect in a complete server.

```rust
use pmcp::server::builder::ServerCoreBuilder;
use pmcp::server::task_store::{InMemoryTaskStore, TaskStore};
use std::sync::Arc;

let store = Arc::new(InMemoryTaskStore::new()) as Arc<dyn TaskStore>;

let server = ServerCoreBuilder::new()
    .name("satellite-analysis")
    .version("1.0.0")
    .task_store(store.clone())
    .tool("analyze_imagery", build_analyze_tool(store.clone()))
    .tool("get_task_result", build_get_task_result_tool(store.clone()))
    .build()?;
```

The  get_task_result  fallback tool below remains useful for clients that cannot speak the native  tasks/*  methods at all (it is a plain  Forbidden  tool that reads the store).


### Key Takeaways

Recommended pattern: register a  with_task_support  tool +  task_store(...)  on  ServerCoreBuilder , and let the SDK mint the id, advertise the capability, and serve  tasks/*  typed. You never hand-write  tasks/*  wire JSON. (Client side:  call_tool_with_task → poll  tasks_get  →  tasks_result .) .task_store()  on  ServerCoreBuilder  auto-advertises the standard  tasks  capability -- you do not set it manually.

```rust
.with_execution(ToolExecution::new().with_task_support(...))  on the tool declares per-tool task behavior
```


### Capability Negotiation

Task support in MCP is negotiated per-request, not per-session.


### The Principle: Capability Inspection, Not Client Sniffing

Servers never need to know which client is calling them. They do not check User-Agent headers. They do not maintain a session table mapping client IDs to feature sets.


### The signal is the  task  field in the  tools/call  request parameters:

```text
// Task-aware client request:
{
  "method": "tools/call",
  "params": {
    "name": "analyze_imagery",
    "arguments": { "image_uri": "s3://bucket/image.tif" },
    "task": { "ttl": 300000 }
  }
}

// Non-task-aware client request:
{
  "method": "tools/call",
  "params": {
    "name": "analyze_imagery",
    "arguments": { "image_uri": "s3://bucket/image.tif" }
  }
}
```

The only difference is the presence or absence of  "task" .

```rust
async fn handle(args: AnalyzeArgs, extra: RequestHandlerExtra) -> Result<Value> {
    if extra.is_task_request() {
        // Client understands tasks -- return CreateTaskResult
        create_task_and_return(&args, &extra).await
    } else {
        // Client does not understand tasks -- return sync result
        run_and_return_inline(&args).await
    }
}
```


### The Other Half: The Server's Capability Follows Its Backend

Per-request negotiation describes the client's signal. The server makes a matching declaration during  initialize , and the SDK derives it for you: the  tasks  capability is advertised only when a real backend exists to serve the  tasks/*  endpoints — i.e. when you registered a  TaskStore  via  task_store(...) . This is the same correctness principle as the client side: the advertised capability is never a hollow claim. The endpoints it promises are always backed.


### Two consequences fall directly out of this rule:

1. You never set it by hand.

A  TaskSupport::Required  tool with no backend is a build-time error.


### than advertising  tasks/*  endpoints that cannot work:

```rust
// No task_store() — build() fails because a Required tool has nothing to back it.
let result = ServerCoreBuilder::new()
    .name("svc").version("1.0.0")
    .tool("analyze_imagery", required_task_tool) // with_task_support(Required)
    .build();
```

assert!(result.is_err()); // "a tool declares TaskSupport::Required but no TaskStore ... is configured"

Neither side can claim support it cannot honor.

```rust
  |                               |                              |
  |-- tools/call { task: {} } --> |                              |
  |                               |-- parse CallToolRequest -->  |
  |                               |   req.task = Some({})        |
  |                               |                              |
  |                               |-- build RequestHandlerExtra  |
  |                               |   .with_task_request(        |
  |                               |       req.task)              |
  |                               |                              |
  |                               |-- call handler(args, extra)->|
  |                               |                              |
  |                               |         extra.is_task_request()
  |                               |         == true              |
  |                               |                              |
  |<-- CreateTaskResult ----------|<-- json!({task: ...}) -------|
```

Your handler never touches raw JSON -- it calls  extra.is_task_request()  and gets a boolean.


### Why No Session State Is Needed

In many protocol designs, capabilities are negotiated once during session initialization and stored server-side.


### Session-based negotiation (NOT how MCP Tasks work):

```rust
  Client                    Server
    |-- initialize -------->|
    |   { supportsTasks }   |-- store in session map --|
    |<-- capabilities ------|                          |
    |                       |                          |
    |-- tools/call -------->|-- lookup session ------->|
    |                       |   "does this client      |
    |                       |    support tasks?"       |
    |                       |<-- yes ------------------|
    |<-- CreateTaskResult --|
```


### Per-request negotiation (how MCP Tasks actually work):

```rust
  Client                    Server
    |-- tools/call -------->|
    |   { task: {} }        |-- req.task.is_some()? -->|
    |                       |   yes                    |
    |<-- CreateTaskResult --|
```


### Per-request negotiation is better for stateless servers because:

1. No session store required. Lambda functions do not share memory between invocations.

requires an external database. Per-request signals avoid this entirely.


### The Three Client Profiles

Your server will encounter three types of clients.


### Profile 1: Task-Native Client

Understands the MCP Tasks specification.


### Client                    Server                    TaskStore

```rust
  |                         |                          |
  |-- tools/call            |                          |
  |   { task: {ttl: 300k} } |                          |
  |                         |-- store.create() ------->|
  |<-- CreateTaskResult ----|                          |
  |                         |                          |
  |-- tasks/get {taskId} -->|-- store.get() ---------->|
  |<-- {status: working} ---|                          |
  |                         |                          |
  |-- tasks/get {taskId} -->|-- store.get() ---------->|
  |<-- {status: completed} -|                          |
  |                         |                          |
  |-- tasks/result -------->|-- (return stored result) |
  |<-- CallToolResult ------|                          |
```

What the handler does: Takes the task path ( extra.is_task_request() == true ), creates a task, spawns background work, returns CreateTaskResult .


### Profile 2: Tool-Polling Client

Does not understand MCP Tasks, but is smart enough to call  get_task_result  when told to.


### Client                    Server

```text
  |                         |
  |-- tools/call            |
  |   (no task field)       |
  |                         |-- sync path: run inline
  |<-- CallToolResult ------|  (or timeout with task hint)
  |                         |
  |-- tools/call            |
  |   get_task_result       |
  |   {task_id: "t-abc"}    |
  |                         |
  |<-- "still working,      |
  |     try again in 5s" ---|
  |                         |
  |-- tools/call            |
  |   get_task_result       |
  |   {task_id: "t-abc"}    |
  |                         |
  |<-- result --------------|
```

What the handler does: Takes the sync path ( extra.is_task_request() == false ). If it cannot, the  get_task_result  tool provides a manual polling path.


### Profile 3: Sync-Only Client

No task support.


### Client                    Server

```text
  |                         |
  |-- tools/call            |
  |   (no task field)       |
  |                         |-- sync path: run inline
  |<-- CallToolResult ------|
```

What the handler does: Takes the sync path.

```rust
TaskSupport::Required , this client cannot use the tool at all -- and that is the correct behavior, because the operation genuinely cannot
```


### Comparison with HTTP Content Negotiation

If this pattern feels familiar, it should.


### HTTP Content Negotiation:

```rust
  Accept: application/json        -->  Server returns JSON
  Accept: text/html               -->  Server returns HTML
  Accept: */*                     -->  Server returns default format
```


### MCP Task Negotiation:

```rust
  task: { ttl: 300000 }           -->  Server returns CreateTaskResult
  (no task field)                  -->  Server returns CallToolResult
```

Both patterns let the server adapt its response format based on what the client declares it can handle. Neither requires the server to remember what format a particular client prefers across requests.

(table/list, cells in reading order)
- The analogy extends further:
- HTTP
- MCP Tasks
- Accept: application/json
- task: { ttl: 300000 }
- Content-Type: application/json
- Response is  CreateTaskResult
- 406 Not Acceptable
- Error when  TaskSupport::Required  but no  task  sent

When the operation might exceed the Lambda timeout,  Required  is the honest declaration. The server cannot guarantee synchronous completion, so it should not pretend it can.


### if extra.is_task_request() {

```rust
    // Always use task path if client requests it
    create_task_and_spawn(&args, &extra, &store).await
} else if estimate_duration(&args) > Duration::from_secs(25) {
    // Sync path, but operation is too slow -- return helpful error
    Err(Error::validation(
        "This analysis is estimated to take >25 seconds. \
         Use task mode or call get_task_result after the operation."
    ))
} else {
    // Sync path, operation should complete in time
    run_inline(&args).await
}
```


### Key Takeaways

Task negotiation is per-request, not per-session -- the  task  field in the request is the only signal The server's  tasks  capability follows the backend: registering a  TaskStore  auto-advertises it, and a  TaskSupport::Required  tool with no store is a build-time error (no hollow capability) extr
