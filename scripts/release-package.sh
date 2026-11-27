#!/bin/sh
# What a release is made of: the archives, their checksums and the notes.
#
#   release-package.sh targets
#       the targets a release is built for, one per line
#   release-package.sh assets <version>
#       the names of the files a release carries: one archive per target, then SHA256SUMS
#   release-package.sh archive <version> <target> <binary> <directory>
#       write the archive of <target> into <directory>: agenticspec-<version>-<target>.zip for
#       a Windows target, agenticspec-<version>-<target>.tar.gz for every other. It holds one
#       directory, agenticspec-<version>-<target>, with the binary as `agenticspec`
#       (`agenticspec.exe` on Windows), LICENSE and README.md. In a .tar.gz each has owner 0,
#       the directory and the binary mode 755, the two files 644. Refused unless
#       `<binary> --version` prints `agenticspec <version>`, that line and no other, and exits 0
#   release-package.sh sums <version> <directory>
#       write <directory>/SHA256SUMS over the archive of every target and verify it.
#       Refused when an archive is not there
#   release-package.sh notes <version> [<CHANGELOG.md>]
#       print the section `## [<version>] - <YYYY-MM-DD>` of the changelog without its
#       heading and without the empty lines around it. Refused when it holds no change: no
#       `- ` line under a `### ` heading
#
# LICENSE, README.md and the default changelog are those of the repository this script is in.
# .github/workflows/release.yml calls `archive` once per target on the runner of the target,
# then `sums` and `notes`.
#
# `archive` runs on Linux, macOS and Windows (Git Bash). A .tar.gz is written by `tar`, GNU tar
# or bsdtar, told apart by `tar --version`; RELEASE_TAR names another tar. A .zip is written by
# `zip` when it is installed, else by `7z`; RELEASE_ZIP names one of the two.
#
# Exit 0: done. Exit 1: refused, with one line on standard error. Exit 2: the call is wrong.
# `sums` needs `sha256sum`.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# x86_64-unknown-linux-musl and aarch64-unknown-linux-musl are static binaries: they need no
# glibc of the host, and the x86_64 one is the archive consumers pin.
targets='x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-musl aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc'

usage() {
  echo "usage: release-package.sh targets" >&2
  echo "       release-package.sh assets <version>" >&2
  echo "       release-package.sh archive <version> <target> <binary> <directory>" >&2
  echo "       release-package.sh sums <version> <directory>" >&2
  echo "       release-package.sh notes <version> [<CHANGELOG.md>]" >&2
  exit 2
}

refuse() {
  echo "release-package: $1" >&2
  exit 1
}

# version_of <text>: refuse a text that is no bare version.
version_of() {
  printf '%s\n' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
    || refuse "$1 is no version of the form <major>.<minor>.<patch>"
}

# archive_of <version> <target>: the file name of the archive of <target>.
archive_of() {
  case $2 in
    *-windows-*) echo "agenticspec-$1-$2.zip" ;;
    *) echo "agenticspec-$1-$2.tar.gz" ;;
  esac
}

command=${1-}
[ $# -gt 0 ] && shift

case $command in
  targets)
    [ $# = 0 ] || usage
    for target in $targets; do echo "$target"; done
    ;;

  assets)
    [ $# = 1 ] || usage
    version_of "$1"
    for target in $targets; do archive_of "$1" "$target"; done
    echo SHA256SUMS
    ;;

  archive)
    [ $# = 4 ] || usage
    version=$1 target=$2 binary=$3 directory=$4
    version_of "$version"
    case " $targets " in
      *" $target "*) ;;
      *) refuse "$target is no target of a release: $targets" ;;
    esac
    [ -f "$binary" ] && [ -x "$binary" ] || refuse "$binary is no executable file"
    case $binary in
      */*) ;;
      *) binary=./$binary ;;
    esac
    status=0
    printed=$("$binary" --version 2> /dev/null) || status=$?
    [ "$status" = 0 ] || refuse "\`agenticspec --version\` exits $status"
    [ "$printed" = "agenticspec $version" ] \
      || refuse "the binary prints \"$(printf '%s\n' "$printed" | sed -n 1p)\", not \"agenticspec $version\""
    case $target in
      *-windows-*) file=agenticspec.exe ;;
      *) file=agenticspec ;;
    esac
    name=agenticspec-$version-$target
    archive=$(archive_of "$version" "$target")
    mkdir -p "$directory"
    rm -rf "${directory:?}/$name" "${directory:?}/$archive"
    mkdir "$directory/$name"
    cp "$binary" "$directory/$name/$file"
    cp "$root/LICENSE" "$root/README.md" "$directory/$name/"
    chmod 755 "$directory/$name" "$directory/$name/$file"
    chmod 644 "$directory/$name/LICENSE" "$directory/$name/README.md"
    case $archive in
      *.zip)
        zip=${RELEASE_ZIP:-}
        if [ -z "$zip" ]; then
          if command -v zip > /dev/null 2>&1; then zip=zip; else zip=7z; fi
        fi
        case $zip in
          zip) (cd "$directory" && zip -q -X -r "$archive" "$name") ;;
          7z) (cd "$directory" && 7z a -tzip -bd -bso0 -bsp0 "$archive" "$name") ;;
          *) refuse "RELEASE_ZIP is $zip, neither zip nor 7z" ;;
        esac || refuse "$zip could not write $archive"
        ;;
      *)
        tar=${RELEASE_TAR:-tar}
        if "$tar" --version 2> /dev/null | grep -q 'GNU tar'; then
          "$tar" -C "$directory" --owner=0 --group=0 --numeric-owner -czf "$directory/$archive" "$name"
        else
          # bsdtar, the tar of macOS: no AppleDouble `._` members, no extended attributes.
          COPYFILE_DISABLE=1 "$tar" -C "$directory" --uid 0 --gid 0 --numeric-owner \
            --no-mac-metadata --no-xattrs --no-acls --no-fflags -czf "$directory/$archive" "$name"
        fi || refuse "$tar could not write $archive"
        ;;
    esac
    rm -rf "${directory:?}/$name"
    echo "release-package: $archive holds $file ($version), LICENSE and README.md"
    ;;

  sums)
    [ $# = 2 ] || usage
    version=$1 directory=$2
    version_of "$version"
    set --
    for target in $targets; do
      archive=$(archive_of "$version" "$target")
      [ -f "$directory/$archive" ] || refuse "no $archive in $directory"
      set -- "$@" "$archive"
    done
    cd "$directory"
    sha256sum "$@" > SHA256SUMS
    sha256sum -c SHA256SUMS
    ;;

  notes)
    [ $# = 1 ] || [ $# = 2 ] || usage
    version=$1 changelog=${2:-$root/CHANGELOG.md}
    version_of "$version"
    [ -r "$changelog" ] || refuse "cannot read $changelog"
    status=0
    notes=$(awk -v start="## [$version] - " '
      inside && /^## / { exit }
      inside && /^### / { headed = 1 }
      inside && /^- / && headed { changes++ }
      inside { lines[++count] = $0; next }
      index($0, start) == 1 && substr($0, length(start) + 1) ~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]$/ { inside = 1 }
      END {
        first = 1
        while (first <= count && lines[first] ~ /^[ \t]*$/) first++
        last = count
        while (last >= first && lines[last] ~ /^[ \t]*$/) last--
        for (i = first; i <= last; i++) print lines[i]
        exit changes ? 0 : 3
      }' "$changelog") || status=$?
    [ -n "$notes" ] || refuse "${changelog##*/} has no notes for $version"
    [ "$status" = 0 ] \
      || refuse "the section of $version in ${changelog##*/} holds no change line under a heading"
    printf '%s\n' "$notes"
    ;;

  *) usage ;;
esac
