use super::{helpers, models::ComboboxModels, prelude::*};

pub(super) fn demo(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> AnyElement {
    let demo_combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .width(Px(260.0))
        .placeholder("Pick a fruit")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-demo")
        .items(helpers::base_items())
        .into_element(cx)
        .test_id("ui-gallery-combobox-demo-trigger");
    let demo_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                demo_combo,
                helpers::state_rows(cx, &value, &query, "ui-gallery-combobox-demo"),
            ]
        },
    );
    helpers::section_card(cx, "Demo", demo_content)
}

pub(super) fn custom_items_top(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    let custom_combo =
        shadcn::Combobox::new(models.custom_value.clone(), models.custom_open.clone())
            .a11y_label("Combobox custom items")
            .width(Px(280.0))
            .placeholder("Select framework")
            .query_model(models.custom_query.clone())
            .items([
                shadcn::ComboboxItem::new("next", "Next.js (React)"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js (Vue)"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit (Svelte)"),
                shadcn::ComboboxItem::new("astro", "Astro (Hybrid)"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-combobox-custom-items-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                custom_combo,
                shadcn::typography::muted(
                    cx,
                    "Fret currently uses string value/label pairs; object-item mapping (`itemToStringValue`) is approximated by richer labels.",
                ),
                helpers::state_rows(
                    cx,
                    &models.custom_value,
                    &models.custom_query,
                    "ui-gallery-combobox-custom-items",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Custom Items", content)
}

pub(super) fn multiple_selection(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let content = shadcn::typography::muted(
        cx,
        "Upstream supports chips + multiple values. Current Fret `Combobox` API is single-select; keep this as an explicit parity gap marker.",
    );
    helpers::section_card(cx, "Multiple Selection", content)
}

pub(super) fn basic(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let basic_combo = shadcn::Combobox::new(models.basic_value.clone(), models.basic_open.clone())
        .a11y_label("Combobox basic")
        .width(Px(260.0))
        .placeholder("Select a framework")
        .query_model(models.basic_query.clone())
        .test_id_prefix("ui-gallery-combobox-basic")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-combobox-basic-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                basic_combo,
                helpers::state_rows(
                    cx,
                    &models.basic_value,
                    &models.basic_query,
                    "ui-gallery-combobox-basic",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Basic", content)
}

pub(super) fn multiple(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let content = shadcn::typography::muted(
        cx,
        "`multiple` + chips behavior is not exposed in current Fret `Combobox`; tracked as a follow-up API expansion.",
    );
    helpers::section_card(cx, "Multiple", content)
}

pub(super) fn clear_button(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let content = shadcn::typography::muted(
        cx,
        "Upstream has `showClear`. Current Fret API can be cleared by external state reset, but does not provide built-in clear trigger yet.",
    );
    helpers::section_card(cx, "Clear Button", content)
}

pub(super) fn groups(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let groups_combo =
        shadcn::Combobox::new(models.groups_value.clone(), models.groups_open.clone())
            .a11y_label("Combobox groups")
            .width(Px(300.0))
            .placeholder("Filter commands")
            .query_model(models.groups_query.clone())
            .items([
                shadcn::ComboboxItem::new("framework-next", "Frameworks / Next.js"),
                shadcn::ComboboxItem::new("framework-nuxt", "Frameworks / Nuxt.js"),
                shadcn::ComboboxItem::new("language-rust", "Languages / Rust"),
                shadcn::ComboboxItem::new("language-typescript", "Languages / TypeScript"),
                shadcn::ComboboxItem::new("tool-cargo", "Tools / Cargo"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-combobox-groups-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                groups_combo,
                shadcn::typography::muted(
                    cx,
                    "Grouped rows are approximated with prefix labels until dedicated group/separator APIs are introduced.",
                ),
                helpers::state_rows(
                    cx,
                    &models.groups_value,
                    &models.groups_query,
                    "ui-gallery-combobox-groups",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Groups", content)
}

pub(super) fn custom_items_example(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let content = shadcn::typography::muted(
        cx,
        "Render-rich custom item surfaces are currently approximated at label level in this gallery.",
    );
    helpers::section_card(cx, "Custom Items", content)
}

pub(super) fn invalid(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
    destructive: fret_core::Color,
) -> AnyElement {
    let invalid_combo =
        shadcn::Combobox::new(models.invalid_value.clone(), models.invalid_open.clone())
            .a11y_label("Combobox invalid")
            .width(Px(260.0))
            .placeholder("Select required option")
            .query_model(models.invalid_query.clone())
            .items(helpers::base_items())
            .refine_style(
                ChromeRefinement::default()
                    .border_1()
                    .border_color(ColorRef::Color(destructive)),
            )
            .into_element(cx)
            .test_id("ui-gallery-combobox-invalid-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                invalid_combo,
                shadcn::typography::muted(
                    cx,
                    "Invalid visual is currently approximated via destructive border style on trigger.",
                ),
                helpers::state_rows(
                    cx,
                    &models.invalid_value,
                    &models.invalid_query,
                    "ui-gallery-combobox-invalid",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Invalid", content)
}

pub(super) fn disabled(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let disabled_combo =
        shadcn::Combobox::new(models.disabled_value.clone(), models.disabled_open.clone())
            .a11y_label("Combobox disabled")
            .width(Px(260.0))
            .placeholder("Disabled")
            .query_model(models.disabled_query.clone())
            .items(helpers::base_items())
            .disabled(true)
            .into_element(cx)
            .test_id("ui-gallery-combobox-disabled-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                disabled_combo,
                helpers::state_rows(
                    cx,
                    &models.disabled_value,
                    &models.disabled_query,
                    "ui-gallery-combobox-disabled",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Disabled", content)
}

pub(super) fn auto_highlight(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    let auto_highlight_combo = shadcn::Combobox::new(
        models.input_group_value.clone(),
        models.input_group_open.clone(),
    )
    .a11y_label("Combobox auto highlight")
    .width(Px(260.0))
    .placeholder("Type to filter")
    .query_model(models.input_group_query.clone())
    .items(helpers::base_items())
    .into_element(cx)
    .test_id("ui-gallery-combobox-auto-highlight-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                auto_highlight_combo,
                shadcn::typography::muted(
                    cx,
                    "Current behavior follows command palette defaults; explicit `autoHighlight` knob is not yet surfaced.",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Auto Highlight", content)
}

pub(super) fn popup(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let content = shadcn::typography::muted(
        cx,
        "Trigger-as-button popup recipe is not yet exposed as a dedicated API in Fret Combobox.",
    );
    helpers::section_card(cx, "Popup", content)
}

pub(super) fn input_group(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let input_group_combo = shadcn::Combobox::new(
        models.input_group_value.clone(),
        models.input_group_open.clone(),
    )
    .a11y_label("Combobox input group")
    .width(Px(220.0))
    .placeholder("Search command")
    .query_model(models.input_group_query.clone())
    .items([
        shadcn::ComboboxItem::new("new-file", "New File"),
        shadcn::ComboboxItem::new("open-file", "Open File"),
        shadcn::ComboboxItem::new("save-all", "Save All"),
    ])
    .into_element(cx)
    .test_id("ui-gallery-combobox-input-group-trigger");
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(360.0))),
        move |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            {
                                let props = cx.with_theme(|theme| {
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Sm)
                                            .px(Space::N2)
                                            .py(Space::N1),
                                        LayoutRefinement::default(),
                                    )
                                });
                                cx.container(props, |cx| vec![shadcn::typography::muted(cx, "Cmd")])
                            },
                            input_group_combo,
                        ]
                    },
                ),
                helpers::state_rows(
                    cx,
                    &models.input_group_value,
                    &models.input_group_query,
                    "ui-gallery-combobox-input-group",
                ),
            ]
        },
    );
    helpers::section_card(cx, "Input Group", content)
}

pub(super) fn rtl(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let rtl_combo = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Combobox::new(models.rtl_value.clone(), models.rtl_open.clone())
                .a11y_label("Combobox RTL")
                .width(Px(260.0))
                .placeholder("???? ???? ?????")
                .query_model(models.rtl_query.clone())
                .items([
                    shadcn::ComboboxItem::new("next", "Next.js"),
                    shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                    shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                ])
                .into_element(cx)
                .test_id("ui-gallery-combobox-rtl-trigger")
        },
    );
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                rtl_combo,
                helpers::state_rows(
                    cx,
                    &models.rtl_value,
                    &models.rtl_query,
                    "ui-gallery-combobox-rtl",
                ),
            ]
        },
    );
    helpers::section_card(cx, "RTL", content)
}

pub(super) fn component_panel(
    cx: &mut ElementContext<'_, App>,
    demo: AnyElement,
    custom_items_top: AnyElement,
    multiple_selection: AnyElement,
    basic: AnyElement,
    multiple: AnyElement,
    clear_button: AnyElement,
    groups: AnyElement,
    custom_items_example: AnyElement,
    invalid: AnyElement,
    disabled: AnyElement,
    auto_highlight: AnyElement,
    popup: AnyElement,
    input_group: AnyElement,
    rtl: AnyElement,
) -> AnyElement {
    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Combobox docs flow; unsupported recipes are kept as explicit gap markers.",
    );
    let stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| {
            vec![
                preview_hint,
                demo,
                custom_items_top,
                multiple_selection,
                basic,
                multiple,
                clear_button,
                groups,
                custom_items_example,
                invalid,
                disabled,
                auto_highlight,
                popup,
                input_group,
                rtl,
            ]
        },
    );
    helpers::shell(cx, stack).test_id("ui-gallery-combobox-component")
}
