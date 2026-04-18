pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let url_value = cx.local_model(String::new);
    let control_id = "button-group-url";

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new_children([shadcn::Label::new("https://")
            .for_control(control_id)
            .into_element(cx)])
        .into(),
        shadcn::Input::new(url_value)
            .control_id(control_id)
            .a11y_label("URL")
            .placeholder("my-app")
            .refine_layout(LayoutRefinement::default().w_px(Px(220.0)).min_w_0())
            .into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-text")
}

// endregion: example
