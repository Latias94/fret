from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Teaching surfaces should prefer `ViewCx::on_action_notify*` helpers.
    #
    # Rationale:
    # - avoids repeating `request_redraw + notify` boilerplate,
    # - keeps view-cache closure behavior consistent across examples,
    # - makes authoring code easier to read and migrate.
    #
    # This is intentionally narrow: it only guards cookbook/examples plus ui-gallery
    # teaching pages/snippets, and does not apply to mechanism or widget internals.
    run_regex_gate(
        "no bare cx.on_action in teaching surfaces",
        roots=[
            Path("apps/fret-cookbook/examples"),
            Path("apps/fret-examples/src"),
            Path("apps/fret-ui-gallery/src/ui/pages"),
            Path("apps/fret-ui-gallery/src/ui/snippets"),
        ],
        include_glob="*.rs",
        patterns=[
            r"\bcx\.on_action\s*\(",
            r"\bcx\.on_action::<",
        ],
    )


if __name__ == "__main__":
    main()

