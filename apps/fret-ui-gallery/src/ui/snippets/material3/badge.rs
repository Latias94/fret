pub const SOURCE: &str = include_str!("badge.rs");

// region: example
use fret_core::{Corners, Px};
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start(),
        |cx| {
            let anchor = |cx: &mut ElementContext<'_, H>, size: Px, test_id: &'static str| {
                let mut props = ContainerProps::default();
                props.layout.size.width = Length::Px(size);
                props.layout.size.height = Length::Px(size);
                props.background =
                    Some(cx.with_theme(|theme| {
                        theme.color_token("md.sys.color.surface-container-low")
                    }));
                props.corner_radii = Corners::all(Px(8.0));
                cx.container(props, |_cx| Vec::<AnyElement>::new())
                    .test_id(test_id)
            };

            let row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N4).items_center(),
                |cx| {
                    let small = Px(24.0);
                    vec![
                        material3::Badge::dot()
                            .navigation_anchor_size(small)
                            .test_id("ui-gallery-material3-badge-dot-nav")
                            .into_element(cx, |cx| vec![anchor(cx, small, "badge-anchor-dot-nav")]),
                        material3::Badge::text("9")
                            .navigation_anchor_size(small)
                            .test_id("ui-gallery-material3-badge-text-nav")
                            .into_element(cx, |cx| {
                                vec![anchor(cx, small, "badge-anchor-text-nav")]
                            }),
                        material3::Badge::dot()
                            .placement(material3::BadgePlacement::TopRight)
                            .test_id("ui-gallery-material3-badge-dot-top-right")
                            .into_element(cx, |cx| {
                                vec![anchor(cx, Px(40.0), "badge-anchor-dot-top-right")]
                            }),
                        material3::Badge::text("99+")
                            .placement(material3::BadgePlacement::TopRight)
                            .test_id("ui-gallery-material3-badge-text-top-right")
                            .into_element(cx, |cx| {
                                vec![anchor(cx, Px(40.0), "badge-anchor-text-top-right")]
                            }),
                    ]
                },
            );

            vec![
                cx.text("Material 3 Badge: dot + large/value variants via md.comp.badge.*."),
                row,
            ]
        },
    )
    .into()
}

// endregion: example
