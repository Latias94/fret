#![cfg(feature = "diagnostics")]

//! Stable automation-surface tests for Material 3 recipes.

use std::sync::Arc;

use fret_core::{AppWindowId, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_move, pointer_up};
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

#[test]
fn material3_tabs_exposes_stable_part_test_ids() {
    use fret_ui_material3::{TabItem, Tabs};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("overview"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let tabs = Tabs::new(selected.clone())
                    .a11y_label("Material tabs")
                    .test_id("m3-tabs")
                    .items(vec![
                        TabItem::new("overview", "Overview").test_id("m3-tab-overview"),
                        TabItem::new("settings", "Settings").test_id("m3-tab-settings"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), tabs)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-tabs",
        "m3-tabs.chrome",
        "m3-tabs.active-indicator",
        "m3-tab-overview",
        "m3-tab-overview.chrome",
        "m3-tab-settings",
        "m3-tab-settings.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Tabs part test_id {id}"
        );
    }
}

#[test]
fn material3_navigation_bar_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("search"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(selected.clone())
                    .a11y_label("Material navigation bar")
                    .test_id("m3-navigation-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .badge_dot()
                            .test_id("m3-nav-search"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .test_id("m3-nav-settings"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-navigation-bar",
        "m3-navigation-bar.chrome",
        "m3-navigation-bar.active-indicator",
        "m3-nav-search",
        "m3-nav-search.chrome",
        "m3-nav-search.icon",
        "m3-nav-search.label",
        "m3-nav-search.badge",
        "m3-nav-settings",
        "m3-nav-settings.chrome",
        "m3-nav-settings.icon",
        "m3-nav-settings.label",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live NavigationBar part test_id {id}"
        );
    }
}

#[test]
fn material3_navigation_rail_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(520.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("play"));
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(selected.clone())
                    .a11y_label("Material navigation rail")
                    .test_id("m3-navigation-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("m3-rail-search"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .badge_text("99+")
                            .test_id("m3-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-navigation-rail",
        "m3-navigation-rail.chrome",
        "m3-navigation-rail.active-indicator",
        "m3-rail-search",
        "m3-rail-search.chrome",
        "m3-rail-search.icon",
        "m3-rail-search.label",
        "m3-rail-play",
        "m3-rail-play.chrome",
        "m3-rail-play.icon",
        "m3-rail-play.label",
        "m3-rail-play.badge",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live NavigationRail part test_id {id}"
        );
    }
}

#[test]
fn material3_text_field_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{TextField, TextFieldVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let value = app.models_mut().insert(String::new());
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let field = TextField::new(value.clone())
                    .variant(TextFieldVariant::Filled)
                    .label("Email")
                    .placeholder("name@example.com")
                    .supporting_text("Required")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .test_id("m3-text-field")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), field)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-text-field",
        "m3-text-field.chrome",
        "m3-text-field.active-indicator",
        "m3-text-field.label",
        "m3-text-field.supporting-text",
        "m3-text-field.leading-icon",
        "m3-text-field.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live TextField part test_id {id}"
        );
    }
}

#[test]
fn material3_autocomplete_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(360.0)),
    );

    let query = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha").test_id("m3-autocomplete-alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let autocomplete = Autocomplete::new(query.clone())
                    .a11y_label("Material autocomplete")
                    .label("Search")
                    .placeholder("Type")
                    .supporting_text("Pick one")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_dropdown_icon(true)
                    .items(items)
                    .test_id("m3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), autocomplete)]
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
        "m3-autocomplete",
        "m3-autocomplete.chrome",
        "m3-autocomplete.label",
        "m3-autocomplete.supporting-text",
        "m3-autocomplete.leading-icon",
        "m3-autocomplete.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Autocomplete field part test_id {id}"
        );
    }

    let trigger = semantics_node_id_by_test_id(&ui, "m3-autocomplete")
        .expect("expected m3-autocomplete semantics node");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger)
        .expect("expected autocomplete bounds");
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
        if live_test_id_exists(&ui, &app, window, "m3-autocomplete.listbox") {
            break;
        }
    }

    for id in [
        "m3-autocomplete.listbox",
        "m3-autocomplete-alpha",
        "m3-autocomplete-alpha.chrome",
        "m3-autocomplete.option.beta",
        "m3-autocomplete.option.beta.chrome",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live Autocomplete popup/item part test_id {id}"
        );
    }
}

#[test]
fn material3_autocomplete_filled_exposes_active_indicator_part_test_id() {
    use fret_icons::ids;
    use fret_ui_material3::{Autocomplete, AutocompleteItem, AutocompleteVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(220.0)),
    );

    let query = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
    ]
    .into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let autocomplete = Autocomplete::new(query.clone())
                    .variant(AutocompleteVariant::Filled)
                    .a11y_label("Material autocomplete")
                    .label("Search")
                    .placeholder("Type")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_dropdown_icon(true)
                    .items(items)
                    .test_id("m3-autocomplete-filled")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), autocomplete)]
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
        "m3-autocomplete-filled",
        "m3-autocomplete-filled.chrome",
        "m3-autocomplete-filled.active-indicator",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live filled Autocomplete part test_id {id}"
        );
    }
}

#[test]
fn material3_search_bar_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::SearchBar;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(180.0)),
    );

    let query = app.models_mut().insert(String::new());
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = SearchBar::new(query.clone())
                    .a11y_label("Material search")
                    .placeholder("Search")
                    .leading_icon(ids::ui::SEARCH)
                    .trailing_icon(ids::ui::CLOSE)
                    .test_id("m3-search-bar")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [
        "m3-search-bar",
        "m3-search-bar.chrome",
        "m3-search-bar.leading-icon",
        "m3-search-bar.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live SearchBar part test_id {id}"
        );
    }
}

#[test]
fn material3_search_view_exposes_stable_part_test_ids() {
    use fret_icons::ids;
    use fret_ui_material3::SearchView;

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

    let open = app.models_mut().insert(true);
    let query = app.models_mut().insert(String::new());
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let view = SearchView::new(open.clone(), query.clone())
                .a11y_label("Material search view")
                .placeholder("Search")
                .leading_icon(ids::ui::SEARCH)
                .trailing_icon(ids::ui::CLOSE)
                .test_id("m3-search-view")
                .into_element(cx, |cx| {
                    vec![
                        cx.text_props(fret_ui::element::TextProps::new(Arc::<str>::from("Result"))),
                    ]
                });
            vec![with_padding(cx, Px(32.0), view)]
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

    for _ in 0..4 {
        if live_test_id_exists(&ui, &app, window, "m3-search-view.overlay") {
            break;
        }
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    for id in [
        "m3-search-view",
        "m3-search-view.chrome",
        "m3-search-view.leading-icon",
        "m3-search-view.trailing-icon",
        "m3-search-view.overlay",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected live SearchView part test_id {id}"
        );
    }
}

#[test]
fn material3_date_picker_exposes_stable_part_test_ids() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    {
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

        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedDatePicker::new(month.clone(), selected.clone())
                        .variant(DatePickerVariant::Docked)
                        .today(Some(today))
                        .test_id("m3-date-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-date-picker",
            "m3-date-picker.chrome",
            "m3-date-picker.docked.prev",
            "m3-date-picker.docked.next",
            "m3-date-picker.cell.0.0",
            "m3-date-picker.cell.5.6",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DatePicker part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let month = app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected = app.models_mut().insert(Some(selected_date));
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
                    .today(Some(today))
                    .open_duration_ms(Some(1))
                    .close_duration_ms(Some(1))
                    .test_id("m3-date-picker-modal")
                    .into_element(cx, |cx| {
                        Button::new("Underlay")
                            .variant(ButtonVariant::Outlined)
                            .test_id("m3-date-picker-underlay")
                            .into_element(cx)
                    });
                vec![with_padding(cx, Px(32.0), dialog)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-date-picker-modal.panel") {
                break;
            }
        }

        for id in [
            "m3-date-picker-modal.scrim",
            "m3-date-picker-modal.scrim.chrome",
            "m3-date-picker-modal.panel",
            "m3-date-picker-modal.modal.prev",
            "m3-date-picker-modal.modal.next",
            "m3-date-picker-modal.cell.0.0",
            "m3-date-picker-modal.actions.cancel",
            "m3-date-picker-modal.actions.confirm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DatePicker dialog part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_time_picker_exposes_stable_part_test_ids() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedTimePicker, TimePickerDialog, TimePickerDisplayMode,
    };
    use time::Time;

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Dial)
                        .test_id("m3-time-picker")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-time-picker",
            "m3-time-picker.chrome",
            "m3-time-picker.mode-toggle",
            "m3-time-picker.hour-selector",
            "m3-time-picker.hour-selector.chrome",
            "m3-time-picker.minute-selector",
            "m3-time-picker.minute-selector.chrome",
            "m3-time-picker.clock-dial",
            "m3-time-picker.clock-dial.chrome",
            "m3-time-picker.period.am",
            "m3-time-picker.period.pm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker dial part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(520.0)),
        );

        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let picker = DockedTimePicker::new(time.clone())
                        .display_mode(TimePickerDisplayMode::Input)
                        .test_id("m3-time-picker-input")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), picker)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-time-picker-input",
            "m3-time-picker-input.chrome",
            "m3-time-picker-input.mode-toggle",
            "m3-time-picker-input.input.hour",
            "m3-time-picker-input.input.hour.chrome",
            "m3-time-picker-input.input.minute",
            "m3-time-picker-input.input.minute.chrome",
            "m3-time-picker-input.input.period.am",
            "m3-time-picker-input.input.period.pm",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker input part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let time = app.models_mut().insert(selected_time);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = TimePickerDialog::new(open.clone(), time.clone())
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .test_id("m3-time-picker-modal")
                        .into_element(cx, |cx| {
                            Button::new("Underlay")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-time-picker-underlay")
                                .into_element(cx)
                        });
                    vec![with_padding(cx, Px(32.0), dialog)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-time-picker-modal.panel") {
                break;
            }
        }

        for id in [
            "m3-time-picker-modal.scrim",
            "m3-time-picker-modal.scrim.chrome",
            "m3-time-picker-modal.panel",
            "m3-time-picker-modal.actions.cancel",
            "m3-time-picker-modal.actions.confirm",
            "m3-time-picker-modal.clock-dial",
            "m3-time-picker-modal.clock-dial.chrome",
            "m3-time-picker-modal.hour-selector",
            "m3-time-picker-modal.minute-selector",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live TimePicker dialog part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_menu_and_dropdown_expose_stable_part_test_ids() {
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};
    use fret_ui_material3::{Button, ButtonVariant, DropdownMenu};

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(260.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = Menu::new()
                        .a11y_label("Material menu")
                        .test_id("m3-menu")
                        .entries(vec![
                            MenuEntry::Item(MenuItem::new("Alpha").test_id("m3-menu-alpha")),
                            MenuEntry::Separator,
                            MenuEntry::Item(MenuItem::new("Beta").test_id("m3-menu-beta")),
                        ])
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), menu)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in [
            "m3-menu",
            "m3-menu.chrome",
            "m3-menu-alpha",
            "m3-menu-alpha.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live Menu part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(360.0)),
        );

        let open = app.models_mut().insert(true);
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dropdown = DropdownMenu::new(open.clone())
                    .a11y_label("Material dropdown")
                    .test_id("m3-dropdown")
                    .into_element(
                        cx,
                        |cx| {
                            Button::new("Open")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-dropdown-trigger")
                                .into_element(cx)
                        },
                        |_cx| {
                            vec![
                                MenuEntry::Item(
                                    MenuItem::new("Alpha").test_id("m3-dropdown-alpha"),
                                ),
                                MenuEntry::Item(MenuItem::new("Beta").test_id("m3-dropdown-beta")),
                            ]
                        },
                    );
                vec![with_padding(cx, Px(32.0), dropdown)]
            })
        };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-dropdown.chrome") {
                break;
            }
        }

        for id in [
            "m3-dropdown",
            "m3-dropdown.chrome",
            "m3-dropdown-alpha",
            "m3-dropdown-alpha.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DropdownMenu part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_dialog_and_bottom_sheet_expose_stable_part_test_ids() {
    use fret_ui_material3::{
        Button, ButtonVariant, Dialog, DialogAction, DockedBottomSheet, DockedBottomSheetVariant,
        ModalBottomSheet,
    };

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(420.0)),
        );

        let open = app.models_mut().insert(true);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = Dialog::new(open.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .actions(vec![
                            DialogAction::new("Cancel").test_id("m3-dialog-action-cancel"),
                            DialogAction::new("OK").test_id("m3-dialog-action-confirm"),
                        ])
                        .test_id("m3-dialog")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Underlay")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("m3-dialog-underlay")
                                    .into_element(cx)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![with_padding(cx, Px(32.0), dialog)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-dialog.panel") {
                break;
            }
        }

        for id in [
            "m3-dialog.scrim",
            "m3-dialog.scrim.chrome",
            "m3-dialog.panel",
            "m3-dialog.panel.chrome",
            "m3-dialog-action-cancel",
            "m3-dialog-action-cancel.chrome",
            "m3-dialog-action-confirm",
            "m3-dialog-action-confirm.chrome",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live Dialog part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(420.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let sheet = DockedBottomSheet::new()
                        .variant(DockedBottomSheetVariant::Standard)
                        .test_id("m3-bottom-sheet")
                        .into_element(cx, |cx| {
                            vec![
                                Button::new("Action")
                                    .variant(ButtonVariant::Filled)
                                    .test_id("m3-bottom-sheet-action")
                                    .into_element(cx),
                            ]
                        });
                    vec![with_padding(cx, Px(32.0), sheet)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        for id in ["m3-bottom-sheet", "m3-bottom-sheet.drag-handle"] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live DockedBottomSheet part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(520.0)),
        );

        let open = app.models_mut().insert(true);
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let sheet = ModalBottomSheet::new(open.clone())
                        .open_duration_ms(Some(1))
                        .close_duration_ms(Some(1))
                        .test_id("m3-modal-bottom-sheet")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Underlay")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("m3-modal-bottom-sheet-underlay")
                                    .into_element(cx)
                            },
                            |cx| {
                                vec![
                                    Button::new("Action")
                                        .variant(ButtonVariant::Filled)
                                        .test_id("m3-modal-bottom-sheet-action")
                                        .into_element(cx),
                                ]
                            },
                        );
                    vec![with_padding(cx, Px(32.0), sheet)]
                })
            };

        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );
            if live_test_id_exists(&ui, &app, window, "m3-modal-bottom-sheet.sheet") {
                break;
            }
        }

        for id in [
            "m3-modal-bottom-sheet.scrim",
            "m3-modal-bottom-sheet.scrim.chrome",
            "m3-modal-bottom-sheet.sheet",
            "m3-modal-bottom-sheet.sheet.drag-handle",
        ] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live ModalBottomSheet part test_id {id}"
            );
        }
    }
}

#[test]
fn material3_tooltip_and_snackbar_expose_stable_part_test_ids() {
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{
        Button, ButtonVariant, PlainTooltip, Snackbar, SnackbarController, SnackbarHost,
        TooltipProvider,
    };

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(320.0)),
        );

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with_elements(cx, |cx| {
                            let trigger = Button::new("Trigger")
                                .variant(ButtonVariant::Outlined)
                                .test_id("m3-tooltip-trigger")
                                .into_element(cx);
                            let tooltip = PlainTooltip::new(trigger, "Tip")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .test_id("m3-tooltip")
                                .into_element(cx);
                            vec![with_padding(cx, Px(32.0), tooltip)]
                        })
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

        let trigger = semantics_node_id_by_test_id(&ui, "m3-tooltip-trigger")
            .expect("expected m3-tooltip-trigger semantics node");
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger)
            .expect("expected tooltip trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

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
            if live_test_id_exists(&ui, &app, window, "m3-tooltip") {
                break;
            }
        }

        for id in ["m3-tooltip", "m3-tooltip.chrome"] {
            assert!(
                live_test_id_exists(&ui, &app, window, id),
                "expected live PlainTooltip part test_id {id}"
            );
        }
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(860.0), Px(520.0)),
        );

        let store = app.models_mut().insert(ToastStore::default());
        let controller = SnackbarController::new(store.clone());
        {
            let mut action_host = UiActionHostAdapter { app: &mut app };
            let _ = controller.show(
                &mut action_host,
                window,
                Snackbar::new("Saved")
                    .supporting_text("Synced")
                    .test_id("m3-snackbar"),
            );
        }

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let store = store.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

        assert!(
            live_test_id_exists(&ui, &app, window, "m3-snackbar"),
            "expected live Snackbar toast root test_id m3-snackbar"
        );
    }
}
