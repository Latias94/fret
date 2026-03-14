pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let text_input = cx.local_model_keyed("text_input", String::new);
    let text_area = cx.local_model_keyed("text_area", String::new);
    let checkbox = cx.local_model_keyed("checkbox", || false);
    let switch = cx.local_model_keyed("switch", || false);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Contact"),
            shadcn::FieldDescription::new(
                "Model-bound controls keep values while you stay in the window.",
            ),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldLabel::new("Email"),
                        shadcn::Input::new(text_input.clone())
                            .a11y_label("Email")
                            .placeholder("name@example.com"),
                    ]),
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldLabel::new("Message"),
                        shadcn::Textarea::new(text_area.clone())
                            .a11y_label("Message")
                            .refine_layout(LayoutRefinement::default().h_px(Px(96.0))),
                    ]),
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::Checkbox::new(checkbox.clone())
                            .control_id("ui-gallery-form-checkbox-terms")
                            .a11y_label("Accept terms"),
                        shadcn::FieldLabel::new("Accept terms")
                            .for_control("ui-gallery-form-checkbox-terms"),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal),
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldContent::new(ui::children![
                            cx;
                            shadcn::FieldLabel::new("Enable feature")
                                .for_control("ui-gallery-form-switch-feature"),
                            shadcn::FieldDescription::new(
                                "This toggles an optional feature for the current session.",
                            ),
                        ]),
                        shadcn::Switch::new(switch.clone())
                            .control_id("ui-gallery-form-switch-feature")
                            .a11y_label("Enable feature"),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal),
                ]
            }),
        ]
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-form-demo")
}
// endregion: example
