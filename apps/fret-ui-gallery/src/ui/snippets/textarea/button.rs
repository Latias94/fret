// region: example
use fret_core::Px;
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

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                shadcn::Textarea::new(value)
                    .a11y_label("Send message")
                    .placeholder("Type your message here.")
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::Button::new("Send message").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-textarea-button")
}
// endregion: example

