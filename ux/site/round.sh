#!/bin/sh
# The Measure step of a ui-iteration-loop round of the docs site: builds the site when its inputs
# changed, plays every docs task at every viewport of ux/site/adapter.json in both themes, sweeps
# every page of the sitemap, and writes ux/site/rounds/<round>/ (report.md, round.json, shots/)
# and a row of ux/site/rounds/trend.md. The review agents and the triage follow by hand.
# Through slot where the host has it (scripts/pool.sh), with its outputs here (--here).
# Usage: sh ux/site/round.sh S-nn
set -eu
round=${1:?usage: sh ux/site/round.sh S-nn}
case $round in S-[0-9][0-9]) ;; *) echo "round.sh: $round is not S-nn" >&2; exit 2 ;; esac
cd "$(dirname "$0")/../.."
. scripts/pool.sh
sh scripts/check-site.sh --build-only
rm -rf target/site-ux/measure
cd website
npx playwright install --only-shell chromium >/dev/null
SITE_UX_ROUND=$round pool --here --timeout 15m npx playwright test --project=report --workers=4
echo "round.sh: ux/site/rounds/$round/report.md"
