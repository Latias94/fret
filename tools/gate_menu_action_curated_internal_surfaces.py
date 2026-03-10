from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep a very small set of curated internal/app-facing menu surfaces on the action-first
    # spelling once their widget families have shipped `action(...)` aliases.
    #
    # This is intentionally not a broad internal policy sweep. It only protects the specific
    # surfaces we have explicitly aligned as part of the post-v1 residue cleanup.
    run_regex_gate(
        "curated internal menu surfaces prefer action(...)",
        roots=[
            Path("ecosystem/fret-workspace/src/tab_strip/overflow.rs"),
            Path("ecosystem/fret-genui-shadcn/src/resolver/overlay.rs"),
        ],
        patterns=[
            r"\.on_select\s*\(",
            r"trailing_on_select\s*\(",
        ],
        include_glob="*.rs",
    )


if __name__ == "__main__":
    main()
