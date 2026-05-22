pub const SOURCE: &str = include_str!("disabled_field.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let disabled_input = cx.local_model_keyed("form_disabled_field_input", || {
        "Disabled profile".to_string()
    });
    let enabled_input = cx.local_model_keyed("form_enabled_field_input", || {
        "Editable profile".to_string()
    });

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Disabled profile field")
                    .test_id("ui-gallery-form-disabled-field-label"),
                shadcn::Input::new(disabled_input.clone())
                    .placeholder("Cannot edit")
                    .a11y_label("Disabled profile field")
                    .disabled(true)
                    .test_id("ui-gallery-form-disabled-field-control"),
                shadcn::FieldDescription::new("The field shell is disabled because its concrete control is disabled."),
            ])
            .disabled(true)
            .test_id("ui-gallery-form-disabled-field"),
            shadcn::Field::new(ui::children![
                cx;
                shadcn::FieldLabel::new("Editable profile field")
                    .test_id("ui-gallery-form-disabled-field-enabled-label"),
                shadcn::Input::new(enabled_input.clone())
                    .placeholder("Can edit")
                    .a11y_label("Editable profile field")
                    .test_id("ui-gallery-form-disabled-field-enabled-control"),
                shadcn::FieldDescription::new("This companion field stays enabled so the gate catches accidental section-wide disabling."),
            ])
            .test_id("ui-gallery-form-disabled-field-enabled"),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-form-disabled-field-group")
}
// endregion: example
