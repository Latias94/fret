pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    url_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let url_value = cx.with_state(Models::default, |st| st.url_value.clone());
    let url_value = match url_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.url_value = Some(model.clone()));
            model
        }
    };

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new("https://").into(),
        shadcn::Input::new(url_value)
            .a11y_label("URL")
            .placeholder("example")
            .refine_layout(LayoutRefinement::default().w_px(Px(220.0)).min_w_0())
            .into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-text")
}

// endregion: example
