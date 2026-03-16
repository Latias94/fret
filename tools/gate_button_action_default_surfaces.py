from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the default first-party button teaching surfaces on the action-first builder spelling.
    #
    # This gate is intentionally narrow: it only protects curated cookbook/examples, UI Gallery
    # driver/snippet surfaces, and the components gallery slices that currently teach button-bound
    # stable action IDs. Advanced/manual callback cases may still use on_activate(...) where the
    # widget does not expose a stable action slot or where imperative host glue is the point.
    run_regex_gate(
        "default button teaching surfaces prefer action(...)",
        roots=[
            Path("apps/fret-cookbook/examples/chart_interactions_basics.rs"),
            Path("apps/fret-cookbook/examples/docking_basics.rs"),
            Path("apps/fret-cookbook/examples/embedded_viewport_basics.rs"),
            Path("apps/fret-cookbook/examples/external_texture_import_basics.rs"),
            Path("apps/fret-cookbook/examples/gizmo_basics.rs"),
            Path("apps/fret-cookbook/examples/utility_window_materials_windows.rs"),
            Path("apps/fret-examples/src/components_gallery.rs"),
            Path("apps/fret-ui-gallery/src/driver/chrome.rs"),
            Path("apps/fret-ui-gallery/src/driver/settings_sheet.rs"),
            Path("apps/fret-ui-gallery/src/ui/previews/pages/harness/view_cache.rs"),
            Path("apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/torture.rs"),
            Path("apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp/header.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/input/file.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/toast/deprecated.rs"),
        ],
        patterns=[
            r"\.on_click\s*\(",
        ],
        include_glob="*.rs",
    )


if __name__ == "__main__":
    main()
