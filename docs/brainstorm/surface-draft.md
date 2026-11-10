# Surface draft: resources, tools, driver interface, ops, GUI

Brainstorm, not requirements. Becomes OpenSpec specs when a change needs it: `mcp-tools`, `macro`, `drivers`, `gui`. Terms: driver (runs the ops), run (not job).

## 1. MCP resources


| Resource       | Args    | Returns                          |
| -------------- | ------- | -------------------------------- |
| `list_drivers` | none    | list of`{id, description}`       |
| `get_drivers`  | `[ids]` | list of`{id, name, description}` |

Example:

```json
list_drivers() -> [
  {"id": "hf:drivers:a11y", "description": "Driver Desktop using accessibility APIs"}
]

get_drivers(["hf:drivers:a11y"]) -> [
  {"id": "hf:drivers:a11y", "name": "Accessibility Driver",
   "description": "Driver Desktop using accessibility APIs"}
]
```

## 2. MCP tools

### 2.1 Observe


| Tool              | Args         | Returns      |
| ----------------- | ------------ | ------------ |
| `list_nodes_tree` | root id      | node tree    |
| `get_nodes_infos` | `[node_ids]` | node details |

### 2.2 Run


| Tool                | Args                                   |
| ------------------- | -------------------------------------- |
| `run_op`            | `driver`, one op                       |
| `run_macro`         | `driver`,`[ops]`                       |
| `run_decision_tree` | `driver`,`[ops]`                       |
| `run_fsm`           | `driver`,`[ops]`(later, FSM is parked) |

### 2.3 Control


| Tool            | Actions                           |
| --------------- | --------------------------------- |
| `run_ctrl`      | start, pause, stop, reset, revert |
| `record_ctrl`   | start, pause, stop, reset         |
| `resume_run`    | resume a paused run               |
| `cancel_run`    | cancel a run                      |
| `policy_status` | read-only policy view             |

### 2.4 Rules

1. Not model-callable: approvals, kill switch.
2. Profiles: `observe`, `operate`, `test`, `guide`, `full`.

### 2.5 v1 tool list from the project summary (for reconciling)

`list_targets`, `describe_driver`, `observe`, `find`, `act` (open), `run_sequence`, `run_decision_tree`, `run_status`, `resume_run`, `cancel_run`, `policy_status`.

## 3. Driver interface v0


| Method     | Purpose                |
| ---------- | ---------------------- |
| `describe` | driver card / manifest |
| `targets`  | what can be controlled |
| `observe`  | read state             |
| `op`       | run one op             |
| `health`   | liveness               |
| `reset`    | back to a clean state  |

Unsupported calls raise `NotSupported`.

Driver caps (capability list): not defined yet.

## 4. Ops

### 4.1 v1


| Group           | Ops                                                       |
| --------------- | --------------------------------------------------------- |
| Input           | `invoke`,`set_value`,`type_text`,`mouse`,`key`,`navigate` |
| Sync and read   | `wait_for`,`read`,`assert`,`checkpoint`                   |
| Control flow    | `if`,`repeat`,`call`,`done`                               |
| Human and model | `approve`,`handoff`,`call llm`,`call llm jev`             |

Plus: handlers, `on_abort`, `budget`, `params`, `bindings`, `until`.

### 4.2 Candidates (not decided)

`launch`, `close`, `focus`, `scroll`, `capture`, `fail`, `drag`.

### 4.3 Result object

Fields or ops: `set`, `unset`, `get`.

### 4.4 Highlight


| Field   | Values                              |
| ------- | ----------------------------------- |
| target  | node, nodes, text range, none (off) |
| options | `fg`color,`bg`color                 |

Open: is `highlight` an op, a tool, or both?

## 5. GUI

### 5.1 System tray menu

list drivers, run ctrl, record ctrl, settings, doctor, help, about.

### 5.2 Settings


| Setting                | Note                     |
| ---------------------- | ------------------------ |
| log folder             |                          |
| screenshots            | on/off, storage          |
| pause on user activity |                          |
| LLM API call           | endpoint / key config    |
| JEV API call           | meaning of "jev" unclear |
| default op delay       |                          |

### 5.3 Doctor

Self-check command. Contents not defined yet.

## 6. Open questions

1. Overlap: `list_drivers` / `get_drivers` vs `list_targets` / `describe_driver`. One set of names.
2. Overlap: `run_ctrl` vs `resume_run` / `cancel_run`. One control tool or three.
3. `run_macro` vs `run_sequence`: same thing? Pick one name.
4. `run_fsm`: v2 per the summary. Keep out of v1.
5. `call llm`, `call llm jev`: the driver has no LLM inside and calls the model only for decisions (via MRTR or Tasks). Is this the decision callback? What is "jev"?
6. `approve` op: a macro can request an approval, but only the owner grants it. Confirm this.
7. `launch` spawns processes: needs an effect class (spawn), an app-id allowlist and approval.
8. `record_ctrl`: recording and replay are parked. Keep in v1?
9. Does `run_op` / `run_macro` take `driver` as a string, or per-op driver?
10. LLM API setting: for the callback only, or something else?
