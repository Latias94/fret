#!/usr/bin/env python3
"""
Produce a "CJK-lite" subset font for local dev / test usage.

Cross-platform replacement for `crates/fret-fonts/scripts/subset_cjk_lite.ps1`.
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
    sources_dir = assets_dir / "_sources"
    in_path = sources_dir / "NotoSansCJKsc-Regular.otf"
    if not in_path.exists():
        raise RuntimeError(f"Missing input font: {in_path} (run scripts/fetch_cjk_font.py first)")

    text_path = assets_dir / "cjk-lite-text.txt"
    if not text_path.exists():
        gen_script = Path(__file__).parent / "generate_cjk_lite_text.py"
        if not gen_script.exists():
            raise RuntimeError(f"Missing generator script: {gen_script}")
        subprocess.run([sys.executable, str(gen_script), "--out", str(text_path)], check=True)

    out_path = assets_dir / "NotoSansCJKsc-Regular-cjk-lite-subset.otf"

    common_args = [
        f"--text-file={text_path}",
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
        f"--output-file={out_path}",
    ]

    code = subprocess.run(["pyftsubset", str(in_path), *common_args], check=False).returncode
    if code != 0:
        return code

    if out_path.exists():
        print(f"{out_path.name}\t{out_path.stat().st_size}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
