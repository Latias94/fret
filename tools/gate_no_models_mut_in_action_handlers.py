from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # This gate is intentionally narrow and teaching-surface-focused: it prevents the
    # cookbook/examples from drifting back to the more verbose `move |host, _acx| host.models_mut()...`
    # style now that `AppUi` exposes `cx.actions().models::<...>(...)`.
    #
    # It is not a global style rule for the whole repo: internal widgets, diagnostics harnesses,
    # and low-level mechanisms can still use `models_mut()` directly where appropriate.
    repo = Path("apps")
    run_regex_gate(
        "no verbose models_mut usage in cookbook/examples action handlers",
        roots=[
            repo / "fret-cookbook" / "examples",
            repo / "fret-examples" / "src",
        ],
        include_glob="*.rs",
        patterns=[
            r"move\s*\|\s*host\s*,\s*_acx\s*\|\s*\{.*host\.models_mut\(\)",
            r"move\s*\|\s*host\s*,\s*_action_cx\s*\|\s*\{.*host\.models_mut\(\)",
            r"move\s*\|\s*host\s*,\s*acx\s*\|\s*\{.*host\.models_mut\(\)",
        ],
    )


if __name__ == "__main__":
    main()
