pub const SOURCE: &str = include_str!("bluetooth.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let blue = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0x3B_82_F6));
    let style = shadcn::raw::switch::SwitchStyle::default().track_background(
        fret_ui_kit::WidgetStateProperty::new(None)
            .when(fret_ui_kit::WidgetStates::SELECTED, Some(blue)),
    );
    let control_id = ControlId::from("ui-gallery-switch-bluetooth");

    ui::h_flex(|cx| {
        vec![
            shadcn::Switch::new_controllable(cx, None, true)
                .control_id(control_id.clone())
                .a11y_label("Bluetooth")
                .style(style)
                .test_id("ui-gallery-switch-bluetooth-toggle")
                .into_element(cx),
            shadcn::Label::new("Bluetooth")
                .for_control(control_id)
                .test_id("ui-gallery-switch-bluetooth-label")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-switch-bluetooth")
}

// endregion: example
