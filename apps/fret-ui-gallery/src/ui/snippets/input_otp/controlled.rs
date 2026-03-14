pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let current = cx.watch_model(&value).cloned().unwrap_or_default();
    let theme = Theme::global(&*cx.app).snapshot();
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    ui::v_flex(|cx| {
        let otp = shadcn::InputOTP::new(value)
            .length(6)
            .test_id_prefix("ui-gallery-input-otp-controlled")
            .refine_layout(max_w_xs.clone())
            .into_element(cx);

        let message: Arc<str> = if current.is_empty() {
            Arc::from("Enter your one-time password.")
        } else {
            Arc::from(format!("You entered: {current}"))
        };

        vec![
            otp.test_id("ui-gallery-input-otp-controlled"),
            ui::label(message)
                .text_size_px(Px(14.0))
                .text_color(fret_ui_shadcn::ColorRef::Color(
                    theme.color_token("muted-foreground"),
                ))
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
