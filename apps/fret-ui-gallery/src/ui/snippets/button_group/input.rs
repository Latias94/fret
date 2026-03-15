pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let search_value = cx.local_model(String::new);

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::Input::new(search_value)
            .a11y_label("Search")
            .placeholder("Search...")
            .into(),
        shadcn::Button::new("")
            .a11y_label("Search")
            .variant(shadcn::ButtonVariant::Outline)
            .children([icon::icon(cx, icon_id("lucide.search"))])
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
