use fret_app::{App, Effect};
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::tree::UiTree;
use fret_ui_headless::table::TableState;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;

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
    state: Model<TableState>,
    output: Model<shadcn::DataTableViewOutput>,
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
        "data-table-pagination-page-size",
        move |cx: &mut ElementContext<'_, App>| {
            let pagination =
                shadcn::DataTablePagination::new(state.clone(), output.clone()).into_element(cx);
            vec![pagination]
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

#[test]
fn data_table_pagination_page_size_dropdown_updates_state_and_resets_page_index() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(720.0), Px(240.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let mut state_value = TableState::default();
    state_value.pagination.page_index = 3;
    let state: Model<TableState> = app.models_mut().insert(state_value);
    let output: Model<shadcn::DataTableViewOutput> = app.models_mut().insert(Default::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    // Mount.
    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            output.clone(),
        );
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
    }

    // Open the "Rows per page" dropdown.
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, "Rows per page: 10");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        rect_center(trigger.bounds),
    );
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let open_settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..open_settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            output.clone(),
        );
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
        if tick + 1 == open_settle_frames {
            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            let trigger = find_by_role_and_label(&snap, SemanticsRole::Button, "Rows per page: 10");
            assert!(
                trigger.flags.expanded,
                "expected trigger expanded while open"
            );
            find_by_role_and_label(&snap, SemanticsRole::MenuItemRadio, "25");
        }
    }

    // Select "25" (should close the menu and update TableState on next render).
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let item = find_by_role_and_label(&snap, SemanticsRole::MenuItemRadio, "25");
    click_at(&mut ui, &mut app, &mut services, rect_center(item.bounds));
    pump_effects(&mut ui, &mut app, &mut services, &mut timers);

    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for _ in 0..close_settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            output.clone(),
        );
        pump_effects(&mut ui, &mut app, &mut services, &mut timers);
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.pagination.page_size, 25,
        "expected page_size to update from dropdown"
    );
    assert_eq!(
        st.pagination.page_index, 0,
        "expected page_index reset when page_size changes"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    find_by_role_and_label(&snap, SemanticsRole::Button, "Rows per page: 25");
    assert!(
        is_missing_or_hidden_by_label(&snap, SemanticsRole::MenuItemRadio, "25"),
        "expected menu items to be absent or hidden after selection"
    );
}
