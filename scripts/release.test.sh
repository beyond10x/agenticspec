#!/bin/sh
# Cases of the release scripts: release-check.sh, release-package.sh and the second form of
# release-verify.sh, which reads a description and a directory and calls no `gh`. Each case
# runs in a directory under target/release-test that is removed at the end.
#
# Exit 0: every case passed. Exit 1: the first case that failed is named.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH= cd -- "$here/.." && pwd)
check=$here/release-check.sh
package=$here/release-package.sh
verify=$here/release-verify.sh

mkdir -p "$root/target"
work=$(mktemp -d "$root/target/release-test.XXXXXX")
trap 'rm -rf "$work"' EXIT
n=0

fail() {
  echo "release.test: $1" >&2
  exit 1
}

# expect <status> <name> <command...>: the command exits with <status>.
expect() {
  want=$1 name=$2
  shift 2
  status=0
  "$@" > "$work/out" 2>&1 || status=$?
  [ "$status" = "$want" ] || { sed 's/^/  /' "$work/out" >&2; fail "$name: exit $status, not $want"; }
  n=$((n + 1))
}

manifest() {
  printf '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "%s"\n' "$1" > "$work/Cargo.toml"
}

changelog() {
  printf '# Changelog\n\n## [Unreleased]\n\n## [%s] - %s\n\n### Added\n\n%s\n' "$1" "$2" "$3" > "$work/CHANGELOG.md"
}

# release-check.sh
manifest 0.1.0
changelog 0.1.0 2026-11-26 '- a change'
expect 0 "check: tag, version and section agree" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" 0.1.0
expect 1 "check: a tag that is no version" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" v0.1.0
expect 1 "check: a tag other than the workspace version" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" 0.1.1
changelog 0.1.0 2026-02-30 '- a change'
expect 1 "check: a date that is not of the calendar" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" 0.1.0
changelog 0.1.0 2026-11-26 ''
expect 1 "check: a section without a change" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" 0.1.0
expect 2 "check: no tag" sh "$check" --manifest "$work/Cargo.toml"
manifest 0.0.1
changelog 0.1.0 2026-11-26 '- a change'
expect 1 "check: the manifest is behind the tag" sh "$check" --manifest "$work/Cargo.toml" --changelog "$work/CHANGELOG.md" 0.1.0

# release-package.sh
expect 0 "package: targets" sh "$package" targets
[ "$(sh "$package" targets | wc -l)" = 3 ] || fail "package: three targets"
sh "$package" targets | grep -qx x86_64-unknown-linux-musl || fail "package: the static musl target"
[ "$(sh "$package" assets 0.1.0 | tail -n 1)" = SHA256SUMS ] || fail "package: SHA256SUMS is the last asset"
mkdir "$work/bin"
printf '#!/bin/sh\necho "agenticspec 0.1.0"\n' > "$work/bin/agenticspec"
chmod 755 "$work/bin/agenticspec"
printf '#!/bin/sh\necho "agenticspec 0.0.1"\n' > "$work/bin/old"
chmod 755 "$work/bin/old"
expect 1 "package: a binary of another version" sh "$package" archive 0.1.0 x86_64-unknown-linux-gnu "$work/bin/old" "$work/dist"
expect 1 "package: a target that is none of a release" sh "$package" archive 0.1.0 riscv64gc-unknown-linux-gnu "$work/bin/agenticspec" "$work/dist"
expect 1 "package: sums without the archives" sh "$package" sums 0.1.0 "$work/dist"
for target in $(sh "$package" targets); do
  expect 0 "package: the archive of $target" sh "$package" archive 0.1.0 "$target" "$work/bin/agenticspec" "$work/dist"
done
expect 0 "package: sums" sh "$package" sums 0.1.0 "$work/dist"
expect 0 "package: notes" sh "$package" notes 0.1.0 "$work/CHANGELOG.md"
[ "$(sh "$package" notes 0.1.0 "$work/CHANGELOG.md")" = "$(printf '### Added\n\n- a change')" ] \
  || fail "package: the notes are the section without its heading"
expect 1 "package: notes of a version without a section" sh "$package" notes 0.2.0 "$work/CHANGELOG.md"

# release-verify.sh, second form
describe() {
  jq -n --arg tag "$1" --arg login "$2" --argjson draft "$3" --arg extra "$4" '
    {tagName: $tag, isDraft: $draft, isPrerelease: false, author: {login: $login},
     assets: ([
       "agenticspec-0.1.0-x86_64-unknown-linux-gnu.tar.gz",
       "agenticspec-0.1.0-aarch64-unknown-linux-gnu.tar.gz",
       "agenticspec-0.1.0-x86_64-unknown-linux-musl.tar.gz",
       "SHA256SUMS"] + (if $extra == "" then [] else [$extra] end)
       | map({name: ., state: "uploaded"}))}' > "$work/description.json"
}
describe 0.1.0 'b10x-bot[bot]' false ''
expect 0 "verify: a release as published" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
describe 0.1.0 'someone' false ''
expect 1 "verify: an author other than the bot" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
describe 0.1.0 'b10x-bot[bot]' true ''
expect 1 "verify: a draft" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
describe 0.1.0 'b10x-bot[bot]' false 'notes.md'
expect 1 "verify: an asset that is none of the release" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
describe 0.1.0 'b10x-bot[bot]' false ''
cp "$work/dist/agenticspec-0.1.0-x86_64-unknown-linux-musl.tar.gz" "$work/musl.tar.gz"
mkdir "$work/musl"
tar -xzf "$work/musl.tar.gz" -C "$work/musl"
printf '#!/bin/sh\necho "agenticspec 0.0.1"\n' > "$work/musl/agenticspec-0.1.0-x86_64-unknown-linux-musl/agenticspec"
tar -C "$work/musl" --owner=0 --group=0 --numeric-owner -czf "$work/dist/agenticspec-0.1.0-x86_64-unknown-linux-musl.tar.gz" agenticspec-0.1.0-x86_64-unknown-linux-musl
(cd "$work/dist" && sha256sum agenticspec-0.1.0-*.tar.gz > SHA256SUMS)
expect 1 "verify: a musl binary of another version" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
cp "$work/musl.tar.gz" "$work/dist/agenticspec-0.1.0-x86_64-unknown-linux-musl.tar.gz"
(cd "$work/dist" && sha256sum agenticspec-0.1.0-*.tar.gz > SHA256SUMS)
expect 0 "verify: the musl archive restored" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
printf 'changed\n' >> "$work/dist/agenticspec-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
expect 1 "verify: an archive its checksum does not match" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"

# release.yml builds the musl target statically and runs it in a plain Alpine container.
grep -q 'target: x86_64-unknown-linux-musl' "$root/.github/workflows/release.yml" \
  || fail "workflows: release.yml builds no x86_64-unknown-linux-musl archive"
grep -q 'docker run --rm .*alpine@sha256:' "$root/.github/workflows/release.yml" \
  || fail "workflows: release.yml does not run the musl binary in a pinned Alpine container"
n=$((n + 1))

# The job Gate of release.yml installs the tools of the job gate of pages.yml, at the same
# versions and digests.
pins() {
  grep -E '^ +[A-Z_]+_(VERSION|SHA256): ' "$1" | sed 's/^ *//' | LC_ALL=C sort -u
}
[ "$(pins "$root/.github/workflows/release.yml")" = "$(pins "$root/.github/workflows/pages.yml")" ] \
  || fail "workflows: release.yml and pages.yml pin different tools"
n=$((n + 1))

# Both gates set up the same Node and the same browser for the app gate of `task check`.
app() {
  grep -E 'actions/setup-node@|node-version: |npm ci |npx playwright install ' "$1" \
    | sed -e 's/^ *//' -e 's/^- //' | LC_ALL=C sort -u
}
release_app=$(app "$root/.github/workflows/release.yml")
[ "$(printf '%s\n' "$release_app" | wc -l)" -eq 4 ] \
  || fail "workflows: release.yml's Gate does not set up Node and the browser"
[ "$release_app" = "$(app "$root/.github/workflows/pages.yml")" ] \
  || fail "workflows: release.yml and pages.yml set up Node or the browser differently"
n=$((n + 1))
sed 's/node-version: "22"/node-version: "20"/' "$root/.github/workflows/release.yml" > "$work/release.yml"
[ "$(app "$work/release.yml")" != "$(app "$root/.github/workflows/pages.yml")" ] \
  || fail "workflows: another Node version in release.yml is not seen"
n=$((n + 1))

echo "release.test: $n cases passed"
