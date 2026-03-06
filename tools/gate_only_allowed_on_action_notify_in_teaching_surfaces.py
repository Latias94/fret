from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, iter_files, ok

GATE_NAME = "teaching surfaces only use approved advanced on_action_notify cases"
PATTERN = re.compile(r"\bcx\.on_action_notify::<")

ALLOWED: dict[str, tuple[str, ...]] = {
    "apps/fret-cookbook/examples/async_inbox_basics.rs": ("act::Start",),
    "apps/fret-cookbook/examples/router_basics.rs": ("act::RouterBack", "act::RouterForward"),
    "apps/fret-cookbook/examples/toast_basics.rs": (
        "act::DefaultToast",
        "act::SuccessToast",
        "act::DismissAll",
    ),
    "apps/fret-cookbook/examples/undo_basics.rs": ("act::Undo", "act::Redo"),
    "apps/fret-examples/src/async_playground_demo.rs": ("act::ToggleTheme",),
}


def main() -> None:
    roots = [
        WORKSPACE_ROOT / "apps/fret-cookbook/examples",
        WORKSPACE_ROOT / "apps/fret-examples/src",
        WORKSPACE_ROOT / "apps/fret-ui-gallery/src/ui/pages",
        WORKSPACE_ROOT / "apps/fret-ui-gallery/src/ui/snippets",
    ]
    files = iter_files(roots, include_glob="*.rs")
    if not files:
        fail(GATE_NAME, "no teaching-surface Rust files found")

    violations: list[tuple[Path, int, str]] = []
    for path in files:
        rel = path.resolve().relative_to(WORKSPACE_ROOT).as_posix()
        allowed_tokens = ALLOWED.get(rel, ())
        text = path.read_text(encoding="utf-8", errors="replace")
        for line_no, line in enumerate(text.splitlines(), start=1):
            if PATTERN.search(line) is None:
                continue
            if any(token in line for token in allowed_tokens):
                continue
            violations.append((Path(rel), line_no, line.strip()))

    if violations:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(violations)} unexpected advanced on_action_notify occurrence(s)")
        for rel, line_no, line in violations[:40]:
            print(f"  - {rel}:{line_no}")
            print(f"      {line}")
        if len(violations) > 40:
            print(f"  ... and {len(violations) - 40} more")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
