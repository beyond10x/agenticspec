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
[ "$(sh "$package" targets | wc -l)" = 7 ] || fail "package: seven targets"
for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl \
  aarch64-unknown-linux-musl aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc; do
  sh "$package" targets | grep -qx "$target" || fail "package: the target $target"
done
[ "$(sh "$package" assets 0.1.0 | wc -l)" = 8 ] || fail "package: an archive per target and SHA256SUMS"
[ "$(sh "$package" assets 0.1.0 | tail -n 1)" = SHA256SUMS ] || fail "package: SHA256SUMS is the last asset"
[ "$(sh "$package" assets 0.1.0 | grep -c '\.zip$')" = 1 ] || fail "package: one .zip"
sh "$package" assets 0.1.0 | grep -qx agenticspec-0.1.0-x86_64-pc-windows-msvc.zip \
  || fail "package: the Windows archive is a .zip"
[ "$(sh "$package" assets 0.1.0 | grep -c '\.tar\.gz$')" = 6 ] || fail "package: six .tar.gz"
n=$((n + 1))
mkdir "$work/bin"
printf '#!/bin/sh\necho "agenticspec 0.1.0"\n' > "$work/bin/agenticspec"
chmod 755 "$work/bin/agenticspec"
printf '#!/bin/sh\necho "agenticspec 0.0.1"\n' > "$work/bin/old"
chmod 755 "$work/bin/old"
expect 1 "package: a binary of another version" sh "$package" archive 0.1.0 x86_64-unknown-linux-gnu "$work/bin/old" "$work/dist"
expect 1 "package: a binary of another version for Windows" sh "$package" archive 0.1.0 x86_64-pc-windows-msvc "$work/bin/old" "$work/dist"
expect 1 "package: a target that is none of a release" sh "$package" archive 0.1.0 riscv64gc-unknown-linux-gnu "$work/bin/agenticspec" "$work/dist"
expect 1 "package: sums without the archives" sh "$package" sums 0.1.0 "$work/dist"
for target in $(sh "$package" targets); do
  expect 0 "package: the archive of $target" sh "$package" archive 0.1.0 "$target" "$work/bin/agenticspec" "$work/dist"
done
[ "$(unzip -Z1 "$work/dist/agenticspec-0.1.0-x86_64-pc-windows-msvc.zip" | LC_ALL=C sort | tr '\n' ' ')" \
  = "agenticspec-0.1.0-x86_64-pc-windows-msvc/ agenticspec-0.1.0-x86_64-pc-windows-msvc/LICENSE agenticspec-0.1.0-x86_64-pc-windows-msvc/README.md agenticspec-0.1.0-x86_64-pc-windows-msvc/agenticspec.exe " ] \
  || fail "package: the .zip holds agenticspec.exe, LICENSE and README.md in its directory"
[ "$(tar -tzf "$work/dist/agenticspec-0.1.0-aarch64-apple-darwin.tar.gz" | LC_ALL=C sort | tr '\n' ' ')" \
  = "agenticspec-0.1.0-aarch64-apple-darwin/ agenticspec-0.1.0-aarch64-apple-darwin/LICENSE agenticspec-0.1.0-aarch64-apple-darwin/README.md agenticspec-0.1.0-aarch64-apple-darwin/agenticspec " ] \
  || fail "package: the .tar.gz holds agenticspec, LICENSE and README.md in its directory"
n=$((n + 1))
expect 0 "package: sums" sh "$package" sums 0.1.0 "$work/dist"
grep -q '  agenticspec-0.1.0-x86_64-pc-windows-msvc.zip$' "$work/dist/SHA256SUMS" || fail "package: SHA256SUMS names the .zip"
n=$((n + 1))
expect 0 "package: notes" sh "$package" notes 0.1.0 "$work/CHANGELOG.md"
[ "$(sh "$package" notes 0.1.0 "$work/CHANGELOG.md")" = "$(printf '### Added\n\n- a change')" ] \
  || fail "package: the notes are the section without its heading"
expect 1 "package: notes of a version without a section" sh "$package" notes 0.2.0 "$work/CHANGELOG.md"
expect 1 "package: a zip tool that is neither zip nor 7z" env RELEASE_ZIP=rar sh "$package" archive 0.1.0 x86_64-pc-windows-msvc "$work/bin/agenticspec" "$work/other"

# The other archivers of the runners: 7z on Windows, bsdtar on macOS. Their archives pass
# release-verify.sh like those of zip and GNU tar. A host without one skips its case.
mkdir "$work/alt"
cp "$work/dist"/* "$work/alt/"
alt=0
if command -v 7z > /dev/null 2>&1; then
  expect 0 "package: the .zip written by 7z" env RELEASE_ZIP=7z sh "$package" archive 0.1.0 x86_64-pc-windows-msvc "$work/bin/agenticspec" "$work/alt"
  alt=1
else
  echo "release.test: no 7z on this host, its case skipped"
fi
if command -v bsdtar > /dev/null 2>&1; then
  expect 0 "package: the .tar.gz written by bsdtar" env RELEASE_TAR=bsdtar sh "$package" archive 0.1.0 aarch64-apple-darwin "$work/bin/agenticspec" "$work/alt"
  alt=1
else
  echo "release.test: no bsdtar on this host, its case skipped"
fi

# release-verify.sh, second form
describe() {
  sh "$package" assets 0.1.0 | jq -R . | jq -s --arg tag "$1" --arg login "$2" --argjson draft "$3" --arg extra "$4" '
    {tagName: $tag, isDraft: $draft, isPrerelease: false, author: {login: $login},
     assets: (. + (if $extra == "" then [] else [$extra] end) | map({name: ., state: "uploaded"}))}' \
    > "$work/description.json"
}
describe 0.1.0 'b10x-bot[bot]' false ''
expect 0 "verify: a release as published" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
if [ "$alt" = 1 ]; then
  (cd "$work/alt" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
  expect 0 "verify: the archives of 7z and bsdtar" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/alt"
fi
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
(cd "$work/dist" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
expect 1 "verify: a musl binary of another version" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
cp "$work/musl.tar.gz" "$work/dist/agenticspec-0.1.0-x86_64-unknown-linux-musl.tar.gz"
(cd "$work/dist" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
expect 0 "verify: the musl archive restored" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
zip_name=agenticspec-0.1.0-x86_64-pc-windows-msvc
cp "$work/dist/$zip_name.zip" "$work/windows.zip"
mkdir -p "$work/extra/$zip_name"
printf 'extra\n' > "$work/extra/$zip_name/extra.txt"
(cd "$work/extra" && zip -q "$work/dist/$zip_name.zip" "$zip_name/extra.txt")
(cd "$work/dist" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
expect 1 "verify: a .zip with a file more" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
cp "$work/windows.zip" "$work/dist/$zip_name.zip"
mkdir -p "$work/bare/$zip_name"
printf 'x\n' > "$work/bare/$zip_name/agenticspec"
cp "$root/LICENSE" "$root/README.md" "$work/bare/$zip_name/"
rm -f "$work/bare.zip"
(cd "$work/bare" && zip -q -r "$work/bare.zip" "$zip_name")
cp "$work/bare.zip" "$work/dist/$zip_name.zip"
(cd "$work/dist" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
expect 1 "verify: a .zip whose binary has no .exe" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
cp "$work/windows.zip" "$work/dist/$zip_name.zip"
(cd "$work/dist" && sha256sum agenticspec-0.1.0-* > SHA256SUMS)
expect 0 "verify: the .zip restored" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"
printf 'changed\n' >> "$work/dist/agenticspec-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
expect 1 "verify: an archive its checksum does not match" sh "$verify" 0.1.0 --description "$work/description.json" --assets "$work/dist"

# release.yml: one matrix entry per target of release-package.sh, each on its native runner;
# the musl targets static and run in a plain Alpine container.
release_yml=$root/.github/workflows/release.yml
matrix() {
  awk '/^ +- target: / { target = $3; next } target != "" && /^ +runner: / { print target, $2; target = "" }' "$1"
}
[ "$(matrix "$release_yml" | cut -d ' ' -f 1 | LC_ALL=C sort)" = "$(sh "$package" targets | LC_ALL=C sort)" ] \
  || fail "workflows: the matrix of release.yml is not the targets of release-package.sh"
for pair in x86_64-unknown-linux-gnu:ubuntu-22.04 aarch64-unknown-linux-gnu:ubuntu-22.04-arm \
  x86_64-unknown-linux-musl:ubuntu-22.04 aarch64-unknown-linux-musl:ubuntu-24.04-arm \
  aarch64-apple-darwin:macos-latest x86_64-apple-darwin:macos-latest \
  x86_64-pc-windows-msvc:windows-latest; do
  matrix "$release_yml" | grep -qx "${pair%%:*} ${pair#*:}" \
    || fail "workflows: release.yml does not build ${pair%%:*} on ${pair#*:}"
done
n=$((n + 1))
grep -q "if: contains(matrix.target, '-linux-musl')" "$release_yml" \
  || fail "workflows: release.yml does not check the musl targets"
grep -q 'docker run --rm .*alpine@sha256:' "$release_yml" \
  || fail "workflows: release.yml does not run the musl binary in a pinned Alpine container"
n=$((n + 1))

# release.yml runs the package matrix on a pull request from release/<version> too, with
# contents: read and no token: it publishes nothing.
grep -q '^  pull_request:$' "$release_yml" || fail "workflows: release.yml does not run on a pull request"
grep -qF "startsWith(github.head_ref, 'release/')" "$release_yml" \
  || fail "workflows: release.yml does not limit its pull request run to release/ branches"
if grep -Eq 'contents: write|secrets\.|GITHUB_TOKEN|github\.token' "$release_yml"; then
  fail "workflows: release.yml writes or reads a token"
fi
[ "$(grep -c 'contents: read' "$release_yml")" = 5 ] || fail "workflows: release.yml and each job read contents only"
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
