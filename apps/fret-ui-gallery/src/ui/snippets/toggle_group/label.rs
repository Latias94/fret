pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn icon_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
    icon: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(value, IconId::new_static(icon))
        .child(cx.text(label))
        .a11y_label(format!("Toggle {label}"))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let control_id = ControlId::from("ui-gallery-toggle-group-label");

    let group = shadcn::ToggleGroup::single_uncontrolled(Some("bold"))
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-toggle-group-label")
        .items([
            icon_item(cx, "bold", "Bold", "lucide.bold"),
            icon_item(cx, "italic", "Italic", "lucide.italic"),
            icon_item(cx, "underline", "Underline", "lucide.underline"),
        ])
        .into_element(cx);

    shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Formatting")
                .for_control(control_id.clone())
                .test_id("ui-gallery-toggle-group-label-label")
                .into_element(cx),
            shadcn::FieldDescription::new("Click the label to focus the current toggle item.")
                .for_control(control_id.clone())
                .into_element(cx),
        ])
        .into_element(cx),
        group,
    ])
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
    .into_element(cx)
    .test_id("ui-gallery-toggle-group-label")
}
// endregion: example
