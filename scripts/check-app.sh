#!/bin/sh
# The gate of the hand-written app in ui/app: types, unit tests (L1), a build into target/ui/www,
# then the fast loop of the ui-iteration-loop (ux/adapter.json) on the fixtures: the layout
# contract, axe-core, the keyboard and the screen-only user (L2, L3), and the ratchet against
# ux/baseline.json and the findings rule. Playwright ends a test at 60 s and the run at 120 s.
set -eu
cd "$(dirname "$0")/../ui/app"
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit
npx vitest run --maxWorkers=2
npx vite build --logLevel warn
npx playwright install --only-shell chromium >/dev/null
npx playwright test --project=ratchet --workers=2
