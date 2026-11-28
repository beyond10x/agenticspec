#!/bin/sh
# The gate of the operator UI of `agenticspec ui build`: the committed build of the operator app
# matches the digest of its sources (scripts/operator-app.sh, no node), the app's types, then
# `agenticspec ui build` of the workstation into target/operator/workstation and the browser test
# of that directory (ui/operator/e2e) over one static server and fixture adapter. Playwright ends
# a test at 60 s and the run at 120 s.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
sh scripts/operator-app.sh
cargo build --quiet --locked -p agenticspec
mkdir -p target/operator
target/debug/agenticspec ui build architectures/workstation.yaml \
  --contract ui/views/examples/workstation.contract.yaml --out target/operator/workstation
cd ui
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx vue-tsc --noEmit -p operator/tsconfig.json
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config operator/e2e/playwright.config.ts --workers=2
