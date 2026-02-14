use super::super::super::super::*;
use fret_ui_kit::declarative::style as decl_style;
use ui_assets::ui::ImageSourceElementContextExt as _;

pub(in crate::ui) fn preview_card(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
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

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let demo = {
        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-title"),
                shadcn::CardDescription::new("Enter your email below to login to your account")
                    .into_element(cx)
                    .test_id("ui-gallery-card-demo-description"),
                shadcn::CardAction::new(vec![
                    shadcn::Button::new("Sign Up")
                        .variant(shadcn::ButtonVariant::Link)
                        .into_element(cx),
                ])
                .into_element(cx),
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
                    vec![
                        shadcn::Button::new("Login")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        shadcn::Button::new("Login with Google")
                            .variant(shadcn::ButtonVariant::Outline)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .refine_layout(max_w_sm.clone())
        .into_element(cx)
        .test_id("ui-gallery-card-demo");

        centered(cx, card)
    };

    let size = {
        let card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Small Card").into_element(cx),
                shadcn::CardDescription::new("This card uses the small size variant.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(
                "The card component supports a size prop that can be set to \"sm\" for a more compact appearance.",
            )])
            .into_element(cx),
            shadcn::CardFooter::new(vec![shadcn::Button::new("Action")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .refine_layout(LayoutRefinement::default().flex_1().w_full())
                .into_element(cx)])
            .into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

        centered(cx, card)
    };

    let image = {
        let cover_bg = cx.with_theme(|theme| theme.color_token("muted"));
        let cover_stack = {
            let theme = Theme::global(&*cx.app);
            let props = decl_style::container_props(
                theme,
                ChromeRefinement::default().bg(ColorRef::Color(cover_bg)),
                LayoutRefinement::default().relative().size_full(),
            );

            cx.container(props, move |cx| {
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
                let (event_cover, event_cover_state, event_cover_path_exists) =
                    (event_cover_fallback, None, false);

                let image = shadcn::MediaImage::maybe(event_cover)
                    .loading(true)
                    .refine_layout(LayoutRefinement::default().size_full())
                    .into_element(cx)
                    .test_id("ui-gallery-card-image-event-cover-image");

                let overlay_props = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::container_props(
                        theme,
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

        let card = shadcn::Card::new(vec![
            cover,
            shadcn::CardHeader::new(vec![
                shadcn::CardAction::new(vec![
                    shadcn::Badge::new("Featured")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardTitle::new("Design systems meetup").into_element(cx),
                shadcn::CardDescription::new(
                    "A practical talk on component APIs, accessibility, and shipping faster.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardFooter::new(vec![
                shadcn::Button::new("View Event")
                    .refine_layout(LayoutRefinement::default().flex_1().w_full())
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_style(ChromeRefinement::default().pt(Space::N0))
        .refine_layout(max_w_sm.clone().relative())
        .into_element(cx)
        .test_id("ui-gallery-card-image");

        centered(cx, card)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                section(cx, "Demo", demo),
                section(cx, "Size", size),
                section(cx, "Image", image),
            ]
        },
    )]
}
