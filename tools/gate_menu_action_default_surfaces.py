from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the default menu teaching surfaces on the action-first builder spelling.
    #
    # This gate is intentionally narrow: it only protects the primary ui-gallery context-menu and
    # menubar snippets that act as first-contact/reference surfaces for those widget families.
    # Lower-level tests, advanced helper surfaces, and runtime internals may still use
    # command-shaped spellings where appropriate.
    run_regex_gate(
        "default menu teaching surfaces prefer action(...)",
        roots=[
            Path("apps/fret-ui-gallery/src/ui/snippets/context_menu"),
            Path("apps/fret-ui-gallery/src/ui/snippets/menubar"),
        ],
        patterns=[
            r"\.on_select\s*\(",
        ],
        include_glob="*.rs",
    )


if __name__ == "__main__":
    main()
