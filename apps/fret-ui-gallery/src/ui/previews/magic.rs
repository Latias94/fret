use super::super::*;

use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui_magic as magic;

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
