from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the default Material3 snackbar teaching surface on the explicit action-first spelling.
    #
    # This gate is intentionally narrow: it only protects the gallery snackbar snippet that serves
    # as the first-contact reference surface for this API family. Lower-level/runtime code and
    # broader tests may still use the compat spellings where appropriate.
    run_regex_gate(
        "material3 snackbar default surface prefers action_id",
        roots=[
            Path("apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs"),
        ],
        patterns=[
            r"\.action_command\s*\(",
            r"\.action\s*\(",
        ],
    )


if __name__ == "__main__":
    main()
