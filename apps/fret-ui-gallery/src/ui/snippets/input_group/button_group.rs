pub const SOURCE: &str = include_str!("button_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let url = cx.local_model(String::new);

    let group = shadcn::InputGroup::new(url)
        .a11y_label("URL")
        .control_test_id("ui-gallery-input-group-button-group-control")
        .trailing([fret_ui_shadcn::icon::icon(
            cx,
            IconId::new_static("lucide.link-2"),
        )])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into_element(cx);

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new("https://").into(),
        group.into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-input-group-button-group")
}
// endregion: example
