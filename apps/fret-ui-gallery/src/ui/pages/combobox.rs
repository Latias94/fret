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
    let destructive = cx.with_theme(|theme| theme.color_token("destructive"));

    let demo = sections::demo(cx, value, open, query);
    let custom_items_top = sections::custom_items_top(cx, &models);
    let multiple_selection = sections::multiple_selection(cx);
    let basic = sections::basic(cx, &models);
    let long_list = sections::long_list(cx, &models);
    let multiple = sections::multiple(cx);
    let clear_button = sections::clear_button(cx);
    let groups = sections::groups(cx, &models);
    let custom_items_example = sections::custom_items_example(cx);
    let invalid = sections::invalid(cx, &models, destructive);
    let disabled = sections::disabled(cx, &models);
    let auto_highlight = sections::auto_highlight(cx, &models);
    let popup = sections::popup(cx);
    let input_group = sections::input_group(cx, &models);
    let rtl = sections::rtl(cx, &models);

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Current Fret `Combobox` focuses on single-select + query filtering; several Base UI recipes are tracked as explicit gaps here.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep unsupported sections visible (multiple/clear/popup) to make parity progress auditable instead of implicit.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For invalid visuals today, apply style overrides on trigger and pair with field-level error copy.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Combobox docs flow; unsupported recipes are kept as explicit gap markers.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic single-select combobox with query filtering.")
                .code(
                    "rust",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .placeholder("Pick a fruit")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("apple", "Apple"),
        shadcn::ComboboxItem::new("banana", "Banana"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Custom Items", custom_items_top)
                .description(
                    "Fret currently uses string value/label pairs; object-item mapping is approximated by richer labels.",
                )
                .code(
                    "rust",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .placeholder("Select framework")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js (React)"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js (Vue)"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Multiple Selection", multiple_selection)
                .description("Parity gap marker: upstream supports chips + multiple values.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Basic", basic)
                .description("Small list of items with stable test IDs for diag scripts.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Long List", long_list)
                .description(
                    "Supports long-list scroll regression gates (and future virtualization invariants).",
                )
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Multiple", multiple)
                .description("Parity gap marker: `multiple` + chips behavior is not exposed yet.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Clear Button", clear_button)
                .description("Parity gap marker: upstream `showClear` is not exposed yet.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Groups", groups)
                .description(
                    "Grouped rows are approximated with prefix labels until group/separator APIs exist.",
                )
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Custom Items (Rich)", custom_items_example).description(
                "Parity gap marker: render-rich item surfaces are currently approximated at label level.",
            )
            .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Invalid", invalid).description(
                "Invalid visual is currently approximated via destructive border style on trigger.",
            )
            .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Disabled", disabled)
                .description("Disabled state should block open/selection and use muted styling.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Auto Highlight", auto_highlight).description(
                "Current behavior follows command palette defaults; explicit `autoHighlight` knob is not surfaced yet.",
            )
            .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Popup", popup).description(
                "Parity gap marker: trigger-as-button popup recipe is not exposed yet.",
            )
            .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("Input Group", input_group)
                .description("Inline keybinding + input grouping example.")
                .code("rust", doc_layout::TODO_RUST_CODE),
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Combobox::new(value, open)
        .placeholder("????")
        .into_element(cx)
})"#,
                ),
            DocSection::new("Notes", notes)
                .description("Guidelines and parity notes for combobox recipes."),
        ],
    );

    vec![body]
}
