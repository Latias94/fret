use super::super::super::super::*;

pub(in crate::ui) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_alert_dialog(cx, open)
}

pub(in crate::ui) fn preview_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_dialog(cx, open)
}

pub(in crate::ui) fn preview_popover(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
) -> Vec<AnyElement> {
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
        let popover = shadcn::Popover::new_controllable(cx, None, false).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                let row = |cx: &mut ElementContext<'_, App>,
                           label: &'static str,
                           model: Model<_>| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4)
                            .items_center(),
                        move |cx| {
                            vec![
                                ui::label(cx, label)
                                    .layout(
                                        LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0(),
                                    )
                                    .into_element(cx),
                                shadcn::Input::new(model)
                                    .size(fret_ui_kit::Size::Small)
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let header = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ]
                    },
                );

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
            },
        );
        let body = centered(cx, popover);
        section(cx, "Demo", body)
    };

    let basic = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx)])
                    .into_element(cx)
                },
            );
        let body = centered(cx, popover);
        section(cx, "Basic", body)
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
                                shadcn::Button::new("Start")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to start")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Center)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("Center")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to center")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::End)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("End")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to end")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                ]
            },
        );
        section(cx, "Align", body)
    };

    let with_form = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
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
                },
            );
        let body = centered(cx, popover);
        section(cx, "With Form", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let popover = |cx: &mut ElementContext<'_, App>,
                               label: &'static str,
                               side: shadcn::PopoverSide| {
                    shadcn::Popover::new_controllable(cx, None, false)
                        .side(side)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new(label)
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx)
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

                let logical = stack::hstack_build(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |cx, out| {
                        for (id, label, side) in [
                            (
                                "inline-start",
                                "بداية السطر",
                                shadcn::PopoverSide::InlineStart,
                            ),
                            ("inline-end", "نهاية السطر", shadcn::PopoverSide::InlineEnd),
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
                    move |_cx| [physical, logical],
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![demo, basic, align, with_form, rtl]
}

pub(in crate::ui) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
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

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

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

        let card = shell(cx, LayoutRefinement::default(), demo_sheet);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let side = {
        let side_sheet = |cx: &mut ElementContext<'_, App>,
                          id: &'static str,
                          label: &'static str,
                          side: shadcn::SheetSide,
                          open_model: Model<bool>| {
            let trigger_open = open_model.clone();
            let save_open = open_model.clone();
            let cancel_open = open_model.clone();
            let size = if matches!(side, shadcn::SheetSide::Top | shadcn::SheetSide::Bottom) {
                Px(320.0)
            } else {
                Px(420.0)
            };

            shadcn::Sheet::new(open_model)
                .side(side)
                .size(size)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new(label)
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(trigger_open.clone())
                            .test_id(format!("ui-gallery-sheet-side-{id}-trigger"))
                            .into_element(cx)
                    },
                    |cx| {
                        let paragraphs = stack::vstack(
                            cx,
                            stack::VStackProps::default().gap(Space::N2),
                            |cx| {
                                (0..8)
                                    .map(|idx| {
                                        shadcn::typography::muted(
                                            cx,
                                            format!(
                                                "Profile section line {}. Keep this content scrollable for constrained sheets.",
                                                idx + 1
                                            ),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            },
                        );

                        let scroll = shadcn::ScrollArea::new([paragraphs])
                            .axis(fret_ui::element::ScrollAxis::Y)
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(180.0)))
                            .into_element(cx);

                        shadcn::SheetContent::new([
                            shadcn::SheetHeader::new([
                                shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                shadcn::SheetDescription::new(
                                    "Use side to control which edge the sheet appears from.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            scroll,
                            shadcn::SheetFooter::new([
                                shadcn::Button::new("Save changes")
                                    .toggle_model(save_open.clone())
                                    .into_element(cx),
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(cancel_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    },
                )
        };

        let row = stack::hstack_build(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx, out| {
                let items = [
                    ("top", "Top", shadcn::SheetSide::Top, side_top_open.clone()),
                    (
                        "right",
                        "Right",
                        shadcn::SheetSide::Right,
                        side_right_open.clone(),
                    ),
                    (
                        "bottom",
                        "Bottom",
                        shadcn::SheetSide::Bottom,
                        side_bottom_open.clone(),
                    ),
                    (
                        "left",
                        "Left",
                        shadcn::SheetSide::Left,
                        side_left_open.clone(),
                    ),
                ];
                for (id, label, side, open_model) in items {
                    out.push(
                        cx.keyed(id, |cx| side_sheet(cx, id, label, side, open_model.clone())),
                    );
                }
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-side"),
        );

        let card = shell(cx, LayoutRefinement::default(), row);
        let body = centered(cx, card);
        section(cx, "Side", body)
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

        let card = shell(cx, LayoutRefinement::default(), sheet);
        let body = centered(cx, card);
        section(cx, "No Close Button", body)
    };

    let rtl = {
        let rtl_demo = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
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
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_demo);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Extends dialog to display side-aligned panels for supplementary tasks."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, side, no_close_button, rtl]
        }),
    ]
}
