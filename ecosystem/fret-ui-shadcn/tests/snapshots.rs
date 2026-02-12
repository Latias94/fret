use fret_app::App;
use fret_core::{
    AppWindowId, Color, Corners, Edges, Paint, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, Px, Rect, Scene, SceneOp, Size as CoreSize, SvgId, SvgService,
    TextBlobId, TextConstraints, TextMetrics, TextService, Transform2D, UvRect,
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
    PushMask {
        bounds: SnapRect,
    },
    PopMask,
    PushCompositeGroup,
    PopCompositeGroup,
    Quad {
        rect: SnapRect,
        background: SnapColor,
        border: SnapEdges,
        border_color: SnapColor,
        corner_radii: SnapCorners,
    },
    Image {
        rect: SnapRect,
        fit: String,
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

fn snap_paint(p: Paint) -> SnapColor {
    match p {
        Paint::Solid(c) => snap_color(c),
        Paint::LinearGradient(_) | Paint::RadialGradient(_) | Paint::Material { .. } => {
            snap_color(Color::TRANSPARENT)
        }
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
        SceneOp::PushMask { bounds, .. } => SnapSceneOp::PushMask {
            bounds: snap_rect(bounds),
        },
        SceneOp::PopMask => SnapSceneOp::PopMask,
        SceneOp::PushCompositeGroup { .. } => SnapSceneOp::PushCompositeGroup,
        SceneOp::PopCompositeGroup => SnapSceneOp::PopCompositeGroup,
        SceneOp::Quad {
            rect,
            background,
            border,
            border_paint,
            corner_radii,
            ..
        } => SnapSceneOp::Quad {
            rect: snap_rect(rect),
            background: snap_paint(background),
            border: snap_edges(border),
            border_color: snap_paint(border_paint),
            corner_radii: snap_corners(corner_radii),
        },
        SceneOp::Image {
            rect, fit, opacity, ..
        } => SnapSceneOp::Image {
            rect: snap_rect(rect),
            fit: format!("{fit:?}"),
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

#[test]
fn snapshot_extras_announcement_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_announcement_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Announcement::new([
                fret_ui_shadcn::extras::AnnouncementTag::new("New").into_element(cx),
                fret_ui_shadcn::extras::AnnouncementTitle::new([cx.text("Announcement")])
                    .into_element(cx),
            ])
            .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_banner_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(720.0), Px(180.0)),
    );
    snapshot_for_root("extras_banner_default", bounds, |cx| {
        let icon =
            fret_ui_kit::declarative::icon::icon(cx, fret_icons::IconId::new_static("lucide.info"));
        vec![
            fret_ui_shadcn::extras::Banner::new([
                fret_ui_shadcn::extras::BannerIcon::new(icon).into_element(cx),
                fret_ui_shadcn::extras::BannerTitle::new("A new version is available.")
                    .into_element(cx),
                fret_ui_shadcn::extras::BannerAction::new("Upgrade").into_element(cx),
                fret_ui_shadcn::extras::BannerClose::new().into_element(cx),
            ])
            .inset(true)
            .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_tags_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_tags_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Tags::new([
                "Alpha",
                "Beta",
                "Gamma",
                "A much longer tag label",
                "Zeta",
            ])
            .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_marquee_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_marquee_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_marquee_right_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_marquee_right_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
                .direction(fret_ui_shadcn::extras::MarqueeDirection::Right)
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_marquee_static_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_marquee_static_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
                .speed_px_per_frame(Px(0.0))
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_marquee_cycle_width_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_marquee_cycle_width_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Marquee::new(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
                .cycle_width_px(Px(240.0))
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_ticker_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_ticker_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Ticker::new("AAPL")
                .price("$199.18")
                .change("+1.01%")
                .change_kind(fret_ui_shadcn::extras::TickerChangeKind::Up)
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_relative_time_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(240.0)),
    );
    snapshot_for_root("extras_relative_time_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::RelativeTime::new([
                fret_ui_shadcn::extras::RelativeTimeZone::new(
                    "UTC",
                    "February 9, 2026",
                    "15:04:05",
                )
                .into_element(cx),
                fret_ui_shadcn::extras::RelativeTimeZone::new(
                    "PST",
                    "February 9, 2026",
                    "07:04:05",
                )
                .into_element(cx),
                fret_ui_shadcn::extras::RelativeTimeZone::new(
                    "CET",
                    "February 9, 2026",
                    "16:04:05",
                )
                .into_element(cx),
            ])
            .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_rating_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(180.0)),
    );
    snapshot_for_root("extras_rating_default", bounds, |cx| {
        vec![
            fret_ui_shadcn::extras::Rating::uncontrolled(3)
                .count(5)
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_avatar_stack_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_avatar_stack_default", bounds, |cx| {
        let a =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("A").into_element(cx)],
            );
        let b =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("B").into_element(cx)],
            );
        let c =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("C").into_element(cx)],
            );
        let d =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("D").into_element(cx)],
            );

        vec![
            fret_ui_shadcn::extras::AvatarStack::new([a, b, c, d])
                .size_px(Px(40.0))
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_avatar_stack_overflow_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(560.0), Px(180.0)),
    );
    snapshot_for_root("extras_avatar_stack_overflow_default", bounds, |cx| {
        let a =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("A").into_element(cx)],
            );
        let b =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("B").into_element(cx)],
            );
        let c =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("C").into_element(cx)],
            );
        let d =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("D").into_element(cx)],
            );
        let e =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("E").into_element(cx)],
            );
        let f =
            fret_ui_shadcn::Avatar::new(
                [fret_ui_shadcn::AvatarFallback::new("F").into_element(cx)],
            );

        vec![
            fret_ui_shadcn::extras::AvatarStack::new([a, b, c, d, e, f])
                .size_px(Px(40.0))
                .max_visible(4)
                .into_element(cx),
        ]
    });
}

#[test]
fn snapshot_extras_kanban_default() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(920.0), Px(420.0)),
    );
    snapshot_for_root("extras_kanban_default", bounds, |cx| {
        let columns = vec![
            fret_ui_shadcn::extras::KanbanColumn::new("backlog", "Backlog"),
            fret_ui_shadcn::extras::KanbanColumn::new("in_progress", "In Progress"),
            fret_ui_shadcn::extras::KanbanColumn::new("done", "Done"),
        ];

        let items = cx.app.models_mut().insert(vec![
            fret_ui_shadcn::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
            fret_ui_shadcn::extras::KanbanItem::new("card-2", "Port block", "backlog"),
            fret_ui_shadcn::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
            fret_ui_shadcn::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
            fret_ui_shadcn::extras::KanbanItem::new("card-5", "Ship", "done"),
        ]);

        vec![fret_ui_shadcn::extras::Kanban::new(columns, items).into_element(cx)]
    });
}

#[test]
fn snapshot_extras_kanban_custom_cards() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(920.0), Px(420.0)),
    );
    snapshot_for_root("extras_kanban_custom_cards", bounds, |cx| {
        let columns = vec![
            fret_ui_shadcn::extras::KanbanColumn::new("backlog", "Backlog"),
            fret_ui_shadcn::extras::KanbanColumn::new("in_progress", "In Progress"),
            fret_ui_shadcn::extras::KanbanColumn::new("done", "Done"),
        ];

        let items = cx.app.models_mut().insert(vec![
            fret_ui_shadcn::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
            fret_ui_shadcn::extras::KanbanItem::new("card-2", "Port block", "backlog"),
            fret_ui_shadcn::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
            fret_ui_shadcn::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
            fret_ui_shadcn::extras::KanbanItem::new("card-5", "Ship", "done"),
        ]);

        let board = fret_ui_shadcn::extras::Kanban::new(columns, items).into_element_with(
            cx,
            |cx, item, ctx| {
                let title = fret_ui_kit::ui::text(cx, item.name.clone())
                    .font_medium()
                    .w_full()
                    .min_w_0()
                    .truncate()
                    .into_element(cx);

                let badge = fret_ui_shadcn::Badge::new(item.column.clone())
                    .variant(fret_ui_shadcn::BadgeVariant::Secondary)
                    .into_element(cx);

                let mut children = vec![title, badge];
                if ctx.mode == fret_ui_shadcn::extras::KanbanCardMode::Overlay {
                    children.push(
                        fret_ui_kit::ui::text(cx, "overlay")
                            .nowrap()
                            .into_element(cx),
                    );
                }

                fret_ui_kit::declarative::stack::vstack(
                    cx,
                    fret_ui_kit::declarative::stack::VStackProps::default()
                        .gap(fret_ui_kit::Space::N1)
                        .layout(fret_ui_kit::LayoutRefinement::default().w_full()),
                    |_cx| children,
                )
            },
        );

        vec![board]
    });
}
