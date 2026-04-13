use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{GridOptions, HorizontalOptions, ImUiFacade, ScrollOptions, VerticalOptions};

pub(super) fn build_imui_children_with_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let mut ui = ImUiFacade {
        cx,
        out,
        build_focus,
    };
    f(&mut ui);
}

pub(super) fn horizontal_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut builder = crate::ui::h_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}

pub(super) fn vertical_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut builder = crate::ui::v_flex_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .gap_metric(options.gap)
        .justify(options.justify)
        .items(options.items);
    if options.wrap {
        builder = builder.wrap();
    } else {
        builder = builder.no_wrap();
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}

pub(super) fn scroll_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let viewport_test_id = options.viewport_test_id.clone();
    let mut builder = crate::ui::scroll_area_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .axis(options.axis)
        .show_scrollbars(options.show_scrollbar_x, options.show_scrollbar_y);
    if let Some(handle) = options.handle {
        builder = builder.handle(handle);
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    if let Some(test_id) = viewport_test_id {
        builder = builder.viewport_test_id(test_id);
    }
    builder.into_element(cx)
}

pub(super) fn grid_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let mut cells: Vec<AnyElement> = Vec::new();
    build_imui_children_with_focus(cx, &mut cells, build_focus, f);

    let columns = options.columns.max(1);
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut row_index = 0usize;
    let mut iter = cells.into_iter();

    loop {
        let mut row_cells: Vec<AnyElement> = Vec::with_capacity(columns);
        for _ in 0..columns {
            let Some(cell) = iter.next() else {
                break;
            };
            row_cells.push(cell);
        }
        if row_cells.is_empty() {
            break;
        }

        let row = cx.keyed(row_index, |cx| {
            crate::ui::h_flex(move |_cx| row_cells)
                .gap_metric(options.column_gap.clone())
                .justify(options.row_justify)
                .items(options.row_items)
                .no_wrap()
                .into_element(cx)
        });
        rows.push(row);
        row_index += 1;
    }

    let mut builder = crate::ui::v_flex(move |_cx| rows)
        .layout(layout)
        .gap_metric(options.row_gap)
        .justify(crate::Justify::Start)
        .items(crate::Items::Stretch)
        .no_wrap();
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, Length};

    use super::{
        GridOptions, HorizontalOptions, ScrollOptions, grid_container_element,
        horizontal_container_element, scroll_container_element, vertical_container_element,
    };
    use crate::LayoutRefinement;
    use crate::imui::UiWriterImUiFacadeExt as _;
    use crate::imui::VerticalOptions;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    #[test]
    fn horizontal_and_vertical_container_options_forward_layout_to_outer_box() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "horizontal", |cx| {
            let element = horizontal_container_element(
                cx,
                None,
                HorizontalOptions {
                    layout: LayoutRefinement::default().w_px(Px(180.0)),
                    ..Default::default()
                },
                |ui| ui.text("row"),
            );

            let ElementKind::Container(props) = &element.kind else {
                panic!("expected horizontal helper outer container");
            };
            assert_eq!(props.layout.size.width, Length::Px(Px(180.0)));
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "vertical", |cx| {
            let element = vertical_container_element(
                cx,
                None,
                VerticalOptions {
                    layout: LayoutRefinement::default().h_px(Px(120.0)),
                    ..Default::default()
                },
                |ui| ui.text("column"),
            );

            let ElementKind::Container(props) = &element.kind else {
                panic!("expected vertical helper outer container");
            };
            assert_eq!(props.layout.size.height, Length::Px(Px(120.0)));
        });
    }

    #[test]
    fn grid_and_scroll_container_options_forward_layout_to_outer_box() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "grid", |cx| {
            let element = grid_container_element(
                cx,
                None,
                GridOptions {
                    layout: LayoutRefinement::default().w_px(Px(200.0)),
                    columns: 2,
                    ..Default::default()
                },
                |ui| {
                    ui.text("A");
                    ui.text("B");
                },
            );

            let ElementKind::Container(props) = &element.kind else {
                panic!("expected grid helper outer container");
            };
            assert_eq!(props.layout.size.width, Length::Px(Px(200.0)));
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "scroll", |cx| {
            let element = scroll_container_element(
                cx,
                None,
                ScrollOptions {
                    layout: LayoutRefinement::default().h_px(Px(96.0)),
                    ..Default::default()
                },
                |ui| ui.text("scroll"),
            );

            let ElementKind::Container(props) = &element.kind else {
                panic!("expected scroll helper outer container");
            };
            assert_eq!(props.layout.size.height, Length::Px(Px(96.0)));
        });
    }

    #[test]
    fn container_option_test_ids_land_on_outer_surface() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "horizontal.test-id",
            |cx| {
                let element = horizontal_container_element(
                    cx,
                    None,
                    HorizontalOptions {
                        test_id: Some(Arc::from("imui-horizontal")),
                        ..Default::default()
                    },
                    |ui| ui.text("row"),
                );

                assert_eq!(
                    element
                        .semantics_decoration
                        .as_ref()
                        .and_then(|decoration| decoration.test_id.as_deref()),
                    Some("imui-horizontal")
                );
            },
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "vertical.test-id", |cx| {
            let element = vertical_container_element(
                cx,
                None,
                VerticalOptions {
                    test_id: Some(Arc::from("imui-vertical")),
                    ..Default::default()
                },
                |ui| ui.text("column"),
            );

            assert_eq!(
                element
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui-vertical")
            );
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "grid.test-id", |cx| {
            let element = grid_container_element(
                cx,
                None,
                GridOptions {
                    test_id: Some(Arc::from("imui-grid")),
                    ..Default::default()
                },
                |ui| {
                    ui.text("A");
                    ui.text("B");
                },
            );

            assert_eq!(
                element
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui-grid")
            );
        });

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "scroll.test-id", |cx| {
            let element = scroll_container_element(
                cx,
                None,
                ScrollOptions {
                    test_id: Some(Arc::from("imui-scroll")),
                    ..Default::default()
                },
                |ui| ui.text("scroll"),
            );

            assert_eq!(
                element
                    .semantics_decoration
                    .as_ref()
                    .and_then(|decoration| decoration.test_id.as_deref()),
                Some("imui-scroll")
            );
        });
    }

    #[test]
    fn scroll_option_viewport_test_id_lands_on_inner_scroll_root() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "scroll.viewport.test-id",
            |cx| {
                let element = scroll_container_element(
                    cx,
                    None,
                    ScrollOptions {
                        viewport_test_id: Some(Arc::from("imui-scroll.viewport")),
                        ..Default::default()
                    },
                    |ui| ui.text("scroll"),
                );

                let inner = match &element.kind {
                    ElementKind::Container(_) => element
                        .children
                        .first()
                        .expect("scroll helper should wrap an inner scroll root"),
                    other => panic!("expected scroll helper outer container, got {other:?}"),
                };

                assert_eq!(
                    inner
                        .semantics_decoration
                        .as_ref()
                        .and_then(|decoration| decoration.test_id.as_deref()),
                    Some("imui-scroll.viewport")
                );
            },
        );
    }
}
