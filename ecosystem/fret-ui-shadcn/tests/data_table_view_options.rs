use fret_app::{App, Effect};
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::{ColumnDef, TableState};
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::click_at;

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

#[path = "support/timers.rs"]
mod timers;
use timers::TimerQueue;

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    state: Model<TableState>,
    columns: Arc<[ColumnDef<()>]>,
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
        "data-table-view-options",
        move |cx| {
            let view_options = shadcn::DataTableViewOptions::from_table_state(
                open.clone(),
                state.clone(),
                columns.clone(),
                |col| match col.id.as_ref() {
                    "status" => Arc::<str>::from("Status"),
                    "title" => Arc::<str>::from("Title"),
                    _ => col.id.clone(),
                },
            )
            .into_element(cx);
            vec![view_options]
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

fn has_role_and_label(
    snap: &fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> bool {
    snap.nodes
        .iter()
        .any(|n| n.role == role && n.label.as_deref() == Some(label))
}

fn mount_view_options(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: &Model<bool>,
    state: &Model<TableState>,
    columns: &Arc<[ColumnDef<()>]>,
) {
    for _ in 0..2 {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            open.clone(),
            state.clone(),
            columns.clone(),
        );
    }
}

fn open_view_options_menu(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    timers: &mut TimerQueue,
    window: AppWindowId,
    bounds: Rect,
    open: &Model<bool>,
    state: &Model<TableState>,
    columns: &Arc<[ColumnDef<()>]>,
) {
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, "View");
    click_at(ui, app, services, rect_center(trigger.bounds));
    pump_effects(ui, app, services, timers);

    let open_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..open_settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            open.clone(),
            state.clone(),
            columns.clone(),
        );
        pump_effects(ui, app, services, timers);
    }
}

#[test]
fn data_table_view_options_from_table_state_updates_column_visibility_and_resets_page_index() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    let status_col: Arc<str> = Arc::from("status");
    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![
            ColumnDef::<()>::new(status_col.clone()),
            ColumnDef::<()>::new("title"),
        ]
        .into_boxed_slice(),
    );
    let mut state_value = TableState::default();
    state_value.pagination.page_index = 2;
    let state = app.models_mut().insert(state_value);
    let open = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    mount_view_options(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );
    open_view_options_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let status_item = find_by_role_and_label(&snap, SemanticsRole::MenuItemCheckbox, "Status");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        rect_center(status_item.bounds),
    );
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    mount_view_options(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.column_visibility.get(&status_col).copied(),
        Some(false),
        "expected view-options checkbox toggle to update TableState.column_visibility"
    );
    assert_eq!(
        st.pagination.page_index, 0,
        "expected view-options column toggle to reset page index"
    );
}

#[test]
fn data_table_view_options_from_table_state_preserves_external_visibility_updates() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    let status_col: Arc<str> = Arc::from("status");
    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![
            ColumnDef::<()>::new(status_col.clone()),
            ColumnDef::<()>::new("title"),
        ]
        .into_boxed_slice(),
    );
    let mut state_value = TableState::default();
    state_value.pagination.page_index = 2;
    let state = app.models_mut().insert(state_value);
    let open = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    mount_view_options(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );

    let external_page_index = 5;
    let _ = app.models_mut().update(&state, |st| {
        st.column_visibility.insert(status_col.clone(), false);
        st.pagination.page_index = external_page_index;
    });

    mount_view_options(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.column_visibility.get(&status_col).copied(),
        Some(false),
        "expected external column_visibility update to remain authoritative"
    );
    assert_eq!(
        st.pagination.page_index, external_page_index,
        "expected view-options sync to avoid replaying stale local visibility back into TableState"
    );
}

#[test]
fn data_table_view_options_from_table_state_uses_group_leaf_columns() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    shadcn::themes::apply_shadcn_new_york(
        &mut app,
        shadcn::themes::ShadcnBaseColor::Neutral,
        shadcn::themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![ColumnDef::<()>::new("group").columns(vec![
            ColumnDef::<()>::new("status"),
            ColumnDef::<()>::new("title"),
        ])]
        .into_boxed_slice(),
    );
    let state = app.models_mut().insert(TableState::default());
    let open = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    mount_view_options(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );
    open_view_options_menu(
        &mut ui,
        &mut app,
        &mut services,
        &mut timers,
        window,
        bounds,
        &open,
        &state,
        &columns,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_role_and_label(&snap, SemanticsRole::MenuItemCheckbox, "Status"),
        "expected grouped columns to expose leaf Status entry"
    );
    assert!(
        has_role_and_label(&snap, SemanticsRole::MenuItemCheckbox, "Title"),
        "expected grouped columns to expose leaf Title entry"
    );
    assert!(
        !has_role_and_label(&snap, SemanticsRole::MenuItemCheckbox, "Group"),
        "expected grouped columns to skip non-leaf group headers"
    );
}
