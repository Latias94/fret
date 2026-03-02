// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .layout(max_w_md)
            .items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checkbox.clone())
                                .a11y_label("Accept terms")
                                .into_element(cx),
                            shadcn::Label::new("Accept terms").into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(switch.clone())
                                .a11y_label("Enable feature")
                                .into_element(cx),
                            shadcn::Label::new("Enable feature").into_element(cx),
                        ]
                    },
                ),
            ]
        },
    )
    .test_id("ui-gallery-form-controls")
}
// endregion: example
