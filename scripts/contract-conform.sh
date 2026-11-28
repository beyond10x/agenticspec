#!/bin/sh
# Run the conformance suite of the building blocks cut to one architecture's contract on ESS's
# interpreter: the scenarios a runtime of that architecture must pass.
#
#   contract-conform.sh <architecture.yaml>
#
# The whole suite is the cached one of scripts/check-blocks-conform.sh
# (target/blocks-conform/<digest>.json), synthesized first when there is none; it is never
# synthesized here. `agenticspec sim contract --suite` cuts it to the scenarios about a command of
# a port of the contract, into target/contract-conform/<architecture>.json, and
# `ess verify conform run` runs the cut suite. Expects target/debug/agenticspec to be built
# (task contract:conform builds it).
#
# Exit status: that of `ess verify conform run` (1 when a kept scenario fails), 2 on a usage
# error or when the suite or the cut cannot be made.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

if [ $# -ne 1 ]; then
  echo "contract-conform: usage: contract-conform.sh <architecture.yaml>" >&2
  exit 2
fi
arch=$1
[ -f "$arch" ] || { echo "contract-conform: no such file: $arch" >&2; exit 2; }
bin=target/debug/agenticspec
[ -x "$bin" ] || { echo "contract-conform: $bin is not built: run task contract:conform" >&2; exit 2; }

suite=target/blocks-conform/$(scripts/check-blocks-conform.sh --digest).json
if [ ! -f "$suite" ]; then
  scripts/check-blocks-conform.sh > /dev/null || {
    echo "contract-conform: scripts/check-blocks-conform.sh failed; run it for the reason" >&2
    exit 2
  }
fi

name=$(basename "$arch" .yaml)
work=target/contract-conform
mkdir -p "$work"
cut=$work/$name.json
answer=$("$bin" sim contract "$arch" --suite "$suite" --out "$cut" --output json) || {
  echo "contract-conform: the cut failed: $answer" >&2
  exit 2
}
kept=$(printf '%s\n' "$answer" | sed -n 's/.*"kept_scenarios": *\([0-9]*\).*/\1/p')
total=$(printf '%s\n' "$answer" | sed -n 's/.*"suite_scenarios": *\([0-9]*\).*/\1/p')

status=0
ess verify conform run --target interpreted --path blocks --suite "$cut" --strict-requires \
  --report-format 2 --report-out "$work/$name.report.json" > "$work/$name.txt" 2>&1 || status=$?
grep -E '^  (passed|failed|error|unsupported) ' "$work/$name.txt" || true
tail -n 1 "$work/$name.txt"
echo "contract-conform: $arch: kept $kept of $total scenarios of the blocks' suite"
[ "$status" = 0 ] || echo "contract-conform: exit $status; report $work/$name.report.json, output $work/$name.txt" >&2
exit "$status"
