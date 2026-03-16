use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade::{
    DataTableFacetedFilterOption, DataTableToolbar, DataTableToolbarResponsiveQuery,
    themes as shadcn_themes,
};
use std::sync::Arc;

use fret_ui_headless::table::{ColumnDef, ColumnFilter, TableState};
use serde_json::Value;

struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    state: fret_runtime::Model<TableState>,
    columns: Arc<[ColumnDef<()>]>,
    toolbar_container_width: Option<Px>,
    query: DataTableToolbarResponsiveQuery,
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
        "data-table-toolbar-faceted-responsive",
        move |cx| {
            let toolbar =
                DataTableToolbar::new(state.clone(), columns.clone(), |col| Arc::clone(&col.id))
                    .show_global_filter(false)
                    .show_columns_menu(false)
                    .show_pinning_menu(false)
                    .show_selected_text(false)
                    .faceted_filter_options(
                        "status",
                        "Status",
                        Arc::from(
                            vec![
                                DataTableFacetedFilterOption::new("open", "Open"),
                                DataTableFacetedFilterOption::new("closed", "Closed"),
                                DataTableFacetedFilterOption::new("backlog", "Backlog"),
                            ]
                            .into_boxed_slice(),
                        ),
                    )
                    .faceted_selected_badges_query(query)
                    .into_element(cx);

            let toolbar = cx.semantics(
                SemanticsProps {
                    test_id: Some(Arc::<str>::from("data-table-toolbar-root")),
                    ..Default::default()
                },
                move |_cx| [toolbar],
            );

            let width = toolbar_container_width
                .map(Length::Px)
                .unwrap_or(Length::Fill);

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |_cx| [toolbar],
            )]
        },
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn find_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
}

#[test]
fn data_table_toolbar_faceted_filter_badges_follow_viewport_breakpoint_by_default() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![
            ColumnDef::<()>::new("status"),
            ColumnDef::<()>::new("priority"),
            ColumnDef::<()>::new("title"),
        ]
        .into_boxed_slice(),
    );
    let mut state_value = TableState::default();
    state_value.column_filters.push(ColumnFilter {
        column: Arc::from("status"),
        value: Value::Array(vec![
            Value::String("open".to_string()),
            Value::String("closed".to_string()),
        ]),
    });
    let state = app.models_mut().insert(state_value);

    // Below `lg`: expect the count badge to be present.
    let bounds_sm = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(400.0)),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds_sm,
        state.clone(),
        columns.clone(),
        None,
        DataTableToolbarResponsiveQuery::Viewport,
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-count").is_some(),
        "expected count badge below lg breakpoint"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-0").is_none(),
        "expected label badges to be hidden below lg breakpoint"
    );

    // At/above `lg`: expect label badges to be present.
    let bounds_lg = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1200.0), Px(400.0)),
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds_lg,
        state.clone(),
        columns.clone(),
        None,
        DataTableToolbarResponsiveQuery::Viewport,
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-count").is_none(),
        "expected count badge to be hidden at/above lg breakpoint"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-0").is_some(),
        "expected label badges at/above lg breakpoint"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-1").is_some(),
        "expected label badges at/above lg breakpoint"
    );
}

#[test]
fn data_table_toolbar_faceted_filter_badges_can_follow_container_width() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let columns: Arc<[ColumnDef<()>]> = Arc::from(
        vec![
            ColumnDef::<()>::new("status"),
            ColumnDef::<()>::new("priority"),
            ColumnDef::<()>::new("title"),
        ]
        .into_boxed_slice(),
    );
    let mut state_value = TableState::default();
    state_value.column_filters.push(ColumnFilter {
        column: Arc::from("status"),
        value: Value::Array(vec![
            Value::String("open".to_string()),
            Value::String("closed".to_string()),
        ]),
    });
    let state = app.models_mut().insert(state_value);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1200.0), Px(400.0)),
    );

    // Narrow container in a wide viewport: after the container-query region has committed bounds,
    // the count badge should win.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        state.clone(),
        columns.clone(),
        Some(Px(600.0)),
        DataTableToolbarResponsiveQuery::Container,
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        state.clone(),
        columns.clone(),
        Some(Px(600.0)),
        DataTableToolbarResponsiveQuery::Container,
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-count").is_some(),
        "expected count badge for narrow container"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-0").is_none(),
        "expected label badges to be hidden for narrow container"
    );

    // Wide container: label badges should win.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        state.clone(),
        columns.clone(),
        Some(Px(1200.0)),
        DataTableToolbarResponsiveQuery::Container,
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        state.clone(),
        columns,
        Some(Px(1200.0)),
        DataTableToolbarResponsiveQuery::Container,
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-count").is_none(),
        "expected count badge to be hidden for wide container"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-0").is_some(),
        "expected label badges for wide container"
    );
    assert!(
        find_test_id(&snap, "data-table-toolbar-faceted-status-badge-label-1").is_some(),
        "expected label badges for wide container"
    );
}
