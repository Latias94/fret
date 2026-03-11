pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trigger =
        shadcn::TooltipTrigger::new(ui::raw_text("Hover").into_element(cx)).into_element(cx);

    let content = shadcn::TooltipContent::new([ui::raw_text("Add to library").into_element(cx)])
        .into_element(cx);

    shadcn::Tooltip::new(trigger, content).into_element(cx)
}
// endregion: example
