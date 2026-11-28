#!/bin/sh
# The gate of the block panels' gallery in ui/views/gallery: its types and unit tests (the live
# re-read and patch, the scope of a widget, the fixture client), a build into target/views/gallery,
# then its browser test (every panel shows its fixture rows; axe-core and overflow at 1366x768 and
# 390x844 in both themes). SCREENS=1 also writes its screenshots into target/screens. Playwright
# ends a test at 60 s and the run at 120 s. The generated app itself is held by check:generated.
set -eu
cd "$(dirname "$0")/../ui"
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit -p views/gallery/tsconfig.json
npx vitest run --config views/gallery/vite.config.ts --maxWorkers=2
npx vite build --config views/gallery/vite.config.ts --logLevel warn
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config views/gallery/e2e/playwright.config.ts --workers=2
