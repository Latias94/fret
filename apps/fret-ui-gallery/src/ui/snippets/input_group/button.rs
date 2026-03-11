pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.with_state(Models::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    shadcn::InputGroup::new(value)
        .a11y_label("Button example")
        .control_test_id("ui-gallery-input-group-button-control")
        .trailing([shadcn::InputGroupButton::new("Search")
            .test_id("ui-gallery-input-group-button-trailing-button")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx)])
        .trailing_has_button(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-button")
        .into_element(cx)
}
// endregion: example
