// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
