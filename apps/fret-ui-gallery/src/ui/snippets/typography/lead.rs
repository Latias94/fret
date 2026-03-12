pub const SOURCE: &str = include_str!("lead.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::lead("A larger lead paragraph introduces a section.").into_element(cx)
}
// endregion: example
