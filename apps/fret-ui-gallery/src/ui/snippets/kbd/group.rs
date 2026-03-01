// region: example
use fret_ui::Theme;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let muted_fg = theme.color_token("muted-foreground");

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N1).items_center(),
        |cx| {
            vec![
                ui::text(cx, "Use")
                    .text_sm()
                    .text_color(ColorRef::Color(muted_fg))
                    .into_element(cx),
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("Ctrl + B").into_element(cx),
                    shadcn::Kbd::new("Ctrl + K").into_element(cx),
                ])
                .into_element(cx),
                ui::text(cx, "to open the command palette")
                    .text_sm()
                    .text_color(ColorRef::Color(muted_fg))
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-kbd-group")
}
// endregion: example
