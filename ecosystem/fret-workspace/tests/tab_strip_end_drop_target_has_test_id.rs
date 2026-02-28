use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId};
use fret_core::{SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::CommandId;
use fret_ui::element::{ColumnProps, LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_workspace::{WorkspaceTab, WorkspaceTabStrip};
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

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "workspace-tab-strip-drop-end",
        move |cx| {
            let tabs = [
                WorkspaceTab::new(
                    Arc::from("a"),
                    Arc::from("A"),
                    CommandId::from("test.workspace.tab.activate.a"),
                ),
                WorkspaceTab::new(
                    Arc::from("b"),
                    Arc::from("B"),
                    CommandId::from("test.workspace.tab.activate.b"),
                ),
            ];

            let tab_strip = WorkspaceTabStrip::new("a")
                .test_id_root("tabstrip")
                .tab_test_id_prefix("tabstrip-tab")
                .tabs(tabs)
                .into_element(cx);

            let focus_target = cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Px(Px(40.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("FocusTarget")),
                        test_id: Some(Arc::from("focus-target")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx, _st| vec![],
            );

            let mut col = ColumnProps::default();
            col.layout.size.width = Length::Fill;
            col.layout.size.height = Length::Fill;
            vec![cx.column(col, move |_cx| vec![focus_target, tab_strip])]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn tab_strip_end_drop_target_has_deterministic_test_id() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(420.0), Px(120.0)),
    );

    render_frame(&mut ui, &mut app, &mut services, window, bounds);
    render_frame(&mut ui, &mut app, &mut services, window, bounds);

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.test_id.as_deref() == Some("tabstrip.drop_end")),
        "expected end drop target to expose a deterministic test_id"
    );
}
