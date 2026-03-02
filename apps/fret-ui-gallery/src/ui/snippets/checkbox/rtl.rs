// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, rtl: Model<bool>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Field::new([
                shadcn::Checkbox::new(rtl)
                    .control_id("ui-gallery-checkbox-rtl")
                    .a11y_label("RTL notifications")
                    .test_id("ui-gallery-checkbox-rtl")
                    .into_element(cx),
                shadcn::FieldLabel::new("Enable notifications (RTL)")
                    .for_control("ui-gallery-checkbox-rtl")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
            .into_element(cx)
        },
    )
    .test_id("ui-gallery-checkbox-rtl-field")
}
// endregion: example
