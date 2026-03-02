pub const SOURCE: &str = include_str!("rtl.rs");

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
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            let icon =
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.folder-search"));
            let input = shadcn::InputGroup::new(query)
                .a11y_label("RTL search")
                .leading([shadcn::InputGroupText::new("亘丨孬").into_element(cx)])
                .trailing([shadcn::InputGroupText::new("/").into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
                .test_id("ui-gallery-empty-rtl-input-group")
                .into_element(cx);

            shadcn::Empty::new([
                shadcn::empty::EmptyHeader::new([
                    shadcn::empty::EmptyMedia::new([icon])
                        .variant(shadcn::empty::EmptyMediaVariant::Icon)
                        .into_element(cx),
                    shadcn::empty::EmptyTitle::new("RTL Empty State").into_element(cx),
                    shadcn::empty::EmptyDescription::new(
                        "This empty state uses RTL direction context for layout and alignment.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::empty::EmptyContent::new([
                    input,
                    shadcn::Button::new("Create Project").into_element(cx),
                ])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
            .into_element(cx)
        },
    )
    .test_id("ui-gallery-empty-rtl")
}
// endregion: example
