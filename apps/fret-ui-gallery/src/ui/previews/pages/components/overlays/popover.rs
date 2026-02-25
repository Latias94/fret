use super::super::super::super::super::*;

pub(in crate::ui) fn preview_popover(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default, Clone)]
    struct PopoverModels {
        demo_width: Option<Model<String>>,
        demo_max_width: Option<Model<String>>,
        demo_height: Option<Model<String>>,
        demo_max_height: Option<Model<String>>,
        form_width: Option<Model<String>>,
        form_height: Option<Model<String>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let state = cx.with_state(PopoverModels::default, |st| st.clone());
    let demo_width = match state.demo_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_width = Some(model.clone())
            });
            model
        }
    };
    let demo_max_width = match state.demo_max_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("300px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_width = Some(model.clone())
            });
            model
        }
    };
    let demo_height = match state.demo_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_height = Some(model.clone())
            });
            model
        }
    };
    let demo_max_height = match state.demo_max_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("none"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_height = Some(model.clone())
            });
            model
        }
    };
    let form_width = match state.form_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_width = Some(model.clone())
            });
            model
        }
    };
    let form_height = match state.form_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_height = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    let trigger = shadcn::Button::new("Open popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                        .test_id("ui-gallery-popover-demo-trigger");
                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                |cx| {
                    let row =
                        |cx: &mut ElementContext<'_, App>, label: &'static str, model: Model<_>| {
                            stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N4)
                                    .items_center(),
                                move |cx| {
                                    vec![
                                        stack::hstack(
                                            cx,
                                            stack::HStackProps::default()
                                                .layout(
                                                    LayoutRefinement::default()
                                                        .w_px(Px(96.0))
                                                        .flex_shrink_0(),
                                                )
                                                .justify_end()
                                                .items_center(),
                                            move |cx| vec![ui::label(cx, label).into_element(cx)],
                                        ),
                                        shadcn::Input::new(model)
                                            .size(fret_ui_kit::Size::Small)
                                            .refine_layout(
                                                LayoutRefinement::default().flex_1().min_w_0(),
                                            )
                                            .into_element(cx),
                                    ]
                                },
                            )
                        };

                    let header = shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx);

                    let fields = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N2)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full()),
                        move |cx| {
                            vec![
                                row(cx, "Width", demo_width.clone()),
                                row(cx, "Max. width", demo_max_width.clone()),
                                row(cx, "Height", demo_height.clone()),
                                row(cx, "Max. height", demo_max_height.clone()),
                            ]
                        },
                    );

                    shadcn::PopoverContent::new([header, fields])
                        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                        .into_element(cx)
                        .test_id("ui-gallery-popover-demo-panel")
                },
            );
        centered(cx, popover).test_id("ui-gallery-popover-demo")
    };

    let basic = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    let trigger = shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                        .test_id("ui-gallery-popover-basic-trigger");
                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx)])
                    .into_element(cx)
                    .test_id("ui-gallery-popover-basic-panel")
                },
            );
        centered(cx, popover).test_id("ui-gallery-popover-basic")
    };

    let align = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N6)
                .items_center()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            |cx| {
                vec![
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Start)
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = shadcn::Button::new("Start")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-start-trigger");
                                shadcn::PopoverTrigger::new(trigger).into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to start")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-start-content")
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Center)
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = shadcn::Button::new("Center")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-center-trigger");
                                shadcn::PopoverTrigger::new(trigger).into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to center")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-center-content")
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::End)
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = shadcn::Button::new("End")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-end-trigger");
                                shadcn::PopoverTrigger::new(trigger).into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to end")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                                    .test_id("ui-gallery-popover-align-end-content")
                            },
                        ),
                ]
            },
        );
        body.test_id("ui-gallery-popover-align")
    };

    let with_form = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    let trigger = shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                        .test_id("ui-gallery-popover-with-form-trigger");
                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([
                        shadcn::PopoverHeader::new([
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::FieldGroup::new([
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Width")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_width.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Height")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_height.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                        ])
                        .gap(Space::N4)
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_px(Px(256.0)))
                    .into_element(cx)
                    .test_id("ui-gallery-popover-with-form-panel")
                },
            );
        centered(cx, popover).test_id("ui-gallery-popover-with-form")
    };

    let rtl = {
        doc_layout::rtl(cx, |cx| {
            let popover = |cx: &mut ElementContext<'_, App>, label: &'static str, side| {
                shadcn::Popover::new_controllable(cx, None, false)
                    .side(side)
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = shadcn::Button::new(label)
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx);
                            shadcn::PopoverTrigger::new(trigger).into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                                shadcn::PopoverTitle::new("الأبعاد").into_element(cx),
                                shadcn::PopoverDescription::new("تعيين الأبعاد للطبقة.")
                                    .into_element(cx),
                            ])
                            .into_element(cx)])
                            .into_element(cx)
                        },
                    )
            };

            let physical = stack::hstack_build(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_center(),
                |cx, out| {
                    for (id, label, side) in [
                        ("left", "يسار", shadcn::PopoverSide::Left),
                        ("top", "أعلى", shadcn::PopoverSide::Top),
                        ("bottom", "أسفل", shadcn::PopoverSide::Bottom),
                        ("right", "يمين", shadcn::PopoverSide::Right),
                    ] {
                        out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                    }
                },
            );

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N4)
                    .layout(LayoutRefinement::default().w_full()),
                move |_cx| [physical],
            )
        })
        .test_id("ui-gallery-popover-rtl")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Popover demo (new-york-v4).",
            "Use `align(Start)` to match the default docs layout; keep content width explicit (e.g. 320px).",
            "For dense input rows, prefer `Field`/`FieldGroup` recipes to keep spacing consistent with other form surfaces.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Popover demo: Dimensions form (align=start, w=320px)."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-popover-demo")
                .code(
                    "rust",
                    r#"shadcn::Popover::new_controllable(cx, None, false)
    .align(shadcn::PopoverAlign::Start)
    .into_element(
        cx,
        |cx| {
            let trigger = shadcn::Button::new("Open popover")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx);
            shadcn::PopoverTrigger::new(trigger).into_element(cx)
        },
        |cx| shadcn::PopoverContent::new([/* content */])
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
            .into_element(cx),
    );"#,
                ),
            DocSection::new("Basic", basic)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-popover-basic")
                .code(
                    "rust",
                    r#"shadcn::Popover::new_controllable(cx, None, false)
    .align(shadcn::PopoverAlign::Start)
    .into_element(
        cx,
        |cx| shadcn::PopoverTrigger::new(trigger).into_element(cx),
        content,
    );"#,
                ),
            DocSection::new("Align", align)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-popover-align")
                .code(
                    "rust",
                    r#"shadcn::Popover::new_controllable(cx, None, false)
    .align(shadcn::PopoverAlign::End)
    .into_element(
        cx,
        |cx| shadcn::PopoverTrigger::new(trigger).into_element(cx),
        content,
    );"#,
                ),
            DocSection::new("With Form", with_form)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-popover-with-form")
                .code(
                    "rust",
                    r#"shadcn::PopoverContent::new([
    shadcn::PopoverHeader::new([title, description]).into_element(cx),
    shadcn::FieldGroup::new([/* fields */]).into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-popover-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Popover::new_controllable(cx, None, false)
        .side(shadcn::PopoverSide::Left)
        .into_element(
            cx,
            |cx| shadcn::PopoverTrigger::new(trigger).into_element(cx),
            content,
        )
});"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-popover")]
}
