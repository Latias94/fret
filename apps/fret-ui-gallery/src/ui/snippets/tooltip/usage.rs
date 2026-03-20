pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content =
        shadcn::TooltipContent::build(cx, |_cx| [shadcn::TooltipContent::text("Add to library")]);

    shadcn::Tooltip::new(
        cx,
        shadcn::TooltipTrigger::build(ui::raw_text("Hover")),
        content,
    )
    .into_element(cx)
}
// endregion: example
