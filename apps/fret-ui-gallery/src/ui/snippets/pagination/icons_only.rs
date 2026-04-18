pub const SOURCE: &str = include_str!("icons_only.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.pagination.icons_only.open";
const CMD_APP_SAVE: &str = "ui_gallery.pagination.icons_only.save";

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let rows_per_page = cx.local_model_keyed("rows_per_page", || Some(Arc::<str>::from("25")));
    let rows_per_page_open = cx.local_model_keyed("rows_per_page_open", || false);

    let rows_per_page = shadcn::Select::new(rows_per_page.clone(), rows_per_page_open.clone())
        .value(shadcn::SelectValue::new().placeholder("25"))
        .trigger_test_id("ui-gallery-pagination-rows-per-page-trigger")
        .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
        .items([
            shadcn::SelectItem::new("10", "10").label_tabular_nums(),
            shadcn::SelectItem::new("25", "25").label_tabular_nums(),
            shadcn::SelectItem::new("50", "50").label_tabular_nums(),
            shadcn::SelectItem::new("100", "100").label_tabular_nums(),
        ])
        .into_element(cx);

    let rows_field = shadcn::Field::new([
        shadcn::FieldLabel::new("Rows per page").into_element(cx),
        rows_per_page,
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
    .into_element(cx);

    let pagination = shadcn::pagination(|cx| {
        ui::children![
            cx;
            shadcn::pagination_content(|cx| {
                ui::children![
                    cx;
                    shadcn::pagination_item(
                        shadcn::PaginationPrevious::new().action(CMD_APP_OPEN),
                    ),
                    shadcn::pagination_item(
                        shadcn::PaginationNext::new().action(CMD_APP_SAVE),
                    ),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
    .into_element(cx);

    ui::h_flex(move |_cx| [rows_field, pagination])
        .layout(LayoutRefinement::default().w_full())
        .items_center()
        .justify_between()
        .gap(Space::N4)
        .into_element(cx)
        .test_id("ui-gallery-pagination-icons-only")
}
// endregion: example
