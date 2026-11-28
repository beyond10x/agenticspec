#!/bin/sh
# Case of scripts/spec-diff.sh: blocks/ against itself is compatible, and a copy with one enum
# variant removed is breaking (exit 4). `task check:scripts` runs it; about 2 s.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sh "$here/spec-diff.sh" --self-test
