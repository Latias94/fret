use fret_app::App;
use fret_core::{
    AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
};
use fret_core::{MaterialDescriptor, MaterialId, MaterialRegistrationError, MaterialService};
use fret_core::{Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService};
use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::CommandId;
use fret_ui::element::{LayoutStyle, Length, PressableA11y, PressableProps, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_workspace::{WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar};
use std::sync::Arc;

#[derive(Default)]
struct FakeServices;

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

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

#[test]
fn top_bar_tab_strip_does_not_overlap_left_menu() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(48.0)),
    );

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "workspace-top-bar-tab-strip-does-not-overlap-left",
        |cx| {
            let left = cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(220.0));
                        layout.size.height = Length::Px(Px(24.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::MenuBar),
                        test_id: Some(Arc::from("topbar-left-menu")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let tab_strip = WorkspaceTabStrip::new("a")
                .tabs([
                    WorkspaceTab::new(
                        Arc::from("a"),
                        Arc::from("Tab A (very long title)"),
                        CommandId::from("test.workspace.tab.activate.a"),
                    ),
                    WorkspaceTab::new(
                        Arc::from("b"),
                        Arc::from("Tab B (very long title)"),
                        CommandId::from("test.workspace.tab.activate.b"),
                    ),
                    WorkspaceTab::new(
                        Arc::from("c"),
                        Arc::from("Tab C (very long title)"),
                        CommandId::from("test.workspace.tab.activate.c"),
                    ),
                ])
                .into_element(cx);

            let center = cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Generic,
                    test_id: Some(Arc::from("topbar-center")),
                    ..Default::default()
                },
                move |_cx| vec![tab_strip],
            );

            vec![
                WorkspaceTopBar::new()
                    .left([left])
                    .center([center])
                    .right([cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Generic,
                            test_id: Some(Arc::from("topbar-right")),
                            ..Default::default()
                        },
                        |cx| {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(24.0));
                            vec![cx.semantics(
                                SemanticsProps {
                                    layout,
                                    role: SemanticsRole::Generic,
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            )]
                        },
                    )])
                    .into_element(cx),
            ]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let left_bounds = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("topbar-left-menu"))
        .map(|n| n.bounds)
        .expect("left menu node");
    let tab_list_bounds = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TabList)
        .map(|n| n.bounds)
        .expect("tab list node");

    let left_right = left_bounds.origin.x.0 + left_bounds.size.width.0;
    assert!(
        tab_list_bounds.origin.x.0 >= left_right - 0.01,
        "expected tab strip to start at/after left menu; left={left_bounds:?} tab_list={tab_list_bounds:?}"
    );

    let hit = ui.debug_hit_test(Point::new(Px(10.0), Px(16.0)));
    let hit = hit.hit.expect("expected hit");
    let hit_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == hit)
        .and_then(|n| n.test_id.as_deref())
        .unwrap_or("<none>");
    assert_eq!(
        hit_test_id, "topbar-left-menu",
        "expected clicks in left menu area to hit the menu, not the tab strip"
    );
}
