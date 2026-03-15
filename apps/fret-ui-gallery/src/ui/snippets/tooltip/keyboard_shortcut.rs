pub const SOURCE: &str = include_str!("keyboard_shortcut.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let keyboard_icon = fret_ui_shadcn::icon::icon(cx, IconId::new_static("lucide.save"))
                .test_id("ui-gallery-tooltip-keyboard-icon");
            let keyboard_trigger = shadcn::Button::new("")
                .a11y_label("Save")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([keyboard_icon])
                .test_id("ui-gallery-tooltip-keyboard-trigger")
                .into_element(cx);
            let keyboard_content = shadcn::TooltipContent::new(vec![
                ui::h_row(|cx| {
                    vec![
                        cx.text("Save Changes"),
                        shadcn::Kbd::new("S").into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
            ]);

            vec![
                shadcn::Tooltip::new(cx, keyboard_trigger, keyboard_content)
                    .side(shadcn::TooltipSide::Top)
                    .into_element(cx)
                    .test_id("ui-gallery-tooltip-keyboard"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
