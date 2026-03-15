pub const SOURCE: &str = include_str!("split.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Secondary)
            .into(),
        shadcn::ButtonGroupSeparator::new().into(),
        shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-split")
}

// endregion: example
