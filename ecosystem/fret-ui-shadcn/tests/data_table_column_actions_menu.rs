use fret_app::{App, Effect};
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::{ColumnDef, RowKey, SortSpec, TableState};
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press};

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

#[path = "support/timers.rs"]
mod timers;
use timers::TimerQueue;

#[derive(Debug, Clone)]
struct RowData {
    status: &'static str,
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
        "data-table-column-actions-menu",
        move |cx| {
            let table = fret_ui_shadcn::DataTable::new()
                .column_actions_menu(true)
                .into_element_retained(
                    cx,
                    data.clone(),
                    data_revision,
                    state.clone(),
                    columns.clone(),
                    |_row, index, _prev| RowKey::from_index(index),
                    |col| col.id.clone(),
                    |cx, col, row| match col.id.as_ref() {
                        "status" => cx.text(row.status),
                        "email" => cx.text(row.email),
                        _ => cx.text("?"),
                    },
                    None,
                    None,
                );
            vec![table]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_command_effects(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) -> bool {
    let effects = app.flush_effects();
    let mut dispatched = false;
    for effect in effects {
        match effect {
            Effect::Command { command, .. } => {
                dispatched |= ui.dispatch_command(app, services, &command);
            }
            other => app.push_effect(other),
        }
    }
    dispatched
}

fn pump_effects(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    timers: &mut TimerQueue,
) {
    // In the real app loop, Command effects are routed back through the UI tree and timers are
    // delivered as events. Tests need to do this explicitly.
    for _ in 0..4 {
        timers.ingest_effects(app);
        let dispatched = dispatch_command_effects(ui, app, services);
        timers.ingest_effects(app);
        timers.fire_all(ui, app, services);
        if !dispatched {
            break;
        }
    }
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
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

fn is_missing_or_hidden_by_label(
    snap: &fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> bool {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .is_none_or(|n| n.flags.hidden)
}

fn mount_table(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    data: &Arc<[RowData]>,
    state: &Model<TableState>,
    columns: &Arc<[ColumnDef<RowData>]>,
    revision: u64,
) {
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        data.clone(),
        revision,
        state.clone(),
        columns.clone(),
    );
    render_frame(
        ui,
        app,
        services,
        window,
        bounds,
        data.clone(),
        revision,
        state.clone(),
        columns.clone(),
    );
}

fn open_status_actions_menu(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    timers: &mut TimerQueue,
    window: AppWindowId,
    bounds: Rect,
    data: &Arc<[RowData]>,
    state: &Model<TableState>,
    columns: &Arc<[ColumnDef<RowData>]>,
    revision: u64,
) {
    let trigger_label = "Column actions for status";
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, trigger_label);
    click_at(ui, app, services, rect_center(trigger.bounds));
    pump_effects(ui, app, services, timers);

    let open_settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..open_settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            data.clone(),
            revision,
            state.clone(),
            columns.clone(),
        );
        if tick + 1 == open_settle_frames {
            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, trigger_label);
            assert!(
                trigger.flags.expanded,
                "expected trigger expanded while open"
            );
        }
    }
}

#[test]
fn data_table_column_actions_menu_sort_asc_updates_state_and_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(420.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        vec![
            RowData {
                status: "success",
                email: "a@example.com",
            },
            RowData {
                status: "pending",
                email: "b@example.com",
            },
        ]
        .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("status"), ColumnDef::new("email")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    mount_table(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &data,
        &state,
        &columns,
        1,
    );
    open_status_actions_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &data,
        &state,
        &columns,
        2,
    );

    // ArrowDown focuses the first menu item, Enter activates it.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);
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
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let focused_label = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused)
        .and_then(|n| n.label.as_deref());
    assert_eq!(
        focused_label,
        Some("Sort Asc"),
        "expected ArrowDown to focus the first menu item"
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..close_settle_frames {
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
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.sorting,
        vec![SortSpec {
            column: Arc::<str>::from("status"),
            desc: false
        }],
        "expected Sort Asc to update TableState.sorting"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        is_missing_or_hidden_by_label(&snap, SemanticsRole::MenuItem, "Sort Asc"),
        "expected menu items to be absent or hidden after selection"
    );
    let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, "Column actions for status");
    assert!(
        !trigger.flags.expanded,
        "expected trigger to collapse after selection"
    );
}

#[test]
fn data_table_column_actions_menu_hide_sets_column_visibility_false_and_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(420.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        vec![
            RowData {
                status: "success",
                email: "a@example.com",
            },
            RowData {
                status: "pending",
                email: "b@example.com",
            },
        ]
        .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("status"), ColumnDef::new("email")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    mount_table(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &data,
        &state,
        &columns,
        1,
    );
    open_status_actions_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &data,
        &state,
        &columns,
        2,
    );

    // From the trigger, ArrowDown focuses Sort Asc. Hide is the 4th focusable item.
    for _ in 0..4 {
        dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
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
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let focused_label = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused)
        .and_then(|n| n.label.as_deref());
    assert_eq!(focused_label, Some("Hide"));

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..close_settle_frames {
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
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    let status_col = Arc::<str>::from("status");
    assert_eq!(
        st.column_visibility.get(&status_col).copied(),
        Some(false),
        "expected Hide to set column_visibility[status]=false"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        is_missing_or_hidden_by_label(&snap, SemanticsRole::MenuItem, "Hide"),
        "expected menu to close after Hide"
    );
}

#[test]
fn data_table_column_actions_menu_pin_left_and_unpin_update_column_pinning() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(420.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let data: Arc<[RowData]> = Arc::from(
        vec![
            RowData {
                status: "success",
                email: "a@example.com",
            },
            RowData {
                status: "pending",
                email: "b@example.com",
            },
        ]
        .into_boxed_slice(),
    );
    let columns: Arc<[ColumnDef<RowData>]> =
        Arc::from(vec![ColumnDef::new("status"), ColumnDef::new("email")].into_boxed_slice());
    let state: Model<TableState> = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    mount_table(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &data,
        &state,
        &columns,
        1,
    );
    open_status_actions_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &data,
        &state,
        &columns,
        2,
    );

    // Pin Left is the 5th focusable item.
    for _ in 0..5 {
        dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
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
        );
    }
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..close_settle_frames {
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
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.column_pinning
            .left
            .iter()
            .map(|c| c.as_ref())
            .collect::<Vec<_>>(),
        vec!["status"],
        "expected Pin Left to add the status column to column_pinning.left"
    );

    // Re-open and Unpin (7th focusable item) to clear pinning.
    open_status_actions_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &data,
        &state,
        &columns,
        5,
    );
    for _ in 0..7 {
        dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            6,
            state.clone(),
            columns.clone(),
        );
    }
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..close_settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            data.clone(),
            7,
            state.clone(),
            columns.clone(),
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert!(
        st.column_pinning.left.is_empty() && st.column_pinning.right.is_empty(),
        "expected Unpin to clear column_pinning state"
    );
}
