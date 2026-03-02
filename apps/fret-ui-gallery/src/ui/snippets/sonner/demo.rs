// region: example
use fret_core::{Axis, Edges};
use fret_ui::element::{FlexProps, LayoutStyle, Length, SemanticsDecoration};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    pending_promise: Option<Model<Option<shadcn::ToastId>>>,
    active_type: Option<Model<Arc<str>>>,
}

fn wrap_controls_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let gap = fret_ui_kit::MetricRef::space(gap).resolve(theme);

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Center,
            wrap: true,
        },
        |_cx| children,
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, last_action: Model<Arc<str>>) -> AnyElement {
    // Keep the action id in one place so UI Gallery's driver can record it.
    // Replace this with your app's command id.
    const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";

    let sonner = shadcn::Sonner::global(&mut *cx.app);

    let (pending_promise, active_type) =
        cx.with_state(Models::default, |st| (st.pending_promise.clone(), st.active_type.clone()));

    let pending_promise = match pending_promise {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<shadcn::ToastId>);
            cx.with_state(Models::default, |st| st.pending_promise = Some(model.clone()));
            model
        }
    };

    let active_type = match active_type {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("Default"));
            cx.with_state(Models::default, |st| st.active_type = Some(model.clone()));
            model
        }
    };

    let current_active = cx
        .get_model_cloned(&active_type, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("Default"));

    let action_button = |cx: &mut ElementContext<'_, H>,
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

    let type_button = |cx: &mut ElementContext<'_, H>,
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

    let give_me = {
        let sonner = sonner.clone();
        let last_action_model = last_action.clone();
        let active_type_model = active_type.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            sonner.toast_message(
                host,
                action_cx.window,
                "Event has been created",
                shadcn::ToastMessageOptions::new().description("Monday, January 3rd at 6:00pm"),
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
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
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let pending = host.models_mut().get_copied(&pending_model).flatten();
            if let Some(id) = pending {
                sonner.toast_success_update(host, action_cx.window, id, "Event has been created");
                let _ = host.models_mut().update(&pending_model, |slot| *slot = None);
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

    let buttons = wrap_controls_row(
        cx,
        Space::N2,
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
        ],
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
}
// endregion: example
