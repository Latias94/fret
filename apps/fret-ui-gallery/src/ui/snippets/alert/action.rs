pub const SOURCE: &str = include_str!("action.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.moon")),
        shadcn::AlertTitle::new("Dark mode is now available for every project")
            .into_element(cx)
            .test_id("ui-gallery-alert-action-title"),
        shadcn::AlertDescription::new(
            "Enable it in profile settings to reduce eye strain during long sessions.",
        )
        .into_element(cx)
        .test_id("ui-gallery-alert-action-description"),
        shadcn::AlertAction::new([shadcn::Button::new("Enable")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)
            .test_id("ui-gallery-alert-action-enable")])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-action")
}
// endregion: example
