# agenticstudio

agenticstudio opens an architecture in your browser as a draft, changes it with patches and shows
the result: the graph, the parts and the structural check. A patch that would break the
architecture is refused as a whole and changes nothing.

The page is the app of `ui/studio`, generated from the `ess-ui/1` document `studio/studio.ui.yaml`
by `agenticspec ui generate` and drawn with the design system of `ui/kit`.

## Run

From a checkout of the repository, build the app once, then start the server:

```console
task studio:build
cargo run --release -p agenticstudio -- serve --root . --port 7431
```

| flag | what |
|---|---|
| `--root <dir>` | the directory whose `architectures/` holds the documents you open; default `.` |
| `--port <n>` | the port on `127.0.0.1`; default `7431`, `0` takes a free port |
| `--www <dir>` | the built app; default `<root>/target/studio/www`, which `task studio:build` writes |

The server listens on `127.0.0.1` only and prints its address. Drafts live in its memory: they are
gone when it stops.

## Open a draft

Open `http://127.0.0.1:7431/`, choose an architecture in the header and press **Open draft**. The
draft starts at revision 1 of that document. Its page has the graph on the left and a narrow panel
on the right:

- the graph of the current revision fills the page. Drag to pan; the wheel or a pinch zooms about
  the pointer; the buttons zoom in and out, fit the whole graph or go to 100 %. It opens at a
  readable scale and is fitted only when it is small. Click a part (or Tab to it and press Enter)
  to see its properties and links; Escape clears the selection;
- **Patch**: the patch editor, **Dry run**, **Apply** and the result or refusal;
- **Draft**: the revision, the first 12 digits of its digest (the SHA-256 of the document) with a
  copy button, the check as a badge, and the description in two lines with **more**;
- **Check**: each refusal of the check;
- **Parts**: the parts with their block and count, with a search; closed until you open it.

The theme switch in the header sets light or dark for the whole page.

## Apply a patch

Write the patch in the **Patch** box and press **Apply**. Applied, the draft moves to the next
revision and the page lists what changed. With **Dry run** on, the page shows what would change,
and the draft stays where it is.

```text
set(P, count, 3) :- part{id: P, block: "agent", count: 12}.
```

This sets the count of every agent part whose count is 12 to 3.

A patch is a list of rules, each ending with `.`. A rule is a transform, then `:-` and a selector:
for every answer of the selector, the transform runs once. The rules run in order, and each
selects on what the rules before it left. Then the check runs on the result. If any rule or the
check refuses, nothing changes, and the page names the rule by its position (counted from 1) and
the error.

### Selectors

A selector is written in the query language of [Queries](queries.md), over the facts of the
architecture:

| relation | fields | holds |
|---|---|---|
| `part` | `id, block, count` | each part: its name, its block, its count (1 when the document gives none) |
| `setting` | `part, name, value` | each other key of a part, such as `harness`, `role`, `acts_for`, `capacity`; a list gives one fact per element |
| `link` | `index, from, to, carries, over` | each link, numbered from 0 in document order; `over` is null when absent |
| `marker` | `path, value` | each marked place, such as `parts.worker.count`, with its value |

The value of a marked place is its `value`. A duration such as `10m` is held in milliseconds and
compares with duration literals: `V > 5m`. A number with a fraction is held as its text.

### Transforms

| transform | what it does |
|---|---|
| `set(P, field, value)` | sets a field of part `P` (a part name), or of link `P` when `P` is a link index. On a marked place it sets the value and keeps the range. A part's `id` and a link's `index` are fixed |
| `add(part{id: "x", block: "agent", ...})` | adds a part; `id` and `block` are required, every other field becomes a setting |
| `add(link{from: A, to: B, carries: "c", over: C})` | adds a link at the end of the list; `over` is optional |

A value is a string, a whole number, `true`, `false`, a duration such as `5m`, or a variable the
selector binds. A transform without a selector runs once: `add(part{id: "spare", block:
"model"}).`

```text
add(part{id: "reviewer", block: "agent", harness: "coding", role: "reviewer"}).
add(link{from: "reviewer", to: "gateway", carries: "model_call"}).
add(link{from: W, to: "reviewer", carries: "review"}) :- part{id: W, block: "agent", count: 12}.
```

The verbs `remove`, `connect`, `replicate`, `wrap`, `prepend`, `append`, `mark` and `unmark` are
part of the patch language and not applied yet.

### Errors

| error | the patch is refused because |
|---|---|
| `EmptyPatch` | it has no rule |
| `UnknownFormat` | a patch document does not name `format: agenticspec-patch/1` |
| `UnparsableRule` | a rule does not end with `.`, or its transform or selector does not parse |
| `UnknownVerb` | a transform's verb is not a verb of the patch language |
| `UnsupportedVerb` | a transform's verb is one that is not applied yet |
| `InvalidHead` | a transform's arguments do not fit its verb |
| `InvalidSelector` | a selector names a relation or field an architecture does not have |
| `UnboundVariable` | a variable of the transform is bound by nothing in the selector |
| `NoSuchPart` | `set` names a part that does not exist |
| `NoSuchLink` | `set` names a link index that does not exist |
| `ReadOnlyField` | `set` writes a part's `id` or a link's `index` |
| `UnknownField` | a link has no such field |
| `MissingField` | `add` leaves out `id` or `block` of a part, or `from`, `to` or `carries` of a link |
| `InvalidCount` | a count is not a whole number of at least 1 |
| `ConflictingValues` | one rule gives one place two different values |
| `PartNameTaken` | `add` names a part that exists |
| `LinkExists` | `add` names a link that exists |
| `CheckRefused` | the result fails the structural check; the refusals are listed, and no rule is named |

## Endpoints

The app reads the views and sends the commands its document names; each answers JSON:

| method and path | what |
|---|---|
| `GET /api/views/studio.Architectures` | `{"rows": [{"name", "source"}]}`: the documents under `architectures/` |
| `GET /api/views/studio.draft.Drafts` | the drafts |
| `GET /api/views/studio.DraftSummary?draft_id=<id>` | name, revision, digest, `passed` and the description |
| `GET /api/views/studio.DraftParts?draft_id=<id>` | each part: `id`, `block`, `count` |
| `GET /api/views/studio.DraftRefusals?draft_id=<id>` | each refusal of the check: `rule`, `part`, `message` |
| `GET /api/views/studio.DraftGraph?draft_id=<id>` | the graph: its SVG, size, parts with their properties, and links |
| `POST /api/commands/studio.draft.OpenDraft` | `{"source"}` opens a draft; answers `201` with `{"row": <draft>}` |
| `POST /api/commands/studio.session.CallTool` | `{"draft_id", "arguments": "<rules>", "dry_run"}` applies a patch; answers `{"row": {"outcome": "Applied" or "DryRun", "message", "diff"}}`, or `422` with `{"row": {"outcome": "Refused", "rule", "error", "message"}}` |

A script can also use these endpoints:

| method and path | what |
|---|---|
| `GET /api/architectures` | the documents under `architectures/` |
| `POST /api/drafts` | opens a draft: `{"source": "architectures/workstation.yaml"}`, optionally with a `name` |
| `GET /api/drafts` | the drafts |
| `GET /api/drafts/<id>` | a draft: its document, its revision with number and digest, its parts, its check and its patch applications |
| `GET /api/drafts/<id>/graph` | the graph page of its current revision (HTML) |
| `POST /api/drafts/<id>/patches` | applies a patch: `{"text": "<rules>", "dry_run": false}`, or a patch document |

A patch document names its format and lists its rules:

```json
{
  "format": "agenticspec-patch/1",
  "name": "fewer workers",
  "rules": ["set(P, count, 3) :- part{id: P, block: \"agent\", count: 12}."]
}
```

An applied patch answers `200`, a refused one `422` with `refusal` holding the rule's position
(`null` when the check refused), the error and a message.
