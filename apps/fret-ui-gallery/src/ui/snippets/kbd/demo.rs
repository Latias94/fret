// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_center(),
        |cx| {
            vec![
                shadcn::KbdGroup::new([
                    shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.command"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.arrow-big-up"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.option"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.chevron-up"),
                    )])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    ui::text(cx, "+").into_element(cx),
                    shadcn::Kbd::new("B").into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-kbd-demo")
}
// endregion: example
