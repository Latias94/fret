pub const SOURCE: &str = include_str!("attachments_empty.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::ui;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(move |cx| {
        vec![
            ui_ai::AttachmentEmpty::new(Vec::new())
                .test_id("ui-ai-attachments-empty-root")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_center()
    .into_element(cx)
}
// endregion: example
