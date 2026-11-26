#!/bin/sh
# The Measure step of a ui-iteration-loop round: builds the app, plays every scenario at every
# viewport of ux/adapter.json in both themes on the fixtures, and writes ux/rounds/<round>/
# (report.md, round.json, shots/) and a row of ux/rounds/trend.md. The review agents and the
# triage follow by hand. Usage: sh ux/round.sh R-nn
set -eu
round=${1:?usage: sh ux/round.sh R-nn}
case $round in R-[0-9][0-9]) ;; *) echo "round.sh: $round is not R-nn" >&2; exit 2 ;; esac
cd "$(dirname "$0")/.."
sh scripts/ui-build.sh
rm -rf target/ux/measure
cd ui/app
UX_ROUND=$round slot --local --timeout 10m npx playwright test --project=report --workers=2
echo "round.sh: ux/rounds/$round/report.md"
