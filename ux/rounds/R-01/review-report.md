# R-01 review: report reviewer

Verdict: not fit to send as is. Every number in `report.md` recomputes exactly from the 50 raw run
files, but the report says two levels ran that did not run, its "17 distinct problems" are about 6
defects, and it gives no cause for the only 2 wrong completions. Default action: send the round with
§ 4 attached and without a re-run, and fix T1 to T4 before R-02.

Sources: `target/ux/measure/<scenario>-<WxH>-<theme>.json` (50 files, written 06:10 to 06:11 by the
measure project of `ux/round.sh`), `target/ux/measure/shots/*.jpg`, `target/ux/round-R-01.txt` (log
of the round command), `target/ux/run1.txt` (the last gates run, from before the round's build), the
tool source under `ui/app/e2e/`, `ux/adapter.json` and `ux/scenarios/*/scenario.json`. `round.json`
was not used. Measure was not re-run: the recomputation did not need it. Source line numbers refer
to commit 302e2fe. The working tree has changed since then.

## 1. Are the numbers right and meaningful?

Recomputed with `jq` from the raw files:

| number in report.md | report | recomputed | differs |
|---|---|---|---|
| runs | 20 gating + 30 measured | 50 files, 20 + 30 | no |
| gating runs that pass every gate | 0 of 20 | 0 of 20 | no |
| measured runs that pass every gate | 2 of 30 | 2 of 30 (find-session-by-search and slice-kpi-averages, both 1024x768 dark) | no |
| completion and right, problems per run (50 table rows) | as printed | the same 50 rows | no |
| problems seen (17 lines, with where each was seen) | as printed | the same 17 lines | no |
| metric table, light theme, gating (10 rows x 9 keys) | as printed | the same; also equal to the gates run's `target/ux/metrics/*.json` | no |
| trend: distinct problems, actions, looks, reading load max (sums) | 17, 13, 21, 1634 | 17, 13, 21, 1634 | no |
| trend: open S1/S2/S3 | 0/0/0 | 0/0/0 when written; `ux/findings/` has held records since 06:12 (5 then, more since) | stale, see § 4 |

No number differs. Whether each one means what its name says:

- **"0 of 20 runs pass every gate"** combines two results. Completion: 20 of 20 gating runs end with
  the target fully in view and right (gate 3 holds). Problems: 0 of 20 runs are free of problems,
  all of them axe-core or cut-off text (gates 1, 2 and 5). The headline reads like a completion
  failure, and it is not one.
- **"pass every gate"** does not include the keyboard half of gate 5: the round does not run
  `keyboard.spec.ts`. Gate 1 is the docs-site version (no sideways scroll, vertical page scroll
  allowed), not the app version of SKILL.md § 4, and `ux/adapter.json` does not record that choice.
  Under the app version every run that opens a session page fails gate 1.
- **"distinct problems: 17"** counts lines keyed on rule plus the first node's selector
  (T2). The 17 lines are about 6 defects:
  1. `aria-pressed` on `role="radio"` chips (`ui/app/src/components/EventLog.vue:160-162`). This is
     2 lines (`.chip[role="radio"]` on the session page, `button:nth-child(1)` on the embed page,
     10 nodes each).
  2. `role="button"` lane rects inside the `role="img"` timeline svg
     (`ui/app/src/components/Timeline.vue:229`, `:320-323`). This is 7 lines, one for each
     `svg[width=…]`, because the svg width changes with the viewport.
  3. Light-theme muted text fails color-contrast on 16 to 41 nodes per screen. This is 3 lines
     (`.hero-note` and two tile `.line.muted` selectors).
  4. Dark theme: the muted text of the selected agent row fails contrast (1 node). This is 1 line.
  5. A cut-off tile note on the overview: "of input tokens read from cache" at 1280 and wider and at
     390, and "per session · 25.0% refused" at 390. This is 2 lines.
  6. Cut-off event-log cells at 390 ("Refuse · Classifier", "by parent agent"). This is 2 lines.
- **The "problems" column** counts problem x moment. A run with more bot steps, or one that starts on
  the overview and then opens a session, scores higher for the same defects. For example,
  agent-lane 390x844 light = 2 + 5 + 5 + 5 + 5 = 22, which is 7 distinct lines. Light is higher than
  dark only because of item 3. Do not compare it across scenarios.
- **"(10: …)" in Problems seen** counts viewport x theme pairs, not runs or scenarios.
- **`screen_only.steers`** is 0 because nothing in `ui/app/e2e/bot.ts` ever increments it (T6). It
  carries no information.
- **`glance_cost`**: the median is 1 in all 50 runs. The p95 is taken over 1 to 4 looks, so it equals
  the maximum. The bot's glance means "fully in view and uncovered / partly / out of view", not the
  SKILL.md § 7 meaning (primary card / elsewhere in view / not in view).
- **`reading_load`** counts the words on the screen at each look. It grows with the viewport
  (agent-lane: 139 at 1280x720, 194 at 1440x900, 210 at 1920x1080), so compare it only within one
  viewport. find-session-by-search and slice-kpi-averages both read 139: the overview page, not the
  task.
- **`layout_shift.score`** is the cumulative shift of the whole run, page load included
  (`buffered: true`). The SKILL.md § 6 key is `layout_shift.per_minute`. Light and dark values are
  equal to 4 decimals in every pair, so the measurement is deterministic.
- **slice-kpi-averages** cannot fail on data. Its truth (Sessions 3, Cost per session $0.32) is the
  same as the 72 h default on the start screen (`target/ux/measure/shots/find-session-by-search-1440x900-light-00.jpg`),
  because all 3 fixture sessions fall inside 24 h. Only `aria-pressed` tells the two states apart
  (T7).
- **Comparability**: this is the first round, with no baseline, no last round and no people. Nothing
  is a trend yet.

## 2. Findings the report misses

1. **agent-lane at 390x844 (both themes): the lane's cost is not shown, and its label runs past its
   box.** The report prints "right: no" and gives no reason. `Timeline.vue:328` drops the lane meta
   (`v-if="!narrow"`, narrow below 640 px), so "$0.07" is not in the timeline and the card's
   expectation "its cost is shown beside its name" fails. The label "Sub-agent 1" runs past its
   78 px rect (`target/ux/measure/shots/agent-lane-390x844-light-04.jpg`, bottom left). No check
   caught that (T5).
2. **failing-check: the failed check is below the fold at a gating viewport.** At 1280x720 the bot
   saw the target partly (glance 2) and scrolled once (`failing-check-1280x720-light-01.jpg`: "Cache
   bust after idle" on the bottom edge). At 1024x768 and 390x844 it was out of view (glance 3). The
   report shows "glance 1" because it looks only at the end state.
3. **embed-session shifts its layout with no user action**: 0.13 at 1920x1080, 0.14 at 1440x900,
   0.11 at 1280x720, 0.09 at 1024x768 and 0.06 at 390x844. Every other scenario stays at 0.01 or
   below. Three of these are above the 0.1 "good" bar of web vitals. The value is in the metric table
   and is not named as a problem. Cause not determined.
4. **Copy: "1 secret exposures"** (`ui/app/src/pages/SessionPage.vue:115`). It is visible in every
   session and embed frame of "Port the store", for example `embed-session-1440x900-dark-01.jpg`.
5. **Size of the light-theme contrast failure**: up to 41 failing nodes per screen on the session
   and embed pages, and 16 to 20 on the overview. The report shows "n nodes" (T2), which hides that
   this is all muted text, not one element.
6. **Keyboard gate**: the last gates log on disk (`target/ux/run1.txt`, before the round's build)
   has "find-session-by-search by keyboard" failing "the target shows the truth". The round does not
   run the keyboard tests, so this build has not been checked. It stays open until the gates
   project runs on the same build.

## 3. Tool defects

- **T1** `ui/app/e2e/report.spec.ts`, the Levels lines, are hard-coded. They say L1 (vitest) and L7
  ran, but `ux/round.sh` runs only `scripts/ui-build.sh` (vue-tsc, vite) and the report project. The
  report was written at 06:11, before any review agent. Input: `target/ux/round-R-01.txt`, which has
  no vitest step.
- **T2** `report.spec.ts` (the problems map) and `ui/app/e2e/layout.ts` (`axe()`) key a problem on
  rule plus `nodes[0].target` and replace the node count with "n nodes". As a result, viewport-
  dependent attributes split one defect into several lines and the node counts are lost. Input: 7
  lines of `nested-interactive … svg[width="…"]`, and color-contrast with 41 nodes printed as "n
  nodes".
- **T3** `report.spec.ts`, the "problems" column, sums problem x moment and labels the sum as if it
  were a count of problems. Input: `agent-lane-390x844-light.json` gives 22 from 7 distinct lines.
- **T4** `report.spec.ts`: `ok()` treats completion and problems as one result. The worst-moments
  list adds +10 to any run with any problem, so a false completion (agent-lane 390x844, right =
  false) weighs the same as a contrast miss. Measured-only runs are labelled "run fails a gate" under
  a heading that says "Not gated". Input: the worst-moments list lines 112 to 114.
- **T5** `layout.ts` `contract()` skips every SVG descendant, so text clipped inside the timeline svg
  is never reported. Input: `agent-lane-390x844-light-04.jpg`, the lane label.
- **T6** `ui/app/e2e/bot.ts`: `steers` is never incremented. `screen_only.steers` is a constant 0
  and is still a ratchet key. Input: all 50 run files.
- **T7** `ux/scenarios/slice-kpi-averages/scenario.json` with `ui/fixtures`: the truth after the
  24 h click equals the 72 h default state, so the data check passes even if the click changes
  nothing. Input: the tiles at the start moment against the card's truth.
- **T8** `report.spec.ts` checks for uncommitted changes in `ui/app/src` only. At measure time
  `ui/fixtures`, `ui/app/e2e`, `ux/adapter.json` and the scenario cards were also uncommitted. They
  first appear in the commit after the round, so "commit 61d5e67" does not reproduce R-01.
- **T9** `ux/round.sh` and `report.spec.ts` count findings when the report is written, before the
  review agents file theirs. "Findings: none" and the trend's "open 0/0/0" are stale once findings
  are filed. Input: `ux/findings/UXF-001..005.md`, written 06:12, after the report (06:11).
- **T10** `ui/app/e2e/measure.spec.ts` takes the `start` moment straight after `goto` with no
  `settle()`, so its layout and axe results may come from a page that has not finished loading.
  `report.spec.ts` builds the run list from the files present and does not check them against
  `ux/adapter.json`, so a missing run would shrink the denominator without a word.
- **T11** `report.spec.ts`, the contact sheet: it shows completion frames only (SKILL.md asks for
  the key moments) at 200 px tall, too small to read. Every caption says "problems". Input:
  `ux/rounds/R-01/shots/contact-dark.jpg`.
- **T12** `bot.ts` `shiftObserver()` reports cumulative `layout_shift.score` with page load
  included, not the `layout_shift.per_minute` key of SKILL.md § 6. The 0.01 tolerance in
  `ratchet.spec.ts` is applied to the cumulative value.

## 4. Caveats to send with the round

- No people, no survey, no recordings of people, no paid level, no baseline and no last round. Every
  run is a bot on synthetic fixtures. The "recording" of a run is its screenshots plus the bot log.
- L1 and L7 did not run as part of the round command, whatever the Levels line says (T1). The
  keyboard half of gate 5 was not run on this build (§ 2, item 6).
- 0 of 20 means 20 of 20 correct completions and 0 of 20 problem-free runs. The problems are about
  6 defects, not 17.
- Gate 1 is measured in its docs-site form: vertical scroll is allowed.
- `screen_only.steers` and `glance_cost.median` carry no signal this round. `reading_load` depends
  on the viewport. `layout_shift` is cumulative.
- slice-kpi-averages proves only that the slice button is pressed (T7).
- The app state measured is commit 61d5e67 plus uncommitted changes in `ui/app/src`, `ui/fixtures`
  and the harness (T8).
- The trend row's open S1/S2/S3 = 0/0/0 was counted before any finding was filed (T9).
- A first `ux/baseline.json` taken from these values would make the current defects the bar,
  including the embed shift of 0.14 and steers 0.

## 5. Decision

No decision this round: the round dir holds no A and B measurements.

Decision: no view, no design decision is open in R-01 and the round measured one version only.
