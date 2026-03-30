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
        "Reference baseline: shadcn base Dropdown Menu docs.",
        "`DropdownMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` is now the default copyable root path, while `build_parts(...)` / `into_element_parts(...)` remain lower-level adapters for closure-driven or already-landed seams.",
        "`DropdownMenuTrigger`, `DropdownMenuPortal`, `DropdownMenuContent`, and `DropdownMenuSub*` keep the shadcn/Base UI part names available without pushing menu policy down into `fret-ui`.",
        "`DropdownMenuSubContent` keeps the upstream Base UI default submenu side (`inline-end`), so a direction provider is enough to mirror submenu opening direction while submenu chevrons also flip toward logical inline-end.",
        "`DropdownMenu::from_open(open)` stays as the explicit advanced seam when the caller already owns the open model; `new_controllable(cx, open, default_open)` still covers the broader controlled/uncontrolled contract.",
        "`DropdownMenuItem::shortcut(...)`, `DropdownMenuCheckboxItem::shortcut(...)`, and radio-item shortcut helpers are now the preferred copyable API for keyboard hints; `DropdownMenuShortcut` remains the explicit trailing escape hatch.",
        "`DropdownMenuCheckboxItem::from_checked(...)` / `.on_checked_change(...)` and `DropdownMenuRadioGroup::from_value(...)` / `.on_value_change(...)` now cover the upstream snapshot + callback path without forcing per-item `Model<_>` state.",
        "The new `compose()` builder keeps typed entries explicit while removing the root closure cliff, so extracted helpers can stay on the same typed authoring lane as the rest of the app surface.",
        "No extra generic heterogeneous children API is currently warranted: the explicit `DropdownMenuEntry` tree is the Fret-equivalent structured surface for upstream nested menu children, and a generic children lane would add hidden scope/collection contracts without unlocking new behavior.",
    ]);

    let notes = doc_layout::notes_block([
        "Preview now mirrors the upstream shadcn Dropdown Menu docs path first: `Demo`, `Usage`, the example set through `RTL`, then `API Reference`.",
        "Mechanism parity is largely covered already: existing web-vs-fret chrome/placement gates and dropdown diag scripts cover placement, dismissal, focus restore, submenu routing, and safe-corridor behavior.",
        "The lead `Demo` preview now keeps the official `dropdown-menu-demo.tsx` row order more closely, including the `Keyboard shortcuts` action and the ungrouped `GitHub` / `Support` / `API` rows after the second separator.",
        "The checkable examples now demonstrate snapshot + callback authoring, so simple menus do not need one `Model<bool>` per checkbox row.",
        "The remaining docs examples now stay closer to the upstream base docs too: `Checkboxes` keeps the `w-40` panel, `Checkboxes Icons` uses the notification-preferences example, `Radio Group` keeps `w-32`, `Avatar` reuses the shared demo image/fallback pipeline, and `RTL` keeps the richer submenu + checkbox + radio + destructive logout shape while documenting that nested submenus still default to logical `inline-end`.",
        "The `Parts` section is intentionally outside the upstream docs path: treat it as the advanced adapter surface for already-landed or closure-driven seams, while `Usage` now shows the default typed `compose()` root.",
        "The explicit entry tree remains intentional, so the page records why we are not widening this family into a generic heterogeneous children API.",
        "Examples are snippet-backed, so preview and code stay in sync.",
        "Keep `ui-gallery-dropdown-menu-*` test IDs stable; multiple diag scripts depend on them.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-dropdown-menu-api-reference")
        .description("Public surface ownership, composition notes, and children API guidance.");
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-dropdown-menu-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official shadcn-style dropdown menu demo with shortcuts and a submenu.")
        .test_id_prefix("ui-gallery-dropdown-menu-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable source-aligned usage closest to the official docs path.")
        .test_id_prefix("ui-gallery-dropdown-menu-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("A minimal dropdown menu with labels, separators, and disabled items.")
        .test_id_prefix("ui-gallery-dropdown-menu-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let submenu = DocSection::build(cx, "Submenu", submenu)
        .description("Use `DropdownMenuSub*` helpers to nest secondary actions.")
        .test_id_prefix("ui-gallery-dropdown-menu-submenu")
        .code_rust_from_file_region(snippets::submenu::SOURCE, "example");
    let shortcuts = DocSection::build(cx, "Shortcuts", shortcuts)
        .description(
            "Prefer `.shortcut(\"...\")` for copyable keyboard hints; keep `DropdownMenuShortcut` for custom trailing content.",
        )
        .test_id_prefix("ui-gallery-dropdown-menu-shortcuts")
        .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example");
    let icons = DocSection::build(cx, "Icons", icons)
        .description("Combine leading icons with labels for quicker scanning.")
        .test_id_prefix("ui-gallery-dropdown-menu-icons")
        .code_rust_from_file_region(snippets::icons::SOURCE, "example");
    let checkboxes = DocSection::build(cx, "Checkboxes", checkboxes)
        .description("Use checkbox items for toggle-style actions.")
        .test_id_prefix("ui-gallery-dropdown-menu-checkboxes")
        .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example");
    let checkboxes_icons = DocSection::build(cx, "Checkboxes Icons", checkboxes_icons)
        .description(
            "Add icons to notification checkbox items without disturbing the indicator slot.",
        )
        .test_id_prefix("ui-gallery-dropdown-menu-checkboxes-icons")
        .code_rust_from_file_region(snippets::checkboxes_icons::SOURCE, "example");
    let radio_group = DocSection::build(cx, "Radio Group", radio_group)
        .description("Use radio items for mutually exclusive choices in the upstream `Panel Position` example.")
        .test_id_prefix("ui-gallery-dropdown-menu-radio-group")
        .code_rust_from_file_region(snippets::radio_group::SOURCE, "example");
    let radio_icons = DocSection::build(cx, "Radio Icons", radio_icons)
        .description("Show radio options with icons while preserving row alignment.")
        .test_id_prefix("ui-gallery-dropdown-menu-radio-icons")
        .code_rust_from_file_region(snippets::radio_icons::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .description("Use `variant=Destructive` for irreversible actions.")
        .test_id_prefix("ui-gallery-dropdown-menu-destructive")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let avatar = DocSection::build(cx, "Avatar", avatar)
        .description(
            "An account switcher dropdown triggered by an avatar, mirroring the base docs example.",
        )
        .test_id_prefix("ui-gallery-dropdown-menu-avatar")
        .code_rust_from_file_region(snippets::avatar::SOURCE, "example");
    let complex = DocSection::build(cx, "Complex", complex)
        .description("A richer menu mixing groups, icons, toggles, and nested submenus.")
        .test_id_prefix("ui-gallery-dropdown-menu-complex")
        .code_rust_from_file_region(snippets::complex::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL layout mirrors the richer base docs shape while keeping nested submenus on logical `inline-end` and flipping submenu chevrons automatically.")
        .test_id_prefix("ui-gallery-dropdown-menu-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description(
            "Advanced Trigger/Content adapter surface kept outside the default copyable docs path.",
        )
        .test_id_prefix("ui-gallery-dropdown-menu-parts")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Dropdown Menu examples aligned with the upstream shadcn docs path first, followed by Fret-specific API guidance and advanced parts notes.",
        ),
        vec![
            demo,
            usage,
            basic,
            submenu,
            shortcuts,
            icons,
            checkboxes,
            checkboxes_icons,
            radio_group,
            radio_icons,
            destructive,
            avatar,
            complex,
            rtl,
            api_reference,
            parts,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-dropdown-menu").into_element(cx)]
}
