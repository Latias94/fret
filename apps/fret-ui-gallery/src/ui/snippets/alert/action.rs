pub const SOURCE: &str = include_str!("action.rs");

// region: example
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::Alert::build(|cx, out| {
                out.push(fret_ui_shadcn::icon::icon(
                    cx,
                    fret_icons::IconId::new_static("lucide.circle-alert"),
                ));
                out.push_ui(
                    cx,
                    shadcn::AlertTitle::new("The selected emails have been marked as spam.")
                        .ui()
                        .test_id("ui-gallery-alert-action-title"),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertAction::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::Button::new("Undo")
                                .size(shadcn::ButtonSize::Xs)
                                .ui()
                                .shadow_none()
                                .test_id("ui-gallery-alert-action-enable"),
                        );
                    }),
                );
            })
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-action"),
            shadcn::Alert::build(|cx, out| {
                out.push(fret_ui_shadcn::icon::icon(
                    cx,
                    fret_icons::IconId::new_static("lucide.circle-alert"),
                ));
                out.push_ui(
                    cx,
                    shadcn::AlertTitle::new("The selected emails have been marked as spam.")
                        .ui()
                        .test_id("ui-gallery-alert-action-badge-title"),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDescription::new(
                        "This is a very long alert title that demonstrates how the component handles extended text content.",
                    )
                    .ui()
                    .test_id("ui-gallery-alert-action-badge-description"),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertAction::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::Badge::new("Badge")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .ui()
                                .test_id("ui-gallery-alert-action-badge-chip"),
                        );
                    }),
                );
            })
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
