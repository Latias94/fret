pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_icons::IconId;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model_keyed("ui_gallery_label_children_checked", || false);
    let control_id = ControlId::from("ui-gallery-label-children-checkbox");

    ui::h_flex(|cx| {
        vec![
            shadcn::Checkbox::new(checked)
                .a11y_label("Email me product updates")
                .control_id(control_id.clone())
                .test_id("ui-gallery-label-children-checkbox")
                .into_element(cx),
            shadcn::Label::new("Email me product updates")
                .for_control(control_id.clone())
                .children([icon::icon(cx, IconId::new_static("lucide.sparkles"))])
                .test_id("ui-gallery-label-children-label")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-label-children")
}
// endregion: example
