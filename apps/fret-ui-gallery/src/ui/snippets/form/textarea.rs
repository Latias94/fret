pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let text_area = cx.local_model(String::new);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .refine_layout(max_w_md.merge(LayoutRefinement::default().h_px(Px(96.0))))
        .into_element(cx)
        .test_id("ui-gallery-form-textarea")
}
// endregion: example
