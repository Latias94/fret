pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    input_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let input_value = cx.with_state(Models::default, |st| st.input_value.clone());
    let input_value = match input_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.input_value = Some(model.clone()));
            model
        }
    };

    shadcn::Field::new([
        shadcn::FieldLabel::new("Input Group").into_element(cx),
        shadcn::InputGroup::new(input_value)
            .a11y_label("Input group")
            .trailing([shadcn::Spinner::new().into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-spinner-input-group")
}

// endregion: example
