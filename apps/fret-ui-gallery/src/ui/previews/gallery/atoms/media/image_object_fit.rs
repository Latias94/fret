use super::super::super::super::super::*;

pub(in crate::ui) fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    pages::preview_image_object_fit(
        cx,
        theme,
        square_image,
        wide_image,
        tall_image,
        streaming_image,
    )
}

#[cfg(any())]
use fret_core::scene::ImageSamplingHint;
#[cfg(any())]
use ui_assets::ui::ImageSourceElementContextExt as _;

#[cfg(any())]
pub(in crate::ui) fn preview_image_object_fit_legacy(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        ui::v_flex(move |cx| vec![shadcn::typography::h4(cx, title), body])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx)
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

        ui::v_stack(|_cx| vec![label, image])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()).into_element(cx)
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

        ui::v_stack(|_cx| vec![label, image])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default()).into_element(cx)
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
        let grid = ui::h_flex(|_cx| vec![stretch, contain, cover])
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx);

        ui::v_flex(|_cx| vec![header, grid])
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx)
    };

    let mapping = {
        let body = ui::v_flex(|cx| {
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
            })
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx);
        section(cx, "SceneOp::Image fit mapping", body)
    };

    let image_source_demo = if let Some(assets) =
        cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned()
    {
        let wide_state = cx.use_image_source_state(&assets.wide_png);
        let tall_state = cx.use_image_source_state(&assets.tall_png);
        let square_state = cx.use_image_source_state(&assets.square_png);
        let pixel_state = cx.use_image_source_state(&assets.pixel_png);

        let status = cx.text(format!(
            "Status — wide: {:?}, tall: {:?}, square: {:?}, pixel: {:?}",
            wide_state.status, tall_state.status, square_state.status, pixel_state.status
        ));

        let row_opt = |cx: &mut ElementContext<'_, App>,
                       title: &'static str,
                       image: Option<ImageId>|
         -> AnyElement {
            let stretch = image_cell_opt(cx, "Stretch", image, fret_core::ViewportFit::Stretch);
            let contain = image_cell_opt(cx, "Contain", image, fret_core::ViewportFit::Contain);
            let cover = image_cell_opt(cx, "Cover", image, fret_core::ViewportFit::Cover);

            let header = cx.text(title);
            let grid = ui::h_flex(|_cx| vec![stretch, contain, cover])
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()).into_element(cx);

            ui::v_flex(|_cx| vec![header, grid])
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()).into_element(cx)
        };

        let body = ui::v_flex(|cx| {
                    vec![
                        cx.text("Loads PNG bytes via `ImageSource` → decode (background) → `ImageAssetCache` → ImageId."),
                        status,
                        row_opt(cx, "Wide source (PNG bytes)", wide_state.image),
                        row_opt(cx, "Tall source (PNG bytes)", tall_state.image),
                        row_opt(cx, "Square source (PNG bytes)", square_state.image),
                        {
                            let linear = shadcn::MediaImage::maybe(pixel_state.image)
                                .fit(fret_core::ViewportFit::Stretch)
                                .loading(true)
                                .sampling_hint(ImageSamplingHint::Linear)
                                .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                                .refine_layout(
                                    LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)),
                                )
                                .into_element(cx)
                                .test_id("ui-gallery-image-sampling-linear");

                            let nearest = shadcn::MediaImage::maybe(pixel_state.image)
                                .fit(fret_core::ViewportFit::Stretch)
                                .loading(true)
                                .sampling_hint(ImageSamplingHint::Nearest)
                                .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                                .refine_layout(
                                    LayoutRefinement::default().w_px(Px(160.0)).h_px(Px(160.0)),
                                )
                                .into_element(cx)
                                .test_id("ui-gallery-image-sampling-nearest");

                            let grid = ui::h_flex(move |cx| {
                                    vec![
                                        ui::v_stack(move |cx| vec![cx.text("Linear (explicit)"), linear])
                                                .gap(Space::N2)
                                                .items_start()
                                                .layout(LayoutRefinement::default()).into_element(cx),
                                        ui::v_stack(move |cx| vec![cx.text("Nearest (opt-in)"), nearest])
                                                .gap(Space::N2)
                                                .items_start()
                                                .layout(LayoutRefinement::default()).into_element(cx),
                                    ]
                                })
                                    .gap(Space::N4)
                                    .items_start()
                                    .layout(LayoutRefinement::default().w_full()).into_element(cx);

                            ui::v_flex(move |cx| {
                                    vec![cx.text("Sampling hints (16×16 → 160×160)"), grid]
                                })
                                    .gap(Space::N3)
                                    .items_start()
                                    .layout(LayoutRefinement::default().w_full()).into_element(cx)
                        },
                    ]
                })
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()).into_element(cx)
            .test_id("ui-gallery-image-object-fit-image-source-demo");

        section(cx, "Ecosystem ImageSource (bytes decode)", body)
    } else {
        let note = cx.text("ImageSource demo assets missing (expected UiGalleryDriver init).");
        let body = ui::v_flex(|_cx| vec![note])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx)
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
                    .border_color(ColorRef::Color(theme.color_token("border"))),
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
                    .border_color(ColorRef::Color(theme.color_token("border"))),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
            .test_id("ui-gallery-image-object-fit-intrinsic-tall");

        let body = ui::v_flex(|cx| {
                vec![
                    header,
                    ui::h_flex(|_cx| vec![wide_intrinsic, tall_intrinsic])
                            .gap(Space::N4)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full()).into_element(cx),
                ]
            })
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx);
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

        let body = ui::v_flex(|_cx| vec![note, image])
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx);
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

                let row = ui::h_flex(|cx| {
                        vec![
                            thumb,
                            ui::v_flex(|_cx| vec![title, subtitle])
                                    .gap(Space::N1)
                                    .items_start()
                                    .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx),
                        ]
                    })
                        .gap(Space::N3)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full()).into_element(cx);

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

        let body = ui::v_flex(|cx| {
                vec![
                    cx.text("Virtualized thumbnails list (alternating wide/tall sources)."),
                    ui::h_row(|cx| {
                            vec![
                                shadcn::Button::new("Jump 80")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-image-object-fit-jump-80")
                                    .on_activate(on_jump_80)
                                    .into_element(cx),
                            ]
                        })
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default()).into_element(cx),
                    list.test_id("ui-gallery-image-object-fit-virtual-list"),
                ]
            })
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()).into_element(cx);

        section(cx, "Thumbnails (VirtualList)", body)
    };

    vec![ui::v_flex(|_cx| vec![mapping, image_source_demo, intrinsic, streaming, thumbnails])
            .gap(Space::N8)
            .items_start()
            .layout(LayoutRefinement::default().w_full()).into_element(cx)]
}
