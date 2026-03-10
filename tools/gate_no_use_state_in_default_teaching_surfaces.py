from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the default local-state story narrow.
    #
    # This gate is intentionally scoped to first-contact/reference teaching surfaces that should
    # stay on `use_local*`. It does not ban `use_state` from the runtime, migration docs, or other
    # advanced explicit raw-model seams.
    #
    # Scaffold templates are covered separately by `templates.rs` unit tests because the emitted
    # source is generated from string builders rather than stored as standalone files.
    run_regex_gate(
        "default teaching surfaces avoid use_state",
        roots=[
            Path("apps/fret-cookbook/examples/hello.rs"),
            Path("apps/fret-cookbook/examples/overlay_basics.rs"),
            Path("apps/fret-cookbook/examples/imui_action_basics.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs"),
            Path("docs/examples/todo-app-golden-path.md"),
        ],
        patterns=[
            r"\bcx\.use_state::<",
            r"\bcx\.use_state_with\s*\(",
            r"\bcx\.use_state_keyed::<",
            r"\buse_state::<",
        ],
    )


if __name__ == "__main__":
    main()
