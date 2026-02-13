use super::super::*;

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CarouselModels {
        demo_inner_clicked: Option<Model<bool>>,
    }

    let demo_inner_clicked =
        cx.with_state(CarouselModels::default, |st| st.demo_inner_clicked.clone());
    let demo_inner_clicked = match demo_inner_clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(CarouselModels::default, |st| {
                st.demo_inner_clicked = Some(model.clone());
            });
            model
        }
    };

    #[derive(Debug, Clone, Copy)]
    struct SlideVisual {
        text_px: Px,
        line_height_px: Px,
        aspect_square: bool,
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

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let mut content_layout = LayoutRefinement::default().w_full();
        if visual.aspect_square {
            content_layout = content_layout.aspect_ratio(1.0);
        }

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .font_semibold()
            .into_element(cx);

        let content = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, content_layout),
                direction: fret_core::Axis::Horizontal,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |_cx| vec![number],
        );

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let demo_slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        if idx != 1 {
            return slide(cx, idx, visual);
        }

        let theme = Theme::global(&*cx.app).clone();
        let mut content_layout = LayoutRefinement::default().w_full();
        if visual.aspect_square {
            content_layout = content_layout.aspect_ratio(1.0);
        }

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .font_semibold()
            .into_element(cx);

        let on_inner_activate: fret_ui::action::OnActivate = {
            let demo_inner_clicked = demo_inner_clicked.clone();
            Arc::new(move |host, cx, _reason| {
                let _ = host.models_mut().update(&demo_inner_clicked, |v| *v = true);
                host.request_redraw(cx.window);
            })
        };

        let inner_button = shadcn::Button::new("Inner button")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .test_id("ui-gallery-carousel-demo-inner-button")
            .on_activate(on_inner_activate)
            .into_element(cx);

        let children = vec![number, inner_button];

        let content = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, content_layout),
                direction: fret_core::Axis::Vertical,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                gap: Px(12.0),
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |_cx| children,
        );

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let build_carousel = |cx: &mut ElementContext<'_, App>,
                          items: Vec<AnyElement>,
                          test_id: &'static str,
                          orientation: shadcn::CarouselOrientation,
                          basis: Px,
                          spacing: Space,
                          max_w: Px,
                          viewport_h: Option<Px>| {
        let mut base = shadcn::Carousel::new(items)
            .orientation(orientation)
            .item_basis_main_px(basis)
            .track_start_neg_margin(spacing)
            .item_padding_start(spacing)
            .test_id(test_id)
            .refine_layout(LayoutRefinement::default().w_full().max_w(max_w));

        if let Some(viewport_h) = viewport_h {
            base = base.refine_viewport_layout(LayoutRefinement::default().h_px(viewport_h));
        }

        base.into_element(cx).test_id(test_id)
    };

    let carousel = |cx: &mut ElementContext<'_, App>,
                    test_id: &'static str,
                    orientation: shadcn::CarouselOrientation,
                    basis: Px,
                    spacing: Space,
                    max_w: Px,
                    viewport_h: Option<Px>,
                    slide_visual: SlideVisual| {
        let items = (1..=5)
            .map(|idx| slide(cx, idx, slide_visual))
            .collect::<Vec<_>>();
        build_carousel(
            cx,
            items,
            test_id,
            orientation,
            basis,
            spacing,
            max_w,
            viewport_h,
        )
    };

    let demo_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
        aspect_square: true,
    };
    let demo_items = (1..=5)
        .map(|idx| demo_slide(cx, idx, demo_visual))
        .collect::<Vec<_>>();
    let demo_content = build_carousel(
        cx,
        demo_items,
        "ui-gallery-carousel-demo",
        shadcn::CarouselOrientation::Horizontal,
        Px(320.0),
        Space::N4,
        Px(320.0),
        None,
    );

    let demo_inner_clicked_now = cx
        .watch_model(&demo_inner_clicked)
        .copied()
        .unwrap_or(false);

    let demo_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        move |cx| {
            let mut out = vec![
                shadcn::typography::muted(
                    cx,
                    "Drag starting on the inner button must not activate it; a click must activate it.",
                ),
                demo_content,
            ];

            if demo_inner_clicked_now {
                out.push(
                    shadcn::typography::muted(cx, "Inner button: clicked")
                        .test_id("ui-gallery-carousel-demo-inner-clicked"),
                );
            }

            out
        },
    );
    let demo = section_card(cx, "Demo", demo_body);

    let sizes_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Sizes`: use a smaller item basis to show multiple active items.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-size",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(133.328),
                    Space::N4,
                    Px(384.0),
                    None,
                    SlideVisual {
                        text_px: Px(30.0),
                        line_height_px: Px(36.0),
                        aspect_square: true,
                    },
                ),
            ]
        },
    );
    let sizes = section_card(cx, "Sizes", sizes_content);

    let spacing_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Spacing`: tune track negative margin + item start padding together.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-spacing",
                    shadcn::CarouselOrientation::Horizontal,
                    Px(129.328),
                    Space::N1,
                    Px(384.0),
                    None,
                    SlideVisual {
                        text_px: Px(24.0),
                        line_height_px: Px(32.0),
                        aspect_square: true,
                    },
                ),
            ]
        },
    );
    let spacing = section_card(cx, "Spacing", spacing_content);

    let orientation_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Match docs `Orientation`: vertical mode stacks items and rotates the controls.",
                ),
                carousel(
                    cx,
                    "ui-gallery-carousel-orientation-vertical",
                    shadcn::CarouselOrientation::Vertical,
                    Px(100.0),
                    Space::N1,
                    Px(320.0),
                    Some(Px(200.0)),
                    SlideVisual {
                        text_px: Px(30.0),
                        line_height_px: Px(36.0),
                        aspect_square: false,
                    },
                ),
            ]
        },
    );
    let orientation = section_card(cx, "Orientation", orientation_content);

    let options_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream exposes Embla `opts` (align/loop/etc). Fret currently does not expose Embla-style options.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use `item_basis_main_px` + spacing refinements to get the core layouts from the docs.",
                ),
            ]
        },
    );
    let options = section_card(cx, "Options", options_content);

    let api_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream exposes `setApi` for Embla events/options. Current Fret API focuses on deterministic swipe + buttons.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Follow-up can expose controlled index/event hooks once API contracts are stabilized.",
                ),
            ]
        },
    );
    let api = section_card(cx, "API", api_content);

    let events_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream can listen to Embla events (e.g. `select`) via the API instance from `setApi`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Fret currently does not expose an event surface. Follow-up can add controlled index + callbacks once contracts are stabilized.",
                ),
            ]
        },
    );
    let events = section_card(cx, "Events", events_content);

    let plugins_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream supports Embla plugins (e.g. autoplay). Fret currently does not expose plugin injection.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep this section as an explicit gap marker to avoid silent parity drift.",
                ),
            ]
        },
    );
    let plugins = section_card(cx, "Plugins", plugins_content);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Carousel docs flow: Demo -> Sizes -> Spacing -> Orientation -> Options -> API -> Events -> Plugins.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                sizes,
                spacing,
                orientation,
                options,
                api,
                events,
                plugins,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-carousel-component");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic",
                    r#"let carousel = shadcn::Carousel::new(items)
    .item_basis_main_px(Px(260.0))
    .refine_layout(LayoutRefinement::default().max_w(Px(360.0)))
    .into_element(cx);"#,
                ),
                code_block(
                    cx,
                    "Spacing + Orientation",
                    r#"shadcn::Carousel::new(items)
    .track_start_neg_margin(Space::N4)
    .item_padding_start(Space::N4)
    .orientation(shadcn::CarouselOrientation::Vertical)
    .refine_viewport_layout(LayoutRefinement::default().h_px(Px(300.0)))"#,
                ),
                code_block(
                    cx,
                    "RTL",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Carousel::new(items)
        .orientation(shadcn::CarouselOrientation::Horizontal)
        .into_element(cx)
})"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "`item_basis_main_px` defines the visible density contract; keep it explicit per page width.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Spacing parity with web examples depends on pairing negative track margin with item start padding.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Vertical orientation should always set viewport height explicitly to prevent clipping ambiguity.",
                ),
                shadcn::typography::muted(
                    cx,
                    "API/plugins gaps are tracked here intentionally so future Embla parity work remains discoverable.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-carousel",
        component_panel,
        code_panel,
        notes_panel,
    )
}
