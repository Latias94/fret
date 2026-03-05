from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    run_regex_gate(
        "no MVU in cookbook",
        roots=[Path("apps/fret-cookbook")],
        include_glob="**/*.rs",
        patterns=[
            r"\bimpl\s+MvuProgram\b",
            r"\brun_mvu\b",
            r"\bMessageRouter\b",
            r"\bKeyedMessageRouter\b",
            r"\bfret::mvu\b",
            r"\bfret::mvu_router\b",
        ],
    )


if __name__ == "__main__":
    main()

