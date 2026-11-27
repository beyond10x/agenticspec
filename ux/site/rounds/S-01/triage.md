# S-01 triage

Inputs: `report.md`, `review-heuristics.md` (7 pass, 8 partial, 1 fail), `review-report.md` (not
fit to send as is; every number reproduces, three mean less than their names).

## Caveats

- Measured-viewport completion (18 of 36) is mostly one bot defect at 768x1024 (T3): ten runs
  refuse menu links the screenshot shows on screen. Only 6 failures are visible to a reader.
- "35 distinct problems" counts one defect again per box size (T6); 12 without them.
- axe ran at 1440x900 only in the sweep (T2). "No axe violation" holds for that size only.
- The baseline was written from this round; the `(base N)` columns cannot differ (T8).
- SUF-001 was checked at 1440x900 only; SUF-006 names the wrong cause at 1024x768 (the table
  is taller than the viewport, it does not scroll sideways there); SUF-010's "10 of 12" is 9 of
  12.

## Did the last change sets move their metrics?

First round: none.

## Priority

`priority = severity weight × frequency × confidence ÷ cost` (S2 = 3, S3 = 1; frequency =
scenarios or swept pages affected ÷ total; confidence 2 when a metric and a reviewer agree).

| finding | sev | freq | conf | cost | priority |
|---|---|---|---|---|---|
| SUF-001 frame shows the landing page | S2 | 5/39 pages, 1/6 tasks | 2 | S | 0.77 |
| SUF-002 best variant buried, no contents | S2 | 1/6 tasks, 5/39 pages | 2 | M | 0.39 |
| SUF-008 scroll box not keyboard reachable | S2 | 8/39 pages at 390 | 1 | S | 0.62 |
| SUF-005 contents list after every link (87 presses) | S3 | 6/6 keyboard tasks | 1 | S | 1.00 |
| SUF-013 key data under 14 px | S3 | 39/39 pages | 2 | S | 2.00 |
| SUF-007 tables and diagrams scroll sideways below 1024 | S3 | 29/39 at 390 | 1 | M | 0.37 |
| SUF-012 navbar "KPIs" is not the catalogue | S3 | 1/6 tasks | 1 | S | 0.17 |
| SUF-014 map cards cut descriptions, "ent/cmd/evt" | S3 | 1/39 pages | 1 | S | 0.03 |
| SUF-003, 004, 006, 009, 010, 011 | S3 | | 1 | S–M | below 0.3 |

No S1 finding.

## Change set (5)

1. SUF-001: the architecture frame shows the named architecture under `docusaurus serve` and on
   GitHub Pages (the `.html` redirect drops `/agenticspec/`). Target: the five page-sweep frame
   tests pass; `best-variant-of-author-reviewer` actions.
2. SUF-002: the architecture page puts the search result (best variant against baseline) above
   the frame, with a contents list. Target: `screen_only.actions` of
   `best-variant-of-author-reviewer` from 9–10 to at most 3.
3. SUF-008: every scrolling box (lifecycle diagrams, wide tables) is a focusable named region.
   Target: axe `scrollable-region-focusable` 0 at 390x844.
4. SUF-005: the page contents list comes before the article links in keyboard order (or a skip
   link to it). Target: key presses to the harness KPIs from 87 to at most 20.
5. SUF-013: KPI markers, table heads and cells at 14 px or more. Target: the font-size check of
   SUF-013 passes; `reading_load` unchanged or lower.

## Tool fixes beside the change set

They change what is measured, not the site, so S-02 records them beside the numbers: T2 (axe at
every gating and measured viewport in the sweep), T3 (`reachable()` accepts links on screen; the
message), T4 (a target taller than the viewport counts glance 1 when its top is in view), T5 (an
aborted run reports no reading load or glance instead of 0), T6 (the problem key without box
sizes), T9 (`globalTimeout` named for the measure project). Findings SUF-001, 006 and 010 get
their corrected text.

The rest wait for S-02: SUF-007 (layout below 1024) leads it.
