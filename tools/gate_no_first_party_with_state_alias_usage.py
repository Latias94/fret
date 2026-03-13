from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the compat aliases (`ElementContext::with_state*`) available for downstream migration,
    # but prevent first-party Rust code from drifting back to them now that
    # `root_state(...)` / `state_for(...)` / `slot_state(...)` are the preferred surfaces.
    run_regex_gate(
        "no first-party with_state compatibility alias usage",
        roots=[
            Path("crates"),
            Path("ecosystem"),
            Path("apps"),
        ],
        include_glob="*.rs",
        patterns=[
            r"\.with_state\(",
            r"\.with_state_for\(",
        ],
    )


if __name__ == "__main__":
    main()
