#!/usr/bin/env bash
# Run inside a clean Linux host with a locally supplied CLI release archive.
set -euo pipefail

archive="$(realpath "${1:?usage: smoke-test-cli-installer.sh ARCHIVE VERSION [musl|gnu|auto] [GNU_ARCHIVE]}")"
version="${2:?expected release version}"
libc="${3:-musl}"
version="${version#v}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir "$work/bin" "$work/archive"
export SMOKE_GNU_ARCHIVE="${4:-}" SMOKE_ARCHIVE="$archive" XBERG_VERSION="$version" XBERG_INSTALL="$work/installed"
unset XBERG_LINUX_LIBC
if [ "$libc" = gnu ]; then
  export XBERG_LINUX_LIBC="$libc"
fi

# Keep target selection observable without requiring network or package installation. ~keep
cat >"$work/bin/curl" <<'SH'
#!/bin/sh
set -eu
[ "$#" = 6 ] && [ "$1" = -fsSL ] && [ "$2" = -w ] && [ "$3" = '%{http_code}' ] && [ "$5" = -o ]
base="https://github.com/xberg-io/xberg/releases/download/v${XBERG_VERSION}"
case "$4" in
  "$base/${SMOKE_ARCHIVE##*/}") cp "$SMOKE_ARCHIVE" "$6" ;;
  "$base/${SMOKE_GNU_ARCHIVE##*/}")
    [ -n "$SMOKE_GNU_ARCHIVE" ]
    cp "$SMOKE_GNU_ARCHIVE" "$6"
    ;;
  *)
    case "${SMOKE_ARCHIVE##*/}:$4" in
      *-linux-musl.tar.gz:"$base/"*-linux-gnu.tar.gz) printf '404'; exit 22 ;;
      *) exit 91 ;;
    esac
    ;;
esac
printf '200'
SH
chmod +x "$work/bin/curl"
PATH="$work/bin:$PATH" bash "$root/scripts/install.sh"

tar -xzf "$archive" -C "$work/archive"
(
  cd "$work/archive/$(basename "$archive" .tar.gz)"
  for file in xberg xberg.bin lib *.so *.so.*; do
    [ -e "$file" ] || continue
    diff -r "$file" "$XBERG_INSTALL/$file"
  done
)
rm -rf "$work/archive"

cd "$work"
cli="$XBERG_INSTALL/xberg"
[ "$("$cli" --version)" = "xberg $version" ]
printf '%s\n' 'Xberg installed layout smoke test' >document.txt
"$cli" extract document.txt | grep -F 'Xberg installed layout smoke test'
"$cli" extract "$root/scripts/ci/fixtures/musl-python-smoke.pdf" | grep -F 'XBERG MUSL SMOKE 490'
"$cli" extract "$root/crates/xberg/tests/fixtures/office/merged_cells.docx" | grep -F 'Blower'
