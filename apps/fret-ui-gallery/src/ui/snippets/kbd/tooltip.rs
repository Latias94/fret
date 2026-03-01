// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::TooltipProvider::new()
        .delay_duration_frames(10)
        .skip_delay_duration_frames(5)
        .with(cx, |cx| {
            let save = shadcn::Tooltip::new(
                shadcn::Button::new("Save")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::TooltipContent::with(cx, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| vec![ui::text(cx, "Save Changes").into_element(cx), shadcn::Kbd::new("S").into_element(cx)],
                    )]
                }),
            )
            .arrow(true)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx);

            let print = shadcn::Tooltip::new(
                shadcn::Button::new("Print")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::TooltipContent::with(cx, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                ui::text(cx, "Print Document").into_element(cx),
                                shadcn::KbdGroup::new([
                                    shadcn::Kbd::new("Ctrl").into_element(cx),
                                    shadcn::Kbd::new("P").into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    )]
                }),
            )
            .arrow(true)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx);

            vec![
                shadcn::ButtonGroup::new([save.into(), print.into()])
                    .into_element(cx)
                    .test_id("ui-gallery-kbd-tooltip"),
            ]
        })
        .into_iter()
        .next()
        .expect("kbd tooltip provider should return one root")
}
// endregion: example

