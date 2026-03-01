// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
