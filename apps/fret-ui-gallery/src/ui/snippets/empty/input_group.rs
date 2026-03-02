pub const SOURCE: &str = include_str!("input_group.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    query: Option<Model<String>>,
}

fn query_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.query {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.query = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let query = query_model(cx);
    let search = shadcn::InputGroup::new(query)
        .a11y_label("Search pages")
        .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
        .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .test_id("ui-gallery-empty-input-group-search")
        .into_element(cx);

    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyTitle::new("404 - Not Found").into_element(cx),
            shadcn::empty::EmptyDescription::new(
                "The page you are looking for doesn't exist. Try searching below.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([
            search,
            shadcn::empty::EmptyDescription::new("Need help? Contact support.").into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-input-group")
}
// endregion: example
