#!/bin/sh
# Classify the change of each ESS system of this repository (blocks/, spec/, studio/) since a git
# revision, by default the newest release tag: `ess verify diff --compatibility` rates each change
# breaking, unknown or compatible for callers, readers and history. A narrowing (a required input
# added, an enum variant a closed reader does not know, a refusal widened) is breaking even where
# it reads as an addition.
#
# The revision is materialised with `git archive` under target/spec-diff/<revision>/, so the
# working tree is compared as it is, uncommitted changes included.
#
#   spec-diff.sh [<revision>]       one summary line per system and every breaking or unknown change
#   spec-diff.sh --self-test        compare blocks/ with itself (exit 0), then a copy with one enum
#                                   variant removed (exit 4 naming the change)
#
# Exit 0: every change compatible. 4: a breaking or unknown change (none is acknowledged here; an
# acknowledgement is bound to both endpoint digests). 1: a refused input. 2: a usage error.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"
work=target/spec-diff

# classify <from dir> <to dir> <label>: prints the summary and the breaking or unknown changes,
# returns the exit status of `ess verify diff --fail-on breaking-or-unknown`.
classify() {
  status=0
  ess verify diff --from "$1" --to "$2" --fail-on breaking-or-unknown --format json \
    > "$work/$3.json" 2> "$work/$3.err" || status=$?
  if [ "$status" != 0 ] && [ "$status" != 4 ]; then
    cat "$work/$3.err" >&2
    return "$status"
  fi
  jq -r --arg label "$3" '
    [.changes[] | .compatibility.verdict] as $v
    | "spec-diff: \($label): \($v | length) change(s): \([$v[] | select(. == "breaking")] | length) breaking, \([$v[] | select(. == "unknown")] | length) unknown, \([$v[] | select(. == "compatible")] | length) compatible"' \
    "$work/$3.json"
  jq -r '.changes[] | select(.compatibility.verdict != "compatible")
    | "  \(.compatibility.verdict) (callers \(.compatibility.callers), readers \(.compatibility.readers), history \(.compatibility.history)\(if .compatibility.narrows then ", narrows " + .compatibility.narrows else "" end)): \(.id)"' \
    "$work/$3.json"
  return "$status"
}

if [ "${1:-}" = --self-test ]; then
  mkdir -p "$work"
  rm -rf "$work/self"
  mkdir -p "$work/self"
  classify blocks blocks self-same || { echo "spec-diff: blocks/ against itself is not compatible" >&2; exit 1; }
  cp -R blocks "$work/self/blocks"
  # Plant: remove the variant `Spec` of agentic.architecture.Carries in the copy.
  sed -i 's/^\(    variants: \[Prompt, .*\)Spec, /\1/' "$work/self/blocks/9-composition/architecture.yaml"
  if cmp -s blocks/9-composition/architecture.yaml "$work/self/blocks/9-composition/architecture.yaml"; then
    echo "spec-diff: the planted change did not apply" >&2
    exit 1
  fi
  status=0
  classify blocks "$work/self/blocks" self-planted > "$work/self-planted.txt" || status=$?
  if [ "$status" = 4 ] && grep -q 'breaking.*variant-removed' "$work/self-planted.txt"; then
    echo "spec-diff: self-test: blocks/ against itself compatible; a removed enum variant is breaking (exit 4)"
    exit 0
  fi
  cat "$work/self-planted.txt" >&2
  echo "spec-diff: self-test: the removed enum variant did not fail (exit $status)" >&2
  exit 1
fi

rev=${1:-$(git describe --tags --abbrev=0 --match '[0-9]*.[0-9]*.[0-9]*')}
base=$work/$rev
rm -rf "$base"
mkdir -p "$base"
git archive "$rev" blocks spec studio | tar -x -C "$base"
result=0
for system in blocks spec studio; do
  if [ ! -d "$base/$system" ]; then
    echo "spec-diff: $system: absent at $rev"
    continue
  fi
  status=0
  classify "$base/$system" "$system" "$system" || status=$?
  case $status in
    0) ;;
    4) result=4 ;;
    *) exit 1 ;;
  esac
done
echo "spec-diff: compared with $rev"
exit "$result"
