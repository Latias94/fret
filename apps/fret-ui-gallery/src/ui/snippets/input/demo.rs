pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let api_key_id = "ui-gallery-input-demo-api-key";

    shadcn::Field::new([
        shadcn::FieldLabel::new("API Key")
            .for_control(api_key_id)
            .into_element(cx),
        shadcn::Input::new(value)
            .control_id(api_key_id)
            .password()
            .placeholder("sk-...")
            .into_element(cx),
        shadcn::FieldDescription::new("Your API key is encrypted and stored securely.")
            .for_control(api_key_id)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-input-demo")
}
// endregion: example
