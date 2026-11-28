# Fields of work: keeping the core neutral

Status: plan. Written after the owner warned of domain creep (2026-11-28): consumers ask for terms
that belong to one kind of work (coding), and the blocks must not become a bag of every field.

## Two layers

- **Core** (`blocks/`, layers 0-10): what every agentic system has, whatever its work: principals,
  delegation, models, gateways, tools, agents, harnesses, work and runs, authority, channels,
  questions, triggers, workspaces, sandboxes, runtimes, resources, architectures, KPIs,
  experiments, ports and contracts. A core block names no field's vocabulary.
- **Field** (`fields/<field>/`): one kind of work (coding, security review, documentation review,
  AI research). A field supplies data and documents, never new core semantics:
  - rows of core catalog entities: work kinds, deliverable kinds, action classes, step kinds,
    resource kinds, permission modes;
  - its KPIs, in its own catalogue, each naming the core block and events it reads;
  - its behaviour profiles (the simulator's input);
  - its corpora: scenarios taken from real work of that field, the benchmark;
  - example and optimized architectures;
  - references to the protocols that do the work (Canon `protocol/1` id and digest; the catalog
    of protocols and their runtime live in Runbook and Canon, not here);
  - **its own blocks**: ESS domains of the field's concepts, named without third parties. They
    depend on core blocks and never the reverse (the layer check holds it). Coding: repository
    (revisions, files, references), forge (hosts repositories, change requests, issues, checks),
    change request (what a forge calls a merge or pull request), issue, board and work queue, and a
    toolset of conceptual tools (read, write, edit, list, search, move, delete a file; run a
    command), each tool typed by its inputs and effects so the contract narrows `tool.invoke` to
    the tools an architecture grants. A concrete forge or harness is a binding, never a block.

## Packs between core and fields

Some field concepts are shared by several fields: a repository serves coding, documentation review
and security review; file tools serve coding, research and documentation. They live once, in a
**pack** (`packs/<pack>/`, its own namespace): `source-control` (repository, forge, change request,
check), `tracking` (issue, board, work queue), `files` (conceptual file tools), `shell` (run a
command). A field lists the packs it uses. Dependencies go one way: core, then packs, then fields; a
field never depends on another field, so a research technique and a code review never share a
bucket. `task check:layers` holds it.

## The rule

A term only one field uses is never an enum variant, entity or field of a core block. Where a
core block needs to tell such values apart, it holds a catalog entity whose rows a field
registers. The test for a new core term: would a security-review or AI-research setup need it
under the same name? If not, it is a row in a field.

## What leaks today (measured at this commit)

- `profiles/coding.yaml` is the only profile; every experiment and catalog entry uses it
  (`builds_per_task`, `build_seconds`, `roles.coding`).
- `blocks/8-place/workspace.yaml`: `branch` and a forge field (merge or pull request) on a
  proposal; a proposal is "merged".
- `blocks/4-agent/harness.yaml`: `PermissionMode` variant `AcceptEdits`, a coding harness's mode.
- `kpis/catalogue.yaml`: 8 KPIs whose text speaks of merges, builds, branches or edits (4 of
  workspace, 2 of work, 1 of verification, 1 of knowledge).
- Consumer requests this week that are field rows, not core: action classes commit, push, merge,
  tag (M19); delegated actions stop, brief, type into a session (M18); a classifier-decided
  permission mode (M27). They are being built as a core `ActionClass` catalog with no coding rows.

## Benchmarks and hill climbing per field

A benchmark is a field's corpus (real tasks with expected outcomes) run against an architecture,
a protocol per work kind and a binding. The search varies the architecture's marked places and the
protocol choice; results compare only within one field. The catalog of the docs groups its
entries by field.

## Steps

1. Rule into `AGENTS.md`; the core KPI check refuses a core KPI whose `block` is a field.
2. Format `agenticspec-field/1` and `fields/coding/`: move `profiles/coding.yaml`, the 8 KPIs,
   the coding rows of the new catalogs; the experiments and catalog name their field.
3. Make the leaks above neutral in the blocks: `branch` and the forge field become one
   `handback` reference; `AcceptEdits` becomes a registered permission mode of the coding field.
4. A second field only with a real corpus behind it; no empty field packs.
