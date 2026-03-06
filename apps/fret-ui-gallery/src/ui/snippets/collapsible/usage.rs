pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::collapsible_primitives as shadcn_col;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn_col::Collapsible::new().into_element(cx, |cx| {
        vec![
            shadcn_col::CollapsibleTrigger::new([
                ui::text("Can I use this in my project?").into_element(cx)
            ])
            .into_element(cx),
            shadcn_col::CollapsibleContent::new([ui::text_block(
                "Yes. Free to use for personal and commercial projects. No attribution required.",
            )
            .wrap(TextWrap::WordBreak)
            .into_element(cx)])
            .into_element(cx),
        ]
    })
}
// endregion: example
