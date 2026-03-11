pub const SOURCE: &str = include_str!("grid.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    first: Option<Model<String>>,
    last: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.first, state.last) {
        (Some(first), Some(last)) => (first, last),
        _ => {
            let models = cx.app.models_mut();
            let first = models.insert(String::new());
            let last = models.insert(String::new());
            cx.with_state(Models::default, |st| {
                st.first = Some(first.clone());
                st.last = Some(last.clone());
            });
            (first, last)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (first, last) = ensure_models(cx);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));

    ui::h_row(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("First Name").into_element(cx),
                shadcn::Input::new(first)
                    .a11y_label("First name")
                    .placeholder("Jordan")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Last Name").into_element(cx),
                shadcn::Input::new(last)
                    .a11y_label("Last name")
                    .placeholder("Lee")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
    .layout(max_w_sm)
    .gap(Space::N4)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-input-grid")
}
// endregion: example
