---
chapter: 6
title: "Resources, Prompts, and Workflows"
part: "Part II: Thoughtful Design"
use_when: "Choosing resources vs tools vs prompts; soft/hybrid/hard workflows; resource templates and caching."
suggested_prerequisites: [ch04]
source_kb: 69
extracted_kb: 46
code_blocks: 64
quality: auto-extracted
---

# Ch 6: Resources, Prompts, and Workflows

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Resources, Prompts, and Workflows

Tools get most of the attention in MCP discussions, but they're only one-third of the picture.


### The Control Problem

Recall from Chapter 4: you don't control the AI client's decisions.

How do you build reliable workflows when you can't control execution?

(table/list, cells in reading order)
- Primitive
- Purpose
- Who Controls
- Tools
- Actions the AI can take
- AI decides when/how to use
- Resources
- Documents the AI can read
- AI decides what to read
- Prompts
- Workflows the user can invoke
- User explicitly selects


### Resources: Stable Data for Context

Resources are addressable data that the AI can read.

Has a stable identity (URI) Doesn't require computation to retrieve Provides context for tool usage Shouldn't trigger actions just by being read

```rust
// Database schema - stable reference data
Resource::new("sales://schema/customers")
    .name("Customer Table Schema")
    .description("Column definitions, types, and constraints for the customers table")
    .mime_type("application/json")

// Configuration - current settings
Resource::new("sales://config/regions")
    .name("Sales Regions")
    .description("Active sales regions with territory mappings")
    .mime_type("application/json")

// Templates - reusable patterns
Resource::new("sales://templates/reports")
    .name("Report Templates")
    .description("Available report formats and their parameters")
    .mime_type("application/json")
```


### Resources vs Tools

A common mistake is implementing read operations as tools when they should be resources:

```rust
// WRONG: Read-only data as a tool
Tool::new("get_schema")
    .description("Get the database schema")
// This implies an action, but it's just reading data

// RIGHT: Read-only data as a resource
Resource::new("db://schema")
    .description("Database schema with all tables and columns")
// Clear that this is stable, readable data
```


### The AI treats resources differently than tools:

Resources can be read proactively for context Resources don't count as "actions taken" Resources are cached by many clients

```rust
Resource::new("sales://customers/{customer_id}")
    .name("Customer Details")
    .description("Detailed information for a specific customer")

Resource::new("sales://reports/{year}/{quarter}")
    .name("Quarterly Report")
    .description("Sales report for a specific quarter")
```


### Prompts: User Control and Workflow Execution

Prompts are the most underutilized MCP primitive—and potentially the most powerful for complex workflows.

```text
                   Claude Desktop
User types: /quarterly-analysis
   Prompt: quarterly-analysis
   Server executes workflow steps
   and returns results to the AI
The AI receives pre-executed context
```

Instead of hoping the AI takes the right approach, users explicitly select a workflow.


### The Workflow Spectrum: Soft → Hybrid → Hard

PMCP provides a spectrum of workflow execution models.

Do as much as possible on the server side, and allow the AI to complete the workflow if you can't complete it on the server side.

```text
                   Workflow Execution Spectrum
SOFT WORKFLOWS              HYBRID WORKFLOWS          HARD WORKFLOWS
Text guidance               Server executes some      Server executes
for AI to follow            AI completes the rest     everything
  "Follow                     Server:                  Server:
   these                        Step 1                   Step 1 [ok]
   steps:                       Step 2                   Step 2 [ok]
   1. ...                     AI:                        Step 3 [ok]
   2. ...                       Step 3                   Step 4 [ok]
   3. ..."                      Step 4                 Return:
                                                        Complete
                                                        results
Use when:                   Use when:                 Use when:
- Complex reasoning         - Some steps need         - All steps are
  required                    LLM judgment              deterministic
- Context-dependent         - Fuzzy matching          - No reasoning
  decisions                 - User clarification        needed
- Dynamic exploration         may be needed           - Single result
Examples:                   Examples:                 Examples:
- Open-ended analysis       - "Add task to project"   - Data pipelines
- Creative tasks              (fuzzy project name)    - Report gen
- Multi-domain queries      - Search + refine         - CRUD workflows
◄    Less deterministic           More deterministic    ►
◄    More AI reasoning            Less AI reasoning    ►
◄    Multiple round-trips         Single round-trip    ►
```

Why Prefer Hard Workflows?

(table/list, cells in reading order)
- Hard workflows (server-side execution) provide significant advantages:
- Aspect
- Soft Workflow
- Hard Workflow
- Round-trips
- 1 per tool call
- 1 total
- Execution order
- AI decides (unpredictable)
- Server enforces (deterministic)
- Data binding
- AI must remember
- Server manages automatically
- Error handling
- AI interprets
- Server controls
- Testing
- Latency
- Requires AI
- Pure function tests
- High (multiple LLM calls)
- Low (single execution)

Best practice: Start with hard workflows. Fall back to hybrid or soft only when LLM reasoning is genuinely required.


### VS Code / Cursor:

Prompts appear in command palette Can be bound to keyboard shortcuts Context-aware prompt suggestions

(table/list, cells in reading order)
- PMCP SDK: Workflow Types
- The PMCP SDK provides two approaches to prompts:
- 1. Text Prompts (Soft Workflows)
- For guidance-based workflows where AI follows instructions:

```rust
use pmcp::server::PromptHandler;

Prompt::new("data-exploration")
    .description("Interactive data exploration session")
    .messages(vec![
        PromptMessage::user(
            "Start an interactive data exploration session:\n\n\
            **Initial Setup:**\n\
            1. Read available schemas\n\
            2. List tables and their row counts\n\
            3. Present a summary of available data\n\n\
            **Then wait for my questions...**"
        )
    ])
```

```rust
use pmcp::server::workflow::{SequentialWorkflow, WorkflowStep, ToolHandle};
use pmcp::server::workflow::dsl::*;

let workflow = SequentialWorkflow::new(
    "quarterly_report",
    "Generate quarterly sales report with analysis"
)
.argument("quarter", "Quarter: Q1, Q2, Q3, Q4", true)
.argument("year", "Year (default: current)", false)

// Step 1: Fetch sales data (server executes)
.step(
    WorkflowStep::new("fetch_sales", ToolHandle::new("sales_query"))
        .arg("quarter", prompt_arg("quarter"))
        .arg("year", prompt_arg("year"))
        .bind("sales_data")  // Output bound for next step
)

// Step 2: Calculate metrics (server executes)
.step(
    WorkflowStep::new("calc_metrics", ToolHandle::new("calculate_metrics"))
        .arg("data", from_step("sales_data"))  // Use previous output
        .bind("metrics")
)

// Step 3: Generate report (server executes)
.step(
    WorkflowStep::new("generate_report", ToolHandle::new("format_report"))
        .arg("sales", from_step("sales_data"))
        .arg("metrics", from_step("metrics"))
        .arg("format", constant(json!("markdown")))
        .bind("report")
);

// Register with server
let server = Server::builder()
    .name("sales-server")
    .version("1.0.0")

    .prompt_workflow(workflow)?
    .build()?;
```


### When a user invokes  /quarterly_report Q3 2024 :

1.

```rust
// RESOURCES: Stable reference data
Resource::new("sales://schema")
Resource::new("sales://regions")
Resource::new("sales://products")

// TOOLS: Actions for direct use and workflow steps
Tool::new("sales_query")       // Query data
Tool::new("sales_aggregate")   // Calculate summaries
Tool::new("sales_export")      // Export results

// PROMPTS: User-controlled workflows

// Soft workflow for exploration
Prompt::new("data-exploration")
    .messages(vec![...])

// Hard workflow for reports
SequentialWorkflow::new("quarterly-analysis")
    .step(WorkflowStep::new(...))
    .step(WorkflowStep::new(...))
```


### A user invoking  /quarterly-analysis :

1.


### Without the workflow, the AI might:

Query random tables Miss the year-over-year comparison Forget to check all regions Present data in an inconsistent format Require 6+ round-trips for 3-step workflow

(table/list, cells in reading order)
- Summary
- Primitive
- Design Question
- User Experience
- Tools
- "What actions should be possible?"
- AI uses as needed
- Resources
- "What context should be available?"
- AI reads for understanding
- Prompts
- "What workflows should users control?"
- User explicitly invokes

Use hard workflows by default, falling back to hybrid or soft workflows only when genuine LLM reasoning is required.

(table/list, cells in reading order)
- Workflow Type
- When to Use
- Hard
- Hybrid
- Soft
- All steps are deterministic, no reasoning needed
- Some steps need LLM judgment (fuzzy matching, clarification)
- Complex reasoning, exploration, creative tasks


### When to Use Resources vs Tools

Resources and tools both provide data to AI clients, but they serve fundamentally different purposes. Understanding when to use each leads to cleaner designs and better AI behavior, and in building domain specific MCP servers.

(table/list, cells in reading order)
- The Key Distinction
- Aspect
- Resources
- Tools
- Purpose
- Identity
- Provide stable data
- Perform actions
- Addressable by URI
- Invoked by name
- Side effects
- None
- May have side effects
- Caching
- Often cached by clients
- Not cached
- AI perception
- Context/reference data
- Operations to perform
- Think of it this way:

```text
Does the operation have side effects?
  YES   ► Use a TOOL
          - Database modifications
          - External API calls that mutate
          - Sending notifications
          - Creating files
  NO   ► Does the data have a stable identity?
            YES   ► Use a RESOURCE
                    - Schema definitions
                    - Configuration
                    - Reference data
                    - Static documentation
            NO   ► Does it require computation?
                      YES   ► Use a TOOL
                              - Complex queries
                              - Aggregations
                              - Reports
                      NO   ► Use a RESOURCE
                              - Simple lookups
                              - Cached data
```

```rust
// Database schema - AI reads to understand what queries are valid
Resource::new("db://schema/customers")
    .name("Customers Table Schema")
    .description("Column names, types, and relationships for customers table")
    .mime_type("application/json")

// API schema - AI reads to construct valid requests
Resource::new("api://openapi/v1")
    .name("API Specification")
    .description("OpenAPI specification for the REST API")
    .mime_type("application/json")
```

```rust
// Feature flags - AI reads to know what's enabled
Resource::new("config://features")
    .name("Feature Flags")
    .description("Currently enabled features and experiments")

// Limits and quotas - AI reads to stay within bounds
Resource::new("config://limits")
    .name("Service Limits")
    .description("Rate limits, quotas, and maximum values")
```

```rust
// Region codes - AI reads when constructing queries
Resource::new("reference://regions")
    .name("Sales Regions")
    .description("Region codes, names, and territories")

// Product catalog - AI reads for lookups
Resource::new("reference://products")
    .name("Product Catalog")
    .description("Product IDs, names, categories, and attributes")
```

```rust
// Query syntax help
Resource::new("docs://sql-guide")
    .name("SQL Query Guide")
    .description("Supported SQL syntax with examples")

// Best practices
Resource::new("docs://best-practices")
    .name("API Best Practices")
    .description("Recommended patterns for using this API")
```

```rust
// Query tool - parameters determine what's returned
Tool::new("sales_query")
    .description("Query sales data with filters")
    .input_schema(json!({
        "properties": {
            "date_range": { ... },
            "region": { ... },
            "product_category": { ... }
        }
    }))
```

```rust
// Create operations
Tool::new("order_create")
    .description("Create a new order")

// Update operations
Tool::new("customer_update")
    .description("Update customer information")

// Delete operations
Tool::new("record_delete")
    .description("Delete a record")
```

```rust
// Third-party integrations
Tool::new("send_email")
    .description("Send email via SendGrid")

// Payment processing
Tool::new("process_payment")
    .description("Process payment via Stripe")
```

```rust
// Aggregation
Tool::new("sales_report")
    .description("Generate sales report with totals and averages")

// Analysis
Tool::new("trend_analysis")
    .description("Analyze trends in historical data")
```

```rust
// WRONG: This is just reading data
Tool::new("get_schema")
    .description("Get the database schema")

// RIGHT: Stable data should be a resource
Resource::new("db://schema")
    .description("Database schema")
```


### Mistake 2: Dynamic Data as Resources

```rust
// WRONG: This data changes based on parameters
Resource::new("sales://today")
    .description("Today's sales data")
// What if user needs yesterday's data?

// RIGHT: Parameterized queries should be tools
Tool::new("sales_query")
    .description("Query sales data for a date range")
    .input_schema(json!({
        "properties": {
            "date": { "type": "string", "format": "date" }
        }
    }))
```


### Mistake 3: Actions as Resources

```rust
// WRONG: Has side effects
Resource::new("notifications://send")
    .description("Send a notification")

// RIGHT: Side effects require tools
Tool::new("send_notification")
    .description("Send a notification to a user")
```

```rust
// Resource: schema for understanding
Resource::new("db://schema/orders")
    .description("Order table structure")

// Tool: query for action
Tool::new("order_query")
    .description("Query orders. See db://schema/orders for available columns.")
```

The AI reads the resource to understand the schema, then uses the tool to query.


### Resource Templates for Entities

```rust
// Template resource for specific entities
Resource::new("customers://{customer_id}")
    .name("Customer Details")
    .description("Read-only view of a specific customer")

// Tool for modifications
Tool::new("customer_update")
    .description("Update customer fields")
```

Reading customer details is a resource; modifying them is a tool.


### Cached Resources for Performance

```rust
// Expensive computation cached as resource
Resource::new("analytics://daily-summary")
    .name("Daily Summary")
    .description("Pre-computed daily analytics (updated hourly)")

// Real-time query as tool
Tool::new("analytics_query")
    .description("Real-time analytics query (slower, but up-to-date)")
```

AI may read proactively to gather context Clients often cache resource contents AI doesn't count resource reads as "actions" Multiple reads don't concern the AI


### Tools

AI calls tools deliberately to accomplish goals Each call is an "action" the AI considers AI may hesitate to call tools repeatedly Tool calls may require user confirmation


### Design with these behaviors in mind:

Put context-setting data in resources (AI reads freely) Put consequential operations in tools (AI considers carefully)

(table/list, cells in reading order)
- Summary
- Use Resources For
- Use Tools For
- Schemas and structure
- Parameterized queries
- Configuration
- Reference data
- Documentation
- Write operations
- External integrations
- Computed results
- Stable, addressable data
- Actions with side effects
- Use Resources For
- Use Tools For
- Context AI reads proactively
- Operations AI performs deliberately


### Soft Workflows: Text Prompts for AI Guidance

When hard workflows aren't possible—when steps require LLM reasoning, context-dependent decisions, or creative interpretation—text prompts provide structured guidance for AI execution.


### When to Use Soft Workflows

Remember the guiding principle: Do as much as possible on the server side. Use soft workflows only when:

(table/list, cells in reading order)
- Scenario
- Why Soft Workflow
- Complex reasoning required
- AI must interpret, analyze, or synthesize
- Context-dependent decisions
- Right choice depends on conversation history
- Dynamic exploration
- AI discovers what to do based on findings
- Creative or open-ended tasks
- Multiple valid approaches exist
- Multi-domain queries
- AI must coordinate across many servers


### The Soft Workflow Tradeoff

```text
                  Soft Workflow Execution
Client                          Server
        prompts/get              ►
   ◄    text guidance
     AI reads guidance...
     AI decides to call tool 1
        tools/call (tool 1)      ►
   ◄    result 1
     AI processes result...
     AI decides to call tool 2
        tools/call (tool 2)      ►
   ◄    result 2
     ... more round trips ...
     AI synthesizes final answer
Total: 1 + N round trips (where N = number of tool calls)
```

Trade-off: More flexibility, but more latency and less predictable execution.

```rust
Prompt::new("database-audit")
    .description("Comprehensive database security audit")
    .messages(vec![
        PromptMessage::user(
            "Perform a security audit of the database:\n\n\
            **Step 1: Schema Analysis**\n\
            - Read db://schema to understand table structure\n\
            - Identify tables containing PII or sensitive data\n\n\
            **Step 2: Access Review**\n\
            - List all users with write permissions\n\
            - Flag any overly broad permission grants\n\n\
            **Step 3: Data Exposure Check**\n\
            - Check for unencrypted sensitive columns\n\
            - Verify no credentials stored in plain text\n\n\
            **Step 4: Report**\n\
            - Summarize findings with severity ratings\n\
            - Provide specific remediation recommendations\n\n\
            Begin with Step 1."
        )
    ])
```

- Don't leave the AI guessing which tools to use:

```rust
Prompt::new("customer-360-view")
    .messages(vec![
        PromptMessage::user(
            "Create a 360-degree view of customer {{customer_id}}:\n\n\
            1. **Profile**: Read resource `customers://{{customer_id}}/profile`\n\
            2. **Orders**: Use `sales_query` to get order history\n\
            3. **Support**: Use `tickets_query` to get support interactions\n\
            4. **Payments**: Use `billing_query` to get payment history\n\n\
            Synthesize into a comprehensive customer summary."
        )
    ])
```

- Specify how results should be presented:

```rust
Prompt::new("weekly-metrics-report")
    .messages(vec![
        PromptMessage::user(
            "Generate the weekly metrics report:\n\n\
            # Data to Gather\n\
            - Revenue by region (use sales_aggregate)\n\
            - New customers (use customers_query)\n\
            - Support tickets (use tickets_summary)\n\n\
            # Output Format\n\
            ```\n\
            # Revenue\n\
            | Region | This Week | Last Week | Change |\n\
            |--------|-----------|-----------|--------|\n\
            | ...    | ...       | ...       | ...    |\n\n\
            # Key Insights\n\
            1. [Insight 1]\n\
            2. [Insight 2]\n\
            ```"
        )
    ])
```

```rust
Prompt::new("data-modification")
    .description("Safely modify production data with review steps")
    .messages(vec![
        PromptMessage::user(
            "Help me modify data in {{table}}:\n\n\
            **Safety Protocol:**\n\
            1. First, show me the current state of affected records\n\
            2. Explain exactly what changes will be made\n\
            3. Ask for my explicit confirmation before proceeding\n\
            4. After modification, show the before/after comparison\n\n\
            **Constraints:**\n\
            - Maximum 100 records per operation\n\
            - No DELETE operations without WHERE clause\n\
            - All changes must be logged\n\n\
            What modification do you need?"
        )
    ])
```

```rust
Prompt::new("sales-analysis-mode")
    .description("Enter sales analysis mode with full context")
    .messages(vec![
        PromptMessage::user(
            "I'm going to analyze sales data. Before I ask my questions:\n\n\
            1. Read the sales://schema resource\n\
            2. Read the sales://config/regions resource\n\
            3. Summarize what data is available and any recent changes\n\n\
            Then wait for my analysis questions."
        )
    ])
```

When to use: User will ask multiple follow-up questions; context needs to be established first.

```rust
Prompt::new("data-exploration")
    .description("Interactive data exploration session")
    .messages(vec![
        PromptMessage::user(
            "Start an interactive data exploration session:\n\n\
            **Initial Setup:**\n\
            1. Read available schemas\n\
            2. List tables and their row counts\n\
            3. Present a summary of available data\n\n\
            **Then wait for my questions. For each question:**\n\
            - If I ask about data: query and visualize\n\
            - If I ask about relationships: show joins and keys\n\
            - If I ask for export: use safe_export with confirmation\n\n\
            **Session rules:**\n\
            - Keep queries under 10,000 rows\n\
            - Warn before expensive operations\n\
            - Maintain context across questions\n\n\
            Begin setup."
        )
    ])
```

When to use: Open-ended exploration where the path isn't known in advance.

```rust
Prompt::new("investigate-anomaly")
    .arguments(vec![
        PromptArgument::new("severity")
            .description("Alert severity: low, medium, high, critical"),
        PromptArgument::new("metric")
            .description("The metric that triggered the alert"),
    ])
    .messages(vec![
        PromptMessage::user(
            "Investigate the {{severity}} severity anomaly in {{metric}}:\n\n\
            {{#if severity == 'critical'}}\n\
            **CRITICAL ALERT PROTOCOL:**\n\
            1. Immediately gather last 24 hours of data\n\
            2. Compare against last 7 days baseline\n\
            3. Identify correlated metrics\n\
            4. Check for system events at anomaly time\n\
            5. Prepare incident summary for escalation\n\
            {{else if severity == 'high'}}\n\
            **HIGH ALERT INVESTIGATION:**\n\
            1. Gather last 48 hours of data\n\
            2. Identify pattern or one-time spike\n\
            3. Check for known causes\n\
            4. Recommend monitoring or action\n\
            {{else}}\n\
            **STANDARD INVESTIGATION:**\n\
            1. Review metric trend for past week\n\
            2. Note if this is recurring\n\
            3. Log finding for pattern analysis\n\
            {{/if}}"
        )
    ])
```

When to use: Response should vary based on parameters; complex conditional logic.

```rust
// First prompt: Discovery
Prompt::new("discover-opportunities")
    .description("Find potential opportunities in sales data")
    .messages(vec![
        PromptMessage::user(
            "Analyze sales data to identify opportunities:\n\n\
            1. Find underperforming products in growing categories\n\
            2. Identify customers with declining purchase frequency\n\
            3. Spot regions with untapped potential\n\n\
            List findings with IDs for follow-up analysis.\n\
            User can then run /deep-dive on any finding."
        )
    ])

// Second prompt: Deep dive
Prompt::new("deep-dive")
    .arguments(vec![
        PromptArgument::new("finding_id")
            .description("ID from discover-opportunities output"),
    ])
    .description("Deep dive into a specific opportunity")
    .messages(vec![
        PromptMessage::user(
            "Perform detailed analysis on finding {{finding_id}}:\n\n\
            1. Gather all related data\n\
            2. Analyze root causes\n\
            3. Model potential impact of intervention\n\
            4. Provide specific, actionable recommendations\n\
            5. Estimate effort and expected return"
        )
    ])
```

When to use: User workflow naturally has distinct phases; each phase produces different outputs.


### Converting Soft to Hard Workflows

As you gain experience with a soft workflow, look for opportunities to harden it:


### Soft Pattern

Can It Be Hardened?

(table/list, cells in reading order)
- Fixed sequence of tool calls
- Yes → Use  SequentialWorkflow
- Deterministic data gathering
- Yes → Use server-side steps
- Fuzzy matching user input
- Hybrid → Server gathers, AI matches
- Dynamic branching based on results
- Maybe → Complex, evaluate case-by-case
- Creative interpretation
- No → Keep as soft workflow
- Multi-domain coordination
- No → AI must reason across servers
- Example: Hardening a Report Workflow
- Before (Soft):

```rust
Prompt::new("weekly-report")
    .messages(vec![
        PromptMessage::user(
            "Generate weekly sales report:\n\
            1. Query revenue by region\n\
            2. Calculate week-over-week change\n\
            3. Format as markdown table"
        )
    ])
```


### After (Hard):

```rust
SequentialWorkflow::new("weekly_report", "Generate weekly sales report")
    .argument("week", "Week number (1-52)", true)
    .step(
        WorkflowStep::new("current", ToolHandle::new("sales_query"))
            .arg("week", prompt_arg("week"))
            .bind("current_data")
    )
    .step(
        WorkflowStep::new("previous", ToolHandle::new("sales_query"))
            .arg("week", /* week - 1 calculation */)
            .bind("previous_data")
    )
    .step(
        WorkflowStep::new("format", ToolHandle::new("format_report"))
            .arg("current", from_step("current_data"))
            .arg("previous", from_step("previous_data"))
            .bind("report")
    )
```

The hard workflow executes in a single round-trip with deterministic results.

Empty data sets Extremely large result sets Missing required resources Permission errors mid-workflow

(table/list, cells in reading order)
- Summary
- Soft workflows are appropriate when:
- Scenario
- Use Soft Workflow
- AI reasoning required
- Text prompts guide interpretation
- Exploration/discovery
- AI determines path based on findings
- Complex conditionals
- AI evaluates and branches
- Multi-server coordination
- AI reasons across domains
- Creative tasks
- Multiple valid approaches
- Design effective soft workflows by:

Convert to soft workflows only when genuine LLM reasoning is required.


### Hard Workflows: Server-Side Execution

Hard workflows execute entirely on the server side.


### The Power of Server-Side Execution

```text
                  Hard Workflow Execution
Client                          Server
        prompts/get              ►
        (quarterly_report Q3)
                                    Step 1: sales_query(Q3)
                                        ► bind("sales_data")
                                    Step 2: calculate_metrics
                                        ► uses sales_data
                                        ► bind("metrics")
                                    Step 3: format_report
                                        ► uses sales_data, metrics
                                        ► bind("report")
   ◄   complete conversation trace
        (all results included)
Total: 1 round trip (vs 6+ for soft workflow)
```


### SequentialWorkflow: The DSL

The PMCP SDK provides  SequentialWorkflow  for declarative workflow definition:

```rust
use pmcp::server::workflow::{SequentialWorkflow, WorkflowStep, ToolHandle};
use pmcp::server::workflow::dsl::*;

let workflow = SequentialWorkflow::new(
    "code_review",                    // Workflow name (becomes prompt name)
    "Comprehensive code review"        // Description
)
// Define arguments users provide
.argument("code", "Source code to review", true)        // required
.argument("language", "Programming language", false)     // optional

// Define sequential steps
.step(
    WorkflowStep::new("analyze", ToolHandle::new("analyze_code"))
        .arg("code", prompt_arg("code"))
        .arg("language", prompt_arg("language"))
        .bind("analysis")  // Store output for later steps
)
.step(
    WorkflowStep::new("review", ToolHandle::new("review_code"))
        .arg("analysis", from_step("analysis"))  // Use previous output
        .bind("review")
)
.step(
    WorkflowStep::new("format", ToolHandle::new("format_results"))
        .arg("analysis", from_step("analysis"))
        .arg("review", from_step("review"))
        .arg("format", constant(json!("markdown")))
        .bind("final_report")
);
```


### Registering with Server

```rust
let server = Server::builder()
    .name("code-review-server")
    .version("1.0.0")
    // Register tools that the workflow uses
    .tool_typed("analyze_code", analyze_code)
    .tool_typed("review_code", review_code)
    .tool_typed("format_results", format_results)
    // Register the workflow (creates a prompt handler)
    .prompt_workflow(workflow)?
    .build()?;
```


### When a user invokes  /code_review , the server:

1.

```rust
WorkflowStep::new(
    "step_name",                    // Identifies this step
    ToolHandle::new("tool_name")    // The tool to call
)
.arg("param", /* source */)         // Tool parameter
.bind("binding_name")               // Store output for other steps
```

```rust
use pmcp::server::workflow::dsl::*;

// 1. From workflow arguments (user-provided)
.arg("code", prompt_arg("code"))

// 2. From a previous step's entire output
.arg("data", from_step("analysis"))

// 3. From a specific field of a previous step's output
.arg("score", field("analysis", "confidence_score"))

// 4. Constant values
.arg("format", constant(json!("markdown")))
.arg("max_issues", constant(json!(10)))
```

```rust
// Step name: "analyze"
// Binding name: "analysis_result"
WorkflowStep::new("analyze", ToolHandle::new("analyzer"))
    .bind("analysis_result")  // ← This is the BINDING name

// Correct: reference the BINDING name
.arg("data", from_step("analysis_result"))  // [ok]

// Wrong: referencing the step name
.arg("data", from_step("analyze"))  // [no] Error!
```


### Complete Example: Code Review Workflow

```rust
use pmcp::server::workflow::{SequentialWorkflow, WorkflowStep, ToolHandle};
use pmcp::server::workflow::dsl::*;
use pmcp::{RequestHandlerExtra, Result, Server};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Tool input types
#[derive(Debug, Deserialize, JsonSchema)]
struct AnalyzeCodeInput {
    code: String,
    #[serde(default = "default_language")]
    language: String,
}

fn default_language() -> String { "rust".to_string() }

#[derive(Debug, Deserialize, JsonSchema)]
struct ReviewCodeInput {
    analysis: String,
    focus: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FormatCodeInput {
    code: String,
    issues: Vec<String>,
}

// Tool implementations
async fn analyze_code(input: AnalyzeCodeInput, _extra: RequestHandlerExtra) -> Result<Value> {
    Ok(json!({
        "language": input.language,
        "lines_of_code": input.code.lines().count(),
        "analysis_summary": format!("Analyzed {} lines", input.code.lines().count()),
        "issue_details": ["High complexity", "Missing error handling"]

    }))
}


async fn review_code(input: ReviewCodeInput, _extra: RequestHandlerExtra) -> Result<Value> {
    Ok(json!({
        "review_summary": format!("Reviewed with focus: {}", input.focus.join(", ")),
        "recommendations": ["Refactor complex functions", "Add error handling"],
        "approval_status": "conditional"
    }))
}

async fn format_results(input: FormatCodeInput, _extra: RequestHandlerExtra) -> Result<Value> {
    let annotations = input.issues.iter()
        .enumerate()
        .map(|(i, issue)| format!("// TODO {}: {}", i + 1, issue))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!({
        "formatted_code": format!("{}\n\n{}", annotations, input.code),
        "issues_annotated": input.issues.len()
    }))
// ... truncated, 55 more lines in source book ...
```


### Workflow Validation

Workflows are automatically validated when you register them with  .prompt_workflow() . If validation fails, registration returns an error and the server won't build.

(table/list, cells in reading order)
- Common validation errors:
- Error
- Cause
- Fix
- UnknownBinding
- from_step("x")  where no step binds to "x"
- Check binding names, add  .bind("x")
- UndefinedArgument
- prompt_arg("x")  where x not declared
- Add  .argument("x", ...)
- InvalidMapping
- Reference to undefined source
- Verify DSL helper usage
- For testing, you can also call  .validate()  directly:

```rust
#[test]
fn test_workflow_structure() {
    let workflow = create_my_workflow();
    workflow.validate().expect("Workflow should be valid");
}
```


### Hybrid Workflows: Graceful Handoff

When some steps require LLM reasoning, use hybrid workflows.

```rust
fn create_task_workflow() -> SequentialWorkflow {
    SequentialWorkflow::new(
        "add_project_task",
        "Add task to project with intelligent name matching"
    )
    .argument("project", "Project name (can be fuzzy)", true)
    .argument("task", "Task description", true)

    // Step 1: Server executes (deterministic)
    .step(
        WorkflowStep::new("list_pages", ToolHandle::new("list_pages"))
            .with_guidance("I'll first get all available project names")
            .bind("pages")
    )

    // Step 2: Server can't complete (requires fuzzy matching)
    // Provides guidance + resources for AI to continue
    .step(
        WorkflowStep::new("add_task", ToolHandle::new("add_task"))
            .with_guidance(
                "I'll now:\n\
                 1. Find the project from the list that best matches '{project}'\n\
                 2. Format the task according to the guide below\n\
                 3. Call add_task with the formatted_task parameter"
            )
            .with_resource("docs://task-format")?  // Embed docs for AI
            // No .arg() mappings - server detects incomplete args
            // and gracefully hands off to client LLM
            .bind("result")
    )
}
```


### How Hybrid Execution Works

1.


### 3. Server attempts Step 2 but:

Can't map "MCP Tester" to exact page name Detects incomplete argument mapping


### 4. Server returns partial conversation trace with:

Step 1 results (page list) Guidance for AI to complete Step 2 Embedded resource content (task formatting docs)

(table/list, cells in reading order)
- When to Use Hybrid
- Server Can Handle
- AI Must Handle
- API calls with exact parameters
- Fuzzy matching user input
- Data transformations
- Contextual decisions
- Resource fetching
- User clarification
- Sequential execution
- Creative interpretation
- Resource Embedding: The Developer's Leverage

Even when you can't fully automate tool binding, embedding relevant resources into the workflow response significantly improves AI success rates.

```rust
// Workflow step with embedded resources
.step(
    WorkflowStep::new("create_record", ToolHandle::new("database_insert"))
        .with_guidance("Create the record using the schema and validation rules below")
        // Embed documentation the AI needs to complete the step
        .with_resource("db://schema/customers")?      // Table structure
        .with_resource("db://constraints/customers")? // Validation rules
        .with_resource("docs://naming-conventions")?  // Format guidelines
        .bind("result")
)
```

- Without embedded resources, the AI must:


### With embedded resources, the AI receives:

1.

(table/list, cells in reading order)
- What to embed:
- Resource Type
- Example
- Why It Helps
- Schema definitions
- db://schema/orders
- AI knows exact field names and types
- Validation rules
- config://validation/email
- AI formats data correctly
- Format templates
- docs://task-format
- AI follows required patterns
- Resource Type
- Example
- Why It Helps
- Configuration
- config://regions
- AI uses valid enumeration values
- Examples
- Constraints
- docs://examples/queries
- AI learns by example
- docs://limits/api
- AI respects rate limits, size limits
- The control hierarchy:

```text
                MCP Developer Control
MOST CONTROL                        ►       LEAST CONTROL
Hard Workflow    Hybrid + Resources    Soft Workflow
Server executes  Server provides       Text guidance
all steps        context + guidance    only
• Deterministic  • AI completes with   • AI figures out
• Single trip      full context          everything
• Guaranteed     • High success rate   • Unpredictable
  results        • Developer curated   • Multiple trips
```

Best practice: When you can't make a step fully deterministic, ask yourself: "What documentation would I need to complete this step?" Then embed those resources.

```rust
SequentialWorkflow::new("analysis", "Multi-faceted analysis")
    .step(
        WorkflowStep::new("fetch", ToolHandle::new("fetch_data"))
            .arg("source", prompt_arg("source"))
            .bind("data")  // Single binding
    )
    .step(
        WorkflowStep::new("analyze", ToolHandle::new("analyzer"))
            .arg("input", from_step("data"))  // Uses "data"
            .bind("analysis")
    )
    .step(
        WorkflowStep::new("summarize", ToolHandle::new("summarizer"))
            .arg("input", from_step("data"))  // Also uses "data"
            .bind("summary")
    )
    .step(
        WorkflowStep::new("validate", ToolHandle::new("validator"))
            .arg("input", from_step("data"))  // Also uses "data"
            .bind("validation")
    )
```

```rust
// Assume "analysis" output is:
// {
//   "summary": { "text": "...", "length": 42 },
//   "scores": { "confidence": 0.95, "accuracy": 0.88 },
//   "metadata": { "timestamp": "..." }
// }

.step(
    WorkflowStep::new("report", ToolHandle::new("reporter"))
        .arg("summary", field("analysis", "summary"))     // Extract object
        .arg("confidence", field("analysis", "scores"))   // Extract object
        .arg("timestamp", field("analysis", "metadata"))  // Extract object
        .bind("report")
)
```

- Terminal or side-effect-only steps don't need bindings:

```rust
.step(
    WorkflowStep::new("process", ToolHandle::new("processor"))
        .arg("input", prompt_arg("data"))
        .bind("result")  // ← Needed by next step
)
.step(
    WorkflowStep::new("log", ToolHandle::new("logger"))
        .arg("message", from_step("result"))
        // NO .bind() - just logs, output not used
)
.step(
    WorkflowStep::new("notify", ToolHandle::new("notifier"))
        .arg("status", constant(json!("complete")))
        // NO .bind() - terminal step, side-effect only
)
```

```rust
SequentialWorkflow::new("research", "Research workflow")
    .instruction(InternalPromptMessage::system(
        "You are a research assistant. Be thorough and cite sources."
    ))
    .instruction(InternalPromptMessage::system(
        "Format all responses in markdown with clear sections."
    ))
    .step(...)
    .step(...)
```

```rust
  "Execute code_review workflow with code: 'fn main() {}', language: 'rust'"
```


### Message 2 [Assistant]:

```text
  "I'll perform a code review in 3 steps: analyze, review, format"
```


### Message 3 [Assistant]:

```rust
  "Calling analyze_code with {code: 'fn main() {}', language: 'rust'}"
```


### Message 4 [User]:

```text
  "Tool result: {analysis_summary: 'Analyzed 1 lines', issue_details: [...]}"
```


### Message 5 [Assistant]:

```text
  "Calling review_code with {analysis: '...', focus: ['security']}"
```


### Message 6 [User]:

```text
  "Tool result: {recommendations: ['Refactor...', 'Add error...']}"
```


### Message 7 [Assistant]:

```text
  "Calling format_results with {code: '...', issues: [...]}"
```


### Message 8 [User]:

```text
  "Tool result: {formatted_code: '// TODO 1: Refactor...\n\nfn main() {}'}"
```

The AI receives this complete trace and can synthesize a final response.

(table/list, cells in reading order)
- Workflow vs Tool: When to Use Each
- Use Tool
- Use Workflow
- Single operation
- Multi-step process
- AI decides when to call
- User explicitly invokes
- Flexible parameter choice
- Fixed execution sequence
- Use Tool
- Use Workflow
- Independent action
- Coordinated pipeline

```rust
// First: Try to make it fully deterministic
SequentialWorkflow::new("report", "Generate report")
    .step(...).step(...).step(...)

// If some steps need AI reasoning:
// Add .with_guidance() for hybrid execution

// If most steps need AI reasoning:
// Consider a soft workflow (text prompt) instead
```


### 2. Use Descriptive Binding Names

```text
// Good: Clear what the binding contains
.bind("customer_orders")
.bind("revenue_metrics")
.bind("formatted_report")

// Bad: Ambiguous
.bind("data")
.bind("result")
.bind("output")
```


### 3. Validation: Automatic and Fail-Fast

Good news: validation is automatic. When you call  .prompt_workflow() , the builder validates the workflow and returns an error if it's

```rust
invalid:

let server = Server::builder()
    .name("my-server")
    .version("1.0.0")
    .tool_typed("analyze_code", analyze_code)
    .prompt_workflow(workflow)?  // ← Validates here, fails if invalid
    .build()?;
```

If there's a validation error (unknown binding, undefined argument, etc.), the server won't start.


### Validation errors are actionable:

Error: Workflow validation failed: Unknown binding "analysis" in step "review".


### 4. Testing Workflows

Since validation is automatic at registration, the best way to catch errors early is with unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Workflow structure is valid
    #[test]
    fn workflow_is_valid() {
        let workflow = create_code_review_workflow();

        // .validate() is useful in tests for explicit validation
        workflow.validate().expect("Workflow should be valid");

        // Check expected structure
        assert_eq!(workflow.name(), "code_review");
        assert_eq!(workflow.steps().len(), 3);
        assert!(workflow.output_bindings().contains(&"formatted_result".into()));
    }

    // Test 2: Workflow executes correctly
    #[tokio::test]
    async fn workflow_execution() {
        let server = Server::builder()
            .name("test")
            .version("1.0.0")
            .tool_typed("analyze_code", analyze_code)
            .tool_typed("review_code", review_code)
            .tool_typed("format_results", format_results)
            .prompt_workflow(create_code_review_workflow())
            .expect("Workflow should register")
            .build()
            .expect("Server should build");

        let handler = server.get_prompt("code_review").unwrap();

        let mut args = HashMap::new();
        args.insert("code".into(), "fn test() {}".into());
        args.insert("language".into(), "rust".into());

        let result = handler.handle(args, test_extra()).await

            .expect("Workflow should execute");

        // Assert on conversation trace
        assert_eq!(result.messages.len(), 8);  // Intent + plan + 3 steps × 2 messages
    }
}
```


### 5. CLI Validation with cargo pmcp validate

For project-wide validation before commits or in CI pipelines, use the CLI:

```bash
# Validate all workflows in the current server
cargo pmcp validate workflows

# Verbose output (shows all test output)
cargo pmcp validate workflows --verbose

# Validate a specific server in a workspace
cargo pmcp validate workflows --server ./servers/my-server

# Generate validation test scaffolding
cargo pmcp validate workflows --generate
```


### What  cargo pmcp validate workflows  does:

1.

```json
  [ok] Compilation successful
```

```json
  [ok] Found 2 workflow test pattern(s)
```

```json
  [ok] Pattern 'workflow': 3 passed
  [ok] Pattern 'test_workflow': 2 passed
```

```text
  Your workflows are structurally valid and ready for use.
```

- If you don't have workflow tests yet, use  --generate :

```bash
cargo pmcp validate workflows --generate
```


### This creates  tests/workflow_validation.rs  with templates:

```rust
//! Workflow validation tests
//!
//! Generated by `cargo pmcp validate workflows --generate`

#[test]
fn test_workflow_is_valid() {
    let workflow = create_my_workflow();
    workflow.validate().expect("Workflow should be valid");
    assert_eq!(workflow.name(), "my_workflow");
}

#[test]
fn test_workflow_bindings() {
    let workflow = create_my_workflow();
    let bindings = workflow.output_bindings();
    assert!(bindings.contains(&"result".into()));
}

#[tokio::test]
async fn test_workflow_execution() {
    // Integration test template
}
```

(table/list, cells in reading order)
- 6. Developer Experience Roadmap
- Workflow validation happens at different stages:
- Stage
- When
- What's Caught
- Status
- Registration
- Server startup
- Binding errors, undefined args
- Unit Tests

```bash
cargo test
```

```bash
cargo pmcp validate
```

(table/list, cells in reading order)
- Project-wide validation
- Compile-Time
- Compilation
- Invalid workflows don't compile
- [GOOD] Automatic [GOOD] Pattern above [GOOD] Available 🔮 Future
- Stage
- When
- What's Caught
- IDE
- While typing
- Real-time feedback
- Status
- 🔮 Future

Best practice: Combine unit tests ( cargo test ) with CLI validation ( cargo pmcp validate ) in your CI pipeline.

(table/list, cells in reading order)
- Summary
- Hard workflows provide:
- The workflow spectrum:
- Benefit
- How
- Single round-trip
- Server executes all steps
- Deterministic execution
- Fixed sequence, no AI decisions
- Automatic data binding
- from_step() ,  field()  DSL
- Early validation
- Catch errors at registration time
- Easy testing
- Pure function tests, no AI required
- Type
- Hard
- Server Executes
- AI Handles
- All steps
- Final synthesis only
- Hybrid
- Deterministic steps
- Fuzzy matching, clarification
- Soft
- Nothing
- All steps (follows text guidance)

Hard workflows should be your default choice. Fall back to hybrid or soft only when genuine LLM reasoning is required.
