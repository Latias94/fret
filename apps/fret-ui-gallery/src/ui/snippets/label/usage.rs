pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let control_id = ControlId::from("ui-gallery-label-usage");
    let email = cx.local_model(String::new);

    ui::v_stack(|cx| {
        vec![
            shadcn::Label::new("Your email address")
                .for_control(control_id.clone())
                .into_element(cx),
            shadcn::Input::new(email)
                .placeholder("you@example.com")
                .control_id(control_id)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-label-usage")
}
// endregion: example
