from __future__ import annotations

import argparse
from pathlib import Path


def add_gb2312(chars: set[str]) -> None:
    for hi in range(0xA1, 0xFF):
        for lo in range(0xA1, 0xFF):
            b = bytes((hi, lo))
            try:
                s = b.decode("gb2312")
            except UnicodeDecodeError:
                continue
            chars.update(s)


def add_euc_kr(chars: set[str]) -> None:
    for hi in range(0xA1, 0xFF):
        for lo in range(0xA1, 0xFF):
            b = bytes((hi, lo))
            try:
                s = b.decode("euc_kr")
            except UnicodeDecodeError:
                continue
            chars.update(s)


def add_shift_jis(chars: set[str]) -> None:
    for b in range(0x20, 0x7F):
        chars.add(bytes((b,)).decode("shift_jis"))
    for b in range(0xA1, 0xE0):
        chars.add(bytes((b,)).decode("shift_jis"))
    for lead in list(range(0x81, 0xA0)) + list(range(0xE0, 0xFD)):
        for trail in list(range(0x40, 0x7F)) + list(range(0x80, 0xFD)):
            b = bytes((lead, trail))
            try:
                s = b.decode("shift_jis")
            except UnicodeDecodeError:
                continue
            chars.update(s)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    out_path = Path(args.out)

    chars: set[str] = set()
    add_gb2312(chars)
    add_shift_jis(chars)
    add_euc_kr(chars)

    ordered = "".join(sorted(chars, key=ord))
    out_path.write_text(ordered + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
