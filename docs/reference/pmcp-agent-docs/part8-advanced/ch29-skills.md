---
chapter: 29
title: "Skills: Agent Workflow Instructions"
part: "Part VIII: Advanced Patterns"
use_when: "Publishing agent workflow instructions from the server (SEP-2640), dual skill/prompt surface."
suggested_prerequisites: [ch06, ch28]
source_kb: 29
extracted_kb: 5
code_blocks: 9
quality: auto-extracted
---

# Ch 29: Skills: Agent Workflow Instructions

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.

## Agent notes (checked against extracted text)

- API: `Skill::new(name, body)`, `.skill(...)` on the builder, `.with_path("a/b/name")`, `.with_reference(SkillReference::new(path, mime, body))`.
- SEP-2640 section 9: reference files are readable via `resources/read` but must NOT appear in `resources/list`.
- `bootstrap_skill_and_prompt(skill, prompt_name)` publishes a skill plus a prompt fallback for hosts without SEP-2640.
- Skill and prompt surfaces are byte-equal by construction.
- A skill body without frontmatter is excluded from the projection, with a build-time warning.

## Extracted content


### Skills: Agent Workflow Instructions (SEP-2640)

Agent-workflow instructions belong on the server, not buried in client code.

Register a multi-file Skill ( SKILL.md  + references) using  Skill::with_reference(...)  and explain SEP-2640 §9 visibility filtering (references readable via  resources/read , never listed via  resources/list ). Use  bootstrap_skill_and_prompt(skill, prompt_name)  to publish a Skill that also exposes a prompt fallback for hosts that don't yet support SEP-2640.


### Why Skills Matter for Enterprise MCP

Without Skills, agent-workflow instructions tend to live as system prompts inside client-side code.

The dual- surface bootstrap (covered in the next section) makes Skills work today, on hosts that don't yet support SEP-2640 — there is zero "wait for the host ecosystem to catch up" friction in adopting Skills.


### The Dual-Surface Invariant

The single most important design property in PMCP's Skills implementation is this: the skill surface and the prompt surface are byte- equal by construction. The two surfaces cannot drift.

```rust
pub fn as_prompt_text(&self) -> String {
    let mut out = String::new();
    out.push_str(&self.body);
    if !self.body.ends_with('\n') {
        out.push('\n');
    }
    for r in &self.references {
        out.push_str("\n--- ");
        out.push_str(&r.relative_path);
        out.push_str(" ---\n");
        out.push_str(&r.body);
        if !r.body.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}
```


### Tier 1: Hello-World Skill

The first tier exists to make the mechanical steps obvious.

```text
# Hello World Skill
```

```rust
const HELLO_WORLD: &str = include_str!("skills/hello-world/SKILL.md");

let _server = pmcp::Server::builder()
    .name("skills-demo")
    .version("0.1.0")
    .skill(Skill::new("hello-world", HELLO_WORLD))
    .build()?;

Skill::new(name, body)  constructs the value.  .skill(...)  registers it on the builder. The resulting URI is  skill://hello-
```

A skill whose body carries no frontmatter is excluded from that projection, with a build-time warning naming it — which is why every snippet in this chapter gives its skill a real frontmatter block.

```text
# Refund Workflow
```

Registration uses  .with_path(...)  to namespace the skill (so  acme/billing/refunds  becomes the URI segment instead of just refunds ) and would chain  .with_reference(...)  calls for each supporting file.

```rust
.skill(Skill::new("refunds", REFUNDS).with_path("acme/billing/refunds"))
```

The crucial SEP-2640 §9 rule: supporting files are addressable via  resources/read , but they MUST NOT appear in  resources/list .

PMCP enforces this in  SkillsHandler::list()  — references are simply never returned, regardless of how many are registered. The test tests/skills_integration.rs  asserts that listing returns only  SKILL.md  URIs, never reference URIs and nothing synthesized.


### Tier 3: Code-Mode Skill (Composition with Another Advanced Feature)

The third tier shows that Skills compose with other advanced PMCP features. The skill teaches the agent the schema, the canonical patterns, and the validation policies it must obey when generating GraphQL queries.


### The skill body lives at  examples/skills/code-mode/SKILL.md :

--- name: code-mode description: Generate validated GraphQL queries against this server's schema ---

```text
# Code Mode
```


### ## Before you generate a query

1.

```rust
fn build_code_mode_skill() -> Skill {
    Skill::new("code-mode", CODE_MODE)
        .with_reference(SkillReference::new(
            "references/schema.graphql",
            "application/graphql",
            CODE_MODE_SCHEMA,
        ))
        .with_reference(SkillReference::new(
            "references/examples.md",
            "text/markdown",
            CODE_MODE_EXAMPLES,
        ))
        .with_reference(SkillReference::new(
            "references/policies.md",
            "text/markdown",
            CODE_MODE_POLICIES,
        ))
}
```

```rust
let _server = pmcp::Server::builder()
    .name("skills-demo")
    .version("0.1.0")
    .skill(Skill::new("hello-world", HELLO_WORLD))
    .skill(Skill::new("refunds", REFUNDS).with_path("acme/billing/refunds"))
    .bootstrap_skill_and_prompt(code_mode_skill.clone(), "start_code_mode")
    .build()?;
```


### bootstrap_skill_and_prompt(skill, "start_code_mode")  registers BOTH:

1.


### assert_eq!(

```text
    sep_2640_text, prompt_text,
    "dual-surface invariant violated: SEP-2640 read concatenation != prompt body"
);
```

If the invariant ever broke, the example would panic on exit.

A developer evaluating which SDK to adopt can implement the same  hello-world  and  refunds  skill in each, count lines of code, and form a calibrated judgment about ergonomics rather than guessing.

A skill that works in one platform's MCP host should work in another's.


### Before continuing, make sure you can answer:

What does the dual-surface invariant guarantee, and why does it require byte-equality between the skill surface and the prompt surface (not just semantic equivalence)?
