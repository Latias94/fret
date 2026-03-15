pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::collapsible::primitives::{
    Collapsible, CollapsibleContent, CollapsibleTrigger,
};
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    Collapsible::new().into_element(cx, |cx| {
        vec![
            CollapsibleTrigger::new([ui::text("Can I use this in my project?").into_element(cx)])
                .into_element(cx),
            CollapsibleContent::new([ui::text_block(
                "Yes. Free to use for personal and commercial projects. No attribution required.",
            )
            .wrap(TextWrap::WordBreak)
            .into_element(cx)])
            .into_element(cx),
        ]
    })
}
// endregion: example
