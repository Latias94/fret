pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::{declarative::text as decl_text, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            decl_text::text_control_readout(cx, "Use"),
            shadcn::KbdGroup::new([
                shadcn::Kbd::new("Ctrl + B").into_element(cx),
                shadcn::Kbd::new("Ctrl + K").into_element(cx),
            ])
            .into_element(cx),
            decl_text::text_control_readout(cx, "to open the command palette"),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-kbd-group")
}
// endregion: example
