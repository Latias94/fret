use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::combobox as snippets;

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let conformance_demo =
        snippets::conformance_demo::render(cx, value.clone(), open.clone(), query.clone());
    let basic = snippets::basic::render(cx);
    let auto_highlight = snippets::auto_highlight::render(cx);
    let clear = snippets::clear_button::render(cx);
    let groups = snippets::groups::render(cx);
    let groups_with_separator = snippets::groups_with_separator::render(cx);
    let trigger_button = snippets::trigger_button::render(cx);
    let multiple = snippets::multiple_selection::render(cx);
    let custom_items = snippets::custom_items::render(cx);
    let long_list = snippets::long_list::render(cx);
    let invalid = snippets::invalid::render(cx);
    let disabled = snippets::disabled::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Combobox is a Popover + Command recipe. Keep shadcn demo order stable so parity gaps are explicit and testable.",
            "Multi-select chips is a recipe-level surface (`ComboboxChips`) built on top of Command + Popover primitives.",
            "For invalid visuals today, apply style overrides on trigger and pair with field-level error copy.",
            "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn ComboboxDemo order, with a small conformance-first section at the top to keep diag scripts stable.",
        ),
        vec![
            DocSection::new("Conformance Demo", conformance_demo)
                .description(
                    "Small deterministic surface for `fretboard diag suite ui-gallery-combobox` scripts.",
                )
                .no_shell()
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/conformance_demo.rs"),
                    "example",
                ),
            DocSection::new("Basic", basic)
                .description("Upstream shadcn demo: basic framework combobox with search.")
                .code_rust_from_file_region(include_str!("../snippets/combobox/basic.rs"), "example"),
            DocSection::new("Auto Highlight", auto_highlight)
                .description(
                    "Base UI opt-in: highlight the first enabled match on open/filter (`autoHighlight`).",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/auto_highlight.rs"),
                    "example",
                )
                .no_shell(),
            DocSection::new("Clear Button", clear)
                .description(
                    "Enable `show_clear` to show a clear affordance when a value is selected.",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/clear_button.rs"),
                    "example",
                ),
            DocSection::new("Groups", groups)
                .description(
                    "Upstream groups items; Fret exposes grouped entries via `ComboboxGroup`.",
                )
                .code_rust_from_file_region(include_str!("../snippets/combobox/groups.rs"), "example")
                .no_shell(),
            DocSection::new("Groups + Separator", groups_with_separator)
                .description(
                    "Use `.group_separators(true)` to insert separators between groups (shadcn `ComboboxSeparator`).",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/groups_with_separator.rs"),
                    "example",
                )
                .no_shell(),
            DocSection::new("Trigger Button", trigger_button)
                .description("Aligns Base UI combobox \"Popup\" recipe: a button-like trigger with the searchable listbox in the popover content.")
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/trigger_button.rs"),
                    "example",
                )
                .no_shell(),
            DocSection::new("Multiple Selection", multiple)
                .description("Upstream multi-select chips recipe: select multiple values and remove them via chips.")
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/multiple_selection.rs"),
                    "example",
                ),
            DocSection::new("Extras: Custom Items", custom_items)
                .description(
                    "Structured item details (e.g. suffix metadata) without pre-formatting richer labels.",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/custom_items.rs"),
                    "example",
                ),
            DocSection::new("Extras: Long List", long_list)
                .description(
                    "Supports long-list scroll regression gates (and future virtualization invariants).",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/long_list.rs"),
                    "example",
                ),
            DocSection::new("Extras: Invalid", invalid)
                .description("Invalid visual uses `aria_invalid(true)` on the combobox trigger.")
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/invalid.rs"),
                    "example",
                ),
            DocSection::new("Extras: Disabled", disabled)
                .description("Disabled state should block open/selection and use muted styling.")
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/disabled.rs"),
                    "example",
                ),
            DocSection::new("Extras: Input Group", input_group)
                .description("Inline keybinding + input grouping example.")
                .code_rust_from_file_region(
                    include_str!("../snippets/combobox/input_group.rs"),
                    "example",
                ),
            DocSection::new("Extras: RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code_rust_from_file_region(include_str!("../snippets/combobox/rtl.rs"), "example"),
            DocSection::new("Notes", notes)
                .test_id_prefix("ui-gallery-combobox-notes")
                .description("Guidelines and parity notes for combobox recipes."),
        ],
    );

    vec![body]
}
