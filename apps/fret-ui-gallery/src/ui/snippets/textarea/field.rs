pub const SOURCE: &str = include_str!("field.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

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

    let id = ControlId::from("ui-gallery-textarea-message");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Label::new("Your message")
                    .for_control(id.clone())
                    .into_element(cx),
                shadcn::Textarea::new(value)
                    .a11y_label("Your message")
                    .placeholder("Type your message here.")
                    .control_id(id)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-field")
}
// endregion: example
