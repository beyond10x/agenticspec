#!/bin/sh
# The gate of the derived examples in ui/views/examples/app: its types and unit tests (the fixture
# client), a build into target/views/examples, then its browser test (the 37-part example at
# 1366x768: the parts table on the first screen, no sideways overflow, every part in two steps,
# the reach table filtered by part, the graph with pan, zoom and fit; the workstation's labels,
# frames and actions). Playwright ends a test at 60 s and the run at 120 s. The generated apps
# themselves are held by check:generated.
set -eu
cd "$(dirname "$0")/../ui"
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit -p views/examples/app/tsconfig.json
npx vitest run --config views/examples/app/vite.config.ts --maxWorkers=2
npx vite build --config views/examples/app/vite.config.ts --logLevel warn
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config views/examples/app/e2e/playwright.config.ts --workers=2
