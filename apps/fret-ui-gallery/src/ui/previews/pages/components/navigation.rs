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

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

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

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Demo", body)
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

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Simple", body)
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

        section(cx, "Icons Only", row)
    };

    let rtl = {
        fn to_arabic_numerals(num: u32) -> String {
            const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
            num.to_string()
                .chars()
                .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
                .collect()
        }

        let pagination = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
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

                shadcn::Pagination::new([content]).into_element(cx)
            },
        );

        let body = centered(cx, pagination);
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N6).items_start(),
        |_cx| vec![demo, simple, icons_only, rtl],
    )]
}
