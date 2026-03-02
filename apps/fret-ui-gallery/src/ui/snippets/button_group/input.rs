pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    search_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let search_value = cx.with_state(Models::default, |st| st.search_value.clone());
    let search_value = match search_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.search_value = Some(model.clone()));
            model
        }
    };

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::Input::new(search_value)
            .a11y_label("Search")
            .placeholder("Search...")
            .into(),
        shadcn::Button::new("")
            .a11y_label("Search")
            .variant(shadcn::ButtonVariant::Outline)
            .children([shadcn::icon::icon(cx, icon_id("lucide.search"))])
            .into(),
    ])
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(420.0)),
    )
    .into_element(cx)
    .test_id("ui-gallery-button-group-input")
}

// endregion: example
