from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Default teaching surfaces should stay on the narrowed helper set:
    # - `on_action_notify_models`
    # - `on_action_notify_transient`
    # - local `on_activate*`
    #
    # Single-model aliases remain supported as advanced/reference helpers, but they should not
    # reappear in `fret-examples` or ui-gallery teaching pages/snippets. Scaffold templates keep
    # their own unit-test assertions because this gate is source-oriented.
    run_regex_gate(
        "default teaching surfaces avoid single-model action helper aliases",
        roots=[
            Path("apps/fret-examples/src"),
            Path("apps/fret-ui-gallery/src/ui/pages"),
            Path("apps/fret-ui-gallery/src/ui/snippets"),
        ],
        include_glob="*.rs",
        patterns=[
            r"\bcx\.on_action_notify_model_update::<",
            r"\bcx\.on_action_notify_model_set::<",
            r"\bcx\.on_action_notify_toggle_bool::<",
        ],
    )


if __name__ == "__main__":
    main()
