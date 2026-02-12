use super::super::*;

pub(super) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct LabelPageModels {
        demo_email: Option<Model<String>>,
        field_email: Option<Model<String>>,
        rtl_name: Option<Model<String>>,
    }

    let (demo_email, field_email, rtl_name) = cx.with_state(LabelPageModels::default, |st| {
        (
            st.demo_email.clone(),
            st.field_email.clone(),
            st.rtl_name.clone(),
        )
    });

    let (demo_email, field_email, rtl_name) = match (demo_email, field_email, rtl_name) {
        (Some(demo_email), Some(field_email), Some(rtl_name)) => {
            (demo_email, field_email, rtl_name)
        }
        _ => {
            let demo_email = cx.app.models_mut().insert(String::new());
            let field_email = cx.app.models_mut().insert(String::new());
            let rtl_name = cx.app.models_mut().insert(String::new());

            cx.with_state(LabelPageModels::default, |st| {
                st.demo_email = Some(demo_email.clone());
                st.field_email = Some(field_email.clone());
                st.rtl_name = Some(rtl_name.clone());
            });

            (demo_email, field_email, rtl_name)
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
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
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

    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let demo = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(max_w.clone()),
            |cx| {
                vec![
                    shadcn::Label::new("Your email address").into_element(cx),
                    shadcn::Input::new(demo_email)
                        .placeholder("you@example.com")
                        .a11y_label("Email")
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-label-demo");
        section_card(cx, "Demo", content)
    };

    let label_in_field = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(max_w.clone()),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "For forms, prefer Field + FieldLabel for built-in description/error structure.",
                    ),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Work email").into_element(cx),
                        shadcn::Input::new(field_email)
                            .placeholder("name@company.com")
                            .a11y_label("Work email")
                            .into_element(cx),
                        shadcn::FieldDescription::new("We use this email for notifications.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-label-field");
        section_card(cx, "Label in Field", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(max_w.clone()),
                    |cx| {
                        vec![
                            shadcn::Label::new("????? ??????").into_element(cx),
                            shadcn::Input::new(rtl_name)
                                .placeholder("???? ???")
                                .a11y_label("????? ??????")
                                .into_element(cx),
                        ]
                    },
                )
            },
        )
        .test_id("ui-gallery-label-rtl");

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
                    "Preview follows shadcn Label docs order: Demo, Label in Field, RTL.",
                ),
                demo,
                label_in_field,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_panel_body).test_id("ui-gallery-label-component");

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
                        ui::text_block(cx, r#"Label::new("Your email address")"#).into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Form Field Composition").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"Field::new([FieldLabel::new("Work email"), Input::new(model), FieldDescription::new("...")])"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("RTL").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"with_direction_provider(cx, LayoutDirection::Rtl, |cx| Label::new("...").into_element(cx))"#,
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
                    "Label in Fret is a lightweight text primitive; form semantics and helper text live in `Field`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current `Label` API does not expose htmlFor binding; accessibility is handled by control a11y labels and form composition.",
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
        "ui-gallery-label",
        component_panel,
        code_panel,
        notes_panel,
    )
}
