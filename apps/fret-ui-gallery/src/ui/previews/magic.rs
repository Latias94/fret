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
