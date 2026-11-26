# R-01 triage

## Caveats

- No people and no paid levels: L4, L5, L6 and L8 did not run, so no survey, no probe and no
  recording of a person. Confidence 2 means two of these agree: a gate or metric of the round, the
  heuristic review, the report review.
- Cost is not estimated: this project does not size work. Every finding gets cost 1, so the
  priority ranks on severity, frequency and confidence alone.
- Frequency is the share of the 5 scenarios a finding shows on, at any viewport.
- The report reviewer found 12 defects in the round tools (review-report.md § 3, T1 to T12). They
  are not UI changes and take no place in the change set. T1, T3, T4, T7, T8, T10 and T11 are fixed
  before R-02, so R-02's report differs from R-01's in shape. The metrics of the 10 gating runs keep
  their definitions.

## Did the last change sets move their metrics?

First round: no earlier change set.

## Priorities

priority = severity weight x frequency x confidence / cost

| finding | severity | frequency | confidence | priority | in the set |
|---|---|---|---|---|---|
| UXF-001 chips carry aria-pressed on role radio | S1 | 3/5 | 1 | 5.4 | yes (every S1) |
| UXF-003 muted notes under 4.5:1 | S2 | 5/5 | 2 (axe, heuristic 15) | 6.0 | yes |
| UXF-004 tile note cut without its full value | S2 | 2/5 | 2 (gate 1, heuristic 3) | 2.4 | yes |
| UXF-002 the timeline is an image that holds buttons | S2 | 3/5 | 1 | 1.8 | yes |
| UXF-005 the embed page jumps while it loads | S2 | 1/5 | 2 (layout shift, both reviews) | 1.2 | yes |
| UXF-007 the lane label hides its cost on a narrow screen | S2 | 1/5 | 2 (heuristic review, report review) | 1.2 | no: tie with UXF-005, neither on an anchor card; UXF-005 shows at the gating viewports, UXF-007 only at 390x844 |
| UXF-006 the figures ignore the search | S2 | 1/5 | 1 | 0.6 | no |
| UXF-010 back to the list forgets the search | S2 | 0/5 (review probe) | 1 | 0 | no |
| UXF-008 Space on a lane scrolls the page | S3 | 3/5 | 1 | 0.6 | no |
| UXF-009 two names for one figure, raw labels | S3 | 5/5 | 1 | 1.0 | no |

## Change set (5)

Each is built test first: its test in `ui/app/e2e/findings.spec.ts` ran red on the R-01 build
(6 of 6 failed, `UXF-003` in both themes), then the change makes it green.

1. UXF-001: the chips keep `role="radio"` with `aria-checked` and drop `aria-pressed`; the chosen
   chip is styled by `aria-checked`. Target: axe aria-allowed-attr 0.
2. UXF-003: the muted colour is `#6b6964` in the light theme and `#a3a19a` in the dark theme, at
   least 4.5:1 on every surface it is drawn on. Target: axe color-contrast 0.
3. UXF-004: a tile note wraps instead of ending in an ellipsis. Target: gate 1, no text cut off.
4. UXF-002: the timeline's svg is a group named "Timeline of n agents", which may hold buttons.
   Target: axe nested-interactive 0.
5. UXF-005: the session and embed pages appear once their timeline is read, not part by part.
   Target: `layout_shift.score` of embed-session under 0.01.

UXF-006, UXF-007, UXF-008, UXF-009 and UXF-010 stay open for R-02's triage.
