pub const SOURCE: &str = include_str!("button_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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

    shadcn::Field::new([
        shadcn::FieldLabel::new("Search").into_element(cx),
        shadcn::ButtonGroup::new([
            shadcn::Input::new(value)
                .a11y_label("Search text")
                .placeholder("Type to search...")
                .into_element(cx)
                .into(),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .a11y_label("Search")
                .icon(fret_icons::IconId::new_static("lucide.search"))
                .into_element(cx)
                .into(),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-button-group")
}
// endregion: example
