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

tarball_url="https://github.com/xberg-io/xberg/archive/${tag}.tar.gz"

echo "Updating Homebrew formula for xberg ${version} (tag ${tag})"

if [[ "$dry_run" == "true" ]]; then
  echo "[dry-run] target formula: $formula"
  echo "[dry-run] would set url to: $tarball_url"
  echo "[dry-run] would compute sha256 of source tarball and rewrite the formula"
  echo "[dry-run] would leave bottle DSL untouched (handled by homebrew-merge-bottles)"
  exit 0
fi

echo "Fetching source tarball SHA256 for ${tag}..."
sha256=$(curl -fsSL "$tarball_url" | shasum -a 256 | awk '{print $1}')
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
