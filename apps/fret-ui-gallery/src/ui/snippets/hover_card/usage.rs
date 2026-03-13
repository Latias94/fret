pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let trigger =
        shadcn::HoverCardTrigger::new(ui::raw_text("Hover").into_element(cx)).into_element(cx);

    let content = shadcn::HoverCardContent::new([ui::raw_text(
        "The React Framework – created and maintained by @vercel.",
    )
    .into_element(cx)]);

    shadcn::HoverCard::new(cx, trigger, content).into_element(cx)
}
// endregion: example
