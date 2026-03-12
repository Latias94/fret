pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::FieldLabel::new("مفتاح API").into_element(cx),
            shadcn::Input::new(value)
                .a11y_label("مفتاح API")
                .placeholder("sk-...")
                .into_element(cx),
            shadcn::FieldDescription::new("استخدم هذا المفتاح للوصول إلى واجهة برمجة التطبيقات.")
                .into_element(cx),
        ])
        .refine_layout(max_w_xs)
        .into_element(cx)
    })
    .test_id("ui-gallery-input-rtl")
}
// endregion: example
