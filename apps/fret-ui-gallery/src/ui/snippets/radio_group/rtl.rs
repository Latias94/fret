pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("خيارات التخطيط")
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .item(
                shadcn::RadioGroupItem::new("default", "افتراضي").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("افتراضي").into_element(cx),
                        shadcn::FieldDescription::new("المسافات القياسية لمعظم حالات الاستخدام.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("comfortable", "مريح").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("مريح").into_element(cx),
                        shadcn::FieldDescription::new("مساحة أكبر بين العناصر.").into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("compact", "مضغوط").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("مضغوط").into_element(cx),
                        shadcn::FieldDescription::new("تباعد أدنى للتخطيطات الكثيفة.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .into_element(cx)
    })
    .test_id("ui-gallery-radio-group-rtl")
}
// endregion: example
