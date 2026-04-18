pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    let small = shadcn::ButtonGroup::new([
        shadcn::Button::new("Small")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::Button::new("Group")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::IconSm)
            .icon(icon_id("lucide.plus"))
            .into(),
    ])
    .into_element(cx);

    let medium = shadcn::ButtonGroup::new([
        shadcn::Button::new("Default")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        shadcn::Button::new("Group")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .icon(icon_id("lucide.plus"))
            .into(),
    ])
    .into_element(cx);

    let large = shadcn::ButtonGroup::new([
        shadcn::Button::new("Large")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Lg)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Lg)
            .into(),
        shadcn::Button::new("Group")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Lg)
            .into(),
        shadcn::Button::new("")
            .a11y_label("Add")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::IconLg)
            .icon(icon_id("lucide.plus"))
            .into(),
    ])
    .into_element(cx);

    ui::v_flex(|_cx| vec![small, medium, large])
        .gap(Space::N8)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-button-group-size")
}

// endregion: example
