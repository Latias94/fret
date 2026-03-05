from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    run_regex_gate(
        "no MVU in tree",
        roots=[Path("apps"), Path("crates"), Path("ecosystem")],
        include_glob="**/*.rs",
        patterns=[
            r"\bimpl\s+MvuProgram\b",
            r"\bMvuProgram\b",
            r"\bMessageRouter\b",
            r"\bKeyedMessageRouter\b",
            r"\brun_mvu\b",
            r"\bMvuWindowState\b",
            r"\bfret::mvu\b",
            r"\bfret::mvu_router\b",
            r"\bfret::legacy\b",
            r"\blegacy[-_]mvu\b",
        ],
    )


if __name__ == "__main__":
    main()

