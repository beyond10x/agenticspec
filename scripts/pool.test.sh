#!/bin/sh
# Cases of scripts/pool.sh: through slot when the host has it, the command alone (the pool's flags
# dropped) when it has not, as on the forge's runner. `task check:scripts` runs it.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
fail=0

# A fake slot on PATH that prints what it was given.
mkdir -p "$here/../target"
bin=$(mktemp -d "$here/../target/pool-test.XXXXXX")
trap 'rm -rf "$bin"' EXIT
printf '#!/bin/sh\necho "slot $*"\n' > "$bin/slot"
chmod +x "$bin/slot"
nobin="$bin/none"
mkdir -p "$nobin"

# case_ <PATH> <expected output> <pool arguments>...
case_() {
  path=$1
  want=$2
  shift 2
  got=$(PATH="$path" sh -c '. "$1"; shift; pool "$@"' sh "$here/pool.sh" "$@")
  if [ "$got" != "$want" ]; then
    echo "pool: [$*] with PATH=$path gave \"$got\", expected \"$want\"" >&2
    fail=1
  fi
}

case_ "$bin:/usr/bin:/bin" "slot --here --timeout 15m echo a" --here --timeout 15m echo a
case_ "$nobin:/usr/bin:/bin" "a" --here --timeout 15m echo a
case_ "$nobin:/usr/bin:/bin" "a b" --timeout=2m --local echo a b
case_ "$nobin:/usr/bin:/bin" "--x" echo --x

if [ "$fail" -ne 0 ]; then exit 1; fi
echo "pool: 4 cases pass"
