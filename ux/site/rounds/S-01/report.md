# S-01 report

Date 2026-11-27. Site at commit eec2f8c with uncommitted changes in website/, docs/ or ux/site/, built into website/build. Written by ux/site/round.sh.

## Levels

- This report: L2 layout contract and axe-core at every moment, L3 screen-only user, on every docs task, viewport and theme; and a sweep of every page of the sitemap in both themes (axe-core, the layout contract at every viewport, console errors, the embedded architecture frames).
- Outside it: the gates run in `task check:site`; L7, the two review agents, write review-heuristics.md and review-report.md after this report.
- Not run: L0 (the site has no guidance cue), L1 (no unit tests of the site components), L4 and L5 (paid levels), L6 and L8 (people).
- So: no survey, no recordings of people, no probe misses. Each run's recording is its screenshots and the bot's log in round.json.

Docs tasks: find-harness-kpis, layer-of-workspace, gate-kpis-of-authority, what-question-depends-on, best-variant-of-author-reviewer, toolcall-lifecycle; anchors find-harness-kpis and layer-of-workspace.

## Gates at the gating viewports

- 24 of 24 runs pass: complete and clean.
- 24 of 24 runs complete: the target in view and right.
- 24 of 24 runs are clean: no layout or axe problem at any moment.

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
| what-question-depends-on | 1440x900 | dark | glance 1 | yes | 0 | done |
| what-question-depends-on | 1440x900 | light | glance 1 | yes | 0 | done |
| what-question-depends-on | 1280x800 | dark | glance 1 | yes | 0 | done |
| what-question-depends-on | 1280x800 | light | glance 1 | yes | 0 | done |

## Measured-only viewports (not gated)

- 2 of 36 runs pass.
- 18 of 36 runs complete.
- 2 of 36 runs are clean.

| task | viewport | theme | completion | right | distinct problems | bot |
|---|---|---|---|---|---|---|
| best-variant-of-author-reviewer | 1024x768 | dark | glance 1 | yes | 1 | done |
| best-variant-of-author-reviewer | 1024x768 | light | glance 1 | yes | 1 | done |
| best-variant-of-author-reviewer | 768x1024 | dark | glance 3 | no: on /docs, not /docs/architectures/author-reviewer | 1 | the screen shows no the link Architectures in the sidebar |
| best-variant-of-author-reviewer | 768x1024 | light | glance 3 | no: on /docs, not /docs/architectures/author-reviewer | 1 | the screen shows no the link Architectures in the sidebar |
| best-variant-of-author-reviewer | 390x844 | dark | glance 2 | yes | 4 | the target is not fully in view and not covered (glance 2) |
| best-variant-of-author-reviewer | 390x844 | light | glance 2 | yes | 4 | the target is not fully in view and not covered (glance 2) |
| find-harness-kpis | 1024x768 | dark | glance 1 | yes | 2 | done |
| find-harness-kpis | 1024x768 | light | glance 1 | yes | 2 | done |
| find-harness-kpis | 768x1024 | dark | glance 3 | no: on /, not /docs/blocks/4-agent/harness | 1 | the screen shows no the link Building blocks in the navbar |
| find-harness-kpis | 768x1024 | light | glance 3 | no: on /, not /docs/blocks/4-agent/harness | 1 | the screen shows no the link Building blocks in the navbar |
| find-harness-kpis | 390x844 | dark | glance 1 | yes | 14 | done |
| find-harness-kpis | 390x844 | light | glance 1 | yes | 14 | done |
| gate-kpis-of-authority | 1024x768 | dark | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 1024x768 | light | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 768x1024 | dark | glance 1 | yes | 2 | done |
| gate-kpis-of-authority | 768x1024 | light | glance 1 | yes | 2 | done |
| gate-kpis-of-authority | 390x844 | dark | glance 2 | yes | 3 | the target is not fully in view and not covered (glance 2) |
| gate-kpis-of-authority | 390x844 | light | glance 2 | yes | 3 | the target is not fully in view and not covered (glance 2) |
| layer-of-workspace | 1024x768 | dark | glance 1 | yes | 0 | done |
| layer-of-workspace | 1024x768 | light | glance 1 | yes | 0 | done |
| layer-of-workspace | 768x1024 | dark | glance 3 | no: on /, not /docs/blocks | 1 | the screen shows no the link Building blocks in the navbar |
| layer-of-workspace | 768x1024 | light | glance 3 | no: on /, not /docs/blocks | 1 | the screen shows no the link Building blocks in the navbar |
| layer-of-workspace | 390x844 | dark | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| layer-of-workspace | 390x844 | light | glance 2 | yes | 2 | the target is not fully in view and not covered (glance 2) |
| toolcall-lifecycle | 1024x768 | dark | glance 1 | yes | 2 | done |
| toolcall-lifecycle | 1024x768 | light | glance 1 | yes | 2 | done |
| toolcall-lifecycle | 768x1024 | dark | glance 3 | no: on /, not /docs/blocks/4-agent/harness | 1 | the screen shows no the link Building blocks in the navbar |
| toolcall-lifecycle | 768x1024 | light | glance 3 | no: on /, not /docs/blocks/4-agent/harness | 1 | the screen shows no the link Building blocks in the navbar |
| toolcall-lifecycle | 390x844 | dark | glance 1 | yes | 14 | done |
| toolcall-lifecycle | 390x844 | light | glance 1 | yes | 14 | done |
| what-question-depends-on | 1024x768 | dark | glance 1 | yes | 1 | done |
| what-question-depends-on | 1024x768 | light | glance 1 | yes | 1 | done |
| what-question-depends-on | 768x1024 | dark | glance 3 | no: on /, not /docs/blocks/7-collaboration/question | 1 | the screen shows no the link Building blocks in the navbar |
| what-question-depends-on | 768x1024 | light | glance 3 | no: on /, not /docs/blocks/7-collaboration/question | 1 | the screen shows no the link Building blocks in the navbar |
| what-question-depends-on | 390x844 | dark | glance 1 | yes | 6 | done |
| what-question-depends-on | 390x844 | light | glance 1 | yes | 6 | done |

## Problems seen in the task runs

35 distinct problems (page and problem).

- /: text cut off without its full value: b.navbar__title.text--truncate "agenticspec" (2: 768x1024 dark (measured), 768x1024 light (measured))
- /docs: text cut off without its full value: b.navbar__title.text--truncate "agenticspec" (2: 768x1024 dark (measured), 768x1024 light (measured))
- /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (633 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (633 > 511) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (376 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (431 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (567 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks: scrolls sideways in its box: div.as-map__frame (780 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks: scrolls sideways in its box: div.b10x-table-wrap (460 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: axe serious scrollable-region-focusable: n nodes, figure:nth-child(38) > .as-life__frame (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.as-life__frame (488 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (361 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (369 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (374 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (395 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (399 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (410 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (416 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (453 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (525 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (525 > 511) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (540 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (540 > 511) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.as-life__frame (368 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (367 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (482 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (603 > 358) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (603 > 511) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (688 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (802 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (802 > 690) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (802 > 734) (2: 768x1024 dark (measured), 768x1024 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 356) (2: 390x844 dark (measured), 390x844 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 690) (2: 1024x768 dark (measured), 1024x768 light (measured))
- /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 734) (2: 768x1024 dark (measured), 768x1024 light (measured))

## Page sweep

39 pages of the sitemap, each in light and dark.

### axe-core, serious and critical, by rule

none

### Layout, by viewport (light theme)

- 1024x768 (measured): 15 page(s)
  - /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (633 > 511)
  - /docs/blocks/1-principals/delegation: scrolls sideways in its box: div.b10x-table-wrap (523 > 511)
  - /docs/blocks/1-principals/principal: scrolls sideways in its box: div.b10x-table-wrap (566 > 511)
  - /docs/blocks/10-evaluation/experiment: scrolls sideways in its box: div.b10x-table-wrap (527 > 511)
  - /docs/blocks/3-capability/knowledge: scrolls sideways in its box: div.b10x-table-wrap (563 > 511); scrolls sideways in its box: div.b10x-table-wrap (542 > 511)
  - /docs/blocks/3-capability/tool: scrolls sideways in its box: div.b10x-table-wrap (563 > 511)
  - /docs/blocks/4-agent/harness: scrolls sideways in its box: div.b10x-table-wrap (525 > 511); scrolls sideways in its box: div.b10x-table-wrap (540 > 511)
  - /docs/blocks/5-work/work: scrolls sideways in its box: div.as-life__frame (537 > 509); scrolls sideways in its box: div.b10x-table-wrap (546 > 511); scrolls sideways in its box: div.b10x-table-wrap (557 > 511)
  - /docs/blocks/6-governance/authority: scrolls sideways in its box: div.b10x-table-wrap (525 > 511); scrolls sideways in its box: div.b10x-table-wrap (568 > 511)
  - /docs/blocks/7-collaboration/channel: scrolls sideways in its box: div.b10x-table-wrap (521 > 511); scrolls sideways in its box: div.b10x-table-wrap (550 > 511)
  - /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.b10x-table-wrap (603 > 511)
  - /docs/blocks/8-place/sandbox: scrolls sideways in its box: div.b10x-table-wrap (552 > 511)
  - /docs/blocks/8-place/workspace: scrolls sideways in its box: div.b10x-table-wrap (513 > 511); scrolls sideways in its box: div.b10x-table-wrap (536 > 511)
  - /docs/blocks/planned: scrolls sideways in its box: div.b10x-table-wrap (553 > 511); scrolls sideways in its box: div.b10x-table-wrap (513 > 511); scrolls sideways in its box: div.b10x-table-wrap (550 > 511); and 5 more
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 690)
- 768x1024 (measured): 1 page(s)
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 734)
- 390x844 (measured): 29 page(s)
  - /docs/architectures: scrolls sideways in its box: div.b10x-table-wrap (633 > 358)
  - /docs/architectures/author-reviewer: scrolls sideways in its box: div.b10x-table-wrap (376 > 358); scrolls sideways in its box: div.b10x-table-wrap (431 > 358); scrolls sideways in its box: div.b10x-table-wrap (567 > 358)
  - /docs/architectures/workstation: scrolls sideways in its box: div.b10x-table-wrap (472 > 358); scrolls sideways in its box: div.b10x-table-wrap (569 > 358)
  - /docs/blocks: scrolls sideways in its box: div.as-map__frame (780 > 356); scrolls sideways in its box: div.b10x-table-wrap (460 > 358)
  - /docs/blocks/1-principals/delegation: scrolls sideways in its box: div.b10x-table-wrap (421 > 358); scrolls sideways in its box: div.b10x-table-wrap (366 > 358); scrolls sideways in its box: div.b10x-table-wrap (416 > 358); and 3 more
  - /docs/blocks/1-principals/principal: scrolls sideways in its box: div.b10x-table-wrap (566 > 358); scrolls sideways in its box: div.b10x-table-wrap (466 > 358); scrolls sideways in its box: div.b10x-table-wrap (446 > 358)
  - /docs/blocks/10-evaluation/experiment: scrolls sideways in its box: div.as-life__frame (470 > 356); scrolls sideways in its box: div.b10x-table-wrap (433 > 358); scrolls sideways in its box: div.b10x-table-wrap (459 > 358); and 1 more
  - /docs/blocks/10-evaluation/kpi: scrolls sideways in its box: div.b10x-table-wrap (362 > 358); scrolls sideways in its box: div.b10x-table-wrap (415 > 358); scrolls sideways in its box: div.b10x-table-wrap (461 > 358); and 4 more
  - /docs/blocks/2-intelligence/gateway: scrolls sideways in its box: div.as-life__frame (373 > 356); scrolls sideways in its box: div.b10x-table-wrap (374 > 358); scrolls sideways in its box: div.b10x-table-wrap (411 > 358); and 1 more
  - /docs/blocks/2-intelligence/model: scrolls sideways in its box: div.b10x-table-wrap (381 > 358); scrolls sideways in its box: div.b10x-table-wrap (460 > 358); scrolls sideways in its box: div.b10x-table-wrap (440 > 358)
  - /docs/blocks/3-capability/knowledge: scrolls sideways in its box: div.b10x-table-wrap (371 > 358); scrolls sideways in its box: div.b10x-table-wrap (375 > 358); scrolls sideways in its box: div.b10x-table-wrap (419 > 358); and 3 more
  - /docs/blocks/3-capability/tool: scrolls sideways in its box: div.b10x-table-wrap (398 > 358); scrolls sideways in its box: div.b10x-table-wrap (370 > 358); scrolls sideways in its box: div.b10x-table-wrap (447 > 358); and 1 more
  - /docs/blocks/4-agent/agent: scrolls sideways in its box: div.b10x-table-wrap (377 > 358); scrolls sideways in its box: div.b10x-table-wrap (380 > 358); scrolls sideways in its box: div.b10x-table-wrap (392 > 358); and 3 more
  - /docs/blocks/4-agent/harness: scrolls sideways in its box: div.as-life__frame (488 > 356); scrolls sideways in its box: div.b10x-table-wrap (374 > 358); scrolls sideways in its box: div.b10x-table-wrap (361 > 358); and 8 more
  - /docs/blocks/5-work/process: scrolls sideways in its box: div.b10x-table-wrap (489 > 358); scrolls sideways in its box: div.b10x-table-wrap (460 > 358)
  - /docs/blocks/5-work/work: scrolls sideways in its box: div.as-life__frame (537 > 356); scrolls sideways in its box: div.as-life__frame (358 > 356); scrolls sideways in its box: div.as-life__frame (401 > 356); and 6 more
  - /docs/blocks/6-governance/authority: scrolls sideways in its box: div.b10x-table-wrap (383 > 358); scrolls sideways in its box: div.b10x-table-wrap (382 > 358); scrolls sideways in its box: div.b10x-table-wrap (525 > 358); and 2 more
  - /docs/blocks/7-collaboration/channel: scrolls sideways in its box: div.as-life__frame (446 > 356); scrolls sideways in its box: div.b10x-table-wrap (371 > 358); scrolls sideways in its box: div.b10x-table-wrap (382 > 358); and 9 more
  - /docs/blocks/7-collaboration/question: scrolls sideways in its box: div.as-life__frame (368 > 356); scrolls sideways in its box: div.b10x-table-wrap (367 > 358); scrolls sideways in its box: div.b10x-table-wrap (482 > 358); and 1 more
  - /docs/blocks/8-place/resource: scrolls sideways in its box: div.as-life__frame (441 > 356); scrolls sideways in its box: div.b10x-table-wrap (471 > 358)
  - /docs/blocks/8-place/runtime: scrolls sideways in its box: div.b10x-table-wrap (397 > 358); scrolls sideways in its box: div.b10x-table-wrap (499 > 358)
  - /docs/blocks/8-place/sandbox: scrolls sideways in its box: div.b10x-table-wrap (552 > 358); scrolls sideways in its box: div.b10x-table-wrap (447 > 358)
  - /docs/blocks/8-place/workspace: scrolls sideways in its box: div.as-life__frame (464 > 356); scrolls sideways in its box: div.b10x-table-wrap (362 > 358); scrolls sideways in its box: div.b10x-table-wrap (513 > 358); and 2 more
  - /docs/blocks/9-composition/architecture: scrolls sideways in its box: div.b10x-table-wrap (370 > 358); scrolls sideways in its box: div.b10x-table-wrap (366 > 358); scrolls sideways in its box: div.b10x-table-wrap (475 > 358); and 9 more
  - /docs/blocks/planned: scrolls sideways in its box: div.b10x-table-wrap (553 > 358); scrolls sideways in its box: div.b10x-table-wrap (513 > 358); scrolls sideways in its box: div.b10x-table-wrap (550 > 358); and 6 more
  - /docs/dashboard: scrolls sideways in its box: div.b10x-table-wrap (381 > 358)
  - /docs/getting-started: scrolls sideways in its box: div.b10x-table-wrap (453 > 358)
  - /docs/kpis: scrolls sideways in its box: div.b10x-table-wrap (409 > 358)
  - /docs/kpis-catalogue: scrolls sideways in its box: div.as-kpis__scroll (918 > 356)

### Embedded architecture frames

- /docs/architectures/author-reviewer dark: frame /agenticspec/architectures/author-reviewer.html: lands on /agenticspec/; frame /agenticspec/architectures/author-reviewer.html: shows "See how your agent sessions actually went. | agenticspec", not the author-reviewer architecture
- /docs/architectures/author-reviewer light: frame /agenticspec/architectures/author-reviewer.html: lands on /agenticspec/; frame /agenticspec/architectures/author-reviewer.html: shows "See how your agent sessions actually went. | agenticspec", not the author-reviewer architecture
- /docs/architectures/handoff dark: frame /agenticspec/architectures/handoff.html: lands on /agenticspec/; frame /agenticspec/architectures/handoff.html: shows "See how your agent sessions actually went. | agenticspec", not the handoff architecture
- /docs/architectures/handoff light: frame /agenticspec/architectures/handoff.html: lands on /agenticspec/; frame /agenticspec/architectures/handoff.html: shows "See how your agent sessions actually went. | agenticspec", not the handoff architecture
- /docs/architectures/parallel-unaware dark: frame /agenticspec/architectures/parallel-unaware.html: lands on /agenticspec/; frame /agenticspec/architectures/parallel-unaware.html: shows "See how your agent sessions actually went. | agenticspec", not the parallel-unaware architecture
- /docs/architectures/parallel-unaware light: frame /agenticspec/architectures/parallel-unaware.html: lands on /agenticspec/; frame /agenticspec/architectures/parallel-unaware.html: shows "See how your agent sessions actually went. | agenticspec", not the parallel-unaware architecture
- /docs/architectures/solo dark: frame /agenticspec/architectures/solo.html: lands on /agenticspec/; frame /agenticspec/architectures/solo.html: shows "See how your agent sessions actually went. | agenticspec", not the solo architecture
- /docs/architectures/solo light: frame /agenticspec/architectures/solo.html: lands on /agenticspec/; frame /agenticspec/architectures/solo.html: shows "See how your agent sessions actually went. | agenticspec", not the solo architecture
- /docs/architectures/workstation dark: frame /agenticspec/architectures/workstation.html: lands on /agenticspec/; frame /agenticspec/architectures/workstation.html: shows "See how your agent sessions actually went. | agenticspec", not the workstation architecture
- /docs/architectures/workstation light: frame /agenticspec/architectures/workstation.html: lands on /agenticspec/; frame /agenticspec/architectures/workstation.html: shows "See how your agent sessions actually went. | agenticspec", not the workstation architecture

### Console errors

none

## Metrics, light theme (lower is better)

| task | screen_only.actions | screen_only.looks | screen_only.steers | screen_only.time_to_act_steps.max | glance_cost.median | glance_cost.p95 | reading_load.max | reading_load.median | layout_shift.score |
|---|---|---|---|---|---|---|---|---|---|
| best-variant-of-author-reviewer@1440x900 | 9 (base 9) | 10 (base 10) | 0 (base 0) | 2 (base 2) | 3 (base 3) | 3 (base 3) | 270 (base 270) | 172 (base 172) | 0 (base 0) |
| best-variant-of-author-reviewer@1280x800 | 10 (base 10) | 11 (base 11) | 0 (base 0) | 2 (base 2) | 3 (base 3) | 3 (base 3) | 269 (base 269) | 147 (base 147) | 0 (base 0) |
| find-harness-kpis@1440x900 | 5 (base 5) | 6 (base 6) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 403 (base 403) | 349.5 (base 349.5) | 0 (base 0) |
| find-harness-kpis@1280x800 | 5 (base 5) | 6 (base 6) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 421 (base 421) | 273.5 (base 273.5) | 0 (base 0) |
| gate-kpis-of-authority@1440x900 | 4 (base 4) | 4 (base 4) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 2 (base 2) | 307 (base 307) | 265 (base 265) | 0 (base 0) |
| gate-kpis-of-authority@1280x800 | 4 (base 4) | 4 (base 4) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 3 (base 3) | 265 (base 265) | 215 (base 215) | 0 (base 0) |
| layer-of-workspace@1440x900 | 2 (base 2) | 3 (base 3) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 2 (base 2) | 403 (base 403) | 367 (base 367) | 0 (base 0) |
| layer-of-workspace@1280x800 | 2 (base 2) | 3 (base 3) | 0 (base 0) | 1 (base 1) | 1 (base 1) | 3 (base 3) | 397 (base 397) | 278 (base 278) | 0 (base 0) |
| toolcall-lifecycle@1440x900 | 6 (base 6) | 7 (base 7) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 403 (base 403) | 293 (base 293) | 0 (base 0) |
| toolcall-lifecycle@1280x800 | 6 (base 6) | 7 (base 7) | 0 (base 0) | 3 (base 3) | 1 (base 1) | 3 (base 3) | 421 (base 421) | 253 (base 253) | 0 (base 0) |
| what-question-depends-on@1440x900 | 4 (base 4) | 5 (base 5) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 427 (base 427) | 185 (base 185) | 0 (base 0) |
| what-question-depends-on@1280x800 | 4 (base 4) | 5 (base 5) | 0 (base 0) | 2 (base 2) | 1 (base 1) | 3 (base 3) | 419 (base 419) | 168 (base 168) | 0 (base 0) |

## Worst moments

- best-variant-of-author-reviewer 390x844 dark, completion on /docs/architectures/author-reviewer: 3 problem(s), the run does not complete (measured only). ![](shots/best-variant-of-author-reviewer-390x844-dark-16.jpg)
- find-harness-kpis 390x844 dark, after the link ^harness\b in the main on /docs/blocks/4-agent/harness: 12 problem(s) (measured only). ![](shots/find-harness-kpis-390x844-dark-10.jpg)
- layer-of-workspace 390x844 dark, completion on /docs/blocks: 2 problem(s), the run does not complete (measured only). ![](shots/layer-of-workspace-390x844-dark-04.jpg)
- toolcall-lifecycle 390x844 dark, after the link ^harness\b in the main on /docs/blocks/4-agent/harness: 12 problem(s) (measured only). ![](shots/toolcall-lifecycle-390x844-dark-10.jpg)
- best-variant-of-author-reviewer 768x1024 dark, completion on /docs: 1 problem(s), the run does not complete (measured only). ![](shots/best-variant-of-author-reviewer-768x1024-dark-02.jpg)
- find-harness-kpis 768x1024 dark, completion on /: 1 problem(s), the run does not complete (measured only). ![](shots/find-harness-kpis-768x1024-dark-02.jpg)
- gate-kpis-of-authority 390x844 dark, completion on /docs/kpis-catalogue: 1 problem(s), the run does not complete (measured only). ![](shots/gate-kpis-of-authority-390x844-dark-07.jpg)
- layer-of-workspace 768x1024 dark, completion on /: 1 problem(s), the run does not complete (measured only). ![](shots/layer-of-workspace-768x1024-dark-02.jpg)

## Completion frames

- best-variant-of-author-reviewer 1440x900 dark: ![](shots/best-variant-of-author-reviewer-1440x900-dark-10.jpg)
- best-variant-of-author-reviewer 1440x900 light: ![](shots/best-variant-of-author-reviewer-1440x900-light-10.jpg)
- find-harness-kpis 1440x900 dark: ![](shots/find-harness-kpis-1440x900-dark-06.jpg)
- find-harness-kpis 1440x900 light: ![](shots/find-harness-kpis-1440x900-light-06.jpg)
- gate-kpis-of-authority 1440x900 dark: ![](shots/gate-kpis-of-authority-1440x900-dark-04.jpg)
- gate-kpis-of-authority 1440x900 light: ![](shots/gate-kpis-of-authority-1440x900-light-04.jpg)
- layer-of-workspace 1440x900 dark: ![](shots/layer-of-workspace-1440x900-dark-03.jpg)
- layer-of-workspace 1440x900 light: ![](shots/layer-of-workspace-1440x900-light-03.jpg)
- toolcall-lifecycle 1440x900 dark: ![](shots/toolcall-lifecycle-1440x900-dark-07.jpg)
- toolcall-lifecycle 1440x900 light: ![](shots/toolcall-lifecycle-1440x900-light-07.jpg)
- what-question-depends-on 1440x900 dark: ![](shots/what-question-depends-on-1440x900-dark-05.jpg)
- what-question-depends-on 1440x900 light: ![](shots/what-question-depends-on-1440x900-light-05.jpg)

Contact sheets: ![light](shots/contact-light.jpg) ![dark](shots/contact-dark.jpg)

## Findings

As of this report; the review agents file theirs after it.

- SUF-001 S2 open: The embedded architecture frame shows the site's landing page, not the architecture, when the site is served by docusaurus serve
- SUF-002 S2 open: On an architecture page the best variant is 6 to 7 scrolls down, under a 2052 px frame, with no on-page contents to jump there
- SUF-003 S3 open: The layer map is taller than a screen, so most block cards are below the fold when the map opens
- SUF-004 S3 open: The filter chips of the KPI catalogue have names that fuse the label and the count, "gate34", "no gate80", "correctness42"
- SUF-005 S3 open: By keyboard, a block page's contents list comes after every link of the article: 87 key presses to reach the KPIs of harness, with the skip link
- SUF-006 S3 open: The KPI catalogue table scrolls sideways in its box below the gating widths, so a filtered row is never fully in view at 1024x768 and 390x844
- SUF-007 S3 open: Tables and lifecycle diagrams scroll sideways in their box at 1024x768 (15 pages) and on a phone (29 of 39 pages)
- SUF-008 S2 open: A lifecycle diagram that scrolls sideways on a phone cannot be reached by keyboard (axe-core serious, scrollable-region-focusable)
- SUF-009 S3 open: On a phone the layer map is 780 px wide in a 356 px box, so a block card and its layer label are not on the screen together
- SUF-010 S3 open: At 768x1024 the open menu draws its 637 px panel in a 352 px drawer, so its links run under the drawer's edge
- SUF-011 S3 open: In the open menu at 768x1024 the site title is cut off with an ellipsis and no way to read it whole
