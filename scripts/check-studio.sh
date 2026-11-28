#!/bin/sh
# The gate of the studio app: its ess-ui document against the studio model, the types and unit
# tests of ui/studio (generated pages, host slots, hooks), a build into target/studio/www, then the
# browser test of crates/agenticstudio/e2e against one `agenticstudio serve` of that build, started
# once for the run on a free port. SCREENS=1 also writes the studio's screenshots into
# target/screens. Playwright ends a test at 60 s and the run at 120 s.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

ess ui check --path studio/studio.ui.yaml --model studio

version() { sed -n '/"node_modules\/@playwright\/test": {/,/}/s/.*"version": "\(.*\)".*/\1/p' "$1" | head -n 1; }
for lock in ui/package-lock.json crates/agenticstudio/e2e/package-lock.json; do
  if [ "$(version "$lock")" != "$(version ui/app/package-lock.json)" ]; then
    echo "check:studio: $lock has @playwright/test $(version "$lock"), ui/app $(version ui/app/package-lock.json); align them" >&2
    exit 1
  fi
done

(
  cd ui
  [ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
  npx vue-tsc --noEmit -p studio/tsconfig.json
  npx vitest run --config studio/vite.config.ts --maxWorkers=2
  npx vite build --config studio/vite.config.ts --logLevel warn
)

cargo build --quiet --locked -p agenticstudio

cd crates/agenticstudio/e2e
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
npx playwright install --only-shell chromium >/dev/null
npx playwright test
