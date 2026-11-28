#!/bin/sh
# Hold the manifests of the packs and fields against the root ess-inputs.yaml, and keep product
# names out of them.
#
# Rules:
#   1. packs/<pack>/pack.yaml: format agenticspec-pack/1, `pack` is the directory's name,
#      `namespace` is agentic.<word>; every file under `blocks:` exists, is listed in the root
#      ess-inputs.yaml `specification:` and declares a domain under the namespace; every file under
#      `scenarios:` exists and is listed in the root `scenarios:`; every `uses:` names a pack;
#   2. fields/<field>/field.yaml: format agenticspec-field/1, `field` is the directory's name;
#      every pack under `packs:` exists; every file under `architectures:` and `profiles:` exists,
#      and the files `kpis:`, `rows:` and `words:` name; the rows file (format agenticspec-rows/1)
#      names commands of core blocks and only their inputs: a field registers rows of core catalog
#      entities, never new semantics;
#   3. every experiments/<file>.yaml names `field:`, a field under fields/, and its `profile:` is
#      a profile that field lists;
#   4. every packs/ or fields/ file the root ess-inputs.yaml lists belongs to a manifest above, and
#      every domain the root lists under packs/ is in packs/system.yaml;
#   5. no file under packs/ or fields/ names a forge, a hosting product or an agent product: the
#      grep below prints nothing.
# Needs yq (mikefarah, v4).
#
# Usage: scripts/check-packs.sh [--path DIR]
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
[ "${1:-}" = --path ] && root="$(cd "$2" && pwd)"
command -v yq >/dev/null 2>&1 || { echo "check-packs: yq is needed" >&2; exit 2; }
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
say() { echo "check-packs: $*" >&2; echo x >>"$work/fail"; }
cd "$root"

list() {
  awk -v key="$1" '$0 ~ "^" key ":" { on = 1; next } /^[a-z_]+:/ { on = 0 } on && /^  - / { sub(/^  - /, ""); print }' ess-inputs.yaml
}
list specification | sort >"$work/spec"
list scenarios | sort >"$work/scenarios"
: >"$work/claimed"
: >"$work/profiles"

packs=0
for manifest in packs/*/pack.yaml; do
  [ -f "$manifest" ] || continue
  packs=$((packs + 1))
  dir="${manifest%/pack.yaml}"
  name="${dir#packs/}"
  [ "$(yq '.format' "$manifest")" = agenticspec-pack/1 ] || say "$manifest: format is not agenticspec-pack/1"
  [ "$(yq '.pack' "$manifest")" = "$name" ] || say "$manifest: pack is not $name"
  ns="$(yq '.namespace' "$manifest")"
  case "$ns" in agentic.[a-z]*) ;; *) say "$manifest: namespace $ns is not agentic.<word>" ;; esac
  for f in $(yq '.blocks // [] | .[]' "$manifest"); do
    path="$dir/$f"
    echo "$path" >>"$work/claimed"
    [ -f "$path" ] || { say "$manifest: block $f does not exist"; continue; }
    grep -q -x "$path" "$work/spec" || say "$manifest: block $f is not in the specification list of ess-inputs.yaml"
    domain="$(sed -n 's/^domain: *//p' "$path" | head -n 1)"
    case "$domain" in "$ns".*) ;; *) say "$path: domain $domain is not under $ns" ;; esac
  done
  for f in $(yq '.scenarios // [] | .[]' "$manifest"); do
    path="$dir/$f"
    echo "$path" >>"$work/claimed"
    [ -f "$path" ] || { say "$manifest: scenario $f does not exist"; continue; }
    grep -q -x "$path" "$work/scenarios" || say "$manifest: scenario $f is not in the scenarios list of ess-inputs.yaml"
  done
  for used in $(yq '.uses // [] | .[]' "$manifest"); do
    [ -f "packs/$used/pack.yaml" ] || say "$manifest: uses $used, which is no pack"
  done
done

fields=0
for manifest in fields/*/field.yaml; do
  [ -f "$manifest" ] || continue
  fields=$((fields + 1))
  dir="${manifest%/field.yaml}"
  name="${dir#fields/}"
  [ "$(yq '.format' "$manifest")" = agenticspec-field/1 ] || say "$manifest: format is not agenticspec-field/1"
  [ "$(yq '.field' "$manifest")" = "$name" ] || say "$manifest: field is not $name"
  for used in $(yq '.packs // [] | .[]' "$manifest"); do
    [ -f "packs/$used/pack.yaml" ] || say "$manifest: packs lists $used, which is no pack"
  done
  for f in $(yq '.architectures // [] | .[]' "$manifest"); do
    [ -f "$dir/$f" ] || say "$manifest: architecture $f does not exist"
  done
  for f in $(yq '.profiles // [] | .[]' "$manifest"); do
    [ -f "$dir/$f" ] || say "$manifest: profile $f does not exist"
    echo "$dir/$f" >>"$work/profiles"
  done
  for key in kpis rows words; do
    f="$(yq ".$key // \"\"" "$manifest")"
    [ -z "$f" ] || [ -f "$dir/$f" ] || say "$manifest: $key $f does not exist"
  done
  rows="$(yq '.rows // ""' "$manifest")"
  if [ -n "$rows" ] && [ -f "$dir/$rows" ]; then
    [ "$(yq '.format' "$dir/$rows")" = agenticspec-rows/1 ] || say "$dir/$rows: format is not agenticspec-rows/1"
    [ "$(yq '.field' "$dir/$rows")" = "$name" ] || say "$dir/$rows: field is not $name"
    n="$(yq '.rows // [] | length' "$dir/$rows")"
    i=0
    while [ "$i" -lt "$n" ]; do
      cmd="$(yq ".rows[$i].command" "$dir/$rows")"
      block="$(grep -l -x "  - name: $cmd" blocks/*/*.yaml 2>/dev/null | head -n 1 || true)"
      if [ -z "$block" ]; then
        say "$dir/$rows: row $i: $cmd is no command of a core block"
      else
        inputs="$(awk -v cmd="  - name: $cmd" '$0 == cmd { on = 1; next } on && /^  - name: / { exit } on && /^    input:/ { inp = 1; next } on && inp && /^    [a-z]/ { exit } on && inp && /name: / { sub(/.*name: */, ""); sub(/[,}].*/, ""); print }' "$block")"
        for k in $(yq ".rows[$i].input // {} | keys | .[]" "$dir/$rows"); do
          echo "$inputs" | grep -q -x "$k" || say "$dir/$rows: row $i: $cmd has no input $k"
        done
      fi
      i=$((i + 1))
    done
  fi
  for f in $(yq '.blocks // [] | .[]' "$manifest"); do
    echo "$dir/$f" >>"$work/claimed"
  done
done

# Every experiment names the field it belongs to, and runs a behaviour profile that field lists.
for e in experiments/*.yaml; do
  [ -f "$e" ] || continue
  field="$(yq '.field // ""' "$e")"
  if [ -z "$field" ] || [ ! -f "fields/$field/field.yaml" ]; then
    say "$e: field \"$field\" is no field under fields/"
    continue
  fi
  profile="$(yq '.profile // ""' "$e")"
  resolved="$(cd "$(dirname "$e")" && cd "$(dirname "$profile")" 2>/dev/null && pwd)/$(basename "$profile")"
  case "$resolved" in
    "$root/fields/$field/"*) grep -q -x "${resolved#"$root"/}" "$work/profiles" || say "$e: profile $profile is not listed by fields/$field/field.yaml" ;;
    *) say "$e: profile $profile is not a profile of the field $field" ;;
  esac
done

sort -u "$work/claimed" -o "$work/claimed"
cat "$work/spec" "$work/scenarios" | grep -E '^(packs|fields)/' | grep -v -x 'packs/system.yaml' | while read -r f; do
  grep -q -x "$f" "$work/claimed" || say "ess-inputs.yaml lists $f, which no pack.yaml or field.yaml lists"
done
grep -E '^packs/.*/blocks/' "$work/spec" | while read -r f; do
  domain="$(sed -n 's/^domain: *//p' "$f" | head -n 1)"
  grep -q -x "  - $domain" packs/system.yaml || say "packs/system.yaml does not list $domain"
done

if grep -r -n -i -E 'github|gitlab|bitbucket|gitea|claude|codex' packs fields; then
  say "packs/ or fields/ name a product (the lines above)"
fi

[ -s "$work/fail" ] && exit 1
echo "check-packs: $packs pack(s) and $fields field(s) agree with ess-inputs.yaml; no product named under packs/ or fields/"
