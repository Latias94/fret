pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::children::UiElementSinkExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Alert::build(|cx, out| {
        let icon = icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert"));
        out.push_ui(cx, icon);
        out.push_ui(cx, shadcn::AlertTitle::new("Heads up!"));
        out.push_ui(
            cx,
            shadcn::AlertDescription::new(
                "You can add components and dependencies to your app using the CLI.",
            ),
        );
        out.push_ui(
            cx,
            shadcn::AlertAction::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::Button::new("Enable")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Xs),
                );
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-usage")
}
// endregion: example
