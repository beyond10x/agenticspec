#!/bin/sh
# The operator app's committed build, ui/operator/built (index.html, app.js, app.css), which the
# agenticspec binary embeds for `agenticspec ui build`, and its digest file,
# ui/operator/built.sources.sha256: one `<sha256>  <path>` line per source of the build (the app
# in ui/operator without its browser test, the kit's and the views package's sources, the
# generated panels, the npm manifest and lock file) and per built file, sorted by path.
#
#   operator-app.sh           compare the digest of those files with the digest file, no node
#   operator-app.sh --write   build the app into ui/operator/built and write the digest file
#   operator-app.sh --slow    build the app afresh into target/operator/fresh and compare its
#                             bytes with ui/operator/built, then the digest as without arguments
#
# Exit 0: one line `operator-app: ...` that names what holds. Exit 1: each difference is named
# with the task that rebuilds (task operator:build).
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

built=ui/operator/built
digest_file=ui/operator/built.sources.sha256

paths() {
  {
    find ui/operator -type f ! -path "$built/*" ! -path 'ui/operator/e2e/*' ! -path "$digest_file"
    find ui/kit/src ui/views/src ui/views/gallery/src/generated -type f
    echo ui/package.json
    echo ui/package-lock.json
    find "$built" -type f
  } | LC_ALL=C sort
}

digest() {
  paths | while IFS= read -r f; do sha256sum "$f"; done
}

build() {
  (
    cd ui
    [ node_modules/.package-lock.json -nt package-lock.json ] || npm ci --no-audit --no-fund --loglevel=error
    OPERATOR_OUT="$1" npx vite build --config operator/vite.config.ts --logLevel warn
  )
}

mode=${1:-check}
case $mode in
  --write)
    build "$root/$built"
    digest > "$digest_file"
    echo "operator-app: $built written ($(du -cb "$built"/* | tail -n 1 | cut -f1) bytes), $digest_file with $(grep -c . "$digest_file") lines"
    exit 0
    ;;
  --slow)
    fresh=target/operator/fresh
    build "$root/$fresh"
    fail=0
    for f in $(cd "$built" && ls) $(cd "$fresh" && ls); do
      if ! cmp -s "$built/$f" "$fresh/$f"; then
        echo "operator-app: $built/$f differs from a fresh build"
        fail=1
      fi
    done
    [ "$fail" = 0 ] || { echo "operator-app: run task operator:build"; exit 1; }
    echo "operator-app: a fresh build equals $built byte for byte"
    ;;
  check) ;;
  *) echo "operator-app: unknown argument $mode" >&2; exit 2 ;;
esac

[ -f "$digest_file" ] || { echo "operator-app: $digest_file is missing; run task operator:build"; exit 1; }
if ! digest | cmp -s - "$digest_file"; then
  digest | diff "$digest_file" - | grep '^[<>]' | sed 's/^/operator-app: /' | head -n 20
  echo "operator-app: the sources or the build of the operator app changed since $built was built; never edit it: run task operator:build"
  exit 1
fi
echo "operator-app: $built matches the digest of its $(grep -c . "$digest_file") sources and files"
