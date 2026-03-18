pub const SOURCE: &str = include_str!("api_reference.rs");

// region: example
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[allow(dead_code)]
pub fn basic_button_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Back").into(),
        shadcn::Button::new("Next").into(),
    ])
    .a11y_label("Wizard navigation")
    .into_element(cx)
}

#[allow(dead_code)]
pub fn button_group_with_separator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Copy").into(),
        shadcn::ButtonGroupSeparator::new().into(),
        shadcn::Button::new("Paste").into(),
    ])
    .into_element(cx)
}

#[allow(dead_code)]
pub fn button_group_with_external_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> impl IntoUiElement<H> + use<H> {
    let label = shadcn::Label::new("Formatting").into_element(cx);
    let label_id = label.id;
    let group = shadcn::ButtonGroup::new([
        shadcn::Button::new("Bold").into(),
        shadcn::Button::new("Italic").into(),
    ])
    .labelled_by_element(label_id)
    .into_element(cx);

    ui::v_flex(|_cx| vec![label, group])
        .gap(Space::N2)
        .items_start()
        .into_element(cx)
}

#[allow(dead_code)]
pub fn button_group_with_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> impl IntoUiElement<H> + use<H> {
    let site_name = cx.local_model(String::new);

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroupText::new_children([ui::text("https://").into_element(cx)]).into(),
        shadcn::Input::new(site_name)
            .a11y_label("Site name")
            .placeholder("my-app")
            .into(),
        shadcn::ButtonGroupText::new(".com").into(),
    ])
    .into_element(cx)
}
// endregion: example
