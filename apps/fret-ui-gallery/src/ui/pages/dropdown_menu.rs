use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::dropdown_menu as snippets;

pub(super) fn preview_dropdown_menu(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let submenu = snippets::submenu::render(cx);
    let shortcuts = snippets::shortcuts::render(cx);
    let icons = snippets::icons::render(cx);
    let checkboxes = snippets::checkboxes::render(cx);
    let checkboxes_icons = snippets::checkboxes_icons::render(cx);
    let radio_group = snippets::radio_group::render(cx);
    let radio_icons = snippets::radio_icons::render(cx);
    let destructive = snippets::destructive::render(cx);
    let avatar = snippets::avatar::render(cx);
    let complex = snippets::complex::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);

    let api_reference = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/base/dropdown-menu.mdx`.",
        "`DropdownMenu::build_parts(...)` and `into_element_parts(...)` are the source-aligned authoring helpers for `Trigger` / `Content`; they keep typed `DropdownMenuEntry` values explicit instead of introducing a generic slot/asChild surface.",
        "`DropdownMenuCheckboxItem::from_checked(...)` / `.on_checked_change(...)` and `DropdownMenuRadioGroup::from_value(...)` / `.on_value_change(...)` now cover the upstream snapshot + callback path without forcing per-item `Model<_>` state.",
        "A separate generic `compose()` / nested children API is still not the next recommended step here; the parts bridge already covers `Trigger` / `Content`, and typed entries remain the more important contract.",
    ]);

    let notes = doc_layout::notes_block([
        "Preview follows the upstream shadcn Dropdown Menu docs (v4 Base UI) order first, then appends Fret-only follow-ups.",
        "Mechanism parity is largely covered already: existing web-vs-fret chrome/placement gates and dropdown diag scripts cover placement, dismissal, focus restore, submenu routing, and safe-corridor behavior.",
        "The checkable examples now demonstrate snapshot + callback authoring, so simple menus do not need one `Model<bool>` per checkbox row.",
        "The `Parts` section is intentionally outside the upstream docs path: it documents the Fret adapter surface rather than a missing shadcn feature.",
        "Examples are snippet-backed, so preview and code stay in sync.",
        "Keep `ui-gallery-dropdown-menu-*` test IDs stable; multiple diag scripts depend on them.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-dropdown-menu-api-reference")
        .description(
            "Public surface ownership, remaining authoring drift, and children API guidance.",
        );
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-dropdown-menu-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Dropdown Menu examples aligned with the upstream shadcn docs path, plus Fret-specific API and parts notes after the docs-aligned sections.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Official shadcn-style dropdown menu demo with shortcuts and a submenu.")
                .test_id_prefix("ui-gallery-dropdown-menu-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable source-aligned usage closest to the official docs path.")
                .test_id_prefix("ui-gallery-dropdown-menu-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("A minimal dropdown menu with labels, separators, and disabled items.")
                .test_id_prefix("ui-gallery-dropdown-menu-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Submenu", submenu)
                .description("Use `DropdownMenuSub*` helpers to nest secondary actions.")
                .test_id_prefix("ui-gallery-dropdown-menu-submenu")
                .code_rust_from_file_region(snippets::submenu::SOURCE, "example"),
            DocSection::new("Shortcuts", shortcuts)
                .description("Use `DropdownMenuShortcut` to show keyboard hints.")
                .test_id_prefix("ui-gallery-dropdown-menu-shortcuts")
                .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example"),
            DocSection::new("Icons", icons)
                .description("Combine leading icons with labels for quicker scanning.")
                .test_id_prefix("ui-gallery-dropdown-menu-icons")
                .code_rust_from_file_region(snippets::icons::SOURCE, "example"),
            DocSection::new("Checkboxes", checkboxes)
                .description("Use checkbox items for toggle-style actions.")
                .test_id_prefix("ui-gallery-dropdown-menu-checkboxes")
                .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example"),
            DocSection::new("Checkboxes Icons", checkboxes_icons)
                .description("Add icons to checkbox items without disturbing the indicator slot.")
                .test_id_prefix("ui-gallery-dropdown-menu-checkboxes-icons")
                .code_rust_from_file_region(snippets::checkboxes_icons::SOURCE, "example"),
            DocSection::new("Radio Group", radio_group)
                .description("Use radio items for mutually exclusive choices.")
                .test_id_prefix("ui-gallery-dropdown-menu-radio-group")
                .code_rust_from_file_region(snippets::radio_group::SOURCE, "example"),
            DocSection::new("Radio Icons", radio_icons)
                .description("Show radio options with icons while preserving row alignment.")
                .test_id_prefix("ui-gallery-dropdown-menu-radio-icons")
                .code_rust_from_file_region(snippets::radio_icons::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .description("Use `variant=Destructive` for irreversible actions.")
                .test_id_prefix("ui-gallery-dropdown-menu-destructive")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .description("An account menu triggered by an avatar-style button.")
                .test_id_prefix("ui-gallery-dropdown-menu-avatar")
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Complex", complex)
                .description("A richer menu mixing groups, icons, toggles, and nested submenus.")
                .test_id_prefix("ui-gallery-dropdown-menu-complex")
                .code_rust_from_file_region(snippets::complex::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("RTL layout keeps spacing, alignment, and submenu direction auditable.")
                .test_id_prefix("ui-gallery-dropdown-menu-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            api_reference,
            DocSection::new("Parts", parts)
                .description("Fret-only Trigger/Content adapter surface kept outside the upstream docs path.")
                .test_id_prefix("ui-gallery-dropdown-menu-parts")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-dropdown-menu")]
}
