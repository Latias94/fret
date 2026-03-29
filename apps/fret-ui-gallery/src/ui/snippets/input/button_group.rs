pub const SOURCE: &str = include_str!("button_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let search_id = "ui-gallery-input-button-group-search";

    shadcn::Field::new([
        shadcn::FieldLabel::new("Search")
            .for_control(search_id)
            .into_element(cx),
        shadcn::ButtonGroup::new([
            shadcn::Input::new(value)
                .control_id(search_id)
                .placeholder("Type to search...")
                .test_id("ui-gallery-input-button-group-control")
                .into(),
            shadcn::Button::new("Search")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-input-button-group")
}
// endregion: example
