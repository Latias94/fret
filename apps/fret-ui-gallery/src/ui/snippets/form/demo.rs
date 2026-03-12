pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text_input = cx.local_model_keyed("text_input", String::new);
    let text_area = cx.local_model_keyed("text_area", String::new);
    let checkbox = cx.local_model_keyed("checkbox", || false);
    let switch = cx.local_model_keyed("switch", || false);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::FieldSet::build(|cx, out| {
        out.push_ui(cx, shadcn::FieldLegend::new("Contact"));
        out.push_ui(
            cx,
            shadcn::FieldDescription::new(
                "Model-bound controls keep values while you stay in the window.",
            ),
        );
        out.push_ui(
            cx,
            shadcn::FieldGroup::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(cx, shadcn::FieldLabel::new("Email"));
                        out.push_ui(
                            cx,
                            shadcn::Input::new(text_input.clone())
                                .a11y_label("Email")
                                .placeholder("name@example.com"),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(cx, shadcn::FieldLabel::new("Message"));
                        out.push_ui(
                            cx,
                            shadcn::Textarea::new(text_area.clone())
                                .a11y_label("Message")
                                .refine_layout(LayoutRefinement::default().h_px(Px(96.0))),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::Checkbox::new(checkbox.clone())
                                .control_id("ui-gallery-form-checkbox-terms")
                                .a11y_label("Accept terms"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::FieldLabel::new("Accept terms")
                                .for_control("ui-gallery-form-checkbox-terms"),
                        );
                    })
                    .orientation(shadcn::FieldOrientation::Horizontal),
                );
                out.push_ui(
                    cx,
                    shadcn::Field::build(|cx, out| {
                        let field_content = shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("Enable feature")
                                .for_control("ui-gallery-form-switch-feature")
                                .into_element(cx),
                            shadcn::FieldDescription::new(
                                "This toggles an optional feature for the current session.",
                            )
                            .into_element(cx),
                        ]);
                        out.push_ui(cx, field_content);
                        out.push_ui(
                            cx,
                            shadcn::Switch::new(switch.clone())
                                .control_id("ui-gallery-form-switch-feature")
                                .a11y_label("Enable feature"),
                        );
                    })
                    .orientation(shadcn::FieldOrientation::Horizontal),
                );
            }),
        );
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-form-demo")
}
// endregion: example
