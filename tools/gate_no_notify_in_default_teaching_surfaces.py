from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Default teaching surfaces should stay on tracked writes / transient helpers.
    # Explicit `notify()` remains a public escape hatch, but it should not drift back into the
    # first-contact ladder or the scaffold defaults.
    run_regex_gate(
        "default teaching surfaces avoid explicit notify escape hatches",
        roots=[
            Path("apps/fret-cookbook/examples/hello.rs"),
            Path("apps/fret-cookbook/examples/simple_todo.rs"),
            Path("apps/fret-cookbook/examples/hello_counter.rs"),
            Path("apps/fret-cookbook/examples/overlay_basics.rs"),
            Path("apps/fret-cookbook/examples/text_input_basics.rs"),
            Path("apps/fret-cookbook/examples/commands_keymap_basics.rs"),
            Path("apps/fret-cookbook/examples/virtual_list_basics.rs"),
            Path("apps/fret-cookbook/examples/effects_layer_basics.rs"),
            Path("apps/fret-cookbook/examples/theme_switching_basics.rs"),
            Path("apps/fret-cookbook/examples/form_basics.rs"),
            Path("apps/fret-cookbook/examples/drag_basics.rs"),
            Path("apps/fretboard/src/scaffold/templates.rs"),
        ],
        patterns=[
            r"\bcx\.notify\(",
            r"\bhost\.notify\(",
        ],
        include_glob="*.rs",
    )


if __name__ == "__main__":
    main()
