# Baseline: real tasks and kill criteria

Purpose: measure a plain step-by-step agent before building. No benchmark exists yet.

## Method

1. Run each task with a plain agent (no handsfree). Run each about 3 times, GUI results vary.
2. Record the metrics below.
3. Later, run the same tasks through handsfree macros. Compare.

## Metrics per run


| Metric      | How                                    |
| ----------- | -------------------------------------- |
| Cost        | tokens / dollars                       |
| Latency     | wall-clock seconds                     |
| Success     | pass / fail against the postcondition  |
| Zero-replan | finished without the model re-planning |

## Tasks

Placeholders. Replace with your real workloads. Keep 2 to 3 unstable tasks and 1 control task.


| #  | Task                                                    | Driver       | Postcondition                      | Kind     |
| -- | ------------------------------------------------------- | ------------ | ---------------------------------- | -------- |
| 1  | Open gedit, type a note, save as`~/notes/x.txt`         | atspi        | file exists, content matches       | stable   |
| 2  | Save over an existing file, cancel the overwrite dialog | atspi        | original file unchanged            | stable   |
| 3  | Rename 10 files via the file manager                    | atspi        | `ls`shows new names                | stable   |
| 4  | Change a system setting (dark mode), verify             | atspi        | setting readback matches           | stable   |
| 5  | Log into a test web app, open the settings page         | chromium     | URL and page title match           | stable   |
| 6  | Fill a 10-field form from JSON, submit                  | chromium     | server receives expected payload   | stable   |
| 7  | Download a report, check the filename                   | chromium     | file in download dir               | stable   |
| 8  | Scrape a 50-row table into CSV                          | chromium     | row count and checksum match       | stable   |
| 9  | Call 5 tools of an existing MCP server in fixed order   | mcp\_wrapper | final state matches                | stable   |
| 10 | Delete a file via a dialog that needs approval          | atspi        | approval requested, then file gone | policy   |
| 11 | Task 5 flow, but a popup appears mid-run                | chromium     | recovers or fails safely           | unstable |
| 12 | Trivial 2-step action                                   | any          | done                               | control  |

Extend to about 20.

## Results


| # | Cost | Latency (s) | Success (n/3) | Zero-replan (n/3) |
| - | ---- | ----------- | ------------- | ----------------- |
| 1 |      |             |               |                   |
| 2 |      |             |               |                   |

## Kill criteria (set before measuring)

Fill in numbers now, not after seeing results.

1. Stop or pivot if handsfree is not at least \_\_\_ % cheaper than baseline on stable tasks.
2. Stop or pivot if success rate drops below \_\_\_ %.
3. Stop or pivot if zero-replan completion is below \_\_\_ % on stable tasks.
4. Stop or pivot if the guarded layer adds more than \_\_\_ s latency per op.
