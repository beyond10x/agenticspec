# `agentspec` CLI contract

Lift agent sessions into one IR and evaluate KPIs over them

This generated package installs an unavailable handler. Application behavior enters through the `Handler` seam; schema-selected calls also require `DynamicValidator`. See `help.txt` for process options and `binding.json` for resolved types and targets.

## Commands

### `analyze`

Find the Claude Code sessions on this machine, evaluate each, and write a dashboard file

Callable: `analyze`. Input: `agentspec.cli.AnalyzeInput`. Result: `agentspec.cli.AnalyzeResult`.

Error `out_unwritable`: `agentspec.cli.Refusal`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `export`

Export a procedure as a canon protocol and loom bindings

Callable: `export`. Input: `agentspec.cli.ExportInput`. Result: `agentspec.cli.ExportResult`.

Error `out_unwritable`: `agentspec.cli.Refusal`.

Error `procedure_invalid`: `agentspec.cli.Refusal`.

### `kpi`

Evaluate the KPIs over every Claude Code session in a time window

Callable: `kpi`. Input: `agentspec.cli.KpiInput`. Result: `agentspec.cli.KpiReport`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `lift`

Lift one Claude Code session and its sub-agents into session IR commands and replay them

Callable: `lift`. Input: `agentspec.cli.LiftInput`. Result: `agentspec.cli.LiftResult`.

Error `transcript_unreadable`: `agentspec.cli.Refusal`.

### `mine`

Find runs of tool calls that recur in agent turns across sessions

Callable: `mine`. Input: `agentspec.cli.MineInput`. Result: `agentspec.cli.MineReport`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `procedure`

Lift a mined pattern into a procedure over its matches

Callable: `procedure`. Input: `agentspec.cli.ProcedureInput`. Result: `agentspec.cli.Procedure`.

Error `no_match`: `agentspec.cli.Refusal`.

Error `out_unwritable`: `agentspec.cli.Refusal`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `query`

Run an sq module of rules, queries and checks over one session or the sessions of a window

Callable: `query`. Input: `agentspec.cli.QueryInput`. Result: `agentspec.cli.QueryReport`.

Error `module_invalid`: `agentspec.cli.Refusal`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `serve`

Serve the UI and its views from the local store on 127.0.0.1, synced every 5 seconds

Callable: `serve`. Input: `agentspec.cli.ServeInput`. Result: `agentspec.cli.ServeResult`.

Error `port_unavailable`: `agentspec.cli.Refusal`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

### `sync`

Lift the new and changed sessions of a window into the local store

Callable: `sync`. Input: `agentspec.cli.SyncInput`. Result: `agentspec.cli.SyncResult`.

Error `root_unreadable`: `agentspec.cli.Refusal`.

Error `store_unavailable`: `agentspec.cli.Refusal`.

## Runtime obligations

- handler:analyze: implement the owner-qualified callable and its declared result/error contract
- handler:export: implement the owner-qualified callable and its declared result/error contract
- handler:kpi: implement the owner-qualified callable and its declared result/error contract
- handler:lift: implement the owner-qualified callable and its declared result/error contract
- handler:mine: implement the owner-qualified callable and its declared result/error contract
- handler:procedure: implement the owner-qualified callable and its declared result/error contract
- handler:query: implement the owner-qualified callable and its declared result/error contract
- handler:serve: implement the owner-qualified callable and its declared result/error contract
- handler:sync: implement the owner-qualified callable and its declared result/error contract
