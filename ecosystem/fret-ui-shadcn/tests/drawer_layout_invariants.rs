use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
    TextMetrics, TextService,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use fret_ui_shadcn::shadcn_themes;
use shadcn::{Drawer, DrawerContent, DrawerSide};
use std::sync::Arc;

#[path = "support/shadcn_motion.rs"]
#[allow(dead_code)]
mod shadcn_motion;

const DRAWER_EDGE_GAP_PX: Px = Px(96.0);
const DRAWER_MAX_HEIGHT_FRACTION: f32 = 0.8;
const DRAWER_SIDE_PANEL_WIDTH_FRACTION: f32 = 0.75;
const DRAWER_SIDE_PANEL_MAX_WIDTH_PX: Px = Px(384.0);

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

fn find_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
}

fn drawer_vertical_max_height(viewport_height: Px) -> Px {
    let cap = (viewport_height.0 * DRAWER_MAX_HEIGHT_FRACTION).max(0.0);
    let by_gap = (viewport_height.0 - DRAWER_EDGE_GAP_PX.0).max(0.0);
    Px(cap.min(by_gap))
}

fn drawer_side_panel_width(viewport_width: Px) -> Px {
    Px((viewport_width.0 * DRAWER_SIDE_PANEL_WIDTH_FRACTION)
        .min(DRAWER_SIDE_PANEL_MAX_WIDTH_PX.0)
        .max(0.0))
}

fn render_open_drawer_and_get_content_bounds(
    bounds: Rect,
    side: DrawerSide,
    content_height: Px,
) -> fret_core::Rect {
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

    let open: Model<bool> = app.models_mut().insert(false);

    let render_frame = |ui: &mut UiTree<App>,
                        app: &mut App,
                        services: &mut dyn fret_core::UiServices,
                        request_semantics: bool,
                        open: Model<bool>| {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let open_for_root = open.clone();
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "drawer-layout-invariants",
            move |cx: &mut ElementContext<'_, App>| {
                let trigger = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                );

                let drawer = Drawer::new(open_for_root.clone()).side(side).into_element(
                    cx,
                    move |_cx| trigger,
                    move |cx| {
                        let content_child = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(content_height);
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        );

                        let content = DrawerContent::new(vec![content_child]).into_element(cx);
                        cx.semantics(
                            SemanticsProps {
                                test_id: Some(Arc::<str>::from("drawer.content")),
                                ..Default::default()
                            },
                            move |_cx| [content],
                        )
                    },
                );

                vec![drawer]
            },
        );

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        if request_semantics {
            ui.request_semantics_snapshot();
        }
        ui.layout_all(app, services, bounds, 1.0);
    };

    // Frame 1: mount closed so element ids are stable before opening overlays.
    render_frame(&mut ui, &mut app, &mut services, false, open.clone());

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and settle the transition before reading bounds.
    render_frame(&mut ui, &mut app, &mut services, false, open.clone());
    let settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            request_semantics,
            open.clone(),
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let node = find_test_id(&snap, "drawer.content").expect("drawer content semantics");
    node.bounds
}

#[test]
fn drawer_side_panel_width_tracks_viewport_fraction_and_caps() {
    let bounds_narrow = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(400.0), Px(400.0)),
    );
    let content =
        render_open_drawer_and_get_content_bounds(bounds_narrow, DrawerSide::Left, Px(0.0));
    let expected = drawer_side_panel_width(bounds_narrow.size.width);
    assert!(
        (content.size.width.0 - expected.0).abs() <= 2.0,
        "expected side drawer width to match w-3/4; got {:?}, expected {:?}",
        content.size.width,
        expected
    );

    let bounds_wide = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1200.0), Px(400.0)),
    );
    let content = render_open_drawer_and_get_content_bounds(bounds_wide, DrawerSide::Left, Px(0.0));
    let expected = drawer_side_panel_width(bounds_wide.size.width);
    assert!(
        (content.size.width.0 - expected.0).abs() <= 2.0,
        "expected side drawer width to cap at sm max width; got {:?}, expected {:?}",
        content.size.width,
        expected
    );
}

#[test]
fn drawer_bottom_height_caps_at_80vh_and_edge_gap() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(800.0), Px(400.0)),
    );
    let content = render_open_drawer_and_get_content_bounds(bounds, DrawerSide::Bottom, Px(2000.0));
    let expected_max = drawer_vertical_max_height(bounds.size.height);
    assert!(
        content.size.height.0 <= expected_max.0 + 2.0,
        "expected bottom drawer height to be capped; got {:?}, expected <= {:?}",
        content.size.height,
        expected_max
    );
    assert!(
        content.size.height.0 + 2.0 >= expected_max.0,
        "expected bottom drawer height to reach cap with large content; got {:?}, expected ~ {:?}",
        content.size.height,
        expected_max
    );
}
