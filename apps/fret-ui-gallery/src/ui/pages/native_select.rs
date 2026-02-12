use super::super::*;

pub(super) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct NativeSelectPageModels {
        styled_select_value: Option<Model<Option<Arc<str>>>>,
        styled_select_open: Option<Model<bool>>,
    }

    let (styled_select_value, styled_select_open) =
        cx.with_state(NativeSelectPageModels::default, |st| {
            (
                st.styled_select_value.clone(),
                st.styled_select_open.clone(),
            )
        });

    let (styled_select_value, styled_select_open) = match (styled_select_value, styled_select_open)
    {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let value = cx.app.models_mut().insert(Some(Arc::<str>::from("apple")));
            let open = cx.app.models_mut().insert(false);
            cx.with_state(NativeSelectPageModels::default, |st| {
                st.styled_select_value = Some(value.clone());
                st.styled_select_open = Some(open.clone());
            });
            (value, open)
        }
    };

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
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(820.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let select_width = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let content = shadcn::NativeSelect::new("Select a fruit")
            .a11y_label("Fruit")
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-demo");
        section_card(cx, "Demo", content)
    };

    let groups = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(select_width.clone()),
            |cx| {
                vec![
                    shadcn::NativeSelect::new("Fruits").a11y_label("Fruits group").into_element(cx),
                    shadcn::NativeSelect::new("Vegetables")
                        .a11y_label("Vegetables group")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "NativeSelect currently exposes a single-label API; optgroup-like grouping is approximated here with multiple selects.",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-native-select-groups");
        section_card(cx, "Groups", content)
    };

    let disabled = {
        let content = shadcn::NativeSelect::new("Disabled")
            .a11y_label("Disabled select")
            .disabled(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-disabled");
        section_card(cx, "Disabled", content)
    };

    let invalid = {
        let content = shadcn::NativeSelect::new("Select a country")
            .a11y_label("Invalid select")
            .aria_invalid(true)
            .refine_layout(select_width.clone())
            .into_element(cx)
            .test_id("ui-gallery-native-select-invalid");
        section_card(cx, "Invalid", content)
    };

    let native_vs_select = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                vec![
                    shadcn::NativeSelect::new("Native select")
                        .a11y_label("Native select")
                        .into_element(cx),
                    shadcn::Select::new(styled_select_value.clone(), styled_select_open.clone())
                        .placeholder("Styled select")
                        .items([
                            shadcn::SelectItem::new("apple", "Apple"),
                            shadcn::SelectItem::new("banana", "Banana"),
                            shadcn::SelectItem::new("blueberry", "Blueberry"),
                        ])
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "Use NativeSelect for native browser behavior/mobile ergonomics; use Select for richer overlays and custom interactions.",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-native-select-vs-select");
        section_card(cx, "Native Select vs Select", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::NativeSelect::new("Select language")
                    .a11y_label("RTL native select")
                    .refine_layout(select_width.clone())
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-native-select-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Native Select docs order: Demo, Groups, Disabled, Invalid, Native Select vs Select, RTL.",
                ),
                demo,
                groups,
                disabled,
                invalid,
                native_vs_select,
                rtl,
            ]
        },
    );
    let component_panel =
        shell(cx, component_panel_body).test_id("ui-gallery-native-select-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Basic Usage").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"NativeSelect::new("Select a fruit").a11y_label("Fruit")"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Disabled and Invalid").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"NativeSelect::new("...").disabled(true); NativeSelect::new("...").aria_invalid(true);"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Compare APIs").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"NativeSelect has native surface semantics; Select provides custom popup/list interactions."#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
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
                    "Current NativeSelect API is label-based; explicit option/optgroup nodes are not exposed yet.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Groups example is a practical approximation until optgroup-level API is added.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-native-select",
        component_panel,
        code_panel,
        notes_panel,
    )
}
