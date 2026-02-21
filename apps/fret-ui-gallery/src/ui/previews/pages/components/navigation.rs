use super::super::super::super::*;

pub(in crate::ui) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_collapsible(cx)
}

pub(in crate::ui) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_drawer(cx)
}

pub(in crate::ui) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_hover_card(cx)
}

pub(in crate::ui) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_group(cx)
}

pub(in crate::ui) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_otp(cx)
}

pub(in crate::ui) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_menubar(cx)
}
pub(in crate::ui) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_navigation_menu(cx)
}
pub(in crate::ui) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct PaginationModels {
        rows_per_page: Option<Model<Option<Arc<str>>>>,
        rows_per_page_open: Option<Model<bool>>,
    }

    use crate::ui::doc_layout::{self, DocSection};

    let state = cx.with_state(PaginationModels::default, |st| st.clone());
    let rows_per_page = match state.rows_per_page {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("25")));
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page = Some(model.clone())
            });
            model
        }
    };
    let rows_per_page_open = match state.rows_per_page_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page_open = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        shadcn::Pagination::new([content])
            .into_element(cx)
            .test_id("ui-gallery-pagination-demo")
    };

    let simple = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("4")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("5")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        shadcn::Pagination::new([content])
            .into_element(cx)
            .test_id("ui-gallery-pagination-simple")
    };

    let icons_only = {
        let rows_per_page = shadcn::Select::new(rows_per_page.clone(), rows_per_page_open.clone())
            .placeholder("25")
            .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
            .items([
                shadcn::SelectItem::new("10", "10"),
                shadcn::SelectItem::new("25", "25"),
                shadcn::SelectItem::new("50", "50"),
                shadcn::SelectItem::new("100", "100"),
            ])
            .into_element(cx);

        let rows_field = shadcn::Field::new([
            shadcn::FieldLabel::new("Rows per page").into_element(cx),
            rows_per_page,
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .into_element(cx);

        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content])
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .into_element(cx);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .justify_between()
                .gap(Space::N4),
            move |_cx| [rows_field, pagination],
        );

        row.test_id("ui-gallery-pagination-icons-only")
    };

    let rtl = {
        fn to_arabic_numerals(num: u32) -> String {
            const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
            num.to_string()
                .chars()
                .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
                .collect()
        }

        doc_layout::rtl(cx, |cx| {
            let content = shadcn::PaginationContent::new([
                shadcn::PaginationItem::new(
                    shadcn::PaginationPrevious::new()
                        .text("السابق")
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::PaginationItem::new(
                    shadcn::PaginationLink::new([cx.text(to_arabic_numerals(1))])
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::PaginationItem::new(
                    shadcn::PaginationLink::new([cx.text(to_arabic_numerals(2))])
                        .on_click(CMD_APP_SAVE)
                        .active(true)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::PaginationItem::new(
                    shadcn::PaginationLink::new([cx.text(to_arabic_numerals(3))])
                        .on_click(CMD_APP_SAVE)
                        .into_element(cx),
                )
                .into_element(cx),
                shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                    .into_element(cx),
                shadcn::PaginationItem::new(
                    shadcn::PaginationNext::new()
                        .text("التالي")
                        .on_click(CMD_APP_SAVE)
                        .into_element(cx),
                )
                .into_element(cx),
            ])
            .into_element(cx);

            shadcn::Pagination::new([content])
                .into_element(cx)
                .test_id("ui-gallery-pagination-rtl")
        })
    };

    let extras = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific recipes and regression gates (not part of upstream shadcn PaginationDemo).",
                ),
                simple,
                icons_only,
            ]
        },
    )
    .test_id("ui-gallery-pagination-extras");

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Pagination demo (new-york-v4).",
            "Pagination primitives are intentionally small; compose them with routing/actions in your app layer.",
            "Use `doc_layout::rtl` to validate icon direction and number shaping under RTL.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Pagination demo: Previous, numbered links, ellipsis, Next."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-pagination-demo")
                .code(
                    "rust",
                    r#"let content = shadcn::PaginationContent::new([
    shadcn::PaginationItem::new(shadcn::PaginationPrevious::new().into_element(cx)).into_element(cx),
    shadcn::PaginationItem::new(shadcn::PaginationLink::new([cx.text("1")]).into_element(cx)).into_element(cx),
    shadcn::PaginationItem::new(shadcn::PaginationLink::new([cx.text("2")]).active(true).into_element(cx)).into_element(cx),
    shadcn::PaginationItem::new(shadcn::PaginationLink::new([cx.text("3")]).into_element(cx)).into_element(cx),
    shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx)).into_element(cx),
    shadcn::PaginationItem::new(shadcn::PaginationNext::new().into_element(cx)).into_element(cx),
])
.into_element(cx);

shadcn::Pagination::new([content]).into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-pagination-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Pagination::new([content]).into_element(cx)
});"#,
                ),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-pagination-extras")
                .code(
                    "rust",
                    r#"// Simple
shadcn::Pagination::new([simple_content]).into_element(cx);

// Icons only + "Rows per page" field (app-level recipe)
stack::hstack(cx, props, |cx| vec![rows_field, pagination]);"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-pagination")]
}
