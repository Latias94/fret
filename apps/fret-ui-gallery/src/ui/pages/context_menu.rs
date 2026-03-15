use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::context_menu as snippets;

pub(super) fn preview_context_menu(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let usage = snippets::usage::render(cx);
    let submenu = snippets::submenu::render(cx);
    let shortcuts = snippets::shortcuts::render(cx);
    let groups = snippets::groups::render(cx);
    let icons = snippets::icons::render(cx);
    let checkboxes = snippets::checkboxes::render(cx);
    let radio = snippets::radio::render(cx);
    let destructive = snippets::destructive::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "Preview follows the upstream shadcn Context Menu docs (v4 Base UI).",
        "Default copyable root path is now `ContextMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)`, while `build_parts(...)` / `into_element_parts(...)` remain the lower-level adapter seams.",
        "Those lower-level adapter seams are still advanced API, not the default copyable teaching lane.",
        "Context Menu already exposes shadcn-style parts plus `ContextMenuSub*` helpers, and the new typed root builder removes the remaining root-authoring cliff without changing the underlying menu infrastructure.",
        "Typed menu entries remain explicit; `compose()` just moves the final landing seam out of extracted helpers and back to the true root call site.",
        "Examples are snippet-backed: preview and code stay in sync.",
        "Keep `ui-gallery-context-menu-*` test IDs stable; multiple diag scripts depend on them.",
    ]);
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-context-menu-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Official-style combined example with submenu, toggles, and radio items.")
        .test_id_prefix("ui-gallery-context-menu-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Right click on the trigger surface to open the menu.")
        .test_id_prefix("ui-gallery-context-menu-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable shadcn-style composition reference for Context Menu.")
        .test_id_prefix("ui-gallery-context-menu-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let submenu = DocSection::build(cx, "Submenu", submenu)
        .description("Nested submenu entries for grouped actions.")
        .test_id_prefix("ui-gallery-context-menu-submenu")
        .code_rust_from_file_region(snippets::submenu::SOURCE, "example");
    let shortcuts = DocSection::build(cx, "Shortcuts", shortcuts)
        .description("Use `ContextMenuShortcut` to show keyboard hints.")
        .test_id_prefix("ui-gallery-context-menu-shortcuts")
        .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Group related actions and separate them with dividers.")
        .test_id_prefix("ui-gallery-context-menu-groups")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example");
    let icons = DocSection::build(cx, "Icons", icons)
        .description("Combine icons with labels for quick scanning.")
        .test_id_prefix("ui-gallery-context-menu-icons")
        .code_rust_from_file_region(snippets::icons::SOURCE, "example");
    let checkboxes = DocSection::build(cx, "Checkboxes", checkboxes)
        .description("Use checkbox items for toggles.")
        .test_id_prefix("ui-gallery-context-menu-checkboxes")
        .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example");
    let radio = DocSection::build(cx, "Radio", radio)
        .description("Use a radio group for exclusive choices.")
        .test_id_prefix("ui-gallery-context-menu-radio")
        .code_rust_from_file_region(snippets::radio::SOURCE, "example");
    let destructive = DocSection::build(cx, "Destructive", destructive)
        .description("Use `variant=Destructive` for irreversible actions.")
        .test_id_prefix("ui-gallery-context-menu-destructive")
        .code_rust_from_file_region(snippets::destructive::SOURCE, "example");
    let sides = DocSection::build(cx, "Sides", sides)
        .description("Control content placement with `side` props.")
        .test_id_prefix("ui-gallery-context-menu-sides")
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("RTL layout keeps spacing and submenu direction parity-auditable.")
        .test_id_prefix("ui-gallery-context-menu-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Context Menu examples aligned with upstream shadcn docs: Demo, Basic, Usage, Submenu, Shortcuts, Groups, Icons, Checkboxes, Radio, Destructive, Sides, RTL.",
        ),
        vec![
            demo,
            basic,
            usage,
            submenu,
            shortcuts,
            groups,
            icons,
            checkboxes,
            radio,
            destructive,
            sides,
            rtl,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-context-menu").into_element(cx)]
}
