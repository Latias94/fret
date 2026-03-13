pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const CMD_APP_OPEN: &str = "ui_gallery.app.open";
const CMD_APP_SAVE: &str = "ui_gallery.app.save";

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let rows_per_page = cx.local_model_keyed("rows_per_page", || Some(Arc::<str>::from("25")));
    let rows_per_page_open = cx.local_model_keyed("rows_per_page_open", || false);

    fn page_number<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H> {
        ui::text(label).tabular_nums()
    }

    let simple = {
        shadcn::pagination(|cx| {
            ui::children![
                cx;
                shadcn::pagination_content(|cx| {
                    ui::children![
                        cx;
                        shadcn::pagination_item(
                            shadcn::pagination_link(|cx| ui::children![cx; page_number("1")])
                                .on_click(CMD_APP_OPEN),
                        ),
                        shadcn::pagination_item(
                            shadcn::pagination_link(|cx| ui::children![cx; page_number("2")])
                                .on_click(CMD_APP_SAVE)
                                .active(true),
                        ),
                        shadcn::pagination_item(
                            shadcn::pagination_link(|cx| ui::children![cx; page_number("3")])
                                .on_click(CMD_APP_SAVE),
                        ),
                        shadcn::pagination_item(
                            shadcn::pagination_link(|cx| ui::children![cx; page_number("4")])
                                .on_click(CMD_APP_SAVE),
                        ),
                        shadcn::pagination_item(
                            shadcn::pagination_link(|cx| ui::children![cx; page_number("5")])
                                .on_click(CMD_APP_SAVE),
                        ),
                    ]
                }),
            ]
        })
        .into_element(cx)
        .test_id("ui-gallery-pagination-simple")
    };

    let icons_only = {
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
                            shadcn::PaginationPrevious::new().on_click(CMD_APP_OPEN),
                        ),
                        shadcn::pagination_item(
                            shadcn::PaginationNext::new().on_click(CMD_APP_SAVE),
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
    };

    ui::v_flex(|cx| {
        ui::children![
            cx;
            shadcn::raw::typography::muted(
                "Extras are Fret-specific recipes and regression gates (not part of upstream shadcn PaginationDemo).",
            ),
            simple,
            icons_only,
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-pagination-extras")
}

// endregion: example
