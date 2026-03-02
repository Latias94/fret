// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    enabled: Option<Model<bool>>,
}

fn enabled_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.enabled {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.enabled = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let enabled = enabled_model(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let control_id = "ui-gallery-field-switch-mfa";
    shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Multi-factor authentication")
                .for_control(control_id)
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Enable MFA. If no dedicated device is available, use one-time email codes.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(enabled)
            .control_id(control_id)
            .a11y_label("Multi-factor authentication")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-switch")
}
// endregion: example

