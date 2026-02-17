use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

use std::sync::Arc;
use std::time::Duration;

use fret_runtime::Model;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, PressableProps, ScrollAxis};

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CarouselModels {
        demo_inner_clicked: Option<Model<bool>>,
        animata_expand_selected: Option<Model<u64>>,
    }

    let demo_inner_clicked =
        cx.with_state(CarouselModels::default, |st| st.demo_inner_clicked.clone());
    let animata_expand_selected = cx.with_state(CarouselModels::default, |st| {
        st.animata_expand_selected.clone()
    });
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
    let animata_expand_selected = match animata_expand_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(1u64);
            cx.with_state(CarouselModels::default, |st| {
                st.animata_expand_selected = Some(model.clone());
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

    let animata_expandable = {
        let theme = Theme::global(&*cx.app).clone();
        let duration_ms = theme.duration_ms_token("duration.motion.layout.expand");
        let duration = Duration::from_millis(duration_ms as u64);
        let easing = theme
            .easing_by_key("easing.motion.layout.expand")
            .unwrap_or_else(|| theme.easing_token("easing.motion.emphasized"));

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center(),
            |cx| {
                (1u64..=5)
                    .map(|idx| {
                        let selected_model = animata_expand_selected.clone();
                        let is_selected = cx
                            .watch_model(&selected_model)
                            .paint()
                            .copied()
                            .unwrap_or(1)
                            == idx;

                        let t = cx.keyed(("ui-gallery-carousel-expandable-transition", idx), |cx| {
                            fret_ui_kit::primitives::transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
                                cx,
                                is_selected,
                                duration,
                                duration,
                                easing,
                                false,
                            )
                        });

                        let collapsed_w = 140.0;
                        let expanded_w = 320.0;
                        let collapsed_h = 160.0;
                        let expanded_h = 220.0;

                        let w = collapsed_w + (expanded_w - collapsed_w) * t.progress;
                        let h = collapsed_h + (expanded_h - collapsed_h) * t.progress;

                        let on_activate: fret_ui::action::OnActivate = {
                            let selected_model = selected_model.clone();
                            Arc::new(move |host, action_cx, _reason| {
                                let _ = host.models_mut().update(&selected_model, |v| *v = idx);
                                host.request_redraw(action_cx.window);
                            })
                        };

                        cx.pressable(PressableProps::default(), move |cx, st| {
                                cx.pressable_on_activate(on_activate.clone());

                                let theme = Theme::global(&*cx.app).clone();

                                let header = stack::hstack(
                                    cx,
                                    stack::HStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .justify_between()
                                        .items_center(),
                                    move |cx| {
                                        vec![
                                            shadcn::Badge::new(format!("{idx}"))
                                                .variant(shadcn::BadgeVariant::Secondary)
                                                .into_element(cx),
                                            shadcn::Badge::new(if is_selected {
                                                "Expanded"
                                            } else {
                                                "Collapsed"
                                            })
                                            .variant(if st.hovered || is_selected {
                                                shadcn::BadgeVariant::Default
                                            } else {
                                                shadcn::BadgeVariant::Outline
                                            })
                                            .into_element(cx),
                                        ]
                                    },
                                );

                                let body = cx.flex(
                                    FlexProps {
                                        layout: decl_style::layout_style(
                                            &theme,
                                            LayoutRefinement::default()
                                                .w_full()
                                                .h_full(),
                                        ),
                                        direction: fret_core::Axis::Vertical,
                                        justify: MainAlign::Center,
                                        align: CrossAlign::Center,
                                        padding: Edges::all(Px(16.0)),
                                        ..Default::default()
                                    },
                                    move |cx| vec![
                                        header,
                                        shadcn::typography::muted(
                                            cx,
                                            "Animata recipe: layout.expand (size interpolation; no DOM FLIP).",
                                        ),
                                    ],
                                );

                                let card = shadcn::Card::new([body])
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(w)).h_px(Px(h)),
                                    )
                                    .into_element(cx)
                                    .test_id(format!("ui-gallery-carousel-expandable-card-{idx}"));

                                vec![card]
                            })
                        .test_id(format!("ui-gallery-carousel-expandable-item-{idx}"))
                    })
                    .collect::<Vec<_>>()
            },
        )
        .test_id("ui-gallery-carousel-expandable-row");

        let scroll = shadcn::ScrollArea::new([row])
            .axis(ScrollAxis::X)
            .show_scrollbar(true)
            .viewport_test_id("ui-gallery-carousel-expandable-viewport")
            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(260.0)))
            .into_element(cx);

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "Animata alignment pilot: expandable carousel that interpolates size via a deterministic transition driver (no DOM-based FLIP).",
                    ),
                    scroll,
                ]
            },
        )
        .test_id("ui-gallery-carousel-expandable");
        content
    };

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

    let rtl = doc_layout::rtl(cx, |cx| {
        carousel(
            cx,
            "ui-gallery-carousel-rtl",
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
        )
    });

    let notes_stack = doc_layout::notes(
        cx,
        [
            "`item_basis_main_px` defines the visible density contract; keep it explicit per page width.",
            "Spacing parity with web examples depends on pairing negative track margin with item start padding.",
            "Vertical orientation should always set viewport height explicitly to prevent clipping ambiguity.",
            "API/plugins gaps are tracked here intentionally so future Embla parity work remains discoverable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Carousel docs flow: Demo -> Sizes -> Spacing -> Orientation -> Options -> API -> Events -> Plugins.",
        ),
        vec![
            DocSection::new("Demo", demo_body)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-demo")
                .code(
                    "rust",
                    r#"let carousel = shadcn::Carousel::new(items)
    .item_basis_main_px(Px(260.0))
    .refine_layout(LayoutRefinement::default().max_w(Px(360.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Animata: Expandable", animata_expandable)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"let selected = cx.app.models_mut().insert(1u64);

// Drive a deterministic expand/collapse transition (no DOM-based FLIP).
let t = fret_ui_kit::primitives::transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
    cx,
    is_selected,
    Duration::from_millis(220),
    Duration::from_millis(220),
    easing,
    false,
);

let w = collapsed_w + (expanded_w - collapsed_w) * t.progress;
let h = collapsed_h + (expanded_h - collapsed_h) * t.progress;

shadcn::Card::new([/* content */])
    .refine_layout(LayoutRefinement::default().w_px(Px(w)).h_px(Px(h)))
    .into_element(cx);"#,
                ),
            DocSection::new("Sizes", sizes_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .orientation(shadcn::CarouselOrientation::Horizontal)
    .item_basis_main_px(Px(133.328))
    .track_start_neg_margin(Space::N4)
    .item_padding_start(Space::N4)
    .into_element(cx);"#,
                ),
            DocSection::new("Spacing", spacing_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-spacing")
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .track_start_neg_margin(Space::N4)
    .item_padding_start(Space::N4)
    .orientation(shadcn::CarouselOrientation::Vertical)
    .refine_viewport_layout(LayoutRefinement::default().h_px(Px(300.0)))"#,
                ),
            DocSection::new("Orientation", orientation_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .orientation(shadcn::CarouselOrientation::Vertical)
    .item_basis_main_px(Px(100.0))
    .track_start_neg_margin(Space::N1)
    .item_padding_start(Space::N1)
    .refine_viewport_layout(LayoutRefinement::default().h_px(Px(200.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Options", options_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::muted(
    cx,
    "Upstream exposes Embla opts (align/loop/etc). Current Fret Carousel does not expose Embla-style options yet.",
);"#,
                ),
            DocSection::new("API", api_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::muted(
    cx,
    "Upstream exposes setApi for Embla events/options. Follow-up can add a controlled index/event surface once contracts stabilize.",
);"#,
                ),
            DocSection::new("Events", events_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::muted(
    cx,
    "Upstream listens to Embla events like select via the API handle. Fret Carousel currently keeps event surface internal.",
);"#,
                ),
            DocSection::new("Plugins", plugins_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::muted(
    cx,
    "Upstream supports Embla plugins (e.g. autoplay). Fret Carousel does not expose plugin injection yet.",
);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-rtl")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Carousel::new(items)
        .orientation(shadcn::CarouselOrientation::Horizontal)
        .into_element(cx)
})"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(760.0)),
        ],
    );

    vec![body.test_id("ui-gallery-carousel-component")]
}
