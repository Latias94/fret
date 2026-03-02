use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::stack;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::click_at;

#[derive(Debug, Clone)]
struct RowData {
    id: usize,
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
    output: Model<fret_ui_shadcn::DataTableViewOutput>,
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
        "data-table-pagination",
        move |cx: &mut ElementContext<'_, App>| {
            let table = fret_ui_shadcn::DataTable::new()
                .output_model(output.clone())
                .into_element(
                    cx,
                    data.clone(),
                    data_revision,
                    state.clone(),
                    columns.clone(),
                    |_row, index, _prev| RowKey::from_index(index),
                    |col| col.id.clone(),
                    |cx, col, row| match col.id.as_ref() {
                        "id" => cx.text(format!("id={}", row.id)),
                        _ => cx.text("?"),
                    },
                );
            let pagination =
                fret_ui_shadcn::DataTablePagination::new(state.clone(), output.clone())
                    .into_element(cx);
            vec![stack::vstack(
                cx,
                stack::VStackProps::default(),
                move |_cx| vec![table, pagination],
            )]
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

fn find_by_role_and_label<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| {
            if role == SemanticsRole::Button {
                panic!(
                    "missing semantics node role={role:?} label={label:?}; button labels={:?}",
                    snapshot_button_labels(snap)
                );
            }
            panic!("missing semantics node role={role:?} label={label:?}")
        })
}

fn snapshot_button_labels(snap: &fret_core::SemanticsSnapshot) -> Vec<&str> {
    let mut labels: Vec<&str> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Button)
        .filter_map(|n| n.label.as_deref())
        .collect();
    labels.sort_unstable();
    labels
}

fn find_button_by_label<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    label: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(label))
}

fn is_missing_or_disabled_button(snap: &fret_core::SemanticsSnapshot, label: &str) -> bool {
    find_button_by_label(snap, label).is_none_or(|n| n.flags.disabled)
}

#[test]
fn data_table_pagination_buttons_update_page_index_and_disabled_states() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(520.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        (0..23usize)
            .map(|id| RowData { id })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("id")].into_boxed_slice());

    let state: Model<TableState> = app.models_mut().insert(TableState::default());
    let output: Model<fret_ui_shadcn::DataTableViewOutput> =
        app.models_mut().insert(Default::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frames 1..=3: allow `TableViewOutput` to populate, then allow pagination to read the
    // previous frame's output (output updates happen during table render/layout, so the pagination
    // view can lag by one frame).
    for _ in 0..3 {
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
            output.clone(),
        );
    }

    assert_eq!(
        app.models()
            .get_cloned(&state)
            .expect("table state")
            .pagination
            .page_index,
        0
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        is_missing_or_disabled_button(&snap, "Go to previous page"),
        "prev should be disabled (or absent) on first page; button labels={:?}",
        snapshot_button_labels(&snap)
    );
    assert!(
        is_missing_or_disabled_button(&snap, "Go to first page"),
        "first should be disabled (or absent) on first page; button labels={:?}",
        snapshot_button_labels(&snap)
    );
    assert!(
        snap.nodes.iter().any(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|l| l.starts_with("Page ") && l.contains('/'))
        }),
        "expected a page label button; button labels={:?}",
        snapshot_button_labels(&snap)
    );

    // Next page.
    let next = find_by_role_and_label(&snap, SemanticsRole::Button, "Go to next page");
    click_at(&mut ui, &mut app, &mut services, rect_center(next.bounds));
    for _ in 0..2 {
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
            output.clone(),
        );
    }
    assert_eq!(
        app.models()
            .get_cloned(&state)
            .expect("table state")
            .pagination
            .page_index,
        1
    );

    // Last page.
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let last = find_by_role_and_label(&snap, SemanticsRole::Button, "Go to last page");
    click_at(&mut ui, &mut app, &mut services, rect_center(last.bounds));
    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            3,
            state.clone(),
            columns.clone(),
            output.clone(),
        );
    }
    assert_eq!(
        app.models()
            .get_cloned(&state)
            .expect("table state")
            .pagination
            .page_index,
        2,
        "expected last page index for 23 rows @ page_size=10"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        is_missing_or_disabled_button(&snap, "Go to next page"),
        "next should be disabled (or absent) on last page; button labels={:?}",
        snapshot_button_labels(&snap)
    );
    assert!(
        is_missing_or_disabled_button(&snap, "Go to last page"),
        "last should be disabled (or absent) on last page; button labels={:?}",
        snapshot_button_labels(&snap)
    );

    // Prev page.
    let prev = find_by_role_and_label(&snap, SemanticsRole::Button, "Go to previous page");
    click_at(&mut ui, &mut app, &mut services, rect_center(prev.bounds));
    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            4,
            state.clone(),
            columns.clone(),
            output.clone(),
        );
    }
    assert_eq!(
        app.models()
            .get_cloned(&state)
            .expect("table state")
            .pagination
            .page_index,
        1
    );

    // First page.
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let first = find_by_role_and_label(&snap, SemanticsRole::Button, "Go to first page");
    click_at(&mut ui, &mut app, &mut services, rect_center(first.bounds));
    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            5,
            state.clone(),
            columns.clone(),
            output.clone(),
        );
    }
    assert_eq!(
        app.models()
            .get_cloned(&state)
            .expect("table state")
            .pagination
            .page_index,
        0
    );
}
