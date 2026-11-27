# S-01 heuristic review

7 pass, 8 partial, 1 fail

Reviewer inputs: report.md, shots/ (contact sheets, completion and worst-moment frames), the scenario
cards, heuristics.md, and my own screenshots of the built site (docusaurus serve on 127.0.0.1:8481,
Playwright, `shots/review-*.png`). No recordings exist in this round. Verdicts are for the gating
viewports (1440x900, 1280x800) unless a row says otherwise; the measured viewports (1024x768,
768x1024, 390x844) are cited where they change a verdict. SUF-001 to SUF-011 were filed before this
review; where my evidence agrees I cite them and file nothing new.

## 1. Heuristics

| # | heuristic | verdict | evidence |
|---|---|---|---|
| 1 | One next thing | partial | Home has one primary action, "Get started", and three text links: review-home-1440x900-light.png. An architecture page offers none toward its result: the frame fills the screen and the best variant is 6 to 7 scrolls down (SUF-002; review-architecture-1440x900-dark.png). |
| 2 | State in words | pass | The catalogue says "3 of 114 KPIs" after the gate filter, the pressed chip is tinted and `aria-pressed=true`, KPI status is the word "planned": review-kpis-gate-authority-1440x900-light.png. |
| 3 | Nothing required hidden | partial | Ratchet and gates clean at the gating viewports. Hidden: the architecture frame shows the landing page (SUF-001; evidence-SUF-001-author-reviewer-1440x900-light.jpg), the map descriptions are given whole only in a hover tooltip (SUF-014), the catalogue is below the sidebar's fold (SUF-012), tables and diagrams scroll sideways below 1280 px (SUF-006, SUF-007, SUF-009). |
| 4 | Stable layout | pass | layout_shift.score 0 for all 12 gating rows of report.md "Metrics"; no jump visible in the completion frames of contact-light.jpg and contact-dark.jpg. |
| 5 | Provenance of every value | pass | Each KPI row names its source ("How: ...", "Source: design") and the page links its YAML (kpis/catalogue.yaml); block pages link blocks/4-agent/harness.yaml; the architecture description cites its papers: review-kpis-1440x900-dark.png, review-harness-1440x900-light.png, review-architecture-1440x900-dark.png. |
| 6 | Errors cheap | pass | The 404 page offers "Go to the documentation" and "Back to the home page" and keeps the navbar and footer: review-404-1440x900-dark.png. A filter chip toggles off with the same click. |
| 7 | Words the user can say | partial | Chip names fuse label and count (SUF-004); map counts read "ent / cmd / evt" (SUF-014: evidence-SUF-003-layer-map-1440x900-light.jpg); the navbar's "KPIs" and the "KPI catalogue" are two different things (SUF-012). |
| 8 | Counterpart's time respected (the reader's) | fail | The architecture task is the worst of the round: 9 actions and glance 3 against 2 to 6 elsewhere (SUF-002, S2), and the page's own frame shows the wrong page (SUF-001, S2; best-variant-of-author-reviewer-1440x900-light-10.jpg). By keyboard 87 presses reach the KPIs of harness (SUF-005). |
| 9 | Keyboard and pointer parity | partial | The focus ring is drawn (2 px outline, light rgb(138,83,0), dark rgb(239,208,154): review-focus-harness-1440x900-light.png and -dark.png) and every task completes by keyboard, but the contents list comes after the article (SUF-005) and a scrolling lifecycle cannot take focus at 390x844 (SUF-008, S2; find-harness-kpis-390x844-dark-10.jpg). |
| 10 | Recognition over recall | pass | Sidebar tree with the open layer, breadcrumbs, a contents column with "Depends on / Entities / KPIs", and every filter option visible as a chip: review-harness-1440x900-light.png, review-kpis-1440x900-dark.png. |
| 11 | Calm by default | pass | Home, docs and 404 hold no banner, popup, carousel or auto-moving element: review-home-full-1440-dark.png; layout_shift 0. Motion itself was not recorded (see § 4). |
| 12 | One state vocabulary | partial | Status words are shared (computed, simulated, both, planned) but "KPIs" names two pages, and "Building blocks" is both the navbar item and a sidebar category while the sidebar's "Using agenticspec" holds a page of the same name as the navbar's KPIs (SUF-012: review-kpis-page-1440x900-light.png). |
| 13 | A wrong cue recovered in one action | partial | The 404 and the breadcrumbs recover in one click. The frame's "Open full screen" link has the same wrong target as the frame (SUF-001), and at 768x1024 the open menu's links cannot be pressed whole, so 10 of 12 runs end on the start page (SUF-010; find-harness-kpis-768x1024-dark-02.jpg). |
| 14 | Unmistakable end state | pass | All 24 gating runs end with the target heading or rows in view and the page named in the breadcrumb: the completion frames of contact-light.jpg and contact-dark.jpg; the catalogue states "3 of 114 KPIs". |
| 15 | Readable in every theme; key data at 14 px or more | partial | Both themes hold contrast (6.1 to 15 for the text measured; axe-core clean in the sweep: report.md "Page sweep"), compare review-kpis-1440x900-dark.png with the light shots. Key data is under 14 px: gate pill 9.7 px, table heads 10.9 px, cells 13.4 px (SUF-013). |
| 16 | Focus follows the work | partial | After a filter the table updates in place and the count is above it (review-kpis-gate-authority-1440x900-light.png). The current page is not in view in the sidebar on the catalogue (925 px of 900, no navbar item active, SUF-012: review-kpis-1440x900-dark.png); on the layer map the target card needs two scrolls (SUF-003). |

## 2. Findings filed

Filed by this review (SUF-001 to SUF-011 pre-date it and are not repeated):

| id | title | severity | class | sessions | cost |
|---|---|---|---|---|---|
| SUF-012 | The KPI catalogue is only the last entry of the sidebar, below the fold, and the navbar's "KPIs" opens a different page that does not link to it | S3 | affordance | none of the 6 scenarios exercises it (gate-kpis-of-authority starts on the page by URL) | S |
| SUF-013 | Key data of the KPI tables and the block map is below 14 px: the gate marker 9.7 px, the column heads 10.9 px, the cells 13.4 px | S3 | cue | gate-kpis-of-authority; every block page with tables | S |
| SUF-014 | The cards of the layer map cut their description with an ellipsis and give it whole only in a hover tooltip, and their counts read "ent", "cmd", "evt" | S3 | copy | layer-of-workspace, find-harness-kpis, what-question-depends-on | S |

Agreement with the earlier findings: SUF-001 and SUF-002 (S2) are the cause of the one fail; I saw
SUF-001 on the architecture page in both themes (review-architecture-1440x900-dark.png: the whole
site, navbar included, nested in the page). I could not confirm SUF-001 on GitHub Pages and did not
try; the gate and the preview use `docusaurus serve`.

## 3. Decision

No decision this round. Decision: no view, no design decision is open in S-01 and the round holds no A and B measurements.

## 4. Not observed

- Heuristic 11 (motion): no recording; the round has stills only. A recording of a theme toggle, a
  filter click and a sidebar expand would show transitions and `prefers-reduced-motion`.
- Heuristic 8 for a second person: the site has no counterpart; read as the reader's time only.
- Heuristic 6 beyond the 404: no form or input of the site can fail except the catalogue's search;
  a scenario with a search that has no result would show its empty state (not run: the card set has
  none).
- Heuristics 13 and 16 on search: the site has no search (adapter.json "search": null).
- Page scenarios run only on six tasks; the architecture frame itself (the content of the page) was
  never seen in its intended state, so heuristics 1, 3 and 14 on an architecture page are judged on
  the wrong frame (SUF-001). Rerun best-variant-of-author-reviewer after SUF-001 closes.
- The trigger block page is not in the build (adapter pending_scenarios); nothing is judged on it.
