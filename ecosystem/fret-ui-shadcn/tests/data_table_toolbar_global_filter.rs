use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

use fret_ui_headless::table::{ColumnDef, TableState};

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_text_input};

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    state: fret_runtime::Model<TableState>,
    columns: Arc<[ColumnDef<()>]>,
    _revision: u64,
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
        "data-table-toolbar-global-filter",
        move |cx| {
            let toolbar =
                fret_ui_shadcn::DataTableToolbar::new(state.clone(), columns.clone(), |col| {
                    Arc::clone(&col.id)
                })
                .show_global_filter(true)
                .show_columns_menu(false)
                .show_pinning_menu(false)
                .show_selected_text(false)
                .into_element(cx);
            vec![toolbar]
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
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

#[test]
fn data_table_toolbar_global_filter_updates_table_state_and_resets_page_index() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> =
        Arc::from(vec![ColumnDef::<()>::new("id")].into_boxed_slice());
    let mut state_value = TableState::default();
    state_value.pagination.page_index = 2;
    let state = app.models_mut().insert(state_value);

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
            state.clone(),
            columns.clone(),
            1,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let filter = find_by_role_and_label(&snap, SemanticsRole::TextField, "Global filter");
    click_at(&mut ui, &mut app, &mut services, rect_center(filter.bounds));

    dispatch_text_input(&mut ui, &mut app, &mut services, "  foo  ");

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            columns.clone(),
            2,
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.pagination.page_index, 0,
        "expected global filter to reset page index"
    );
    assert_eq!(
        st.global_filter.as_ref().and_then(|v| v.as_str()),
        Some("foo"),
        "expected global filter to trim and update TableState"
    );
}
