pub const SOURCE: &str = include_str!("accessibility.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let external_label = shadcn::Label::new("Formatting").into_element(cx);
    let external_label_id = external_label.id;

    ui::v_flex(|cx| {
        vec![
            shadcn::ButtonGroup::new([
                shadcn::Button::new("Button 1").into(),
                shadcn::Button::new("Button 2").into(),
            ])
            .a11y_label("Button group")
            .into_element(cx),
            ui::v_flex(|cx| {
                vec![
                    external_label,
                    shadcn::ButtonGroup::new([
                        shadcn::Button::new("Bold").into(),
                        shadcn::Button::new("Italic").into(),
                    ])
                    .labelled_by_element(external_label_id)
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_start()
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-button-group-accessibility")
}
// endregion: example
