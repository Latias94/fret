pub const SOURCE: &str = include_str!("horizontal.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let rail = ui::container(|cx| {
        vec![stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_start(),
            |cx| {
                let artists = ["Ornella Binni", "Tom Byrom", "Vladimir Malyavko"];
                artists
                    .iter()
                    .enumerate()
                    .map(|(idx, artist)| {
                        let art = shadcn::Skeleton::new()
                            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(150.0)).h_px(Px(200.0)),
                            )
                            .into_element(cx);

                        let caption = shadcn::typography::muted(cx, format!("Photo by {artist}"));

                        let mut figure = stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N2)
                                .items_start()
                                .layout(LayoutRefinement::default().flex_none()),
                            |_cx| vec![art, caption],
                        );

                        if idx == artists.len() - 1 {
                            figure = figure.test_id("ui-gallery-scroll-area-horizontal-last");
                        }

                        figure
                    })
                    .collect::<Vec<_>>()
            },
        )]
    })
    .p_4()
    .into_element(cx);

    let area = shadcn::ScrollArea::new([rail])
        .axis(fret_ui::element::ScrollAxis::X)
        .viewport_test_id("ui-gallery-scroll-area-horizontal-viewport")
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-scroll-area-horizontal"),
        );

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Md),
        LayoutRefinement::default()
            .w_px(Px(384.0))
            .overflow_hidden(),
    );

    cx.container(props, move |_cx| [area])
}
// endregion: example
