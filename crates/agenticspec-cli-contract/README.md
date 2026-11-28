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

### `deploy bundle`

Admit an architecture with its binding and packing and write its deployment bundle (agenticspec-deployment/1); a refusal writes nothing and names every finding

Callable: `deploy_bundle`. Input: `agenticspec.cli.DeployBundleInput`. Result: `agenticspec.cli.DeployBundleResult`.

Error `accepts_unreadable`: `agenticspec.cli.Refusal`.

Error `architecture_refused`: `agenticspec.cli.Refusal`.

Error `binding_invalid`: `agenticspec.cli.Refusal`.

Error `contract_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `not_admitted`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `ports_unreadable`: `agenticspec.cli.Refusal`.

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

### `packing default`

Print the resolved packing section of a binding (agenticspec-binding/1) for an architecture, single by default, to paste and change

Callable: `packing_default`. Input: `agenticspec.cli.PackingDefaultInput`. Result: `agenticspec.cli.PackingDefaultResult`.

Error `architecture_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `packing_refused`: `agenticspec.cli.Refusal`.

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

Error `kpis_unreadable`: `agenticspec.cli.Refusal`.

Error `ports_unreadable`: `agenticspec.cli.Refusal`.

### `sim contract`

Derive and check the contract of an architecture (agenticspec-contract/1), check a written contract with --check, cut the blocks' conformance suite to its ports with --suite, or check its routing invariants on a trace with --trace

Callable: `sim_contract`. Input: `agenticspec.cli.SimContractInput`. Result: `agenticspec.cli.SimContractResult`.

Error `architecture_refused`: `agenticspec.cli.Refusal`.

Error `contract_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `invariant_violated`: `agenticspec.cli.Refusal`.

Error `kpis_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `ports_unreadable`: `agenticspec.cli.Refusal`.

Error `suite_unreadable`: `agenticspec.cli.Refusal`.

Error `trace_unreadable`: `agenticspec.cli.Refusal`.

### `sim corpus`

Run every entry of a task corpus on the simulator and print the predicted class beside the expected one

Callable: `sim_corpus`. Input: `agenticspec.cli.SimCorpusInput`. Result: `agenticspec.cli.SimCorpusResult`.

Error `corpus_failed`: `agenticspec.cli.Refusal`.

Error `corpus_invalid`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `scenario_invalid`: `agenticspec.cli.Refusal`.

Error `scenario_refused`: `agenticspec.cli.Refusal`.

### `sim digest`

Print the digest a document pins a nested architecture with (sha256 of its canonical parts, links and border)

Callable: `sim_digest`. Input: `agenticspec.cli.SimDigestInput`. Result: `agenticspec.cli.SimDigestResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

### `sim ladder`

Check the agentic ladder and write the data file of its page, or compare it with --check

Callable: `sim_ladder`. Input: `agenticspec.cli.SimLadderInput`. Result: `agenticspec.cli.SimLadderResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `ladder_refused`: `agenticspec.cli.Refusal`.

Error `ladder_stale`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `sim ports`

Derive the ports of the building blocks into blocks/ports.json, check its digest with --check, or list how they differ from an expected list with --expect

Callable: `sim_ports`. Input: `agenticspec.cli.SimPortsInput`. Result: `agenticspec.cli.SimPortsResult`.

Error `blocks_unreadable`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `ports_differ`: `agenticspec.cli.Refusal`.

Error `ports_refused`: `agenticspec.cli.Refusal`.

Error `ports_stale`: `agenticspec.cli.Refusal`.

### `sim render`

Write the page of an architecture, with the result of a run when given

Callable: `sim_render`. Input: `agenticspec.cli.SimRenderInput`. Result: `agenticspec.cli.SimRenderResult`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `sim run`

Simulate an experiment, search its marked places, and write result.json, index.html and best.yaml, with --events also events.jsonl

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

### `sim trace`

Simulate an architecture once at its built values and write the port uses of the run as a trace (agenticspec-trace/1)

Callable: `sim_trace`. Input: `agenticspec.cli.SimTraceInput`. Result: `agenticspec.cli.SimTraceResult`.

Error `architecture_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `simulation_refused`: `agenticspec.cli.Refusal`.

### `sync`

Lift the new and changed sessions of a window into the local store

Callable: `sync`. Input: `agenticspec.cli.SyncInput`. Result: `agenticspec.cli.SyncResult`.

Error `root_unreadable`: `agenticspec.cli.Refusal`.

Error `store_unavailable`: `agenticspec.cli.Refusal`.

### `ui conform`

Check a runtime adapter, or the fixture source, against the runtime interface of the generated UI (ui/runtime.openapi.yaml)

Callable: `ui_conform`. Input: `agenticspec.cli.UiConformInput`. Result: `agenticspec.cli.UiConformResult`.

Error `conformance_failed`: `agenticspec.cli.Refusal`.

Error `plan_unreadable`: `agenticspec.cli.Refusal`.

Error `source_unavailable`: `agenticspec.cli.Refusal`.

### `ui generate`

Generate the Vue app of an ess-ui/1 document over the components of ui/kit

Callable: `ui_generate`. Input: `agenticspec.cli.UiGenerateInput`. Result: `agenticspec.cli.UiGenerateResult`.

Error `document_refused`: `agenticspec.cli.Refusal`.

Error `document_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

### `ui panels`

Write the block panels, ui/views/blocks.ui.yaml with its fixtures and digest, from blocks/, or compare them with --check

Callable: `ui_panels`. Input: `agenticspec.cli.UiPanelsInput`. Result: `agenticspec.cli.UiPanelsResult`.

Error `blocks_unreadable`: `agenticspec.cli.Refusal`.

Error `out_unwritable`: `agenticspec.cli.Refusal`.

Error `panels_stale`: `agenticspec.cli.Refusal`.

## Runtime obligations

- handler:analyze: implement the owner-qualified callable and its declared result/error contract
- handler:blocks_lift: implement the owner-qualified callable and its declared result/error contract
- handler:blocks_scenario: implement the owner-qualified callable and its declared result/error contract
- handler:deploy_bundle: implement the owner-qualified callable and its declared result/error contract
- handler:export: implement the owner-qualified callable and its declared result/error contract
- handler:kpi: implement the owner-qualified callable and its declared result/error contract
- handler:lift: implement the owner-qualified callable and its declared result/error contract
- handler:mine: implement the owner-qualified callable and its declared result/error contract
- handler:packing_default: implement the owner-qualified callable and its declared result/error contract
- handler:procedure: implement the owner-qualified callable and its declared result/error contract
- handler:query: implement the owner-qualified callable and its declared result/error contract
- handler:serve: implement the owner-qualified callable and its declared result/error contract
- handler:sim_check: implement the owner-qualified callable and its declared result/error contract
- handler:sim_contract: implement the owner-qualified callable and its declared result/error contract
- handler:sim_corpus: implement the owner-qualified callable and its declared result/error contract
- handler:sim_digest: implement the owner-qualified callable and its declared result/error contract
- handler:sim_ladder: implement the owner-qualified callable and its declared result/error contract
- handler:sim_ports: implement the owner-qualified callable and its declared result/error contract
- handler:sim_render: implement the owner-qualified callable and its declared result/error contract
- handler:sim_run: implement the owner-qualified callable and its declared result/error contract
- handler:sim_scenario: implement the owner-qualified callable and its declared result/error contract
- handler:sim_trace: implement the owner-qualified callable and its declared result/error contract
- handler:sync: implement the owner-qualified callable and its declared result/error contract
- handler:ui_conform: implement the owner-qualified callable and its declared result/error contract
- handler:ui_generate: implement the owner-qualified callable and its declared result/error contract
- handler:ui_panels: implement the owner-qualified callable and its declared result/error contract
