from __future__ import annotations

from pathlib import Path

from _gate_lib import run_regex_gate


def main() -> None:
    # Keep the curated UI Gallery action-capable widget snippets on the action-first spelling.
    #
    # This gate intentionally focuses on first-party teaching surfaces whose widgets already
    # expose stable action slots (`Pagination*`, `Item`, `BreadcrumbLink`,
    # `NavigationMenuItem`, `SidebarMenuButton`, and the link-render button snippet). It does not
    # cover callback-driven demos or business-table/internal command surfaces.
    run_regex_gate(
        "ui-gallery action-capable default surfaces prefer action(...)",
        roots=[
            Path("apps/fret-ui-gallery/src/ui/nav.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/navigation_menu/link_component.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/button/link_render.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/breadcrumb/link_component.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/breadcrumb/usage.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/avatar.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/demo.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/extras_rtl.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/gallery.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/image.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/link.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/link_render.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/item/size.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/demo.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/extras.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/icons_only.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/rtl.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/simple.rs"),
            Path("apps/fret-ui-gallery/src/ui/snippets/pagination/usage.rs"),
        ],
        patterns=[
            r"\.on_click\s*\(",
        ],
        include_glob="*.rs",
    )


if __name__ == "__main__":
    main()
