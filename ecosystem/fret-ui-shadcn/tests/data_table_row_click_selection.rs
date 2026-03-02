use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::click_at;

#[derive(Debug, Clone)]
struct RowData {
    id: &'static str,
    email: &'static str,
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    data: Arc<[RowData]>,
    data_revision: u64,
    state: Model<TableState>,
    columns: Arc<[ColumnDef<RowData>]>,
    row_click_selection: bool,
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
        "data-table-row-click-selection",
        move |cx| {
            let table = fret_ui_shadcn::DataTable::new()
                .row_click_selection(row_click_selection)
                .into_element_retained(
                    cx,
                    data.clone(),
                    data_revision,
                    state.clone(),
                    columns.clone(),
                    |_row, index, _prev| RowKey::from_index(index),
                    |col| col.id.clone(),
                    |cx, col, row| match col.id.as_ref() {
                        "id" => cx.text(format!("id={}", row.id)),
                        "email" => cx.text(format!("email={}", row.email)),
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

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| {
            let mut sample: Vec<&str> = snap
                .nodes
                .iter()
                .filter_map(|n| n.test_id.as_deref())
                .filter(|tid| tid.starts_with("data-table-row/"))
                .take(16)
                .collect();
            sample.sort_unstable();
            panic!("missing semantics node with test_id={id:?}; sample row test_ids={sample:?}")
        })
}

fn rect_center(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

#[test]
fn data_table_row_click_selection_toggles_single_row_by_default() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(360.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        vec![
            RowData {
                id: "r0",
                email: "a@example.com",
            },
            RowData {
                id: "r1",
                email: "b@example.com",
            },
        ]
        .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("id"), ColumnDef::new("email")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        true,
    );
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
        true,
    );

    // Click row 1.
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let row_1 = find_by_test_id(&snap, "data-table-row/1");
    click_at(&mut ui, &mut app, &mut services, rect_center(row_1.bounds));

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
        true,
    );
    let st = app.models().get_cloned(&state).expect("table state");
    assert!(
        st.row_selection.contains(&RowKey(1)),
        "expected clicked row to be selected"
    );
    assert_eq!(
        st.row_selection.len(),
        1,
        "expected single-row selection by default"
    );

    // Clicking row 0 should replace the selection (single-row selection).
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let row_0 = find_by_test_id(&snap, "data-table-row/0");
    click_at(&mut ui, &mut app, &mut services, rect_center(row_0.bounds));
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
        true,
    );
    let st = app.models().get_cloned(&state).expect("table state");
    assert!(st.row_selection.contains(&RowKey(0)));
    assert!(!st.row_selection.contains(&RowKey(1)));
}

#[test]
fn data_table_row_click_selection_can_be_disabled() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(360.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        vec![
            RowData {
                id: "r0",
                email: "a@example.com",
            },
            RowData {
                id: "r1",
                email: "b@example.com",
            },
        ]
        .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("id"), ColumnDef::new("email")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

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
        false,
    );
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
        false,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let row_1 = find_by_test_id(&snap, "data-table-row/1");
    click_at(&mut ui, &mut app, &mut services, rect_center(row_1.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        data,
        2,
        state.clone(),
        columns,
        false,
    );

    let st = app.models().get_cloned(&state).expect("table state");
    assert!(
        st.row_selection.is_empty(),
        "expected no selection mutation when row_click_selection(false)"
    );
}
