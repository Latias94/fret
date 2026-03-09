pub const SOURCE: &str = include_str!("action.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new("The selected emails have been marked as spam.")
                    .into_element(cx)
                    .test_id("ui-gallery-alert-action-title"),
                shadcn::AlertDescription::new(
                    "Use the action to reverse the classification without leaving the list view.",
                )
                .into_element(cx)
                .test_id("ui-gallery-alert-action-description"),
                shadcn::AlertAction::new([shadcn::Button::new("Undo")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Xs)
                    .into_element(cx)
                    .test_id("ui-gallery-alert-action-enable")])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-action"),
            shadcn::Alert::new([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.circle-alert")),
                shadcn::AlertTitle::new("The selected emails have been marked as spam.")
                    .into_element(cx),
                shadcn::AlertDescription::new(
                    "This is a very long alert description that demonstrates how the action slot keeps content clear of the top-right badge.",
                )
                .into_element(cx),
                shadcn::AlertAction::new([
                    shadcn::Badge::new("Badge")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-action-badge"),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
