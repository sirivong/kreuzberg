"""CLI entry point for the kreuzberg proxy.

A platform-specific wheel bundles the native binary under
``kreuzberg_cli/bin/<target>/``; the sdist (and unknown platforms) ship no binary
and fall back to the runtime downloader. Each wheel contains exactly one ``bin/<target>``
directory, so we locate the binary by globbing rather than recomputing the target
triple (which cannot distinguish glibc from musl at runtime).
"""

from __future__ import annotations

import os
import platform
import sys
from pathlib import Path

from .downloader import run


def _find_bundled_binary() -> str | None:
    """Return the path to the bundled native binary if this wheel shipped one."""
    bin_root = Path(__file__).parent / "bin"
    if not bin_root.is_dir():
        return None

    binary_name = "kreuzberg.exe" if platform.system().lower() == "windows" else "kreuzberg"
    for candidate in bin_root.glob(f"*/{binary_name}"):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    return None


def main() -> None:
    """Resolve the native binary (bundled or downloaded) and exec it with forwarded argv."""
    bundled = _find_bundled_binary()
    if bundled:
        import subprocess

        completed = subprocess.run([bundled, *sys.argv[1:]], check=False)
        sys.exit(completed.returncode)

    # Fall back to the runtime download path (sdist / unknown platform).
    sys.exit(run(sys.argv[1:]))


if __name__ == "__main__":
    main()
