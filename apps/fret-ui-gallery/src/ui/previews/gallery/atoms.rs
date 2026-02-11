use super::super::super::*;

pub(in crate::ui) fn preview_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
        let cover_bg = cx.with_theme(|theme| theme.color_required("muted"));

        let cover = shadcn::AspectRatio::new(
            16.0 / 9.0,
            cx.container(
                fret_ui::element::ContainerProps {
                    background: Some(cover_bg),
                    ..Default::default()
                },
                |cx| vec![cx.text("Event cover")],
            ),
        )
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

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
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

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

pub(in crate::ui) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

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

    let badge_icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static(name),
            Some(Px(12.0)),
            Some(fg),
        )
    };

    let variants = {
        let children = vec![
            shadcn::Badge::new("Default").into_element(cx),
            shadcn::Badge::new("Secondary")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Badge::new("Destructive")
                .variant(shadcn::BadgeVariant::Destructive)
                .into_element(cx),
            shadcn::Badge::new("Outline")
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Variants", body)
    };

    let with_icon = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Verified")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                .into_element(cx),
            shadcn::Badge::new("Bookmark")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.bookmark", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Icon", body)
    };

    let with_spinner = {
        let destructive_fg = ColorRef::Color(theme.color_required("destructive-foreground"));
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let children = vec![
            shadcn::Badge::new("Deleting")
                .variant(shadcn::BadgeVariant::Destructive)
                .children([shadcn::Spinner::new()
                    .color(destructive_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
            shadcn::Badge::new("Generating")
                .variant(shadcn::BadgeVariant::Secondary)
                .children([shadcn::Spinner::new()
                    .color(secondary_fg.clone())
                    .into_element(cx)])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "With Spinner", body)
    };

    let link = {
        let outline_fg = ColorRef::Color(theme.color_required("foreground"));

        let children = vec![
            shadcn::Badge::new("Open Link")
                .variant(shadcn::BadgeVariant::Outline)
                .children([badge_icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Link", body)
    };

    let custom_colors = {
        let border_transparent =
            ChromeRefinement::default().border_color(ColorRef::Color(CoreColor::TRANSPARENT));

        let children = vec![
            shadcn::Badge::new("Blue")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.95,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Green")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.91,
                            g: 0.98,
                            b: 0.91,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Sky")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.90,
                            g: 0.97,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Purple")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 0.95,
                            g: 0.92,
                            b: 1.00,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
            shadcn::Badge::new("Red")
                .variant(shadcn::BadgeVariant::Outline)
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(CoreColor {
                            r: 1.00,
                            g: 0.92,
                            b: 0.92,
                            a: 1.0,
                        }))
                        .merge(border_transparent.clone()),
                )
                .into_element(cx),
        ];
        let body = row(cx, children);
        section(cx, "Custom Colors", body)
    };

    let rtl = {
        let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let children = vec![
                    shadcn::Badge::new("شارة").into_element(cx),
                    shadcn::Badge::new("ثانوي")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    shadcn::Badge::new("متحقق")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([badge_icon(cx, "lucide.badge-check", secondary_fg.clone())])
                        .into_element(cx),
                ];
                row(cx, children)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![variants, with_icon, with_spinner, link, custom_colors, rtl],
    )]
}

pub(in crate::ui) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let a = {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("FR")
            .when_image_missing_model(avatar_image.clone())
            .delay_ms(120)
            .into_element(cx);
        shadcn::Avatar::new([image, fallback]).into_element(cx)
    };

    let b =
        shadcn::Avatar::new([shadcn::AvatarFallback::new("WK").into_element(cx)]).into_element(cx);

    let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("?").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| [a, b, c],
        ),
        cx.text("Tip: use AvatarImage when you have an ImageId; AvatarFallback covers missing/slow loads."),
    ]
}

pub(in crate::ui) fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
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

    let image_cell = |cx: &mut ElementContext<'_, App>,
                      label: &'static str,
                      source: Model<Option<ImageId>>,
                      fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::model(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-{:?}", fit).to_lowercase());

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()),
            |_cx| vec![label, image],
        )
    };

    let image_cell_opt = |cx: &mut ElementContext<'_, App>,
                          label: &'static str,
                          source: Option<ImageId>,
                          fit: fret_core::ViewportFit|
     -> AnyElement {
        let label = cx.text(label);
        let image = shadcn::MediaImage::maybe(source)
            .fit(fit)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(96.0)))
            .into_element(cx)
            .test_id(format!("ui-gallery-image-object-fit-cell-source-{:?}", fit).to_lowercase());

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()),
            |_cx| vec![label, image],
        )
    };

    let row = |cx: &mut ElementContext<'_, App>,
               title: &'static str,
               image: Model<Option<ImageId>>|
     -> AnyElement {
        let stretch = image_cell(
            cx,
            "Stretch",
            image.clone(),
            fret_core::ViewportFit::Stretch,
        );
        let contain = image_cell(
            cx,
            "Contain",
            image.clone(),
            fret_core::ViewportFit::Contain,
        );
        let cover = image_cell(cx, "Cover", image, fret_core::ViewportFit::Cover);

        let header = cx.text(title);
        let grid = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![stretch, contain, cover],
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![header, grid],
        )
    };

    let mapping = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    row(
                        cx,
                        "Wide source (320×180) → fixed 160×96",
                        wide_image.clone(),
                    ),
                    row(
                        cx,
                        "Tall source (180×320) → fixed 160×96",
                        tall_image.clone(),
                    ),
                    row(
                        cx,
                        "Square source (96×96) → fixed 160×96",
                        square_image.clone(),
                    ),
                ]
            },
        );
        section(cx, "SceneOp::Image fit mapping", body)
    };

    let image_source_demo = if let Some(assets) =
        cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned()
    {
        let wide_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.wide_png);
        let tall_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.tall_png);
        let square_state = ui_assets::use_image_source_state(cx.app, cx.window, &assets.square_png);

        let status = cx.text(format!(
            "Status — wide: {:?}, tall: {:?}, square: {:?}",
            wide_state.status, tall_state.status, square_state.status
        ));

        let row_opt = |cx: &mut ElementContext<'_, App>,
                       title: &'static str,
                       image: Option<ImageId>|
         -> AnyElement {
            let stretch = image_cell_opt(cx, "Stretch", image, fret_core::ViewportFit::Stretch);
            let contain = image_cell_opt(cx, "Contain", image, fret_core::ViewportFit::Contain);
            let cover = image_cell_opt(cx, "Cover", image, fret_core::ViewportFit::Cover);

            let header = cx.text(title);
            let grid = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![stretch, contain, cover],
            );

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |_cx| vec![header, grid],
            )
        };

        let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        cx.text("Loads PNG bytes via `ImageSource` → decode (background) → `ImageAssetCache` → ImageId."),
                        status,
                        row_opt(cx, "Wide source (PNG bytes)", wide_state.image),
                        row_opt(cx, "Tall source (PNG bytes)", tall_state.image),
                        row_opt(cx, "Square source (PNG bytes)", square_state.image),
                    ]
                },
            )
            .test_id("ui-gallery-image-object-fit-image-source-demo");

        section(cx, "Ecosystem ImageSource (bytes decode)", body)
    } else {
        let note = cx.text("ImageSource demo assets missing (expected UiGalleryDriver init).");
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![note],
        )
        .test_id("ui-gallery-image-object-fit-image-source-demo");
        section(cx, "Ecosystem ImageSource (bytes decode)", body)
    };

    let intrinsic = {
        let header = cx.text(
            "Policy-owned intrinsic aspect ratio (opt-in): width-only MediaImage can stamp a ratio from ImageMetadataStore.",
        );

        let wide_intrinsic = shadcn::MediaImage::model(wide_image.clone())
            .intrinsic_aspect_ratio_from_metadata(true)
            .fit(fret_core::ViewportFit::Contain)
            .loading(true)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_required("border"))),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-intrinsic-wide");

        let tall_intrinsic = shadcn::MediaImage::model(tall_image.clone())
            .intrinsic_aspect_ratio_from_metadata(true)
            .fit(fret_core::ViewportFit::Contain)
            .loading(true)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_required("border"))),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-intrinsic-tall");

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    header,
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N4)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full()),
                        |_cx| vec![wide_intrinsic, tall_intrinsic],
                    ),
                ]
            },
        );
        section(cx, "Intrinsic aspect ratio (metadata)", body)
    };

    let streaming = {
        let note = cx.text(
            "Streaming updates: the demo pushes partial ImageUpdateRgba8 writes each frame (moving bar).",
        );
        let image = shadcn::MediaImage::model(streaming_image.clone())
            .fit(fret_core::ViewportFit::Cover)
            .loading(true)
            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)).h_px(Px(200.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-streaming");

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![note, image],
        );
        section(cx, "Streaming updates", body)
    };

    let thumbnails = {
        let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
        let len = 500usize;

        let list_layout = fret_ui::element::LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: fret_ui::element::Length::Fill,
                height: fret_ui::element::Length::Px(Px(360.0)),
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let options = fret_ui::element::VirtualListOptions::known(Px(72.0), 10, |_index| Px(72.0));

        let wide = wide_image.clone();
        let tall = tall_image.clone();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout,
            len,
            options,
            &scroll_handle,
            |i| i as fret_ui::ItemKey,
            move |cx, index| {
                let source = if index % 2 == 0 {
                    wide.clone()
                } else {
                    tall.clone()
                };
                let thumb = shadcn::MediaImage::model(source)
                    .fit(fret_core::ViewportFit::Cover)
                    .loading(true)
                    .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                    .refine_layout(LayoutRefinement::default().w_px(Px(56.0)).h_px(Px(56.0)))
                    .into_element(cx);

                let title = cx.text(format!("Row {index}"));
                let subtitle = cx.text(if index % 2 == 0 {
                    "wide → cover"
                } else {
                    "tall → cover"
                });

                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N3)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            thumb,
                            stack::vstack(
                                cx,
                                stack::VStackProps::default()
                                    .gap(Space::N1)
                                    .items_start()
                                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                                |_cx| vec![title, subtitle],
                            ),
                        ]
                    },
                );

                cx.container(
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Md)
                            .p(Space::N2),
                        LayoutRefinement::default().w_full(),
                    ),
                    |_cx| vec![row],
                )
                .test_id(Arc::<str>::from(format!(
                    "ui-gallery-image-object-fit-row-{index}"
                )))
            },
        );

        let scroll_for_jump_80 = scroll_handle.clone();
        let on_jump_80: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            scroll_for_jump_80.scroll_to_item(80, fret_ui::scroll::ScrollStrategy::Start);
            host.request_redraw(action_cx.window);
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    cx.text("Virtualized thumbnails list (alternating wide/tall sources)."),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default()),
                        |cx| {
                            vec![
                                shadcn::Button::new("Jump 80")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-image-object-fit-jump-80")
                                    .on_activate(on_jump_80)
                                    .into_element(cx),
                            ]
                        },
                    ),
                    list.test_id("ui-gallery-image-object-fit-virtual-list"),
                ]
            },
        );

        section(cx, "Thumbnails (VirtualList)", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N8)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![mapping, image_source_demo, intrinsic, streaming, thumbnails],
    )]
}

pub(in crate::ui) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let round = |cx: &mut ElementContext<'_, App>, size: f32| {
        shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(size))
                    .h_px(Px(size))
                    .flex_shrink_0(),
            )
            .into_element(cx)
    };

    let demo = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(250.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 48.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-demo"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Demo", body)
    };

    let avatar = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(150.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(100.0)))
                        .into_element(cx),
                ]
            },
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 40.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-avatar"),
        );

        let framed = shell(cx, row);
        let body = centered(cx, framed);
        section(cx, "Avatar", body)
    };

    let card = {
        let demo_card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(170.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(144.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-card"),
        );

        let body = centered(cx, demo_card);
        section(cx, "Card", body)
    };

    let text_section = {
        let text = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-text"),
        );

        let framed = shell(cx, text);
        let body = centered(cx, framed);
        section(cx, "Text", body)
    };

    let form = {
        let row = |cx: &mut ElementContext<'_, App>, label_w: Px| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| {
                    vec![
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_px(label_w))
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(32.0)))
                            .into_element(cx),
                    ]
                },
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    row(cx, Px(80.0)),
                    row(cx, Px(96.0)),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-form"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Form", body)
    };

    let table = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                (0..5)
                    .map(|_| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .gap(Space::N4)
                                .items_center()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::Skeleton::new()
                                        .refine_layout(
                                            LayoutRefinement::default().flex_1().min_w_0(),
                                        )
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-table"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "Table", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let text_lines = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_px(Px(250.0))),
                    |cx| {
                        vec![
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_full())
                                .into_element(cx),
                            shadcn::Skeleton::new()
                                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                                .into_element(cx),
                        ]
                    },
                );

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    |cx| vec![round(cx, 48.0), text_lines],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-rtl"),
        );

        let framed = shell(cx, content);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Use to show a placeholder while content is loading."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, avatar, card, text_section, form, table, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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

    let demo = {
        let versions: Vec<Arc<str>> = (1..=50)
            .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{:02}", 51 - idx)))
            .collect();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let mut rows: Vec<AnyElement> = Vec::with_capacity(versions.len() * 2 + 1);
                rows.push(shadcn::typography::small(cx, "Tags"));
                for tag in versions {
                    rows.push(cx.text(tag));
                    rows.push(
                        shadcn::Separator::new()
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
                rows
            },
        );

        let scroll = shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-demo"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let horizontal = {
        let rail = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(760.0))),
            |cx| {
                let artists = [
                    "Ornella Binni",
                    "Tom Byrom",
                    "Vladimir Malyavko",
                    "Silvia Serra",
                ];
                artists
                    .iter()
                    .map(|artist| {
                        shadcn::Card::new(vec![
                            shadcn::CardContent::new(vec![
                                {
                                    let photo_props = cx.with_theme(|theme| {
                                        decl_style::container_props(
                                            theme,
                                            ChromeRefinement::default()
                                                .rounded(Radius::Md)
                                                .border_1()
                                                .bg(ColorRef::Color(theme.color_required("muted"))),
                                            LayoutRefinement::default()
                                                .w_px(Px(140.0))
                                                .h_px(Px(180.0)),
                                        )
                                    });
                                    cx.container(photo_props, |_cx| Vec::new())
                                },
                                shadcn::typography::muted(cx, format!("Photo by {artist}")),
                            ])
                            .into_element(cx),
                        ])
                        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            },
        );

        let scroll = shadcn::ScrollArea::new([rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)).h_px(Px(280.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-horizontal"),
            );

        let card = shell(cx, LayoutRefinement::default(), scroll);
        let body = centered(cx, card);
        section(cx, "Horizontal", body)
    };

    let rtl = {
        let rtl_scroll = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let content = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        let mut rows: Vec<AnyElement> =
                            vec![shadcn::typography::small(cx, "العلامات")];
                        for idx in 1..=40 {
                            rows.push(cx.text(format!("v1.2.0-beta.{:02}", 41 - idx)));
                            rows.push(
                                shadcn::Separator::new()
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            );
                        }
                        rows
                    },
                );

                shadcn::ScrollArea::new([content])
                    .axis(fret_ui::element::ScrollAxis::Y)
                    .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
                    .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-scroll-area-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_scroll);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Scrollable region with custom scrollbars and nested content."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, horizontal, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_tooltip(cx)
}

pub(in crate::ui) fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        #[derive(Default)]
        struct SliderPageState {
            last_commit: Option<Model<Vec<f32>>>,
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

        let max_width_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

        let last_commit = cx.with_state(SliderPageState::default, |st| st.last_commit.clone());
        let last_commit = match last_commit {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(Vec::<f32>::new());
                cx.with_state(SliderPageState::default, |st| {
                    st.last_commit = Some(model.clone());
                });
                model
            }
        };

        let controlled_values =
            cx.with_state(SliderPageState::default, |st| st.controlled_values.clone());
        let controlled_values = match controlled_values {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(vec![0.3, 0.7]);
                cx.with_state(SliderPageState::default, |st| {
                    st.controlled_values = Some(model.clone());
                });
                model
            }
        };

        let demo = cx.keyed("ui_gallery.slider.demo", |cx| {
            let last_commit_for_cb = last_commit.clone();
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .test_id("ui-gallery-slider-single")
                .a11y_label("Slider")
                .refine_layout(max_width_xs.clone())
                .on_value_commit(move |host, _cx, values| {
                    let _ = host.models_mut().update(&last_commit_for_cb, |v| {
                        *v = values;
                    });
                })
                .into_element(cx);

            let last_commit_values = cx
                .watch_model(&last_commit)
                .layout()
                .cloned()
                .unwrap_or_default();
            let last_commit_text = if last_commit_values.is_empty() {
                "<none>".to_string()
            } else {
                format!("{last_commit_values:?}")
            };
            let meta = shadcn::typography::muted(cx, format!("onValueCommit: {last_commit_text}"));

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![slider, meta],
            );
            let body = centered(cx, body);
            section(cx, "Demo", body)
        });

        let range = cx.keyed("ui_gallery.slider.range", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
                .range(0.0, 100.0)
                .step(5.0)
                .test_id("ui-gallery-slider-range")
                .a11y_label("Range slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Range", body)
        });

        let multiple = cx.keyed("ui_gallery.slider.multiple", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0, 70.0])
                .range(0.0, 100.0)
                .step(10.0)
                .test_id("ui-gallery-slider-multiple")
                .a11y_label("Multiple thumbs slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Multiple Thumbs", body)
        });

        let vertical = cx.keyed("ui_gallery.slider.vertical", |cx| {
            let a = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .test_id("ui-gallery-slider-vertical")
                .a11y_label("Vertical slider")
                .into_element(cx);

            let b = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .a11y_label("Vertical slider")
                .into_element(cx);

            let body = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N6)
                    .items_center()
                    .justify_center()
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![a, b],
            );

            section(cx, "Vertical", body)
        });

        let controlled = cx.keyed("ui_gallery.slider.controlled", |cx| {
            let values_snapshot = cx
                .watch_model(&controlled_values)
                .layout()
                .cloned()
                .unwrap_or_default();
            let values_text = values_snapshot
                .iter()
                .map(|v| format!("{v:.1}"))
                .collect::<Vec<_>>()
                .join(", ");

            let header = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .items_center()
                    .justify_between(),
                |cx| {
                    vec![
                        shadcn::Label::new("Temperature").into_element(cx),
                        shadcn::typography::muted(cx, values_text),
                    ]
                },
            );
            let slider = shadcn::Slider::new(controlled_values.clone())
                .range(0.0, 1.0)
                .step(0.1)
                .test_id("ui-gallery-slider-controlled")
                .a11y_label("Temperature")
                .into_element(cx);

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![header, slider],
            );

            let body = centered(cx, body);
            section(cx, "Controlled", body)
        });

        let disabled = cx.keyed("ui_gallery.slider.disabled", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .disabled(true)
                .test_id("ui-gallery-slider-disabled")
                .a11y_label("Disabled slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Disabled", body)
        });

        let rtl = cx.keyed("ui_gallery.slider.rtl", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .step(1.0)
                .dir(fret_ui_kit::primitives::direction::LayoutDirection::Rtl)
                .test_id("ui-gallery-slider-rtl")
                .a11y_label("RTL slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "RTL", body)
        });

        let inverted = cx.keyed("ui_gallery.slider.inverted", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .inverted(true)
                .test_id("ui-gallery-slider-inverted")
                .a11y_label("Inverted slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Extras: Inverted", body)
        });

        vec![
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N6)
                    .items_start(),
                |_cx| vec![
                    demo,
                    range,
                    multiple,
                    vertical,
                    controlled,
                    disabled,
                    rtl,
                    inverted,
                ],
            ),
            shadcn::typography::muted(
                cx,
                "Note: demo/range/multiple/vertical/disabled/RTL are uncontrolled (element state). Controlled uses a shared model."
                    .to_string(),
            ),
        ]
    })
}

pub(in crate::ui) fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;

    let icon_cell =
        |cx: &mut ElementContext<'_, App>, label: &str, icon_id: IconId| -> AnyElement {
            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                },
            );

            let theme = Theme::global(&*cx.app);
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Md)
                        .border_1()
                        .p(Space::N3),
                    LayoutRefinement::default().w_full(),
                ),
                |_cx| [row],
            )
        };

    let grid = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                icon_cell(cx, "ui.search", ids::ui::SEARCH),
                icon_cell(cx, "ui.settings", ids::ui::SETTINGS),
                icon_cell(cx, "ui.chevron.right", ids::ui::CHEVRON_RIGHT),
                icon_cell(cx, "ui.close", ids::ui::CLOSE),
                icon_cell(
                    cx,
                    "lucide.loader-circle",
                    IconId::new_static("lucide.loader-circle"),
                ),
            ]
        },
    );

    let spinner_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
                cx.text("Spinner (animated / static)"),
            ]
        },
    );

    vec![grid, spinner_row]
}
