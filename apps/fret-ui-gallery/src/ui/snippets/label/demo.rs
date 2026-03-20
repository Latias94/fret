pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model_keyed("ui_gallery_label_demo_checked", || false);
    let id = ControlId::from("ui-gallery-label-demo-checkbox");

    ui::h_flex(|cx| {
        vec![
            shadcn::Checkbox::new(checked)
                .a11y_label("Accept terms and conditions")
                .control_id(id.clone())
                .test_id("ui-gallery-label-demo-checkbox")
                .into_element(cx),
            shadcn::Label::new("Accept terms and conditions")
                .for_control(id.clone())
                .test_id("ui-gallery-label-demo-label")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-label-demo")
}
// endregion: example
