#!/bin/sh
# The gate of the docs site (website/): installs its dependencies and the browser when they are
# missing, builds the site into website/build only when its inputs changed or there is no build,
# then runs the fast loop of the ui-iteration-loop (ux/site/adapter.json) on the build, served by
# docusaurus serve on a free port: the layout contract, axe-core, the keyboard, the screen-only
# user and the page sweep (L2, L3), then the ratchet against ux/site/baseline.json and the findings
# rule. Playwright ends a test at 60 s and the run at 120 s.
#
# Builds and tests go through slot where the host has it (scripts/pool.sh); on the forge's runner
# they run as they are. The browser is the one of ui/app's Playwright (the same version, checked
# below), which the forge installs with its system libraries before the gate.
#
# Usage: sh scripts/check-site.sh [--build-only]
set -eu
cd "$(dirname "$0")/.."
. scripts/pool.sh
scratch=target/site-ux
stamp=$scratch/build.digest
mkdir -p "$scratch"

inputs() {
  find docs website \( -path website/build -o -path website/node_modules -o -path website/.docusaurus \
    -o -path website/e2e -o -path website/playwright.config.ts \) -prune -o -type f -print
}

digest=$(inputs | LC_ALL=C sort | while IFS= read -r f; do sha256sum "$f"; done | sha256sum | cut -d' ' -f1)

build=no
if [ ! -f website/build/sitemap.xml ]; then
  build="no build in website/build"
elif [ -f "$stamp" ]; then
  [ "$(cat "$stamp")" = "$digest" ] || build="its inputs changed since the last build"
else
  newer=$(inputs | while IFS= read -r f; do [ "$f" -nt website/build/sitemap.xml ] && echo "$f"; done | head -n 3)
  [ -z "$newer" ] || build="inputs newer than the build: $(echo "$newer" | tr '\n' ' ')"
fi

# One Playwright for the app and the site (the versions their lockfiles install), so one browser
# serves both gates.
version() { sed -n '/"node_modules\/@playwright\/test": {/,/}/s/.*"version": "\(.*\)".*/\1/p' "$1" | head -n 1; }
site_pw=$(version website/package-lock.json)
app_pw=$(version ui/app/package-lock.json)
if [ "$site_pw" != "$app_pw" ]; then
  echo "check-site: website has @playwright/test $site_pw, ui/app $app_pw; align them" >&2
  exit 1
fi

cd website
[ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
if [ "$build" != no ]; then
  echo "check-site: building the site: $build"
  started=$(date +%s)
  pool --here --timeout 15m npm run build
  echo "check-site: site built in $(($(date +%s) - started)) s"
else
  echo "check-site: the build in website/build is current"
fi
echo "$digest" > "../$stamp"
[ "${1:-}" = --build-only ] && exit 0

npx playwright install --only-shell chromium >/dev/null
started=$(date +%s)
pool --here npx playwright test --project=ratchet --workers=4
echo "check-site: gates and ratchet in $(($(date +%s) - started)) s"
