pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    text_area: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text_area = cx.with_state(Models::default, |st| st.text_area.clone());
    let text_area = text_area.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.text_area = Some(model.clone()));
        model
    });

    let max_w_md = LayoutRefinement::default().w_full().min_w_0().max_w(Px(520.0));

    shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .refine_layout(max_w_md.merge(LayoutRefinement::default().h_px(Px(96.0))))
        .into_element(cx)
        .test_id("ui-gallery-form-textarea")
}
// endregion: example
