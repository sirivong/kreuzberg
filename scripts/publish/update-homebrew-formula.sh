#!/usr/bin/env bash
set -euo pipefail

#   TAG=v5.0.0-rc.2 VERSION=5.0.0-rc.2 \

tag="${TAG:?TAG is required (e.g. v5.0.0-rc.2)}"
version="${VERSION:?VERSION is required (e.g. 5.0.0-rc.2)}"
tap_dir="${TAP_DIR:?TAP_DIR is required (path to homebrew-tap checkout)}"
dry_run="${DRY_RUN:-false}"

formula="${tap_dir}/Formula/xberg.rb"

[[ -f "$formula" ]] || {
  echo "Missing $formula" >&2
  exit 1
}

github_archive="https://github.com/xberg-io/xberg/archive/${tag}.tar.gz"

# GitHub's auto-generated /archive/<tag>.tar.gz is NOT byte-stable over time: the
# gzip stream can differ between requests for the same ref, so its sha256 changes.
# Hashing it here (formula-update) and re-downloading it later (bottle build,
# `brew install`) yields mismatched checksums and the bottle job fails with
# "Formula reports different checksum". Pin the formula to a release asset whose
# exact bytes we own instead: download the archive once, publish it as a stable
# release asset, and point the formula url/sha at that asset.
asset_name="xberg-${version}.tar.gz"
tarball_url="https://github.com/xberg-io/xberg/releases/download/${tag}/${asset_name}"

echo "Updating Homebrew formula for xberg ${version} (tag ${tag})"

if [[ "$dry_run" == "true" ]]; then
  echo "[dry-run] target formula: $formula"
  echo "[dry-run] would download $github_archive once"
  echo "[dry-run] would upload it as release asset $asset_name on $tag"
  echo "[dry-run] would set url to: $tarball_url and pin its sha256"
  echo "[dry-run] would leave bottle DSL untouched (handled by homebrew-merge-bottles)"
  exit 0
fi

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

echo "Fetching source tarball from ${github_archive}..."
curl -fsSL "$github_archive" -o "${workdir}/${asset_name}"
sha256=$(shasum -a 256 "${workdir}/${asset_name}" | awk '{print $1}')

echo "Publishing stable source tarball as release asset ${asset_name}..."
gh release upload "$tag" "${workdir}/${asset_name}" --repo xberg-io/xberg --clobber

echo "  url:    $tarball_url"
echo "  sha256: $sha256"

python3 - "$formula" "$tarball_url" "$sha256" <<'PY'
import re
import sys

formula_path, new_url, new_sha = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(formula_path).read()

# Split off the bottle block so the regex only touches the formula header.
bottle_start = text.find("bottle do")
if bottle_start == -1:
    head, tail = text, ""
else:
    head, tail = text[:bottle_start], text[bottle_start:]

head = re.sub(r'^(\s*url\s+)"[^"]*"', rf'\1"{new_url}"', head, count=1, flags=re.MULTILINE)
head = re.sub(r'^(\s*sha256\s+)"[^"]*"', rf'\1"{new_sha}"', head, count=1, flags=re.MULTILINE)

required_deps = ['"libheif"']
for dep in required_deps:
    if f"depends_on {dep}" not in head:
        head = re.sub(
            r'(^(\s*)depends_on\s+"rust"\s+=>\s+:build)([ \t]*$)',
            rf'\1\3\n\2depends_on {dep}',
            head,
            count=1,
            flags=re.MULTILINE,
        )

with open(formula_path, "w") as f:
    f.write(head + tail)
PY

echo "Updated $formula"
