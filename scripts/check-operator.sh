#!/bin/sh
# The gate of the operator UI of `agenticspec ui build`: the committed build of the operator app
# matches the digest of its sources (scripts/operator-app.sh, no node), the app's types, then
# `agenticspec ui build` of the workstation into target/operator/workstation and the browser test
# of that directory (ui/operator/e2e) over one static server and fixture adapter, and of one
# `agenticspec ui serve` of the workstation. Playwright ends a test at 60 s and the run at 120 s.
# Beside it, the project `ui generate --project` writes for the workstation is built with vite over
# ui/node_modules, and its packer is held to the bytes of `ui build --archive`.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
sh scripts/operator-app.sh
cargo build --quiet --locked -p agenticspec
mkdir -p target/operator
target/debug/agenticspec ui build architectures/workstation.yaml \
  --contract ui/views/examples/workstation.contract.yaml --out target/operator/workstation --archive >/dev/null
cd ui
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
cd "$root"
# The project of `ui generate --project` builds with the packages its lock pins (linked from
# ui/node_modules, the same versions, so no network), and its packer writes the bytes of
# `ui build --archive` for the same files.
project=target/operator/project
rm -rf "$project" target/operator/workstation.ref.tar
target/debug/agenticspec ui generate --path ui/views/examples/workstation.ui.yaml --out "$project" --project >/dev/null
ln -s "$root/ui/node_modules" "$project/node_modules"
(cd "$project" && npx vite build --logLevel warn && node pack.mjs dist >/dev/null)
for f in index.html app.js app.css; do
  [ -s "$project/dist/$f" ] || { echo "check-operator: ui generate --project built no dist/$f"; exit 1; }
done
cp target/operator/workstation.tar target/operator/workstation.ref.tar
node "$project/pack.mjs" target/operator/workstation >/dev/null
cmp -s target/operator/workstation.ref.tar target/operator/workstation.tar ||
  { echo "check-operator: pack.mjs of ui generate --project does not write the bytes of ui build --archive"; exit 1; }
echo "check-operator: the project of ui generate --project builds, and its packer writes the bytes of ui build --archive"
cd ui
npx vue-tsc --noEmit -p operator/tsconfig.json
npx playwright install --only-shell chromium >/dev/null
npx playwright test --config operator/e2e/playwright.config.ts --workers=2
