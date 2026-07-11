#!/usr/bin/env bash
# Recursively vendor a macOS .node's non-system dylib closure beside it and
# rewrite every absolute (non-system) load command to @loader_path, then ad-hoc
# re-sign each rewritten binary.
#
# Why: the NAPI .node links libheif (HEIC/AVIF) via an absolute Homebrew path
# (/opt/homebrew/opt/libheif/lib/libheif.1.dylib), and libheif in turn pulls a
# transitive closure (libde265, libx265, libaom, libsharpyuv, libvmaf, ...).
# None of these are vendored by the shared build-node-napi action (it only
# vendors ONNX Runtime, which is already @rpath-relocatable), so the published
# darwin package fails to dlopen on any Mac without those Homebrew formulae at
# those exact paths. ONNX Runtime is left untouched here: it is referenced via
# @rpath and resolved by co-location + the .node's @loader_path LC_RPATH.
#
# Usage: vendor-macos-node-dylibs.sh <dir-containing-the-.node>
set -euo pipefail

DIR="${1:?usage: vendor-macos-node-dylibs.sh <dir>}"
DIR="$(cd "$DIR" && pwd)"

# Absolute path outside the system prefixes = vendorable. @rpath/@loader_path/
# @executable_path are already relocatable and left alone.
is_vendorable() {
  case "$1" in
    /usr/lib/*|/System/*) return 1 ;;
    @*)                   return 1 ;;
    /*)                   return 0 ;;
    *)                    return 1 ;;
  esac
}

# Load-command deps of a Mach-O, excluding the header line and the binary's own
# install id (LC_ID_DYLIB) — for a .node the id is often a CI build path that
# must never be vendored.
deps_of() {
  local self_id
  self_id="$(otool -D "$1" | tail -n +2 | head -1 | sed 's/^[[:space:]]*//')"
  # Drop the header (line 1) and exclude the binary's own id. `|| true` keeps a
  # binary with no vendorable deps from failing the pipeline under `set -e`.
  otool -L "$1" | tail -n +2 | sed 's/^[[:space:]]*//; s/ (compatibility.*//' \
    | grep -vxF -e "$self_id" || true
}

resolve() { readlink -f "$1" 2>/dev/null || python3 -c 'import os,sys;print(os.path.realpath(sys.argv[1]))' "$1"; }
resign()  { codesign --remove-signature "$1" 2>/dev/null || true; codesign -f -s - "$1"; }

declare -A seen
queue=()
for node in "$DIR"/*.node; do
  [ -e "$node" ] || continue
  base="$(basename "$node")"; queue+=("$base"); seen["$base"]=1
done
[ ${#queue[@]} -gt 0 ] || { echo "::error::no .node in $DIR to vendor for"; exit 1; }

i=0
while [ $i -lt ${#queue[@]} ]; do
  bin="${queue[$i]}"; i=$((i+1))
  target="$DIR/$bin"
  [ -f "$target" ] || { echo "::warning::$bin queued but not present"; continue; }
  changed=0
  while IFS= read -r dep; do
    [ -n "$dep" ] || continue
    is_vendorable "$dep" || continue
    b="$(basename "$dep")"
    if [ ! -f "$DIR/$b" ]; then
      cp -f "$(resolve "$dep")" "$DIR/$b"; chmod u+w "$DIR/$b"
      echo "vendored $b"
    fi
    install_name_tool -change "$dep" "@loader_path/$b" "$target"
    changed=1
    if [ -z "${seen[$b]:-}" ]; then seen["$b"]=1; queue+=("$b"); fi
  done < <(deps_of "$target")
  case "$bin" in
    *.dylib) install_name_tool -id "@loader_path/$bin" "$target" 2>/dev/null || true; changed=1 ;;
  esac
  [ $changed -eq 1 ] && resign "$target"
done

# Fail loudly if any absolute non-system dep survived anywhere in the closure.
leaks=0
for f in "$DIR"/*.node "$DIR"/*.dylib; do
  [ -e "$f" ] || continue
  while IFS= read -r dep; do
    is_vendorable "$dep" && { echo "::error::unvendored dep $(basename "$f") -> $dep"; leaks=$((leaks+1)); }
  done < <(deps_of "$f")
done
[ $leaks -eq 0 ] || { echo "::error::$leaks unvendored absolute deps remain"; exit 1; }
echo "macOS dylib closure vendored and self-contained under $DIR"
