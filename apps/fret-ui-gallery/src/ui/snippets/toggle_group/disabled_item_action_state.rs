pub const SOURCE: &str = include_str!("disabled_item_action_state.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
    disabled: bool,
) -> shadcn::ToggleGroupItem {
    let item = shadcn::ToggleGroupItem::new(value, [decl_text::text_button_label(cx, label)])
        .a11y_label(format!("Toggle {label}"));

    if disabled { item.disabled(true) } else { item }
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let selected = cx.local_model_keyed("toggle_group_disabled_item_action_state", || {
        Some(Arc::<str>::from("alpha"))
    });
    let control_id = ControlId::from("ui-gallery-toggle-group-disabled-item-action-state");

    let group = shadcn::ToggleGroup::single(selected)
        .deselectable(false)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-toggle-group-disabled-item-action-state")
        .items([
            text_item(cx, "alpha", "Alpha", false),
            text_item(cx, "beta", "Beta", true),
            text_item(cx, "gamma", "Gamma", false),
        ])
        .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Formatting mode")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-toggle-group-disabled-item-action-state-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new(
                        "Arrow navigation skips disabled items; disabled items expose no focus or invoke action.",
                    )
                    .for_control(control_id.clone())
                    .into_element(cx),
                ])
                .into_element(cx),
                group,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-toggle-group-disabled-item-action-state")
}
// endregion: example
