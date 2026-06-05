use super::*;

use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

#[derive(Clone)]
struct TestDragPayload {
    label: Arc<str>,
}

#[derive(Clone, PartialEq, Eq)]
struct TestSortableItem {
    id: Arc<str>,
    label: Arc<str>,
}

#[derive(Clone)]
struct TestSortablePayload {
    id: Arc<str>,
    label: Arc<str>,
}

fn test_sortable_items() -> Vec<TestSortableItem> {
    vec![
        TestSortableItem {
            id: Arc::from("camera"),
            label: Arc::from("Camera"),
        },
        TestSortableItem {
            id: Arc::from("cube"),
            label: Arc::from("Cube"),
        },
        TestSortableItem {
            id: Arc::from("key-light"),
            label: Arc::from("Key light"),
        },
    ]
}

fn test_sortable_order_line(items: &[TestSortableItem]) -> String {
    items
        .iter()
        .map(|item| item.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn render_test_sortable_rows(
    items: &Rc<RefCell<Vec<TestSortableItem>>>,
    preview_status: &Rc<RefCell<String>>,
    delivered_status: &Rc<RefCell<String>>,
    order_status: &Rc<RefCell<String>>,
    delivered_flag: &Rc<Cell<bool>>,
) -> impl FnOnce(&mut ElementContext<'_, TestHost>) -> crate::Elements + use<> {
    let items = items.clone();
    let preview_status = preview_status.clone();
    let delivered_status = delivered_status.clone();
    let order_status = order_status.clone();
    let delivered_flag = delivered_flag.clone();

    move |cx| {
        crate::imui_raw(cx, |ui| {
            let snapshot = items.borrow().clone();
            let mut pending_reorder: Option<(
                Arc<str>,
                Arc<str>,
                Arc<str>,
                Arc<str>,
                SortableInsertionSide,
            )> = None;
            let mut preview = String::new();

            ui.vertical(|ui| {
                for item in &snapshot {
                    let row = ui.button_with_options(
                        item.label.clone(),
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from(format!("imui-sortable-row.{}", item.id))),
                            ..Default::default()
                        },
                    );
                    let payload = TestSortablePayload {
                        id: item.id.clone(),
                        label: item.label.clone(),
                    };
                    let sortable = sortable_row(ui, row, payload);

                    if let Some(signal) = sortable.delivered_reorder() {
                        let dragged = signal.payload();
                        if dragged.id != item.id {
                            pending_reorder = Some((
                                dragged.id.clone(),
                                dragged.label.clone(),
                                item.id.clone(),
                                item.label.clone(),
                                signal.side(),
                            ));
                        }
                    } else if let Some(signal) = sortable.preview_reorder() {
                        let dragged = signal.payload();
                        let side = signal.side();
                        if dragged.id != item.id {
                            preview = format!(
                                "Preview: move {} {} {}",
                                dragged.label,
                                side.label(),
                                item.label
                            );
                        }
                    }
                }
            });

            let mut delivered_message = String::new();
            let mut delivered = false;
            if let Some((active_id, active_label, over_id, over_label, side)) = pending_reorder {
                delivered = reorder_vec_by_key(
                    &mut items.borrow_mut(),
                    active_id.as_ref(),
                    over_id.as_ref(),
                    side,
                    |item| item.id.as_ref(),
                );
                if delivered {
                    delivered_message =
                        format!("Moved {} {} {}", active_label, side.label(), over_label);
                }
            }

            preview_status.replace(preview);
            delivered_status.replace(delivered_message);
            delivered_flag.set(delivered);
            order_status.replace(test_sortable_order_line(&items.borrow()));
        })
    }
}

mod collection_drag;
mod drag_core;
mod drag_preview;
mod multi_select;
mod sortable;
