#!/bin/sh
# Cases of scripts/change-class.sh: each side of every path list. `task check:scripts` runs it.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
fail=0

# case <expected class> <path>...: the paths, one per line, are the list of changed files.
case_() {
  want=$1
  shift
  got=$(printf '%s\n' "$@" | sh "$here/change-class.sh")
  if [ "$got" != "$want" ]; then
    echo "change-class: [$*] gave $got, expected $want" >&2
    fail=1
  fi
}

case_ planning .engineering/planning/a.md
case_ planning README.md
case_ planning AGENTS.md CHANGELOG.md
case_ docs-site docs/getting-started.md
case_ planning design/design.md
case_ planning design/query-design.md research/2026-11-26-kpi-72h.json
case_ planning research/2026-11-26-baseline.md
case_ code ui/README.md
# ux/findings records and round reports are read by the app gate (ratchet, findings rule).
case_ code ux/findings/UXF-001.md
case_ code ux/rounds/R-01/report.md
case_ docs-site docs/queries.md design/design.md
case_ docs-site website/product.json README.md
case_ code crates/sessionspec/src/main.rs
case_ code crates/sessionspec/README.md
case_ code generated/README.md
case_ code spec/domains/session.yaml
case_ code ui/sessionspec.ui.yaml
case_ code queries/session.sq
case_ code Cargo.toml
case_ code .github/workflows/pages.yml
case_ code docs/queries.md crates/sessionspec-records/src/wire.rs
case_ code ''

# An empty list (a push whose base cannot be read) is code.
got=$(printf '' | sh "$here/change-class.sh")
if [ "$got" != code ]; then
  echo "change-class: the empty list gave $got, expected code" >&2
  fail=1
fi

[ "$fail" = 0 ] && echo "change-class: every case as expected"
exit "$fail"
