from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, ok


GATE_NAME = "FnDriver example naming"
EXAMPLES_DIR = WORKSPACE_ROOT / "apps/fret-examples/src"
BUILD_DRIVER_RE = re.compile(r"\bfn\s+build_driver\s*\(")


def main() -> None:
    problems: list[str] = []

    for path in sorted(EXAMPLES_DIR.glob("*.rs")):
        text = path.read_text(encoding="utf-8", errors="replace")
        if "FnDriver::new" not in text:
            continue
        if BUILD_DRIVER_RE.search(text) is None:
            continue
        problems.append(
            f"{path.relative_to(WORKSPACE_ROOT)} uses `FnDriver::new(...)` but still names the helper `build_driver(...)`"
        )

    if problems:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(problems)} issue(s)")
        for problem in problems:
            print(f"  - {problem}")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()