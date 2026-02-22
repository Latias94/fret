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

    let demo = sections::demo(cx, value, open, query);
    let clear = sections::clear_button(cx, &models);
    let custom_items_top = sections::custom_items_top(cx, &models);
    let long_list = sections::long_list(cx, &models);
    let groups = sections::groups(cx, &models);
    let invalid = sections::invalid(cx, &models);
    let disabled = sections::disabled(cx, &models);
    let input_group = sections::input_group(cx, &models);
    let rtl = sections::rtl(cx, &models);

    let (trigger_title, trigger) = doc_layout::gap_card(
        cx,
        "Trigger Button",
        "Upstream shows a trigger-as-button recipe (`ComboboxTrigger` + `ComboboxValue`). Current Fret `Combobox` is a Popover+Command recipe with an integrated trigger; trigger-as-child composition is not exposed yet.",
        "ui-gallery-combobox-trigger-gap",
    );

    let (multiple_title, multiple) = doc_layout::gap_card(
        cx,
        "Multiple Selection",
        "Upstream supports `multiple` + chips (`ComboboxChips`) with `autoHighlight`. Current Fret `Combobox` is single-select; multi-select + chips is tracked as an API expansion.",
        "ui-gallery-combobox-multiple-gap",
    );

    let notes = doc_layout::notes(
        cx,
        [
            "Combobox is a Popover + Command recipe. Keep shadcn demo order stable so parity gaps are explicit and testable.",
            "Current Fret `Combobox` focuses on single-select + query filtering; multi-select and trigger composition are tracked as explicit gaps.",
            "For invalid visuals today, apply style overrides on trigger and pair with field-level error copy.",
            "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn ComboboxDemo order: Basic, Clear Button, Groups, Trigger Button, Multiple Selection. Extras are Fret-specific gates.",
        ),
        vec![
            DocSection::new("Basic", demo)
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
                .description("Upstream groups items; current Fret demo approximates groups via prefixed labels.")
                .code(
                    "rust",
                    r#"shadcn::Combobox::new(value, open)
    .placeholder("Select a timezone")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("americas-ny", "Americas / (GMT-5) New York"),
        shadcn::ComboboxItem::new("europe-lon", "Europe / (GMT+0) London"),
        shadcn::ComboboxItem::new("asia-tokyo", "Asia/Pacific / (GMT+9) Tokyo"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new(trigger_title, trigger)
                .description("Upstream trigger-as-button recipe; kept as an explicit parity gap marker.")
                .code(
                    "rust",
                    r#"// Not yet implemented: trigger-as-child / trigger-as-button composition is not exposed yet.
// Track this as a dedicated API surface before mirroring Base UI's `ComboboxTrigger`/`ComboboxValue`."#,
                ),
            DocSection::new(multiple_title, multiple)
                .description("Upstream multi-select chips recipe; kept as an explicit parity gap marker.")
                .code(
                    "rust",
                    r#"// Not yet implemented: upstream supports `multiple` + chips (`ComboboxChips`) + `autoHighlight`.
// Current Fret `Combobox` is single-select."#,
                ),
            DocSection::new("Extras: Custom Items", custom_items_top)
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
