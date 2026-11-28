# `agenticspec` CLI contract

Lift agent sessions into one IR and evaluate KPIs over them

This generated package installs an unavailable handler. Application behavior enters through the `Handler` seam; schema-selected calls also require `DynamicValidator`. See `help.txt` for process options and `binding.json` for resolved types and targets.

## Commands

### `analyze`

Find the Claude Code sessions on this machine, evaluate each, and write a dashboard file

Callable: `analyze`. Input: `agenticspec.cli.AnalyzeInput`. Result: `agenticspec.cli.AnalyzeResult`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `blocks lift`

Lift one Claude Code session into rows of the building blocks, each naming the node it came from

Callable: `blocks_lift`. Input: `agenticspec.cli.BlocksLiftInput`. Result: `agenticspec.cli.BlocksLiftResult`.

Error `rows_invalid`: `agenticspec.cli.Refusal`.

Error `transcript_unreadable`: `agenticspec.cli.Refusal`.

### `blocks scenario`

Write a draft scenario of a session's outside inputs and the block events of the turns they started

Callable: `blocks_scenario`. Input: `agenticspec.cli.BlocksScenarioInput`. Result: `agenticspec.cli.BlocksScenarioResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `map_invalid`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `scenario_refused`: `agenticspec.cli.Refusal`.

Error `transcript_unreadable`: `agenticspec.cli.Refusal`.

### `export`

Export a procedure as a canon protocol and loom bindings

Callable: `export`. Input: `agenticspec.cli.ExportInput`. Result: `agenticspec.cli.ExportResult`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `procedure_invalid`: `agenticspec.cli.Refusal`.

### `kpi`

Evaluate the KPIs over every Claude Code session in a time window

Callable: `kpi`. Input: `agenticspec.cli.KpiInput`. Result: `agenticspec.cli.KpiReport`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `lift`

Lift one Claude Code session and its sub-agents into session IR commands and replay them

Callable: `lift`. Input: `agenticspec.cli.LiftInput`. Result: `agenticspec.cli.LiftResult`.

Error `transcript_unreadable`: `agenticspec.cli.Refusal`.

### `mine`

Find runs of tool calls that recur in agent turns across sessions

Callable: `mine`. Input: `agenticspec.cli.MineInput`. Result: `agenticspec.cli.MineReport`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `procedure`

Lift a mined pattern into a procedure over its matches

Callable: `procedure`. Input: `agenticspec.cli.ProcedureInput`. Result: `agenticspec.cli.Procedure`.

Error `no_match`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `query`

Run an sq module of rules, queries and checks over one session or the sessions of a window

Callable: `query`. Input: `agenticspec.cli.QueryInput`. Result: `agenticspec.cli.QueryReport`.

Error `module_invalid`: `agenticspec.cli.Refusal`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `serve`

Serve the UI and its views from the local store on 127.0.0.1, synced every 5 seconds

Callable: `serve`. Input: `agenticspec.cli.ServeInput`. Result: `agenticspec.cli.ServeResult`.

Error `port_unavailable`: `agenticspec.cli.Refusal`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `sim check`

Check an architecture document against the building blocks and name each refusal

Callable: `sim_check`. Input: `agenticspec.cli.SimCheckInput`. Result: `agenticspec.cli.SimCheckResult`.

Error `architecture_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

### `sim corpus`

Run every entry of a task corpus on the simulator and print the predicted class beside the expected one

Callable: `sim_corpus`. Input: `agenticspec.cli.SimCorpusInput`. Result: `agenticspec.cli.SimCorpusResult`.

Error `corpus_failed`: `agenticspec.cli.Refusal`.

Error `corpus_invalid`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `scenario_invalid`: `agenticspec.cli.Refusal`.

Error `scenario_refused`: `agenticspec.cli.Refusal`.

### `sim ladder`

Check the agentic ladder and write the data file of its page, or compare it with --check

Callable: `sim_ladder`. Input: `agenticspec.cli.SimLadderInput`. Result: `agenticspec.cli.SimLadderResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `ladder_refused`: `agenticspec.cli.Refusal`.

Error `ladder_stale`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `sim render`

Write the page of an architecture, with the result of a run when given

Callable: `sim_render`. Input: `agenticspec.cli.SimRenderInput`. Result: `agenticspec.cli.SimRenderResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `sim run`

Simulate an experiment, search its marked places, and write result.json, index.html and best.yaml

Callable: `sim_run`. Input: `agenticspec.cli.SimRunInput`. Result: `agenticspec.cli.SimRunResult`.

Error `experiment_invalid`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `sim scenario`

Check a scenario against its architecture, or run it on the simulator and write scenario-result.json

Callable: `sim_scenario`. Input: `agenticspec.cli.SimScenarioInput`. Result: `agenticspec.cli.SimScenarioResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `expectation_failed`: `agenticspec.cli.Refusal`.

Error `out_missing`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `scenario_invalid`: `agenticspec.cli.Refusal`.

Error `scenario_refused`: `agenticspec.cli.Refusal`.

### `sync`

Lift the new and changed sessions of a window into the local store

Callable: `sync`. Input: `agenticspec.cli.SyncInput`. Result: `agenticspec.cli.SyncResult`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

## Runtime obligations

- handler:analyze: implement the owner-qualified callable and its declared result/error contract
- handler:blocks_lift: implement the owner-qualified callable and its declared result/error contract
- handler:blocks_scenario: implement the owner-qualified callable and its declared result/error contract
- handler:export: implement the owner-qualified callable and its declared result/error contract
- handler:kpi: implement the owner-qualified callable and its declared result/error contract
- handler:lift: implement the owner-qualified callable and its declared result/error contract
- handler:mine: implement the owner-qualified callable and its declared result/error contract
- handler:procedure: implement the owner-qualified callable and its declared result/error contract
- handler:query: implement the owner-qualified callable and its declared result/error contract
- handler:serve: implement the owner-qualified callable and its declared result/error contract
- handler:sim_check: implement the owner-qualified callable and its declared result/error contract
- handler:sim_corpus: implement the owner-qualified callable and its declared result/error contract
- handler:sim_ladder: implement the owner-qualified callable and its declared result/error contract
- handler:sim_render: implement the owner-qualified callable and its declared result/error contract
- handler:sim_run: implement the owner-qualified callable and its declared result/error contract
- handler:sim_scenario: implement the owner-qualified callable and its declared result/error contract
- handler:sync: implement the owner-qualified callable and its declared result/error contract
