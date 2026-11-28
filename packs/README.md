# Packs

Blocks more than one field of work reuses, above the core (`blocks/`) and below the fields
(`fields/`). Each pack is a directory:

- `pack.yaml`: format `agenticspec-pack/1`, the pack's name and namespace (`agentic.<word>`), the
  packs it uses, its block files and its authored scenarios;
- `blocks/`: one ESS domain per file, in the pack's namespace;
- `scenarios/`: authored `ess-scenario/7` scenarios for the paths synthesis does not reach;
- `kpis.yaml`: the pack's KPIs, format `agenticspec-kpis/1`, at least 2 per block.

| pack | namespace | uses | blocks |
|---|---|---|---|
| `source-control` | `agentic.scm` | – | forge, repository, check, change |
| `tracking` | `agentic.tracking` | `source-control` | issue, board, queue |
| `files` | `agentic.files` | – | toolset |
| `shell` | `agentic.shell` | – | toolset |

Rules (held by `task check:layers`, `task check:fields` and `task check:kpis`):

- A pack depends on the core and on the packs its `pack.yaml` lists, never on a field.
- A pack names no product: a forge, tracker or harness is a binding of these blocks.
- A new block goes into `pack.yaml`, the root `ess-inputs.yaml` and `packs/system.yaml` together.
- A refusal synthesis cannot reach gets an authored scenario.

`system.yaml` declares the whole `agentic` system (the core domains and the pack domains) for the
root `ess-inputs.yaml`. `known-synthesis-defects.txt` lists the synthesized scenarios ESS's
interpreter fails because synthesis arranged them wrong, each with the authored scenario that walks
its path.
