pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::TooltipProvider::new()
        .with(cx, |cx| {
            let content = shadcn::TooltipContent::build(cx, |_cx| {
                [shadcn::TooltipContent::text("Add to library")]
            });

            vec![
                shadcn::Tooltip::new(
                    cx,
                    shadcn::TooltipTrigger::build(
                        shadcn::Button::new("Hover").variant(shadcn::ButtonVariant::Outline),
                    ),
                    content,
                )
                .into_element(cx),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
