#!/usr/bin/env python3
"""
Subset bundled UI fonts for lighter builds.

Cross-platform replacement for `crates/fret-fonts/scripts/subset_fonts.ps1`.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def _default_assets_dir() -> Path:
    return Path(__file__).parent.parent / "assets"


def _require_tool(name: str, hint: str) -> None:
    if shutil.which(name) is None:
        raise RuntimeError(f"{name} not found. {hint}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--assets-dir", default=str(_default_assets_dir()))
    args = parser.parse_args(argv)

    _require_tool("pyftsubset", "Install with: python -m pip install fonttools brotli")

    assets_dir = Path(args.assets_dir).expanduser().resolve()

    unicodes = ",".join(
        [
            "U+000A",
            "U+000D",
            "U+0020-007E",
            "U+00A0-00FF",
            "U+0100-017F",
            "U+0180-024F",
            "U+2000-206F",
            "U+20A0-20CF",
            "U+2190-21FF",
            "U+2200-22FF",
            "U+2300-23FF",
            "U+2460-24FF",
            "U+2500-257F",
            "U+2580-259F",
            "U+25A0-25FF",
        ]
    )

    common_args = [
        f"--unicodes={unicodes}",
        "--layout-features=*",
        "--glyph-names",
        "--symbol-cmap",
        "--notdef-glyph",
        "--notdef-outline",
        "--recommended-glyphs",
        "--name-IDs=*",
        "--name-legacy",
        "--name-languages=*",
        "--drop-tables+=DSIG",
        "--no-hinting",
    ]

    fonts = [
        ("Inter-roman.ttf", "Inter-roman-subset.ttf"),
        ("Inter-italic.ttf", "Inter-italic-subset.ttf"),
        ("JetBrainsMono-roman.ttf", "JetBrainsMono-roman-subset.ttf"),
        ("JetBrainsMono-italic.ttf", "JetBrainsMono-italic-subset.ttf"),
    ]

    for in_name, out_name in fonts:
        in_path = assets_dir / in_name
        out_path = assets_dir / out_name
        if not in_path.exists():
            raise RuntimeError(f"Missing input font: {in_path}")
        code = subprocess.run(
            ["pyftsubset", str(in_path), *common_args, f"--output-file={out_path}"],
            check=False,
        ).returncode
        if code != 0:
            return code

    for p in sorted(assets_dir.glob("*-subset.ttf")):
        if p.is_file():
            print(f"{p.name}\t{p.stat().st_size}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
