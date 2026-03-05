from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, run_regex_gate


def main() -> None:
    gate_name = "legacy stack removed from fret-ui-kit"

    stack_file = WORKSPACE_ROOT / "ecosystem/fret-ui-kit/src/declarative/stack.rs"
    if stack_file.exists():
        fail(gate_name, "unexpected file exists: ecosystem/fret-ui-kit/src/declarative/stack.rs")

    run_regex_gate(
        gate_name,
        roots=[Path("ecosystem/fret-ui-kit/src")],
        include_glob="**/*.rs",
        patterns=[
            r"\bmod\s+stack\b",
            r"\bpub\s+mod\s+stack\b",
            r"\bpub\(crate\)\s+mod\s+stack\b",
            r"\bpub\(super\)\s+mod\s+stack\b",
            r"\bdeclarative::stack\b",
            r"\bstack::(hstack|vstack)(_build)?\b",
        ],
    )


if __name__ == "__main__":
    main()
