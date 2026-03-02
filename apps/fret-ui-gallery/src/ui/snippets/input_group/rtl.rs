pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
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

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::InputGroup::new(value.clone())
            .a11y_label("RTL input group")
            .leading([shadcn::InputGroupText::new("lock").into_element(cx)])
            .trailing([shadcn::InputGroupText::new("sk-...").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-input-group-rtl")
}
// endregion: example
