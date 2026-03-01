// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    checked_controlled: Model<bool>,
    checked_optional: Model<Option<bool>>,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N3)
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checked_controlled)
                                .control_id("ui-gallery-checkbox-controlled")
                                .a11y_label("Controlled checkbox")
                                .test_id("ui-gallery-checkbox-controlled")
                                .into_element(cx),
                            shadcn::FieldLabel::new("Controlled checked state")
                                .for_control("ui-gallery-checkbox-controlled")
                                .test_id("ui-gallery-checkbox-controlled-label")
                                .into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N3)
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new_optional(checked_optional)
                                .control_id("ui-gallery-checkbox-optional")
                                .a11y_label("Optional checkbox")
                                .test_id("ui-gallery-checkbox-optional")
                                .into_element(cx),
                            shadcn::FieldLabel::new("Optional / indeterminate state")
                                .for_control("ui-gallery-checkbox-optional")
                                .test_id("ui-gallery-checkbox-optional-label")
                                .into_element(cx),
                        ]
                    },
                ),
            ]
        },
    )
    .test_id("ui-gallery-checkbox-checked-state")
}
// endregion: example

