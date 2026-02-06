use super::super::*;

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ComboboxModels {
        custom_value: Option<Model<Option<Arc<str>>>>,
        custom_open: Option<Model<bool>>,
        custom_query: Option<Model<String>>,
        basic_value: Option<Model<Option<Arc<str>>>>,
        basic_open: Option<Model<bool>>,
        basic_query: Option<Model<String>>,
        groups_value: Option<Model<Option<Arc<str>>>>,
        groups_open: Option<Model<bool>>,
        groups_query: Option<Model<String>>,
        invalid_value: Option<Model<Option<Arc<str>>>>,
        invalid_open: Option<Model<bool>>,
        invalid_query: Option<Model<String>>,
        disabled_value: Option<Model<Option<Arc<str>>>>,
        disabled_open: Option<Model<bool>>,
        disabled_query: Option<Model<String>>,
        input_group_value: Option<Model<Option<Arc<str>>>>,
        input_group_open: Option<Model<bool>>,
        input_group_query: Option<Model<String>>,
        rtl_value: Option<Model<Option<Arc<str>>>>,
        rtl_open: Option<Model<bool>>,
        rtl_query: Option<Model<String>>,
    }

    let (
        custom_value,
        custom_open,
        custom_query,
        basic_value,
        basic_open,
        basic_query,
        groups_value,
        groups_open,
        groups_query,
        invalid_value,
        invalid_open,
        invalid_query,
        disabled_value,
        disabled_open,
        disabled_query,
        input_group_value,
        input_group_open,
        input_group_query,
        rtl_value,
        rtl_open,
        rtl_query,
    ) = cx.with_state(ComboboxModels::default, |st| {
        (
            st.custom_value.clone(),
            st.custom_open.clone(),
            st.custom_query.clone(),
            st.basic_value.clone(),
            st.basic_open.clone(),
            st.basic_query.clone(),
            st.groups_value.clone(),
            st.groups_open.clone(),
            st.groups_query.clone(),
            st.invalid_value.clone(),
            st.invalid_open.clone(),
            st.invalid_query.clone(),
            st.disabled_value.clone(),
            st.disabled_open.clone(),
            st.disabled_query.clone(),
            st.input_group_value.clone(),
            st.input_group_open.clone(),
            st.input_group_query.clone(),
            st.rtl_value.clone(),
            st.rtl_open.clone(),
            st.rtl_query.clone(),
        )
    });

    let (
        custom_value,
        custom_open,
        custom_query,
        basic_value,
        basic_open,
        basic_query,
        groups_value,
        groups_open,
        groups_query,
        invalid_value,
        invalid_open,
        invalid_query,
        disabled_value,
        disabled_open,
        disabled_query,
        input_group_value,
        input_group_open,
        input_group_query,
        rtl_value,
        rtl_open,
        rtl_query,
    ) = match (
        custom_value,
        custom_open,
        custom_query,
        basic_value,
        basic_open,
        basic_query,
        groups_value,
        groups_open,
        groups_query,
        invalid_value,
        invalid_open,
        invalid_query,
        disabled_value,
        disabled_open,
        disabled_query,
        input_group_value,
        input_group_open,
        input_group_query,
        rtl_value,
        rtl_open,
        rtl_query,
    ) {
        (
            Some(custom_value),
            Some(custom_open),
            Some(custom_query),
            Some(basic_value),
            Some(basic_open),
            Some(basic_query),
            Some(groups_value),
            Some(groups_open),
            Some(groups_query),
            Some(invalid_value),
            Some(invalid_open),
            Some(invalid_query),
            Some(disabled_value),
            Some(disabled_open),
            Some(disabled_query),
            Some(input_group_value),
            Some(input_group_open),
            Some(input_group_query),
            Some(rtl_value),
            Some(rtl_open),
            Some(rtl_query),
        ) => (
            custom_value,
            custom_open,
            custom_query,
            basic_value,
            basic_open,
            basic_query,
            groups_value,
            groups_open,
            groups_query,
            invalid_value,
            invalid_open,
            invalid_query,
            disabled_value,
            disabled_open,
            disabled_query,
            input_group_value,
            input_group_open,
            input_group_query,
            rtl_value,
            rtl_open,
            rtl_query,
        ),
        _ => {
            let custom_value = cx.app.models_mut().insert(None);
            let custom_open = cx.app.models_mut().insert(false);
            let custom_query = cx.app.models_mut().insert(String::new());

            let basic_value = cx.app.models_mut().insert(None);
            let basic_open = cx.app.models_mut().insert(false);
            let basic_query = cx.app.models_mut().insert(String::new());

            let groups_value = cx.app.models_mut().insert(None);
            let groups_open = cx.app.models_mut().insert(false);
            let groups_query = cx.app.models_mut().insert(String::new());

            let invalid_value = cx.app.models_mut().insert(None);
            let invalid_open = cx.app.models_mut().insert(false);
            let invalid_query = cx.app.models_mut().insert(String::new());

            let disabled_value = cx.app.models_mut().insert(Some(Arc::<str>::from("banana")));
            let disabled_open = cx.app.models_mut().insert(false);
            let disabled_query = cx.app.models_mut().insert(String::new());

            let input_group_value = cx.app.models_mut().insert(None);
            let input_group_open = cx.app.models_mut().insert(false);
            let input_group_query = cx.app.models_mut().insert(String::new());

            let rtl_value = cx.app.models_mut().insert(None);
            let rtl_open = cx.app.models_mut().insert(false);
            let rtl_query = cx.app.models_mut().insert(String::new());

            cx.with_state(ComboboxModels::default, |st| {
                st.custom_value = Some(custom_value.clone());
                st.custom_open = Some(custom_open.clone());
                st.custom_query = Some(custom_query.clone());

                st.basic_value = Some(basic_value.clone());
                st.basic_open = Some(basic_open.clone());
                st.basic_query = Some(basic_query.clone());

                st.groups_value = Some(groups_value.clone());
                st.groups_open = Some(groups_open.clone());
                st.groups_query = Some(groups_query.clone());

                st.invalid_value = Some(invalid_value.clone());
                st.invalid_open = Some(invalid_open.clone());
                st.invalid_query = Some(invalid_query.clone());

                st.disabled_value = Some(disabled_value.clone());
                st.disabled_open = Some(disabled_open.clone());
                st.disabled_query = Some(disabled_query.clone());

                st.input_group_value = Some(input_group_value.clone());
                st.input_group_open = Some(input_group_open.clone());
                st.input_group_query = Some(input_group_query.clone());

                st.rtl_value = Some(rtl_value.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.rtl_query = Some(rtl_query.clone());
            });

            (
                custom_value,
                custom_open,
                custom_query,
                basic_value,
                basic_open,
                basic_query,
                groups_value,
                groups_open,
                groups_query,
                invalid_value,
                invalid_open,
                invalid_query,
                disabled_value,
                disabled_open,
                disabled_query,
                input_group_value,
                input_group_open,
                input_group_query,
                rtl_value,
                rtl_open,
                rtl_query,
            )
        }
    };

    let theme = Theme::global(&*cx.app).clone();
    let destructive = theme.color_required("destructive");

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let state_rows = |cx: &mut ElementContext<'_, App>,
                      value: &Model<Option<Arc<str>>>,
                      query: &Model<String>,
                      test_id_prefix: &'static str| {
        let selected = cx
            .app
            .models()
            .read(value, |v| v.clone())
            .ok()
            .flatten()
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let query_text = cx
            .get_model_cloned(query, Invalidation::Layout)
            .unwrap_or_default();

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N1)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::typography::muted(cx, format!("Selected: {selected}"))
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .test_id(format!("{test_id_prefix}-selected")),
                        ),
                    shadcn::typography::muted(cx, format!("Query: {query_text}")).attach_semantics(
                        SemanticsDecoration::default().test_id(format!("{test_id_prefix}-query")),
                    ),
                ]
            },
        )
    };

    let base_items = || {
        vec![
            shadcn::ComboboxItem::new("apple", "Apple"),
            shadcn::ComboboxItem::new("banana", "Banana"),
            shadcn::ComboboxItem::new("orange", "Orange"),
            shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
        ]
    };

    let demo_combo = shadcn::Combobox::new(value.clone(), open.clone())
        .a11y_label("Combobox demo")
        .width(Px(260.0))
        .placeholder("Pick a fruit")
        .query_model(query.clone())
        .items(base_items())
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-demo-trigger"),
        );
    let demo_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                demo_combo,
                state_rows(cx, &value, &query, "ui-gallery-combobox-demo"),
            ]
        },
    );
    let demo = section_card(cx, "Demo", demo_content);

    let custom_combo = shadcn::Combobox::new(custom_value.clone(), custom_open.clone())
        .a11y_label("Combobox custom items")
        .width(Px(280.0))
        .placeholder("Select framework")
        .query_model(custom_query.clone())
        .items([
            shadcn::ComboboxItem::new("next", "Next.js (React)"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js (Vue)"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit (Svelte)"),
            shadcn::ComboboxItem::new("astro", "Astro (Hybrid)"),
        ])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-custom-items-trigger"),
        );
    let custom_items_top_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
        |cx| {
            vec![
                custom_combo,
                shadcn::typography::muted(
                    cx,
                    "Fret currently uses string value/label pairs; object-item mapping (`itemToStringValue`) is approximated by richer labels.",
                ),
                state_rows(
                    cx,
                    &custom_value,
                    &custom_query,
                    "ui-gallery-combobox-custom-items",
                ),
            ]
        },
    );
    let custom_items_top = section_card(cx, "Custom Items", custom_items_top_content);

    let multiple_selection = section_card(
        cx,
        "Multiple Selection",
        shadcn::typography::muted(
            cx,
            "Upstream supports chips + multiple values. Current Fret `Combobox` API is single-select; keep this as an explicit parity gap marker.",
        ),
    );

    let basic_combo = shadcn::Combobox::new(basic_value.clone(), basic_open.clone())
        .a11y_label("Combobox basic")
        .width(Px(260.0))
        .placeholder("Select a framework")
        .query_model(basic_query.clone())
        .items([
            shadcn::ComboboxItem::new("next", "Next.js"),
            shadcn::ComboboxItem::new("svelte", "SvelteKit"),
            shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
            shadcn::ComboboxItem::new("remix", "Remix"),
            shadcn::ComboboxItem::new("astro", "Astro"),
        ])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-basic-trigger"),
        );
    let basic = section_card(
        cx,
        "Basic",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    basic_combo,
                    state_rows(cx, &basic_value, &basic_query, "ui-gallery-combobox-basic"),
                ]
            },
        ),
    );

    let multiple = section_card(
        cx,
        "Multiple",
        shadcn::typography::muted(
            cx,
            "`multiple` + chips behavior is not exposed in current Fret `Combobox`; tracked as a follow-up API expansion.",
        ),
    );

    let clear_button = section_card(
        cx,
        "Clear Button",
        shadcn::typography::muted(
            cx,
            "Upstream has `showClear`. Current Fret API can be cleared by external state reset, but does not provide built-in clear trigger yet.",
        ),
    );

    let groups_combo = shadcn::Combobox::new(groups_value.clone(), groups_open.clone())
        .a11y_label("Combobox groups")
        .width(Px(300.0))
        .placeholder("Filter commands")
        .query_model(groups_query.clone())
        .items([
            shadcn::ComboboxItem::new("framework-next", "Frameworks / Next.js"),
            shadcn::ComboboxItem::new("framework-nuxt", "Frameworks / Nuxt.js"),
            shadcn::ComboboxItem::new("language-rust", "Languages / Rust"),
            shadcn::ComboboxItem::new("language-typescript", "Languages / TypeScript"),
            shadcn::ComboboxItem::new("tool-cargo", "Tools / Cargo"),
        ])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-groups-trigger"),
        );
    let groups = section_card(
        cx,
        "Groups",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(340.0))),
            |cx| {
                vec![
                    groups_combo,
                    shadcn::typography::muted(
                        cx,
                        "Grouped rows are approximated with prefix labels until dedicated group/separator APIs are introduced.",
                    ),
                    state_rows(
                        cx,
                        &groups_value,
                        &groups_query,
                        "ui-gallery-combobox-groups",
                    ),
                ]
            },
        ),
    );

    let custom_items_example = section_card(
        cx,
        "Custom Items",
        shadcn::typography::muted(
            cx,
            "Render-rich custom item surfaces are currently approximated at label level in this gallery.",
        ),
    );

    let invalid_combo = shadcn::Combobox::new(invalid_value.clone(), invalid_open.clone())
        .a11y_label("Combobox invalid")
        .width(Px(260.0))
        .placeholder("Select required option")
        .query_model(invalid_query.clone())
        .items(base_items())
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .border_color(ColorRef::Color(destructive)),
        )
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-invalid-trigger"),
        );
    let invalid = section_card(
        cx,
        "Invalid",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    invalid_combo,
                    shadcn::typography::muted(
                        cx,
                        "Invalid visual is currently approximated via destructive border style on trigger.",
                    ),
                    state_rows(
                        cx,
                        &invalid_value,
                        &invalid_query,
                        "ui-gallery-combobox-invalid",
                    ),
                ]
            },
        ),
    );

    let disabled_combo = shadcn::Combobox::new(disabled_value.clone(), disabled_open.clone())
        .a11y_label("Combobox disabled")
        .width(Px(260.0))
        .placeholder("Disabled")
        .query_model(disabled_query.clone())
        .items(base_items())
        .disabled(true)
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default().test_id("ui-gallery-combobox-disabled-trigger"),
        );
    let disabled = section_card(
        cx,
        "Disabled",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    disabled_combo,
                    state_rows(
                        cx,
                        &disabled_value,
                        &disabled_query,
                        "ui-gallery-combobox-disabled",
                    ),
                ]
            },
        ),
    );

    let auto_highlight_combo =
        shadcn::Combobox::new(input_group_value.clone(), input_group_open.clone())
            .a11y_label("Combobox auto highlight")
            .width(Px(260.0))
            .placeholder("Type to filter")
            .query_model(input_group_query.clone())
            .items(base_items())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .test_id("ui-gallery-combobox-auto-highlight-trigger"),
            );
    let auto_highlight = section_card(
        cx,
        "Auto Highlight",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    auto_highlight_combo,
                    shadcn::typography::muted(
                        cx,
                        "Current behavior follows command palette defaults; explicit `autoHighlight` knob is not yet surfaced.",
                    ),
                ]
            },
        ),
    );

    let popup = section_card(
        cx,
        "Popup",
        shadcn::typography::muted(
            cx,
            "Trigger-as-button popup recipe is not yet exposed as a dedicated API in Fret Combobox.",
        ),
    );

    let input_group_combo =
        shadcn::Combobox::new(input_group_value.clone(), input_group_open.clone())
            .a11y_label("Combobox input group")
            .width(Px(220.0))
            .placeholder("Search command")
            .query_model(input_group_query.clone())
            .items([
                shadcn::ComboboxItem::new("new-file", "New File"),
                shadcn::ComboboxItem::new("open-file", "Open File"),
                shadcn::ComboboxItem::new("save-all", "Save All"),
            ])
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-combobox-input-group-trigger"),
            );
    let input_group = section_card(
        cx,
        "Input Group",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(360.0))),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                cx.container(
                                    decl_style::container_props(
                                        &theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Sm)
                                            .px(Space::N2)
                                            .py(Space::N1),
                                        LayoutRefinement::default(),
                                    ),
                                    |cx| vec![shadcn::typography::muted(cx, "Cmd")],
                                ),
                                input_group_combo,
                            ]
                        },
                    ),
                    state_rows(
                        cx,
                        &input_group_value,
                        &input_group_query,
                        "ui-gallery-combobox-input-group",
                    ),
                ]
            },
        ),
    );

    let rtl_combo = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Combobox::new(rtl_value.clone(), rtl_open.clone())
                .a11y_label("Combobox RTL")
                .width(Px(260.0))
                .placeholder("???? ???? ?????")
                .query_model(rtl_query.clone())
                .items([
                    shadcn::ComboboxItem::new("next", "Next.js"),
                    shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
                    shadcn::ComboboxItem::new("svelte", "SvelteKit"),
                ])
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default().test_id("ui-gallery-combobox-rtl-trigger"),
                )
        },
    );
    let rtl = section_card(
        cx,
        "RTL",
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    rtl_combo,
                    state_rows(cx, &rtl_value, &rtl_query, "ui-gallery-combobox-rtl"),
                ]
            },
        ),
    );

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Combobox docs flow; unsupported recipes are kept as explicit gap markers.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
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
    let component_panel = shell(cx, component_stack)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-combobox-component"));

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic",
                    r#"let combo = shadcn::Combobox::new(value, open)
    .placeholder("Select a framework")
    .query_model(query)
    .items([
        shadcn::ComboboxItem::new("next", "Next.js"),
        shadcn::ComboboxItem::new("nuxt", "Nuxt.js"),
    ])
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Style + Disabled",
                    r#"let invalid = shadcn::Combobox::new(value, open)
    .refine_style(ChromeRefinement::default().border_1())
    .disabled(true)
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Combobox::new(value, open)
        .placeholder("????")
        .into_element(cx)
})"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
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
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-combobox",
        component_panel,
        code_panel,
        notes_panel,
    )
}
