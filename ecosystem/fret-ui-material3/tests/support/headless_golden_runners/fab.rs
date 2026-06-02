use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, NodeId, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

use super::scale_segment;
use crate::support::{
    events::{key_down, key_up, pointer_move},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_fab_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{Fab, FabSize, FabVariant};

    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(240.0)),
            );

            let render =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "fab_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                            let content = cx.flex(props, |cx| {
                                let row =
                                    |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                     variant: FabVariant,
                                     id_prefix: &'static str| {
                                        let mut props = FlexProps::default();
                                        props.direction = fret_core::Axis::Horizontal;
                                        props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
                                        cx.flex(props, move |cx| {
                                            vec![
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .a11y_label("fab")
                                                    .test_id(format!("{id_prefix}-fab"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Small)
                                                    .a11y_label("fab small")
                                                    .test_id(format!("{id_prefix}-fab-small"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Large)
                                                    .a11y_label("fab large")
                                                    .test_id(format!("{id_prefix}-fab-large"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .label("Create")
                                                    .test_id(format!("{id_prefix}-extended-fab"))
                                                    .into_element(cx),
                                            ]
                                        })
                                    };

                                vec![
                                    row(cx, FabVariant::Surface, "fab-surface"),
                                    row(cx, FabVariant::Primary, "fab-primary"),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            ui.set_focus(None);
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );

            let fab_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("fab-surface-fab")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected fab-surface-fab in semantics snapshot ({label}, {scale})")
                });

            let bounds_message =
                format!("expected fab-surface bounds in headless suite ({label}, {scale})");
            let fab_bounds = ui
                .debug_node_visual_bounds(fab_node)
                .unwrap_or_else(|| panic!("{bounds_message}"));
            let fab_center = Point::new(
                Px(fab_bounds.origin.x.0 + fab_bounds.size.width.0 * 0.5),
                Px(fab_bounds.origin.y.0 + fab_bounds.size.height.0 * 0.5),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 fab idle scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), fab_center),
            );
            let hover_message = format!(
                "expected the Material3 fab hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(fab_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 fab focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &focus_visible_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-fab.{scale}.{label}"),
                "material3_headless_fab_suite_goldens_v1",
                &suite,
            );
        }
    }
}
