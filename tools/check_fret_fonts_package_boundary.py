#!/usr/bin/env python3
"""Assert the published package boundary for the Fret bundled-font crates."""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class PackageBoundaryExpectation:
    package: str
    required_paths: tuple[str, ...]
    forbidden_paths: tuple[str, ...]


EXPECTATIONS: tuple[PackageBoundaryExpectation, ...] = (
    PackageBoundaryExpectation(
        package="fret-fonts",
        required_paths=(
            "assets/FiraMono-subset.ttf",
            "assets/Inter-italic-subset.ttf",
            "assets/Inter-roman-subset.ttf",
            "assets/JetBrainsMono-italic-subset.ttf",
            "assets/JetBrainsMono-roman-subset.ttf",
            "assets/RobotoSlab-VariableFont_wght.ttf",
        ),
        forbidden_paths=(
            "assets/Inter-italic.ttf",
            "assets/Inter-roman.ttf",
            "assets/JetBrainsMono-italic.ttf",
            "assets/JetBrainsMono-roman.ttf",
            "assets/NotoColorEmoji.ttf",
            "assets/NotoEmoji-LICENSE.txt",
            "assets/NotoSansCJK-LICENSE.txt",
            "assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf",
            "assets/cjk-lite-text.txt",
        ),
    ),
    PackageBoundaryExpectation(
        package="fret-fonts-cjk",
        required_paths=(
            "assets/NotoSansCJK-LICENSE.txt",
            "assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf",
            "assets/cjk-lite-text.txt",
        ),
        forbidden_paths=(
            "assets/NotoColorEmoji.ttf",
            "assets/NotoEmoji-LICENSE.txt",
            "assets/Inter-italic.ttf",
            "assets/Inter-roman.ttf",
            "assets/JetBrainsMono-italic.ttf",
            "assets/JetBrainsMono-roman.ttf",
        ),
    ),
    PackageBoundaryExpectation(
        package="fret-fonts-emoji",
        required_paths=(
            "assets/NotoColorEmoji.ttf",
            "assets/NotoEmoji-LICENSE.txt",
        ),
        forbidden_paths=(
            "assets/NotoSansCJK-LICENSE.txt",
            "assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf",
            "assets/cjk-lite-text.txt",
            "assets/Inter-italic.ttf",
            "assets/Inter-roman.ttf",
            "assets/JetBrainsMono-italic.ttf",
            "assets/JetBrainsMono-roman.ttf",
        ),
    ),
)


def package_file_list(package: str) -> set[str]:
    proc = subprocess.run(
        [
            "cargo",
            "package",
            "--allow-dirty",
            "--locked",
            "--list",
            "-p",
            package,
        ],
        check=False,
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        if proc.stdout:
            print(proc.stdout, file=sys.stderr, end="")
        if proc.stderr:
            print(proc.stderr, file=sys.stderr, end="")
        raise SystemExit(
            f"[fonts-package-boundary] cargo package --list failed for {package}"
            f" (exit code: {proc.returncode})"
        )
    return {
        line.strip()
        for line in proc.stdout.splitlines()
        if line.strip() and not line.lstrip().startswith("Blocking waiting")
    }


def check_expectation(expectation: PackageBoundaryExpectation) -> None:
    print(f"[fonts-package-boundary] {expectation.package}")
    files = package_file_list(expectation.package)

    missing = sorted(
        path for path in expectation.required_paths if path not in files
    )
    unexpected = sorted(
        path for path in expectation.forbidden_paths if path in files
    )

    if not missing and not unexpected:
        print("[fonts-package-boundary] ok")
        return

    print("[fonts-package-boundary] FAIL")
    for path in missing:
        print(f"  missing: {path}")
    for path in unexpected:
        print(f"  unexpected: {path}")
    raise SystemExit(1)


def main(argv: list[str]) -> int:
    if argv:
        raise SystemExit("This script takes no arguments.")

    for expectation in EXPECTATIONS:
        check_expectation(expectation)

    print("[fonts-package-boundary] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
