pub const SOURCE: &str = include_str!("label_in_field.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    email: Option<Model<String>>,
}

fn email_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.email = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let email = email_model(cx);
    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let control_id = "work_email";

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(max_w),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "For forms, prefer Field + FieldLabel for built-in description/error structure.",
                ),
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
        },
    )
    .test_id("ui-gallery-label-field")
}
// endregion: example
