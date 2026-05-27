#![cfg(feature = "diagnostics")]

//! Stable automation-surface tests for Material 3 recipes.

use std::sync::Arc;

use fret_core::{AppWindowId, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_up};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn live_test_id_exists(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> bool {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .any(|m| ui.debug_node_visual_bounds(m.node).is_some())
}

#[test]
fn material3_select_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{Select, SelectItem, SelectVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha")
            .leading_icon(ids::ui::CHECK)
            .trailing_icon(ids::ui::CLOSE)
            .test_id("m3-select-item-alpha"),
        SelectItem::new("beta", "Beta").test_id("m3-select-item-beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            let selected_model = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let select = Select::new(selected_model)
                    .variant(SelectVariant::Filled)
                    .a11y_label("Material select")
                    .placeholder("Pick one")
                    .items(items)
                    .test_id("m3-select")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), select)]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    for id in [
        "m3-select",
        "m3-select.chrome",
        "m3-select.active-indicator",
        "m3-select.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Select part test_id {id}"
        );
    }

    let trigger =
        semantics_node_id_by_test_id(&ui, "m3-select").expect("expected m3-select semantics node");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger)
        .expect("expected select trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    for _ in 0..8 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        if live_test_id_exists(&ui, &app, window, "m3-select-listbox") {
            break;
        }
    }

    for id in [
        "m3-select-listbox",
        "m3-select-item-alpha",
        "m3-select-item-alpha.chrome",
        "m3-select-item-alpha.leading-icon",
        "m3-select-item-alpha.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Select popup/item part test_id {id}"
        );
    }
}

#[test]
fn material3_switch_exposes_stable_part_test_ids() {
    use fret_ui_material3::Switch;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let selected = app.models_mut().insert(false);
    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let switch = Switch::new(selected_model)
                    .icons(true)
                    .a11y_label("Material switch")
                    .test_id("m3-switch")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), switch)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-switch",
        "m3-switch.chrome",
        "m3-switch.track",
        "m3-switch.handle",
        "m3-switch.icon-on",
        "m3-switch.icon-off",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Switch part test_id {id}"
        );
    }
}
