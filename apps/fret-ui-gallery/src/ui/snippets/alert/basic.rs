pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Alert::new([
        // Upstream shadcn/ui v4 docs use `<Terminal />` here.
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.terminal")),
        shadcn::AlertTitle::new("Heads up!").into_element(cx),
        shadcn::AlertDescription::new(
            "You can add components and dependencies to your app using the cli.",
        )
        .into_element(cx),
    ])
    .variant(shadcn::AlertVariant::Default)
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-basic")
}
// endregion: example
