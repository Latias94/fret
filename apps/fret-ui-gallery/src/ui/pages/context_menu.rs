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
        "Default copyable root path is `ContextMenu::uncontrolled(cx).build_parts(...)`, while `ContextMenu::from_open(...)` and `new_controllable(...)` stay as the explicit managed-open seams.",
        "Context Menu already exposes shadcn-style parts plus `ContextMenuSub*` helpers, so the remaining gap is mostly docs/page parity rather than missing menu infrastructure.",
        "A separate generic children API is not required here yet: typed menu entries stay explicit, while `build_parts(...)` keeps docs-style authoring ergonomic.",
        "Examples are snippet-backed: preview and code stay in sync.",
        "Keep `ui-gallery-context-menu-*` test IDs stable; multiple diag scripts depend on them.",
    ]);
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-context-menu-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Context Menu examples aligned with upstream shadcn docs: Demo, Basic, Usage, Submenu, Shortcuts, Groups, Icons, Checkboxes, Radio, Destructive, Sides, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Official-style combined example with submenu, toggles, and radio items.",
                )
                .test_id_prefix("ui-gallery-context-menu-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Right click on the trigger surface to open the menu.")
                .test_id_prefix("ui-gallery-context-menu-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Context Menu.")
                .test_id_prefix("ui-gallery-context-menu-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Submenu", submenu)
                .description("Nested submenu entries for grouped actions.")
                .test_id_prefix("ui-gallery-context-menu-submenu")
                .code_rust_from_file_region(snippets::submenu::SOURCE, "example"),
            DocSection::new("Shortcuts", shortcuts)
                .description("Use `ContextMenuShortcut` to show keyboard hints.")
                .test_id_prefix("ui-gallery-context-menu-shortcuts")
                .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example"),
            DocSection::new("Groups", groups)
                .description("Group related actions and separate them with dividers.")
                .test_id_prefix("ui-gallery-context-menu-groups")
                .code_rust_from_file_region(snippets::groups::SOURCE, "example"),
            DocSection::new("Icons", icons)
                .description("Combine icons with labels for quick scanning.")
                .test_id_prefix("ui-gallery-context-menu-icons")
                .code_rust_from_file_region(snippets::icons::SOURCE, "example"),
            DocSection::new("Checkboxes", checkboxes)
                .description("Use checkbox items for toggles.")
                .test_id_prefix("ui-gallery-context-menu-checkboxes")
                .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example"),
            DocSection::new("Radio", radio)
                .description("Use a radio group for exclusive choices.")
                .test_id_prefix("ui-gallery-context-menu-radio")
                .code_rust_from_file_region(snippets::radio::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .description("Use `variant=Destructive` for irreversible actions.")
                .test_id_prefix("ui-gallery-context-menu-destructive")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("Sides", sides)
                .description("Control content placement with `side` props.")
                .test_id_prefix("ui-gallery-context-menu-sides")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("RTL layout keeps spacing and submenu direction parity-auditable.")
                .test_id_prefix("ui-gallery-context-menu-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-context-menu")]
}
