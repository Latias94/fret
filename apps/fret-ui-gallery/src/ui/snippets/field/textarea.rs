pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    feedback: Option<Model<String>>,
}

fn feedback_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.feedback {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.feedback = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let feedback = feedback_model(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldLabel::new("Feedback").into_element(cx),
        shadcn::Textarea::new(feedback)
            .a11y_label("Feedback")
            .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
            .into_element(cx),
        shadcn::FieldDescription::new("Share your thoughts about our service.").into_element(cx),
    ])
    .into_element(cx)])
    .into_element(cx)])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-textarea")
}
// endregion: example
