use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
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
use input_events::{click_at, dispatch_text_input};

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    state: fret_runtime::Model<TableState>,
    columns: Arc<[ColumnDef<()>]>,
    show_global_filter: bool,
    show_columns_menu: bool,
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
            let toolbar = shadcn::DataTableToolbar::new(state.clone(), columns.clone(), |col| {
                Arc::clone(&col.id)
            })
            .show_global_filter(show_global_filter)
            .show_columns_menu(show_columns_menu)
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

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

#[derive(Clone)]
struct RecipeRow {
    id: u64,
    name: Arc<str>,
}

fn render_recipe_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    recipe: shadcn::DataTableRecipe<RecipeRow>,
    rows: Arc<[RecipeRow]>,
    data_revision: u64,
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
        "data-table-recipe-prefixed-ids",
        move |cx| {
            recipe
                .into_elements(cx, rows, data_revision, |cx, _column, row| {
                    cx.text(row.name.as_ref())
                })
                .into_vec()
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn data_table_toolbar_global_filter_updates_table_state_and_resets_page_index() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
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
            true,
            false,
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
            true,
            false,
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

#[test]
fn data_table_toolbar_test_id_prefix_scopes_owned_inputs() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> =
        Arc::from(vec![ColumnDef::<()>::new("name")].into_boxed_slice());
    let state = app.models_mut().insert(TableState::default());

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let render = |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "data-table-toolbar-prefixed-ids",
            |cx| {
                let toolbar =
                    shadcn::DataTableToolbar::new(state.clone(), columns.clone(), |col| {
                        Arc::clone(&col.id)
                    })
                    .show_global_filter(true)
                    .column_filter("name")
                    .column_filter_a11y_label("Name filter")
                    .show_columns_menu(false)
                    .show_pinning_menu(false)
                    .show_selected_text(false)
                    .test_id_prefix("orders-toolbar")
                    .into_element(cx);
                vec![toolbar]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    };

    for _ in 0..2 {
        render(&mut ui, &mut app, &mut services);
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    find_by_test_id(&snap, "orders-toolbar-global-filter-input");
    find_by_test_id(&snap, "orders-toolbar-column-filter-input");
    assert!(
        snap.nodes.iter().all(|n| {
            !matches!(
                n.test_id.as_deref(),
                Some(
                    "data-table-toolbar-global-filter-input"
                        | "data-table-toolbar-column-filter-input"
                )
            )
        }),
        "prefixed toolbar should not also expose the historical unscoped input ids"
    );
}

#[test]
fn data_table_recipe_wires_toolbar_and_table_debug_ids() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let state = app.models_mut().insert(TableState::default());
    let output = app
        .models_mut()
        .insert(shadcn::DataTableViewOutput::default());
    let columns: Arc<[ColumnDef<RecipeRow>]> = Arc::from(
        vec![
            fret_ui_headless::table::create_column_helper::<RecipeRow>()
                .accessor_str("name", |row| row.name.as_ref()),
        ]
        .into_boxed_slice(),
    );
    let rows: Arc<[RecipeRow]> = Arc::from(
        vec![RecipeRow {
            id: 7,
            name: Arc::from("Ada"),
        }]
        .into_boxed_slice(),
    );
    let recipe = shadcn::DataTableRecipe::new(
        state.clone(),
        output.clone(),
        columns,
        |row, _index, _parent| shadcn::RowKey(row.id),
    )
    .column_labels([shadcn::DataTableColumnLabel::new("name", "Name")])
    .debug_ids(shadcn::TableDebugIds {
        header_row_test_id: Some(Arc::from("orders-header")),
        body_test_id: Some(Arc::from("orders-body")),
        header_cell_test_id_prefix: Some(Arc::from("orders-header-")),
        row_test_id_prefix: Some(Arc::from("orders-row-")),
        row_cell_test_ids: true,
    })
    .toolbar_test_id_prefix("orders-table");

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    for _ in 0..2 {
        render_recipe_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            recipe.clone(),
            rows.clone(),
            1,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    find_by_test_id(&snap, "orders-table-global-filter-input");
    find_by_test_id(&snap, "orders-header");
    find_by_test_id(&snap, "orders-body");

    let populated_output = app
        .models()
        .get_cloned(&output)
        .expect("recipe output after populated render");
    assert_eq!(populated_output.filtered_row_count, 1);
    assert_eq!(populated_output.pagination.page_count, 1);
    assert!(!populated_output.pagination.can_prev);
    assert!(!populated_output.pagination.can_next);

    app.models_mut()
        .update(&state, |table_state| {
            table_state.global_filter = Some(serde_json::Value::String("missing".to_string()));
        })
        .expect("set recipe filter");
    for _ in 0..2 {
        render_recipe_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            recipe.clone(),
            rows.clone(),
            1,
        );
    }

    let no_results_output = app
        .models()
        .get_cloned(&output)
        .expect("recipe output after filtering");
    assert_eq!(no_results_output.filtered_row_count, 0);
    assert_eq!(no_results_output.pagination.page_count, 0);
    assert!(!no_results_output.pagination.can_prev);
    assert!(!no_results_output.pagination.can_next);

    app.models_mut()
        .update(&state, |table_state| {
            table_state.global_filter = None;
        })
        .expect("clear recipe filter");
    let empty_rows: Arc<[RecipeRow]> = Arc::from(Vec::<RecipeRow>::new().into_boxed_slice());
    for _ in 0..2 {
        render_recipe_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            recipe.clone(),
            empty_rows.clone(),
            2,
        );
    }

    let empty_output = app
        .models()
        .get_cloned(&output)
        .expect("recipe output after empty render");
    assert_eq!(empty_output.filtered_row_count, 0);
    assert_eq!(empty_output.pagination.page_count, 0);
    assert!(!empty_output.pagination.can_prev);
    assert!(!empty_output.pagination.can_next);
}

#[test]
fn data_table_toolbar_external_column_visibility_update_does_not_get_overwritten() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(900.0), Px(240.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
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
            false,
            true,
        );
    }

    let external_page_index = 5;
    let _ = app.models_mut().update(&state, |st| {
        st.column_visibility.insert(status_col.clone(), false);
        st.pagination.page_index = external_page_index;
    });

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            state.clone(),
            columns.clone(),
            false,
            true,
        );
    }

    let st = app.models().get_cloned(&state).expect("table state");
    assert_eq!(
        st.column_visibility.get(&status_col).copied(),
        Some(false),
        "expected external column_visibility update to remain authoritative"
    );
    assert_eq!(
        st.pagination.page_index, external_page_index,
        "expected toolbar sync to avoid replaying stale local visibility back into TableState"
    );
}
