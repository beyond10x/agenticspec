#!/bin/sh
# A release binary names no home directory of a build machine: no user home on Linux or macOS
# and no user directory on Windows. Exit 1 and print each kind of path found, with its count,
# when one is left.
#
#   release-paths.sh <binary>...
set -eu
[ "$#" -gt 0 ] || { echo "usage: release-paths.sh <binary>..." >&2; exit 2; }
status=0
for binary in "$@"; do
  [ -f "$binary" ] || { echo "release-paths: $binary: no such file" >&2; exit 2; }
  left=0
  for pattern in '/(home|Users)/[A-Za-z0-9_.-]+/' '[A-Za-z]:\\Users\\[A-Za-z0-9_.-]+\\'; do
    count=$(LC_ALL=C grep -a -o -E "$pattern" "$binary" | wc -l | tr -d ' ')
    if [ "$count" -gt 0 ]; then
      echo "release-paths: $binary: $count match(es) of $pattern" >&2
      left=1
    fi
  done
  if [ "$left" -eq 1 ]; then status=1; else echo "release-paths: $binary: no build machine path"; fi
done
exit "$status"
