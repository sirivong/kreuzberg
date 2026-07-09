#!/usr/bin/env python3
"""Sanitize pandoc-generated markdown ground truth files.

Removes common pandoc artifacts that don't represent actual document structure.

Usage:
    # Single file (in-place):
    python sanitize_pandoc_gt.py input.md

    # Pipe mode:
    pandoc -f docbook -t gfm --wrap=none input.xml | python sanitize_pandoc_gt.py > output.md

    # Dry run (show diff without modifying):
    python sanitize_pandoc_gt.py --dry-run input.md

    # Batch all GT files (dry run):
    python sanitize_pandoc_gt.py --dry-run --batch test_documents/ground_truth/

    # Batch all GT files (apply):
    python sanitize_pandoc_gt.py --batch test_documents/ground_truth/
"""

import argparse
import difflib
import os
import re
import sys


def sanitize(text: str) -> str:
    in_code = False
    lines = text.split("\n")
    result = []

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_code = not in_code
            if not in_code or stripped.startswith("```") or stripped.startswith("~~~"):
                m = re.match(r"^(`{3,}|~{3,})\s*\{\s*\.(\w+)(?:\s+[^}]*)?\}\s*$", line)
                if m:
                    line = f"{m.group(1)}{m.group(2)}"
                else:
                    line = re.sub(r"^(`{3,}|~{3,})\s*\{[^}]*\}\s*$", r"\1", line)
            result.append(line)
            continue

        if in_code:
            result.append(line)
            continue

        if re.match(r"^:::\s*(\{.*\})?\s*$", stripped):
            continue

        if re.match(r"^#{1,6}\s", line):
            line = re.sub(r"\s*\{[.#][^}]*\}\s*$", "", line)

        if stripped == "<!-- end list -->":
            if not (result and result[-1].strip() == ""):
                result.append("")
            continue

        if stripped == "<!-- end list -->" or stripped == "<!-- -->":
            continue

        result.append(line)

    while result and result[-1].strip() == "":
        result.pop()

    return "\n".join(result) + "\n" if result else ""


def process_file(path: str, dry_run: bool = False) -> tuple[bool, str]:
    """Process a single file. Returns (changed, diff_text)."""
    with open(path) as f:
        original = f.read()

    cleaned = sanitize(original)

    if original == cleaned:
        return False, ""

    diff = "".join(
        difflib.unified_diff(
            original.splitlines(keepends=True),
            cleaned.splitlines(keepends=True),
            fromfile=f"a/{path}",
            tofile=f"b/{path}",
            n=3,
        )
    )

    if not dry_run:
        with open(path, "w") as f:
            f.write(cleaned)

    return True, diff


def main():
    parser = argparse.ArgumentParser(description="Sanitize pandoc GT markdown files")
    parser.add_argument("path", nargs="?", help="File or directory to process")
    parser.add_argument("--dry-run", action="store_true", help="Show diff without modifying files")
    parser.add_argument("--batch", action="store_true", help="Process all .md files in directory recursively")
    args = parser.parse_args()

    if args.path is None and not sys.stdin.isatty():
        sys.stdout.write(sanitize(sys.stdin.read()))
        return

    if args.path is None:
        parser.print_help()
        return

    if args.batch or os.path.isdir(args.path):
        changed_count = 0
        total_count = 0
        for root, _dirs, files in os.walk(args.path):
            for fname in sorted(files):
                if not fname.endswith(".md"):
                    continue
                fpath = os.path.join(root, fname)
                total_count += 1
                changed, diff = process_file(fpath, dry_run=args.dry_run)
                if changed:
                    changed_count += 1
                    if args.dry_run:
                        print(diff)
                    else:
                        print(f"  cleaned: {fpath}")

        action = "would change" if args.dry_run else "cleaned"
        print(f"\n{action} {changed_count}/{total_count} files")
        return

    changed, diff = process_file(args.path, dry_run=args.dry_run)
    if changed:
        if args.dry_run:
            print(diff)
        else:
            print(f"cleaned: {args.path}")
    else:
        print(f"no changes: {args.path}")


if __name__ == "__main__":
    main()
