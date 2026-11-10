---
chapter: 30
title: "Agents & Teams"
part: "Part VIII: Advanced Patterns"
use_when: "Running agent loops and teams against MCP servers; deploy-anywhere agents."
suggested_prerequisites: [ch27]
source_kb: 31
extracted_kb: 8
code_blocks: 14
quality: auto-extracted
---

# Ch 30: Agents & Teams

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Agents & Teams: Deploy-Anywhere Agent Loops

Every chapter before this one put you on the server side of MCP: you expose tools, and something else decides when to call them. PMCP ships that loop as a crate:  pmcp-agent  gives you an  AgentEngine  built over three seams (a completion source, a tool invoker, and a store), plus an  AgentPackage  manifest that describes the agent as data rather than as Rust.

Explain the source-agnostic loop invariant and articulate why swapping a completion source must not change a single line of the decision loop (the load-bearing design property of PMCP's agent implementation). Distinguish the two shapes an agent takes — standalone (it owns the model credential) and hosted-sampled (its caller owns the credential) — and choose between them on deployment grounds rather than code grounds.


### Why Agents & Teams Matter for Enterprise MCP

Without a first-class agent loop, "agentic" behaviour tends to live as a hand-rolled  while  loop inside whichever application happened to need it first. The loop's termination policy is buried in control flow rather than declared; the model endpoint is a compile-time dependency rather than a deployment choice; and the credential for that endpoint is baked into every process that runs the loop.

The loop reports termination as a  RunOutcome  ( Completed , LimitReached ,  RetryRequired ,  Failed ) rather than as a  Result , so "the agent ran out of iterations" is a decision point you inspect, not an error you swallow.


### The Source-Agnostic Loop

The single most important design property in PMCP's agent implementation is this: the decision loop is source-agnostic — the same AgentEngine , driven by the same  ResolvedAgentConfig , runs against any  CompletionSource  with no change to the loop.

```rust
use pmcp_agent::{AgentEngine, InMemoryStore};

let engine = AgentEngine::new(source, invoker, InMemoryStore::new(), config);
let outcome = engine.run("standalone-run").await;
```

Hosted, you do not have a source at construction time.

```rust
use std::sync::Arc;
use pmcp_agent::{AgentServer, CompletionSourceFactory, SamplingSourceFactory};

let factory: Arc<dyn CompletionSourceFactory> = Arc::new(SamplingSourceFactory::new());
let agent = AgentServer::builder(package, config, factory, invoker, store).build()?;
```


### agent.run(transport).await?;

Look at what did not change between those two snippets: the config, the invoker, the store, and — crucially — the loop.

```rust
AgentEngine::run  is called identically in both. Only the provenance of the source differs, and in the hosted case even that is deferred
```

behind a factory rather than restructured.

The seam design eliminates that failure mode by construction, and the repository proves it rather than asserting it.

That line is asserted by an automated test in this repository ( tests/docs04_examples_run.rs ), so the claim is checked on every run of the integration suite rather than merely documented.


### Tier 1: Scaffold and Run an Agent (cargo pmcp)

The first tier exists to make the mechanical steps obvious, and it uses no crate API at all.

```bash
cargo install cargo-pmcp
cargo pmcp agent new research-agent
```


### cd research-agent

```bash
cargo pmcp agent new  validates the name, refuses a symlinked destination outright, refuses a non-empty directory unless you pass  --
```

```text
agent.package.json   # the AgentPackage: instructions, llm slot, limits
Cargo.toml           # the complete dependency set, already pinned
src/
    main.rs          # a manifest-driven runner
tests/
    pin.rs           # a tripwire that fails if the pinned versions drift
```

The interesting artifact is  agent.package.json .


### Now run the loop:

```bash
cargo pmcp agent dev --source fixed

cargo pmcp agent dev  loads an  AgentPackage  — from  --package <path> , else from  ./agent.package.json , else from a built-in
```

demo package identical to the scaffold's — resolves it, and drives the loop against the source you name:

(table/list, cells in reading order)
- fixed
- A scripted, offline source
- No
- No
- --source
- What supplies completions

(table/list, cells in reading order)
- openai-compat  (default)
- An OpenAI-compatible HTTP endpoint
- Yes
- Via  --api-key-env
- sampling
- The MCP host, over stdio
- No (stdio)
- None — the host's

```bash
cargo pmcp agent dev --source openai-compat --model llama3.2
```

That refusal is deliberate: a credential travelling over cleartext to a remote host is a mistake worth failing loudly for, and failing at construction means it cannot happen halfway through a run.


### Tier 2: One Loop, Two Completion Sources

The second tier drops below the CLI to watch both shapes run in the same process, with no network and no API key:

```bash
cargo run -p pmcp-agent --example s50_standalone_vs_sampled
```

This example lives in  crates/pmcp-agent/examples/ , and the  s50  number is also taken in the root  examples/  directory — a bare example number is ambiguous in this repository, which is why this course always cites examples by their full runnable invocation.


### == 1. STANDALONE (mock CompletionSource) ==

```text
   outcome = Completed, tools dispatched = 1
```

The second is the hosted-sampled run.


### == 2. HOSTED-SAMPLED (AgentServer + SamplingSource) ==

```text
   terminal task status = completed, result = ...
```


### And then, only after both have finished:

Done — the same AgentEngine ran standalone and hosted-sampled.

```bash
cargo run --example s49_sampling_host
```


### Round-trip complete. Server received completion:

Rather than re-derive the disambiguation here, read the page devoted to it: Sampling & Hosting in the PMCP book, and its companion Chapter 12.15: Agents as MCP Clients.

Both sections must be present. A run that printed only the standalone section and then the banner would be reporting a claim it did not earn — which is exactly why the automated test asserts the banner and why Exercise 2 asks you to observe both sections rather than just the last line.


### Tier 3: A Team — Many Agents, Four Reference Servers

The third tier shows that agents compose.

```bash
cargo pmcp team dev
```

Pass  --serve  to expose team-mcp over HTTP instead of running the transcript,  --package  to run your own  TeamPackage , and  --llm to swap the offline fixed source for a real OpenAI-compatible endpoint.

(table/list, cells in reading order)
- The four reference servers divide the team's shared concerns:
- Server
- Owns
- Representative tools
- team-fs
- The shared workspace and the draft/review split
- fs__write ,  fs__sync_to_review ,  fs__read
- approval-mcp
- The human-in-the-loop gate
- team_approval__ask_<role> ,  resolve_approval
- mem-mcp
- team-mcp
- Durable team memory across runs
- mem__add
- Agent-facing dispatch to member agents
- team_mcp__<member>

```bash
cargo run -p pmcp-team-servers --example doc_review_team --features runtime
```

The  --features runtime  is required — this example declares  required-features = ["runtime"]  and does not build without it.

```text
   All four reference servers cooperated in ONE offline process.
```

```text
Try this: run the example and count the     Step N     markers in the output against the five-step flow described above. Then re-run it
```

Cargo refuses to build the target at all rather than building a degraded one — feature-gating an example is how the repository makes "this needs the runtime" a compile-time fact instead of a runtime surprise.


### Cross-SDK Compatibility

Nothing in the two shapes is PMCP-specific at the wire level, and that is the point.

The related-task pointer the team's dispatch surfaces is likewise a spec-level key —  io.modelcontextprotocol/related-task  in  _meta — rather than a PMCP invention, so a client that understands task-augmented results from any SDK will follow the pointer PMCP emits.


### Before continuing, make sure you can answer:

What does the source-agnostic loop invariant guarantee, and why does the hosted shape need a  CompletionSourceFactory rather than a  CompletionSource ?
