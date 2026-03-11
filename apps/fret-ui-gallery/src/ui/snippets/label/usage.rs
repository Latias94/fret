pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    email: Option<Model<String>>,
}

fn email_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.email = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = ControlId::from("ui-gallery-label-usage");

    ui::v_stack(|cx| {
        vec![
            shadcn::Label::new("Your email address")
                .for_control(control_id.clone())
                .into_element(cx),
            shadcn::Input::new(email_model(cx))
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
