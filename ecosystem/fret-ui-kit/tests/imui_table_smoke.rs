#![cfg(feature = "imui")]

use fret_core::Px;
use fret_ui::UiHost;
use fret_ui_kit::imui::{
    TableColumn, TableColumnWidth, TableOptions, TableRowOptions, UiWriterImUiFacadeExt,
};

#[allow(dead_code)]
fn table_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let columns = [
        TableColumn::fill("Name"),
        TableColumn::weighted("Kind", 2.0),
        TableColumn::unlabeled(TableColumnWidth::px(Px(72.0))),
    ];

    ui.table("table.basic", &columns, |table| {
        table.row("alpha", |row| {
            row.cell_text("Alpha");
            row.cell_text("Folder");
            row.cell(|ui| {
                let _ = ui.button("Open");
            });
        });
    });

    ui.table_with_options(
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
                },
                |row| {
                    row.cell_text("Beta");
                    row.cell_text("Queue");
                    row.cell_text("42");
                },
            );
        },
    );
}

#[test]
fn table_option_defaults_compile() {
    let options = TableOptions::default();
    assert!(options.show_header);
    assert!(!options.striped);
    assert!(options.clip_cells);
    assert!(options.test_id.is_none());
}

#[test]
fn table_column_helpers_compile() {
    let fill = TableColumn::fill("Name");
    assert_eq!(fill.header.as_deref(), Some("Name"));
    assert_eq!(fill.id.as_deref(), Some("Name"));
    assert_eq!(fill.width, TableColumnWidth::Fill(1.0));

    let weighted = TableColumn::weighted("Kind", 2.5);
    assert_eq!(weighted.header.as_deref(), Some("Kind"));
    assert_eq!(weighted.id.as_deref(), Some("Kind"));
    assert_eq!(weighted.width, TableColumnWidth::Fill(2.5));

    let px = TableColumn::px("State", Px(96.0));
    assert_eq!(px.header.as_deref(), Some("State"));
    assert_eq!(px.id.as_deref(), Some("State"));
    assert_eq!(px.width, TableColumnWidth::Px(Px(96.0)));

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
