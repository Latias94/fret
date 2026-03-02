// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    name: Option<Model<String>>,
}

fn name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.name = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let name = name_model(cx);
    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));

    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        move |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(max_w.clone()),
                |cx| {
                    vec![
                        shadcn::Label::new("الاسم الكامل").into_element(cx),
                        shadcn::Input::new(name)
                            .placeholder("اكتب هنا")
                            .a11y_label("الاسم الكامل")
                            .into_element(cx),
                    ]
                },
            )
        },
    )
    .test_id("ui-gallery-label-rtl")
}
// endregion: example
