#!/usr/bin/env python3
"""
Fetch the CJK source font (Noto Sans CJK SC) into `crates/fret-fonts/assets/_sources`.

Cross-platform replacement for `crates/fret-fonts/scripts/fetch_cjk_font.ps1`.
"""

from __future__ import annotations

import argparse
import os
import sys
import urllib.request
from pathlib import Path


def _default_assets_dir() -> Path:
    return Path(__file__).parent.parent / "assets"


def _download(url: str, out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with urllib.request.urlopen(url) as resp:
        out_path.write_bytes(resp.read())


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--assets-dir", default=str(_default_assets_dir()))
    args = parser.parse_args(argv)

    assets_dir = Path(args.assets_dir).expanduser().resolve()
    sources_dir = assets_dir / "_sources"
    sources_dir.mkdir(parents=True, exist_ok=True)

    font_url = "https://raw.githubusercontent.com/googlefonts/noto-cjk/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"
    license_url = "https://raw.githubusercontent.com/notofonts/noto-fonts/main/LICENSE"

    _download(font_url, sources_dir / "NotoSansCJKsc-Regular.otf")
    _download(license_url, assets_dir / "NotoSansCJK-LICENSE.txt")

    for p in sorted(assets_dir.glob("NotoSansCJK*")):
        if p.is_file():
            print(f"{p.name}\t{p.stat().st_size}")
    for p in sorted(sources_dir.glob("NotoSansCJK*")):
        if p.is_file():
            print(f"_sources/{p.name}\t{p.stat().st_size}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
