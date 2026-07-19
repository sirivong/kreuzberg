#!/usr/bin/env python3
"""Normalize external ground-truth markdown to canonical GFM.

External GT (ReaDoc, ParseBench, …) carries source-specific artifacts — LaTeXML/pandoc noise on
ReaDoc's arXiv side, HTML tables on ParseBench's — that must be canonicalized before the GT can be
scored consistently against xberg's markdown. This normalizes to the target convention the metric's
`markdown_quality::parse_markdown_blocks` parses: ATX headings, `$…$` / `$$…$$` math, GFM pipe
tables, fenced code. It preserves semantics (heading levels, list depth/order, table cells, formula
boundaries) and only rewrites presentation artifacts.

Usage:
    python normalize_gt.py <in.md> [<out.md>]      # normalize one file (stdout if no out)
    from normalize_gt import normalize             # normalize(md: str, source: str) -> str
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# --- LaTeXML/pandoc artifacts on ReaDoc arXiv GT ---------------------------------------------------

# Inline/display math delimiters: \( \) -> $ … $ ; \[ \] -> $$ … $$
_MATH_INLINE = re.compile(r"\\\((.+?)\\\)", re.DOTALL)
_MATH_DISPLAY = re.compile(r"\\\[(.+?)\\\]", re.DOTALL)

# Adjacent bold close+open collapsed by pandoc: "**Definition 1.1****.**" -> "**Definition 1.1.**".
# Four consecutive stars are a bold-close immediately followed by a bold-open; drop them to merge.
_DOUBLE_BOLD = re.compile(r"\*\*\*\*")

# Trailing whitespace and >2 blank lines.
_TRAILING_WS = re.compile(r"[ \t]+$", re.MULTILINE)
_BLANK_RUN = re.compile(r"\n{3,}")


# Each transform is (name, human description, compiled regex, replacement). Keeping them declarative
# lets build_corpus.py both apply them and generate the "how the data was modified" documentation
# from this single source of truth — the docs cannot drift from the code.
READOC_TRANSFORMS = [
    ("math_display", r"display math \[…\] → $$…$$", _MATH_DISPLAY, lambda m: f"$$ {m.group(1).strip()} $$"),
    ("math_inline", r"inline math \(…\) → $…$", _MATH_INLINE, lambda m: f"${m.group(1).strip()}$"),
    ("double_bold", "merge pandoc doubled bold ****  (bold-close+bold-open)", _DOUBLE_BOLD, ""),
]
COMMON_TRANSFORMS = [
    ("trailing_ws", "strip trailing whitespace", _TRAILING_WS, ""),
    ("blank_runs", "collapse >2 blank lines to one", _BLANK_RUN, "\n\n"),
]


def normalize_with_report(md: str, source: str = "") -> tuple[str, dict[str, int]]:
    """Normalize GT markdown to canonical GFM, returning (normalized, {transform: count}).

    The report records exactly which transforms fired and how many times — the precise, per-document
    record of how the data was modified from source.
    """
    report: dict[str, int] = {}
    passes = []
    if source.startswith("readoc") or source == "":
        passes += READOC_TRANSFORMS
    passes += COMMON_TRANSFORMS
    for name, _desc, pattern, repl in passes:
        md, n = pattern.subn(repl, md)
        if n:
            report[name] = n
    md = md.strip() + "\n"
    return md, report


def normalize(md: str, source: str = "") -> str:
    """Normalize GT markdown to canonical GFM (see normalize_with_report for the change record)."""
    return normalize_with_report(md, source)[0]


# --- HTML → GFM via the xberg-io html-to-markdown CLI (our lossless engine) -------------------------

import shutil  # noqa: E402
import subprocess  # noqa: E402

_H2M_BIN: str | None = None


def _html_to_markdown_cli() -> str:
    """Locate the sibling xberg-io/html-to-markdown CLI (prefer release), else PATH."""
    global _H2M_BIN
    if _H2M_BIN:
        return _H2M_BIN
    polyrepo = Path(__file__).resolve().parents[4]  # …/xberg-io
    for profile in ("release", "debug"):
        cand = polyrepo / "html-to-markdown" / "target" / profile / "html-to-markdown"
        if cand.exists():
            _H2M_BIN = str(cand)
            return _H2M_BIN
    found = shutil.which("html-to-markdown")
    if not found:
        raise RuntimeError(
            "html-to-markdown CLI not found — build it in ../html-to-markdown "
            "(`cargo build --release -p html-to-markdown-cli`) or put it on PATH"
        )
    _H2M_BIN = found
    return _H2M_BIN


def html_to_gfm(html: str) -> str:
    """Convert an HTML fragment (e.g. a ParseBench `<table>`) to GFM using the xberg-io
    html-to-markdown engine — lossless, colspan/rowspan/`<strong>`/`<br>`-aware — instead of a
    hand-rolled regex. Then apply the common canonicalization passes.
    """
    out = subprocess.run([_html_to_markdown_cli(), "-"], input=html.encode("utf-8"), capture_output=True, timeout=60)
    md = out.stdout.decode("utf-8", "replace")
    for _name, _desc, pattern, repl in COMMON_TRANSFORMS:
        md, _ = pattern.subn(repl, md)
    return md.strip() + "\n"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    src = Path(sys.argv[1])
    out = normalize(src.read_text("utf-8", "replace"), source="readoc")
    if len(sys.argv) >= 3:
        Path(sys.argv[2]).write_text(out)
    else:
        sys.stdout.write(out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
