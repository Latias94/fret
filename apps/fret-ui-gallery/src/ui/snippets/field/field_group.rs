pub const SOURCE: &str = include_str!("field_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let responses_push = cx.local_model_keyed("responses_push", || true);
    let tasks_push = cx.local_model_keyed("tasks_push", || false);
    let tasks_email = cx.local_model_keyed("tasks_email", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::field_set(|cx| {
                ui::children![
                    cx;
                    shadcn::FieldLabel::new("Responses"),
                    shadcn::FieldDescription::new(
                        "Get notified when ChatGPT responds to requests that take time, like research or image generation.",
                    ),
                    shadcn::field_group(|cx| {
                        ui::children![
                            cx;
                            shadcn::Field::new(ui::children![
                                cx;
                                shadcn::Checkbox::new(responses_push.clone())
                                    .control_id("ui-gallery-field-group-responses-push")
                                    .disabled(true)
                                    .a11y_label("Push notifications"),
                                shadcn::FieldLabel::new("Push notifications")
                                    .for_control("ui-gallery-field-group-responses-push"),
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
                    shadcn::FieldDescription::new("Get notified when tasks you've created have updates."),
                    shadcn::field_group(|cx| {
                        ui::children![
                            cx;
                            shadcn::Field::new(ui::children![
                                cx;
                                shadcn::Checkbox::new(tasks_push)
                                    .control_id("ui-gallery-field-group-tasks-push")
                                    .a11y_label("Push notifications"),
                                shadcn::FieldLabel::new("Push notifications")
                                    .for_control("ui-gallery-field-group-tasks-push"),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal),
                            shadcn::Field::new(ui::children![
                                cx;
                                shadcn::Checkbox::new(tasks_email)
                                    .control_id("ui-gallery-field-group-tasks-email")
                                    .a11y_label("Email notifications"),
                                shadcn::FieldLabel::new("Email notifications")
                                    .for_control("ui-gallery-field-group-tasks-email"),
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
