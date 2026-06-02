#![cfg(feature = "diagnostics")]

//! Material 3 chip semantics, layout, parts, and state-layer tests.

use fret_core::{
    AppWindowId, Axis, Paint, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsCheckedState,
    SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::action::{OnActivate, UiActionHostExt as _};
use fret_ui::element::{FlexProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{AssistChip, ChipSet, ChipSetItem, FilterChip, InputChip, SuggestionChip};

mod support;

use support::events::{pointer_down, pointer_move, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::{apply_material_theme, apply_material_theme_rtl};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(360.0)),
    )
}

fn render_chips(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    filter_selected: Model<bool>,
    input_selected: Model<bool>,
    assist_activations: Model<u32>,
    suggestion_activations: Model<u32>,
    trailing_activations: Model<u32>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(16.0).into();
        column.layout.size.width = Length::Px(Px(560.0));

        let content = cx.flex(column, |cx| {
            let assist_activations_for_handler = assist_activations.clone();
            let assist_activate: OnActivate =
                std::sync::Arc::new(move |host, action_cx, _reason| {
                    let _ = host.update_model(&assist_activations_for_handler, |v| *v += 1);
                    host.request_redraw(action_cx.window);
                });
            let suggestion_activations_for_handler = suggestion_activations.clone();
            let suggestion_activate: OnActivate =
                std::sync::Arc::new(move |host, action_cx, _reason| {
                    let _ = host.update_model(&suggestion_activations_for_handler, |v| *v += 1);
                    host.request_redraw(action_cx.window);
                });
            let trailing_activations_for_handler = trailing_activations.clone();
            let trailing_activate: OnActivate =
                std::sync::Arc::new(move |host, action_cx, _reason| {
                    let _ = host.update_model(&trailing_activations_for_handler, |v| *v += 1);
                    host.request_redraw(action_cx.window);
                });
            vec![
                ChipSet::new(vec![
                    ChipSetItem::from(
                        AssistChip::new("Assist")
                            .leading_icon(fret_icons::ids::ui::SEARCH)
                            .on_activate(assist_activate)
                            .test_id("m3-assist-chip"),
                    ),
                    ChipSetItem::from(
                        SuggestionChip::new("Suggest")
                            .leading_icon(fret_icons::ids::ui::SEARCH)
                            .on_activate(suggestion_activate)
                            .test_id("m3-suggestion-chip"),
                    ),
                    ChipSetItem::from(
                        FilterChip::new(filter_selected, "Filter")
                            .trailing_icon(fret_icons::ids::ui::CLOSE)
                            .on_trailing_icon_activate(trailing_activate.clone())
                            .test_id("m3-filter-chip"),
                    ),
                    ChipSetItem::from(
                        InputChip::new(input_selected, "Input")
                            .leading_icon(fret_icons::ids::ui::SEARCH)
                            .trailing_icon(fret_icons::ids::ui::CLOSE)
                            .on_trailing_icon_activate(trailing_activate.clone())
                            .test_id("m3-input-chip"),
                    ),
                ])
                .a11y_label("Material chip set")
                .test_id("m3-chip-set")
                .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_wrapping_chip_set(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.layout.size.width = Length::Px(Px(90.0));

        let content = cx.flex(column, |cx| {
            vec![
                ChipSet::new(vec![
                    ChipSetItem::from(SuggestionChip::new("Alpha").test_id("wrap-chip-a")),
                    ChipSetItem::from(SuggestionChip::new("Beta").test_id("wrap-chip-b")),
                    ChipSetItem::from(SuggestionChip::new("Gamma").test_id("wrap-chip-c")),
                ])
                .wrap_layout(true)
                .gap(Px(8.0))
                .a11y_label("Wrapping chip set")
                .test_id("wrap-chip-set")
                .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn paint(ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds(), &mut scene, 1.0);
    scene
}

fn semantics_node<'a>(ui: &'a UiTree<TestHost>, test_id: &str) -> &'a SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn visual_bounds_by_test_id(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    test_id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, test_id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn assert_size(rect: Rect, width: f32, height: f32, label: &str) {
    assert!(
        (rect.size.width.0 - width).abs() <= 0.01,
        "expected {label} width {width}, got {}",
        rect.size.width.0
    );
    assert!(
        (rect.size.height.0 - height).abs() <= 0.01,
        "expected {label} height {height}, got {}",
        rect.size.height.0
    );
}

fn assert_centered_vertically(outer: Rect, inner: Rect, label: &str) {
    let outer_center = outer.origin.y.0 + outer.size.height.0 * 0.5;
    let inner_center = inner.origin.y.0 + inner.size.height.0 * 0.5;
    let delta = (outer_center - inner_center).abs();
    assert!(
        delta <= 1.0,
        "expected {label} to be vertically centered, outer={outer:?}, inner={inner:?}, delta={delta}"
    );
}

fn state_layer_alphas_for_chrome(scene: &Scene, chrome: Rect) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } if rect.origin.x.0 <= chrome.origin.x.0 + chrome.size.width.0 + 0.1
                && rect.origin.x.0 + rect.size.width.0 >= chrome.origin.x.0 - 0.1
                && rect.origin.y.0 <= chrome.origin.y.0 + chrome.size.height.0 + 0.1
                && rect.origin.y.0 + rect.size.height.0 >= chrome.origin.y.0 - 0.1 =>
            {
                match background.paint {
                    Paint::Solid(color) if color.a > 0.0 && color.a < 0.2 => Some(color.a),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

struct ChipHarness {
    app: TestHost,
    window: AppWindowId,
    services: FakeUiServices,
    ui: UiTree<TestHost>,
    filter_selected: Model<bool>,
    input_selected: Model<bool>,
    assist_activations: Model<u32>,
    suggestion_activations: Model<u32>,
    trailing_activations: Model<u32>,
}

fn chip_harness() -> ChipHarness {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let filter_selected = app.models_mut().insert(true);
    let input_selected = app.models_mut().insert(false);
    let assist_activations = app.models_mut().insert(0_u32);
    let suggestion_activations = app.models_mut().insert(0_u32);
    let trailing_activations = app.models_mut().insert(0_u32);

    ChipHarness {
        app,
        window,
        services,
        ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    }
}

fn click_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    test_id: &str,
    pointer_id: u64,
) {
    click_test_id_at(ui, app, services, test_id, pointer_id, 0.5, 0.5);
}

fn click_test_id_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    test_id: &str,
    pointer_id: u64,
    x_fraction: f32,
    y_fraction: f32,
) {
    let node = semantics_node(ui, test_id);
    let bounds = ui
        .debug_node_visual_bounds(node.id)
        .unwrap_or_else(|| panic!("expected visual bounds for {test_id}"));
    let click_at = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * x_fraction),
        Px(bounds.origin.y.0 + bounds.size.height.0 * y_fraction),
    );
    ui.dispatch_event(
        app,
        services,
        &pointer_move(PointerId(pointer_id), click_at),
    );
    ui.dispatch_event(
        app,
        services,
        &pointer_down(PointerId(pointer_id), click_at),
    );
    ui.dispatch_event(app, services, &pointer_up(PointerId(pointer_id), click_at));
}

#[test]
fn chips_expose_material_roles_and_checked_state() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    } = chip_harness();
    render_chips(
        &mut ui,
        &mut app,
        &mut services,
        window,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    );

    let chip_set = semantics_node(&ui, "m3-chip-set");
    assert_eq!(chip_set.role, SemanticsRole::Group);
    assert_eq!(chip_set.label.as_deref(), Some("Material chip set"));

    for id in ["m3-assist-chip", "m3-suggestion-chip"] {
        let node = semantics_node(&ui, id);
        assert_eq!(node.role, SemanticsRole::Button);
        assert_eq!(node.flags.checked, None);
        assert_eq!(node.flags.checked_state, None);
    }

    let filter = semantics_node(&ui, "m3-filter-chip");
    assert_eq!(filter.role, SemanticsRole::Checkbox);
    assert_eq!(filter.flags.checked, Some(true));
    assert_eq!(
        filter.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );
    assert!(!filter.flags.selected);

    let input = semantics_node(&ui, "m3-input-chip");
    assert_eq!(input.role, SemanticsRole::Checkbox);
    assert_eq!(input.flags.checked, Some(false));
    assert_eq!(
        input.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
    assert!(!input.flags.selected);

    for (id, label) in [
        ("m3-filter-chip.trailing-icon", "Remove Filter"),
        ("m3-input-chip.trailing-icon", "Remove Input"),
    ] {
        let trailing = semantics_node(&ui, id);
        assert_eq!(trailing.role, SemanticsRole::Button);
        assert_eq!(trailing.label.as_deref(), Some(label));
        assert!(trailing.actions.invoke);
    }
}

#[test]
fn chips_route_primary_and_trailing_activation() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    } = chip_harness();
    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        render_chips(
            ui,
            app,
            services,
            window,
            filter_selected.clone(),
            input_selected.clone(),
            assist_activations.clone(),
            suggestion_activations.clone(),
            trailing_activations.clone(),
        );
    };

    render(&mut ui, &mut app, &mut services);
    click_test_id(&mut ui, &mut app, &mut services, "m3-assist-chip", 1);
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&assist_activations), Some(1));

    click_test_id(&mut ui, &mut app, &mut services, "m3-suggestion-chip", 2);
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&suggestion_activations), Some(1));

    click_test_id_at(
        &mut ui,
        &mut app,
        &mut services,
        "m3-filter-chip",
        3,
        0.25,
        0.5,
    );
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&filter_selected), Some(false));

    click_test_id_at(
        &mut ui,
        &mut app,
        &mut services,
        "m3-input-chip",
        4,
        0.25,
        0.5,
    );
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&input_selected), Some(true));

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "m3-filter-chip.trailing-icon",
        5,
    );
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&trailing_activations), Some(1));
    assert_eq!(
        app.models().get_copied(&filter_selected),
        Some(false),
        "trailing action should not toggle the chip primary checked state"
    );

    click_test_id(
        &mut ui,
        &mut app,
        &mut services,
        "m3-input-chip.trailing-icon",
        6,
    );
    app.advance_frame();
    render(&mut ui, &mut app, &mut services);
    assert_eq!(app.models().get_copied(&trailing_activations), Some(2));
    assert_eq!(
        app.models().get_copied(&input_selected),
        Some(true),
        "trailing action should not toggle the chip primary checked state"
    );
}

#[test]
fn chips_expose_material_touch_chrome_and_content_parts() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    } = chip_harness();
    render_chips(
        &mut ui,
        &mut app,
        &mut services,
        window,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    );

    for id in [
        "m3-assist-chip",
        "m3-suggestion-chip",
        "m3-filter-chip",
        "m3-input-chip",
    ] {
        let touch = ui
            .debug_node_visual_bounds(semantics_node(&ui, id).id)
            .unwrap_or_else(|| panic!("expected touch bounds for {id}"));
        assert!(
            (touch.size.height.0 - 48.0).abs() <= 0.01,
            "expected {id} touch height 48, got {}",
            touch.size.height.0
        );

        let chrome = visual_bounds_by_test_id(&ui, &app, window, &format!("{id}.chrome"));
        assert!(
            (chrome.size.height.0 - 32.0).abs() <= 0.01,
            "expected {id}.chrome height 32, got {}",
            chrome.size.height.0
        );
        assert_centered_vertically(touch, chrome, &format!("{id}.chrome"));

        let label = visual_bounds_by_test_id(&ui, &app, window, &format!("{id}.label"));
        assert_centered_vertically(chrome, label, &format!("{id}.label"));
    }

    for id in [
        "m3-assist-chip.leading-icon",
        "m3-suggestion-chip.leading-icon",
        "m3-filter-chip.leading-icon",
        "m3-input-chip.leading-icon",
        "m3-filter-chip.trailing-icon.glyph",
        "m3-input-chip.trailing-icon.glyph",
    ] {
        let icon = visual_bounds_by_test_id(&ui, &app, window, id);
        assert_size(icon, 18.0, 18.0, id);
    }

    for id in [
        "m3-filter-chip.trailing-icon",
        "m3-input-chip.trailing-icon",
    ] {
        let action = visual_bounds_by_test_id(&ui, &app, window, id);
        assert_size(action, 34.0, 48.0, id);
    }

    let assist = visual_bounds_by_test_id(&ui, &app, window, "m3-assist-chip");
    let suggestion = visual_bounds_by_test_id(&ui, &app, window, "m3-suggestion-chip");
    assert!(
        (suggestion.origin.x.0 - (assist.origin.x.0 + assist.size.width.0) - 8.0).abs() <= 1.0,
        "expected default ChipSet horizontal gap to be 8px, assist={assist:?}, suggestion={suggestion:?}"
    );
}

#[test]
fn chip_set_wrap_layout_keeps_material_gap_between_rows() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        ..
    } = chip_harness();
    render_wrapping_chip_set(&mut ui, &mut app, &mut services, window);

    let chip_set = semantics_node(&ui, "wrap-chip-set");
    assert_eq!(chip_set.role, SemanticsRole::Group);
    assert_eq!(chip_set.label.as_deref(), Some("Wrapping chip set"));

    let a = visual_bounds_by_test_id(&ui, &app, window, "wrap-chip-a");
    let b = visual_bounds_by_test_id(&ui, &app, window, "wrap-chip-b");
    let c = visual_bounds_by_test_id(&ui, &app, window, "wrap-chip-c");

    assert!(
        b.origin.y.0 > a.origin.y.0,
        "expected wrap-chip-b to wrap below wrap-chip-a, a={a:?}, b={b:?}"
    );
    assert!(
        c.origin.y.0 > b.origin.y.0,
        "expected wrap-chip-c to wrap below wrap-chip-b, b={b:?}, c={c:?}"
    );
    assert!(
        (b.origin.y.0 - (a.origin.y.0 + a.size.height.0) - 8.0).abs() <= 1.0,
        "expected wrapped ChipSet row gap to be 8px, a={a:?}, b={b:?}"
    );
    assert!(
        (c.origin.y.0 - (b.origin.y.0 + b.size.height.0) - 8.0).abs() <= 1.0,
        "expected wrapped ChipSet row gap to be 8px, b={b:?}, c={c:?}"
    );
}

#[test]
fn rtl_filter_and_input_chips_mirror_inline_content_edges() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    } = chip_harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    render_chips(
        &mut ui,
        &mut app,
        &mut services,
        window,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    );

    for base in ["m3-filter-chip", "m3-input-chip"] {
        let label = visual_bounds_by_test_id(&ui, &app, window, &format!("{base}.label"));
        let leading = visual_bounds_by_test_id(&ui, &app, window, &format!("{base}.leading-icon"));
        let trailing_glyph =
            visual_bounds_by_test_id(&ui, &app, window, &format!("{base}.trailing-icon.glyph"));
        let trailing_action =
            visual_bounds_by_test_id(&ui, &app, window, &format!("{base}.trailing-icon"));

        assert!(
            leading.origin.x.0 > label.origin.x.0,
            "expected {base} leading icon to sit on physical right in RTL, leading={leading:?}, label={label:?}"
        );
        assert!(
            trailing_glyph.origin.x.0 < label.origin.x.0,
            "expected {base} trailing glyph to sit on physical left in RTL, trailing={trailing_glyph:?}, label={label:?}"
        );
        assert!(
            trailing_action.origin.x.0 < label.origin.x.0,
            "expected {base} trailing action target to sit on physical left in RTL, action={trailing_action:?}, label={label:?}"
        );
    }
}

#[test]
fn chip_pressed_state_layer_animates_over_chrome() {
    let ChipHarness {
        mut app,
        window,
        mut services,
        mut ui,
        filter_selected,
        input_selected,
        assist_activations,
        suggestion_activations,
        trailing_activations,
    } = chip_harness();
    render_chips(
        &mut ui,
        &mut app,
        &mut services,
        window,
        filter_selected.clone(),
        input_selected.clone(),
        assist_activations.clone(),
        suggestion_activations.clone(),
        trailing_activations.clone(),
    );

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-input-chip.chrome");
    assert!(
        state_layer_alphas_for_chrome(&paint(&mut ui, &mut app, &mut services), chrome).is_empty(),
        "idle input chip should not paint a visible state layer"
    );

    let node = semantics_node(&ui, "m3-input-chip");
    let touch = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected input chip touch bounds");
    let press_at = Point::new(
        Px(touch.origin.x.0 + touch.size.width.0 * 0.25),
        Px(touch.origin.y.0 + touch.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), press_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );

    let mut animated = Vec::new();
    for _ in 0..4 {
        app.advance_frame();
        render_chips(
            &mut ui,
            &mut app,
            &mut services,
            window,
            filter_selected.clone(),
            input_selected.clone(),
            assist_activations.clone(),
            suggestion_activations.clone(),
            trailing_activations.clone(),
        );
        animated.extend(state_layer_alphas_for_chrome(
            &paint(&mut ui, &mut app, &mut services),
            chrome,
        ));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed chip state layer to animate through partial alpha, got {animated:?}"
    );
}
