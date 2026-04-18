pub const SOURCE: &str = include_str!("label_in_field.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let email = cx.local_model(String::new);
    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let control_id = "work_email";

    ui::v_stack(|cx| {
        vec![
            shadcn::raw::typography::muted(
                "For forms, prefer Field + FieldLabel for built-in description/error structure.",
            )
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Work email")
                    .for_control(control_id)
                    .into_element(cx),
                shadcn::Input::new(email)
                    .placeholder("name@company.com")
                    .control_id(control_id)
                    .into_element(cx),
                shadcn::FieldDescription::new("We use this email for notifications.")
                    .for_control(control_id)
                    .into_element(cx),
            ])
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(max_w)
    .into_element(cx)
    .test_id("ui-gallery-label-field")
}
// endregion: example
