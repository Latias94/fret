use fret_app::App;
use fret_core::scene::{Scene, SceneOp};
use fret_core::{
    AppWindowId, Color, Paint, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_ui::tree::UiTree;

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
                size: CoreSize::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
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

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    )
}

fn render_markdown_snapshot(source: &str) -> fret_core::SemanticsSnapshot {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "markdown_semantics",
        |cx| vec![crate::markdown(cx, source)],
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);
    ui.semantics_snapshot().expect("semantics snapshot").clone()
}

#[test]
fn markdown_inherits_foreground_scope_color_for_plain_text() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let expected = Color {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 1.0,
    };

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "markdown_foreground_scope_inheritance",
        |cx| vec![cx.foreground_scope(expected, |cx| vec![crate::markdown(cx, "Hello markdown")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds(), &mut scene, 1.0);

    let painted = scene.ops().iter().find_map(|op| match op {
        SceneOp::Text { paint, .. } => match paint.paint {
            Paint::Solid(color) => Some(color),
            _ => None,
        },
        _ => None,
    });
    assert_eq!(
        painted,
        Some(expected),
        "expected Markdown text to inherit ForegroundScope when no explicit color is set"
    );
}

#[test]
fn markdown_headings_publish_heading_role_and_level() {
    let snapshot = render_markdown_snapshot("# Title\n\n## Subtitle\n");

    let h1_test_id = crate::anchors::heading_anchor_test_id("Title");
    let h1 = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(h1_test_id.as_ref()))
        .expect("h1 heading semantics node (by test_id)");
    assert_eq!(h1.role, SemanticsRole::Heading);
    assert_eq!(h1.label.as_deref(), Some("Title"));
    assert_eq!(h1.extra.level, Some(1));

    let h2_test_id = crate::anchors::heading_anchor_test_id("Subtitle");
    let h2 = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(h2_test_id.as_ref()))
        .expect("h2 heading semantics node (by test_id)");
    assert_eq!(h2.role, SemanticsRole::Heading);
    assert_eq!(h2.label.as_deref(), Some("Subtitle"));
    assert_eq!(h2.extra.level, Some(2));
}

#[test]
fn markdown_heading_id_suffix_is_not_in_semantics_label() {
    let snapshot = render_markdown_snapshot("# Title {#custom-id}\n");

    let test_id = crate::anchors::heading_anchor_test_id_with_id("Title", Some("custom-id"));
    let heading = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id.as_ref()))
        .expect("heading semantics node (by explicit id test_id)");
    assert_eq!(heading.role, SemanticsRole::Heading);
    assert_eq!(heading.label.as_deref(), Some("Title"));
}
