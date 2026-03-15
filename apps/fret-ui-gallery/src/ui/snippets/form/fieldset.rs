pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let text_input = cx.local_model_keyed("text_input", String::new);
    let text_area = cx.local_model_keyed("text_area", String::new);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::field_set(|cx| {
        ui::children![
            cx;
            shadcn::FieldLegend::new("Profile"),
            shadcn::FieldDescription::new("Group related fields to keep structure explicit."),
            shadcn::field_group(|cx| {
                ui::children![
                    cx;
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldLabel::new("Display name"),
                        shadcn::Input::new(text_input.clone())
                            .placeholder("Evil Rabbit")
                            .a11y_label("Display name"),
                    ]),
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::FieldLabel::new("Bio"),
                        shadcn::Textarea::new(text_area.clone())
                            .a11y_label("Bio")
                            .refine_layout(LayoutRefinement::default().h_px(Px(88.0))),
                    ]),
                    shadcn::Field::new(ui::children![
                        cx;
                        shadcn::Button::new("Submit"),
                        shadcn::Button::new("Cancel")
                            .variant(shadcn::ButtonVariant::Outline),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal),
                ]
            }),
        ]
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-form-fieldset")
}
// endregion: example
