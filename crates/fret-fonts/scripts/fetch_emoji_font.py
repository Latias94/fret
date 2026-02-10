#!/usr/bin/env python3
"""
Fetch the emoji font (Noto Color Emoji) into `crates/fret-fonts/assets`.

Cross-platform replacement for `crates/fret-fonts/scripts/fetch_emoji_font.ps1`.
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
    assets_dir.mkdir(parents=True, exist_ok=True)

    font_url = "https://raw.githubusercontent.com/googlefonts/noto-emoji/main/fonts/NotoColorEmoji.ttf"
    license_url = "https://raw.githubusercontent.com/googlefonts/noto-emoji/main/LICENSE"

    _download(font_url, assets_dir / "NotoColorEmoji.ttf")
    _download(license_url, assets_dir / "NotoEmoji-LICENSE.txt")

    for p in sorted(assets_dir.glob("Noto*")):
        if p.is_file():
            print(f"{p.name}\t{p.stat().st_size}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
