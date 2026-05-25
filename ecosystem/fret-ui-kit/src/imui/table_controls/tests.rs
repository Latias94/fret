use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length, ScrollAxis};
use fret_ui::elements;
use fret_ui::scroll::ScrollHandle;

use super::{BuiltTableCell, BuiltTableRow, header, render};
use crate::imui::{TableColumn, TableOptions, TableSortDirection};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

fn contains_text(root: &AnyElement, expected: &str) -> bool {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => true,
        _ => root
            .children
            .iter()
            .any(|child| contains_text(child, expected)),
    }
}

fn count_x_scrolls(root: &AnyElement) -> usize {
    let here = match &root.kind {
        ElementKind::Scroll(props) if props.axis == ScrollAxis::X => 1,
        _ => 0,
    };
    here + root.children.iter().map(count_x_scrolls).sum::<usize>()
}

#[test]
fn table_header_label_uses_shared_table_cell_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        header::table_header_label_text(cx, Arc::from("Very long table header"))
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected table header label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(el.inherited_text_style.is_some());
}

#[test]
fn table_sort_indicator_uses_shared_chrome_glyph_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        header::table_sort_indicator_text(cx, TableSortDirection::Ascending)
    });

    let ElementKind::Text(props) = &el.kind else {
        panic!("expected table sort indicator to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(el.inherited_text_style.is_some());
}

#[test]
fn hidden_table_columns_do_not_render_header_body_or_response() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let columns = vec![
        TableColumn::fill("Name###name"),
        TableColumn::px("Hidden###hidden", Px(96.0)).hidden(),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];
    let (el, response) = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let rows = vec![BuiltTableRow {
            key: Arc::from("row-0"),
            test_id: None,
            background: None,
            cells: vec![
                BuiltTableCell {
                    test_id: None,
                    explicit_test_id: None,
                    background: None,
                    content: cx.text("Name Body"),
                },
                BuiltTableCell {
                    test_id: None,
                    explicit_test_id: None,
                    background: None,
                    content: cx.text("Hidden Body"),
                },
                BuiltTableCell {
                    test_id: None,
                    explicit_test_id: None,
                    background: None,
                    content: cx.text("Owner Body"),
                },
            ],
        }];
        render::render_table(cx, "hidden-columns", columns, rows, TableOptions::default())
    });

    assert_eq!(response.headers().len(), 2);
    assert!(response.header("name").is_some());
    assert!(response.header("hidden").is_none());
    assert!(response.header("owner").is_some());
    assert!(!contains_text(&el, "Hidden"));
    assert!(contains_text(&el, "Name Body"));
    assert!(!contains_text(&el, "Hidden Body"));
    assert!(contains_text(&el, "Owner Body"));
}

#[test]
fn horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let columns = vec![
        TableColumn::px("Name###name", Px(180.0)),
        TableColumn::px("Status###status", Px(120.0)),
    ];
    let (el, response) = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let rows = vec![BuiltTableRow {
            key: Arc::from("row-0"),
            test_id: None,
            background: None,
            cells: vec![
                BuiltTableCell {
                    test_id: None,
                    explicit_test_id: None,
                    background: None,
                    content: cx.text("Name Body"),
                },
                BuiltTableCell {
                    test_id: None,
                    explicit_test_id: None,
                    background: None,
                    content: cx.text("Status Body"),
                },
            ],
        }];
        render::render_table(
            cx,
            "horizontal-scroll",
            columns,
            rows,
            TableOptions {
                horizontal_scroll: Some(ScrollHandle::default()),
                ..Default::default()
            },
        )
    });

    assert_eq!(response.headers().len(), 2);
    assert_eq!(count_x_scrolls(&el), 2);
}
