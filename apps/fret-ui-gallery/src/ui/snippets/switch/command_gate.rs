pub const SOURCE: &str = include_str!("command_gate.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

use crate::spec::{CMD_SWITCH_COMMAND_GATE_ACTION, CMD_SWITCH_COMMAND_GATE_TOGGLE_ENABLED};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model(|| false);
    let control_id = ControlId::from("ui-gallery-switch-command-gate");

    let field = shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Command-gated sync")
                .for_control(control_id.clone())
                .into_element(cx)
                .test_id("ui-gallery-switch-command-gate-label"),
            shadcn::FieldDescription::new(
                "External command availability controls whether this switch can invoke.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(checked)
            .control_id(control_id)
            .action(CMD_SWITCH_COMMAND_GATE_ACTION)
            .a11y_label("Command-gated sync")
            .test_id("ui-gallery-switch-command-gate-control")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-command-gate-field");

    let toggle = shadcn::Button::new("Toggle command availability")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .action(CMD_SWITCH_COMMAND_GATE_TOGGLE_ENABLED)
        .test_id("ui-gallery-switch-command-gate-enabled-toggle")
        .into_element(cx);

    ui::v_stack(|_cx| vec![field, toggle])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-command-gate")
}
// endregion: example
