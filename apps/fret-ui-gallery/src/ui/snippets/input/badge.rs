pub const SOURCE: &str = include_str!("badge.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<String>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let label = ui::h_row(|cx| {
        vec![
            shadcn::FieldLabel::new("Webhook URL").into_element(cx),
            shadcn::Badge::new("Recommended")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    shadcn::Field::new([
        label,
        shadcn::Input::new(value)
            .a11y_label("Webhook URL")
            .placeholder("https://example.com/webhook")
            .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-badge")
}
// endregion: example
