#!/bin/sh
# A published release is what the procedure of AGENTS.md § "Release" makes.
#
#   release-verify.sh <version> [--repository <owner>/<name>]
#   release-verify.sh <version> --description <file> --assets <directory>
#
# The first form reads the release with `gh release view <version> --repo <repository> --json
# assets,author,isDraft,isPrerelease,tagName` and downloads its assets by name with `gh release
# download`, both read-only. The repository is the one `repository` in Cargo.toml names. The
# second form reads the same description from a file and the assets from a directory, and
# calls no `gh`.
#
# The checks are the same for both, in this order; the first that fails ends the run:
#
# 1. the description is of the release <version>, and the release is no draft and no
#    prerelease;
# 2. its author is the bot of the organization;
# 3. the assets of the description are the names of `release-package.sh assets <version>`,
#    each uploaded, and no other; each is a file of the directory;
# 4. SHA256SUMS names each archive once, and `sha256sum -c SHA256SUMS` passes there;
# 5. each archive holds one directory, named as the archive without .tar.gz or .zip, with
#    `agenticspec` (`agenticspec.exe` in the .zip of Windows), `LICENSE` and `README.md`,
#    nothing else and no path outside it, and each of the three is a regular file: no link, no
#    directory. A .zip may list the directory itself or not;
# 6. `agenticspec --version` of each x86_64 Linux archive (gnu and the static musl one) prints
#    `agenticspec <version>`, that line and no other, and exits 0. No file of an archive is run
#    before check 5 passed. The binaries of the other targets print their version on their own
#    runners in .github/workflows/release.yml before they are packed.
#
# Exit 0: every check passed, one line each. Exit 1: one line on standard error names the
# check that failed. Exit 2: the call is wrong.
# Needs `jq`, `tar`, `unzip` and `sha256sum`; the first form needs `gh`. Files are written
# under a directory made with `mktemp -d` inside ${TMPDIR:-target} and removed.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
package=$root/scripts/release-package.sh
bot='b10x-bot[bot]'
run_targets='x86_64-unknown-linux-gnu x86_64-unknown-linux-musl'

usage() {
  echo "usage: release-verify.sh <version> [--repository <owner>/<name>]" >&2
  echo "       release-verify.sh <version> --description <file> --assets <directory>" >&2
  exit 2
}

refuse() {
  echo "release-verify: $1" >&2
  exit 1
}

version= description= directory= repository=
while [ $# -gt 0 ]; do
  case $1 in
    --description) [ $# -ge 2 ] || usage; description=$2; shift 2 ;;
    --assets) [ $# -ge 2 ] || usage; directory=$2; shift 2 ;;
    --repository) [ $# -ge 2 ] || usage; repository=$2; shift 2 ;;
    -*) usage ;;
    *) [ -z "$version" ] || usage; version=$1; shift ;;
  esac
done
[ -n "$version" ] || usage
if [ -n "$description" ] || [ -n "$directory" ]; then
  [ -n "$description" ] && [ -n "$directory" ] && [ -z "$repository" ] || usage
fi

for tool in jq tar unzip sha256sum; do
  command -v "$tool" > /dev/null 2>&1 || refuse "$tool is not installed"
done
assets=$(sh "$package" assets "$version") || exit 1

base=${TMPDIR:-$root/target}
mkdir -p "$base"
work=$(mktemp -d "$base/release-verify.XXXXXX") || refuse "mktemp -d in $base failed"
trap 'rm -rf "$work"' EXIT
trap 'exit 1' HUP INT TERM

if [ -z "$description" ]; then
  command -v gh > /dev/null 2>&1 || refuse "gh is not installed"
  if [ -z "$repository" ]; then
    repository=$(sed -n 's|^repository = "https://github.com/\(.*\)"$|\1|p' "$root/Cargo.toml")
    [ -n "$repository" ] || refuse "Cargo.toml names no repository on github.com"
  fi
  description=$work/description.json
  if ! gh release view "$version" --repo "$repository" --json assets,author,isDraft,isPrerelease,tagName \
    > "$description" 2> "$work/gh"; then
    sed 's/^/release-verify: gh: /' "$work/gh" >&2
    refuse "gh release view $version found no release in $repository"
  fi
fi

[ -r "$description" ] || refuse "cannot read the description $description"
jq -e 'type == "object"' "$description" > /dev/null 2>&1 \
  || refuse "the description $description is no JSON object"

# 1. The release and its state.
tag=$(jq -r '.tagName // ""' "$description")
[ "$tag" = "$version" ] || refuse "the description is of the release ${tag:-without a tag}, not $version"
for state in isDraft:draft isPrerelease:prerelease; do
  case $(jq -r --arg field "${state%%:*}" '.[$field] | tostring' "$description") in
    false) ;;
    true) refuse "the release $version is a ${state#*:}" ;;
    *) refuse "the description does not say whether the release $version is a ${state#*:}" ;;
  esac
done
echo "release-verify: the release $version is published"

# 2. The author.
login=$(jq -r '.author.login // ""' "$description")
[ "$login" = "$bot" ] || refuse "the author of the release is ${login:-not named}, not $bot"
echo "release-verify: its author is $bot"

# 3. The assets: attached to the release, these and no other, then files of the directory.
# The first form downloads them between the two.
count=0 archives=
for asset in $assets; do
  jq -e --arg name "$asset" '[.assets[]? | select(.name == $name and .state == "uploaded")] | length == 1' \
    "$description" > /dev/null 2>&1 || refuse "the asset $asset is not attached"
  count=$((count + 1))
  [ "$asset" = SHA256SUMS ] || archives="$archives $asset"
done
other=$(jq -r --arg names "$assets" '[.assets[]? | .name | tostring] - ($names | split("\n")) | .[0] // empty' "$description")
[ -z "$other" ] || refuse "the release carries an asset that is none of its $count: $other"
if [ -z "$directory" ]; then
  directory=$work/assets
  set --
  for asset in $assets; do set -- "$@" --pattern "$asset"; done
  if ! gh release download "$version" --repo "$repository" --dir "$directory" "$@" \
    > "$work/gh" 2>&1; then
    sed 's/^/release-verify: gh: /' "$work/gh" >&2
    refuse "gh release download $version failed in $repository"
  fi
fi
[ -d "$directory" ] || refuse "$directory is no directory"
for asset in $assets; do
  [ -f "$directory/$asset" ] || refuse "the asset $asset is not in $directory"
done
echo "release-verify: its $count assets are attached, and no other:" $assets

# 4. The checksums.
for archive in $archives; do
  awk -v name="$archive" '
    length($1) == 64 && $1 !~ /[^0-9a-f]/ && $2 == name { found++ }
    END { exit found == 1 ? 0 : 1 }' "$directory/SHA256SUMS" \
    || refuse "SHA256SUMS does not name $archive once"
done
if ! (cd "$directory" && sha256sum -c SHA256SUMS) > "$work/sums" 2>&1; then
  sed 's/^/release-verify: sha256sum: /' "$work/sums" >&2
  refuse "sha256sum -c SHA256SUMS fails in $directory"
fi
echo "release-verify: sha256sum -c SHA256SUMS passes"

# 5. What each archive holds, read from its listing: nothing is unpacked here. The names are
# the three files in the directory, and in a .tar.gz the directory, written with its `/`; a
# name twice, a name in another directory and a directory in the place of a file each change
# the list. The first character of a line of `tar -tv` and of `unzip -Z` is the kind of the
# member: `d` at most once, for the directory, and `-`, a regular file, for the three. A link
# (`l`, `h`) is none.
for archive in $archives; do
  case $archive in
    *.zip) name=${archive%.zip} file=agenticspec.exe ;;
    *) name=${archive%.tar.gz} file=agenticspec ;;
  esac
  printf '%s\n' "$name/LICENSE" "$name/README.md" "$name/$file" | LC_ALL=C sort > "$work/files"
  case $archive in
    *.zip)
      unzip -Z1 "$directory/$archive" 2> /dev/null | grep -vxF "$name/" | LC_ALL=C sort > "$work/members"
      cmp -s "$work/members" "$work/files" \
        || refuse "$archive does not hold $file, LICENSE and README.md in $name, and nothing else"
      unzip -Z "$directory/$archive" 2> /dev/null | awk -v dir="$name/" '
        NR <= 2 || /^[0-9]+ files?, / { next }
        $NF == dir && substr($1, 1, 1) == "d" { dirs++; next }
        $NF != dir && substr($1, 1, 1) == "-" { regular++; next }
        { other++ }
        END { exit (regular == 3 && dirs <= 1 && !other) ? 0 : 1 }' \
        || refuse "$archive holds a member that is no regular file"
      ;;
    *)
      { echo "$name/"; cat "$work/files"; } | LC_ALL=C sort > "$work/wanted"
      tar -tzf "$directory/$archive" 2> /dev/null | LC_ALL=C sort > "$work/members"
      cmp -s "$work/members" "$work/wanted" \
        || refuse "$archive does not hold $file, LICENSE and README.md in $name, and nothing else"
      tar -tvzf "$directory/$archive" 2> /dev/null | awk '
        { kinds[substr($0, 1, 1)]++ }
        END { exit (NR == 4 && kinds["d"] == 1 && kinds["-"] == 3) ? 0 : 1 }' \
        || refuse "$archive holds a member that is no regular file"
      ;;
  esac
done
echo "release-verify: each archive holds agenticspec, LICENSE and README.md"

# 6. The binaries of the archives this check can run.
mkdir "$work/unpacked"
for run_target in $run_targets; do
  name=agenticspec-$version-$run_target
  tar -xzf "$directory/$name.tar.gz" -C "$work/unpacked" 2> /dev/null || refuse "$name.tar.gz does not unpack"
  binary=$work/unpacked/$name/agenticspec
  [ -f "$binary" ] && [ ! -L "$binary" ] && [ -x "$binary" ] \
    || refuse "the agenticspec of the $run_target archive is no regular file that can be run"
  status=0
  printed=$("$binary" --version 2> /dev/null) || status=$?
  [ "$status" = 0 ] || refuse "\`agenticspec --version\` of the $run_target archive exits $status"
  [ "$printed" = "agenticspec $version" ] \
    || refuse "the agenticspec of the $run_target archive prints \"$(printf '%s\n' "$printed" | sed -n 1p)\", not \"agenticspec $version\""
  echo "release-verify: the agenticspec of the $run_target archive prints \"agenticspec $version\" and exits 0"
done
