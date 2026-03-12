pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::KbdGroup::new([
                    shadcn::Kbd::from_children([fret_ui_shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.command"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([fret_ui_shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.arrow-big-up"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([fret_ui_shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.option"),
                    )])
                    .into_element(cx),
                    shadcn::Kbd::from_children([fret_ui_shadcn::kbd::kbd_icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.chevron-up"),
                    )])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    shadcn::Kbd::new("B").into_element(cx),
                ])
                .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
    })
    .test_id("ui-gallery-kbd-rtl")
}
// endregion: example
