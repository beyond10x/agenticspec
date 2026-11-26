# R-02 review: report reviewer

Verdict: fit to send. Every number in `report.md` recomputes exactly from the 50 raw run files, and
none of its lines claims something that did not happen. Three things in it carry less than they seem
to: the `(base …)` cells equal the run's own values by construction (§ 1), the completion column
reads "glance 1" in all 50 rows, and "Problems seen" hides that its one defect shows in 3 scenarios.
Default action: send the round with § 4 attached and without a re-run. In R-02's triage, file the two
defects in § 2 items 1 and 2 that have no record. Fix T2, T13, T14, T15 and T16 before R-03.

Sources: `target/ux/measure/<scenario>-<WxH>-<theme>.json` (50 files, written 06:26:37 to 06:28:42
by the measure project of `ux/round.sh`), `target/ux/measure/shots/*.jpg`, `target/ux/round-R-02.txt`
(the round command's log), `target/ux/check-app.txt` (the app gate, 06:25), the tool source under
`ui/app/e2e/`, `ux/adapter.json`, `ux/scenarios/*/scenario.json`, `ux/baseline.json`,
`ux/findings/`, `ux/rounds/R-01/` and `ux/rounds/trend.md`. `round.json` was used only to compare
with. Measure was not re-run: the recomputation did not need it. No people, no survey, no operator
notes, no round README and no paid level exist for this round.

## 1. Are the numbers right and meaningful?

Recomputed with `jq` from the raw files:

| number in report.md | report | recomputed | differs |
|---|---|---|---|
| runs | 20 gating + 30 measured | 50 files, 20 + 30; `report.spec.ts` now also checks the count against the adapter | no |
| gating runs complete / clean | 20 of 20 / 20 of 20 | 20 / 20 | no |
| measured runs complete / clean | 28 of 30 / 24 of 30 | 28 (agent-lane 390x844 both themes: right = false, no error) / 24 | no |
| table rows: completion, right, distinct problems (50 rows) | as printed | the same 50 rows | no |
| problems seen | 2 lines | the same 2 lines, in 6 runs | no, but see T2 |
| metric table, light theme, gating (10 rows x 9 keys) | as printed | the same 90 values; dark equals light in all 25 run pairs | no |
| R-01 column in the metric table | embed-session 0.1435 and 0.1098 | equal to `ux/rounds/R-01/round.json`; the other 88 values equal R-01 | no |
| trend R-02: gating pass, measured pass, distinct problems, actions, looks, reading load max | 20/20, 24/30, 2, 13, 21, 1634 | 20/20, 24/30, 2, 13, 21, 1634 | no |
| trend R-02: open S1/S2/S3 | 0/3/2 | 0/3/2 (UXF-006, 007, 010 S2; UXF-008, 009 S3) when written | no |
| trend R-01: open S1/S2/S3 | 0/0/0 | 1/7/2: UXF-001 to UXF-010 were all filed in R-01, before its triage | 1 S1, 7 S2, 2 S3 (T9) |
| `round.json` runs (completion, problems, log) | | equal to the raw files, byte for byte after sorting | no |

Only the copied R-01 trend row differs. Whether each number means what its name says:

- **The `(base …)` cells** compare R-02 with itself. `ux/baseline.json` was written at 06:25:14
  by `UX_RATCHET=write` from the gates run of the same app source (`target/ux/check-app.txt`,
  06:25), 83 s before measure started. The base equals the run in 90 of 90 cells. The baseline was
  taken after R-01's change set, so it shows nothing about that change set. The ratchet's check
  against `main` was skipped ("no baseline on main yet"). The only comparison in the table is the
  R-01 column (T15).
- **The state measured** is the R-01 change set. The report says "commit 302e2fe with uncommitted
  changes"; that was true when it was written. Every file under `ui/` and `ux/` that commit 95b37b2
  (06:30) changed was last written before 06:26:37. The only later change is
  `ui/app/e2e/findings.spec.ts` (06:31), which holds tests only. So R-02 measured 95b37b2's app.
- **"20 of 20 runs are clean"** covers gates 1, 2 and 5 (axe-core) only. Gate 1 is still the
  docs-site form (vertical page scroll allowed), and `ux/adapter.json` does not record that choice.
  The keyboard half of gate 5 is not in the round. It passed 5 of 5 in `task check:app` at 06:25 on
  the same source. That closes R-01 review § 2 item 6, but the report does not say it. Gate 4's
  "every fix takes at most 2 actions" is not measured: no scenario contains a fix.
- **"completion: glance 1"** is the glance at the end, after the bot's scrolls, so it is 1 in 50 of
  50 rows and tells nothing (T13). The first look at the target is in the bot's log. failing-check
  saw it partly at 1280x720, a gating viewport (`look the target: 2`, then one scroll), and out of
  view at 1024x768 and 390x844 (`3`).
- **"distinct problems: 2"** (trend 17 to 2) uses the same key in both rounds, so the trend is
  comparable. R-01's 17 lines were about 6 defects (R-01 review § 1). R-02's 2 lines are 1 defect:
  event-log cells cut off at 390x844.
- **"(2: 390x844 dark, 390x844 light)"** counts viewport x theme pairs. The defect shows in 6 runs
  over 3 scenarios (agent-lane, embed-session, failing-check) (T2).
- **`screen_only.steers`** is 0 everywhere. `ux/adapter.json` `metric_notes` now explains why, but
  it is still a ratchet key with value 0 and tolerance 0 in 10 baseline rows (T6).
  **`glance_cost.median`** is 1 in all 50 runs. **`reading_load`** depends on the viewport (embed-session:
  89 at 390x844, 247 at 1920x1080). **`layout_shift.score`** is cumulative, page load included
  (T12; documented in the adapter, not changed).
- **slice-kpi-averages** now means something. Its truth requires the heading "in the last 24 h",
  and the 72 h default reads "in the last 72 h" (contact sheet: slice-kpi-averages against
  find-session-by-search). The tile values still equal the default and add no check.
- **Comparability with R-01**: the same 5 scenarios, 5 viewports and 2 themes, with the same bot
  and metric definitions. Two tool changes touch the runs. (a) `settle()` runs before the `start`
  moment (T10). It cannot explain the drop in problems, because R-01's axe problems showed at every
  moment, completion included. (b) The embed-session target changed from role `img` to `group` in
  `ui/app/e2e/scenarios.ts`, in step with the app's UXF-002 change. Every scenario runs every round
  and none rotates, so the round can find nothing new (§ 4).

Triage rule 4, R-01's change set against its target metrics (run counts from R-01's `round.json`
and R-02's raw files):

| change | target metric | R-01 | R-02 | moved? |
|---|---|---|---|---|
| UXF-001 chips without aria-pressed | axe aria-allowed-attr | 30 runs | 0 runs | yes |
| UXF-003 muted colour 4.5:1 | axe color-contrast | 30 runs | 0 runs | yes |
| UXF-004 tile note wraps | gate 1, text cut off | 34 runs (28 tile notes, 6 event-log cells) | 6 runs (the event-log cells only) | yes; the tile notes are gone, and the 6 left are not UXF-004's |
| UXF-002 timeline svg is a group | axe nested-interactive | 30 runs | 0 runs | yes |
| UXF-005 page appears at once | embed-session `layout_shift.score` < 0.01 | 0.1435 / 0.1098 at the gating viewports; 0.13 / 0.09 / 0.06 at 1920 / 1024 / 390 (R-01 review) | 0 at all 5 viewports | yes |

All 5 moved in one round. The report shows only UXF-005's movement, in the R-01 column. It has no
place where the other four movements can be seen (T16). Wrapping the tile notes does not break them
badly at 390x844: "of input tokens read from / cache" (`find-session-by-search-390x844-light-00.jpg`).

## 2. Findings the report misses

1. **Event-log cells cut off at 390x844 have no finding record.** "Refuse · Classifier" and "by
   parent agent" end in an ellipsis without a title in 6 runs (agent-lane, embed-session and
   failing-check, both themes), at every moment from the session page on. UXF-009 names "Refuse ·
   Classifier" as a vocabulary problem, not as cut text. None of the report's three worst-moment
   shots shows the cells, because the event log is below the fold (`shots/embed-session-390x844-dark-00.jpg`)
   (T14).
2. **failing-check: the failed check is partly below the fold at 1280x720, a gating viewport.**
   "Cache bust after idle" sits on the bottom edge (`target/ux/measure/shots/failing-check-1280x720-light-01.jpg`)
   and costs a scroll. At 1024x768 and 390x844 it is fully out of view. This was R-01 review § 2
   item 2 and was not filed. `glance_cost.p95` = 2 and `screen_only.actions` = 2 at
   failing-check@1280x720 are now baseline values. failing-check is an anchor card.
3. **UXF-007 shows on two more scenarios than its record names.** At 390x844 the embed and failing-check
   session pages draw no lane meta either: "Main agent" and "Sub-agent 1" appear without model or
   cost (`shots/embed-session-390x844-dark-00.jpg`). embed-session passes only because its truth checks
   the session total $0.55. This bears on UXF-007's frequency in triage (1/5 in R-01's table).
4. **The last time label of the timeline reaches the card's border at 1440x900.** "10:20:20"
   ends at the card edge, past the plot (`target/ux/measure/shots/embed-session-1440x900-light-00.jpg`,
   right end of the axis). No check sees text inside the svg (T5). It is minor and not proven to clip.
5. **What the report does not say has changed**: the keyboard gate passes on this source (§ 1),
   and slice-kpi-averages now tests the click (T7).

## 3. Tool defects

Status of R-01's T1 to T12. Triage says T1, T3, T4, T7, T8, T10 and T11 were fixed.

| R-01 defect | status in R-02 | evidence |
|---|---|---|
| T1 Levels line claimed L1 and L7 ran | fixed | `report.spec.ts` Levels lines; the report says where L1 and L7 run. Left over: `ux/adapter.json` `levels.run` still lists L1 and L7 as run levels without saying where |
| T2 problem key = rule + first node; node counts lost | not fixed | `report.spec.ts` problems map, `layout.ts` `axe()`; R-02 input: "(2: …)" for 6 runs in 3 scenarios |
| T3 problems column summed problem x moment | fixed | column "distinct problems" = `distinct(r)`; agent-lane 390x844 reads 2, not 8 |
| T4 completion and problems merged | fixed | `completes()` and `clean()` apart; measured runs say "measured only". Left over: contact-sheet captions and `round.json` `pass` still use `ok()` |
| T5 contract skips svg descendants | not fixed | `layout.ts:29`; § 2 item 4, UXF-007's lane label |
| T6 steers never counted | documented, not changed | `ux/adapter.json` `metric_notes`; still a ratchet key in `ux/baseline.json` |
| T7 slice truth equal to the default | fixed | `scenarios.ts` heading check; card `truth.heading` |
| T8 dirty check covered `ui/app/src` only | fixed | `git status -- ui ux ':!ux/rounds'`. Left over: the report can only name the parent commit; see T15 for the same timing problem |
| T9 findings counted before the reviews | not fixed | R-02 open counts are taken before R-02's review agents; trend keeps R-01's 0/0/0 (true value 1/7/2) |
| T10 no settle before `start`; run count unchecked | fixed | `measure.spec.ts` `settle()` before `moment('start')`; `report.spec.ts:47` throws on a count other than 50 |
| T11 contact sheet: completion frames, 200 px, every caption "problems" | partly fixed | frames 360 px tall, caption only on runs that fail; still completion frames only, not the key moments SKILL.md asks for (`shots/contact-light.jpg` title: "completion frames") |
| T12 cumulative `layout_shift.score`, not `per_minute` | documented, not changed | `ux/adapter.json` `metric_notes`; tolerance 0.01 still on the cumulative value |

New:

- **T13** `ui/app/e2e/report.spec.ts`, the table's completion column: prints `r.completion.seen`,
  the glance after the bot's scrolls, so it is "glance 1" in 50 of 50 rows. The first look is
  dropped. Input: `failing-check-1280x720-light.json` log `look the target: 2`, `scroll 51.65625`.
- **T14** `report.spec.ts`, worst moments: the shot is the viewport at the moment, chosen without
  checking whether the problem's element is in the frame. Input: all three R-02 worst-moment shots;
  `shots/embed-session-390x844-dark-00.jpg` ends at the timeline and the cut cells are below it. The
  three entries are the same defect, so the list shows one problem three times.
- **T15** The baseline was written before the round, from the same source (`ux/baseline.json`
  06:25:14, measure 06:26:37). `report.spec.ts` prints `(base x)` without saying where the baseline
  came from, so 90 of 90 cells read as a held ratchet. `ratchet.spec.ts` skipped "the baseline is no
  worse than on main" because `main` has no baseline yet. Input: `target/ux/check-app.txt` (1
  skipped), `ux/baseline.json` against the raw metrics.
- **T16** `report.spec.ts` reads the last round's `round.json` only for the ratchet metrics. It has
  no section for "did the last change set move its target metrics" (SKILL.md § 3.6). The axe and
  cut-text targets of 4 of the 5 changes cannot be seen in the report. Input: R-02 `report.md`, where
  only the UXF-005 movement appears.

## 4. Caveats to send with the round

- No people, no survey, no recordings of people, no paid level. Every run is a bot on synthetic
  fixtures. A run's "recording" is its screenshots plus the bot log.
- R-02 measured the R-01 change set, which is the app of commit 95b37b2. The report names its parent
  302e2fe plus changes.
- The baseline was taken from this same app minutes before the round: "base" equals "now" in every
  cell. R-01's change set shows only against R-01's `round.json` (§ 1 table: all 5 changes moved
  their target metric).
- "Clean" means no layout or axe problem. Gate 1 is measured in its docs-site form. The keyboard
  gate passed in `task check:app` on the same source, outside the round.
- "glance 1" in the completion column is the end state. failing-check at 1280x720 needs a scroll
  (§ 2 item 2).
- The one remaining problem, event-log cells cut at 390x844, shows in 3 scenarios and has no finding
  record (§ 2 item 1).
- `screen_only.steers` and `glance_cost.median` carry no signal. `reading_load` depends on the
  viewport. `layout_shift` is cumulative.
- No card rotates: the same 5 scenarios run every round. UXF-008 and UXF-010 came from review
  probes, not from any of the 50 runs.
- The trend's R-01 row says open 0/0/0. At the end of R-01 it was 1/7/2 (T9).

## 5. Decision

No decision this round: no design decision is open, and the round dir holds no measurements of an A
and a B.

Decision: no view, no design decision is open in R-02 and the round measured one version only.
