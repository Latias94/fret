use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::command as snippets;

pub(super) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
    _query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let usage_palette = snippets::usage::render(cx, last_action.clone());
    let docs_demo_palette = snippets::docs_demo::render(cx, last_action.clone());
    let basic_dialog = snippets::basic::render(cx, last_action.clone());
    let shortcuts_section = snippets::shortcuts::render(cx, last_action.clone());
    let groups_palette = snippets::groups::render(cx, last_action.clone());
    let scrollable_palette = snippets::scrollable::render(cx, last_action.clone());
    let rtl = snippets::rtl::render(cx, last_action.clone());
    let loading_palette = snippets::loading::render(cx, last_action.clone());

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Use `CommandDialog` for global discovery (Ctrl/Cmd+P), and keep `CommandPalette` embedded for local filtering surfaces.",
            "Attach either `on_select`, `on_select_action`, or `on_select_value_action` for every interactive item; otherwise entries are treated as disabled.",
            "Mirror docs order even when APIs differ so parity gaps stay explicit and testable.",
            "For long command catalogs, constrain list height via `refine_scroll_layout` to keep dialog geometry stable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Command docs order: Usage, Demo, Basic, Shortcuts, Groups, Scrollable, RTL, Loading.",
        ),
        vec![
            DocSection::new("Usage", usage_palette)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-usage")
                .descriptions([
                    "This mirrors shadcn's docs structure (`Command` + `CommandInput` + `CommandList`) using Fret's `CommandPalette` recipe.",
                    "Use this pattern for inline command menus (as opposed to `CommandDialog`).",
                ])
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Demo", docs_demo_palette)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-docs-demo")
                .descriptions([
                    "This aligns with the shadcn `command-demo` example (icons + disabled item + shortcuts).",
                    "Use `leading_icon(...)` so icons inherit the row foreground (`currentColor`) for hover/active/disabled states.",
                ])
                .code_rust_from_file_region(
                    snippets::docs_demo::SOURCE,
                    "example",
                ),
            DocSection::new("Basic", basic_dialog)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Shortcuts", shortcuts_section)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-shortcuts")
                .code_rust_from_file_region(
                    snippets::shortcuts::SOURCE,
                    "example",
                ),
            DocSection::new("Groups", groups_palette)
                .max_w(Px(760.0))
                .code_rust_from_file_region(
                    snippets::groups::SOURCE,
                    "example",
                ),
            DocSection::new("Scrollable", scrollable_palette)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-scrollable")
                .code_rust_from_file_region(
                    snippets::scrollable::SOURCE,
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Loading", loading_palette)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-loading")
                .descriptions([
                    "cmdk supports a non-selectable loading row inside the list (`Command.Loading`).",
                    "In Fret this maps to `shadcn::CommandLoading` as an extra `CommandEntry`.",
                ])
                .code_rust_from_file_region(
                    snippets::loading::SOURCE,
                    "example",
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-command-component")]
}
