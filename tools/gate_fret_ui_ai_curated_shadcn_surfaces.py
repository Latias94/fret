from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


def main() -> None:
    gate_name = "fret-ui-ai prefers explicit shadcn facade/raw lanes"
    root = WORKSPACE_ROOT / "ecosystem/fret-ui-ai/src/elements"
    if not root.is_dir():
        fail(gate_name, f"missing source directory: {root}")

    violations: list[str] = []
    for path in sorted(root.rglob("*.rs")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for line_no, line in enumerate(text.splitlines(), start=1):
            trimmed = line.strip()
            if not trimmed.startswith("use fret_ui_shadcn::"):
                continue

            allowed = (
                trimmed.startswith("use fret_ui_shadcn::facade::")
                or trimmed.startswith("use fret_ui_shadcn::raw::")
                or trimmed.startswith("use fret_ui_shadcn::advanced::")
                or trimmed == "use fret_ui_shadcn::prelude::*;"
            )
            if allowed:
                continue

            rel = path.relative_to(WORKSPACE_ROOT)
            violations.append(f"{rel}:{line_no}: {trimmed}")

    if violations:
        preview = "\n".join(f"  - {line}" for line in violations[:40])
        suffix = ""
        if len(violations) > 40:
            suffix = f"\n  ... and {len(violations) - 40} more"
        fail(gate_name, f"found flat fret_ui_shadcn import lanes:\n{preview}{suffix}")

    ok(gate_name)


if __name__ == "__main__":
    main()
