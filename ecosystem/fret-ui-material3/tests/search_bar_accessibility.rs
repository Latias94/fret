#![cfg(feature = "diagnostics")]

//! Focused Material 3 SearchBar accessibility semantics tests.

use fret_core::{AppWindowId, Point, Px, Rect, SemanticsNode, SemanticsRole, Size, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::SearchBar;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(160.0)),
    )
}

fn render_search_bar(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    query: Model<String>,
    expanded: Option<Model<bool>>,
    explicit_label: Option<&'static str>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut bar = SearchBar::new(query.clone())
            .placeholder("Search")
            .test_id("m3-search-bar");
        if let Some(expanded) = expanded.clone() {
            bar = bar.expanded_model(expanded);
        }
        if let Some(label) = explicit_label {
            bar = bar.a11y_label(label);
        }
        let bar = bar.into_element(cx);
        vec![with_padding(cx, Px(24.0), bar)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn search_bar_node(ui: &UiTree<TestHost>) -> &SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some("m3-search-bar"))
        })
        .expect("expected SearchBar input semantics node")
}

fn harness() -> (TestHost, AppWindowId, FakeUiServices, UiTree<TestHost>) {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    (app, window, services, ui)
}

#[test]
fn search_bar_uses_material_default_accessible_name_without_explicit_label() {
    let (mut app, window, mut services, mut ui) = harness();
    let query = app.models_mut().insert(String::new());

    render_search_bar(&mut ui, &mut app, &mut services, window, query, None, None);

    let node = search_bar_node(&ui);
    assert_eq!(node.role, SemanticsRole::TextField);
    assert_eq!(node.label.as_deref(), Some("Search"));
    assert_eq!(node.extra.placeholder.as_deref(), Some("Search"));
    assert!(!node.flags.expanded);
}

#[test]
fn search_bar_explicit_accessible_name_overrides_material_default() {
    let (mut app, window, mut services, mut ui) = harness();
    let query = app.models_mut().insert(String::new());

    render_search_bar(
        &mut ui,
        &mut app,
        &mut services,
        window,
        query,
        None,
        Some("Find project files"),
    );

    let node = search_bar_node(&ui);
    assert_eq!(node.label.as_deref(), Some("Find project files"));
}

#[test]
fn expanded_search_bar_publishes_suggestions_state_description() {
    let (mut app, window, mut services, mut ui) = harness();
    let query = app.models_mut().insert(String::new());
    let expanded = app.models_mut().insert(true);

    render_search_bar(
        &mut ui,
        &mut app,
        &mut services,
        window,
        query,
        Some(expanded),
        None,
    );

    let node = search_bar_node(&ui);
    assert!(node.flags.expanded);
    assert_eq!(
        node.extra.state_description.as_deref(),
        Some("Suggestions below")
    );
}
