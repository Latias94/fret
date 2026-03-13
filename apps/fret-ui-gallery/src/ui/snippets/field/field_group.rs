pub const SOURCE: &str = include_str!("field_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checkbox_a = cx.local_model_keyed("checkbox_a", || true);
    let checkbox_b = cx.local_model_keyed("checkbox_b", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::field_set(|cx| {
                ui::children![
                    cx;
                    shadcn::FieldLabel::new("Responses"),
                    shadcn::FieldDescription::new("Get notified for long-running responses."),
                    shadcn::field_group(|cx| {
                        ui::children![
                            cx;
                            shadcn::Field::new(ui::children![
                                cx;
                                shadcn::Checkbox::new(checkbox_a.clone())
                                    .disabled(true)
                                    .a11y_label("Push notifications"),
                                shadcn::FieldLabel::new("Push notifications"),
                            ])
                            .disabled(true)
                            .orientation(shadcn::FieldOrientation::Horizontal),
                        ]
                    })
                    .checkbox_group(),
                ]
            }),
            shadcn::FieldSeparator::new(),
            shadcn::field_set(|cx| {
                ui::children![
                    cx;
                    shadcn::FieldLabel::new("Tasks"),
                    shadcn::FieldDescription::new("Get notified when task status changes."),
                    shadcn::field_group(|cx| {
                        ui::children![
                            cx;
                            shadcn::Field::new(ui::children![
                                cx;
                                shadcn::Checkbox::new(checkbox_b)
                                    .a11y_label("Email notifications"),
                                shadcn::FieldLabel::new("Email notifications"),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal),
                        ]
                    })
                    .checkbox_group(),
                ]
            }),
        ]
    })
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-group")
}
// endregion: example
