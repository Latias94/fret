pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::table(|cx| {
        ui::children![
            cx;
            shadcn::table_header(|cx| {
                ui::children![
                    cx;
                    shadcn::table_row(2, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_head("King's Treasury"),
                            shadcn::table_head("People's Happiness"),
                        ]
                    })
                    .border_bottom(true),
                ]
            }),
            shadcn::table_body(|cx| {
                vec![
                    shadcn::table_row(2, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(ui::text("Empty")),
                            shadcn::table_cell(ui::text("Overflowing")),
                        ]
                    })
                    .into_element(cx),
                    shadcn::table_row(2, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_cell(ui::text("Modest")),
                            shadcn::table_cell(ui::text("Satisfied")),
                        ]
                    })
                    .into_element(cx),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-typography-table")
}
// endregion: example
