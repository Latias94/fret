pub const SOURCE: &str = include_str!("open_on_hover.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::PopoverContent::build(cx, |cx| {
        ui::children![
            cx;
            shadcn::PopoverHeader::new(ui::children![
                cx;
                shadcn::PopoverTitle::new("Notifications"),
                shadcn::PopoverDescription::new(
                    "Hover intent opens this popover without changing the default click-first docs lane.",
                )
            ])
        ]
    })
    .refine_layout(LayoutRefinement::default().w_px(Px(288.0)))
    .test_id("ui-gallery-popover-open-on-hover-panel");

    shadcn::Popover::new(
        cx,
        shadcn::PopoverTrigger::build(
            shadcn::Button::new("Hover for preview")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-popover-open-on-hover-trigger"),
        ),
        content,
    )
    .align(shadcn::PopoverAlign::Start)
    .open_on_hover(true)
    .hover_open_delay_frames(12)
    .hover_close_delay_frames(6)
    .into_element(cx)
    .test_id("ui-gallery-popover-open-on-hover")
}
// endregion: example
