#!/bin/sh
# Hold the KPI catalogues: kpis/catalogue.yaml (the core blocks), packs/<pack>/kpis.yaml (one per
# pack) and fields/<field>/kpis.yaml (one per field that has KPIs) parse, follow kpis/README.md,
# and name real blocks and real events.
#
# Rules, per catalogue:
#   1. format is agenticspec-kpis/1; every KPI has id, block, kind, unit, summary, how,
#      direction, gate, reads, status and source;
#   2. ids are snake_case and unique, across all catalogues;
#   3. block is one of the catalogue's blocks, and each of them has at least 2 KPIs:
#      - kpis/catalogue.yaml: a domain of blocks/system.yaml (without `agentic.`, `shared`
#        excluded) or one of the planned blocks;
#      - packs/<pack>/kpis.yaml: a domain of a block file its pack.yaml lists (without
#        `agentic.`), so `scm.change`; every pack has this file;
#      - fields/<field>/kpis.yaml: a core block or a block of a pack its field.yaml lists, the
#        block whose events it reads, with no least number per block;
#      a core KPI whose block is a field is refused with a pointer to the field's catalogue;
#   4. kind, unit, direction, status and each mode value come from their lists; a correctness KPI
#      is a gate unless it says `gate: false`; source is `design`, `docs/kpis.md` or
#      `research/<id>`;
#   5. every event in `reads` is declared in the `events:` section of a block the catalogue may
#      read, or of the session IR: the core reads core blocks; a pack reads core blocks, its own
#      and those of the packs its pack.yaml `uses:`; a field reads core blocks and those of
#      the packs its field.yaml lists.
# Needs yq (mikefarah, v4).
#
# Usage: scripts/check-kpis.sh          every catalogue
#        scripts/check-kpis.sh FILE     FILE as the core catalogue
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
planned="verification memory plan monitor"

command -v yq >/dev/null 2>&1 || { echo "check-kpis: yq is needed" >&2; exit 2; }
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
say() { echo "check-kpis: $*" >&2; echo x >>"$work/fail"; }

# events <file>...: the events declared in the `events:` section of each file
events() {
  for f in "$@"; do
    [ -f "$f" ] || continue
    awk '/^events:/ {e=1; next} /^[a-z]/ {e=0} e && /^  - name: / {print $3}' "$f"
  done
}

# check <catalogue> <blocks> <events file> [min]: rules 1 and 3 to 5 for one catalogue, with at least
# <min> KPIs per block (2 when not given); ids go to $work/ids
check() {
  file=$1; blocks=$2; events=$3; min=${4:-2}
  rel="${file#"$root"/}"
  yq '.format' "$file" >"$work/format" 2>"$work/err" || { cat "$work/err" >&2; echo x >>"$work/fail"; return; }
  [ "$(cat "$work/format")" = "agenticspec-kpis/1" ] || say "$rel: format is not agenticspec-kpis/1"

  yq '.kpis[] | .id + " " + .block' "$file" >"$work/id-block"
  yq '.kpis[] | .id + " " + .kind + " " + .unit + " " + .direction + " " + .status + " " + (.gate | tag)' "$file" >"$work/fields"
  yq '.kpis[] | .id + " " + .kind + " " + .gate' "$file" >"$work/gates"
  yq '.kpis[] | .id + " " + .source' "$file" >"$work/sources"
  yq '.kpis[] | select(.summary == null or .how == null or .reads == null) | .id' "$file" >"$work/missing"
  yq '.kpis[] | .id + " " + (.modes.autonomous // "-") + " " + (.modes.pairing // "-") + " " + (.modes.review // "-") + " " + ((.modes // {}) | keys | join(","))' "$file" >"$work/modes"
  yq '.kpis[] | .id + " " + (.reads | join(" "))' "$file" >"$work/reads"
  cut -d' ' -f1 "$work/id-block" >>"$work/ids"

  while read -r id; do
    say "$rel: $id: summary, how or reads missing"
  done <"$work/missing"

  cut -d' ' -f1 "$work/id-block" | grep -v -E '^[a-z][a-z0-9_]*$' | while read -r id; do
    say "$rel: id $id is not snake_case"
  done

  while read -r id block; do
    case " $(echo $blocks) " in
      *" $block "*) ;;
      *)
        if [ -f "$root/fields/$block/field.yaml" ]; then
          say "$rel: $id: block $block is a field; a KPI of a field goes in fields/$block/kpis.yaml and names the block it reads"
        else
          say "$rel: $id: block $block is not a block of this catalogue"
        fi ;;
    esac
  done <"$work/id-block"
  if [ "$min" -gt 0 ]; then for b in $blocks; do
    n="$(cut -d' ' -f2 "$work/id-block" | grep -c -x "$b" || true)"
    [ "$n" -ge "$min" ] || say "$rel: block $b has $n KPIs, needs at least $min"
  done; fi

  while read -r id kind unit direction status gatetag; do
    case "$kind" in correctness|completeness|quality) ;; *) say "$rel: $id: kind $kind" ;; esac
    case "$unit" in share|count|seconds|usd|tokens|ratio|points) ;; *) say "$rel: $id: unit $unit" ;; esac
    case "$direction" in higher|lower) ;; *) say "$rel: $id: direction $direction" ;; esac
    case "$status" in computed|simulated|both|planned) ;; *) say "$rel: $id: status $status" ;; esac
    case "$gatetag" in '!!bool') ;; *) say "$rel: $id: gate is not a boolean" ;; esac
  done <"$work/fields"

  while read -r id kind gate; do
    # A gate on a quality KPI makes no sense; a completeness gate is a share that must be 1.
    if [ "$kind" = quality ] && [ "$gate" = true ]; then
      say "$rel: $id: a quality KPI is a gate"
    fi
  done <"$work/gates"

  while read -r id source; do
    case "$source" in
      design|docs/kpis.md) ;;
      research/[0-9][0-9][0-9][0-9].[0-9][0-9][0-9][0-9][0-9]) ;;
      *) say "$rel: $id: source $source" ;;
    esac
  done <"$work/sources"

  while read -r id a p r keys; do
    for v in $a $p $r; do
      case "$v" in higher|lower|ignore|-) ;; *) say "$rel: $id: mode value $v" ;; esac
    done
    for k in $(echo "${keys:-}" | tr ',' ' '); do
      case "$k" in autonomous|pairing|review) ;; *) say "$rel: $id: mode $k" ;; esac
    done
  done <"$work/modes"

  while read -r id evs; do
    for event in $evs; do
      grep -q -x "$event" "$events" || say "$rel: $id: reads $event, which is not a declared event it may read"
    done
  done <"$work/reads"
  echo "$(wc -l <"$work/id-block" | tr -d ' ') $rel" >>"$work/counts"
}

: >"$work/ids"
: >"$work/counts"
events "$root"/blocks/*/*.yaml "$root"/spec/domains/session.yaml | sort -u >"$work/core-events"

core_blocks="$(yq '.domains[]' "$root/blocks/system.yaml" | sed -n 's/^agentic\.//p' | grep -v '^shared$') $planned"
check "${1:-$root/kpis/catalogue.yaml}" "$core_blocks" "$work/core-events"

if [ $# -eq 0 ]; then
  for manifest in "$root"/packs/*/pack.yaml; do
    [ -f "$manifest" ] || continue
    dir="$(dirname "$manifest")"
    pack="$(basename "$dir")"
    if [ ! -f "$dir/kpis.yaml" ]; then
      say "packs/$pack has no kpis.yaml"
      continue
    fi
    files="$(yq '.blocks // [] | .[]' "$manifest" | sed "s#^#$dir/#")"
    pack_blocks="$(for f in $files; do sed -n 's/^domain: *agentic\.//p' "$f" | head -n 1; done)"
    [ -n "$pack_blocks" ] || say "packs/$pack: pack.yaml lists no block"
    used="$(yq '.uses // [] | .[]' "$manifest")"
    used_files="$(for u in $used; do yq '.blocks // [] | .[]' "$root/packs/$u/pack.yaml" | sed "s#^#$root/packs/$u/#"; done)"
    { cat "$work/core-events"; events $files $used_files; } | sort -u >"$work/pack-events"
    check "$dir/kpis.yaml" "$pack_blocks" "$work/pack-events"
  done

  for manifest in "$root"/fields/*/field.yaml; do
    [ -f "$manifest" ] || continue
    dir="$(dirname "$manifest")"
    [ -f "$dir/kpis.yaml" ] || continue
    field_files=""
    for p in $(yq '.packs // [] | .[]' "$manifest"); do
      field_files="$field_files $(yq '.blocks // [] | .[]' "$root/packs/$p/pack.yaml" | sed "s#^#$root/packs/$p/#")"
    done
    field_blocks="$core_blocks $(for f in $field_files; do sed -n 's/^domain: *agentic\.//p' "$f" | head -n 1; done)"
    { cat "$work/core-events"; events $field_files; } | sort -u >"$work/field-events"
    check "$dir/kpis.yaml" "$field_blocks" "$work/field-events" 0
  done
fi

sort "$work/ids" | uniq -d | while read -r id; do
  say "duplicate id $id"
done

if [ -s "$work/fail" ]; then
  exit 1
fi
echo "check-kpis: $(awk '{ n += $1 } END { print n }' "$work/counts") KPIs ok in $(wc -l <"$work/counts" | tr -d ' ') catalogue(s): $(awk '{ printf "%s%s %s", (NR > 1 ? ", " : ""), $2, $1 }' "$work/counts")"
