pub const SOURCE: &str = include_str!("text.rs");

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
        .control_test_id("ui-gallery-input-group-text-control")
        .leading([shadcn::InputGroupText::new("https://")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-leading")])
        .trailing([shadcn::InputGroupText::new(".com")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-trailing")])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-text")
        .into_element(cx)
}
// endregion: example
