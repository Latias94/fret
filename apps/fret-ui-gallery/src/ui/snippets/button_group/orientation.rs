pub const SOURCE: &str = include_str!("orientation.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::Button::new("")
            .a11y_label("Increase")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into(),
        shadcn::Button::new("")
            .a11y_label("Decrease")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.minus"))
            .into(),
    ])
    .orientation(shadcn::ButtonGroupOrientation::Vertical)
    .a11y_label("Media controls")
    .into_element(cx)
    .test_id("ui-gallery-button-group-orientation")
}

// endregion: example
