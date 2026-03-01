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

    shadcn::InputGroup::new(value)
        .textarea()
        .a11y_label("Block end addon")
        .block_end([
            shadcn::InputGroupText::new("0/200").into_element(cx),
            shadcn::InputGroupButton::new("Publish")
                .size(shadcn::InputGroupButtonSize::Sm)
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .block_end_border_top(true)
        .textarea_min_height(Px(84.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-align-block-end")
        .into_element(cx)
}
// endregion: example
