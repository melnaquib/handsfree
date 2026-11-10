---
chapter: 26
title: "MCP Apps: Interactive Widgets"
part: "Part VIII: Advanced Patterns"
use_when: "Shipping interactive widget UIs from tools (UI resources, MIME type, CSP, bridge)."
suggested_prerequisites: [ch06]
source_kb: 41
extracted_kb: 24
code_blocks: 51
quality: auto-extracted
---

# Ch 26: MCP Apps: Interactive Widgets

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### MCP Apps: Interactive Widgets

In this chapter, you'll learn to build rich, interactive widgets -- charts, maps, games, dashboards -- that run alongside your MCP tools. Instead of returning plain JSON that an AI describes in words, your server will serve full HTML interfaces that users can see and interact with directly.


### from tool calls

2.


### from  @modelcontextprotocol/ext-apps

+-----------------------------------------------------+ |  Host (Claude Desktop, ChatGPT, VS Code, etc.)      | |                                                      | |  tools/list --- _meta.ui.resourceUri ---> knows      | |                                          which tool  | |                

(table/list, cells in reading order)
- Here is how each layer works:
- Layer
- What It Is
- What It Does
- Widget
- An HTML file bundled with Vite
- Renders the UI the user sees and interacts with
- ext- apps
- Host

```text
@modelcontextprotocol/ext-apps   App
```

(table/list, cells in reading order)
- Embeds the widget in a sandboxed iframe
- Server
- Your Rust MCP server
- Processes tool calls, serves widget HTML, returns structuredContent


### Quick Start: Your First Widget

Let's go from zero to a working interactive widget in three steps.


### Step 1: Scaffold the Project

```bash
cargo pmcp app new my-widget-app
```

```text
  src/
    main.rs          # MCP server with tool handlers
  widget/
    mcp-app.html     # Starter widget using ext-apps App class
    package.json     # npm dependencies (ext-apps SDK, Vite)
    vite.config.ts   # Vite bundling config
  Cargo.toml
  README.md
```


### Step 2: Build the Widget

The widget must be bundled into self-contained HTML before the Rust server can embed it. This is required because Claude Desktop's iframe CSP blocks external script loading -- CDN imports fail silently.


### cd widget

```bash
npm install
npm run build
```

cd ..


### Step 3: Build and Run the Server

```bash
cargo build
cargo run
```


### Then preview your widget:

```bash
cargo pmcp preview --url http://localhost:3000 --open
```

The preview opens in your browser.

- MCP Apps requires the  mcp-apps  feature flag in your  Cargo.toml :

```toml
[dependencies]
pmcp = { version = "2.0", features = ["mcp-apps"] }
```

Widget Communication with ext-apps -- Build widgets using the ext-apps  App  class, implement required protocol handlers,


### bundle with Vite, and follow cross-host best practices

Reference: For complete API documentation covering every method, type, and edge case, see Chapter 12.5: MCP Apps Extension in the PMCP book.


### UI Resources and Widget Registration

In this section, you'll learn how to register widget HTML as MCP resources with the correct MIME type, declare Content Security Policy for external domains, and implement the resource handler pattern that connects widgets to the MCP protocol.


### The Correct MIME Type

Widgets must use the MIME type  text/html;profile=mcp-app  to be recognized by Claude Desktop, ChatGPT, and other MCP hosts.

```rust
use pmcp::types::ui::{UIResource, UIResourceContents};

// For resources/list -- declares the resource
let resource = UIResource::html_mcp_app(
    "ui://my-app/explorer.html",
    "Image Explorer",
);
// resource.mime_type == "text/html;profile=mcp-app"

// For resources/read -- serves the content
let contents = UIResourceContents::html(
    "ui://my-app/explorer.html",
    &html_content,  // your widget HTML string
);
// contents.mime_type == "text/html;profile=mcp-app"
```

Warning: Do not use the legacy  UIResource::html_mcp()  constructor -- it produces  text/html+mcp  which is not recognized by Claude Desktop. Always use  html_mcp_app()  for new code.

Notice it takes two parameters: the resource URI (always  ui://  scheme) and a display name. The MIME type is set automatically -- you never need to specify it manually.


### Registering Widget Resources

The  ResourceCollection  helper manages widget resource registration.

```rust
use pmcp::types::ui::{UIResource, UIResourceContents, ResourceCollection};

// Create the collection
let mut resources = ResourceCollection::new();

// Register a widget
let resource = UIResource::html_mcp_app(
    "ui://my-app/explorer.html",
    "Image Explorer",
);

let contents = UIResourceContents::html(
    "ui://my-app/explorer.html",
    &html_content,
);
```


### resources.add_ui_resource(resource, contents);

When the host calls  resources/list , the collection returns all registered  UIResource  entries.


### Embedding Widget HTML

Widget HTML is typically embedded in the Rust binary at compile time using  include_str! :

```text
const WIDGET_HTML: &str = include_str!("../../widget/dist/explorer.html");
```

The  include_str!  macro reads the file at compile time, so the built widget HTML must exist before  cargo build  runs.


### Declaring CSP for External Domains

If your widget loads external resources (images, API calls, fonts), you must declare them using  WidgetCSP .

```rust
use pmcp::types::mcp_apps::{WidgetCSP, WidgetMeta};

let csp = WidgetCSP::new()
    .resources("https://*.staticflickr.com")  // img-src: images, scripts, fonts
    .connect("https://*.staticflickr.com");   // connect-src: fetch/XHR

let meta = WidgetMeta::new()
    .resource_uri("ui://my-app/explorer.html")
    .prefers_border(true)
    .csp(csp);
```

(table/list, cells in reading order)
- What Each CSP Method Controls
- Method
- CSP Directive
- Use Case

```text
.resources("https://cdn.example.com")
```

```text
.connect("https://api.example.com")
```

Important: CSP metadata goes on the resource contents (returned by  resources/read ), not just the resource listing.

Important: Always use HTTPS for external domains --  http://  URLs are blocked by mixed-content policy even with CSP declarations.


### The Resource Handler Pattern

Every MCP Apps server needs a resource handler that connects widget HTML to the MCP protocol.

```rust
use pmcp::types::protocol::{ListResourcesResult, ResourceInfo};
use pmcp::types::ui::UIResource;

// Return all registered widget resources
fn list_resources(&self) -> ListResourcesResult {
    let resources = vec![
        UIResource::html_mcp_app(
            "ui://my-app/explorer.html",
            "Image Explorer",
        ).into(),
    ];
    ListResourcesResult::new(resources)
}
```


### Implementing resources/read

The  read()  method extracts the widget name from the URI and returns the HTML content:

```rust
use pmcp::types::ui::UIResourceContents;

fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
    match uri {
        "ui://my-app/explorer.html" => {
            let contents = UIResourceContents::html(
                uri,
                WIDGET_HTML,  // embedded at compile time
            );
            Ok(ReadResourceResult::from(contents))
        }
        _ => Err(Error::resource_not_found(uri)),
    }
}
```

(table/list, cells in reading order)
- The Three-Step Read Pattern
- Every resource read follows three steps:
- Step
- 1
- 2
- 3
- What Happens
- Code
- Extract widget name from  ui://app/{name}  URI
- uri.strip_prefix("ui://my-app/")
- Look up the widget HTML content
- include_str!  or  ResourceCollection
- Return as  UIResourceContents  with correct MIME type

```rust
UIResourceContents::html(uri, &html)
```


### Complete Example: Registering a Widget

Here is a complete example combining resource registration with the server builder:

```rust
use pmcp::types::mcp_apps::HostType;
use pmcp::types::protocol::ToolInfo;
use pmcp::types::ui::{UIResource, UIResourceContents, ResourceCollection};
use serde_json::json;

// 1. Embed widget HTML at compile time
const EXPLORER_HTML: &str = include_str!("../../widget/dist/explorer.html");

// 2. Create resource collection
let mut resources = ResourceCollection::new();

let resource = UIResource::html_mcp_app(
    "ui://my-app/explorer.html",
    "Image Explorer",
);

let contents = UIResourceContents::html(
    "ui://my-app/explorer.html",
    EXPLORER_HTML,
);
```


### resources.add_ui_resource(resource, contents);

```rust
// 3. Build server with host layer and resources
let server = Server::builder()
    .name("my-server")
    .version("1.0.0")
    .with_host_layer(HostType::ChatGpt)  // multi-host support
    .resources(resources)
    .build()?;
```


### Hot-Reload During Development

During development, you can read widget HTML from disk instead of embedding it:

```rust
use std::fs;

fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
    // Read from disk on every request -- changes appear on browser refresh
    let html = fs::read_to_string("widget/dist/explorer.html")?;
    let contents = UIResourceContents::html(uri, &html);
    Ok(ReadResourceResult::from(contents))
}
```

Only when you change Rust code (tool handlers,  main.rs ). Widget HTML changes during development are always instant -- just refresh the browser.

```rust
UIResource::html_mcp_app()  creates resource declarations with the correct MIME type ( text/html;profile=mcp-app )
UIResourceContents::html()  creates resource content for serving widget HTML
```


### Tool-UI Association and Data Flow

In this section, you'll learn how to associate tools with their widgets, return structured data for widget rendering, enable multi-host support, and add output schema validation.


### Associating Tools with Widgets

The  ToolInfo::with_ui()  method creates a tool definition that includes UI metadata linking the tool to its widget resource:

```rust
use pmcp::types::protocol::ToolInfo;
use serde_json::json;

let tool = ToolInfo::with_ui(
    "search_images",
    Some("Search for images by class name".to_string()),
    json!({
        "type": "object",
        "properties": {
            "class_name": { "type": "string" }
        },
        "required": ["class_name"]
    }),
    "ui://my-app/explorer.html",  // points to the widget resource
);
```

(table/list, cells in reading order)
- The four parameters are:
- Parameter
- Type
- Description
- name
- &str
- Tool name (e.g.,  "search_images" )
- description
- Option<String>
- Human-readable description for the AI model
- input_schema
- Value
- JSON Schema defining the tool's input parameters
- resource_uri
- &str
- URI of the widget resource (must match  UIResource  registration)

Important: The  resource_uri  must exactly match the URI used in  UIResource::html_mcp_app() . If these don't match, the host can't find the widget when the tool is called.


### Returning structuredContent

When a tool has an associated widget, the tool response should include  structuredContent  -- the data payload that the widget will

```rust
render:

use pmcp::types::protocol::{CallToolResult, Content};
use serde_json::json;

let result = CallToolResult::new(vec![
    Content::text("Found 42 images of dogs"),
])
.with_structured_content(json!({
    "columns": [
        { "name": "image_id", "data_type": "varchar" },
        { "name": "thumbnail_url", "data_type": "varchar" }
    ],
    "rows": [
        { "image_id": "abc123", "thumbnail_url": "https://..." }
    ]
}));
```

(table/list, cells in reading order)
- content vs structuredContent
- The two fields serve different audiences:
- Field
- Audience
- Purpose
- content
- AI model
- Text the model reads to understand the result
- structuredContent
- Widget
- JSON data the widget renders as a UI

Always include both --  content  with a human-readable summary, and  structuredContent  with the full data.


### Enabling Multi-Host Support

The  with_host_layer()  method on the server builder enables host-specific metadata enrichment.

```rust
use pmcp::types::mcp_apps::HostType;

Server::builder()
    .name("my-server")
    .version("1.0.0")
    .with_host_layer(HostType::ChatGpt)  // adds openai/* keys to _meta
    // ... tools, resources, etc.
    .build()
```


### What with_host_layer Does

Without  with_host_layer() , your server emits standard metadata only ( _meta.ui.resourceUri ).

(table/list, cells in reading order)
- Key
- Value
- Purpose
- openai/outputTemplate
- Widget resource URI
- Tells ChatGPT which widget to render
- openai/widgetAccessible
- true
- Marks the tool as widget-capable
- openai/toolInvocation/*
- Tool invocation metadata
- ChatGPT's tool result routing

These keys are required for ChatGPT but harmless for other hosts -- they simply ignore unknown  _meta  keys. This is why with_host_layer()  is the recommended default for any server that might be used with ChatGPT.


### When to Use It

Always recommended if your server might be used with ChatGPT Required if you're targeting ChatGPT specifically Optional if you're only targeting Claude Desktop (standard metadata is sufficient) Harmless for hosts that don't understand  openai/*  keys

```rust
use pmcp::types::mcp_apps::WidgetMeta;

let tool = ToolInfo::with_ui("my_tool", None, schema, "ui://my-app/widget.html")
    .with_widget_meta(WidgetMeta::new().prefers_border(true));
```

(table/list, cells in reading order)
- WidgetMeta Fields
- Method
- Effect
- Description

```text
.prefers_border(true)
```

```text
.description("...")
```

```text
.csp(widget_csp)
```

```text
.resource_uri("ui://...")
```

```rust
let tool = ToolInfo::with_ui(
    "search_images",
    None,
    input_schema,
    "ui://my-app/explorer.html",
)
.with_output_schema(json!({
    "type": "object",
    "properties": {
        "columns": { "type": "array" },
        "rows": { "type": "array" }
    }
}));
```

Adding  outputSchema  is optional but recommended.


### Complete Server Example

Here is a complete server setup combining all the concepts from this section:

```rust
use pmcp::types::mcp_apps::{HostType, WidgetMeta, WidgetCSP};
use pmcp::types::protocol::{ToolInfo, CallToolResult, Content};
use pmcp::types::ui::{UIResource, UIResourceContents};
use serde_json::json;

// Widget HTML embedded at compile time
const EXPLORER_HTML: &str = include_str!("../../widget/dist/explorer.html");

// 1. Define the tool with UI metadata
let tool = ToolInfo::with_ui(
    "search_images",
    Some("Search for images by class name".to_string()),
    json!({
        "type": "object",
        "properties": {
            "class_name": { "type": "string" }
        },
        "required": ["class_name"]
    }),
    "ui://my-app/explorer.html",
)
.with_output_schema(json!({
    "type": "object",
    "properties": {
        "columns": { "type": "array" },
        "rows": { "type": "array" }
    }
}))
.with_widget_meta(WidgetMeta::new().prefers_border(true));

// 2. Define the widget resource
let resource = UIResource::html_mcp_app(
    "ui://my-app/explorer.html",
    "Image Explorer",
);
let contents = UIResourceContents::html(
    "ui://my-app/explorer.html",
    EXPLORER_HTML,
);

// 3. Build the server with host layer
let server = Server::builder()
    .name("my-server")
    .version("1.0.0")
    .with_host_layer(HostType::ChatGpt)
    // register tool and resources...
    .build()?;
```

```rust
ToolInfo::with_ui()  associates a tool with its widget by adding  _meta.ui.resourceUri  to the  tools/list  response
```

In the next section, you'll learn how to build widgets using the ext-apps  App  class, implement required protocol handlers, and bundle with Vite.


### Widget Communication with ext-apps

In this section, you'll learn how to build widgets using the  @modelcontextprotocol/ext-apps  SDK, implement the required protocol handlers, bundle with Vite, and follow cross-host best practices.


### The ext-apps SDK

Widgets use the  @modelcontextprotocol/ext-apps  SDK to communicate with MCP hosts.

Create  App  -- no capabilities needed (do not pass  tools ) 2.


### Required Protocol Handlers

+-----------------------------------------------------------------------+ |                    CRITICAL: Handler Registration                      | +-----------------------------------------------------------------------+ |                                                                       | |  


### Register all handlers before connecting:

```text
// ALL of these are required -- not just ontoolresult
```

app.onteardown = async () => { return {}; }; app.ontoolinput = (params) => { console.debug("Tool input:", params); }; app.ontoolcancelled = (params) => { console.debug("Cancelled:", params.reason); }; app.onerror = (err) => { console.error("App error:", err); }; app.ontoolresult = (result) => { /* y

Required?

(table/list, cells in reading order)
- onteardown
- Host is shutting down the connection
- Yes -- must return  {}
- ontoolinput
- Host is about to call a tool
- ontoolcancelled
- Tool call was cancelled
- onerror
- Protocol error occurred
- Yes
- Yes
- Yes
- ontoolresult
- Tool call completed with result
- Recommended -- this is where you get data
- Capabilities Declaration

Do not pass  tools  capability to  new App() .

```text
// Correct -- receives tool results, can call server tools
```


### const app = new App({ name: "my-widget", version: "1.0.0" });

```text
// WRONG -- ChatGPT rejects the tools capability
// const app = new App({ name: "my-widget", version: "1.0.0", capabilities: { tools: true } });
```

Tool results are delivered via  hostContext.toolOutput  and the  ontoolresult  callback without needing the  tools  capability.


### Hands-On: Minimal Widget

Create your source HTML with a bare import (Vite resolves it from  node_modules ):


### <!DOCTYPE html>

```rust
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>My Widget</title>
</head>
<body>
  <div id="output">Waiting for data...</div>

  <script type="module">
    import { App } from "@modelcontextprotocol/ext-apps";

    const app = new App({ name: "my-widget", version: "1.0.0" });

    // 1. Register ALL handlers BEFORE connecting (required by protocol)
    app.onteardown = async () => { return {}; };
    app.ontoolinput = (params) => { console.debug("Tool input:", params); };
    app.ontoolcancelled = (params) => { console.debug("Cancelled:", params.reason); };
    app.onerror = (err) => { console.error("App error:", err); };

    app.ontoolresult = (result) => {
      if (result.structuredContent) {
        renderData(result.structuredContent);
      }
    };

    // 2. Connect to host
    await app.connect();

    // 3. Read initial data from hostContext
    const ctx = app.getHostContext();
    if (ctx?.toolOutput) {
      renderData(ctx.toolOutput);
    }

    // 4. Optionally call server tools from the widget
    async function refresh() {
      const result = await app.callServerTool({

        name: "search_images",
        arguments: { class_name: "Dog" },
      });
      if (result.structuredContent) {
        renderData(result.structuredContent);
      }
    }

    function renderData(data) {
      document.getElementById("output").textContent = JSON.stringify(data);
    }
  </script>
```

1.


### Calling Server Tools from Widgets

Interactive widgets can call tools on the MCP server using  app.callServerTool() :


### const result = await app.callServerTool({

```yaml
  name: "search_images",
  arguments: { class_name: "Dog" },
});
```


### if (result.structuredContent) {

```text
  // Render the structured data
  renderData(result.structuredContent);
}
```

This works on hosts that support it (mcp-preview, Claude Desktop). Add a  .catch()  fallback for hosts that don't:


### try {

```yaml
  const result = await app.callServerTool({
    name: "search_images",
    arguments: { class_name: "Dog" },
  });
  renderData(result.structuredContent);
} catch (err) {
  console.warn("callServerTool not supported on this host:", err);
  // Fallback: display cached data or show a message
}
```

```bash
import { useApp, useHostStyles } from "@modelcontextprotocol/ext-apps/react";

export default function MyWidget() {
  const { app, isConnected } = useApp({
    appInfo: { name: "my-widget", version: "1.0.0" },
    // Do not pass capabilities.tools -- ChatGPT rejects it
    onAppCreated: (app) => {
      app.ontoolresult = (result) => {
        // handle new tool results
      };
    },
  });

  // Apply host theme, CSS variables, and fonts
  useHostStyles(app, app?.getHostContext());

  if (!isConnected) return <div>Connecting...</div>;

  const handleSearch = async () => {
    const result = await app.callServerTool({
      name: "search_images",
      arguments: { class_name: "Dog" },
    });
    // use result.structuredContent
  };

  return <button onClick={handleSearch}>Search</button>;
}
```

The  useApp  hook handles the full lifecycle: creating the App instance, registering required handlers, and calling  connect() .

Required protocol handlers ( onteardown ,  ontoolinput ,  ontoolcancelled ,  onerror ) are registered automatically by the hook.


### Loading External Images

Hosts enforce strict CSP on widget iframes.


### function loadImage(img, url) {

```text
    fetch(url, { mode: 'cors' }).then(function(r) {
        if (!r.ok) throw new Error(r.status);
        return r.blob();
    }).then(function(blob) {
        img.src = URL.createObjectURL(blob);
        // Revoke blob URL after render to prevent memory leaks
        img.onload = function() { URL.revokeObjectURL(img.src); img.onload = null; };
    }).catch(function() {
        // Fallback to direct URL -- works on permissive hosts (ChatGPT, mcp-preview).
        // On strict hosts, onerror fires and shows a placeholder.
        img.src = url;
    });
}
```

Why both server-side CSP and client-side fetch-blob?

Memory: Always call  URL.revokeObjectURL()  after the image loads.


### Bundling Widgets with Vite

Widgets must be self-contained HTML files with all JavaScript inlined.


### Why Bundling Is Required

Claude Desktop's iframe CSP blocks external script loading.

```bash
npm init -y
npm install @modelcontextprotocol/ext-apps
npm install -D vite vite-plugin-singlefile
```


### vite.config.ts

vite-plugin-singlefile  uses  inlineDynamicImports  which only supports a single input per build.

```text
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";
```


### const widget = process.env.WIDGET || "mcp-app";

```bash
export default defineConfig({
  plugins: [viteSingleFile()],
  build: {
    target: "esnext",  // required -- ext-apps SDK uses top-level await
    rollupOptions: {
      input: `${widget}.html`,
    },
    outDir: "dist",
    emptyOutDir: false,  // preserve other widgets' output
  },
});
```


### Building Multiple Widgets

```json
{
  "scripts": {
    "build": "rm -rf dist && WIDGET=image-explorer vite build && WIDGET=relationship-viewer vite build"
  }
}
```


### Build and Embed

```bash
npm run build
# -> dist/image-explorer.html (~130KB, self-contained)
```


### Embed the built output in your Rust binary:

```text
const WIDGET_HTML: &str = include_str!("../../widget/dist/image-explorer.html");
```


### Build Order

Widget HTML must exist before  cargo build  (since  include_str!  runs at compile time):

```bash
cargo build --release
```

ALL handlers must be registered before

```text
connect() , even if they only log a debug message. This is the #1 issue when porting widgets from mcp-preview (which is more forgiving)
```


### Widget loads but never shows tool results:

Do NOT pass  tools  capability to  new App() .


### "Received a response for an unknown message ID" (mcp-preview):

Two App instances on the same postMessage channel.


### Images/external resources blocked (Claude.ai):

Claude.ai enforces strict CSP:  img-src 'self' data: blob:  -- external image URLs are blocked. Always use HTTPS.


### Widget not rendered at all (Claude Desktop):

Check  resources/read  returns  _meta.ui.resourceUri  (required by Claude Desktop).

(table/list, cells in reading order)
- Chapter Summary
- Here's what you've learned across all three sections of Chapter 20:
- Concept
- What You Learned

```rust
UIResource::html_mcp_app()
```

```rust
ToolInfo::with_ui()
```

```text
with_host_layer()
```

(table/list, cells in reading order)
- Enable multi-host support (ChatGPT, Claude Desktop, VS Code)
- ext-apps App class
- Cross-host widget communication with required protocol handlers
- Vite bundling
- Self-contained HTML required by Claude Desktop CSP
- The standard development workflow:


### Practice Ideas

Ready to experiment?
