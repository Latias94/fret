pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    text_area: Model<String>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
    shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .refine_layout(max_w_md.merge(LayoutRefinement::default().h_px(Px(96.0))))
        .into_element(cx)
        .test_id("ui-gallery-form-textarea")
}
// endregion: example
