use super::super::super::*;

pub(in crate::ui) fn preview_progress(
    cx: &mut ElementContext<'_, App>,
    _progress: Model<f32>,
) -> Vec<AnyElement> {
    use std::time::Duration;

    use fret_core::{SemanticsRole, TimerToken};
    use fret_runtime::Effect;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::primitives::direction as direction_prim;

    #[derive(Default, Clone)]
    struct ProgressModels {
        demo_value: Option<Model<f32>>,
        demo_token: Option<Model<Option<TimerToken>>>,
        label_value: Option<Model<f32>>,
        controlled_values: Option<Model<Vec<f32>>>,
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

    let state = cx.with_state(ProgressModels::default, |st| st.clone());

    let demo_value = match state.demo_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(13.0);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_value = Some(model.clone())
            });
            model
        }
    };

    let demo_token = match state.demo_token {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<TimerToken>);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_token = Some(model.clone())
            });
            model
        }
    };

    let label_value = match state.label_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(66.0);
            cx.with_state(ProgressModels::default, |st| {
                st.label_value = Some(model.clone())
            });
            model
        }
    };

    let controlled_values = match state.controlled_values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![50.0]);
            cx.with_state(ProgressModels::default, |st| {
                st.controlled_values = Some(model.clone())
            });
            model
        }
    };

    let demo = cx.keyed("ui_gallery.progress.demo", |cx| {
        let demo_value_for_timer = demo_value.clone();
        let demo_token_for_timer = demo_token.clone();

        let body = cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-progress-demo")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&demo_token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }
                        let _ = host
                            .models_mut()
                            .update(&demo_value_for_timer, |v| *v = 66.0);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let armed = cx
                    .get_model_copied(&demo_token, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                if !armed {
                    let token = cx.app.next_timer_token();
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&demo_token, |v| *v = Some(token));
                    let _ = cx.app.models_mut().update(&demo_value, |v| *v = 13.0);
                    cx.app.push_effect(Effect::SetTimer {
                        window: Some(cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });
                }

                let bar = shadcn::Progress::new(demo_value.clone())
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx);

                vec![centered(cx, bar)]
            },
        );

        section(cx, "Demo", body)
    });

    let label = cx.keyed("ui_gallery.progress.label", |cx| {
        let label_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Upload progress").into_element(cx),
                    shadcn::FieldLabel::new("66%")
                        .refine_layout(LayoutRefinement::default().ml_auto())
                        .into_element(cx),
                ]
            },
        );

        let field = shadcn::Field::new(vec![
            label_row,
            shadcn::Progress::new(label_value.clone()).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        let body = centered(cx, field);
        section(cx, "Label", body)
    });

    let controlled = cx.keyed("ui_gallery.progress.controlled", |cx| {
        let values = controlled_values.clone();
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    shadcn::Progress::new_values_first(values.clone()).into_element(cx),
                    shadcn::Slider::new(values)
                        .range(0.0, 100.0)
                        .step(1.0)
                        .a11y_label("Progress value")
                        .into_element(cx),
                ]
            },
        );

        let centered_body = centered(cx, body);
        section(cx, "Controlled", centered_body)
    });

    let rtl = cx.keyed("ui_gallery.progress.rtl", |cx| {
        let body = direction_prim::with_direction_provider(
            cx,
            direction_prim::LayoutDirection::Rtl,
            |cx| {
                let label_row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::FieldLabel::new("٦٦%").into_element(cx),
                            shadcn::FieldLabel::new("تقدم الرفع")
                                .refine_layout(LayoutRefinement::default().ml_auto())
                                .into_element(cx),
                        ]
                    },
                );

                let field = shadcn::Field::new(vec![
                    label_row,
                    shadcn::Progress::new(label_value.clone())
                        .mirror_in_rtl(true)
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                .into_element(cx);

                centered(cx, field)
            },
        );

        section(cx, "RTL", body)
    });

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![label, controlled, rtl],
    );

    vec![demo, examples]
}

pub(in crate::ui) fn preview_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_dropdown_menu(cx, open, last_action)
}

pub(in crate::ui) fn preview_menus(
    cx: &mut ElementContext<'_, App>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-dropdown-trigger")
                .toggle_model(dropdown_open.clone())
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-menus-dropdown-item-apple")
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-context-trigger")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-menus-context-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| [dropdown, context_menu],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

pub(in crate::ui) fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_context_menu(cx, open, last_action)
}

pub(in crate::ui) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_command_palette(cx, open, query, last_action)
}

fn sonner_position_key(position: shadcn::ToastPosition) -> &'static str {
    match position {
        shadcn::ToastPosition::TopLeft => "top-left",
        shadcn::ToastPosition::TopCenter => "top-center",
        shadcn::ToastPosition::TopRight => "top-right",
        shadcn::ToastPosition::BottomLeft => "bottom-left",
        shadcn::ToastPosition::BottomCenter => "bottom-center",
        shadcn::ToastPosition::BottomRight => "bottom-right",
    }
}

pub(in crate::ui) fn preview_sonner(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct SonnerModels {
        pending_promise: Option<Model<Option<shadcn::ToastId>>>,
    }

    let pending_promise = cx.with_state(SonnerModels::default, |st| st.pending_promise.clone());
    let sonner = shadcn::Sonner::global(&mut *cx.app);

    let pending_promise = match pending_promise {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<shadcn::ToastId>);
            cx.with_state(SonnerModels::default, |st| {
                st.pending_promise = Some(model.clone())
            });
            model
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

    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| children,
        )
    };

    let button = |cx: &mut ElementContext<'_, App>,
                  label: &'static str,
                  test_id: &'static str,
                  on_activate: fret_ui::action::OnActivate| {
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let demo = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast_message(
                host,
                action_cx.window,
                "Event has been created",
                shadcn::ToastMessageOptions::new()
                    .description("Sunday, December 03, 2023 at 9:00 AM")
                    .action("Undo", CMD_TOAST_ACTION),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.demo");
            });
            host.request_redraw(action_cx.window);
        });

        let show = button(cx, "Show Toast", "ui-gallery-sonner-demo-show", on_activate);
        let content = centered(cx, show).attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-demo"),
        );
        section(cx, "Demo", content)
    };

    let types = {
        let default_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.default");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Default",
                "ui-gallery-sonner-types-default",
                on_activate,
            )
        };

        let success_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_success_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.success");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Success",
                "ui-gallery-sonner-types-success",
                on_activate,
            )
        };

        let info_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_info_message(
                        host,
                        action_cx.window,
                        "Be at the area 10 minutes before the event time",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.info");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(cx, "Info", "ui-gallery-sonner-types-info", on_activate)
        };

        let warning_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_warning_message(
                        host,
                        action_cx.window,
                        "Event start time cannot be earlier than 8am",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.warning");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Warning",
                "ui-gallery-sonner-types-warning",
                on_activate,
            )
        };

        let error_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_error_message(
                        host,
                        action_cx.window,
                        "Event has not been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.error");
                    });
                    host.request_redraw(action_cx.window);
                });
            button(cx, "Error", "ui-gallery-sonner-types-error", on_activate)
        };

        let promise_button = {
            let sonner = sonner.clone();
            let pending_model = pending_promise.clone();
            let last_action_model = last_action.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let pending = host.models_mut().get_copied(&pending_model).flatten();
                    if let Some(id) = pending {
                        sonner.toast_success_update(
                            host,
                            action_cx.window,
                            id,
                            "Event has been created",
                        );
                        let _ = host
                            .models_mut()
                            .update(&pending_model, |slot| *slot = None);
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.resolve");
                        });
                    } else {
                        let promise = sonner.toast_promise(host, action_cx.window, "Loading...");
                        let _ = host
                            .models_mut()
                            .update(&pending_model, |slot| *slot = Some(promise.id()));
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.start");
                        });
                    }
                    host.request_redraw(action_cx.window);
                });
            button(
                cx,
                "Promise",
                "ui-gallery-sonner-types-promise",
                on_activate,
            )
        };

        let buttons_row = row(
            cx,
            vec![
                default_button,
                success_button,
                info_button,
                warning_button,
                error_button,
                promise_button,
            ],
        );

        let pending = cx
            .get_model_copied(&pending_promise, Invalidation::Layout)
            .flatten()
            .is_some();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    buttons_row,
                    shadcn::typography::muted(
                        cx,
                        if pending {
                            "Promise toast pending: click Promise again to resolve."
                        } else {
                            "Promise toast idle: click Promise to start loading state."
                        },
                    ),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-types"),
        );

        section(cx, "Types", content)
    };

    let description = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast_message(
                host,
                action_cx.window,
                "Event has been created",
                shadcn::ToastMessageOptions::new().description("Monday, January 3rd at 6:00pm"),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.description");
            });
            host.request_redraw(action_cx.window);
        });

        let show = button(
            cx,
            "Show Toast",
            "ui-gallery-sonner-description-show",
            on_activate,
        );
        let content = centered(cx, show).attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-description"),
        );

        section(cx, "Description", content)
    };

    let position = {
        let current = cx
            .get_model_copied(&sonner_position, Invalidation::Layout)
            .unwrap_or(shadcn::ToastPosition::TopCenter);

        let make_position_button =
            |cx: &mut ElementContext<'_, App>,
             label: &'static str,
             test_id: &'static str,
             target: shadcn::ToastPosition| {
                let sonner = sonner.clone();
                let position_model = sonner_position.clone();
                let last_action_model = last_action.clone();
                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&position_model, |v| *v = target);
                        sonner.toast_message(
                            host,
                            action_cx.window,
                            "Event has been created",
                            shadcn::ToastMessageOptions::new()
                                .description(format!("position: {}", sonner_position_key(target))),
                        );
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from(format!(
                                "sonner.position.{}",
                                sonner_position_key(target)
                            ));
                        });
                        host.request_redraw(action_cx.window);
                    });
                button(cx, label, test_id, on_activate)
            };

        let make_position_button = make_position_button;
        let top_left = make_position_button(
            cx,
            "Top Left",
            "ui-gallery-sonner-position-top-left",
            shadcn::ToastPosition::TopLeft,
        );
        let top_center = make_position_button(
            cx,
            "Top Center",
            "ui-gallery-sonner-position-top-center",
            shadcn::ToastPosition::TopCenter,
        );
        let top_right = make_position_button(
            cx,
            "Top Right",
            "ui-gallery-sonner-position-top-right",
            shadcn::ToastPosition::TopRight,
        );
        let bottom_left = make_position_button(
            cx,
            "Bottom Left",
            "ui-gallery-sonner-position-bottom-left",
            shadcn::ToastPosition::BottomLeft,
        );
        let bottom_center = make_position_button(
            cx,
            "Bottom Center",
            "ui-gallery-sonner-position-bottom-center",
            shadcn::ToastPosition::BottomCenter,
        );
        let bottom_right = make_position_button(
            cx,
            "Bottom Right",
            "ui-gallery-sonner-position-bottom-right",
            shadcn::ToastPosition::BottomRight,
        );

        let top_row = row(cx, vec![top_left, top_center, top_right]);
        let bottom_row = row(cx, vec![bottom_left, bottom_center, bottom_right]);
        let rows = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default()),
            move |_cx| vec![top_row, bottom_row],
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            move |cx| {
                vec![
                    centered(cx, rows),
                    shadcn::typography::muted(
                        cx,
                        format!("Current toaster position: {}", sonner_position_key(current)),
                    ),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-position"),
        );

        section(cx, "Position", content)
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        cx.text("An opinionated toast component for React."),
        cx.text(format!("last action: {last}")),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, types, description, position]
        }),
    ]
}

pub(in crate::ui) fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let deprecated_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Toast is deprecated").into_element(cx),
            shadcn::CardDescription::new(
                "The toast component is deprecated in shadcn/ui docs. Use Sonner instead.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![shadcn::typography::muted(
            cx,
            "This page intentionally keeps only the deprecation guidance to match upstream docs.",
        )])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Open Sonner page")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_NAV_SONNER)
                .test_id("ui-gallery-toast-open-sonner")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-toast-deprecated");

    let centered_card = centered(cx, deprecated_card);

    vec![
        cx.text("A succinct message that is displayed temporarily."),
        centered_card,
    ]
}

pub(in crate::ui) fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnDismissRequest;

    let last_action_status = {
        let last = cx
            .app
            .models()
            .get_cloned(&last_action)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let text = format!("last action: {last}");
        cx.text(text).test_id("ui-gallery-overlay-last-action")
    };

    let overlays =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let overlay_reset = {
                use fret_ui::action::OnActivate;

                let dropdown_open = dropdown_open.clone();
                let context_menu_open = context_menu_open.clone();
                let context_menu_edge_open = context_menu_edge_open.clone();
                let popover_open = popover_open.clone();
                let dialog_open = dialog_open.clone();
                let alert_dialog_open = alert_dialog_open.clone();
                let sheet_open = sheet_open.clone();
                let portal_geometry_popover_open = portal_geometry_popover_open.clone();
                let last_action = last_action.clone();

                let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
                    let _ = host.models_mut().update(&dropdown_open, |v| *v = false);
                    let _ = host.models_mut().update(&context_menu_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&context_menu_edge_open, |v| *v = false);
                    let _ = host.models_mut().update(&popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&alert_dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&sheet_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&portal_geometry_popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&last_action, |v| {
                        *v = Arc::<str>::from("overlay:reset");
                    });
                });

                shadcn::Button::new("Reset overlays")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-overlay-reset")
                    .on_activate(on_activate)
                    .into_element(cx)
            };

            let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone())
                .modal(false)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("DropdownMenu")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-dropdown-trigger")
                            .toggle_model(dropdown_open.clone())
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Apple")
                                    .test_id("ui-gallery-dropdown-item-apple")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("More")
                                    .test_id("ui-gallery-dropdown-item-more")
                                    .close_on_select(false)
                                    .submenu(vec![
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested action")
                                                .test_id("ui-gallery-dropdown-submenu-item-nested")
                                                .on_select(CMD_MENU_CONTEXT_ACTION),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested disabled")
                                                .disabled(true),
                                        ),
                                    ]),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Orange")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                            shadcn::DropdownMenuEntry::Separator,
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("ContextMenu (right click)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-context-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Action")
                                .test_id("ui-gallery-context-item-action")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Disabled").disabled(true),
                        ),
                    ]
                },
            );

            let context_menu_edge = shadcn::ContextMenu::new(context_menu_edge_open.clone())
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("ContextMenu (edge, right click)")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-context-trigger-edge")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Action")
                                    .test_id("ui-gallery-context-edge-item-action")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Separator,
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let underlay = shadcn::Button::new("Underlay (outside-press target)")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-gallery-overlay-underlay")
                .into_element(cx);

            let tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Tooltip (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Tooltip: hover intent + placement",
                )])
                .into_element(cx)
                .test_id("ui-gallery-tooltip-content"),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-arrow")
            .panel_test_id("ui-gallery-tooltip-panel")
            .open_delay_frames(10)
            .close_delay_frames(10)
            .side(shadcn::TooltipSide::Top)
            .into_element(cx);

            let hover_card = shadcn::HoverCard::new(
                shadcn::Button::new("HoverCard (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-hovercard-trigger")
                    .into_element(cx),
                shadcn::HoverCardContent::new(vec![
                    cx.text("HoverCard content (overlay-root)"),
                    cx.text("Move pointer from trigger to content."),
                ])
                .into_element(cx)
                .test_id("ui-gallery-hovercard-content"),
            )
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx);

            let popover_open_for_dismiss = popover_open.clone();
            let last_action_for_dismiss = last_action.clone();
            let popover_on_dismiss: OnDismissRequest = Arc::new(move |host, _cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&popover_open_for_dismiss, |open| *open = false);
                let _ = host.models_mut().update(&last_action_for_dismiss, |cur| {
                    *cur = Arc::<str>::from("popover:dismissed");
                });
            });

            let popover = shadcn::Popover::new(popover_open.clone())
                .auto_focus(true)
                .on_dismiss_request(Some(popover_on_dismiss))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Popover")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-trigger")
                            .toggle_model(popover_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        let open_dialog = shadcn::Button::new("Open dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-dialog-trigger")
                            .toggle_model(dialog_open.clone())
                            .into_element(cx);

                        let close = shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-popover-close")
                            .toggle_model(popover_open.clone())
                            .into_element(cx);

                        shadcn::PopoverContent::new(vec![
                            cx.text("Popover content"),
                            open_dialog,
                            close,
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-popover-content")
                    },
                );

            let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-trigger")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::DialogContent::new(vec![
                        shadcn::DialogHeader::new(vec![
                            shadcn::DialogTitle::new("Dialog").into_element(cx),
                            shadcn::DialogDescription::new("Escape / overlay click closes")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        {
                            let body = stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N2).layout(
                                    LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                ),
                                |cx| {
                                    (0..64)
                                        .map(|i| {
                                            cx.text(format!("Scrollable content line {}", i + 1))
                                        })
                                        .collect::<Vec<_>>()
                                },
                            );

                            shadcn::ScrollArea::new([body])
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .w_full()
                                        .h_px(Px(240.0))
                                        .min_w_0()
                                        .min_h_0(),
                                )
                                .viewport_test_id("ui-gallery-dialog-scroll-viewport")
                                .into_element(cx)
                        },
                        shadcn::DialogFooter::new(vec![
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-dialog-close")
                                .toggle_model(dialog_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Confirm")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-dialog-confirm")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-dialog-content")
                },
            );

            let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("AlertDialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-trigger")
                        .toggle_model(alert_dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::AlertDialogContent::new(vec![
                        shadcn::AlertDialogHeader::new(vec![
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "This is non-closable by overlay click.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::AlertDialogFooter::new(vec![
                            shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                                .test_id("ui-gallery-alert-dialog-cancel")
                                .into_element(cx),
                            shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
                                .test_id("ui-gallery-alert-dialog-action")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-alert-dialog-content")
                },
            );

            let sheet = shadcn::Sheet::new(sheet_open.clone())
                .side(shadcn::SheetSide::Right)
                .size(Px(360.0))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Sheet")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-sheet-trigger")
                            .toggle_model(sheet_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::SheetContent::new(vec![
                            shadcn::SheetHeader::new(vec![
                                shadcn::SheetTitle::new("Sheet").into_element(cx),
                                shadcn::SheetDescription::new("A modal side panel.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                            {
                                let body = stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N2).layout(
                                        LayoutRefinement::default().w_full().min_w_0().min_h_0(),
                                    ),
                                    |cx| {
                                        (0..96)
                                            .map(|i| cx.text(format!("Sheet body line {}", i + 1)))
                                            .collect::<Vec<_>>()
                                    },
                                );

                                shadcn::ScrollArea::new([body])
                                    .refine_layout(
                                        LayoutRefinement::default()
                                            .flex_1()
                                            .w_full()
                                            .min_w_0()
                                            .min_h_0(),
                                    )
                                    .viewport_test_id("ui-gallery-sheet-scroll-viewport")
                                    .into_element(cx)
                            },
                            shadcn::SheetFooter::new(vec![
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-sheet-close")
                                    .toggle_model(sheet_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                        .test_id("ui-gallery-sheet-content")
                    },
                );

            let portal_geometry = {
                let popover = shadcn::Popover::new(portal_geometry_popover_open.clone())
                    .side(shadcn::PopoverSide::Right)
                    .align(shadcn::PopoverAlign::Start)
                    .side_offset(Px(8.0))
                    .window_margin(Px(8.0))
                    .arrow(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Portal geometry (scroll + clamp)")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-portal-geometry-trigger")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            let close = shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-portal-geometry-popover-close")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx);

                            shadcn::PopoverContent::new(vec![
                                cx.text("Popover content (placement + clamp)"),
                                cx.text("Wheel-scroll the viewport while open."),
                                close,
                            ])
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(360.0)).h_px(Px(220.0)),
                            )
                            .into_element(cx)
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .test_id("ui-gallery-portal-geometry-popover-content"),
                            )
                        },
                    );

                let items = (1..=48)
                    .map(|i| cx.text(format!("Scroll item {i:02}")))
                    .collect::<Vec<_>>();

                let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
                    let mut out: Vec<AnyElement> = Vec::with_capacity(items.len() + 2);
                    out.push(popover);
                    out.extend(items);
                    out
                });

                let scroll = shadcn::ScrollArea::new(vec![body])
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)).h_px(Px(160.0)))
                    .into_element(cx);

                let scroll = scroll.attach_semantics(
                    SemanticsDecoration::default()
                        .test_id("ui-gallery-portal-geometry-scroll-area"),
                );

                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Portal geometry").into_element(cx),
                        shadcn::CardDescription::new(
                            "Validates floating placement under scroll + window clamp.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![scroll]).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
            };

            let body = stack::vstack(
                cx,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let gap = cx.with_theme(|theme| {
                        fret_ui_kit::MetricRef::space(Space::N2).resolve(theme)
                    });

                    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: true,
                            },
                            |_cx| children,
                        )
                    };

                    let row_end = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = cx.with_theme(|theme| {
                            decl_style::layout_style(
                                theme,
                                LayoutRefinement::default().w_full().min_w_0(),
                            )
                        });
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::End,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| children,
                        )
                    };

                    vec![
                        row(cx, vec![dropdown, context_menu, overlay_reset]),
                        row_end(cx, vec![context_menu_edge]),
                        row(cx, vec![tooltip, hover_card, popover, underlay, dialog]),
                        row(cx, vec![alert_dialog, sheet]),
                        portal_geometry,
                    ]
                },
            );

            vec![body]
        });

    let dialog_open_flag = {
        let open = cx
            .get_model_copied(&dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(cx.text("Dialog open").test_id("ui-gallery-dialog-open"))
        } else {
            None
        }
    };

    let alert_dialog_open_flag = {
        let open = cx
            .get_model_copied(&alert_dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(
                cx.text("AlertDialog open")
                    .test_id("ui-gallery-alert-dialog-open"),
            )
        } else {
            None
        }
    };

    let popover_dismissed_flag = {
        let last = cx
            .get_model_cloned(&last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        if last.as_ref() == "popover:dismissed" {
            Some(
                cx.text("Popover dismissed")
                    .test_id("ui-gallery-popover-dismissed"),
            )
        } else {
            None
        }
    };

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];

    if let Some(flag) = popover_dismissed_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_open_flag {
        out.push(flag);
    }
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }

    out
}
