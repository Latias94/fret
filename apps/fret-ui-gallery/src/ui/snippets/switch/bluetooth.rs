pub const SOURCE: &str = include_str!("bluetooth.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let blue = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0x3B_82_F6));
    let style = shadcn::switch::SwitchStyle::default().track_background(
        fret_ui_kit::WidgetStateProperty::new(None)
            .when(fret_ui_kit::WidgetStates::SELECTED, Some(blue)),
    );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_w(Px(520.0)),
            ),
        |cx| {
            vec![
                shadcn::Switch::new_controllable(cx, None, true)
                    .a11y_label("Bluetooth")
                    .style(style)
                    .test_id("ui-gallery-switch-bluetooth-toggle")
                    .into_element(cx),
                shadcn::Label::new("Bluetooth").into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-switch-bluetooth")
}

// endregion: example
