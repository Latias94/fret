use super::{helpers, models::ComboboxModels, prelude::*};

pub(super) fn demo(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> AnyElement {
    shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .width(Px(260.0))
        .placeholder("Select a fruit")
        .query_model(query.clone())
        .test_id_prefix("ui-gallery-combobox-demo")
        .trigger_test_id("ui-gallery-combobox-demo-trigger")
        .items(helpers::base_items())
        .into_element(cx)
}

pub(super) fn basic_frameworks(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    shadcn::Combobox::new(models.basic_value.clone(), models.basic_open.clone())
        .a11y_label("Combobox basic")
        .width(Px(260.0))
        .placeholder("Select a framework")
        .query_model(models.basic_query.clone())
        .test_id_prefix("ui-gallery-combobox-basic")
        .trigger_test_id("ui-gallery-combobox-basic-trigger")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element(cx)
}

pub(super) fn clear_button(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    let combo = shadcn::Combobox::new(models.clear_value.clone(), models.clear_open.clone())
        .a11y_label("Combobox clear")
        .width(Px(260.0))
        .placeholder("Select a framework")
        .query_model(models.clear_query.clone())
        .show_clear(true)
        .test_id_prefix("ui-gallery-combobox-clear")
        .trigger_test_id("ui-gallery-combobox-clear-trigger")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                combo,
                helpers::state_rows(
                    cx,
                    &models.clear_value,
                    &models.clear_query,
                    "ui-gallery-combobox-clear",
                ),
            ]
        },
    )
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
            .test_id_prefix("ui-gallery-combobox-custom-items")
            .trigger_test_id("ui-gallery-combobox-custom-items-trigger")
            .items([
                shadcn::ComboboxItem::new("next", "Next.js (React)"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js (Vue)"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit (Svelte)"),
                shadcn::ComboboxItem::new("astro", "Astro (Hybrid)"),
            ])
            .into_element(cx);
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                custom_combo,
                helpers::state_rows(
                    cx,
                    &models.custom_value,
                    &models.custom_query,
                    "ui-gallery-combobox-custom-items",
                ),
            ]
        },
    );
    content
}

pub(super) fn long_list(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let items: Vec<shadcn::ComboboxItem> = (0..250)
        .map(|i| {
            let value = format!("{i:03}");
            let label = format!("Item {i:03}");
            shadcn::ComboboxItem::new(value, label)
        })
        .collect();

    let combo = shadcn::Combobox::new(
        models.long_list_value.clone(),
        models.long_list_open.clone(),
    )
    .a11y_label("Combobox long list")
    .width(Px(320.0))
    .placeholder("Pick an item")
    .query_model(models.long_list_query.clone())
    .test_id_prefix("ui-gallery-combobox-long-list")
    .trigger_test_id("ui-gallery-combobox-long-list-trigger")
    .items(items)
    .into_element(cx);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(360.0))),
        move |cx| {
            vec![
                combo,
                helpers::state_rows(
                    cx,
                    &models.long_list_value,
                    &models.long_list_query,
                    "ui-gallery-combobox-long-list",
                ),
            ]
        },
    );
    content
}

pub(super) fn groups(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let groups_combo =
        shadcn::Combobox::new(models.groups_value.clone(), models.groups_open.clone())
            .a11y_label("Combobox groups")
            .width(Px(300.0))
            .placeholder("Select a timezone")
            .query_model(models.groups_query.clone())
            .test_id_prefix("ui-gallery-combobox-groups")
            .trigger_test_id("ui-gallery-combobox-groups-trigger")
            .groups([
                shadcn::ComboboxGroup::new(
                    "Americas",
                    [
                        shadcn::ComboboxItem::new("americas-ny", "(GMT-5) New York"),
                        shadcn::ComboboxItem::new("americas-la", "(GMT-8) Los Angeles"),
                        shadcn::ComboboxItem::new("americas-chi", "(GMT-6) Chicago"),
                    ],
                ),
                shadcn::ComboboxGroup::new(
                    "Europe",
                    [
                        shadcn::ComboboxItem::new("europe-lon", "(GMT+0) London"),
                        shadcn::ComboboxItem::new("europe-paris", "(GMT+1) Paris"),
                        shadcn::ComboboxItem::new("europe-berlin", "(GMT+1) Berlin"),
                    ],
                ),
                shadcn::ComboboxGroup::new(
                    "Asia/Pacific",
                    [
                        shadcn::ComboboxItem::new("asia-tokyo", "(GMT+9) Tokyo"),
                        shadcn::ComboboxItem::new("asia-shanghai", "(GMT+8) Shanghai"),
                        shadcn::ComboboxItem::new("asia-singapore", "(GMT+8) Singapore"),
                    ],
                ),
            ])
            .into_element(cx);
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                groups_combo,
                helpers::state_rows(
                    cx,
                    &models.groups_value,
                    &models.groups_query,
                    "ui-gallery-combobox-groups",
                ),
            ]
        },
    );
    content
}

pub(super) fn popup_trigger(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    let combo = shadcn::Combobox::new(models.popup_value.clone(), models.popup_open.clone())
        .a11y_label("Combobox popup trigger")
        .width(Px(256.0))
        .placeholder("Select a framework")
        .query_model(models.popup_query.clone())
        .trigger_variant(shadcn::ComboboxTriggerVariant::Button)
        .trigger_test_id("ui-gallery-combobox-popup-trigger")
        .test_id_prefix("ui-gallery-combobox-popup")
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |cx| {
            vec![
                combo,
                helpers::state_rows(
                    cx,
                    &models.popup_value,
                    &models.popup_query,
                    "ui-gallery-combobox-popup",
                ),
            ]
        },
    )
}

pub(super) fn multiple_selection(
    cx: &mut ElementContext<'_, App>,
    models: &ComboboxModels,
) -> AnyElement {
    let combo =
        shadcn::ComboboxChips::new(models.multiple_values.clone(), models.multiple_open.clone())
            .a11y_label("Combobox multiple selection")
            .width(Px(260.0))
            .placeholder("Select frameworks")
            .query_model(models.multiple_query.clone())
            .trigger_test_id("ui-gallery-combobox-multiple-trigger")
            .test_id_prefix("ui-gallery-combobox-multiple")
            .items([
                shadcn::ComboboxItem::new("next", "Next.js"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                shadcn::ComboboxItem::new("remix", "Remix"),
                shadcn::ComboboxItem::new("astro", "Astro"),
            ])
            .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        move |_cx| vec![combo],
    )
}

pub(super) fn invalid(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let invalid_combo =
        shadcn::Combobox::new(models.invalid_value.clone(), models.invalid_open.clone())
            .a11y_label("Combobox invalid")
            .width(Px(260.0))
            .placeholder("Select required option")
            .query_model(models.invalid_query.clone())
            .test_id_prefix("ui-gallery-combobox-invalid")
            .trigger_test_id("ui-gallery-combobox-invalid-trigger")
            .items(helpers::base_items())
            .aria_invalid(true)
            .into_element(cx);
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        move |cx| {
            vec![
                invalid_combo,
                helpers::state_rows(
                    cx,
                    &models.invalid_value,
                    &models.invalid_query,
                    "ui-gallery-combobox-invalid",
                ),
            ]
        },
    );
    content
}

pub(super) fn disabled(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let disabled_combo =
        shadcn::Combobox::new(models.disabled_value.clone(), models.disabled_open.clone())
            .a11y_label("Combobox disabled")
            .width(Px(260.0))
            .placeholder("Disabled")
            .query_model(models.disabled_query.clone())
            .test_id_prefix("ui-gallery-combobox-disabled")
            .trigger_test_id("ui-gallery-combobox-disabled-trigger")
            .items(helpers::base_items())
            .disabled(true)
            .into_element(cx);
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
    content
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
    .test_id_prefix("ui-gallery-combobox-input-group")
    .trigger_test_id("ui-gallery-combobox-input-group-trigger")
    .items([
        shadcn::ComboboxItem::new("new-file", "New File"),
        shadcn::ComboboxItem::new("open-file", "Open File"),
        shadcn::ComboboxItem::new("save-all", "Save All"),
    ])
    .into_element(cx);
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
    content
}

pub(super) fn rtl(cx: &mut ElementContext<'_, App>, models: &ComboboxModels) -> AnyElement {
    let rtl_combo = doc_layout::rtl(cx, |cx| {
        shadcn::Combobox::new(models.rtl_value.clone(), models.rtl_open.clone())
            .a11y_label("Combobox RTL")
            .width(Px(260.0))
            .placeholder("ابحث عن إطار عمل")
            .query_model(models.rtl_query.clone())
            .test_id_prefix("ui-gallery-combobox-rtl")
            .trigger_test_id("ui-gallery-combobox-rtl-trigger")
            .items([
                shadcn::ComboboxItem::new("next", "Next.js"),
                shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            ])
            .into_element(cx)
    });
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
    content
}
