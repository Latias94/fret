pub const SOURCE: &str = include_str!("upstream_demo.rs");

// region: example
use fret_app::App;
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_kit::declarative::form::{FormRegistry, FormRegistryOptions, FormRevalidateMode};
use fret_ui_kit::headless::form_state::{FormState, FormValidateMode};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

pub fn render(cx: &mut ElementContext<'_, App>, max_w_sm: LayoutRefinement) -> AnyElement {
    #[derive(Default)]
    struct FormDemoModels {
        form_state: Option<Model<FormState>>,
        username: Option<Model<String>>,
        email: Option<Model<Option<Arc<str>>>>,
        email_open: Option<Model<bool>>,
        bio: Option<Model<String>>,
        notify_type: Option<Model<Option<Arc<str>>>>,
        mobile: Option<Model<bool>>,
        sidebar_recents: Option<Model<bool>>,
        sidebar_home: Option<Model<bool>>,
        sidebar_applications: Option<Model<bool>>,
        sidebar_desktop: Option<Model<bool>>,
        sidebar_downloads: Option<Model<bool>>,
        sidebar_documents: Option<Model<bool>>,
        sidebar_items: Option<Model<Vec<Arc<str>>>>,
        dob_open: Option<Model<bool>>,
        dob_month: Option<Model<CalendarMonth>>,
        dob_selected: Option<Model<Option<Date>>>,
        marketing_emails: Option<Model<bool>>,
        security_emails: Option<Model<bool>>,
    }

    let (
        form_state,
        username,
        email,
        email_open,
        bio,
        notify_type,
        mobile,
        sidebar_recents,
        sidebar_home,
        sidebar_applications,
        sidebar_desktop,
        sidebar_downloads,
        sidebar_documents,
        sidebar_items,
        dob_open,
        dob_month,
        dob_selected,
        marketing_emails,
        security_emails,
    ) = cx.with_state(FormDemoModels::default, |st| {
        (
            st.form_state.clone(),
            st.username.clone(),
            st.email.clone(),
            st.email_open.clone(),
            st.bio.clone(),
            st.notify_type.clone(),
            st.mobile.clone(),
            st.sidebar_recents.clone(),
            st.sidebar_home.clone(),
            st.sidebar_applications.clone(),
            st.sidebar_desktop.clone(),
            st.sidebar_downloads.clone(),
            st.sidebar_documents.clone(),
            st.sidebar_items.clone(),
            st.dob_open.clone(),
            st.dob_month.clone(),
            st.dob_selected.clone(),
            st.marketing_emails.clone(),
            st.security_emails.clone(),
        )
    });

    let today = time::OffsetDateTime::now_utc().date();
    let min_dob = Date::from_calendar_date(1900, time::Month::January, 1).expect("valid date");
    let items_initial: Vec<Arc<str>> = vec![Arc::<str>::from("recents"), Arc::<str>::from("home")];

    let (
        form_state,
        username,
        email,
        email_open,
        bio,
        notify_type,
        mobile,
        sidebar_recents,
        sidebar_home,
        sidebar_applications,
        sidebar_desktop,
        sidebar_downloads,
        sidebar_documents,
        sidebar_items,
        dob_open,
        dob_month,
        dob_selected,
        marketing_emails,
        security_emails,
    ) = match (
        form_state,
        username,
        email,
        email_open,
        bio,
        notify_type,
        mobile,
        sidebar_recents,
        sidebar_home,
        sidebar_applications,
        sidebar_desktop,
        sidebar_downloads,
        sidebar_documents,
        sidebar_items,
        dob_open,
        dob_month,
        dob_selected,
        marketing_emails,
        security_emails,
    ) {
        (
            Some(form_state),
            Some(username),
            Some(email),
            Some(email_open),
            Some(bio),
            Some(notify_type),
            Some(mobile),
            Some(sidebar_recents),
            Some(sidebar_home),
            Some(sidebar_applications),
            Some(sidebar_desktop),
            Some(sidebar_downloads),
            Some(sidebar_documents),
            Some(sidebar_items),
            Some(dob_open),
            Some(dob_month),
            Some(dob_selected),
            Some(marketing_emails),
            Some(security_emails),
        ) => (
            form_state,
            username,
            email,
            email_open,
            bio,
            notify_type,
            mobile,
            sidebar_recents,
            sidebar_home,
            sidebar_applications,
            sidebar_desktop,
            sidebar_downloads,
            sidebar_documents,
            sidebar_items,
            dob_open,
            dob_month,
            dob_selected,
            marketing_emails,
            security_emails,
        ),
        _ => {
            let form_state = cx.app.models_mut().insert(FormState {
                validate_mode: FormValidateMode::OnSubmit,
                ..FormState::default()
            });
            let username = cx.app.models_mut().insert(String::new());
            let email = cx.app.models_mut().insert(None::<Arc<str>>);
            let email_open = cx.app.models_mut().insert(false);
            let bio = cx.app.models_mut().insert(String::new());
            let notify_type = cx.app.models_mut().insert(None::<Arc<str>>);
            let mobile = cx.app.models_mut().insert(false);

            let sidebar_recents = cx.app.models_mut().insert(true);
            let sidebar_home = cx.app.models_mut().insert(true);
            let sidebar_applications = cx.app.models_mut().insert(false);
            let sidebar_desktop = cx.app.models_mut().insert(false);
            let sidebar_downloads = cx.app.models_mut().insert(false);
            let sidebar_documents = cx.app.models_mut().insert(false);
            let sidebar_items = cx.app.models_mut().insert(items_initial.clone());

            let dob_open = cx.app.models_mut().insert(false);
            let dob_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let dob_selected = cx.app.models_mut().insert(None::<Date>);

            let marketing_emails = cx.app.models_mut().insert(false);
            let security_emails = cx.app.models_mut().insert(true);

            cx.with_state(FormDemoModels::default, |st| {
                st.form_state = Some(form_state.clone());
                st.username = Some(username.clone());
                st.email = Some(email.clone());
                st.email_open = Some(email_open.clone());
                st.bio = Some(bio.clone());
                st.notify_type = Some(notify_type.clone());
                st.mobile = Some(mobile.clone());
                st.sidebar_recents = Some(sidebar_recents.clone());
                st.sidebar_home = Some(sidebar_home.clone());
                st.sidebar_applications = Some(sidebar_applications.clone());
                st.sidebar_desktop = Some(sidebar_desktop.clone());
                st.sidebar_downloads = Some(sidebar_downloads.clone());
                st.sidebar_documents = Some(sidebar_documents.clone());
                st.sidebar_items = Some(sidebar_items.clone());
                st.dob_open = Some(dob_open.clone());
                st.dob_month = Some(dob_month.clone());
                st.dob_selected = Some(dob_selected.clone());
                st.marketing_emails = Some(marketing_emails.clone());
                st.security_emails = Some(security_emails.clone());
            });

            (
                form_state,
                username,
                email,
                email_open,
                bio,
                notify_type,
                mobile,
                sidebar_recents,
                sidebar_home,
                sidebar_applications,
                sidebar_desktop,
                sidebar_downloads,
                sidebar_documents,
                sidebar_items,
                dob_open,
                dob_month,
                dob_selected,
                marketing_emails,
                security_emails,
            )
        }
    };

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
    .description("This is your public display name.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-username");

    let email_field = shadcn::FormField::new(
        form_state.clone(),
        "email",
        [shadcn::Select::new(email.clone(), email_open.clone())
            .value(
                shadcn::SelectValue::new().placeholder("Select a verified email to display"),
            )
            .item(shadcn::SelectItem::new("m@example.com", "m@example.com"))
            .item(shadcn::SelectItem::new("m@google.com", "m@google.com"))
            .item(shadcn::SelectItem::new("m@support.com", "m@support.com"))
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)],
    )
    .label("Email")
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
    .description("You can @mention other users and organizations.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-bio");

    let notify_field = shadcn::FormField::new(
        form_state.clone(),
        "type",
        [shadcn::RadioGroup::new(notify_type.clone())
            .a11y_label("Notify type")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .item(shadcn::RadioGroupItem::new("all", "All new messages"))
            .item(shadcn::RadioGroupItem::new(
                "mentions",
                "Direct messages and mentions",
            ))
            .item(shadcn::RadioGroupItem::new("none", "Nothing"))
            .into_element(cx)],
    )
    .label("Notify me about...")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-notify-type");

    let mobile_field = {
        let control_id = "ui-gallery-form-demo-mobile";
        shadcn::Card::new(vec![shadcn::CardContent::new(vec![stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                let label = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N1)
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    |cx| {
                        vec![
                            shadcn::FieldLabel::new(
                                "Use different settings for my mobile devices",
                            )
                            .for_control(control_id)
                            .into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "You can manage your mobile notifications in the mobile settings page.",
                            ),
                        ]
                    },
                );
                vec![
                    shadcn::Checkbox::new(mobile.clone())
                        .control_id(control_id)
                        .a11y_label("Use different settings for my mobile devices")
                        .into_element(cx),
                    label,
                ]
            },
        )])
        .into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-form-demo-mobile")
    };

    let sidebar_field = shadcn::FormField::new(
        form_state.clone(),
        "items",
        [stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_start(),
            |cx| {
                let header = stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    |cx| {
                        vec![
                            shadcn::typography::large(cx, "Sidebar"),
                            shadcn::typography::muted(
                                cx,
                                "Select the items you want to display in the sidebar.",
                            ),
                        ]
                    },
                );

                let item_row = |cx: &mut ElementContext<'_, App>,
                                model: Model<bool>,
                                id: &'static str,
                                label: &'static str| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N3)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        |cx| {
                            vec![
                                shadcn::Checkbox::new(model)
                                    .control_id(id)
                                    .a11y_label(label)
                                    .into_element(cx),
                                shadcn::FieldLabel::new(label)
                                    .for_control(id)
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let list = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    |cx| {
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
                    },
                );

                vec![header, list]
            },
        )],
    )
    .decorate_control(false)
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
        [shadcn::Popover::new(dob_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
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

                    shadcn::PopoverContent::new([calendar])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .into_element(cx)
                },
            )],
    )
    .label("Date of birth")
    .description("Your date of birth is used to calculate your age.")
    .into_element(cx)
    .test_id("ui-gallery-form-demo-dob");

    let email_notifications = {
        let marketing = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .justify_between(),
                |cx| {
                    let text = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N1)
                            .layout(LayoutRefinement::default().min_w_0()),
                        |cx| {
                            vec![
                                shadcn::Label::new("Marketing emails").into_element(cx),
                                shadcn::typography::muted(
                                    cx,
                                    "Receive emails about new products, features, and more.",
                                ),
                            ]
                        },
                    );
                    vec![
                        text,
                        shadcn::Switch::new(marketing_emails.clone())
                            .a11y_label("Marketing emails")
                            .into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-form-demo-email-notify-marketing");

        let security = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .justify_between(),
                |cx| {
                    let text = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N1)
                            .layout(LayoutRefinement::default().min_w_0()),
                        |cx| {
                            vec![
                                shadcn::Label::new("Security emails").into_element(cx),
                                shadcn::typography::muted(
                                    cx,
                                    "Receive emails about your account security.",
                                ),
                            ]
                        },
                    );
                    vec![
                        text,
                        shadcn::Switch::new(security_emails.clone())
                            .a11y_label("Security emails")
                            .disabled(true)
                            .into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-form-demo-email-notify-security");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_start(),
            |cx| {
                vec![
                    shadcn::typography::h3(cx, "Email Notifications"),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .gap(Space::N4)
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        |_cx| vec![marketing, security],
                    ),
                ]
            },
        )
        .test_id("ui-gallery-form-demo-email-notifications")
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(max_w_sm),
        |_cx| {
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
        },
    )
}
// endregion: example
