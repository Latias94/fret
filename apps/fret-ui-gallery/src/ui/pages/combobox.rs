mod helpers;
mod models;
mod sections;

mod prelude {
    pub(super) use super::super::super::*;
}

use crate::ui::doc_layout::{self, DocSection};
use prelude::*;

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let models = models::get_or_init(cx);

    let conformance_demo = sections::demo(cx, value, open, query);
    let basic = sections::basic_frameworks(cx, &models);
    let auto_highlight = sections::auto_highlight(cx, &models);
    let clear = sections::clear_button(cx, &models);
    let custom_items_top = sections::custom_items_top(cx, &models);
    let long_list = sections::long_list(cx, &models);
    let groups = sections::groups(cx, &models);
    let groups_with_separator = sections::groups_with_separator(cx, &models);
    let popup = sections::popup_trigger(cx, &models);
    let multiple = sections::multiple_selection(cx, &models);
    let invalid = sections::invalid(cx, &models);
    let disabled = sections::disabled(cx, &models);
    let input_group = sections::input_group(cx, &models);
    let rtl = sections::rtl(cx, &models);

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
                .description("Small deterministic surface for `fretboard diag suite ui-gallery-combobox` scripts.")
                .no_shell(),
            DocSection::new("Basic", basic)
                .description("Upstream shadcn demo: basic framework combobox with search.")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .a11y_label("Combobox basic")
    .width(Px(260.0))
    .placeholder("Select a framework")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("svelte", "SvelteKit"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
        shadcn::ComboboxItem::new("remix", "Remix"),
        shadcn::ComboboxItem::new("astro", "Astro"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Auto Highlight", auto_highlight)
                .description("Base UI opt-in: highlight the first enabled match on open/filter (`autoHighlight`).")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .placeholder("Select a framework")
    .query_model(query)
    .auto_highlight(true)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("svelte", "SvelteKit"),
    ])
    .into_element(cx);"#,
                )
                .no_shell(),
            DocSection::new("Clear Button", clear)
                .description("Enable `show_clear` to show a clear affordance when a value is selected.")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .placeholder("Select a framework")
    .query_model(query)
    .show_clear(true)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("svelte", "SvelteKit"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
        shadcn::ComboboxItem::new("remix", "Remix"),
        shadcn::ComboboxItem::new("astro", "Astro"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Groups", groups)
                .description("Upstream groups items; Fret exposes grouped entries via `ComboboxGroup`.")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .placeholder("Select a timezone")
    .query_model(query)
    .groups([
        shadcn::ComboboxGroup::new(
            "Americas",
            [shadcn::ComboboxItem::new("americas-ny", "(GMT-5) New York")],
        ),
        shadcn::ComboboxGroup::new(
            "Europe",
            [shadcn::ComboboxItem::new("europe-lon", "(GMT+0) London")],
        ),
    ])
    .into_element(cx);"#,
                )
                .no_shell(),
            DocSection::new("Groups + Separator", groups_with_separator)
                .description("Use `.group_separators(true)` to insert separators between groups (shadcn `ComboboxSeparator`).")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .placeholder("Select a timezone")
    .query_model(query)
    .group_separators(true)
    .groups([
        shadcn::ComboboxGroup::new(
            "Americas",
            [shadcn::ComboboxItem::new("americas-ny", "(GMT-5) New York")],
        ),
        shadcn::ComboboxGroup::new(
            "Europe",
            [shadcn::ComboboxItem::new("europe-lon", "(GMT+0) London")],
        ),
    ])
    .into_element(cx);"#,
                )
                .no_shell(),
            DocSection::new("Trigger Button", popup)
                .description("Aligns Base UI combobox \"Popup\" recipe: a button-like trigger with the searchable listbox in the popover content.")
                .no_shell(),
            DocSection::new("Multiple Selection", multiple)
                .description("Upstream multi-select chips recipe: select multiple values and remove them via chips.")
                .code(
                    "rust",
                    r#"let values: Model<Vec<Arc<str>>> = cx.app.models_mut().insert(vec![]);
let open: Model<bool> = cx.app.models_mut().insert(false);
let query: Model<String> = cx.app.models_mut().insert(String::new());

shadcn::ComboboxChips::new(values, open)
    .a11y_label("Combobox multiple selection")
    .width(Px(260.0))
    .placeholder("Select frameworks")
    .query_model(query)
    .test_id_prefix("ui-gallery-combobox-multiple")
    .trigger_test_id("ui-gallery-combobox-multiple-trigger")
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("svelte", "SvelteKit"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Extras: Custom Items", custom_items_top)
                .description("Structured item details (e.g. suffix metadata) without pre-formatting richer labels.")
                .code(
                    "rust",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .placeholder("Select framework")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js").detail("React"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js").detail("Vue"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Extras: Long List", long_list)
                .description(
                    "Supports long-list scroll regression gates (and future virtualization invariants).",
                )
                .code(
                    "rust",
                    r#"let items: Vec<shadcn::ComboboxItem> = (0..250)
    .map(|i| shadcn::ComboboxItem::new(format!("{i:03}"), format!("Item {i:03}")))
    .collect();

shadcn::Combobox::new(value, open)
    .a11y_label("Combobox long list")
    .width(Px(320.0))
    .placeholder("Pick an item")
    .query_model(query)
    .test_id_prefix("ui-gallery-combobox-long-list")
    .items(items)
    .into_element(cx);"#,
                ),
            DocSection::new("Extras: Invalid", invalid)
                .description("Invalid visual uses `aria_invalid(true)` on the combobox trigger.")
            .code(
                "rust",
                r#"shadcn::Combobox::new(value, open)
    .a11y_label("Combobox invalid")
    .width(Px(260.0))
    .placeholder("Select required option")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("apple", "Apple"),
        shadcn::ComboboxItem::new("banana", "Banana"),
    ])
    .aria_invalid(true)
    .into_element(cx);"#,
            ),
            DocSection::new("Extras: Disabled", disabled)
                .description("Disabled state should block open/selection and use muted styling.")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .a11y_label("Combobox disabled")
    .placeholder("Disabled")
    .query_model(query)
    .items([shadcn::ComboboxItem::new("apple", "Apple")])
    .disabled(true)
    .into_element(cx);"#,
                ),
            DocSection::new("Extras: Input Group", input_group)
                .description("Inline keybinding + input grouping example.")
                .code(
                    "rust",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .a11y_label("Combobox input group")
    .width(Px(220.0))
    .placeholder("Search command")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("new-file", "New File"),
        shadcn::ComboboxItem::new("open-file", "Open File"),
    ])
    .into_element(cx);

stack::hstack(
    cx,
    stack::HStackProps::default().gap(Space::N2).items_center(),
    |cx| vec![shadcn::typography::muted(cx, "Cmd"), combo],
);"#,
                ),
            DocSection::new("Extras: RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                     "rust",
                     r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
     shadcn::Combobox::new(value, open)
         .placeholder("ابحث عن أمر")
         .into_element(cx)
})"#,
                 ),
             DocSection::new("Notes", notes)
                 .test_id_prefix("ui-gallery-combobox-notes")
                 .description("Guidelines and parity notes for combobox recipes."),
        ],
    );

    vec![body]
}
