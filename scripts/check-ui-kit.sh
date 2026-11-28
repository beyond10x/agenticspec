#!/bin/sh
# The gate of the design system in ui/kit: types, unit tests, a build of its gallery into
# target/ui-kit/gallery, then the gallery's browser test (axe-core, overflow, the select's keyboard).
# SCREENS=1 also writes the gallery's screenshots into target/screens. Playwright ends a test at
# 60 s and the run at 120 s.
set -eu
cd "$(dirname "$0")/../ui"
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit -p kit/tsconfig.json
npx vitest run --config kit/vite.config.ts --maxWorkers=2
npx vite build --config kit/vite.config.ts --logLevel warn
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config kit/e2e/playwright.config.ts --workers=2
