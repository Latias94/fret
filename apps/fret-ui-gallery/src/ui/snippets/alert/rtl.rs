pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3).items_start(),
                |cx| {
                    vec![
                        shadcn::Alert::new([
                            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info")),
                            shadcn::AlertTitle::new("RTL alert sample").into_element(cx),
                            shadcn::AlertDescription::new(
                                "This alert validates right-to-left layout and text alignment.",
                            )
                            .into_element(cx),
                        ])
                        .variant(shadcn::AlertVariant::Default)
                        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
                        .into_element(cx)
                        .test_id("ui-gallery-alert-rtl"),
                    ]
                },
            )
        },
    )
}
// endregion: example
