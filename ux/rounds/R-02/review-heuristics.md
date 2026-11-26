# R-02 heuristic review

4 pass, 11 partial, 0 fail (1 heuristic not observed: 8).

Inputs: report.md, shots/ (both contact sheets, all completion and worst-moment frames), the
full-size frames of all 50 runs in target/ux/measure/shots/, each run's json (moments, problems,
metrics, log), the five scenario cards, ux/adapter.json, ux/findings/ and the app source at the
round's commit for the lines cited. No people took part: no operator notes, no survey. No rrweb
recording exists; a run json and its moment label stand in for "recording file and second". No run
of R-02 presses a key, hovers or goes back, so heuristics 6, 9 and 16 rest partly on the R-01 probe
shots and on code unchanged since then. Extra shots: shots/review-event-log-390x844-light.png and
shots/review-event-log-390x844-dark.png, the event log at 390x844, which no frame of the round
reaches. They were taken on the app build present at review time, which is newer than the round's
(the tree had uncommitted changes outside the event log); EventLog.vue is the one the round measured.

## 1. Heuristics

| # | heuristic | verdict | evidence |
|---|---|---|---|
| 1 | One next thing | partial | The session page names one next action, "Click an agent to focus it" (target/ux/measure/shots/failing-check-1440x900-dark-02.jpg); the overview names none (target/ux/measure/shots/slice-kpi-averages-1440x900-dark-02.jpg). No guidance cue in the adapter, and all 20 gating runs found the target at glance 1 (report.md, gates table), so no finding. |
| 2 | State in words | pass | Checks say "Found · 1", "Unknown · 1 (1 possible)", "Passed" beside their marks (target/ux/measure/shots/failing-check-1440x900-dark-02.jpg); the chosen slice is in the heading, "3 sessions in the last 24 h" (target/ux/measure/shots/slice-kpi-averages-1440x900-dark-02.jpg); a chosen lane changes the hint to "Click the agent again to show all" and marks the agent row (target/ux/measure/shots/agent-lane-1440x900-light-02.jpg). The hint does not name the chosen agent; the named chip is in the event log, below the fold. |
| 3 | Nothing required hidden | partial | At 390x844 the chosen lane reads "Sub-agent 1" cut at its box and its cost is not drawn (UXF-007, target/ux/measure/shots/agent-lane-390x844-light-04.jpg); the event log cuts "Refuse · Clas..." and "by parent ag..." with no title and clips its Cost column (UXF-011, shots/review-event-log-390x844-light.png); mark details on hover only (UXF-008, by the code; R-01 probe shot ux/rounds/R-01/shots/review-timeline-hover-1440x900-dark.png). The tile note that R-01 found cut now wraps (target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg, "of input tokens read from cache"). |
| 4 | Stable layout | pass | layout_shift.score is at most 0.0097 on all 50 runs, embed-session 0 at every viewport (target/ux/measure/embed-session-1440x900-light.json; UXF-005 closed); the session page holds its place between moments (target/ux/measure/shots/agent-lane-1440x900-light-01.jpg against -02.jpg). |
| 5 | Provenance of every value | partial | After the search "planner" the list shows one row while the heading says "3 sessions in the last 72 h · $0.96" and the Sessions tile says 3 (UXF-006, target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg); the tile trend lines carry no period or scale (target/ux/measure/shots/slice-kpi-averages-1440x900-dark-02.jpg). Other values name their source: "at API list price", "tokens, any agent" (target/ux/measure/shots/agent-lane-1440x900-light-03.jpg). |
| 6 | Errors cheap | partial | A wrong search is cleared by the ✕ in the box (target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg) and a wrong lane by one click (target/ux/measure/shots/agent-lane-1440x900-light-02.jpg); a wrong session costs the whole search again, since "← Sessions" goes to "/" (UXF-010; the link's code is unchanged since the R-01 probe, ux/rounds/R-01/shots/review-back-to-list-1440x900-light.png). |
| 7 | Words the user can say | partial | "1 sessions" (UXF-009, target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg); "1 secret exposures" (UXF-009, target/ux/measure/shots/agent-lane-1440x900-light-03.jpg); raw "SubAgent", "Main", "ttl expired" in the event log (UXF-009, shots/review-event-log-390x844-light.png); "Unknown · 1 (1 possible)" (UXF-012, target/ux/measure/shots/failing-check-1440x900-dark-02.jpg). |
| 8 | The counterpart's time respected | not observed | See § 4. |
| 9 | Keyboard and pointer parity | partial | No R-02 run uses the keyboard. By the code the lane and the session and agent rows answer Enter only, and mark details need a mouse hover (UXF-008; R-01 probe ux/rounds/R-01/shots/review-lane-space-1440x900-light.png). |
| 10 | Recognition over recall | partial | The legend names every mark of the timeline (target/ux/measure/shots/agent-lane-1440x900-light-03.jpg); going back to the list makes the user recall and retype the search (UXF-010, ux/rounds/R-01/shots/review-back-to-list-1440x900-light.png). |
| 11 | Calm by default | pass | No motion between moments; colour marks only a cause: red "1 refused", the orange "!" of a found check, a static "Live" (target/ux/measure/shots/slice-kpi-averages-1440x900-dark-02.jpg, target/ux/measure/shots/failing-check-1440x900-dark-02.jpg). |
| 12 | One state vocabulary | partial | The overview's "Cache hits", "Waiting share", "Working time", "at list price" against the session page's "Cache hit share", "Waiting on a human", "Agent working time", "at API list price"; the legend's "Refused" against the log's "Refuse · Classifier" (UXF-009, target/ux/measure/shots/slice-kpi-averages-1440x900-dark-02.jpg, target/ux/measure/shots/agent-lane-1440x900-light-03.jpg); a failed check is "Found" in its row and "1 found" in the card head (UXF-012). |
| 13 | A wrong cue recovered in one action | pass | A chosen lane is released by one click on it, as the hint says, "Click the agent again to show all" (target/ux/measure/shots/agent-lane-1440x900-light-02.jpg); a search by the ✕ in the box (target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg). |
| 14 | Unmistakable end state | partial | 20 of 20 gating runs end with the target in view and right (report.md); agent-lane at 390x844 ends wrong in both themes (UXF-007, target/ux/measure/agent-lane-390x844-dark.json, completion.right false); the search ends beside tiles that still count 3 (UXF-006). |
| 15 | Readable in every theme; key data at 14 px | partial | No axe problem on any of the 50 runs (report.md, problems seen: only the two cut cells), so the R-01 contrast failures are gone (UXF-003 closed; target/ux/measure/shots/failing-check-1440x900-dark-02.jpg); the lane cost, the scenario's truth, is drawn at 11 px (UXF-007, target/ux/measure/shots/agent-lane-1440x900-light-03.jpg, "model-demo-large · $0.07"). |
| 16 | Focus follows the work | partial | Choosing a lane filters the event log, which is below the fold at 1440x900 (target/ux/measure/shots/agent-lane-1440x900-light-03.jpg); at 390x844 the checks lie 450 px below the opened page (target/ux/measure/failing-check-390x844-light.json, "look the target: 3", "scroll 449.859375"); Space on a lane scrolls the page, by the code (UXF-008). |

## 2. Findings filed

Cost is left out of every record: this project does not size work.

| id | title | severity | class | sessions | cost |
|---|---|---|---|---|---|
| UXF-011 | On a narrow screen the event log cuts two cells and its cost column with no way to read them | S3 | layout | agent-lane, failing-check, embed-session at 390x844: 6 of 50 runs | not estimated |
| UXF-012 | A check's status is a bare count, "Unknown · 1 (1 possible)", that says neither what was counted nor what unknown means | S3 | copy | failing-check, agent-lane, embed-session: 30 of 50 runs open a page with the list | not estimated |

Open findings, all still shown by R-02; records unchanged:

- UXF-006: the search frames still show "3 sessions" and Sessions 3 beside one row
  (target/ux/measure/shots/find-session-by-search-1440x900-light-02.jpg).
- UXF-007: agent-lane at 390x844 still ends wrong in both themes, and the lane cost is still 11 px
  (target/ux/measure/shots/agent-lane-390x844-light-04.jpg,
  target/ux/measure/shots/agent-lane-1440x900-light-03.jpg).
- UXF-008: no R-02 run uses the keyboard; the code at the round's commit still handles Enter only
  (ui/app/src/components/Timeline.vue, SessionTable.vue).
- UXF-009: "1 sessions", "1 secret exposures", the two names per figure and the raw event names all
  still show (frames cited under heuristics 7 and 12).
- UXF-010: no R-02 run goes back; the "← Sessions" link at the round's commit still goes to the
  overview without the search or the slice (ui/app/src/pages/SessionPage.vue).

Not filed again: the closed UXF-001 to UXF-005 no longer show; heuristic 1's partial costs no time
(every run finds its target at glance 1).

## 3. Decision

No decision this round: the round dir holds no A and B.

Decision: no view, no design decision is open in R-02.

## 4. Not observed

- 8 The counterpart's time respected. The dashboard has no live counterpart, and the round still
  measures no load time. Next round: the time from opening to the target fully drawn on the
  session and embed pages (a host page waits on the embed), or a scenario where a second person
  waits on the reader.
