#!/bin/sh
# Hold the layering of an ESS system: every dependency between domains points down.
#
# Three tiers, each in its own directories of the specification list of ess-inputs.yaml:
#   - core: a numbered layer directory (0-foundation, 1-principals, ..., 10-evaluation), at the top
#     of the list or below blocks/; a domain's layer is the number of its directory;
#   - packs: packs/<pack>/..., concepts more than one field reuses; packs/<pack>/pack.yaml lists the
#     other packs it uses (`uses:`);
#   - fields: fields/<field>/..., one kind of work; fields/<field>/field.yaml lists the packs it
#     uses (`packs:`).
# A dependency of domain A on domain B is any of:
#   - a declared type of B used anywhere in A: an entity field, a command input, an event, error
#     or view field, or a type A declares (nested in structs, unions, lists, optionals);
#   - the identity type of an entity of B (an identifier in the shared domain counts as a
#     dependency on the domain whose entity it identifies);
#   - a relation of an entity of A whose target is an entity of B.
# Rules:
#   1. no dependency of a core domain points to a higher layer;
#   2. no core domain depends on a pack or a field;
#   3. a pack depends on the core and on the packs its pack.yaml lists, never on a field;
#   4. a field depends on the core and on the packs its field.yaml lists, never on another field;
#   5. dependencies inside one layer, one pack or one field form no cycle, and the packs' uses
#      form no cycle;
#   6. every domain of the system lives in a layer, pack or field directory.
# Read from `ess specify compile` output.
#
# Usage: scripts/check-layers.sh [--path DIR] [--graph]
#        scripts/check-layers.sh --negative   (each fixture under scripts/fixtures/layers must fail)
#        scripts/check-layers.sh --positive   (each fixture under scripts/fixtures/layers-pass must pass)
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
path="$root/blocks"
graph=0

while [ $# -gt 0 ]; do
  case "$1" in
    --path) path="$(cd "$2" && pwd)"; shift 2 ;;
    --graph) graph=1; shift ;;
    --negative)
      bad=0
      for fixture in "$root"/scripts/fixtures/layers/*/; do
        name="$(basename "$fixture")"
        "$0" --path "$fixture" > /dev/null 2>&1 && rc=0 || rc=$?
        if [ "$rc" != 1 ]; then
          echo "negative: fixture $name exited $rc, it must fail the layer check (exit 1)" >&2
          bad=1
        else
          echo "negative: fixture $name refused"
        fi
      done
      exit "$bad"
      ;;
    --positive)
      bad=0
      for fixture in "$root"/scripts/fixtures/layers-pass/*/; do
        name="$(basename "$fixture")"
        "$0" --path "$fixture" > /dev/null 2>&1 && rc=0 || rc=$?
        if [ "$rc" != 0 ]; then
          echo "positive: fixture $name exited $rc, it must pass the layer check" >&2
          bad=1
        else
          echo "positive: fixture $name passed"
        fi
      done
      exit "$bad"
      ;;
    *) echo "usage: $0 [--path DIR] [--graph] | --negative | --positive" >&2; exit 2 ;;
  esac
done

command -v yq > /dev/null 2>&1 || { echo "check-layers: yq is needed" >&2; exit 2; }
work="$root/target/check-layers/$(printf '%s' "$path" | cksum | cut -d' ' -f1)"
mkdir -p "$work"

ess specify compile --path "$path" --format json > "$work/ir.json" 2> "$work/compile.err" || {
  cat "$work/compile.err" >&2
  exit 2
}

# domain -> tier: a layer number, pack:<name>, field:<name> or none, from the directory of each
# domain file in the specification list of ess-inputs.yaml.
: > "$work/layers.txt"
awk '/^specification:/ { on = 1; next } /^[a-z_]+:/ { on = 0 } on && /^  - / { sub(/^  - /, ""); print }' "$path/ess-inputs.yaml" | while read -r file; do
  case "$file" in
    */*.yaml) ;;
    *) continue ;;
  esac
  domain="$(sed -n 's/^domain: *//p' "$path/$file" | head -n 1)"
  [ -n "$domain" ] || continue
  rel="${file#blocks/}"
  dir="${rel%%/*}"
  case "$file" in
    packs/*/*) tier="pack:$(echo "$file" | cut -d/ -f2)" ;;
    fields/*/*) tier="field:$(echo "$file" | cut -d/ -f2)" ;;
    *)
      case "$dir" in
        [0-9]-*|[0-9][0-9]-*) tier="${dir%%-*}" ;;
        *) tier=none ;;
      esac
      ;;
  esac
  printf '%s %s\n' "$domain" "$tier" >> "$work/layers.txt"
done

# what each pack and field declares it uses: "<tier> <pack>" per line
: > "$work/uses.txt"
for manifest in "$path"/packs/*/pack.yaml; do
  [ -f "$manifest" ] || continue
  name="$(basename "$(dirname "$manifest")")"
  yq -r '.uses // [] | .[]' "$manifest" | while read -r used; do
    printf 'pack:%s pack:%s\n' "$name" "$used" >> "$work/uses.txt"
  done
done
for manifest in "$path"/fields/*/field.yaml; do
  [ -f "$manifest" ] || continue
  name="$(basename "$(dirname "$manifest")")"
  yq -r '.packs // [] | .[]' "$manifest" | while read -r used; do
    printf 'field:%s pack:%s\n' "$name" "$used" >> "$work/uses.txt"
  done
done

jq -r '
  ((.domains // {}) | to_entries
    | map(.key as $d | .value | [(.types // [])[], (.entities // [])[], (.commands // [])[],
        (.events // [])[], (.errors // [])[], (.views // [])[]] | map({key: ., value: $d}))
    | add // [] | from_entries) as $declared
  | (.entities | to_entries | map({key: .value.identity.type_ref.name, value: .value.domain})
    | map(select(.key != null)) | from_entries) as $identity
  | ((.types // {}) | to_entries
      | map({key: .key, value: [.value | .. | objects | select(.kind? == "declared") | .name]})
      | from_entries) as $direct
  | def closure:
      . as $s
      | ([$s[], ($s[] | ($direct[.] // [])[])] | unique) as $n
      | if ($n | length) == ($s | length) then $s else ($n | closure) end;
  def prefix: split(".")[0:2] | join(".");
  def owner: $identity[.] // $declared[.] // prefix;
  (
    ("entities", "commands", "events", "views", "errors", "types") as $c
    | (.[$c] // {}) | to_entries[]
    | .key as $k | .value as $v
    | ($v.domain? // $declared[$k] // ($k | prefix)) as $from
    | ([$v | .. | objects | select(.kind? == "declared") | .name] | unique | closure)[]
    | owner as $to
    | select($to != $from)
    | "\($from) \($to) \($k)"
  ),
  (
    .entities as $entities
    | .entities | to_entries[] | .value as $e
    | ($e.relations // [])[]
    | ($entities[.target].domain // (.target | prefix)) as $to
    | select($to != $e.domain)
    | "\($e.domain) \($to) \($e.name).\(.name)"
  )
' "$work/ir.json" > "$work/edges.raw" || { echo "check-layers: jq failed on the compiled IR" >&2; exit 2; }
sort -u "$work/edges.raw" > "$work/edges.txt"

if [ "$graph" = 1 ]; then
  awk 'NR == FNR { layer[$1] = $2; next } { print layer[$1] " " $1 " -> " layer[$2] " " $2 }' \
    "$work/layers.txt" "$work/edges.txt" | sort | uniq -c | sort -k2,2n -k3,3
  echo
fi

status=0

awk '
  FILENAME == ARGV[1] { uses[$1 " " $2] = 1; next }
  FILENAME == ARGV[2] { layer[$1] = $2; next }
  function kind(t) { return t ~ /^pack:/ ? "pack" : t ~ /^field:/ ? "field" : t == "none" ? "none" : "core" }
  {
    from = $1; to = $2
    if (!(from in layer) || !(to in layer)) { print "unknown domain: " $0; bad++; next }
    f = layer[from]; t = layer[to]
    if (f == "none") { seen[from]++; if (seen[from] == 1) { print "no layer directory: " from; bad++ } next }
    if (t == "none" || f == t) next
    if (kind(f) == "core" && kind(t) == "core") {
      if (t + 0 > f + 0) {
        printf "upward: %s (layer %s) -> %s (layer %s) via %s\n", from, f, to, t, $3
        bad++
      }
      next
    }
    if (kind(f) == "core") {
      printf "core on %s: %s (layer %s) -> %s (%s) via %s\n", kind(t), from, f, to, t, $3
      bad++
      next
    }
    if (kind(t) == "core") next
    if (kind(t) == "field") {
      printf "%s on field: %s (%s) -> %s (%s) via %s\n", kind(f), from, f, to, t, $3
      bad++
      next
    }
    if (!((f " " t) in uses)) {
      printf "undeclared pack: %s (%s) -> %s (%s) via %s; %s does not list it\n", from, f, to, t, $3, f
      bad++
    }
  }
  END { if (bad) { printf "%d layering violation(s)\n", bad; exit 1 } }
' "$work/uses.txt" "$work/layers.txt" "$work/edges.txt" || status=1

# no cycle among the dependencies inside one layer, pack or field
awk 'NR == FNR { layer[$1] = $2; next } layer[$1] == layer[$2] && layer[$1] != "none" { print $1, $2 }' \
  "$work/layers.txt" "$work/edges.txt" | sort -u > "$work/same-layer.txt"
if [ -s "$work/same-layer.txt" ]; then
  if ! tsort "$work/same-layer.txt" > /dev/null 2> "$work/tsort.err"; then
    echo "cycle inside a layer, pack or field:" >&2
    cat "$work/tsort.err" >&2
    status=1
  fi
fi
# no cycle among the packs' uses
grep '^pack:' "$work/uses.txt" | grep ' pack:' > "$work/pack-uses.txt" || true
if [ -s "$work/pack-uses.txt" ] && ! tsort "$work/pack-uses.txt" > /dev/null 2> "$work/tsort-packs.err"; then
  echo "cycle among the packs' uses:" >&2
  cat "$work/tsort-packs.err" >&2
  status=1
fi

if [ "$status" = 0 ]; then
  core=$(awk '$2 ~ /^[0-9]+$/' "$work/layers.txt" | wc -l | tr -d ' ')
  packs=$(awk '$2 ~ /^pack:/' "$work/layers.txt" | wc -l | tr -d ' ')
  fields=$(awk '$2 ~ /^field:/' "$work/layers.txt" | wc -l | tr -d ' ')
  echo "layers: $(wc -l < "$work/layers.txt" | tr -d ' ') domains ($core core, $packs in packs, $fields in fields), $(wc -l < "$work/edges.txt" | tr -d ' ') dependencies, every one points to its own or a lower layer, the core to no pack or field, each pack and field to what it declares, no cycle"
fi
exit "$status"
