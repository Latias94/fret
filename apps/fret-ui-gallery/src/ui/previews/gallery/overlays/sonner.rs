use super::super::super::super::*;
use super::command_palette::sonner_position_key;

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
