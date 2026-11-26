#!/bin/sh
# Generates the React app from ui/sessionspec.ui.yaml, bound to the `session-view` surface the
# specification serves, adds the host components of ui/host/, and builds it into target/ui/www,
# which `sessionspec serve` serves. Nothing here is committed: target/ is ignored.
set -eu
cd "$(dirname "$0")/.."
app=target/ui/app
rm -rf "$app" target/ui/www
mkdir -p target/ui
ess generate ui --target react --path ui/sessionspec.ui.yaml --model spec --out "$app"
cp ui/host/host-components.tsx "$app/src/host-components.tsx"
(cd "$app" && npm install --no-audit --no-fund --loglevel=error)
# --local: the bundle must land in this checkout; a build node keeps its output.
(cd "$app" && slot --local npm run build)
cp -r "$app/www" target/ui/www
echo "ui-build: target/ui/www"
