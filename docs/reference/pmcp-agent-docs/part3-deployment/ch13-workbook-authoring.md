---
chapter: 13
title: "Authoring Workbooks as MCP Tools"
part: "Part III: Deployment"
use_when: "Authoring workbook tables as MCP tools: the table contract, tiers, governance."
suggested_prerequisites: [ch12]
source_kb: 10
extracted_kb: 4
code_blocks: 10
quality: auto-extracted
---

# Ch 13: Authoring Workbooks as MCP Tools

> Auto-extracted from the PMCP book print export. Code blocks are complete where the PDF kept them; flattened tables read cell by cell; exercises excluded. Verify APIs against https://github.com/paiml/rust-mcp-sdk before use.


### Authoring Workbooks as MCP Tools: The Table Contract

Who this is for.


### 1. The big idea: the workbook IS the contract

You already have the process in a spreadsheet.


### Two ends of one pipeline:

You author — name input and output regions as Excel Tables, fill realistic example values, pick governance from a dropdown.


### Always lead with the CLI. The command you will lean on throughout:

```bash
cargo pmcp workbook explain your-workbook.xlsx
```

It prints exactly what an AI will see. You are never guessing.

(table/list, cells in reading order)
- Input Table
- Excel Table,  name | value | description | tier
- yes — tool inputs
- Output Table(s)
- Excel Table,  name | value | description
- yes — one tool per Table
- Reference / lookup
- any cells/ranges ( VLOOKUP  rate cards)
- Intermediate calc
- any sheets
- no
- no


### Output Table — name | value | description

name  — the served output key. value  — the authored expected result = the reconciliation oracle the gate checks.

(table/list, cells in reading order)
- What gets harvested from where
- Schema field
- Source
- field key
- name  column cell
- type
- unit
- enum
- value  cell type

(table/list, cells in reading order)
- data-validation list on the  value  cell
- description
- description  column cell
- tier / strict
- tier  column cell (inputs only)
- Schema field
- Source
- test input
- value  cell
- output oracle
- output  value  cell

```yaml
  inputs:  [ InputField{ key, type, unit?, enum?, description, tier } ],  // shared pool
  tools:   [ Tool{ name, description, input_keys, outputs, oracle } ]
}
```

```text
                          income    | 100000   | annual gross (USD $)   | variable
                          filing    | single ▼ | filing status          | variable   ← enum from dropdown
                          withheld  | 15000    | tax withheld YTD (USD) | variable
                          rate      | 0.22     | statutory bracket rate | strict     ← not caller-exposed
```

```text
    (formula DAG, cross-sheet)            ▲
  ▼                                          VLOOKUP(ref_brackets)
calc sheets + lookup/reference regions (DAG interior, not exposed)
```

```text
        ▼                                        ▼
```

```text
                 tax_owed       | 18241 | federal tax liability (USD)
                 effective_rate | 0.182 | effective tax rate (%)
                   inputs (DAG-derived): income, filing
```

```text
                 refund  | -3241 | estimated refund (neg = owed)
                   inputs (DAG-derived): income, filing, withheld
```


### 6. Governance via the tier dropdown

variable  — a normal caller-exposed input (default). strict  — a BA-governed constant, not offered to the AI.

Governance is a column, co-located with the field, so it can never drift.

```bash
cargo pmcp workbook explain template.xlsx
```


### tool calculate_tax

```yaml
  description: Compute federal tax from income & filing
  inputs:
    income: number
  outputs:
    tax_owed: number
    effective_rate: number
```


### tool estimate_refund

```yaml
  description: Estimate refund given withholding
  inputs:
    income: number
    withheld: number
  outputs:
    refund: number
```

Read this the way an LLM does — as a coherent "what I do / how to call me." The preview is projected through the same compiler path the server registers, so it can't drift from what gets served.

```bash
cargo pmcp workbook explain template.xlsx --format json
```


### A note for readers upgrading

The table model replaces the old named-range authoring model.


### Summary

Author two declaration Tables: an Inputs Table and one output Table per tool.
