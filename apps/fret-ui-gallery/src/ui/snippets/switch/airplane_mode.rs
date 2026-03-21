pub const SOURCE: &str = include_str!("airplane_mode.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let model = cx.local_model(|| false);
    let control_id = ControlId::from("ui-gallery-switch-airplane");

    ui::h_flex(|cx| {
        vec![
            shadcn::Switch::new(model)
                .control_id(control_id.clone())
                .a11y_label("Airplane mode")
                .test_id("ui-gallery-switch-airplane-toggle")
                .into_element(cx),
            shadcn::Label::new("Airplane Mode")
                .for_control(control_id)
                .test_id("ui-gallery-switch-airplane-label")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-switch-airplane")
}

// endregion: example
