pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center(),
        |cx| {
            vec![
                shadcn::Checkbox::new(model)
                    // Required for label click -> focus/toggle forwarding.
                    .control_id("ui-gallery-checkbox-demo-toggle")
                    .a11y_label("Accept terms")
                    .test_id("ui-gallery-checkbox-demo-toggle")
                    .into_element(cx),
                shadcn::FieldLabel::new("Accept terms and conditions")
                    .for_control("ui-gallery-checkbox-demo-toggle")
                    .test_id("ui-gallery-checkbox-demo-label")
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-checkbox-demo")
}
// endregion: example
