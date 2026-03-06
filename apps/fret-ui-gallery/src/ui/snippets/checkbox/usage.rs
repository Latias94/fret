pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    checked: Option<Model<bool>>,
}

fn checked_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.checked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.checked = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checked = checked_model(cx);

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
