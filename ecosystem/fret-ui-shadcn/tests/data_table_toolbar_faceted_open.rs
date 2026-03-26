use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

use fret_ui_headless::table::{ColumnDef, TableState};

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
    state: fret_runtime::Model<TableState>,
    columns: Arc<[ColumnDef<()>]>,
    request_semantics: bool,
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
        "data-table-toolbar-faceted-open",
        move |cx| {
            let toolbar = shadcn::DataTableToolbar::new(state.clone(), columns.clone(), |col| {
                Arc::clone(&col.id)
            })
            .show_global_filter(false)
            .show_columns_menu(false)
            .show_pinning_menu(false)
            .show_selected_text(false)
            .faceted_filter_options(
                "status",
                "Status",
                Arc::from(
                    vec![
                        shadcn::DataTableFacetedFilterOption::new("open", "Open"),
                        shadcn::DataTableFacetedFilterOption::new("closed", "Closed"),
                        shadcn::DataTableFacetedFilterOption::new("backlog", "Backlog"),
                    ]
                    .into_boxed_slice(),
                ),
            )
            .into_element(cx);
            vec![toolbar]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
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
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

#[test]
fn data_table_toolbar_faceted_trigger_opens_palette_input() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(320.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![
            ColumnDef::<()>::new("status"),
            ColumnDef::<()>::new("priority"),
            ColumnDef::<()>::new("title"),
        ]
        .into_boxed_slice(),
    );
    let state = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            columns.clone(),
            true,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "data-table-toolbar-faceted-status-trigger");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        rect_center(trigger.bounds),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    let settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            columns.clone(),
            tick + 1 == settle_frames,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    find_by_test_id(&snap, "data-table-toolbar-faceted-status-input");
}
