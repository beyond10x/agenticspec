#!/bin/sh
# Hold the KPI catalogue: kpis/catalogue.yaml parses, follows kpis/README.md, and names real
# blocks and real events.
#
# Rules:
#   1. format is agenticspec-kpis/1; every KPI has id, block, kind, unit, summary, how,
#      direction, gate, reads, status and source;
#   2. ids are snake_case and unique;
#   3. block is a domain of blocks/system.yaml (without `agentic.`, `shared` excluded) or one of
#      the planned blocks; every block of the system has at least 2 KPIs;
#   4. kind, unit, direction, status and each mode value come from their lists; a correctness KPI
#      is a gate unless it says `gate: false`; source is `design`, `docs/kpis.md` or
#      `research/<id>`;
#   5. every event in `reads` is declared in the `events:` section of a block or of the session IR.
# Needs yq (mikefarah, v4).
#
# Usage: scripts/check-kpis.sh [FILE]
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
file="${1:-$root/kpis/catalogue.yaml}"
planned="verification memory plan monitor"

command -v yq >/dev/null 2>&1 || { echo "check-kpis: yq is needed" >&2; exit 2; }
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
say() { echo "check-kpis: $*" >&2; echo x >>"$work/fail"; }

yq '.format' "$file" >"$work/format" 2>"$work/err" || { cat "$work/err" >&2; exit 1; }
[ "$(cat "$work/format")" = "agenticspec-kpis/1" ] || say "format is not agenticspec-kpis/1"

yq '.kpis[] | .id + " " + .block' "$file" >"$work/id-block"
yq '.kpis[] | .id + " " + .kind + " " + .unit + " " + .direction + " " + .status + " " + (.gate | tag)' "$file" >"$work/fields"
yq '.kpis[] | .id + " " + .kind + " " + .gate' "$file" >"$work/gates"
yq '.kpis[] | .id + " " + .source' "$file" >"$work/sources"
yq '.kpis[] | select(.summary == null or .how == null or .reads == null) | .id' "$file" >"$work/missing"
yq '.kpis[] | .id + " " + (.modes.autonomous // "-") + " " + (.modes.pairing // "-") + " " + (.modes.review // "-") + " " + ((.modes // {}) | keys | join(","))' "$file" >"$work/modes"
yq '.kpis[] | .id + " " + (.reads | join(" "))' "$file" >"$work/reads"

while read -r id; do
  say "$id: summary, how or reads missing"
done <"$work/missing"

cut -d' ' -f1 "$work/id-block" | sort | uniq -d | while read -r id; do
  say "duplicate id $id"
done
cut -d' ' -f1 "$work/id-block" | grep -v -E '^[a-z][a-z0-9_]*$' | while read -r id; do
  say "id $id is not snake_case"
done

blocks="$(yq '.domains[]' "$root/blocks/system.yaml" | sed -n 's/^agentic\.//p' | grep -v '^shared$')"
while read -r id block; do
  case " $(echo $blocks) $planned " in
    *" $block "*) ;;
    *) say "$id: block $block is not a block" ;;
  esac
done <"$work/id-block"
for b in $blocks $planned; do
  n="$(cut -d' ' -f2 "$work/id-block" | grep -c -x "$b" || true)"
  [ "$n" -ge 2 ] || say "block $b has $n KPIs, needs at least 2"
done

while read -r id kind unit direction status gatetag; do
  case "$kind" in correctness|completeness|quality) ;; *) say "$id: kind $kind" ;; esac
  case "$unit" in share|count|seconds|usd|tokens|ratio|points) ;; *) say "$id: unit $unit" ;; esac
  case "$direction" in higher|lower) ;; *) say "$id: direction $direction" ;; esac
  case "$status" in computed|simulated|both|planned) ;; *) say "$id: status $status" ;; esac
  case "$gatetag" in '!!bool') ;; *) say "$id: gate is not a boolean" ;; esac
done <"$work/fields"

while read -r id kind gate; do
  # A gate on a quality KPI makes no sense; a completeness gate is a share that must be 1.
  if [ "$kind" = quality ] && [ "$gate" = true ]; then
    say "$id: a quality KPI is a gate"
  fi
done <"$work/gates"

while read -r id source; do
  case "$source" in
    design|docs/kpis.md) ;;
    research/[0-9][0-9][0-9][0-9].[0-9][0-9][0-9][0-9][0-9]) ;;
    *) say "$id: source $source" ;;
  esac
done <"$work/sources"

while read -r id a p r keys; do
  for v in $a $p $r; do
    case "$v" in higher|lower|ignore|-) ;; *) say "$id: mode value $v" ;; esac
  done
  for k in $(echo "${keys:-}" | tr ',' ' '); do
    case "$k" in autonomous|pairing|review) ;; *) say "$id: mode $k" ;; esac
  done
done <"$work/modes"

# Events declared under `events:` in each block file and in the session IR.
for f in "$root"/blocks/*/*.yaml "$root"/spec/domains/session.yaml; do
  awk '/^events:/ {e=1; next} /^[a-z]/ {e=0} e && /^  - name: / {print $3}' "$f"
done | sort -u >"$work/events"
while read -r id events; do
  for event in $events; do
    grep -q -x "$event" "$work/events" || say "$id: reads $event, which is not a declared event"
  done
done <"$work/reads"

if [ -s "$work/fail" ]; then
  exit 1
fi
echo "check-kpis: $(wc -l <"$work/id-block" | tr -d ' ') KPIs ok"
