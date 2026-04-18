pub const SOURCE: &str = include_str!("nested.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroup::new([
            shadcn::Button::new("1")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .test_id("ui-gallery-button-group-nested-step-1")
                .into(),
            shadcn::Button::new("2")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("3")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("4")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("5")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
        ])
        .into(),
        shadcn::ButtonGroup::new([
            shadcn::Button::new("")
                .a11y_label("Previous")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .test_id("ui-gallery-button-group-nested-previous")
                .icon(icon_id("lucide.arrow-left"))
                .into(),
            shadcn::Button::new("")
                .a11y_label("Next")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .test_id("ui-gallery-button-group-nested-next")
                .icon(icon_id("lucide.arrow-right"))
                .into(),
        ])
        .into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-nested")
}

// endregion: example
