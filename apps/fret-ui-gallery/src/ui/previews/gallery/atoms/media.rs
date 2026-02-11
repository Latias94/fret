use super::super::super::super::*;

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
