use super::*;

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
