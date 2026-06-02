use std::collections::BTreeMap;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::{Theme, UiTree, element::AnyElement};

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::apply_material_theme,
};

pub(crate) fn run_material3_headless_badge_suite_goldens_v1() {
    use fret_core::Corners;
    use fret_ui::element::{ContainerProps, FlexProps, Length};
    use fret_ui_material3::{Badge, BadgePlacement};

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(200.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "badge_root",
                    |cx| {
                        let theme = Theme::global(&*cx.app).clone();
                        let anchor_color = theme.color_token("md.sys.color.surface-container-low");

                        let anchor = |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                      size: Px| {
                            let mut props = ContainerProps::default();
                            props.layout.size.width = Length::Px(size);
                            props.layout.size.height = Length::Px(size);
                            props.background = Some(anchor_color);
                            props.corner_radii = Corners::all(Px(8.0));
                            cx.container(props, |_cx| Vec::<AnyElement>::new())
                        };

                        let mut props = FlexProps::default();
                        props.direction = fret_core::Axis::Horizontal;
                        props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                        props.align = fret_ui::element::CrossAlign::Center;
                        props.wrap = false;

                        let content = cx.flex(props, |cx| {
                            let small = Px(24.0);
                            vec![
                                Badge::dot()
                                    .navigation_anchor_size(small)
                                    .test_id("badge-dot-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::text("9")
                                    .navigation_anchor_size(small)
                                    .test_id("badge-text-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::dot()
                                    .placement(BadgePlacement::TopRight)
                                    .anchor_size(Px(40.0))
                                    .test_id("badge-dot-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                                Badge::text("99+")
                                    .placement(BadgePlacement::TopRight)
                                    .anchor_size(Px(40.0))
                                    .test_id("badge-text-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                            ]
                        });

                        vec![with_padding(cx, Px(24.0), content)]
                    },
                )
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 badge scene to be stable after animations settle ({label}, {scale})"
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

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-badge.{scale}.{label}"),
                "material3_headless_badge_suite_goldens_v1",
                &suite,
            );
        }
    }
}
