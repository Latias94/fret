pub const SOURCE: &str = include_str!("upstream_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::form::{FormRegistry, FormRegistryOptions, FormRevalidateMode};
use fret_ui_kit::headless::form_state::{FormState, FormValidateMode};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(384.0));

    let today = time::OffsetDateTime::now_utc().date();
    let min_dob = Date::from_calendar_date(1900, time::Month::January, 1).expect("valid date");
    let items_initial: Vec<Arc<str>> = vec![Arc::<str>::from("recents"), Arc::<str>::from("home")];

    let form_state = cx.local_model_keyed("form_state", || FormState {
        validate_mode: FormValidateMode::OnSubmit,
        ..FormState::default()
    });
    let username = cx.local_model_keyed("username", String::new);
    let email = cx.local_model_keyed("email", || None::<Arc<str>>);
    let email_open = cx.local_model_keyed("email_open", || false);
    let bio = cx.local_model_keyed("bio", String::new);
    let notify_type = cx.local_model_keyed("notify_type", || None::<Arc<str>>);
    let mobile = cx.local_model_keyed("mobile", || false);
    let sidebar_recents = cx.local_model_keyed("sidebar_recents", || true);
    let sidebar_home = cx.local_model_keyed("sidebar_home", || true);
    let sidebar_applications = cx.local_model_keyed("sidebar_applications", || false);
    let sidebar_desktop = cx.local_model_keyed("sidebar_desktop", || false);
    let sidebar_downloads = cx.local_model_keyed("sidebar_downloads", || false);
    let sidebar_documents = cx.local_model_keyed("sidebar_documents", || false);
    let sidebar_items = cx.local_model_keyed("sidebar_items", || items_initial.clone());
    let dob_open = cx.local_model_keyed("dob_open", || false);
    let dob_month = cx.local_model_keyed("dob_month", || CalendarMonth::from_date(today));
    let dob_selected = cx.local_model_keyed("dob_selected", || None::<Date>);
    let marketing_emails = cx.local_model_keyed("marketing_emails", || false);
    let security_emails = cx.local_model_keyed("security_emails", || true);

    let read_bool = |m: &Model<bool>| cx.app.models().read(m, |v| *v).ok().unwrap_or(false);
    let selected_items = {
        let mut items: Vec<Arc<str>> = Vec::new();
        if read_bool(&sidebar_recents) {
            items.push(Arc::<str>::from("recents"));
        }
        if read_bool(&sidebar_home) {
            items.push(Arc::<str>::from("home"));
        }
        if read_bool(&sidebar_applications) {
            items.push(Arc::<str>::from("applications"));
        }
        if read_bool(&sidebar_desktop) {
            items.push(Arc::<str>::from("desktop"));
        }
        if read_bool(&sidebar_downloads) {
            items.push(Arc::<str>::from("downloads"));
        }
        if read_bool(&sidebar_documents) {
            items.push(Arc::<str>::from("documents"));
        }
        items
    };

    let prev_items = cx
        .app
        .models()
        .read(&sidebar_items, |v| v.clone())
        .ok()
        .unwrap_or_default();
    if prev_items != selected_items {
        let next = selected_items.clone();
        let _ = cx
            .app
            .models_mut()
            .update(&sidebar_items, move |v| *v = next);
    }

    let registry = {
        let mut registry = FormRegistry::new().options(FormRegistryOptions {
            touch_on_change: true,
            revalidate_mode: FormRevalidateMode::OnChange,
        });

        registry.register_field("username", username.clone(), String::new(), |v| {
            let v = v.trim();
            if v.chars().count() < 2 {
                Some(Arc::from("Username must be at least 2 characters."))
            } else {
                None
            }
        });

        registry.register_field("email", email.clone(), None::<Arc<str>>, |v| {
            if v.is_none() {
                Some(Arc::from("Please select an email to display."))
            } else {
                None
            }
        });

        registry.register_field("bio", bio.clone(), String::new(), |v| {
            let v = v.trim();
            let len = v.chars().count();
            if len < 10 {
                Some(Arc::from("Bio must be at least 10 characters."))
            } else if len > 160 {
                Some(Arc::from("Bio must not be longer than 30 characters."))
            } else {
                None
            }
        });

        registry.register_field("type", notify_type.clone(), None::<Arc<str>>, |v| {
            if v.is_none() {
                Some(Arc::from("You need to select a notification type."))
            } else {
                None
            }
        });

        registry.register_field("mobile", mobile.clone(), false, |_v| None);

        registry.register_field("items", sidebar_items.clone(), items_initial.clone(), |v| {
            if v.is_empty() {
                Some(Arc::from("You have to select at least one item."))
            } else {
                None
            }
        });

        registry.register_field("dob", dob_selected.clone(), None::<Date>, |v| {
            if v.is_none() {
                Some(Arc::from("A date of birth is required."))
            } else {
                None
            }
        });

        registry.register_field("marketing_emails", marketing_emails.clone(), false, |_v| {
            None
        });
        registry.register_field("security_emails", security_emails.clone(), true, |_v| None);

        registry
    };

    registry.register_into_form_state(&mut *cx.app, &form_state);
    registry.handle_model_changes(
        &mut *cx.app,
        &form_state,
        &[
            username.id(),
            email.id(),
            bio.id(),
            notify_type.id(),
            mobile.id(),
            sidebar_items.id(),
            dob_selected.id(),
            marketing_emails.id(),
            security_emails.id(),
        ],
    );

    let submit = {
        let registry = registry.clone();
        let sonner = shadcn::Sonner::global(&mut *cx.app);
        let form_state = form_state.clone();
        let username = username.clone();
        let email = email.clone();
        let bio = bio.clone();
        let notify_type = notify_type.clone();
        let mobile = mobile.clone();
        let sidebar_items = sidebar_items.clone();
        let dob_selected = dob_selected.clone();
        let marketing_emails = marketing_emails.clone();
        let security_emails = security_emails.clone();

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let valid = registry.submit_action_host(host, &form_state);
            if !valid {
                host.request_redraw(action_cx.window);
                return;
            }

            fn read_model<T: std::any::Any, R>(
                host: &mut dyn fret_ui::action::UiActionHost,
                model: &Model<T>,
                f: impl FnOnce(&T) -> R,
            ) -> Option<R> {
                host.models_mut().read(model, f).ok()
            }

            let username = read_model(host, &username, |v| v.clone()).unwrap_or_default();
            let email = read_model(host, &email, |v| v.clone())
                .flatten()
                .map(|v| v.to_string());
            let bio = read_model(host, &bio, |v| v.clone()).unwrap_or_default();
            let notify_type = read_model(host, &notify_type, |v| v.clone())
                .flatten()
                .map(|v| v.to_string());

            let mobile = read_model(host, &mobile, |v| *v).unwrap_or(false);
            let items: Vec<String> = read_model(host, &sidebar_items, |v| v.clone())
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.to_string())
                .collect();
            let dob = read_model(host, &dob_selected, |v| *v)
                .flatten()
                .map(|v| v.to_string());
            let marketing_emails = read_model(host, &marketing_emails, |v| *v).unwrap_or(false);
            let security_emails = read_model(host, &security_emails, |v| *v).unwrap_or(true);

            let payload = serde_json::json!({
                "username": username,
                "email": email,
                "bio": bio,
                "type": notify_type,
                "mobile": mobile,
                "items": items,
                "dob": dob,
                "marketing_emails": marketing_emails,
                "security_emails": security_emails,
            });
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());

            sonner.toast_message(
                host,
                action_cx.window,
                "You submitted the following values:",
                shadcn::ToastMessageOptions::new().description(pretty),
            );
            host.request_redraw(action_cx.window);
        });

        shadcn::Button::new("Submit")
            .on_activate(on_activate)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-form-demo-submit")
    };

    let username_field = shadcn::FormField::new(
        form_state.clone(),
        "username",
        [shadcn::Input::new(username.clone())
            .placeholder("shadcn")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Username")
    .required(true)
    .description("This is your public display name.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-username");

    let email_field = shadcn::FormField::new(
        form_state.clone(),
        "email",
        [shadcn::Select::new(email.clone(), email_open.clone())
            .value(shadcn::SelectValue::new().placeholder("Select a verified email to display"))
            .item(shadcn::SelectItem::new("m@example.com", "m@example.com"))
            .item(shadcn::SelectItem::new("m@google.com", "m@google.com"))
            .item(shadcn::SelectItem::new("m@support.com", "m@support.com"))
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Email")
    .required(true)
    .description("You can manage email addresses in your email settings.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-email");

    let bio_field = shadcn::FormField::new(
        form_state.clone(),
        "bio",
        [shadcn::Textarea::new(bio.clone())
            .placeholder("Tell us a little bit about yourself")
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .h_px(Px(96.0)),
            )
            .into_element(cx)],
    )
    .label("Bio")
    .required(true)
    .description("You can @mention other users and organizations.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-bio");

    let notify_field = shadcn::FormField::new(
        form_state.clone(),
        "type",
        [shadcn::radio_group(
            notify_type.clone(),
            vec![
                shadcn::RadioGroupItem::new("all", "All new messages"),
                shadcn::RadioGroupItem::new("mentions", "Direct messages and mentions"),
                shadcn::RadioGroupItem::new("none", "Nothing"),
            ],
        )
        .a11y_label("Notify type")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)],
    )
    .label("Notify me about...")
    .required(true)
    .into_element(cx)
    .test_id("ui-gallery-form-demo-notify-type");

    let mobile_field = {
        let control_id = "ui-gallery-form-demo-mobile";
        let body = ui::h_flex(|cx| {
            let label = ui::v_flex(|cx| {
                vec![
                    shadcn::FieldLabel::new("Use different settings for my mobile devices")
                        .for_control(control_id)
                        .into_element(cx),
                    shadcn::raw::typography::muted(
                        "You can manage your mobile notifications in the mobile settings page.",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);
            vec![
                shadcn::Checkbox::new(mobile.clone())
                    .control_id(control_id)
                    .a11y_label("Use different settings for my mobile devices")
                    .into_element(cx),
                label,
            ]
        })
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-form-demo-mobile")
    };

    let sidebar_field = shadcn::FormField::new(
        form_state.clone(),
        "items",
        [ui::v_flex(|cx| {
            let header = ui::v_stack(|cx| {
                vec![
                    shadcn::raw::typography::large("Sidebar").into_element(cx),
                    shadcn::raw::typography::muted(
                        "Select the items you want to display in the sidebar.",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .into_element(cx);

            let item_row =
                |cx: &mut UiCx<'_>, model: Model<bool>, id: &'static str, label: &'static str| {
                    ui::h_flex(|cx| {
                        vec![
                            shadcn::Checkbox::new(model)
                                .control_id(id)
                                .a11y_label(label)
                                .into_element(cx),
                            shadcn::FieldLabel::new(label)
                                .for_control(id)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx)
                };

            let list = ui::v_flex(|cx| {
                vec![
                    item_row(
                        cx,
                        sidebar_recents.clone(),
                        "ui-gallery-form-demo-items-recents",
                        "Recents",
                    ),
                    item_row(
                        cx,
                        sidebar_home.clone(),
                        "ui-gallery-form-demo-items-home",
                        "Home",
                    ),
                    item_row(
                        cx,
                        sidebar_applications.clone(),
                        "ui-gallery-form-demo-items-applications",
                        "Applications",
                    ),
                    item_row(
                        cx,
                        sidebar_desktop.clone(),
                        "ui-gallery-form-demo-items-desktop",
                        "Desktop",
                    ),
                    item_row(
                        cx,
                        sidebar_downloads.clone(),
                        "ui-gallery-form-demo-items-downloads",
                        "Downloads",
                    ),
                    item_row(
                        cx,
                        sidebar_documents.clone(),
                        "ui-gallery-form-demo-items-documents",
                        "Documents",
                    ),
                ]
            })
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

            vec![header, list]
        })
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start()
        .into_element(cx)],
    )
    .decorate_control(false)
    .required(true)
    .into_element(cx)
    .test_id("ui-gallery-form-demo-sidebar");

    let dob_text = cx
        .app
        .models()
        .read(&dob_selected, |v| v.map(|d| d.to_string()))
        .ok()
        .flatten()
        .unwrap_or_else(|| "Pick a date".to_string());

    let dob_field = shadcn::FormField::new(
        form_state.clone(),
        "dob",
        [shadcn::Popover::from_open(dob_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element_with(
                cx,
                |cx| {
                    shadcn::Button::new(dob_text)
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(dob_open.clone())
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx)
                },
                |cx| {
                    let calendar = shadcn::Calendar::new(dob_month.clone(), dob_selected.clone())
                        .disabled_by({
                            let today = today;
                            move |d| d > today || d < min_dob
                        })
                        .into_element(cx)
                        .test_id("ui-gallery-form-demo-dob-calendar");

                    shadcn::PopoverContent::build(cx, |_cx| [calendar])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .into_element(cx)
                },
            )],
    )
    .label("Date of birth")
    .required(true)
    .description("Your date of birth is used to calculate your age.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-dob");

    let email_notifications = {
        let marketing_body = ui::h_flex(|cx| {
            let text = ui::v_stack(|cx| {
                vec![
                    shadcn::Label::new("Marketing emails").into_element(cx),
                    shadcn::raw::typography::muted(
                        "Receive emails about new products, features, and more.",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .layout(LayoutRefinement::default().min_w_0())
            .into_element(cx);
            vec![
                text,
                shadcn::Switch::new(marketing_emails.clone())
                    .a11y_label("Marketing emails")
                    .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_between()
        .into_element(cx);

        let marketing = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_content(|cx| ui::children![cx; marketing_body]),
            ]
        })
        .into_element(cx)
        .test_id("ui-gallery-form-demo-email-notify-marketing");

        let security_body = ui::h_flex(|cx| {
            let text = ui::v_stack(|cx| {
                vec![
                    shadcn::Label::new("Security emails").into_element(cx),
                    shadcn::raw::typography::muted("Receive emails about your account security.")
                        .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .layout(LayoutRefinement::default().min_w_0())
            .into_element(cx);
            vec![
                text,
                shadcn::Switch::new(security_emails.clone())
                    .a11y_label("Security emails")
                    .disabled(true)
                    .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_between()
        .into_element(cx);

        let security = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_content(|cx| ui::children![cx; security_body]),
            ]
        })
        .into_element(cx)
        .test_id("ui-gallery-form-demo-email-notify-security");

        ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::h3("Email Notifications").into_element(cx),
                ui::v_flex(|_cx| vec![marketing, security])
                    .gap(Space::N4)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start()
        .into_element(cx)
        .test_id("ui-gallery-form-demo-email-notifications")
    };

    ui::v_stack(|_cx| {
        vec![
            username_field,
            email_field,
            bio_field,
            notify_field,
            mobile_field,
            sidebar_field,
            dob_field,
            email_notifications,
            submit,
        ]
    })
    .gap(Space::N6)
    .items_start()
    .layout(max_w_sm)
    .into_element(cx)
}
// endregion: example
