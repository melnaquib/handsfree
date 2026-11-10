---
chapter: 0
title: "Prerequisites"
part: "Part I: Foundations"
use_when: "Setting up toolchain: Rust, cargo-pmcp, an MCP client."
suggested_prerequisites: []
source_kb: 5
extracted_kb: 3
code_blocks: 5
quality: auto-extracted
---

# Ch 0: Prerequisites

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

- Before starting this course, you should have:


### Familiarity with REST APIs and JSON

See the Prerequisites chapter for detailed setup instructions.


### Let's Begin

Enterprise AI integration is no longer optional.


### Prerequisites

Welcome!


### Our Learning Philosophy: Read, Don't Write

You need to know how to read Rust code, not how to write it.


### 3. Review the generated code 4. Run the compiler to catch any issues

The Rust compiler becomes your safety net—if it compiles, it almost certainly works correctly.


### Why This Approach Works

Rust has an exceptional compiler that provides clear, actionable error messages.


### You describe what you want


### AI generates Rust code

```text
    ↓
Compiler catches issues (if any)
    ↓
```


### AI fixes issues automatically


### Working, production-ready code

We cover this in depth in Part VI: AI-Assisted Development, where you'll learn how to effectively collaborate with AI assistants to build MCP servers.


### Rust Concepts You'll Encounter

Don't worry if these aren't familiar yet—you'll learn them through the code examples.

(table/list, cells in reading order)
- Familiar Concepts (Coming from Java/C#)
- Java/C#
- Rust
- Example
- class
- interface
- struct
- trait
- struct User { name: String }

```rust
trait Tool { fn call(&self); }
```

```text
Ok(value)  or  Err(error)
```

```text
Some(value)  or  None
```

When you see  result? , it means "return the error if there is one, otherwise continue."

(table/list, cells in reading order)
- What You Don't Need to Master
- These advanced topics are handled by AI assistants and the PMCP SDK:
- Lifetime annotations ( 'a ,  'static ) Unsafe Rust
- Advanced trait bounds Macro writing Memory layout optimization
- Technical Prerequisites
- Required Tools

```text
# You'll set these up in Chapter 2
```

(table/list, cells in reading order)
- Helpful Background
- HTTP and APIs (you probably already know this):
- HTTP methods (GET, POST) JSON format REST API concepts
- Command Line (basic comfort):
- Running commands Environment variables
- Cloud Platforms (For Deployment Chapters)
- Parts III-V cover deployment. Familiarity with one is helpful:
- AWS - Lambda, API Gateway Cloudflare - Workers
- Google Cloud - Cloud Run

Don't worry if cloud is new—we guide you step by step.


### Environment Setup

Chapter 2 includes an interactive setup exercise that guides you through:

Error handling uses explicit types instead of exceptions—cleaner once you're used to it 4.


### You're ready if you can:

Read code and understand its intent Describe what you want to build in plain English


### Run commands in a terminal Accept that AI will write most of your code

That's it.
