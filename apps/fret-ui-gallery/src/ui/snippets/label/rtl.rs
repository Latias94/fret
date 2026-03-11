pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
    let control_id = "full_name";

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::Label::new("الاسم الكامل")
                    .for_control(control_id)
                    .into_element(cx),
                shadcn::Input::new(name)
                    .placeholder("اكتب هنا")
                    .control_id(control_id)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_start()
        .layout(max_w.clone())
        .into_element(cx)
    })
    .test_id("ui-gallery-label-rtl")
}
// endregion: example
