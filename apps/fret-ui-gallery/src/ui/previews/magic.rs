use super::super::*;

use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_magic as magic;

pub(in crate::ui) fn preview_magic_lens(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let muted = cx.with_theme(|theme| theme.color_required("muted"));
    let border = cx.with_theme(|theme| theme.color_required("border"));

    let mut surface_layout = LayoutStyle::default();
    surface_layout.size.width = Length::Px(Px(560.0));
    surface_layout.size.height = Length::Px(Px(320.0));

    let surface = ContainerProps {
        layout: surface_layout,
        background: Some(muted),
        border: Edges::all(Px(1.0)),
        border_color: Some(border),
        corner_radii: Corners::all(Px(12.0)),
        ..Default::default()
    };

    let mut lens_layout = LayoutStyle::default();
    lens_layout.size.width = Length::Fill;
    lens_layout.size.height = Length::Fill;

    let props = magic::LensProps {
        layout: lens_layout,
        lens_size: Px(170.0),
        zoom_factor: 1.35,
        default_position: Some(Point::new(Px(160.0), Px(120.0))),
        ..Default::default()
    };

    let lens = magic::lens(cx, props, |cx| {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().h_full())
                .items_start(),
            |cx| {
                vec![
                    shadcn::typography::h4(cx, "Lens"),
                    shadcn::typography::p(
                        cx,
                        "Move the pointer to reveal a masked zoomed copy of the content.",
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .justify_between()
                            .items_center(),
                        |cx| {
                            vec![
                                shadcn::Badge::new("left target")
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx)
                                    .test_id("ui-gallery-magic-lens-target-left"),
                                shadcn::Badge::new("right target")
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx)
                                    .test_id("ui-gallery-magic-lens-target-right"),
                            ]
                        },
                    ),
                ]
            },
        );

        vec![body]
    })
    .test_id("ui-gallery-magic-lens");

    let demo = cx.container(surface, |_cx| vec![lens]);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full())
            .items_start(),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Lens (Phase 0)"),
                shadcn::typography::p(
                    cx,
                    "Built from MaskLayer + VisualTransform by duplicating the subtree. \
                     Intended for visual content in Phase 0.",
                ),
                demo,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_magic_border_beam(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let base = cx.with_theme(|theme| theme.color_required("card"));
    let border = cx.with_theme(|theme| theme.color_required("border"));
    let ring = cx.with_theme(|theme| theme.color_required("ring"));

    let mut highlight = ring;
    highlight.a = (highlight.a * 0.85).clamp(0.0, 1.0);

    let mut card_layout = LayoutStyle::default();
    card_layout.size.width = Length::Px(Px(520.0));
    card_layout.size.height = Length::Px(Px(240.0));

    let card = magic::border_beam(
        cx,
        magic::BorderBeamProps {
            layout: card_layout,
            background: base,
            border_base: border,
            beam_color: highlight,
            ..Default::default()
        },
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .items_start(),
                |cx| {
                    vec![
                        shadcn::typography::h4(cx, "BorderBeam"),
                        shadcn::typography::p(
                            cx,
                            "The beam is animated using the runner-owned frame clock; \
                             glow uses blur + additive compositing.",
                        ),
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between()
                                .items_center(),
                            |cx| {
                                vec![
                                    shadcn::Badge::new("left")
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into_element(cx)
                                        .test_id("ui-gallery-magic-border-beam-target-left"),
                                    shadcn::Badge::new("right")
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into_element(cx)
                                        .test_id("ui-gallery-magic-border-beam-target-right"),
                                ]
                            },
                        ),
                    ]
                },
            )]
        },
    )
    .test_id("ui-gallery-magic-border-beam");

    vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        |_cx| [card],
    )]
}

pub(in crate::ui) fn preview_magic_dock(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let muted = cx.with_theme(|theme| theme.color_required("muted"));
    let border = cx.with_theme(|theme| theme.color_required("border"));

    let mut dock_layout = LayoutStyle::default();
    dock_layout.size.width = Length::Px(Px(620.0));
    dock_layout.size.height = Length::Px(Px(92.0));

    let dock = magic::dock(
        cx,
        magic::DockProps {
            layout: dock_layout,
            background: Some(muted),
            border_color: Some(border),
            ..Default::default()
        },
        |cx| {
            (0..9)
                .map(|i| {
                    shadcn::Badge::new(format!("APP-{i}"))
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx)
                        .test_id(match i {
                            1 => "ui-gallery-magic-dock-target-left",
                            4 => "ui-gallery-magic-dock-target-middle",
                            7 => "ui-gallery-magic-dock-target-right",
                            _ => "ui-gallery-magic-dock-target",
                        })
                })
                .collect::<Vec<_>>()
        },
    )
    .test_id("ui-gallery-magic-dock");

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full())
            .items_start(),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Dock (Phase 0)"),
                shadcn::typography::p(
                    cx,
                    "Pointer-proximity magnification. Phase 0 uses a fixed-size layout; \
                     hover gates magnification and reduced-motion is respected for ambient motion.",
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |_cx| [dock],
                ),
            ]
        },
    )]
}

pub(in crate::ui) fn preview_magic_bloom(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let ring = cx.with_theme(|theme| theme.color_required("ring"));

    let mut panel_layout = LayoutStyle::default();
    panel_layout.size.width = Length::Px(Px(560.0));
    panel_layout.size.height = Length::Px(Px(260.0));

    let panel = fret_ui_kit::declarative::bloom::bloom_panel(
        cx,
        fret_ui_kit::declarative::bloom::BloomPanelProps {
            layout: panel_layout,
            effect: fret_ui_kit::recipes::bloom::BloomEffect {
                cutoff: 0.6,
                soft: 0.12,
                blur_radius_px: Px(18.0),
                blur_downsample: 1,
                strength: 1.6,
            },
            ..Default::default()
        },
        |cx| {
            let mut target_layout = LayoutStyle::default();
            target_layout.size.width = Length::Px(Px(240.0));
            target_layout.size.height = Length::Px(Px(92.0));

            let target = cx
                .container(
                    ContainerProps {
                        layout: target_layout,
                        background: Some(ring),
                        corner_radii: Corners::all(Px(18.0)),
                        ..Default::default()
                    },
                    |cx| vec![shadcn::typography::h4(cx, "BLOOM")],
                )
                .test_id("ui-gallery-magic-bloom-target");

            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify_center()
                    .items_center(),
                |_cx| [target],
            )]
        },
    )
    .test_id("ui-gallery-magic-bloom");

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full())
            .items_start(),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Bloom (Tier B recipe example)"),
                shadcn::typography::p(
                    cx,
                    "Threshold -> blur -> add compositing (best-effort). \
                     Intended as an authoring example for creative effects.",
                ),
                panel,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_magic_marquee(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let muted = cx.with_theme(|theme| theme.color_required("muted"));
    let border = cx.with_theme(|theme| theme.color_required("border"));

    let mut surface_layout = LayoutStyle::default();
    surface_layout.size.width = Length::Fill;
    surface_layout.size.height = Length::Px(Px(56.0));

    let surface = ContainerProps {
        layout: surface_layout,
        background: Some(muted),
        border: Edges::all(Px(1.0)),
        border_color: Some(border),
        corner_radii: Corners::all(Px(12.0)),
        ..Default::default()
    };

    let props = magic::MarqueeProps {
        wrap_width: Px(1200.0),
        speed_px_per_s: 80.0,
        ..Default::default()
    };

    let marquee = magic::marquee(cx, props, |cx| {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N6)
                .items_center()
                .layout(LayoutRefinement::default().h_full()),
            |cx| {
                (0..12)
                    .map(|i| {
                        shadcn::Badge::new(format!("MAGIC-{i}"))
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            },
        );

        vec![row]
    })
    .test_id("ui-gallery-magic-marquee");

    let demo = cx.container(surface, |_cx| vec![marquee]);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full())
            .items_start(),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Marquee (Phase 0)"),
                shadcn::typography::p(
                    cx,
                    "Uses runner-owned time + continuous frames; respects reduced-motion. \
                     Provide wrap_width explicitly in v1.",
                ),
                demo,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_magic_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let base = cx.with_theme(|theme| theme.color_required("card"));
    let border = cx.with_theme(|theme| theme.color_required("border"));
    let ring = cx.with_theme(|theme| theme.color_required("ring"));

    let mut highlight = ring;
    highlight.a = (highlight.a * 0.35).clamp(0.0, 1.0);

    let mut border_highlight = ring;
    border_highlight.a = (border_highlight.a * 0.65).clamp(0.0, 1.0);

    let mut card_layout = LayoutStyle::default();
    card_layout.size.width = Length::Px(Px(520.0));
    card_layout.size.height = Length::Px(Px(240.0));

    let card = magic::magic_card(
        cx,
        magic::MagicCardProps {
            layout: card_layout,
            base,
            highlight,
            border: Edges::all(Px(1.0)),
            border_base: border,
            border_highlight,
            ..Default::default()
        },
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full())
                    .items_start(),
                |cx| {
                    vec![
                        shadcn::typography::h4(cx, "MagicCard"),
                        shadcn::typography::p(
                            cx,
                            "Move the pointer over the card to drive the radial highlight.",
                        ),
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between()
                                .items_center(),
                            |cx| {
                                vec![
                                    shadcn::Badge::new("left")
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into_element(cx)
                                        .test_id("ui-gallery-magic-card-target-left"),
                                    shadcn::Badge::new("right")
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into_element(cx)
                                        .test_id("ui-gallery-magic-card-target-right"),
                                ]
                            },
                        ),
                    ]
                },
            )]
        },
    )
    .test_id("ui-gallery-magic-card");

    vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        |_cx| [card],
    )]
}
