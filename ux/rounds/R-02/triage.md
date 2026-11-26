# R-02 triage

## Caveats

- No people and no paid levels, as in R-01. Confidence 2 means two of: a gate or metric of the
  round, the heuristic review, the report review (of this round or, for a finding still open, of
  R-01).
- Cost is not estimated: every finding gets cost 1.
- Frequency is the share of the 5 scenarios a finding shows on, at any viewport. UXF-010 shows on
  none: no scenario goes back to the list (found by R-01's review probe), so its priority is 0.
- The report reviewer's T13 to T16 are tool defects, not UI changes; they stay listed in
  review-report.md for the next round.
- UXF-013 is filed from the report review (the failed check below the fold at 1280x720).

## Did the last change sets move their metrics?

| change | finding | target | R-01 | R-02 | moved |
|---|---|---|---|---|---|
| 1 | UXF-001 | axe aria-allowed-attr, runs | 30 | 0 | yes |
| 2 | UXF-003 | axe color-contrast, runs | 30 | 0 | yes |
| 3 | UXF-004 | text cut off in a tile note, runs | 28 | 0 | yes |
| 4 | UXF-002 | axe nested-interactive, runs | 30 | 0 | yes |
| 5 | UXF-005 | layout_shift.score of embed-session@1440x900 | 0.1435 | 0 | yes |

Counts in runs from review-report.md § 1 (recomputed from the raw run files). Gating runs that pass
every gate: 0 of 20 in R-01, 20 of 20 in R-02.

## Priorities

priority = severity weight x frequency x confidence / cost

| finding | severity | frequency | confidence | priority | in the set |
|---|---|---|---|---|---|
| UXF-007 lane label hides its cost on a narrow screen | S2 | 1/5 | 2 (R-01 heuristic and report reviews) | 1.2 | yes |
| UXF-011 event log cut off on a phone | S3 | 3/5 | 2 (R-02 heuristic and report reviews) | 1.2 | yes |
| UXF-009 two names for one figure, raw labels, plurals | S3 | 5/5 | 1 | 1.0 | yes |
| UXF-006 the figures ignore the search | S2 | 1/5 | 1 | 0.6 | yes (anchor card) |
| UXF-008 Space on a lane scrolls the page | S3 | 3/5 | 1 | 0.6 | yes (tie with UXF-012; both show on the anchor failing-check) |
| UXF-012 a check's status words | S3 | 3/5 | 1 | 0.6 | no: the set is full |
| UXF-013 failed check below the fold at 1280x720 | S3 | 1/5 | 1 | 0.2 | no |
| UXF-010 back to the list forgets the search | S2 | 0/5 | 1 | 0 | no; its change and test wait (test.fixme in findings.spec.ts) |

## Change set (5)

Each test in `ui/app/e2e/findings.spec.ts` ran red on the R-02 build, then the change makes it
green.

1. UXF-007: on a narrow plot the label column is 104 px and keeps the lane's cost on its second
   line; the second line is 12 px. Target: agent-lane completes at 390x844.
2. UXF-011: on a phone the What cell wraps, so the table fits its card; every What cell carries its
   full text as a title. Target: no cut cell and no table past its card at 390x844.
3. UXF-009: the overview's figures take the session page's names (Agent working time, Waiting on a
   human, Cache hit share, at API list price); counts use the plural helper; the event log writes
   roles and raw names in words. A tile label wraps instead of being cut off. Target: the test's
   labels.
4. UXF-006: with a search, the figures are summed from the matching rows and the heading says how
   many sessions match. Target: Sessions 1 and the matching session's cost after the search
   "planner".
5. UXF-008: Space acts like Enter on lanes, agent rows and session rows and does not scroll the
   page; a session opened from the list puts focus on its title. Target: aria-pressed after Space,
   scroll position unchanged. The details of a mark, still on hover only, are UXF-014.

## Build notes

- The first build of the set failed gate 1: the longer tile labels were cut off next to the trend
  line. Tile labels now wrap.
- The second build failed the ratchet: reading_load rose by up to 11.5 words on the overview
  (labels, and a heading that said the figures leave the search out). That would have needed a
  decision. The change to UXF-006 took the other fix its record names, figures summed from the
  matching rows, with a heading of the old length; the cost note dropped "at API list price",
  which the heading says. The third build passes every gate and the ratchet with no key worse.
- The UXF-006 test was rewritten with that change. Its first form ran red on the R-02 build, and the
  new form asserts the heading "1 session matching", which the R-02 build does not show.
- UXF-010's change was written before this triage and set aside; it waits for a scenario that goes
  back to the list.
