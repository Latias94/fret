pub const SOURCE: &str = include_str!("invalid.rs");

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

    shadcn::Field::new([
        shadcn::FieldLabel::new("Message").into_element(cx),
        shadcn::Textarea::new(value)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .aria_invalid(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
    ])
    .invalid(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-invalid")
}
// endregion: example
