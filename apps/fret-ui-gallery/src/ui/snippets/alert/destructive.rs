pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.circle-alert"),
                    ),
                    shadcn::AlertTitle::new("Error"),
                    shadcn::AlertDescription::new(
                        "Your session has expired. Please log in again.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Destructive)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-destructive-session"),
        ]
    })
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-alert-destructive")
}
// endregion: example
