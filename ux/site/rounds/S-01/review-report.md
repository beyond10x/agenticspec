# S-01 review: report reviewer

Verdict: not fit to send as is. Default action: send after three edits (the measured-viewport completion headline, SUF-006's cause at 1024x768, SUF-010's "10 of 12 runs") and the caveats in § 4. Every headline number reproduces; three of them mean less than their names say.

Method: recomputed from `target/site-ux/measure/*.json` (60 task runs) and `measure/pages/*.json` (78 sweep files) with jq (`target/site-review-r/recompute.sh`, `logmetrics.sh`), from the bot logs, and by rerunning `SITE_UX_ROUND=S-99 slot --here --timeout 15m npx playwright test --project=measure --workers=2` from website/ with the axe cache emptied first. The rerun took 6.9 min, 138 tests, slowest 22.1 s. All 60 task files and all 78 sweep files are byte-identical to the round's, and the axe cache rebuilt to the same content. So the measure is deterministic on this build. (A first rerun without `--here` ran on the remote build node and wrote nothing locally; it proved nothing and was discarded.)

## 1. Are the numbers right and meaningful?

Numbers that differ from the round's: none.

| number | round | recomputed |
|---|---|---|
| gating runs pass / complete / clean | 24/24, 24/24, 24/24 | same |
| measured pass / complete / clean | 2/36, 18/36, 2/36 | same |
| distinct problems | 35 | 35 (see below: 12 when sizes are dropped) |
| sweep pages | 39 | 39 |
| sweep layout, light: 1024x768 / 768x1024 / 390x844 / 1440x900 / 1280x800 | 15 / 1 / 29 / none / none | same |
| sweep axe, console errors | none, none | same |
| frame problems | 10 page-theme rows (5 pages x 2 themes) | same |
| trend: actions / looks / reading load max (sums, light, gating) | 61 / 71 / 4405 | 61 / 71 / 4405 |
| actions, looks, glance median per gating run | in report | equal to the counts rebuilt from each run's `log` |
| layer map / table counts in SUF-007, SUF-008 | 15 pages, 29 of 39, lifecycle on 8 | same (8 pages with `as-life__frame` at 390x844) |
| key presses in SUF-005 | 87, 72, 54, 47, 17, 8 | same, from check-site.log |

Meaning, metric by metric:

- Gating pass 24/24 means "task done and no layout or axe problem at the gating widths". It does not cover the embedded frame: the best-variant task completes "right" while the frame shows the landing page (SUF-001, S2). The frame is only checked in the sweep and in `pages.spec.ts`, at 1440x900.
- Measured completion 18/36 is not "18 runs the page fails". Of the 18 failures: 10 are one cause at 768x1024 (the bot refuses the menu links, see T3; shot `shots/find-harness-kpis-768x1024-dark-02.jpg` shows a readable, usable menu); 2 are gate-kpis at 1024x768 where the target is taller than the viewport (T4); 6 are at 390x844 (best-variant, layer-of-workspace, gate-kpis, both themes), where the content really is cut or scrolls sideways. Read as "6 of 36 runs fail on what the reader sees".
- "35 distinct problems" counts the same defect again per width, because the width pair `(633 > 358)` is part of the key. With the sizes dropped it is 12 distinct page-and-message pairs (T6).
- Axe: "none" in the sweep and `axe rules (sweep) 0` in the trend are axe at 1440x900 only (T2). At 390x844 axe found `scrollable-region-focusable` on the 3 harness states the bots reached; the other 7 block pages with a sideways-scrolling lifecycle at 390x844 were never scanned.
- Reading load, actions, looks, glance: right for the 24 gating runs. For the measured runs that abort, the numbers read as best: 768x1024, 10 runs, `reading_load.max` 0, actions 1, glance median 0 (T5). They are not in the trend or the ratchet, but they are in the per-run tables of `round.json`.
- `layout_shift.score` is 0 in all 60 runs. I did not verify that the observer sees the frame's late load; treat as "not seen to shift", not "proved zero".
- `(base N)` columns equal the round in every cell, because the baseline was written from this same site at 18:15 by `SITE_UX_RATCHET=write` (T8). They show the ratchet is wired, not that nothing moved.

## 2. Findings the report misses

- SUF-006 names the wrong cause at 1024x768. Per-moment problems of `gate-kpis-of-authority-1024x768-light.json`: sideways scroll at `start` (918 > 690) and after the Block list (802 > 690), none after the gate chip and none at `completion`. A browser probe of the filtered state at 1024x768 gives box scrollWidth 690 = clientWidth 690, all seven columns inside 316..1007 px. The shot `evidence-SUF-006-kpi-table-1024x768-light.jpg` shows all seven columns. The run ends at glance 2 because the three-row tbody, with its long "How:" text wrapped in a 195 px KPI column, is taller than a 768 px viewport (it fits at 800 and 1024). At 390x844 the sideways scroll is real at completion (688 > 356). The finding's "right columns past the box edge" holds for 390 and for the unfiltered table only.
- SUF-010 says "10 of 12 runs end on their start page". Not so: best-variant ends on /docs, the others on /. Also the bot's line "the screen shows no the link Building blocks" is false: the link is on the screen. The defect (panel 637 px in a 352 px drawer, title ellipsis) is real, S3, and the drawer is usable by a person; the headline cost is the bot's clipping rule.
- SUF-001 says the wrong frame is seen "in every viewport". The frame check runs at 1440x900 only (both themes). The task runs at other widths do not check it.
- SUF-002 and the baseline: the 9 and 10 actions of best-variant were measured with the landing page in the frame (finding's own caveat). Once SUF-001 is fixed the frame height changes, so the ratchet value for that task will move for a reason unrelated to the contents list.
- One defect class is spread over 29 of 39 pages at 390x844, 130 sideways-scroll lines in all. SUF-007 reports the pages; the report's table shows only the first 3 messages per page.

## 3. Tool defects

- T1. `website/e2e/layout.ts` `axeOnce`: the cache key is path, viewport and theme, plus `end-<scenario>` only at completion (`measure.spec.ts` `moment()`, `gates.spec.ts` `check()`). Axe therefore does not run "at every moment" as the report's Levels line says. The first state that reaches a path decides for all later states of it. The open menu at 390x844 and 768x1024 (path `/` after "Toggle navigation bar") is never scanned. The cache is never pruned: `target/site-ux/axe/<digest>/390x844-*/` held 3 `_docs_blocks_end-*.json` files per theme that this build's runs no longer produce (they appear in the original cache and not in the rebuilt one).
- T2. `measure.spec.ts`, sweep test: `axeOnce` is called once, at `adapter.gating_viewports[0]` (1440x900), before the viewport loop. `report.spec.ts` prints "axe-core, serious and critical, by rule: none" and the trend row `axe rules (sweep) 0` without the width. Input: `measure/pages/*.json` all have `axe: []`; `axe/<digest>/390x844-light/_docs_blocks_4-agent_harness.json` has the violation.
- T3. `website/e2e/bot.ts` `reachable()` (used by `pick()`): rejects a link whose box runs past an ancestor with overflow (`r.right > b.right`), even when its text is on screen. Input: `find-harness-kpis-768x1024-light.json`, log `["open Toggle navigation bar"]`, `completion.error` "the screen shows no the link Building blocks in the navbar". The message is built as `the screen shows no ${what}` where `what` already starts with "the", so it reads "no the link". It also states that nothing is on screen, which is false here.
- T4. `bot.ts` `where()` with `scenarios.ts` target kind `kpi-rows` (`.as-kpis tbody`): a target taller than the viewport can never reach glance 1, and `reach()` stops scrolling once the top is aligned. Input: `gate-kpis-of-authority-1024x768-light.json` (completion seen 2, no problem at completion); the same task passes at 1280x800 and 768x1024.
- T5. `bot.ts` `metrics()`: a run that throws before its first look returns `Math.max(0, ...[])` = 0 reading load and glance median 0, with 1 action. Input: the ten 768x1024 runs other than gate-kpis. "Lower is better" ranks a failed run above a passed one. Return null and leave aborted runs out of the tables and sums.
- T6. `report.spec.ts` `problems` map: the key includes the box sizes. Input: round.json `problems` has 35 keys, 12 without the `(a > b)` part. The per-run column "distinct problems" uses the same key.
- T7. `report.spec.ts` header: "commit eec2f8c with uncommitted changes". The commit is the 0.3.0 release; the whole building-blocks site (docs/architectures, blocks/, crates/agenticspec-*) is untracked, so the hash pins nothing. Record `target/site-ux/build.digest` (or a digest of the build's HTML) in the report. The date comes from `toISOString()` (UTC), not the host's date.
- T8. `ratchet.spec.ts`: on a first round `SITE_UX_RATCHET=write` takes the baseline from the same measurement, so the `(base N)` column cannot differ. Say so in the report on a first round, or print "baseline = this round".
- T9. `playwright.config.ts`: with `SITE_UX_ROUND` unset `globalTimeout` is 120 s, so `--project=measure` alone is cut at 2 min; the measure takes 6.9 min at 2 workers (2.7 min at 4 workers in round.log). Not a test over 60 s (slowest 22.1 s), but the step is over the 120 s budget; it is round-only, so the note belongs in the README of the round.
- T10. Running the measure through `slot` without `--here` (the form in the review brief) syncs the tree to a build node and discards the outputs; `round.sh` and `check-site.sh` use `--here`. No change needed in the tools; say it in the round README.

## 4. Caveats to send with the round

- All numbers are from a build of an uncommitted tree; the commit hash does not identify it.
- Gating is 24/24 and the page's embedded architecture frame is wrong in all 5 pages (SUF-001, S2). The gates do not fail on it (marked expected-fail in `debt.ts`).
- Measured viewports: 6 of the 18 failed completions are on-screen problems (390x844); 10 are the one drawer defect plus a bot rule; 2 are a tall table at 768 px height.
- Axe was run at 1440x900 for the sweep and on the first state of each path in the tasks: no other widths, no open menu.
- No survey, no recordings of people, no probe, no previous round: nothing here is a trend.

## 5. Decision

No decision this round.
