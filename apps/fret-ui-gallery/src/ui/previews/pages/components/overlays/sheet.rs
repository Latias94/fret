use super::super::super::super::super::*;

pub(in crate::ui) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default, Clone)]
    struct SheetModels {
        demo_name: Option<Model<String>>,
        demo_username: Option<Model<String>>,
        side_top_open: Option<Model<bool>>,
        side_right_open: Option<Model<bool>>,
        side_bottom_open: Option<Model<bool>>,
        side_left_open: Option<Model<bool>>,
        no_close_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        rtl_name: Option<Model<String>>,
        rtl_username: Option<Model<String>>,
    }

    let state = cx.with_state(SheetModels::default, |st| st.clone());

    let demo_name = match state.demo_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_name = Some(model.clone())
            });
            model
        }
    };

    let demo_username = match state.demo_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("@peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_username = Some(model.clone())
            });
            model
        }
    };

    let side_top_open = match state.side_top_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_top_open = Some(model.clone())
            });
            model
        }
    };

    let side_right_open = match state.side_right_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_right_open = Some(model.clone())
            });
            model
        }
    };

    let side_bottom_open = match state.side_bottom_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_bottom_open = Some(model.clone())
            });
            model
        }
    };

    let side_left_open = match state.side_left_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_left_open = Some(model.clone())
            });
            model
        }
    };

    let no_close_open = match state.no_close_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.no_close_open = Some(model.clone())
            });
            model
        }
    };

    let rtl_open = match state.rtl_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| st.rtl_open = Some(model.clone()));
            model
        }
    };

    let rtl_name = match state.rtl_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| st.rtl_name = Some(model.clone()));
            model
        }
    };

    let rtl_username = match state.rtl_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.rtl_username = Some(model.clone())
            });
            model
        }
    };

    let profile_fields =
        |cx: &mut ElementContext<'_, App>, name: Model<String>, username: Model<String>| {
            let field =
                |cx: &mut ElementContext<'_, App>, label: &'static str, model: Model<String>| {
                    shadcn::Field::new([
                        shadcn::FieldLabel::new(label).into_element(cx),
                        shadcn::Input::new(model)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    ])
                    .into_element(cx)
                };

            shadcn::FieldSet::new([field(cx, "Name", name), field(cx, "Username", username)])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
        };

    let demo = {
        let trigger_open = open.clone();
        let save_open = open.clone();
        let close_open = open.clone();
        let name_model = demo_name.clone();
        let username_model = demo_username.clone();

        let demo_sheet = shadcn::Sheet::new(open.clone())
            .side(shadcn::SheetSide::Right)
            .size(Px(420.0))
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::SheetContent::new([
                        shadcn::SheetHeader::new([
                            shadcn::SheetTitle::new("Edit profile").into_element(cx),
                            shadcn::SheetDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        profile_fields(cx, name_model.clone(), username_model.clone()),
                        shadcn::SheetFooter::new([
                            shadcn::Button::new("Save changes")
                                .toggle_model(save_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id("ui-gallery-sheet-demo-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sheet-demo"),
            );

        demo_sheet
    };

    let side = {
        let side_sheet = |cx: &mut ElementContext<'_, App>,
                          id: &'static str,
                          label: &'static str,
                          side: shadcn::SheetSide,
                          open_model: Model<bool>| {
            let trigger_open = open_model.clone();
            let save_open = open_model.clone();

            let sheet = shadcn::Sheet::new(open_model).side(side);
            let sheet = if matches!(side, shadcn::SheetSide::Left | shadcn::SheetSide::Right) {
                sheet.size(Px(420.0))
            } else {
                // Upstream shadcn uses `h-auto` for top/bottom sheets; keep them auto-sized so the
                // footer actions remain fully visible on typical viewport heights.
                sheet
            };

            sheet.into_element(
                cx,
                |cx| {
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .test_id(format!("ui-gallery-sheet-side-{id}-trigger"))
                        .into_element(cx)
                },
                |cx| {
                    let fields = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N4)
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        |cx| {
                            vec![
                                shadcn::Field::new([
                                    shadcn::FieldLabel::new("Name").into_element(cx),
                                    shadcn::Input::new(demo_name.clone())
                                        .refine_layout(LayoutRefinement::default().w_full())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                                shadcn::Field::new([
                                    shadcn::FieldLabel::new("Username").into_element(cx),
                                    shadcn::Input::new(demo_username.clone())
                                        .refine_layout(LayoutRefinement::default().w_full())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    );

                    shadcn::SheetContent::new([
                        shadcn::SheetHeader::new([
                            shadcn::SheetTitle::new("Edit profile").into_element(cx),
                            shadcn::SheetDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        fields,
                        shadcn::SheetFooter::new([shadcn::Button::new("Save changes")
                            .toggle_model(save_open.clone())
                            .test_id(format!("ui-gallery-sheet-side-{id}-save"))
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id(format!("ui-gallery-sheet-side-{id}-content")),
                    )
                },
            )
        };

        let theme = Theme::global(&*cx.app).snapshot();
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            let items = [
                ("top", "top", shadcn::SheetSide::Top, side_top_open.clone()),
                (
                    "right",
                    "right",
                    shadcn::SheetSide::Right,
                    side_right_open.clone(),
                ),
                (
                    "bottom",
                    "bottom",
                    shadcn::SheetSide::Bottom,
                    side_bottom_open.clone(),
                ),
                (
                    "left",
                    "left",
                    shadcn::SheetSide::Left,
                    side_left_open.clone(),
                ),
            ];

            items
                .into_iter()
                .map(|(id, label, side, open_model)| {
                    cx.keyed(id, |cx| side_sheet(cx, id, label, side, open_model.clone()))
                })
                .collect::<Vec<_>>()
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-side"),
        )
    };

    let no_close_button = {
        let trigger_open = no_close_open.clone();

        let sheet = shadcn::Sheet::new(no_close_open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("No Close Button").into_element(cx),
                        shadcn::SheetDescription::new(
                            "This example intentionally omits footer actions. Use outside press or Escape to close.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-no-close-button"),
        );

        sheet
    };

    let rtl = {
        let rtl_demo = doc_layout::rtl(cx, |cx| {
            let trigger_open = rtl_open.clone();
            let save_open = rtl_open.clone();
            let close_open = rtl_open.clone();
            let name_model = rtl_name.clone();
            let username_model = rtl_username.clone();

            shadcn::Sheet::new(rtl_open.clone())
                .side(shadcn::SheetSide::Left)
                .size(Px(420.0))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(trigger_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::SheetContent::new([
                            shadcn::SheetHeader::new([
                                shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                shadcn::SheetDescription::new(
                                    "RTL layout keeps spacing and focus flow aligned.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            profile_fields(cx, name_model.clone(), username_model.clone()),
                            shadcn::SheetFooter::new([
                                shadcn::Button::new("Save changes")
                                    .toggle_model(save_open.clone())
                                    .into_element(cx),
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(close_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    },
                )
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-rtl"),
        );

        rtl_demo
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Sheet demo (new-york-v4).",
            "Upstream exposes `SheetClose` and a default close button; Fret currently relies on Escape/outside press + explicit `open` toggles in actions.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Extends dialog to display side-aligned panels for supplementary tasks."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-demo")
                .code(
                    "rust",
                    r#"shadcn::Sheet::new(open)
    .side(shadcn::SheetSide::Right)
    .size(Px(420.0))
    .into_element(cx, trigger, content);"#,
                ),
            DocSection::new("Side", side)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-side")
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);

shadcn::Sheet::new(open.clone())
    .side(shadcn::SheetSide::Top)
    .into_element(cx, trigger, content);"#,
                ),
            DocSection::new("No Close Button", no_close_button)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-no-close")
                .code(
                    "rust",
                    r#"shadcn::Sheet::new(open)
    .into_element(cx, trigger, content);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sheet-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Sheet::new(open).into_element(cx, trigger, content)
});"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sheet-notes"),
        ],
    );

    vec![body]
}
