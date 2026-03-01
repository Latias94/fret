// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-check")),
        shadcn::AlertTitle::new("Account updated successfully").into_element(cx),
        shadcn::AlertDescription::new(
            "Your profile information has been saved and applied immediately.",
        )
        .into_element(cx),
    ])
    .variant(shadcn::AlertVariant::Default)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-basic")
}
// endregion: example

