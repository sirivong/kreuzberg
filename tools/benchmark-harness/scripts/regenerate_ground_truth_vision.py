#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""Regenerate ground truth: Phase 1 — prepare readable files for AI extraction.

Prepares all benchmark fixture documents into formats readable by AI subagents
(PDF for vision, HTML/text for text reading). Writes a manifest JSON listing
all items for Phase 2 processing.

Phase 1 (this script): Pre-render documents → readable format → manifest.json
Phase 2 (subagents):   Read files → extract text → write GT → update fixtures

Rendering strategy:
  - PDF:        Pass-through (AI reads directly with vision)
  - Images:     Pass-through (AI reads directly with vision)
  - HTML/text:  Pass-through (AI reads directly as text)
  - Binary supported by pandoc (docx/epub/odt/rtf/pptx/xlsx/ppsx/pptm/xlsm):
                pandoc → HTML
  - Binary not supported by pandoc (doc/xls/ods/xlsb/ppt):
                libreoffice → HTML
  - Email (eml/msg): Python stdlib → plain text

Usage:
    uv run tools/benchmark-harness/scripts/regenerate_ground_truth_vision.py [OPTIONS]
"""

from __future__ import annotations

import argparse
import email as email_mod
import json
import os
import subprocess
import sys
from pathlib import Path


# ---------------------------------------------------------------------------
# Format categories
# ---------------------------------------------------------------------------

DIRECT_READ_PDF = frozenset({"pdf"})

DIRECT_READ_IMAGE = frozenset({
    "png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "jp2",
    "j2c", "j2k", "jpm", "jpx", "mj2", "pbm", "pgm", "pnm", "ppm",
})

DIRECT_READ_TEXT = frozenset({
    "html", "htm",
    "md", "markdown", "mdx", "rst", "org", "commonmark", "djot",
    "txt", "csv", "tsv", "toml", "yaml", "yml", "json", "xml", "svg",
    "bib", "enw", "nbib", "ris", "vtt", "asciidoc",
    "tex", "latex", "typ", "typst",
    "ipynb", "jats", "nxml",
    "docbook", "dbk", "fb2", "opml",
})

# Binary: pandoc can convert these to HTML
PANDOC_TO_HTML = frozenset({
    "docx", "epub", "odt", "rtf", "pptx", "xlsx",
    "ppsx", "pptm", "xlsm",
})

PANDOC_FORMAT_ALIASES = {
    "ppsx": "pptx", "pptm": "pptx", "xlsm": "xlsx",
}

# Binary: libreoffice converts these to HTML
LIBREOFFICE_TO_HTML = frozenset({"doc", "xls", "ods", "xlsb", "ppt"})

EMAIL_TYPES = frozenset({"eml", "msg"})

SKIP_TYPES = frozenset({"7z", "gz", "tar", "tgz", "zip", "lz4", "xla", "xlam"})

ALL_HANDLED = (
    DIRECT_READ_PDF | DIRECT_READ_IMAGE | DIRECT_READ_TEXT |
    PANDOC_TO_HTML | LIBREOFFICE_TO_HTML | EMAIL_TYPES
)


# ---------------------------------------------------------------------------
# Conversion helpers
# ---------------------------------------------------------------------------

def pandoc_to_html(doc_path: Path, file_type: str, out_dir: Path) -> Path:
    """Convert binary format to HTML via pandoc."""
    out_html = out_dir / f"{doc_path.stem}_{file_type}.html"
    input_format = PANDOC_FORMAT_ALIASES.get(file_type, file_type)
    cmd = ["pandoc", str(doc_path), "-f", input_format, "-t", "html",
           "--standalone", "-o", str(out_html)]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        raise RuntimeError(f"pandoc failed: {result.stderr[:300]}")
    return out_html


def libreoffice_to_html(doc_path: Path, out_dir: Path) -> Path:
    """Convert binary format to HTML via LibreOffice."""
    result = subprocess.run(
        ["soffice", "--headless", "--convert-to", "html",
         "--outdir", str(out_dir), str(doc_path)],
        capture_output=True, text=True, timeout=120,
    )
    if result.returncode != 0:
        raise RuntimeError(f"libreoffice failed: {result.stderr[:300]}")
    html_files = list(out_dir.glob(f"{doc_path.stem}*.html"))
    if not html_files:
        raise RuntimeError("libreoffice produced no HTML")
    return html_files[0]


def extract_email_text(doc_path: Path, file_type: str, out_dir: Path) -> Path:
    """Extract email body as plain text."""
    out_txt = out_dir / f"{doc_path.stem}.txt"

    if file_type == "msg":
        # Try extract-msg, fall back to raw read
        try:
            import extract_msg
            msg = extract_msg.openMsg(str(doc_path))
            parts = []
            for attr, label in [("subject", "Subject"), ("sender", "From"), ("to", "To")]:
                val = getattr(msg, attr, None)
                if val:
                    parts.append(f"{label}: {val}")
            if parts:
                parts.append("")
            if msg.body:
                parts.append(msg.body)
            msg.close()
            text = "\n".join(parts)
        except ImportError:
            text = doc_path.read_text(encoding="utf-8", errors="replace")
    else:
        # .eml via stdlib
        try:
            raw = doc_path.read_bytes()
            msg = email_mod.message_from_bytes(raw)
        except Exception:
            raw_text = doc_path.read_text(encoding="utf-8", errors="replace")
            msg = email_mod.message_from_string(raw_text)

        parts = []
        for header in ("From", "To", "Subject", "Date"):
            val = msg.get(header)
            if val:
                parts.append(f"{header}: {val}")
        if parts:
            parts.append("")

        if msg.is_multipart():
            for part in msg.walk():
                if part.get_content_type() == "text/plain":
                    payload = part.get_payload(decode=True)
                    if payload:
                        charset = part.get_content_charset() or "utf-8"
                        try:
                            parts.append(payload.decode(charset, errors="replace"))
                        except (LookupError, UnicodeDecodeError):
                            parts.append(payload.decode("utf-8", errors="replace"))
        else:
            payload = msg.get_payload(decode=True)
            if payload:
                charset = msg.get_content_charset() or "utf-8"
                try:
                    parts.append(payload.decode(charset, errors="replace"))
                except (LookupError, UnicodeDecodeError):
                    parts.append(payload.decode("utf-8", errors="replace"))
        text = "\n".join(parts)

    out_txt.write_text(text, encoding="utf-8")
    return out_txt


# ---------------------------------------------------------------------------
# Core
# ---------------------------------------------------------------------------

def get_repo_root() -> Path:
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / "Cargo.toml").exists() and (current / "test_documents").exists():
            return current
        current = current.parent
    raise RuntimeError("Could not find repository root")


def make_mapping_key(fixture_path: Path, fixtures_dir: Path) -> str:
    rel = fixture_path.relative_to(fixtures_dir)
    parts = rel.parts
    if len(parts) > 1:
        return f"{parts[0]}/{fixture_path.stem}"
    return fixture_path.stem


def main() -> int:
    parser = argparse.ArgumentParser(description="Phase 1: Prepare documents for AI GT extraction")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--format-filter", type=str, default="")
    parser.add_argument("--skip-types", type=str, default="")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--manifest", type=str, default="/tmp/gt-vision-manifest.json")
    args = parser.parse_args()

    repo_root = get_repo_root()
    fixtures_dir = repo_root / "tools" / "benchmark-harness" / "fixtures"
    render_dir = Path("/tmp/gt-vision-rendered")
    render_dir.mkdir(parents=True, exist_ok=True)

    print(f"Repository root: {repo_root}")
    print(f"Fixtures dir:    {fixtures_dir}")
    print(f"Render dir:      {render_dir}")
    if args.dry_run:
        print("DRY RUN MODE\n")

    format_filter = set(args.format_filter.split(",")) if args.format_filter else None
    skip_types = set(args.skip_types.split(",")) if args.skip_types else set()

    fixture_paths = sorted(fixtures_dir.rglob("*.json"))
    print(f"Found {len(fixture_paths)} fixture files\n")

    manifest: list[dict] = []
    stats = {"prepared": 0, "skipped": 0, "errors": 0}

    for fixture_path in fixture_paths:
        try:
            with open(fixture_path) as f:
                fixture = json.load(f)
        except (json.JSONDecodeError, OSError):
            stats["errors"] += 1
            continue

        file_type = fixture.get("file_type", "")

        if file_type in SKIP_TYPES or file_type not in ALL_HANDLED:
            stats["skipped"] += 1
            continue
        if format_filter and file_type not in format_filter:
            continue
        if file_type in skip_types:
            continue

        gt = fixture.get("ground_truth")
        if gt and gt.get("source") == "vision" and not args.force:
            stats["skipped"] += 1
            continue

        doc_rel = fixture.get("document", "")
        if not doc_rel:
            stats["skipped"] += 1
            continue
        doc_path = (fixture_path.parent / doc_rel).resolve()
        if not doc_path.exists():
            print(f"  SKIP (missing): {fixture_path.name} -> {doc_path}")
            stats["skipped"] += 1
            continue

        gt_dir = repo_root / "test_documents" / "ground_truth" / file_type
        gt_filename = fixture_path.stem + ".txt"
        gt_path = gt_dir / gt_filename
        gt_rel = os.path.relpath(gt_path, fixture_path.parent)
        mapping_key = make_mapping_key(fixture_path, fixtures_dir)

        # Determine read mode and prepare readable file
        read_file = str(doc_path)
        read_mode = "text"

        if file_type in DIRECT_READ_PDF:
            read_mode = "pdf"
        elif file_type in DIRECT_READ_IMAGE:
            read_mode = "image"
        elif file_type in DIRECT_READ_TEXT:
            read_mode = "text"
        elif file_type in PANDOC_TO_HTML:
            if not args.dry_run:
                try:
                    html_path = pandoc_to_html(doc_path, file_type, render_dir)
                    read_file = str(html_path)
                except Exception:
                    # Fallback to libreoffice
                    try:
                        html_path = libreoffice_to_html(doc_path, render_dir)
                        read_file = str(html_path)
                    except Exception as e2:
                        print(f"  ERROR (pandoc+lo): {fixture_path.name}: {e2}")
                        stats["errors"] += 1
                        continue
        elif file_type in LIBREOFFICE_TO_HTML:
            if not args.dry_run:
                try:
                    html_path = libreoffice_to_html(doc_path, render_dir)
                    read_file = str(html_path)
                except Exception as e:
                    print(f"  ERROR (libreoffice): {fixture_path.name}: {e}")
                    stats["errors"] += 1
                    continue
        elif file_type in EMAIL_TYPES:
            if not args.dry_run:
                try:
                    txt_path = extract_email_text(doc_path, file_type, render_dir)
                    read_file = str(txt_path)
                except Exception as e:
                    print(f"  ERROR (email): {fixture_path.name}: {e}")
                    stats["errors"] += 1
                    continue

        entry = {
            "fixture_path": str(fixture_path),
            "fixture_name": fixture_path.stem,
            "file_type": file_type,
            "read_file": read_file,
            "read_mode": read_mode,
            "gt_path": str(gt_path),
            "gt_rel": gt_rel,
            "mapping_key": mapping_key,
        }
        manifest.append(entry)
        stats["prepared"] += 1

        if args.dry_run:
            print(f"  [DRY] {fixture_path.name} ({file_type}) mode={read_mode}")
        else:
            print(f"  OK {fixture_path.name} ({file_type}) -> {read_file}")

    manifest_path = Path(args.manifest)
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    print(f"\n{'='*50}")
    print(f"Manifest: {manifest_path}")
    print(f"  Prepared: {stats['prepared']}")
    print(f"  Skipped:  {stats['skipped']}")
    print(f"  Errors:   {stats['errors']}")
    print(f"\nNext: Process manifest with AI subagents (Phase 2)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
