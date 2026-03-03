pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
        shadcn::AlertTitle::new("Payment failed").into_element(cx),
        shadcn::AlertDescription::new(
            "Please verify card details, billing address, and available funds.",
        )
        .into_element(cx),
    ])
    .variant(shadcn::AlertVariant::Destructive)
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-destructive")
}
// endregion: example
