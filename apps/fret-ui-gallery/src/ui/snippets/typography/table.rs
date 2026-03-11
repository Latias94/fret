pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push(shadcn::TableHead::new("King's Treasury").into_element(cx));
                        out.push(shadcn::TableHead::new("People's Happiness").into_element(cx));
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Empty")));
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Overflowing")));
                    })
                    .into_element(cx),
                );
                out.push(
                    shadcn::TableRow::build(2, |cx, out| {
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Modest")));
                        out.push_ui(cx, shadcn::TableCell::build(ui::text("Satisfied")));
                    })
                    .into_element(cx),
                );
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-typography-table")
}
// endregion: example
