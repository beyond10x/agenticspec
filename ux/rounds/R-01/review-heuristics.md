# R-01 heuristic review

3 pass, 12 partial, 0 fail (1 heuristic not observed: 8).

Inputs: report.md, shots/ (both contact sheets, all completion and worst-moment frames), the
full-size frames of all 50 runs in target/ux/measure/shots/, each run's json (moments, problems,
log), the five scenario cards, ux/adapter.json and the app source for the lines cited. No people
took part: no operator notes, no survey. No rrweb recording exists; a run json and its moment label
stand in for "recording file and second". Extra shots taken by a review probe on the built app with
the fixtures: shots/review-*.png. In shots/review-session-full-1440x900-light.png the top bar
appears halfway down the page: that is the sticky bar caught by the full-page capture, not a defect.

## 1. Heuristics

| # | heuristic | verdict | evidence |
|---|---|---|---|
| 1 | One next thing | partial | The session page names one next action ("Click an agent to focus it", shots/failing-check-1440x900-light-02.jpg); the overview names none (shots/slice-kpi-averages-1440x900-light-02.jpg). The adapter has no guidance cue, and every gating run found its target at glance 1 (report.md, gates table), so no finding. |
| 2 | State in words | pass | Checks say "Found · 1", "Unknown · 1 (1 possible)", "Passed" beside their marks (shots/failing-check-1440x900-dark-02.jpg); the chosen slice is named in the heading, "in the last 24 h" (shots/slice-kpi-averages-1440x900-light-02.jpg); the chosen agent is named in the hint and the chip "Sub-agent 1 only ✕" (shots/review-session-full-1440x900-light.png). |
| 3 | Nothing required hidden | partial | Tile note cut with no full text (UXF-004, shots/slice-kpi-averages-1440x900-light-02.jpg); lane cost not drawn at 390x844 (UXF-007, shots/agent-lane-390x844-dark-04.jpg); mark details on hover only (UXF-008, shots/review-timeline-hover-1440x900-dark.png); at 390x844 the failing check is out of view after opening the session (target/ux/measure/failing-check-390x844-light.json, "look the target: 3", scroll 450 px). |
| 4 | Stable layout | partial | embed-session shifts 0.14 at 1440x900 and 0.11 at 1280x720, every other run under 0.008 (UXF-005, target/ux/measure/embed-session-1440x900-light.json). |
| 5 | Provenance of every value | partial | Most values say their source ("at API list price", "tokens, any agent", shots/agent-lane-1440x900-light-03.jpg), but after a search the tiles and heading still count all 3 sessions beside 1 row (UXF-006, shots/find-session-by-search-1440x900-dark-02.jpg); the tile trend lines have no period label. |
| 6 | Errors cheap | partial | A wrong search is cleared by the ✕ in the box and a wrong lane by one click (shots/find-session-by-search-1440x900-dark-02.jpg, shots/review-session-full-1440x900-light.png); a wrong session costs the whole search again, since "← Sessions" forgets it (UXF-010, shots/review-back-to-list-1440x900-light.png). |
| 7 | Words the user can say | partial | Raw names in the event log ("SubAgent", "Main", "ttl expired") and "1 secret exposures", "1 sessions" (UXF-009, shots/review-session-full-1440x900-light.png, shots/find-session-by-search-1440x900-dark-02.jpg). |
| 8 | The counterpart's time respected | not observed | See § 4. |
| 9 | Keyboard and pointer parity | partial | Enter works on rows and lanes (the L2 keyboard spec passes) and a focused row shows a clear ring (shots/review-row-focus-1440x900-light.png), but Space on a lane does nothing and scrolls the page 644 px; mark details need a mouse hover (UXF-008, shots/review-lane-space-1440x900-light.png, shots/review-timeline-hover-1440x900-dark.png). |
| 10 | Recognition over recall | partial | The timeline legend names every mark (shots/agent-lane-1440x900-light-03.jpg); returning to the list makes the user recall and retype the search (UXF-010, shots/review-back-to-list-1440x900-light.png). |
| 11 | Calm by default | pass | No motion; colour marks only a cause: red "1 refused", the orange "!" of a found check, a static "Live" (shots/slice-kpi-averages-1440x900-light-02.jpg, shots/failing-check-1440x900-dark-02.jpg). |
| 12 | One state vocabulary | partial | "Cache hits" vs "Cache hit share", "Waiting share" vs "Waiting on a human", "Refused" vs "Refuse · Classifier", "at list price" vs "at API list price" (UXF-009, shots/slice-kpi-averages-1440x900-light-02.jpg, shots/agent-lane-1440x900-light-03.jpg). |
| 13 | A wrong cue recovered in one action | pass | A chosen lane is released by one click on it or on the chip ✕ (shots/review-session-full-1440x900-light.png, hint "Click the agent again to show all"); a search by the ✕ in the box (shots/find-session-by-search-1440x900-dark-02.jpg). |
| 14 | Unmistakable end state | partial | All 20 gating runs end with the target in view and right (report.md); at 390x844 agent-lane ends wrong in both themes (UXF-007, target/ux/measure/agent-lane-390x844-dark.json, completion.right false); the search end state is muddied by tiles that still say 3 (UXF-006). |
| 15 | Readable in every theme; key data at 14 px | partial | Muted notes under 4.5:1 (UXF-003, shots/agent-lane-1440x900-dark-03.jpg); the lane cost is 11 px (UXF-007, shots/agent-lane-1440x900-light-03.jpg); in the dark theme the hover tip has the same surface as the timeline card and its text lies over the lane (shots/review-timeline-hover-1440x900-dark.png). |
| 16 | Focus follows the work | partial | After Enter on a session row focus falls to the page body; Space on a lane moves the page 644 px off the timeline (UXF-008, shots/review-lane-space-1440x900-light.png); choosing a lane filters the event log, which is below the fold at 1440x900 (shots/agent-lane-1440x900-light-03.jpg). |

## 2. Findings filed

Cost is left out of every record: this project's rules forbid effort sizes, so the caller asks the
operator for them.

| id | title | severity | class | sessions | cost |
|---|---|---|---|---|---|
| UXF-006 | With a search, the figures and the heading still count every session in the slice | S2 | copy | find-session-by-search, 10 of 50 runs | not estimated |
| UXF-007 | The lane label hides its cost on a narrow screen and prints it at 11 px elsewhere | S2 | layout | agent-lane: 2 runs end wrong (390x844), 8 runs show the 11 px cost | not estimated |
| UXF-008 | The timeline answers the pointer only; Space on a lane scrolls the page away | S3 | affordance | agent-lane, failing-check, embed-session (review probe; no run presses Space) | not estimated |
| UXF-009 | One figure has two names across the pages, and some labels are raw or wrongly plural | S3 | copy | all 50 runs | not estimated |
| UXF-010 | Back to the list forgets the search | S2 | engine | follows find-session-by-search (review probe; no run goes back) | not estimated |

Not filed again: UXF-001 to UXF-005 cover the axe-core, contrast, cut-note and embed-shift problems
cited above.

## 3. Decision

No decision this round: the round dir holds no A and B.

Decision: no view, no design decision is open in R-01.

## 4. Not observed

- 8 The counterpart's time respected. The dashboard has no live counterpart, and the round measures
  no load time. Next round: time from opening to the target fully drawn on the session and embed
  pages (a host page waits on the embed), or a scenario where a second person waits on the reader.
