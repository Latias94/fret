use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_ui_kit::declarative::style as decl_style;
use ui_assets::ui::ImageSourceElementContextExt as _;

pub(super) fn preview_card(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    #[derive(Default)]
    struct CardModels {
        email: Option<Model<String>>,
        password: Option<Model<String>>,
    }

    let email = cx.with_state(CardModels::default, |st| st.email.clone());
    let email = match email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(CardModels::default, |st| st.email = Some(model.clone()));
            model
        }
    };

    let password = cx.with_state(CardModels::default, |st| st.password.clone());
    let password = match password {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(CardModels::default, |st| st.password = Some(model.clone()));
            model
        }
    };

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let login = {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-title"),
                shadcn::CardDescription::new("Enter your email below to login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-description"),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let email =
                        stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
                            vec![
                                shadcn::Label::new("Email").into_element(cx),
                                shadcn::Input::new(email.clone())
                                    .a11y_label("Email")
                                    .placeholder("m@example.com")
                                    .into_element(cx),
                            ]
                        });

                    let password =
                        stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |cx| {
                            vec![
                                stack::hstack(
                                    cx,
                                    stack::HStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .justify_between()
                                        .items_center(),
                                    |cx| {
                                        vec![
                                            shadcn::Label::new("Password").into_element(cx),
                                            shadcn::Button::new("Forgot your password?")
                                                .variant(shadcn::ButtonVariant::Link)
                                                .size(shadcn::ButtonSize::Sm)
                                                .into_element(cx),
                                        ]
                                    },
                                ),
                                shadcn::Input::new(password.clone())
                                    .a11y_label("Password")
                                    .placeholder("••••••••")
                                    .into_element(cx),
                            ]
                        });

                    vec![email, password]
                },
            )])
            .into_element(cx),
            shadcn::CardFooter::new(vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let sign_up = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().mt(Space::N4))
                            .items_center()
                            .justify_center()
                            .gap(Space::N1),
                        |cx| {
                            vec![
                                ui::text(cx, "Don't have an account?")
                                    .text_sm()
                                    .into_element(cx),
                                shadcn::Button::new("Sign up")
                                    .variant(shadcn::ButtonVariant::Link)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx),
                            ]
                        },
                    );
                    vec![
                        shadcn::Button::new("Login")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        shadcn::Button::new("Login with Google")
                            .variant(shadcn::ButtonVariant::Outline)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        sign_up,
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-card-demo")
    };

    let meeting_notes = {
        let avatars = {
            // Upstream shadcn Card docs use GitHub avatar images. In the in-tree gallery we prefer
            // a deterministic in-repo texture to avoid network dependency in diagnostics runs.
            let avatar_source = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    static AVATAR_TEST_JPG: OnceLock<Option<ui_assets::ImageSource>> =
                        OnceLock::new();
                    AVATAR_TEST_JPG.get_or_init(|| {
                        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join("../../assets/textures/test.jpg");
                        if path.exists() {
                            Some(ui_assets::ImageSource::from_path(Arc::new(path)))
                        } else {
                            None
                        }
                    })
                }

                #[cfg(target_arch = "wasm32")]
                {
                    static AVATAR_TEST_JPG: OnceLock<ui_assets::ImageSource> = OnceLock::new();
                    Some(AVATAR_TEST_JPG.get_or_init(|| {
                        ui_assets::ImageSource::from_url(Arc::<str>::from("textures/test.jpg"))
                    }))
                }
            };

            let avatar_fallbacks = ["CN", "LR", "ER"];
            let avatars = avatar_fallbacks
                .iter()
                .map(|fallback| {
                    let image_id = avatar_source
                        .as_ref()
                        .map(|source| cx.use_image_source_state(source).image)
                        .flatten();
                    let image = shadcn::AvatarImage::maybe(image_id).into_element(cx);
                    let fallback = shadcn::AvatarFallback::new(*fallback)
                        .when_image_missing(image_id)
                        .delay_ms(120)
                        .into_element(cx);
                    shadcn::Avatar::new([image, fallback]).into_element(cx)
                })
                .collect::<Vec<_>>();

            let count = shadcn::AvatarGroupCount::new([ui::text(cx, "+8")
                .font_medium()
                .nowrap()
                .into_element(cx)])
            .into_element(cx)
            .test_id("ui-gallery-card-notes-avatar-count");

            shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
                .into_element(cx)
                .test_id("ui-gallery-card-notes-avatars")
        };

        let list = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let paragraph = ui::text(
                    cx,
                    "Client requested dashboard redesign with focus on mobile responsiveness.",
                )
                .text_sm()
                .into_element(cx);

                let ordered_list = {
                    let props = decl_style::container_props(
                        &theme,
                        // shadcn/ui v4 uses `ol.list-decimal.pl-6` where the marker lives outside the
                        // text flow. Model that by giving the marker its own column instead of
                        // prefixing `"1."` into the item string (which double-counts marker width and
                        // causes earlier wraps like "Dark mode" / "support").
                        ChromeRefinement::default().pt(Space::N4),
                        LayoutRefinement::default().w_full(),
                    );

                    cx.container(props, |cx| {
                        vec![stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N2)
                                .items_stretch()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                let marker = |cx: &mut ElementContext<'_, App>, text: &str| {
                                    ui::text(cx, text)
                                        .text_sm()
                                        // `pl-6` ≈ marker(16px) + gap(8px).
                                        .w_space(Space::N4)
                                        .text_align(fret_core::TextAlign::End)
                                        .into_element(cx)
                                };

                                let item =
                                    |cx: &mut ElementContext<'_, App>,
                                     n: &str,
                                     content: &str,
                                     test_id: Option<&'static str>| {
                                        let row = stack::hstack(
                                            cx,
                                            stack::HStackProps::default()
                                                .gap(Space::N2)
                                                .items_start()
                                                .layout(LayoutRefinement::default().w_full()),
                                            |cx| {
                                                vec![
                                                    marker(cx, n),
                                                    ui::text(cx, content)
                                                        .text_sm()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .into_element(cx),
                                                ]
                                            },
                                        );

                                        match test_id {
                                            Some(id) => row.test_id(id),
                                            None => row,
                                        }
                                    };

                                vec![
                                    item(
                                        cx,
                                        "1.",
                                        "New analytics widgets for daily/weekly metrics",
                                        None,
                                    ),
                                    item(cx, "2.", "Simplified navigation menu", None),
                                    item(
                                        cx,
                                        "3.",
                                        "Dark mode support",
                                        Some("ui-gallery-card-notes-item-dark-mode"),
                                    ),
                                    item(cx, "4.", "Timeline: 6 weeks", None),
                                    item(
                                        cx,
                                        "5.",
                                        "Follow-up meeting scheduled for next Tuesday",
                                        None,
                                    ),
                                ]
                            },
                        )]
                    })
                };

                vec![paragraph, ordered_list]
            },
        )
        .test_id("ui-gallery-card-notes-list");

        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Meeting Notes").into_element(cx),
                shadcn::CardDescription::new("Transcript from the meeting with the client.")
                    .into_element(cx),
                shadcn::CardAction::new([shadcn::Button::new("Transcribe")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .leading_icon(fret_icons::IconId::new_static("lucide.captions"))
                    .into_element(cx)
                    .test_id("ui-gallery-card-notes-transcribe")])
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![list]).into_element(cx),
            shadcn::CardFooter::new(vec![avatars]).into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-card-meeting-notes")
    };

    let image = {
        let cover_bg = theme.color_token("muted");
        let cover_stack = {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default().bg(ColorRef::Color(cover_bg)),
                LayoutRefinement::default().relative().size_full(),
            );

            cx.container(props, |cx| {
                static DEBUG_IMAGE_LOADING: OnceLock<bool> = OnceLock::new();
                let debug_image_loading = *DEBUG_IMAGE_LOADING.get_or_init(|| {
                    std::env::var_os("FRET_UI_GALLERY_DEBUG_IMAGE_LOADING")
                        .is_some_and(|v| !v.is_empty())
                });

                let event_cover_fallback = cx.watch_model(&event_cover_image).copied().flatten();

                #[cfg(not(target_arch = "wasm32"))]
                let (event_cover, event_cover_state, event_cover_path_exists) = {
                    static EVENT_COVER_TEST_JPG: OnceLock<Option<ui_assets::ImageSource>> =
                        OnceLock::new();
                    let source = EVENT_COVER_TEST_JPG.get_or_init(|| {
                        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                            .join("../../assets/textures/test.jpg");
                        if path.exists() {
                            Some(ui_assets::ImageSource::from_path(Arc::new(path)))
                        } else {
                            None
                        }
                    });
                    let (state, image) = source.as_ref().map_or((None, None), |source| {
                        let state = cx.use_image_source_state(source);
                        let image = state.image;
                        (Some(state), image)
                    });
                    let path_exists = source.is_some();

                    (image.or(event_cover_fallback), state, path_exists)
                };

                #[cfg(target_arch = "wasm32")]
                let (event_cover, event_cover_state, event_cover_path_exists) = {
                    static EVENT_COVER_TEST_JPG: OnceLock<ui_assets::ImageSource> = OnceLock::new();
                    let source = EVENT_COVER_TEST_JPG.get_or_init(|| {
                        ui_assets::ImageSource::from_url(Arc::<str>::from("textures/test.jpg"))
                    });
                    let state = cx.use_image_source_state(source);
                    let image = state.image;
                    (image.or(event_cover_fallback), Some(state), true)
                };

                let image = shadcn::MediaImage::maybe(event_cover)
                    .loading(true)
                    .refine_layout(LayoutRefinement::default().size_full())
                    .into_element(cx)
                    .test_id("ui-gallery-card-image-event-cover-image");

                let overlay_props = {
                    decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().bg(ColorRef::Color(CoreColor {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.35,
                        })),
                        LayoutRefinement::default()
                            .absolute()
                            .inset(Space::N0)
                            .size_full(),
                    )
                };

                let overlay = cx
                    .container(overlay_props, |_cx| Vec::new())
                    .test_id("ui-gallery-card-image-event-cover-overlay");

                let debug_overlay = if debug_image_loading
                    || event_cover_state.as_ref().and_then(|s| s.error.as_deref()).is_some()
                {
                    let status = event_cover_state
                        .as_ref()
                        .map(|s| format!("{:?}", s.status))
                        .unwrap_or_else(|| "<no-state>".to_string());
                    let intrinsic = event_cover_state
                        .as_ref()
                        .and_then(|s| s.intrinsic_size_px)
                        .map(|(w, h)| format!("{w}x{h}"))
                        .unwrap_or_else(|| "-".to_string());
                    let has_image = event_cover_state
                        .as_ref()
                        .map(|s| s.image.is_some())
                        .unwrap_or(false);
                    let error = event_cover_state
                        .as_ref()
                        .and_then(|s| s.error.as_deref())
                        .unwrap_or("-");

                    let text: Arc<str> = Arc::from(format!(
                        "event_cover: status={status} image={has_image} intrinsic={intrinsic} path_exists={event_cover_path_exists} err={error}"
                    ));
                    Some(
                        shadcn::Badge::new(text)
                            .variant(shadcn::BadgeVariant::Secondary)
                            .refine_layout(
                                LayoutRefinement::default()
                                    .absolute()
                                    .left(Space::N2)
                                    .bottom(Space::N2),
                            )
                            .into_element(cx)
                            .test_id("ui-gallery-card-image-event-cover-debug"),
                    )
                } else {
                    None
                };

                let mut out = vec![image, overlay];
                if let Some(debug_overlay) = debug_overlay {
                    out.push(debug_overlay);
                }
                out
            })
            .test_id("ui-gallery-card-image-event-cover-stack")
        };

        let cover = shadcn::AspectRatio::new(16.0 / 9.0, cover_stack)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-card-image-event-cover");

        let badge =
            |cx: &mut ElementContext<'_, App>, icon_id: &'static str, text: &'static str| {
                shadcn::Badge::new("")
                    .variant(shadcn::BadgeVariant::Outline)
                    .children([
                        doc_layout::icon(cx, icon_id),
                        ui::text(cx, text)
                            .nowrap()
                            .into_element(cx)
                            .test_id(format!("ui-gallery-card-image-badge-{text}")),
                    ])
                    .into_element(cx)
            };

        let footer = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_center()
                .justify_between(),
            |cx| {
                let badges = stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            badge(cx, "lucide.bed", "4"),
                            badge(cx, "lucide.bath", "2"),
                            badge(cx, "lucide.land-plot", "350m²"),
                        ]
                    },
                );
                let price = ui::text(cx, "$135,000")
                    .font_medium()
                    .into_element(cx)
                    .test_id("ui-gallery-card-image-price");

                vec![badges, price]
            },
        )
        .test_id("ui-gallery-card-image-footer");

        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Is this an image?").into_element(cx),
                shadcn::CardDescription::new("This is a card with an image.").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![cover])
                .refine_style(ChromeRefinement::default().px(Space::N0))
                .into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .refine_layout(max_w_sm.clone().relative())
        .into_element(cx)
        .test_id("ui-gallery-card-image")
    };

    let compositions = {
        let cell = |cx: &mut ElementContext<'_, App>, card: shadcn::Card| {
            card.refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0)))
                .into_element(cx)
        };

        let gap = MetricRef::space(Space::N4).resolve(&theme);
        let layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());

        cx.grid(
            fret_ui::element::GridProps {
                layout,
                cols: 2,
                gap: gap.into(),
                align: fret_ui::element::CrossAlign::Stretch,
                ..Default::default()
            },
            |cx| {
                let content_only = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content Only").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let header_only = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Header Only").into_element(cx),
                            shadcn::CardDescription::new(
                                "This is a card with a header and a description.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let header_and_content = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Header and Content").into_element(cx),
                            shadcn::CardDescription::new(
                                "This is a card with a header and a content.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let footer_only = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardFooter::new(vec![
                            ui::text(cx, "Footer Only").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let header_and_footer = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Header + Footer").into_element(cx),
                            shadcn::CardDescription::new(
                                "This is a card with a header and a footer.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardFooter::new(vec![
                            ui::text(cx, "Footer").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let content_and_footer = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardFooter::new(vec![
                            ui::text(cx, "Footer").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let header_content_footer = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Header + Footer").into_element(cx),
                            shadcn::CardDescription::new(
                                "This is a card with a header and a footer.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardFooter::new(vec![
                            ui::text(cx, "Footer").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let header_with_border = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Header with Border").into_element(cx),
                            shadcn::CardDescription::new(
                                "This is a card with a header that has a bottom border.",
                            )
                            .into_element(cx),
                        ])
                        .border_bottom(true)
                        .into_element(cx),
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                let footer_with_border = {
                    let card = shadcn::Card::new(vec![
                        shadcn::CardContent::new(vec![
                            ui::text(cx, "Content").text_sm().into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardFooter::new(vec![
                            ui::text(cx, "Footer with Border")
                                .text_sm()
                                .into_element(cx),
                        ])
                        .border_top(true)
                        .into_element(cx),
                    ]);
                    cell(cx, card)
                };

                vec![
                    content_only,
                    header_only,
                    header_and_content,
                    footer_only,
                    header_and_footer,
                    content_and_footer,
                    header_content_footer,
                    header_with_border,
                    footer_with_border,
                ]
            },
        )
        .test_id("ui-gallery-card-compositions")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Card provides structure (header/content/footer) but leaves layout decisions to composition.",
            "Prefer consistent max widths for card-based forms to avoid layout jumps across pages.",
            "MediaImage demos use `ImageSourceElementContextExt` to resolve local/URL image sources into `ImageId`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Card docs order: Login, Meeting Notes, Image, Compositions."),
        vec![
            DocSection::new("Login", login)
                .no_shell()
                .max_w(Px(980.0))
                .description("Login card layout (CardHeader + CardContent + CardFooter).")
                .code(
                    "rust",
                    r#"shadcn::Card::new(vec![
    shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new("...").into_element(cx),
        shadcn::CardDescription::new("...").into_element(cx),
    ])
    .into_element(cx),
    shadcn::CardContent::new(vec![/* ... */]).into_element(cx),
    shadcn::CardFooter::new(vec![/* actions */]).into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Meeting Notes", meeting_notes)
                .no_shell()
                .max_w(Px(980.0))
                .description("Card with text content and a footer stack.")
                .code(
                    "rust",
                    r#"shadcn::Card::new(vec![
    shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new("Meeting Notes").into_element(cx),
        shadcn::CardDescription::new("...").into_element(cx),
        shadcn::CardAction::new([shadcn::Button::new("Transcribe")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .leading_icon(fret_icons::IconId::new_static("lucide.captions"))
            .into_element(cx)])
        .into_element(cx),
    ])
    .into_element(cx),
    shadcn::CardContent::new(vec![
        ui::text(cx, "Client requested dashboard redesign with focus on mobile responsiveness.")
            .text_sm()
            .into_element(cx),
        // `ol.list-decimal.pl-6`-style: keep the marker out of the text flow.
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let marker = |cx: &mut ElementContext<'_, App>, n: &str| {
                    ui::text(cx, n)
                        .text_sm()
                        .w_space(Space::N4)
                        .text_align(fret_core::TextAlign::End)
                        .into_element(cx)
                };
                let item = |cx: &mut ElementContext<'_, App>, n: &str, content: &str| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                marker(cx, n),
                                ui::text(cx, content)
                                    .text_sm()
                                    .flex_1()
                                    .min_w_0()
                                    .into_element(cx),
                            ]
                        },
                    )
                };
                vec![
                    item(cx, "1.", "New analytics widgets for daily/weekly metrics"),
                    item(cx, "2.", "Simplified navigation menu"),
                    item(cx, "3.", "Dark mode support"),
                    item(cx, "4.", "Timeline: 6 weeks"),
                    item(cx, "5.", "Follow-up meeting scheduled for next Tuesday"),
                ]
            },
        )
        .into_element(cx),
    ])
    .into_element(cx),
    shadcn::CardFooter::new(vec![shadcn::AvatarGroup::new(vec![
        shadcn::Avatar::new([/* image + fallback */]).into_element(cx),
        shadcn::Avatar::new([/* image + fallback */]).into_element(cx),
        shadcn::Avatar::new([/* image + fallback */]).into_element(cx),
        shadcn::AvatarGroupCount::new([ui::text(cx, "+8").into_element(cx)]).into_element(cx),
    ])
    .into_element(cx)])
    .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Image", image)
                .no_shell()
                .max_w(Px(980.0))
                .description("Card with a media cover and a richer footer row.")
                .code(
                    "rust",
                    r#"let cover = shadcn::AspectRatio::new(
    16.0 / 9.0,
    shadcn::MediaImage::maybe(image_id)
        .loading(true)
        .refine_layout(LayoutRefinement::default().size_full())
        .into_element(cx),
)
.into_element(cx);"#,
                ),
            DocSection::new("Compositions", compositions)
                .no_shell()
                .max_w(Px(980.0))
                .description("Spot-check slot combinations: header/content/footer permutations.")
                .code(
                    "rust",
                    r#"shadcn::Card::new(vec![
    shadcn::CardHeader::new(vec![/* ... */]).into_element(cx),
    shadcn::CardContent::new(vec![/* ... */]).into_element(cx),
    shadcn::CardFooter::new(vec![/* ... */]).into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Implementation notes and pointers."),
        ],
    );

    vec![body.test_id("ui-gallery-card")]
}
