use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::context_menu as snippets;

pub(super) fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let basic = snippets::basic::render(cx);
    let submenu = snippets::submenu::render(cx);
    let shortcuts = snippets::shortcuts::render(cx);
    let groups = snippets::groups::render(cx);
    let icons = snippets::icons::render(cx);
    let checkboxes = snippets::checkboxes::render(cx);
    let radio = snippets::radio::render(cx);
    let destructive = snippets::destructive::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows the upstream shadcn Context Menu docs (v4 Base UI).",
            "Examples are snippet-backed: Preview ≡ Code (single source of truth).",
            "Keep `ui-gallery-context-menu-*` test IDs stable; multiple diag scripts depend on them.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Context Menu examples, aligned with upstream shadcn docs (Base UI variant): \
             Basic, Submenu, Shortcuts, Groups, Icons, Checkboxes, Radio, Destructive, RTL.",
        ),
        vec![
            DocSection::new("Basic", basic)
                .description("Right click on the trigger surface to open the menu.")
                .test_id_prefix("ui-gallery-context-menu-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
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
            DocSection::new("RTL", rtl)
                .description("RTL layout keeps spacing and submenu direction parity-auditable.")
                .test_id_prefix("ui-gallery-context-menu-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-page-context-menu")]
}
