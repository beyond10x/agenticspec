#!/bin/sh
# Run the conformance suite of the building blocks (blocks/) on ESS's interpreter, with every
# authored scenario that blocks/ess-inputs.yaml lists, inside the gate's budget. With `--path .`,
# the same for the whole system: the core and the packs, from the root ess-inputs.yaml.
#
# Synthesis is the expensive part: for blocks/ about 60 s on one thread and 29 s on four for 623
# scenarios; for the whole system about 85-100 s on four for 909 scenarios. Running the suite
# takes about 2-6 s. So the suite is synthesized once per state of its sources and kept in a cache
# on disk, keyed by the digest of those sources:
#
#   target/blocks-conform/<digest>.json     (blocks/)
#   target/system-conform/<digest>.json     (--path .)
#
# The digest is the SHA-256 over the synthesis line below, the `ess --version` line, the
# ess-inputs.yaml and every file it lists under `specification:` and `scenarios:`, each with its
# path. The suite is not committed: compact, it is about 9 MB (blocks/) and 17 MB (system).
#
# `--scenarios <path>` reads the `scenarios:` list of <path>/ess-inputs.yaml, so authored
# scenarios in any directory below it are compiled into the suite.
#
# With `--path .`, packs/known-synthesis-defects.txt lists synthesized scenarios ESS 0.99.2's
# interpreter fails because synthesis arranged them wrong, each with the authored scenario that
# walks its path. The run passes when the failed scenarios are exactly those; one more failure, or
# a listed scenario that passes, fails it.
#
#   check-blocks-conform.sh [--path DIR]           run the cached suite; synthesize it first when
#                                                  there is none
#   check-blocks-conform.sh [--path DIR] --slow    synthesize afresh, compare bytes with the cached
#                                                  suite of the same digest when there is one, keep
#                                                  it, then run it
#   check-blocks-conform.sh [--path DIR] --digest  print the digest
#
# Exit status: that of `ess verify conform run` (1 when a scenario fails beyond the known list),
# 1 when --slow finds a cached suite that differs from a fresh synthesis, 2 when synthesis fails.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

spec=blocks
mode=run
while [ $# -gt 0 ]; do
  case $1 in
    --path) spec=${2%/}; shift 2 ;;
    run | --slow | --digest) mode=$1; shift ;;
    *) echo "check-blocks-conform: unknown argument $1" >&2; exit 2 ;;
  esac
done
case $spec in
  blocks) work=target/blocks-conform; known= ;;
  .) work=target/system-conform; known=packs/known-synthesis-defects.txt ;;
  *) echo "check-blocks-conform: --path is blocks or ." >&2; exit 2 ;;
esac
jobs=${BLOCKS_CONFORM_JOBS:-4}
synth="ess verify conform synthesize --path $spec --scenarios $spec --compact --strict-requires"

# The files the suite is made of, relative to the specification: ess-inputs.yaml and its two lists.
sources() {
  echo ess-inputs.yaml
  awk '/^(specification|scenarios):/ { on = 1; next } /^[a-z_]+:/ { on = 0 } on && /^  - / { sub(/^  - /, ""); print }' \
    "$spec/ess-inputs.yaml"
}

digest() {
  {
    printf '%s\n' "$synth"
    ess --version
    sources | while IFS= read -r file; do
      printf '%s %s\n' "$(sha256sum < "$spec/$file" | cut -d' ' -f1)" "$file"
    done
  } | sha256sum | cut -d' ' -f1
}

missing=$(sources | while IFS= read -r file; do [ -f "$spec/$file" ] || echo "$spec/$file"; done)
if [ -n "$missing" ]; then
  echo "check-blocks-conform: listed in $spec/ess-inputs.yaml and missing: $missing" >&2
  exit 2
fi
key=$(digest)
if [ "$mode" = --digest ]; then
  echo "$key"
  exit 0
fi

mkdir -p "$work"
suite=$work/$key.json

# synthesize <file>: write a fresh suite into <file>, its log beside it.
synthesize() {
  start=$(date +%s)
  if ! $synth --jobs "$jobs" --out "$1" > "$1.log" 2>&1; then
    tail -n 20 "$1.log" >&2
    echo "check-blocks-conform: synthesis failed: $synth" >&2
    exit 2
  fi
  echo "check-blocks-conform: synthesized in $(( $(date +%s) - start )) s: $(tail -n 1 "$1.log")"
}

if [ "$mode" = --slow ]; then
  synthesize "$work/fresh.json"
  if [ -f "$suite" ]; then
    if cmp -s "$work/fresh.json" "$suite"; then
      echo "check-blocks-conform: the cached suite $suite is byte for byte a fresh synthesis"
    else
      echo "check-blocks-conform: the cached suite $suite differs from a fresh synthesis of the same sources" >&2
      exit 1
    fi
  fi
  mv "$work/fresh.json" "$suite"
  mv "$work/fresh.json.log" "$suite.log"
elif [ -f "$suite" ]; then
  echo "check-blocks-conform: suite $key from the cache"
else
  synthesize "$work/next.json"
  mv "$work/next.json" "$suite"
  mv "$work/next.json.log" "$suite.log"
fi

# Keep one suite: the cache of other digests is stale.
for old in "$work"/*.json; do
  case $old in "$suite" | */report.json) ;; *) rm -f "$old" "$old.log" ;; esac
done

status=0
ess verify conform run --target interpreted --path "$spec" --suite "$suite" --strict-requires \
  --report-format 2 --report-out "$work/report.json" > "$work/run.txt" 2>&1 || status=$?

if [ -n "$known" ] && [ "$status" = 0 ] && grep -q -v -e "^#" -e "^$" "$known"; then
  echo "check-blocks-conform: every scenario passed, so $known lists defects that are gone: empty it" >&2
  status=1
elif [ "$status" = 1 ] && [ -n "$known" ]; then
  sed -n 's/^  failed //p; s/^  error //p' "$work/run.txt" | sort > "$work/failed.txt"
  grep -v -e '^#' -e '^$' "$known" | cut -d' ' -f1 | sort > "$work/known.txt"
  if cmp -s "$work/failed.txt" "$work/known.txt"; then
    tail -n 1 "$work/run.txt"
    echo "check-blocks-conform: the $(wc -l < "$work/known.txt" | tr -d ' ') failed scenarios are exactly the known synthesis defects of $known"
    exit 0
  fi
  comm -23 "$work/failed.txt" "$work/known.txt" | sed 's/^/  failed, not known: /'
  comm -13 "$work/failed.txt" "$work/known.txt" | sed "s#^#  known, did not fail (remove it from $known): #"
else
  grep -E '^  (failed|error|unsupported) ' "$work/run.txt" || true
fi
tail -n 1 "$work/run.txt"
[ "$status" = 0 ] || echo "check-blocks-conform: exit $status; report $work/report.json, output $work/run.txt" >&2
exit "$status"
