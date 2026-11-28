#!/bin/sh
# Keep the core neutral: no word of a field of work in the core blocks or the core KPI catalogue.
#
# Every field lists the words only it uses in fields/<field>/words.txt, one word or phrase per
# line. This check refuses each of them, matched as a whole word in any case, in every file under
# blocks/ (blocks, scenarios, comments) and in kpis/catalogue.yaml. A concept the core needs that a
# field names in its own words gets a neutral name in the core, or a catalog entity whose rows the
# field registers.
#
# Usage: scripts/check-neutral.sh [--path DIR]   check DIR (default: the repository)
#        scripts/check-neutral.sh --negative     every fixture under scripts/fixtures/neutral must
#                                                fail
set -eu

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"

if [ "${1:-}" = --negative ]; then
  n=0
  for d in "$here"/fixtures/neutral/*/; do
    [ -d "$d" ] || continue
    n=$((n + 1))
    if sh "$here/check-neutral.sh" --path "$d" >/dev/null 2>&1; then
      echo "check-neutral: fixture $(basename "$d") passed, but must fail" >&2
      exit 1
    fi
  done
  [ "$n" -gt 0 ] || { echo "check-neutral: no fixture under scripts/fixtures/neutral" >&2; exit 1; }
  echo "check-neutral: $n refusing fixture(s) refused"
  exit 0
fi
[ "${1:-}" = --path ] && root="$(cd "$2" && pwd)"
cd "$root"

core=""
[ -d blocks ] && core="blocks"
[ -f kpis/catalogue.yaml ] && core="$core kpis/catalogue.yaml"
[ -n "$core" ] || { echo "check-neutral: no blocks/ and no kpis/catalogue.yaml under $root" >&2; exit 2; }

fail=0
fields=0
words=0
for list in fields/*/words.txt; do
  [ -f "$list" ] || continue
  fields=$((fields + 1))
  field="${list#fields/}"
  field="${field%/words.txt}"
  while IFS= read -r word || [ -n "$word" ]; do
    [ -n "$word" ] || continue
    words=$((words + 1))
    # shellcheck disable=SC2086
    if hits="$(grep -r -n -i -w -F -e "$word" $core)"; then
      echo "$hits" | sed "s/^/check-neutral: field $field word \"$word\": /" >&2
      fail=1
    fi
  done <"$list"
done

[ "$fail" -eq 0 ] || { echo "check-neutral: the core names words of a field (the lines above)" >&2; exit 1; }
echo "check-neutral: no word of $fields field(s) ($words words) in $core"
