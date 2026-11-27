# S-02 report

Date 2026-11-27. Site at commit eec2f8c with uncommitted changes in website/, docs/ or ux/site/, built into website/build. Written by ux/site/round.sh.

## Levels

- This report: L2 layout contract and axe-core at every moment, L3 screen-only user, on every docs task, viewport and theme; and a sweep of every page of the sitemap in both themes (axe-core, the layout contract at every viewport, console errors, the embedded architecture frames).
- Outside it: the gates run in `task check:site`; L7, the two review agents, write review-heuristics.md and review-report.md after this report.
- Not run: L0 (the site has no guidance cue), L1 (no unit tests of the site components), L4 and L5 (paid levels), L6 and L8 (people).
- So: no survey, no recordings of people, no probe misses. Each run's recording is its screenshots and the bot's log in round.json.

Docs tasks: find-harness-kpis, layer-of-workspace, gate-kpis-of-authority, what-question-depends-on, best-variant-of-author-reviewer, toolcall-lifecycle, trigger-block-lifecycle; anchors find-harness-kpis and layer-of-workspace.

## Gates at the gating viewports

- 28 of 28 runs pass: complete and clean.
- 28 of 28 runs complete: the target in view and right.
- 28 of 28 runs are clean: no layout or axe problem at any moment.

| task | viewport | theme | completion | right | distinct problems | bot |
|---|---|---|---|---|---|---|
| best-variant-of-author-reviewer | 1440x900 | dark | glance 1 | yes | 0 | done |
| best-variant-of-author-reviewer | 1440x900 | light | glance 1 | yes | 0 | done |
| best-variant-of-author-reviewer | 1280x800 | dark | glance 1 | yes | 0 | done |
| best-variant-of-author-reviewer | 1280x800 | light | glance 1 | yes | 0 | done |
| find-harness-kpis | 1440x900 | dark | glance 1 | yes | 0 | done |
| find-harness-kpis | 1440x900 | light | glance 1 | yes | 0 | done |
| find-harness-kpis | 1280x800 | dark | glance 1 | yes | 0 | done |
| find-harness-kpis | 1280x800 | light | glance 1 | yes | 0 | done |
| gate-kpis-of-authority | 1440x900 | dark | glance 1 | yes | 0 | done |
| gate-kpis-of-authority | 1440x900 | light | glance 1 | yes | 0 | done |
| gate-kpis-of-authority | 1280x800 | dark | glance 1 | yes | 0 | done |
| gate-kpis-of-authority | 1280x800 | light | glance 1 | yes | 0 | done |
| layer-of-workspace | 1440x900 | dark | glance 1 | yes | 0 | done |
| layer-of-workspace | 1440x900 | light | glance 1 | yes | 0 | done |
| layer-of-workspace | 1280x800 | dark | glance 1 | yes | 0 | done |
| layer-of-workspace | 1280x800 | light | glance 1 | yes | 0 | done |
| toolcall-lifecycle | 1440x900 | dark | glance 1 | yes | 0 | done |
| toolcall-lifecycle | 1440x900 | light | glance 1 | yes | 0 | done |
| toolcall-lifecycle | 1280x800 | dark | glance 1 | yes | 0 | done |
| toolcall-lifecycle | 1280x800 | light | glance 1 | yes | 0 | done |
| trigger-block-lifecycle | 1440x900 | dark | glance 1 | yes | 0 | done |
| trigger-block-lifecycle | 1440x900 | light | glance 1 | yes | 0 | done |
| trigger-block-lifecycle | 1280x800 | dark | glance 1 | yes | 0 | done |
| trigger-block-lifecycle | 1280x800 | light | glance 1 | yes | 0 | done |
| what-question-depends-on | 1440x900 | dark | glance 1 | yes | 0 | done |
| what-question-depends-on | 1440x900 | light | glance 1 | yes | 0 | done |
| what-question-depends-on | 1280x800 | dark | glance 1 | yes | 0 | done |
| what-question-depends-on | 1280x800 | light | glance 1 | yes | 0 | done |

## Measured-only viewports (not gated)

- 2 of 42 runs pass.
- 30 of 42 runs complete.
- 2 of 42 runs are clean.

| task | viewport | theme | completion | right | distinct problems | bot |
|---|---|---|---|---|---|---|
| best-variant-of-author-reviewer | 1024x768 | dark | glance 1 | yes | 2 | done |
| best-variant-of-author-reviewer | 1024x768 | light | glance 1 | yes | 2 | done |
| best-variant-of-author-reviewer | 768x1024 | dark | glance 3 | no: on /docs, not /docs/architectures/author-reviewer | 1 | on the page but not on the screen: the link Architectures in the sidebar |
| best-variant-of-author-reviewer | 768x1024 | light | glance 3 | no: on /docs, not /docs/architectures/author-reviewer | 1 | on the page but not on the screen: the link Architectures in the sidebar |
| best-variant-of-author-reviewer | 390x844 | dark | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| best-variant-of-author-reviewer | 390x844 | light | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| find-harness-kpis | 1024x768 | dark | glance 1 | yes | 1 | done |
| find-harness-kpis | 1024x768 | light | glance 1 | yes | 1 | done |
| find-harness-kpis | 768x1024 | dark | glance 1 | yes | 1 | done |
| find-harness-kpis | 768x1024 | light | glance 1 | yes | 1 | done |
| find-harness-kpis | 390x844 | dark | glance 1 | yes | 4 | done |
| find-harness-kpis | 390x844 | light | glance 1 | yes | 4 | done |
| gate-kpis-of-authority | 1024x768 | dark | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 1024x768 | light | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 768x1024 | dark | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 768x1024 | light | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 390x844 | dark | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 390x844 | light | glance 2 | yes | 1 | the target is not fully in view and not covered (glance 2) |
| layer-of-workspace | 1024x768 | dark | glance 1 | yes | 0 | done |
| layer-of-workspace | 1024x768 | light | glance 1 | yes | 0 | done |
| layer-of-workspace | 768x1024 | dark | glance 1 | yes | 1 | done |
| layer-of-workspace | 768x1024 | light | glance 1 | yes | 1 | done |
| layer-of-workspace | 390x844 | dark | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| layer-of-workspace | 390x844 | light | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| toolcall-lifecycle | 1024x768 | dark | glance 1 | yes | 1 | done |
| toolcall-lifecycle | 1024x768 | light | glance 1 | yes | 1 | done |
| toolcall-lifecycle | 768x1024 | dark | glance 1 | yes | 1 | done |
| toolcall-lifecycle | 768x1024 | light | glance 1 | yes | 1 | done |
| toolcall-lifecycle | 390x844 | dark | glance 1 | yes | 4 | done |
| toolcall-lifecycle | 390x844 | light | glance 1 | yes | 4 | done |
| trigger-block-lifecycle | 1024x768 | dark | glance 1 | yes | 1 | done |
| trigger-block-lifecycle | 1024x768 | light | glance 1 | yes | 1 | done |
| trigger-block-lifecycle | 768x1024 | dark | glance 1 | yes | 1 | done |
| trigger-block-lifecycle | 768x1024 | light | glance 1 | yes | 1 | done |
| trigger-block-lifecycle | 390x844 | dark | glance 1 | yes | 4 | done |
| trigger-block-lifecycle | 390x844 | light | glance 1 | yes | 4 | done |
| what-question-depends-on | 1024x768 | dark | glance 1 | yes | 1 | done |
| what-question-depends-on | 1024x768 | light | glance 1 | yes | 1 | done |
| what-question-depends-on | 768x1024 | dark | glance 1 | yes | 1 | done |
| what-question-depends-on | 768x1024 | light | glance 1 | yes | 1 | done |
| what-question-depends-on | 390x844 | dark | glance 1 | yes | 4 | done |
| what-question-depends-on | 390x844 | light | glance 1 | yes | 4 | done |

## Problems seen in the task runs

13 distinct problems (page and problem).

- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (6: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured), 768x1024 dark (measured), 768x1024 light (measured))
- /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (4: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured))
- /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (4: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (4: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (4: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/trigger: scrolls sideways in its box: div.b10x-table-wrap (4: 1024x768 dark (measured), 1024x768 light (measured), 390x844 dark (measured), 390x844 light (measured))
- /: text cut off without its full value: b.navbar__title.text--truncate "agenticspec" (2: 768x1024 dark (measured), 768x1024 light (measured))
- /docs: text cut off without its full value: b.navbar__title.text--truncate "agenticspec" (2: 768x1024 dark (measured), 768x1024 light (measured))
- /docs/blocks: scrolls sideways in its box: div.as-map__frame (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks: scrolls sideways in its box: div.b10x-table-wrap (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.as-life__frame (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.as-life__frame (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/trigger: scrolls sideways in its box: div.as-life__frame (2: 390x844 dark (measured), 390x844 light (measured))

## Page sweep

45 pages of the sitemap, each in light and dark.

### axe-core, serious and critical, by rule

At 1440x900, 1280x800, 1024x768, 768x1024, 390x844, in light and dark.

none

### Layout, by viewport (light theme)

- 1024x768 (measured): 25 page(s)
  - /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (716 > 692)
  - /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (605 > 511); scrolls sideways in its box: div.b10x-table-wrap (637 > 511)
  - /docs/architectures/orchestrator-workers: scrolls sideways in its box: div.b10x-table-wrap (529 > 511); scrolls sideways in its box: div.b10x-table-wrap (598 > 511); scrolls sideways in its box: div.b10x-table-wrap (630 > 511)
  - /docs/architectures/scheduled-loops: scrolls sideways in its box: div.b10x-table-wrap (600 > 511); scrolls sideways in its box: div.b10x-table-wrap (632 > 511)
  - /docs/architectures/scope-partition: scrolls sideways in its box: div.b10x-table-wrap (599 > 511); scrolls sideways in its box: div.b10x-table-wrap (631 > 511)
  - /docs/architectures/shared-board: scrolls sideways in its box: div.b10x-table-wrap (641 > 511); scrolls sideways in its box: div.b10x-table-wrap (637 > 511)
  - /docs/architectures/workstation: scrolls sideways in its box: div.b10x-table-wrap (530 > 511); scrolls sideways in its box: div.b10x-table-wrap (640 > 511)
  - /docs/blocks/1-principals/delegation: scrolls sideways in its box: div.b10x-table-wrap (581 > 511)
  - /docs/blocks/1-principals/principal: scrolls sideways in its box: div.b10x-table-wrap (626 > 511)
  - /docs/blocks/10-evaluation/experiment: scrolls sideways in its box: div.b10x-table-wrap (563 > 511)
  - /docs/blocks/3-capability/knowledge: scrolls sideways in its box: div.b10x-table-wrap (521 > 511); scrolls sideways in its box: div.b10x-table-wrap (612 > 511); scrolls sideways in its box: div.b10x-table-wrap (585 > 511)
  - /docs/blocks/3-capability/tool: scrolls sideways in its box: div.b10x-table-wrap (606 > 511)
  - /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (513 > 511); scrolls sideways in its box: div.b10x-table-wrap (573 > 511); scrolls sideways in its box: div.b10x-table-wrap (579 > 511)
  - /docs/blocks/5-work/process: scrolls sideways in its box: div.b10x-table-wrap (549 > 511)
  - /docs/blocks/5-work/work: scrolls sideways in its box: div.as-life__frame (537 > 509); scrolls sideways in its box: div.b10x-table-wrap (600 > 511); scrolls sideways in its box: div.b10x-table-wrap (598 > 511)
  - /docs/blocks/6-governance/authority: scrolls sideways in its box: div.b10x-table-wrap (573 > 511); scrolls sideways in its box: div.b10x-table-wrap (616 > 511)
  - /docs/blocks/7-collaboration/channel: scrolls sideways in its box: div.b10x-table-wrap (558 > 511); scrolls sideways in its box: div.b10x-table-wrap (594 > 511)
  - /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (540 > 511); scrolls sideways in its box: div.b10x-table-wrap (658 > 511)
  - /docs/blocks/7-collaboration/trigger: scrolls sideways in its box: div.b10x-table-wrap (573 > 511)
  - /docs/blocks/8-place/runtime: scrolls sideways in its box: div.b10x-table-wrap (546 > 511)
  - /docs/blocks/8-place/sandbox: scrolls sideways in its box: div.b10x-table-wrap (608 > 511)
  - /docs/blocks/8-place/workspace: scrolls sideways in its box: div.b10x-table-wrap (558 > 511); scrolls sideways in its box: div.b10x-table-wrap (571 > 511)
  - /docs/blocks/9-composition/architecture: scrolls sideways in its box: div.b10x-table-wrap (532 > 511); scrolls sideways in its box: div.b10x-table-wrap (530 > 511)
  - /docs/blocks/planned: scrolls sideways in its box: div.b10x-table-wrap (609 > 511); scrolls sideways in its box: div.b10x-table-wrap (548 > 511); scrolls sideways in its box: div.b10x-table-wrap (606 > 511); and 6 more
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (939 > 690)
- 768x1024 (measured): 1 page(s)
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (939 > 734)
- 390x844 (measured): 36 page(s)
  - /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (716 > 358)
  - /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (489 > 358); scrolls sideways in its box: div.b10x-table-wrap (605 > 358); scrolls sideways in its box: div.b10x-table-wrap (464 > 358); and 1 more
  - /docs/architectures/orchestrator-workers: scrolls sideways in its box: div.b10x-table-wrap (529 > 358); scrolls sideways in its box: div.b10x-table-wrap (598 > 358); scrolls sideways in its box: div.b10x-table-wrap (420 > 358); and 1 more
  - /docs/architectures/planner-executor: scrolls sideways in its box: div.b10x-table-wrap (388 > 358)
  - /docs/architectures/scheduled-loops: scrolls sideways in its box: div.b10x-table-wrap (450 > 358); scrolls sideways in its box: div.b10x-table-wrap (600 > 358); scrolls sideways in its box: div.b10x-table-wrap (414 > 358); and 1 more
  - /docs/architectures/scope-partition: scrolls sideways in its box: div.b10x-table-wrap (424 > 358); scrolls sideways in its box: div.b10x-table-wrap (599 > 358); scrolls sideways in its box: div.b10x-table-wrap (368 > 358); and 1 more
  - /docs/architectures/shared-board: scrolls sideways in its box: div.b10x-table-wrap (430 > 358); scrolls sideways in its box: div.b10x-table-wrap (641 > 358); scrolls sideways in its box: div.b10x-table-wrap (373 > 358); and 1 more
  - /docs/architectures/workstation: scrolls sideways in its box: div.b10x-table-wrap (530 > 358); scrolls sideways in its box: div.b10x-table-wrap (384 > 358); scrolls sideways in its box: div.b10x-table-wrap (420 > 358); and 1 more
  - /docs/blocks: scrolls sideways in its box: div.as-map__frame (780 > 356); scrolls sideways in its box: div.b10x-table-wrap (472 > 358)
  - /docs/blocks/0-foundation/shared: scrolls sideways in its box: div.b10x-table-wrap (378 > 358)
  - /docs/blocks/1-principals/delegation: scrolls sideways in its box: div.b10x-table-wrap (371 > 358); scrolls sideways in its box: div.b10x-table-wrap (391 > 358); scrolls sideways in its box: div.b10x-table-wrap (485 > 358); and 7 more
  - /docs/blocks/1-principals/principal: scrolls sideways in its box: div.b10x-table-wrap (362 > 358); scrolls sideways in its box: div.b10x-table-wrap (388 > 358); scrolls sideways in its box: div.b10x-table-wrap (626 > 358); and 2 more
  - /docs/blocks/10-evaluation/experiment: scrolls sideways in its box: div.as-life__frame (470 > 356); scrolls sideways in its box: div.b10x-table-wrap (391 > 358); scrolls sideways in its box: div.b10x-table-wrap (398 > 358); and 6 more
  - /docs/blocks/10-evaluation/kpi: scrolls sideways in its box: div.b10x-table-wrap (415 > 358); scrolls sideways in its box: div.b10x-table-wrap (454 > 358); scrolls sideways in its box: div.b10x-table-wrap (502 > 358); and 4 more
  - /docs/blocks/2-intelligence/gateway: scrolls sideways in its box: div.as-life__frame (373 > 356); scrolls sideways in its box: div.b10x-table-wrap (382 > 358); scrolls sideways in its box: div.b10x-table-wrap (380 > 358); and 3 more
  - /docs/blocks/2-intelligence/model: scrolls sideways in its box: div.b10x-table-wrap (417 > 358); scrolls sideways in its box: div.b10x-table-wrap (360 > 358); scrolls sideways in its box: div.b10x-table-wrap (493 > 358); and 1 more
  - /docs/blocks/3-capability/knowledge: scrolls sideways in its box: div.b10x-table-wrap (412 > 358); scrolls sideways in its box: div.b10x-table-wrap (387 > 358); scrolls sideways in its box: div.b10x-table-wrap (413 > 358); and 5 more
  - /docs/blocks/3-capability/tool: scrolls sideways in its box: div.b10x-table-wrap (375 > 358); scrolls sideways in its box: div.b10x-table-wrap (441 > 358); scrolls sideways in its box: div.b10x-table-wrap (401 > 358); and 2 more
  - /docs/blocks/4-agent/agent: scrolls sideways in its box: div.b10x-table-wrap (380 > 358); scrolls sideways in its box: div.b10x-table-wrap (369 > 358); scrolls sideways in its box: div.b10x-table-wrap (360 > 358); and 6 more
  - /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (390 > 358); scrolls sideways in its box: div.as-life__frame (488 > 356); scrolls sideways in its box: div.b10x-table-wrap (412 > 358); and 9 more
  - /docs/blocks/5-work/process: scrolls sideways in its box: div.b10x-table-wrap (361 > 358); scrolls sideways in its box: div.b10x-table-wrap (549 > 358); scrolls sideways in its box: div.b10x-table-wrap (501 > 358)
  - /docs/blocks/5-work/work: scrolls sideways in its box: div.as-life__frame (537 > 356); scrolls sideways in its box: div.as-life__frame (358 > 356); scrolls sideways in its box: div.as-life__frame (401 > 356); and 13 more
  - /docs/blocks/6-governance/authority: scrolls sideways in its box: div.b10x-table-wrap (397 > 358); scrolls sideways in its box: div.b10x-table-wrap (438 > 358); scrolls sideways in its box: div.b10x-table-wrap (430 > 358); and 7 more
  - /docs/blocks/7-collaboration/channel: scrolls sideways in its box: div.as-life__frame (446 > 356); scrolls sideways in its box: div.b10x-table-wrap (430 > 358); scrolls sideways in its box: div.b10x-table-wrap (408 > 358); and 14 more
  - /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.as-life__frame (368 > 356); scrolls sideways in its box: div.b10x-table-wrap (372 > 358); scrolls sideways in its box: div.b10x-table-wrap (419 > 358); and 3 more
  - /docs/blocks/7-collaboration/trigger: scrolls sideways in its box: div.as-life__frame (418 > 356); scrolls sideways in its box: div.as-life__frame (507 > 356); scrolls sideways in its box: div.b10x-table-wrap (417 > 358); and 9 more
  - /docs/blocks/8-place/resource: scrolls sideways in its box: div.as-life__frame (486 > 356); scrolls sideways in its box: div.b10x-table-wrap (384 > 358); scrolls sideways in its box: div.b10x-table-wrap (391 > 358); and 1 more
  - /docs/blocks/8-place/runtime: scrolls sideways in its box: div.b10x-table-wrap (459 > 358); scrolls sideways in its box: div.b10x-table-wrap (387 > 358); scrolls sideways in its box: div.b10x-table-wrap (546 > 358)
  - /docs/blocks/8-place/sandbox: scrolls sideways in its box: div.b10x-table-wrap (608 > 358); scrolls sideways in its box: div.b10x-table-wrap (482 > 358)
  - /docs/blocks/8-place/workspace: scrolls sideways in its box: div.as-life__frame (464 > 356); scrolls sideways in its box: div.b10x-table-wrap (384 > 358); scrolls sideways in its box: div.b10x-table-wrap (377 > 358); and 4 more
  - /docs/blocks/9-composition/architecture: scrolls sideways in its box: div.b10x-table-wrap (470 > 358); scrolls sideways in its box: div.b10x-table-wrap (412 > 358); scrolls sideways in its box: div.b10x-table-wrap (532 > 358); and 10 more
  - /docs/blocks/planned: scrolls sideways in its box: div.b10x-table-wrap (609 > 358); scrolls sideways in its box: div.b10x-table-wrap (548 > 358); scrolls sideways in its box: div.b10x-table-wrap (606 > 358); and 6 more
  - /docs/dashboard: scrolls sideways in its box: div.b10x-table-wrap (381 > 358)
  - /docs/getting-started: scrolls sideways in its box: div.b10x-table-wrap (453 > 358)
  - /docs/kpis: scrolls sideways in its box: div.b10x-table-wrap (409 > 358)
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (939 > 356)

### Embedded architecture frames

none

### Console errors

none

## Metrics, light theme (lower is better)

A run that stopped before its target has no metrics here and in no sum (S-02, T5).

| task | screen_only.actions | screen_only.looks | screen_only.steers | screen_only.time_to_act_steps.max | glance_cost.median | glance_cost.p95 | reading_load.max | reading_load.median | layout_shift.score |
|---|---|---|---|---|---|---|---|---|---|
| best-variant-of-author-reviewer@1440x900 | 4 (base 4) (S-01 9) | 5 (base 5) (S-01 10) | 0 (base 0) | 2 (base 2) | 1 (base 1) (S-01 3) | 3 (base 3) | 318 (base 318) (S-01 270) | 240 (base 240) (S-01 172) | 0 (base 0) |
| best-variant-of-author-reviewer@1280x800 | 4 (base 4) (S-01 10) | 5 (base 5) (S-01 11) | 0 (base 0) | 2 (base 2) | 1 (base 1) (S-01 3) | 3 (base 3) | 269 (base 269) | 210 (base 210) (S-01 147) | 0 (base 0) |
| find-harness-kpis@1440x900 | 5 (base 5) | 6 (base 6) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 417 (base 403) (S-01 403) | 350.5 (base 349.5) (S-01 349.5) | 0 (base 0) |
| find-harness-kpis@1280x800 | 5 (base 5) | 6 (base 6) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 435 (base 421) (S-01 421) | 273.5 (base 273.5) | 0 (base 0) |
| gate-kpis-of-authority@1440x900 | 4 (base 4) | 4 (base 4) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 2 (base 2) | 307 (base 307) | 258 (base 265) (S-01 265) | 0 (base 0) |
| gate-kpis-of-authority@1280x800 | 4 (base 4) | 4 (base 4) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 3 (base 3) | 223 (base 223) (S-01 265) | 215 (base 215) | 0 (base 0) |
| layer-of-workspace@1440x900 | 2 (base 2) | 3 (base 3) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 2 (base 2) | 403 (base 403) | 367 (base 367) | 0 (base 0) |
| layer-of-workspace@1280x800 | 2 (base 2) | 3 (base 3) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 3 (base 3) | 397 (base 397) | 278 (base 278) | 0 (base 0) |
| toolcall-lifecycle@1440x900 | 6 (base 6) | 7 (base 7) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 417 (base 403) (S-01 403) | 293 (base 293) | 0 (base 0) |
| toolcall-lifecycle@1280x800 | 6 (base 6) | 7 (base 7) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 435 (base 421) (S-01 421) | 253 (base 253) | 0 (base 0) |
| trigger-block-lifecycle@1440x900 | 5 (base 5) | 6 (base 6) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 441 (base 441) | 203.5 (base 203.5) | 0 (base 0) |
| trigger-block-lifecycle@1280x800 | 6 (base 6) | 7 (base 7) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 433 (base 433) | 171 (base 171) | 0 (base 0) |
| what-question-depends-on@1440x900 | 4 (base 4) | 5 (base 5) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 441 (base 427) (S-01 427) | 188 (base 185) (S-01 185) | 0 (base 0) |
| what-question-depends-on@1280x800 | 4 (base 4) | 5 (base 5) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 433 (base 419) (S-01 419) | 175 (base 168) (S-01 168) | 0 (base 0) |

## Worst moments

- best-variant-of-author-reviewer 390x844 dark, completion on /docs/architectures/author-reviewer: 4 problem(s), the run does not complete (measured only). ![](shots/best-variant-of-author-reviewer-390x844-dark-08.jpg)
- layer-of-workspace 390x844 dark, completion on /docs/blocks: 2 problem(s), the run does not complete (measured only). ![](shots/layer-of-workspace-390x844-dark-04.jpg)
- best-variant-of-author-reviewer 768x1024 dark, completion on /docs: 1 problem(s), the run does not complete (measured only). ![](shots/best-variant-of-author-reviewer-768x1024-dark-02.jpg)
- gate-kpis-of-authority 1024x768 dark, completion on /docs/kpis-catalogue: 1 problem(s), the run does not complete (measured only). ![](shots/gate-kpis-of-authority-1024x768-dark-15.jpg)
- gate-kpis-of-authority 768x1024 dark, completion on /docs/kpis-catalogue: 1 problem(s), the run does not complete (measured only). ![](shots/gate-kpis-of-authority-768x1024-dark-04.jpg)
- gate-kpis-of-authority 390x844 dark, completion on /docs/kpis-catalogue: 1 problem(s), the run does not complete (measured only). ![](shots/gate-kpis-of-authority-390x844-dark-07.jpg)
- trigger-block-lifecycle 390x844 dark, after the link ^trigger\b in the main on /docs/blocks/7-collaboration/trigger: 11 problem(s) (measured only). ![](shots/trigger-block-lifecycle-390x844-dark-10.jpg)
- find-harness-kpis 390x844 dark, after the link ^harness\b in the main on /docs/blocks/4-agent/harness: 10 problem(s) (measured only). ![](shots/find-harness-kpis-390x844-dark-11.jpg)

## Completion frames

- best-variant-of-author-reviewer 1440x900 dark: ![](shots/best-variant-of-author-reviewer-1440x900-dark-05.jpg)
- best-variant-of-author-reviewer 1440x900 light: ![](shots/best-variant-of-author-reviewer-1440x900-light-05.jpg)
- find-harness-kpis 1440x900 dark: ![](shots/find-harness-kpis-1440x900-dark-06.jpg)
- find-harness-kpis 1440x900 light: ![](shots/find-harness-kpis-1440x900-light-06.jpg)
- gate-kpis-of-authority 1440x900 dark: ![](shots/gate-kpis-of-authority-1440x900-dark-04.jpg)
- gate-kpis-of-authority 1440x900 light: ![](shots/gate-kpis-of-authority-1440x900-light-04.jpg)
- layer-of-workspace 1440x900 dark: ![](shots/layer-of-workspace-1440x900-dark-03.jpg)
- layer-of-workspace 1440x900 light: ![](shots/layer-of-workspace-1440x900-light-03.jpg)
- toolcall-lifecycle 1440x900 dark: ![](shots/toolcall-lifecycle-1440x900-dark-07.jpg)
- toolcall-lifecycle 1440x900 light: ![](shots/toolcall-lifecycle-1440x900-light-07.jpg)
- trigger-block-lifecycle 1440x900 dark: ![](shots/trigger-block-lifecycle-1440x900-dark-06.jpg)
- trigger-block-lifecycle 1440x900 light: ![](shots/trigger-block-lifecycle-1440x900-light-06.jpg)
- what-question-depends-on 1440x900 dark: ![](shots/what-question-depends-on-1440x900-dark-05.jpg)
- what-question-depends-on 1440x900 light: ![](shots/what-question-depends-on-1440x900-light-05.jpg)

Contact sheets: ![light](shots/contact-light.jpg) ![dark](shots/contact-dark.jpg)

## Findings

As of this report; the review agents file theirs after it.

- SUF-001 S2 closed: The embedded architecture frame shows the site's landing page, not the architecture, when the site is served by docusaurus serve
- SUF-002 S2 closed: On an architecture page the best variant is 6 to 7 scrolls down, under a 2052 px frame, with no on-page contents to jump there
- SUF-003 S3 open: The layer map is taller than a screen, so most block cards are below the fold when the map opens
- SUF-004 S3 open: The filter chips of the KPI catalogue have names that fuse the label and the count, "gate34", "no gate80", "correctness42"
- SUF-005 S3 closed: By keyboard, a block page's contents list comes after every link of the article: 87 key presses to reach the KPIs of harness, with the skip link
- SUF-006 S3 open: The KPI catalogue table scrolls sideways in its box below the gating widths; at 390x844 a filtered row is never fully in view
- SUF-007 S3 open: Tables and lifecycle diagrams scroll sideways in their box at 1024x768 (15 pages) and on a phone (29 of 39 pages)
- SUF-008 S2 closed: A lifecycle diagram that scrolls sideways on a phone cannot be reached by keyboard (axe-core serious, scrollable-region-focusable)
- SUF-009 S3 open: On a phone the layer map is 780 px wide in a 356 px box, so a block card and its layer label are not on the screen together
- SUF-010 S3 open: At 768x1024 the open menu draws its 637 px panel in a 352 px drawer, so its links run under the drawer's edge
- SUF-011 S3 open: In the open menu at 768x1024 the site title is cut off with an ellipsis and no way to read it whole
- SUF-012 S3 open: The KPI catalogue is only the last entry of the sidebar, below the fold, and the navbar's "KPIs" opens a different page that does not link to it
- SUF-013 S3 closed: Key data of the KPI tables and the block map is below 14 px: the gate marker is 9.7 px, the column heads 10.9 px, the cell text 13.4 px
- SUF-014 S3 open: The cards of the layer map cut their description with an ellipsis and give it whole only in a hover tooltip, and their counts read "ent", "cmd", "evt"

## Did the S-01 change set move its metrics?

Written by hand after round.sh. Before: S-01 (its report, and keyboard.spec on the S-01 site). After: this round and `task check:site` on the same build.

| finding | target metric | target | S-01 | S-02 | moved |
|---|---|---|---|---|---|
| SUF-001 | architecture frames that land on the home page (sweep, both themes) | 0 | 10 (5 pages x 2 themes) | 0; and 0 on a host with the GitHub Pages layout | yes |
| SUF-002 | best-variant-of-author-reviewer screen_only.actions 1440 / 1280 | ≤ 3 | 9 / 10 | 4 / 4 | yes, target missed by 1 |
| SUF-002 | same, glance_cost.median 1440 / 1280 | 1 | 3 / 3 | 1 / 1 | yes |
| SUF-002 | same, looks 1440 / 1280 | – | 10 / 11 | 5 / 5 | yes |
| SUF-002 (trade-off) | same, reading_load.median 1440 / 1280 | – | 172 / 147 | 240 / 210 | worse: S-01 counted no word of the frame; the baseline entry carries a decision note |
| SUF-008 | axe scrollable-region-focusable at 390x844, harness page | 0 | 1 | 0; no axe rule at any viewport in the sweep | yes |
| SUF-005 | key presses from the skip link to the harness contents' KPIs entry | ≤ 20 | about 62 | 25 (23 Tab) | yes, target missed by 5 |
| SUF-005 | keyboard.spec key presses: find-harness-kpis / what-question / best-variant / toolcall | – | 87 / 54 / 47 / 72 | 47 / 30 / 41 / 32 | yes |
| SUF-013 | smallest font of gate marker / column head / cell (KPI catalogue) | ≥ 14 px | 9.7 / 10.9 / 13.4 px | 14 / 14 / 14 px | yes |

Headline numbers (trend.md):

| | S-01 | S-02 |
|---|---|---|
| gating runs pass | 24/24 | 28/28 (trigger-block-lifecycle added) |
| measured-only runs pass / complete | 2/36, 18/36 | 2/42, 30/42 |
| distinct problems in task runs | 35 | 13 |
| axe rules in the sweep | 0 (one viewport) | 0 (every viewport, both themes, T2) |
| actions, sum over the S-01 tasks | 61 | 50 (61 with the new task's 11) |
| open S1 / S2 / S3 | 0 / 3 / 8 | 0 / 0 / 9 |

New in the sweep at 1024x768 (measured only): 10 pages whose tables now scroll in their box. The six architecture pages got a contents column (SUF-002), and the 14 px cells (SUF-013) widened tables on process, runtime and architecture. Each such table is a focusable named region. Same class as SUF-007, which is open.
