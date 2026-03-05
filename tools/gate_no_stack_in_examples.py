from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    run_regex_gate(
        "no stack helpers in examples",
        roots=[Path("apps/fret-examples")],
        include_glob="**/*.rs",
        patterns=[
            r"\bstack::(hstack|vstack)(_build)?\b",
            r"\bshadcn::stack::(hstack|vstack)\b",
            r"\buse\s+fret_ui_kit::declarative::stack\b",
            r"\buse\s+fret_ui_shadcn::stack\b",
        ],
    )


if __name__ == "__main__":
    main()

