pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model(|| false);

    ui::h_flex(|cx| {
        vec![
            shadcn::Checkbox::new(checked)
                .control_id("ui-gallery-checkbox-usage")
                .into_element(cx),
            shadcn::Label::new("Accept terms and conditions")
                .for_control("ui-gallery-checkbox-usage")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
}
// endregion: example
