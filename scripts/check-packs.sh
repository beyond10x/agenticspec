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
#      every pack under `packs:` exists; every file under `architectures:` exists;
#   3. every packs/ or fields/ file the root ess-inputs.yaml lists belongs to a manifest above, and
#      every domain the root lists under packs/ is in packs/system.yaml;
#   4. no file under packs/ or fields/ names a forge, a hosting product or an agent product: the
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
  for f in $(yq '.blocks // [] | .[]' "$manifest"); do
    echo "$dir/$f" >>"$work/claimed"
  done
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
