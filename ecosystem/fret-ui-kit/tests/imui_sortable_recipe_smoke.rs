#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{ResponseExt, UiWriterImUiFacadeExt};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, SortableRowOptions, SortableRowResponse, reorder_vec_by_key,
    sortable_row, sortable_row_with_options,
};

#[derive(Clone)]
#[allow(dead_code)]
struct DemoPayload {
    id: &'static str,
}

#[allow(dead_code)]
fn sortable_recipe_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let trigger = ResponseExt::default();

    let _: SortableRowResponse<DemoPayload> = sortable_row(ui, trigger, DemoPayload { id: "cube" });
    let _: SortableRowResponse<DemoPayload> = sortable_row_with_options(
        ui,
        trigger,
        DemoPayload { id: "light" },
        SortableRowOptions::default(),
    );
}

#[test]
fn sortable_insertion_side_labels_compile() {
    assert_eq!(SortableInsertionSide::Before.label(), "before");
    assert_eq!(SortableInsertionSide::After.label(), "after");
}

#[test]
fn reorder_vec_by_key_helper_compiles() {
    #[derive(Clone)]
    struct Item {
        id: &'static str,
    }

    let mut items = vec![Item { id: "a" }, Item { id: "b" }, Item { id: "c" }];
    assert!(reorder_vec_by_key(
        &mut items,
        "a",
        "b",
        SortableInsertionSide::After,
        |item| item.id,
    ));
}
