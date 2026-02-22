use super::super::super::super::*;
use super::command_palette::sonner_position_key;

use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_sonner(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SonnerModels {
        pending_promise: Option<Model<Option<shadcn::ToastId>>>,
        active_type: Option<Model<Arc<str>>>,
    }

    let sonner = shadcn::Sonner::global(&mut *cx.app);

    let pending_promise =
        match cx.with_state(SonnerModels::default, |st| st.pending_promise.clone()) {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(None::<shadcn::ToastId>);
                cx.with_state(SonnerModels::default, |st| {
                    st.pending_promise = Some(model.clone())
                });
                model
            }
        };

    let active_type = match cx.with_state(SonnerModels::default, |st| st.active_type.clone()) {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("Default"));
            cx.with_state(SonnerModels::default, |st| {
                st.active_type = Some(model.clone())
            });
            model
        }
    };

    let current_active = cx
        .get_model_cloned(&active_type, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("Default"));
    let theme = Theme::global(&*cx.app).clone();

    let action_button = |cx: &mut ElementContext<'_, App>,
                         label: &'static str,
                         variant: shadcn::ButtonVariant,
                         test_id: &'static str,
                         on_activate: fret_ui::action::OnActivate| {
        shadcn::Button::new(label)
            .variant(variant)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let type_button = |cx: &mut ElementContext<'_, App>,
                       label: &'static str,
                       test_id: &'static str,
                       on_activate: fret_ui::action::OnActivate| {
        let active = current_active.as_ref() == label;
        let mut button = shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Ghost)
            .on_activate(on_activate)
            .test_id(test_id);

        if active {
            let bg = shadcn::ColorRef::Token {
                key: "accent",
                fallback: fret_ui_kit::ColorFallback::ThemeHoverBackground,
            };
            let fg = shadcn::ColorRef::Token {
                key: "accent-foreground",
                fallback: fret_ui_kit::ColorFallback::ThemeTextPrimary,
            };
            button = button.style(
                shadcn::button::ButtonStyle::default()
                    .background(fret_ui_kit::WidgetStateProperty::new(Some(bg)))
                    .foreground(fret_ui_kit::WidgetStateProperty::new(Some(fg))),
            );
        }

        button.into_element(cx)
    };

    let demo = {
        let give_me = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "My first toast",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Default");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.demo.give_me");
                    });
                    host.request_redraw(action_cx.window);
                });
            action_button(
                cx,
                "Give me a toast",
                shadcn::ButtonVariant::Outline,
                "ui-gallery-sonner-demo-give-me",
                on_activate,
            )
        };

        let show = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new()
                            .description("Sunday, December 03, 2023 at 9:00 AM")
                            .action("Undo", CMD_TOAST_ACTION),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Action");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.demo.show");
                    });
                    host.request_redraw(action_cx.window);
                });
            action_button(
                cx,
                "Show Toast",
                shadcn::ButtonVariant::Outline,
                "ui-gallery-sonner-demo-show",
                on_activate,
            )
        };

        let default_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Default");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.default");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(
                cx,
                "Default",
                "ui-gallery-sonner-types-default",
                on_activate,
            )
        };

        let description_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new()
                            .description("Monday, January 3rd at 6:00pm"),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Description");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.description");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(
                cx,
                "Description",
                "ui-gallery-sonner-types-description",
                on_activate,
            )
        };

        let success_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_success_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Success");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.success");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(
                cx,
                "Success",
                "ui-gallery-sonner-types-success",
                on_activate,
            )
        };

        let info_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_info_message(
                        host,
                        action_cx.window,
                        "Be at the area 10 minutes before the event time",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Info");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.info");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(cx, "Info", "ui-gallery-sonner-types-info", on_activate)
        };

        let warning_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_warning_message(
                        host,
                        action_cx.window,
                        "Event start time cannot be earlier than 8am",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Warning");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.warning");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(
                cx,
                "Warning",
                "ui-gallery-sonner-types-warning",
                on_activate,
            )
        };

        let error_button = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_error_message(
                        host,
                        action_cx.window,
                        "Event has not been created",
                        shadcn::ToastMessageOptions::new(),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Error");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.error");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(cx, "Error", "ui-gallery-sonner-types-error", on_activate)
        };

        let action = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new().action("Undo", CMD_TOAST_ACTION),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Action");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.action");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(cx, "Action", "ui-gallery-sonner-types-action", on_activate)
        };

        let cancel = {
            let sonner = sonner.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    sonner.toast_message(
                        host,
                        action_cx.window,
                        "Event has been created",
                        shadcn::ToastMessageOptions::new().cancel("Cancel", CMD_TOAST_ACTION),
                    );
                    let _ = host.models_mut().update(&active_type_model, |v| {
                        *v = Arc::<str>::from("Cancel");
                    });
                    let _ = host.models_mut().update(&last_action_model, |v| {
                        *v = Arc::<str>::from("sonner.types.cancel");
                    });
                    host.request_redraw(action_cx.window);
                });
            type_button(cx, "Cancel", "ui-gallery-sonner-types-cancel", on_activate)
        };

        let promise = {
            let sonner = sonner.clone();
            let pending_model = pending_promise.clone();
            let last_action_model = last_action.clone();
            let active_type_model = active_type.clone();
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
                        let _ = host.models_mut().update(&active_type_model, |v| {
                            *v = Arc::<str>::from("Promise");
                        });
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.resolve");
                        });
                    } else {
                        let promise = sonner.toast_promise(host, action_cx.window, "Loading...");
                        let _ = host
                            .models_mut()
                            .update(&pending_model, |slot| *slot = Some(promise.id()));
                        let _ = host.models_mut().update(&active_type_model, |v| {
                            *v = Arc::<str>::from("Promise");
                        });
                        let _ = host.models_mut().update(&last_action_model, |v| {
                            *v = Arc::<str>::from("sonner.types.promise.start");
                        });
                    }
                    host.request_redraw(action_cx.window);
                });
            type_button(
                cx,
                "Promise",
                "ui-gallery-sonner-types-promise",
                on_activate,
            )
        };

        let pending = cx
            .get_model_copied(&pending_promise, Invalidation::Layout)
            .flatten()
            .is_some();

        let buttons = doc_layout::wrap_row(
            cx,
            &theme,
            Space::N2,
            fret_ui::element::CrossAlign::Center,
            |_cx| {
                vec![
                    give_me,
                    show,
                    default_button,
                    description_button,
                    success_button,
                    info_button,
                    warning_button,
                    error_button,
                    action,
                    cancel,
                    promise,
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-demo"),
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    buttons,
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
        .test_id("ui-gallery-sonner-demo-root")
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
                action_button(
                    cx,
                    label,
                    shadcn::ButtonVariant::Outline,
                    test_id,
                    on_activate,
                )
            };

        let top_row = doc_layout::wrap_controls_row(cx, &theme, Space::N2, |cx| {
            vec![
                make_position_button(
                    cx,
                    "Top Left",
                    "ui-gallery-sonner-position-top-left",
                    shadcn::ToastPosition::TopLeft,
                ),
                make_position_button(
                    cx,
                    "Top Center",
                    "ui-gallery-sonner-position-top-center",
                    shadcn::ToastPosition::TopCenter,
                ),
                make_position_button(
                    cx,
                    "Top Right",
                    "ui-gallery-sonner-position-top-right",
                    shadcn::ToastPosition::TopRight,
                ),
            ]
        });

        let bottom_row = doc_layout::wrap_controls_row(cx, &theme, Space::N2, |cx| {
            vec![
                make_position_button(
                    cx,
                    "Bottom Left",
                    "ui-gallery-sonner-position-bottom-left",
                    shadcn::ToastPosition::BottomLeft,
                ),
                make_position_button(
                    cx,
                    "Bottom Center",
                    "ui-gallery-sonner-position-bottom-center",
                    shadcn::ToastPosition::BottomCenter,
                ),
                make_position_button(
                    cx,
                    "Bottom Right",
                    "ui-gallery-sonner-position-bottom-right",
                    shadcn::ToastPosition::BottomRight,
                ),
            ]
        });

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    top_row,
                    bottom_row,
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
        )
        .test_id("ui-gallery-sonner-position-root")
    };

    let extras = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast(
                host,
                action_cx.window,
                shadcn::ToastRequest::new("Swipe to dismiss")
                    .description("Drag up to dismiss (pinned)")
                    .duration(None)
                    .dismissible(true)
                    .test_id("ui-gallery-sonner-demo-toast-swipe"),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from("sonner.extras.swipe_dismiss");
            });
            host.request_redraw(action_cx.window);
        });

        let swipe = action_button(
            cx,
            "Swipe Dismiss Toast",
            shadcn::ButtonVariant::Outline,
            "ui-gallery-sonner-demo-show-swipe",
            on_activate,
        );

        doc_layout::wrap_controls_row(cx, &theme, Space::N2, |_cx| vec![swipe]).attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-extras"),
        )
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                doc_layout::muted_full_width(cx, format!("Last action: {last}")),
                doc_layout::muted_full_width(
                    cx,
                    "Preview follows `sonner-demo.tsx` (new-york-v4): buttons that trigger different toast types.",
                ),
                doc_layout::muted_full_width(
                    cx,
                    "Fret exposes extra knobs (position, pinned + swipe dismiss) for testing overlay behavior.",
                ),
                doc_layout::muted_full_width(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/sonner.rs`.",
                ),
            ]
        },
    );

    let setup = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                doc_layout::muted_full_width(cx, "Mount a `Toaster` once per window."),
                doc_layout::muted_full_width(
                    cx,
                    "This installs the toast overlay layer and drives default styling + icons.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("An opinionated toast component (Sonner)."),
        vec![
            DocSection::new("Setup", setup)
                .description("Mount a toaster layer in your window root.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"// In your window root view:
shadcn::Toaster::new()
    .position(shadcn::ToastPosition::TopCenter)
    .shadcn_lucide_icons()
    .into_element(cx);"#,
                ),
            DocSection::new("Demo", demo)
                .description("Buttons that fire different toast styles and actions.")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"let sonner = shadcn::Sonner::global(&mut *cx.app);

sonner.toast_message(host, window, "Event has been created", ToastMessageOptions::new());
sonner.toast_message(
    host,
    window,
    "Event has been created",
    ToastMessageOptions::new().description("Monday, January 3rd at 6:00pm"),
);
sonner.toast_success_message(host, window, "Event has been created", ToastMessageOptions::new());
sonner.toast_info_message(
    host,
    window,
    "Be at the area 10 minutes before the event time",
    ToastMessageOptions::new(),
);
sonner.toast_warning_message(
    host,
    window,
    "Event start time cannot be earlier than 8am",
    ToastMessageOptions::new(),
);
sonner.toast_error_message(host, window, "Event has not been created", ToastMessageOptions::new());
sonner.toast_message(
    host,
    window,
    "Event has been created",
    ToastMessageOptions::new().action("Undo", CMD_TOAST_ACTION),
);
sonner.toast_message(
    host,
    window,
    "Event has been created",
    ToastMessageOptions::new().cancel("Cancel", CMD_TOAST_ACTION),
);

let promise = sonner.toast_promise(host, window, "Loading...");
promise.success(host, "Sonner toast has been added");"#,
                ),
            DocSection::new("Position", position)
                .description(
                    "Fret-specific: mutate global toaster position for overlay regression tests.",
                )
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"let _ = host.models_mut().update(&sonner_position, |v| {
    *v = shadcn::ToastPosition::BottomRight
});"#,
                ),
            DocSection::new("Extras", extras)
                .description("Pinned + swipe-dismiss toasts.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"sonner.toast(
    host,
    window,
    ToastRequest::new("Swipe to dismiss")
        .duration(None)
        .dismissible(true),
);"#,
                ),
            DocSection::new("Notes", notes).description("Status + parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-sonner")]
}
