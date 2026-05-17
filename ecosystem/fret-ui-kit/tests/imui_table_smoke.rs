#![cfg(feature = "imui")]

use fret_core::{Color, Px};
use fret_ui::UiHost;
use fret_ui_kit::imui::{
    ImUiTableColumnVisibilityState, MenuItemOptions, TableCellOptions, TableColumn,
    TableColumnVisibilityHeaderContextMenuOptions, TableColumnVisibilityMenuOptions,
    TableColumnWidth, TableOptions, TableResponse, TableRowOptions, TableSortDirection,
    UiWriterImUiFacadeExt, table_column_visibility_header_context_menu,
    table_column_visibility_menu_item, table_column_visibility_menu_items,
};

#[allow(dead_code)]
fn table_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let columns = [
        TableColumn::fill("Name"),
        TableColumn::weighted("Kind", 2.0),
        TableColumn::unlabeled(TableColumnWidth::px(Px(72.0))),
    ];

    let response = ui.table("table.basic", &columns, |table| {
        table.row("alpha", |row| {
            row.cell_text("Alpha");
            row.cell_text("Folder");
            row.cell_with_options(
                TableCellOptions {
                    background: Some(Color::from_srgb_hex_rgb(0x101820)),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.button("Open");
                },
            );
        });
    });
    let _ = response.headers();

    let response = ui.table_with_options(
        "table.with_options",
        &columns,
        TableOptions {
            striped: true,
            test_id: Some("table.root".into()),
            ..Default::default()
        },
        |table| {
            table.row_with_options(
                "beta",
                TableRowOptions {
                    test_id: Some("table.row.beta".into()),
                    background: Some(Color::from_srgb_hex_rgb(0x201010)),
                },
                |row| {
                    row.cell_text("Beta");
                    row.cell_text_with_options(
                        "Queue",
                        TableCellOptions {
                            test_id: Some("table.cell.kind".into()),
                            background: Some(Color::from_srgb_hex_rgb(0x102020)),
                        },
                    );
                    row.cell_text("42");
                },
            );
        },
    );
    let _ = response.header_at(0);
}

#[allow(dead_code)]
fn table_column_visibility_menu_item_api_compiles<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    model: &fret_runtime::Model<ImUiTableColumnVisibilityState>,
) {
    let column = TableColumn::px("Status###asset-status", Px(96.0));

    let response = table_column_visibility_menu_item(
        ui,
        &column,
        model,
        MenuItemOptions {
            test_id: Some("table.column.status".into()),
            ..Default::default()
        },
    );

    if let Some(response) = response {
        let _ = response.clicked();
        let _ = response.changed();
    }
}

#[allow(dead_code)]
fn table_column_visibility_menu_items_api_compiles<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    model: &fret_runtime::Model<ImUiTableColumnVisibilityState>,
) {
    let columns = [
        TableColumn::fill("Name###asset-name"),
        TableColumn::px("Status###asset-status", Px(96.0)),
        TableColumn::unlabeled(TableColumnWidth::px(Px(72.0))).with_id("asset-actions"),
    ];

    let response = table_column_visibility_menu_items(
        ui,
        &columns,
        model,
        TableColumnVisibilityMenuOptions {
            test_id_prefix: Some("table.columns.".into()),
            ..Default::default()
        },
    );

    let _ = response.items();
    let _ = response.item("asset-status");
    let _ = response.len();
    let _ = response.is_empty();
    let _ = response.changed();
    if let Some(item) = response.item("asset-name") {
        let _ = item.column_id();
        let _ = item.visible();
        let _ = item.response();
        let _ = item.clicked();
        let _ = item.changed();
    }
}

#[allow(dead_code)]
fn table_column_visibility_header_context_menu_api_compiles<H: UiHost>(
    ui: &mut impl UiWriterImUiFacadeExt<H>,
    table: &TableResponse,
    model: &fret_runtime::Model<ImUiTableColumnVisibilityState>,
) {
    let columns = [
        TableColumn::fill("Name###asset-name").sortable(),
        TableColumn::px("Status###asset-status", Px(96.0)),
    ];

    let response = table_column_visibility_header_context_menu(
        ui,
        "table.columns",
        table,
        &columns,
        model,
        TableColumnVisibilityHeaderContextMenuOptions {
            menu: TableColumnVisibilityMenuOptions {
                test_id_prefix: Some("table.columns.".into()),
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let _ = response.open();
    let _ = response.changed();
    let _ = response.items().item("asset-status");
}

#[test]
fn table_option_defaults_compile() {
    let options = TableOptions::default();
    assert!(options.show_header);
    assert!(!options.striped);
    assert!(options.clip_cells);
    assert!(options.test_id.is_none());

    let row_options = TableRowOptions::default();
    assert!(row_options.test_id.is_none());
    assert!(row_options.background.is_none());

    let cell_options = TableCellOptions::default();
    assert!(cell_options.test_id.is_none());
    assert!(cell_options.background.is_none());
}

#[test]
fn table_column_helpers_compile() {
    let fill = TableColumn::fill("Name");
    assert_eq!(fill.header.as_deref(), Some("Name"));
    assert_eq!(fill.id.as_deref(), Some("Name"));
    assert_eq!(fill.width, TableColumnWidth::Fill(1.0));
    assert!(fill.visible);
    assert!(!fill.sortable);
    assert_eq!(fill.sort_direction, None);
    assert!(fill.resize.is_none());

    let weighted = TableColumn::weighted("Kind", 2.5);
    assert_eq!(weighted.header.as_deref(), Some("Kind"));
    assert_eq!(weighted.id.as_deref(), Some("Kind"));
    assert_eq!(weighted.width, TableColumnWidth::Fill(2.5));
    assert!(weighted.visible);
    assert!(!weighted.sortable);
    assert_eq!(weighted.sort_direction, None);
    assert!(weighted.resize.is_none());

    let px = TableColumn::px("State", Px(96.0));
    assert_eq!(px.header.as_deref(), Some("State"));
    assert_eq!(px.id.as_deref(), Some("State"));
    assert_eq!(px.width, TableColumnWidth::Px(Px(96.0)));
    assert!(px.visible);
    assert!(!px.sortable);
    assert_eq!(px.sort_direction, None);
    assert!(px.resize.is_none());

    let double_hash = TableColumn::fill("Name##asset-name-column");
    assert_eq!(
        double_hash.header.as_deref(),
        Some("Name##asset-name-column")
    );
    assert_eq!(double_hash.id.as_deref(), Some("Name##asset-name-column"));

    let stable = TableColumn::px("Status###status-column", Px(96.0));
    assert_eq!(stable.header.as_deref(), Some("Status###status-column"));
    assert_eq!(stable.id.as_deref(), Some("status-column"));

    let unlabeled = TableColumn::unlabeled(TableColumnWidth::px(Px(72.0))).with_id("actions");
    assert_eq!(unlabeled.header, None);
    assert_eq!(unlabeled.id.as_deref(), Some("actions"));
}

#[test]
fn table_column_visibility_helpers_compile() {
    let hidden = TableColumn::px("Internal###internal", Px(64.0)).hidden();
    assert_eq!(hidden.id.as_deref(), Some("internal"));
    assert!(!hidden.visible);

    let visible_again = hidden.with_visible(true);
    assert!(visible_again.visible);
}

#[test]
fn table_column_visibility_state_applies_runtime_visibility_by_column_id() {
    let columns = [
        TableColumn::fill("Name###asset-name"),
        TableColumn::px("Status###asset-status", Px(96.0)),
        TableColumn::px("Owner###asset-owner", Px(88.0)).hidden(),
    ];
    let state = ImUiTableColumnVisibilityState::new([
        ("asset-status", false),
        ("asset-owner", true),
        ("missing", false),
    ]);

    let applied = state.apply_to_columns(&columns);

    assert!(applied[0].visible);
    assert!(!applied[1].visible);
    assert!(applied[2].visible);
    assert_eq!(state.visibility_for("asset-status"), Some(false));
    assert!(state.is_visible("asset-owner", false));
}

#[test]
fn table_resizable_column_api_compiles() {
    let default_resize = TableColumn::px("Name###asset-name", Px(180.0)).resizable();
    let resize = default_resize.resize.expect("default resize options");
    assert_eq!(default_resize.id.as_deref(), Some("asset-name"));
    assert_eq!(resize.min_width, Some(Px(32.0)));
    assert_eq!(resize.max_width, None);

    let limited = TableColumn::weighted("Kind###asset-kind", 1.5)
        .resizable_with_limits(Some(Px(72.0)), Some(Px(280.0)));
    let resize = limited.resize.expect("limited resize options");
    assert_eq!(limited.id.as_deref(), Some("asset-kind"));
    assert_eq!(resize.min_width, Some(Px(72.0)));
    assert_eq!(resize.max_width, Some(Px(280.0)));
}

#[test]
fn table_sortable_header_api_compiles() {
    let sorted = TableColumn::fill("Name###asset-name")
        .sortable()
        .sorted(TableSortDirection::Ascending);
    assert_eq!(sorted.id.as_deref(), Some("asset-name"));
    assert!(sorted.sortable);
    assert_eq!(sorted.sort_direction, Some(TableSortDirection::Ascending));

    let unsorted = TableColumn::px("Status###asset-status", Px(120.0)).sortable();
    assert_eq!(unsorted.id.as_deref(), Some("asset-status"));
    assert!(unsorted.sortable);
    assert_eq!(unsorted.sort_direction, None);

    let descending = TableColumn::weighted("Kind###asset-kind", 1.5)
        .with_sort_direction(Some(TableSortDirection::Descending));
    assert_eq!(descending.id.as_deref(), Some("asset-kind"));
    assert!(descending.sortable);
    assert_eq!(
        descending.sort_direction,
        Some(TableSortDirection::Descending)
    );
}
