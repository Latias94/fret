use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::table::PointerRowSelectionPolicy;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    data: Arc<[usize]>,
    data_revision: u64,
    state: Model<TableState>,
    columns: Arc<[ColumnDef<usize>]>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "data-table-row-range-selection",
        move |cx| {
            let table = shadcn::DataTable::new()
                .row_click_selection(true)
                .row_click_selection_policy(PointerRowSelectionPolicy::ListLike)
                .single_row_selection(false)
                .into_element_retained(
                    cx,
                    data.clone(),
                    data_revision,
                    state.clone(),
                    columns.clone(),
                    |_row, index, _prev| RowKey::from_index(index),
                    |col| col.id.clone(),
                    |cx, col, row| match col.id.as_ref() {
                        "id" => cx.text(format!("id={row}")),
                        _ => cx.text("?"),
                    },
                    None,
                    Some(Arc::<str>::from("data-table-row/")),
                );
            vec![table]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn rect_center(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node test_id={id:?}"))
}

fn click_at_with_modifiers(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
    modifiers: Modifiers,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Left,
            modifiers,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: PointerId(0),
            position,
            button: MouseButton::Left,
            modifiers,
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

#[test]
fn data_table_row_click_selection_list_like_shift_click_selects_range() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(360.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[usize]> = Arc::from((0..6usize).collect::<Vec<_>>().into_boxed_slice());
    let columns: Arc<[ColumnDef<usize>]> = Arc::from(vec![ColumnDef::new("id")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            1,
            state.clone(),
            columns.clone(),
        );
    }

    // Click row 1 to establish the anchor.
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let row_1 = find_by_test_id(&snap, "data-table-row/1");
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        rect_center(row_1.bounds),
        Modifiers::default(),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        data.clone(),
        2,
        state.clone(),
        columns.clone(),
    );
    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(st.row_selection.len(), 1);
    assert!(st.row_selection.contains(&RowKey(1)));

    // Shift-click row 4 should select a contiguous range (1..=4).
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let row_4 = find_by_test_id(&snap, "data-table-row/4");
    click_at_with_modifiers(
        &mut ui,
        &mut app,
        &mut services,
        rect_center(row_4.bounds),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        data,
        3,
        state.clone(),
        columns,
    );
    let st = app.models().get_cloned(&state).expect("table state");
    for key in [RowKey(1), RowKey(2), RowKey(3), RowKey(4)] {
        assert!(
            st.row_selection.contains(&key),
            "expected {key:?} in range selection"
        );
    }
    assert_eq!(st.row_selection.len(), 4);
}
