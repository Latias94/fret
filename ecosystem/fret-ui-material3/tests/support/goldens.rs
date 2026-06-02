use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use fret_core::{AppWindowId, NodeId, Rect, Scene, UiServices};
use fret_ui::UiTree;
use serde::{Deserialize, Serialize};

use super::host::TestHost;
use super::interaction_harness::{QuadSig, SceneSig, scene_quad_signature, scene_signature};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Material3HeadlessGoldenV1 {
    pub(crate) signature: Vec<SceneOpSigV1>,
    pub(crate) quads: Vec<QuadV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Material3HeadlessSuiteV1 {
    pub(crate) cases: BTreeMap<String, Material3HeadlessGoldenV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SceneOpSigV1 {
    PushTransform,
    PopTransform,
    PushOpacity,
    PopOpacity,
    PushLayer,
    PopLayer,
    PushClipRect,
    PushClipRRect,
    PushClipPath,
    PopClip,
    PushMask,
    PopMask,
    PushEffect,
    PopEffect,
    PushBackdropSourceGroup,
    PopBackdropSourceGroup,
    PushCompositeGroup,
    PopCompositeGroup,
    StrokeRRect { order: u32 },
    ShadowRRect { order: u32 },
    Quad { order: u32 },
    VertexColorQuad { order: u32 },
    ImageQuad { order: u32 },
    VertexColorTriangle { order: u32 },
    ImageTriangle { order: u32 },
    Image { order: u32 },
    ImageRegion { order: u32 },
    MaskImage { order: u32 },
    SvgMaskIcon { order: u32 },
    SvgImage { order: u32 },
    Text { order: u32 },
    Path { order: u32 },
    ViewportSurface { order: u32 },
}

impl From<SceneSig> for SceneOpSigV1 {
    fn from(sig: SceneSig) -> Self {
        match sig {
            SceneSig::PushTransform => Self::PushTransform,
            SceneSig::PopTransform => Self::PopTransform,
            SceneSig::PushOpacity => Self::PushOpacity,
            SceneSig::PopOpacity => Self::PopOpacity,
            SceneSig::PushLayer => Self::PushLayer,
            SceneSig::PopLayer => Self::PopLayer,
            SceneSig::PushClipRect => Self::PushClipRect,
            SceneSig::PushClipRRect => Self::PushClipRRect,
            SceneSig::PushClipPath => Self::PushClipPath,
            SceneSig::PopClip => Self::PopClip,
            SceneSig::PushMask => Self::PushMask,
            SceneSig::PopMask => Self::PopMask,
            SceneSig::PushEffect => Self::PushEffect,
            SceneSig::PopEffect => Self::PopEffect,
            SceneSig::PushBackdropSourceGroup => Self::PushBackdropSourceGroup,
            SceneSig::PopBackdropSourceGroup => Self::PopBackdropSourceGroup,
            SceneSig::PushCompositeGroup => Self::PushCompositeGroup,
            SceneSig::PopCompositeGroup => Self::PopCompositeGroup,
            SceneSig::StrokeRRect(order) => Self::StrokeRRect { order: order.0 },
            SceneSig::ShadowRRect(order) => Self::ShadowRRect { order: order.0 },
            SceneSig::Quad(order) => Self::Quad { order: order.0 },
            SceneSig::VertexColorQuad(order) => Self::VertexColorQuad { order: order.0 },
            SceneSig::ImageQuad(order) => Self::ImageQuad { order: order.0 },
            SceneSig::VertexColorTriangle(order) => Self::VertexColorTriangle { order: order.0 },
            SceneSig::ImageTriangle(order) => Self::ImageTriangle { order: order.0 },
            SceneSig::Image(order) => Self::Image { order: order.0 },
            SceneSig::ImageRegion(order) => Self::ImageRegion { order: order.0 },
            SceneSig::MaskImage(order) => Self::MaskImage { order: order.0 },
            SceneSig::SvgMaskIcon(order) => Self::SvgMaskIcon { order: order.0 },
            SceneSig::SvgImage(order) => Self::SvgImage { order: order.0 },
            SceneSig::Text(order) => Self::Text { order: order.0 },
            SceneSig::Path(order) => Self::Path { order: order.0 },
            SceneSig::ViewportSurface(order) => Self::ViewportSurface { order: order.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct QuadV1 {
    order: u32,
    rect: [i32; 4],
    background: [i32; 4],
    border: [i32; 4],
    corner_radii: [i32; 4],
}

impl From<QuadSig> for QuadV1 {
    fn from(quad: QuadSig) -> Self {
        Self {
            order: quad.order.0,
            rect: [quad.rect.x, quad.rect.y, quad.rect.w, quad.rect.h],
            background: [
                quad.background.r,
                quad.background.g,
                quad.background.b,
                quad.background.a,
            ],
            border: [
                quad.border.top,
                quad.border.right,
                quad.border.bottom,
                quad.border.left,
            ],
            corner_radii: [
                quad.corner_radii.top_left,
                quad.corner_radii.top_right,
                quad.corner_radii.bottom_right,
                quad.corner_radii.bottom_left,
            ],
        }
    }
}

pub(crate) fn material3_scene_snapshot_v1(scene: &Scene) -> Material3HeadlessGoldenV1 {
    Material3HeadlessGoldenV1 {
        signature: scene_signature(scene)
            .into_iter()
            .map(SceneOpSigV1::from)
            .collect(),
        quads: scene_quad_signature(scene)
            .into_iter()
            .map(QuadV1::from)
            .collect(),
    }
}

pub(crate) fn settle_material3_scene_snapshot_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    bounds: Rect,
    scale_factor: f32,
    settle_from_frame: usize,
    total_frames: usize,
    stable_message: &str,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut settled: Option<Material3HeadlessGoldenV1> = None;
    for frame in 0..total_frames {
        app.advance_frame();
        let root = render(ui, app, services);
        ui.set_root(root);
        ui.layout_all(app, services, bounds, scale_factor);

        let mut scene = Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, scale_factor);

        if frame < settle_from_frame {
            continue;
        }

        let snapshot = material3_scene_snapshot_v1(&scene);
        if let Some(prev) = settled.as_ref() {
            assert_eq!(snapshot, *prev, "{stable_message}");
        } else {
            settled = Some(snapshot);
        }
    }

    settled.unwrap_or_else(|| panic!("expected a settled snapshot: {stable_message}"))
}

pub(crate) fn snapshot_material3_scene_at_frame_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    bounds: Rect,
    scale_factor: f32,
    snapshot_frame: usize,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut snapshot: Option<Material3HeadlessGoldenV1> = None;
    for _frame in 0..=snapshot_frame {
        app.advance_frame();
        let root = render(ui, app, services);
        ui.set_root(root);
        ui.layout_all(app, services, bounds, scale_factor);

        let mut scene = Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, scale_factor);
        snapshot = Some(material3_scene_snapshot_v1(&scene));
    }

    snapshot.unwrap_or_else(|| panic!("expected a snapshot at frame {snapshot_frame}"))
}

pub(crate) fn settle_material3_overlay_scene_snapshot_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    settle_from_frame: usize,
    total_frames: usize,
    stable_message: &str,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut settled: Option<Material3HeadlessGoldenV1> = None;
    for frame in 0..total_frames {
        let scene = run_overlay_frame_with_scene_scaled(
            ui,
            app,
            services,
            window,
            bounds,
            scale_factor,
            false,
            |ui, app, services| render(ui, app, services),
        );

        if frame < settle_from_frame {
            continue;
        }

        let snapshot = material3_scene_snapshot_v1(&scene);
        if let Some(prev) = settled.as_ref() {
            assert_eq!(snapshot, *prev, "{stable_message}");
        } else {
            settled = Some(snapshot);
        }
    }

    settled.unwrap_or_else(|| panic!("expected a settled snapshot: {stable_message}"))
}

pub(crate) fn write_or_assert_material3_suite_v1(name: &str, suite: &Material3HeadlessSuiteV1) {
    let path = material3_goldens_dir().join(format!("{name}.json"));

    if std::env::var_os("FRET_UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(material3_goldens_dir()).expect("create material3 goldens dir");
        let json = serde_json::to_string_pretty(suite).expect("serialize material3 suite golden");
        std::fs::write(&path, json).expect("write material3 suite golden");
        return;
    }

    let golden_json = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing material3 suite golden: {}\nerror: {err}\n\nTo (re)generate:\n  $env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment -- material3_headless\n",
            path.display()
        )
    });
    let golden: Material3HeadlessSuiteV1 =
        serde_json::from_str(&golden_json).expect("parse material3 suite golden");

    assert_eq!(
        *suite,
        golden,
        "material3 suite golden mismatch: {}\n\nTo update:\n  $env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment -- material3_headless",
        path.display()
    );
}

pub(crate) fn run_overlay_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

pub(crate) fn run_overlay_frame_scaled(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, scale_factor);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, scale_factor);
}

pub(crate) fn run_overlay_frame_with_scene_scaled(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Scene {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, scale_factor);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, scale_factor);
    scene
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn material3_goldens_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("material3-headless")
        .join("v1")
}
