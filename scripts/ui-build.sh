#!/bin/sh
# Builds the hand-written UI of ui/app/ into target/ui/www, which `sessionspec serve` serves. Its
# types come from the specification (ui/app/src/generated/, written by `task generate` and held by
# `task check:generated`); nothing in the app is generated. target/ is ignored.
set -eu
cd "$(dirname "$0")/../ui/app"
npm ci --no-audit --no-fund --loglevel=error
# --local: the bundle must land in this checkout; a build node keeps its output.
slot --local npm run build
echo "ui-build: target/ui/www"
