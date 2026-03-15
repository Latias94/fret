pub const SOURCE: &str = include_str!("separator.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Copy")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::ButtonGroupSeparator::new().into(),
        shadcn::Button::new("Paste")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-separator")
}

// endregion: example
