#!/bin/sh
# The gate of the package of ui/views: its types and unit tests (a panel mounted with a fixture
# source shows its rows; a part page sends agentic-command), the bundle of its custom elements into
# target/views/elements, the React host page into target/views/hosts, then the browser test of the
# elements in a plain page and in a React page over one fixture server (the rows, the host's
# --kit-accent, the agentic-navigate and agentic-command events). Playwright ends a test at 60 s
# and the run at 120 s.
set -eu
cd "$(dirname "$0")/../ui"
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit -p views/tsconfig.json
npx vitest run --config views/vite.config.ts --maxWorkers=2
npx vite build --config views/vite.config.ts --logLevel warn
npx vite build --config views/hosts/vite.config.ts --logLevel warn
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config views/hosts/e2e/playwright.config.ts --workers=2
