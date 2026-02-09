use fret_app::App;
use fret_core::{
    AppWindowId, Corners, Edges, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Scene, SceneOp, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService, Transform2D, UvRect,
};
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapPoint {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapSize {
    w: f32,
    h: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapRect {
    origin: SnapPoint,
    size: SnapSize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapEdges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapCorners {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct SnapColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum SnapSceneOp {
    PushTransform {
        transform: [f32; 6],
    },
    PopTransform,
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,
    PushLayer {
        layer: u32,
    },
    PopLayer,
    PushClipRect {
        rect: SnapRect,
    },
    PushClipRRect {
        rect: SnapRect,
        corner_radii: SnapCorners,
    },
    PopClip,
    Quad {
        rect: SnapRect,
        background: SnapColor,
        border: SnapEdges,
        border_color: SnapColor,
        corner_radii: SnapCorners,
    },
    Image {
        rect: SnapRect,
        opacity: f32,
    },
    ImageRegion {
        rect: SnapRect,
        uv: [f32; 4],
        opacity: f32,
    },
    MaskImage {
        rect: SnapRect,
        uv: [f32; 4],
        color: SnapColor,
        opacity: f32,
    },
    SvgMaskIcon {
        rect: SnapRect,
        fit: String,
        color: SnapColor,
        opacity: f32,
    },
    SvgImage {
        rect: SnapRect,
        fit: String,
        opacity: f32,
    },
    Text {
        origin: SnapPoint,
        color: SnapColor,
    },
    Path {
        origin: SnapPoint,
        color: SnapColor,
    },
    ViewportSurface {
        rect: SnapRect,
        opacity: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SnapSemanticsNode {
    role: String,
    label: Option<String>,
    value: Option<String>,
    bounds: SnapRect,
    flags: SnapSemanticsFlags,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct SnapSemanticsFlags {
    focused: bool,
    captured: bool,
    disabled: bool,
    selected: bool,
    expanded: bool,
    checked: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    semantics: Vec<SnapSemanticsNode>,
    scene_ops: Vec<SnapSceneOp>,
}

fn round3(v: f32) -> f32 {
    (v * 1000.0).round() / 1000.0
}

fn snap_point(p: Point) -> SnapPoint {
    SnapPoint {
        x: round3(p.x.0),
        y: round3(p.y.0),
    }
}

fn snap_size(s: CoreSize) -> SnapSize {
    SnapSize {
        w: round3(s.width.0),
        h: round3(s.height.0),
    }
}

fn snap_rect(r: Rect) -> SnapRect {
    SnapRect {
        origin: snap_point(r.origin),
        size: snap_size(r.size),
    }
}

fn snap_edges(e: Edges) -> SnapEdges {
    SnapEdges {
        top: round3(e.top.0),
        right: round3(e.right.0),
        bottom: round3(e.bottom.0),
        left: round3(e.left.0),
    }
}

fn snap_corners(c: Corners) -> SnapCorners {
    SnapCorners {
        top_left: round3(c.top_left.0),
        top_right: round3(c.top_right.0),
        bottom_right: round3(c.bottom_right.0),
        bottom_left: round3(c.bottom_left.0),
    }
}

fn snap_color(c: fret_core::Color) -> SnapColor {
    SnapColor {
        r: round3(c.r),
        g: round3(c.g),
        b: round3(c.b),
        a: round3(c.a),
    }
}

fn snap_transform(t: Transform2D) -> [f32; 6] {
    [
        round3(t.a),
        round3(t.b),
        round3(t.c),
        round3(t.d),
        round3(t.tx),
        round3(t.ty),
    ]
}

fn snap_uv(uv: UvRect) -> [f32; 4] {
    [round3(uv.u0), round3(uv.v0), round3(uv.u1), round3(uv.v1)]
}

fn snap_scene_op(op: SceneOp) -> SnapSceneOp {
    match op {
        SceneOp::PushTransform { transform } => SnapSceneOp::PushTransform {
            transform: snap_transform(transform),
        },
        SceneOp::PopTransform => SnapSceneOp::PopTransform,
        SceneOp::PushOpacity { opacity } => SnapSceneOp::PushOpacity {
            opacity: round3(opacity),
        },
        SceneOp::PopOpacity => SnapSceneOp::PopOpacity,
        SceneOp::PushLayer { layer } => SnapSceneOp::PushLayer { layer },
        SceneOp::PopLayer => SnapSceneOp::PopLayer,
        SceneOp::PushClipRect { rect } => SnapSceneOp::PushClipRect {
            rect: snap_rect(rect),
        },
        SceneOp::PushClipRRect { rect, corner_radii } => SnapSceneOp::PushClipRRect {
            rect: snap_rect(rect),
            corner_radii: snap_corners(corner_radii),
        },
        SceneOp::PopClip => SnapSceneOp::PopClip,
        SceneOp::Quad {
            rect,
            background,
            border,
            border_color,
            corner_radii,
            ..
        } => SnapSceneOp::Quad {
            rect: snap_rect(rect),
            background: snap_color(background),
            border: snap_edges(border),
            border_color: snap_color(border_color),
            corner_radii: snap_corners(corner_radii),
        },
        SceneOp::Image { rect, opacity, .. } => SnapSceneOp::Image {
            rect: snap_rect(rect),
            opacity: round3(opacity),
        },
        SceneOp::ImageRegion {
            rect, uv, opacity, ..
        } => SnapSceneOp::ImageRegion {
            rect: snap_rect(rect),
            uv: snap_uv(uv),
            opacity: round3(opacity),
        },
        SceneOp::MaskImage {
            rect,
            uv,
            color,
            opacity,
            ..
        } => SnapSceneOp::MaskImage {
            rect: snap_rect(rect),
            uv: snap_uv(uv),
            color: snap_color(color),
            opacity: round3(opacity),
        },
        SceneOp::SvgMaskIcon {
            rect,
            fit,
            color,
            opacity,
            ..
        } => SnapSceneOp::SvgMaskIcon {
            rect: snap_rect(rect),
            fit: format!("{fit:?}"),
            color: snap_color(color),
            opacity: round3(opacity),
        },
        SceneOp::SvgImage {
            rect, fit, opacity, ..
        } => SnapSceneOp::SvgImage {
            rect: snap_rect(rect),
            fit: format!("{fit:?}"),
            opacity: round3(opacity),
        },
        SceneOp::Text { origin, color, .. } => SnapSceneOp::Text {
            origin: snap_point(origin),
            color: snap_color(color),
        },
        SceneOp::Path { origin, color, .. } => SnapSceneOp::Path {
            origin: snap_point(origin),
            color: snap_color(color),
        },
        SceneOp::ViewportSurface { rect, opacity, .. } => SnapSceneOp::ViewportSurface {
            rect: snap_rect(rect),
            opacity: round3(opacity),
        },
        SceneOp::PushEffect { .. } | SceneOp::PopEffect => {
            unreachable!("effect ops are not expected in fret-ui-shadcn snapshots")
        }
    }
}

fn snapshot_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{name}.json"))
}

fn load_snapshot(path: &Path) -> Snapshot {
    let bytes = std::fs::read(path).expect("read snapshot json");
    serde_json::from_slice(&bytes).expect("parse snapshot json")
}

fn write_snapshot(path: &Path, snapshot: &Snapshot) {
    let bytes = serde_json::to_vec_pretty(snapshot).expect("serialize snapshot json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create snapshots dir");
    }
    std::fs::write(path, bytes).expect("write snapshot json");
}

fn assert_snapshot(name: &str, snapshot: Snapshot) {
    let path = snapshot_path(name);
    let update = std::env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1");

    if update || !path.exists() {
        if !path.exists() && !update {
            panic!(
                "missing snapshot: {}\nre-run with UPDATE_SNAPSHOTS=1 to generate it",
                path.display()
            );
        }
        write_snapshot(&path, &snapshot);
    }

    let expected = load_snapshot(&path);
    let expected_json = serde_json::to_string_pretty(&expected).expect("serialize expected");
    let actual_json = serde_json::to_string_pretty(&snapshot).expect("serialize actual");
    assert_eq!(
        expected_json,
        actual_json,
        "snapshot mismatch: {}",
        path.display()
    );
}

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

fn snapshot_for_root<I, F>(name: &str, bounds: Rect, build: F)
where
    F: FnOnce(&mut fret_ui::ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
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
        bounds,
        name,
        build,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let semantics = ui.semantics_snapshot_arc().expect("semantics snapshot");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut semantics_nodes: Vec<SnapSemanticsNode> = semantics
        .nodes
        .iter()
        .map(|n| SnapSemanticsNode {
            role: format!("{:?}", n.role),
            label: n.label.clone(),
            value: n.value.clone(),
            bounds: snap_rect(n.bounds),
            flags: SnapSemanticsFlags {
                focused: n.flags.focused,
                captured: n.flags.captured,
                disabled: n.flags.disabled,
                selected: n.flags.selected,
                expanded: n.flags.expanded,
                checked: n.flags.checked,
            },
        })
        .collect();

    semantics_nodes.sort_by(|a, b| {
        (
            a.role.as_str(),
            a.label.as_deref().unwrap_or(""),
            a.bounds.origin.x.to_bits(),
            a.bounds.origin.y.to_bits(),
            a.bounds.size.w.to_bits(),
            a.bounds.size.h.to_bits(),
        )
            .cmp(&(
                b.role.as_str(),
                b.label.as_deref().unwrap_or(""),
                b.bounds.origin.x.to_bits(),
                b.bounds.origin.y.to_bits(),
                b.bounds.size.w.to_bits(),
                b.bounds.size.h.to_bits(),
            ))
    });

    let scene_ops: Vec<SnapSceneOp> = scene.ops().iter().copied().map(snap_scene_op).collect();

    assert_snapshot(
        name,
        Snapshot {
            version: 1,
            semantics: semantics_nodes,
            scene_ops,
        },
    );
}

#[test]
fn snapshot_button_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(180.0)),
    );
    snapshot_for_root("button_default", bounds, |cx| {
        vec![fret_ui_shadcn::Button::new("Hello").into_element(cx)]
    });
}

#[test]
fn snapshot_tabs_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(420.0)),
    );
    snapshot_for_root("tabs_default", bounds, |cx| {
        let items = vec![
            fret_ui_shadcn::TabsItem::new("alpha", "Alpha", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("beta", "Beta", vec![cx.text("Panel")]),
            fret_ui_shadcn::TabsItem::new("gamma", "Gamma", vec![cx.text("Panel")]),
        ];
        vec![
            fret_ui_shadcn::Tabs::uncontrolled(Some("alpha"))
                .items(items)
                .into_element(cx),
        ]
    });
}
