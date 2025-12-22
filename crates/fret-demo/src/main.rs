mod asset_drop;
mod asset_open;
mod command_palette;
mod demo_ui;
mod dnd_probe;
mod editor_shell;
mod elements_mvp2;
mod hierarchy;
mod ime_probe;
mod overlay_layouts;
mod project_panel;
mod scene_background;
mod scene_document;
mod text_probe_panel;
mod undo;
mod unsaved_changes;
mod viewport_asset_drop;
mod viewport_targets;
mod world;

use demo_ui::{
    DebugHudService, DebugInspectorOutline, DebugInspectorService, DebugInspectorSnapshot,
    DemoLayers, DemoUiConfig, DemoUiKind, build_demo_ui,
};
use editor_shell::{DemoSelection, HierarchyPanel, InspectorPanel};
use hierarchy::DemoHierarchy;
use project_panel::ProjectPanel;
use scene_background::{SceneBackgroundRenderer, SceneCameraParams};
use undo::{EditCommand, UndoStack};
use viewport_targets::{ViewportTarget, ViewportTargets};
use world::DemoWorld;

use asset_drop::{
    AssetDropDecision, AssetDropRegistry, AssetDropRule, AssetDropService, AssetDropTarget,
    CurrentSceneService,
};
use asset_open::{AssetOpenDecision, AssetOpenRegistry, AssetOpenRule};
use fret_app::{
    App, CommandId, CommandMeta, CommandScope, CreateWindowKind, CreateWindowRequest, Effect,
    Keymap, KeymapFileV1, KeymapService, Model, WhenExpr, WindowRequest,
    keymap::{BindingV1, KeySpecV1},
};
use fret_core::{
    Axis, Color, DockLayoutNodeV1, DockLayoutV1, DockNode, DockOp, DropZone, PanelKey,
    PlatformCapabilities, Rect, RenderTargetId, Scene, WindowAnchor,
};
use fret_editor::{
    InspectorEditKind, InspectorEditService, MarqueeSelectInteraction, PanOrbitInteraction,
    PanOrbitKind, ProjectSelectionService, ProjectService, PropertyEditKind, PropertyEditRequest,
    PropertyEditService, PropertyPath, PropertyValue, RotateGizmoInteraction,
    TranslateAxisConstraint, TranslateGizmoInteraction, ViewportInteraction, ViewportToolManager,
    ViewportToolMode, parse_value,
};
use fret_render::{Renderer, WgpuContext};
use fret_runner_winit_wgpu::{
    EngineFrameUpdate, RenderTargetUpdate, WindowCreateSpec, WinitDriver, WinitRunner,
    WinitRunnerConfig,
};
use fret_ui::Invalidation;
use fret_ui::dock::ViewportMarquee;
use fret_ui::{
    ContextMenuService, DockManager, DockPanel, DockPanelContentService, Theme, ThemeConfig,
    UiTree, ViewportPanel,
};
use scene_document::{SceneDocumentService, SceneSnapshot};
use std::{
    collections::HashMap,
    collections::VecDeque,
    ffi::OsString,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
use text_probe_panel::{TextProbePanel, TextProbeService};
use unsaved_changes::{UnsavedChangesService, UnsavedContinuation};
use winit::event_loop::EventLoop;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub(crate) struct DemoPlayState {
    pub playing: bool,
}

const TEXT_PROBE_DEFAULT: &str = r#"Multiline text probe (MVP11 validation)

Try:
- Click to place the caret (including near line ends).
- Drag to select across wrapped lines.
- Arrow Up/Down should preserve x as much as possible.
- Scroll while typing; IME candidate window should follow the caret.

Hard line breaks:
line 1
line 2
line 3

Long paragraph to test wrapping and hit testing:
The quick brown fox jumps over the lazy dog. 1234567890.
Symbols: []{}() <> /\\ | _-+=* &%$#@! ?
Unicode: 你好，世界。日本語。한글. 😀✨
"#;

fn project_renamed_candidate(file_name: &str, attempt: u32) -> String {
    if attempt == 0 {
        format!("{file_name}_renamed")
    } else {
        format!("{file_name}_renamed_{attempt}")
    }
}

fn next_project_rename_name(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    let stem = path.file_stem()?.to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy());

    let mut attempt: u32 = 0;
    loop {
        let candidate = if ext.is_some() {
            let base = project_renamed_candidate(stem.as_ref(), attempt);
            format!("{base}.{}", ext.as_ref().unwrap().as_ref())
        } else {
            project_renamed_candidate(file_name.as_ref(), attempt)
        };
        let parent = path.parent()?;
        if !parent.join(&candidate).exists() {
            return Some(candidate);
        }
        attempt = attempt.saturating_add(1);
        if attempt > 1000 {
            return None;
        }
    }
}

fn meta_path_for_asset(path: &Path) -> PathBuf {
    let mut os: OsString = path.as_os_str().to_os_string();
    os.push(".meta");
    PathBuf::from(os)
}

fn unique_scene_path(scenes_dir: &Path, base_stem: &str) -> PathBuf {
    let base_stem = base_stem.trim();
    let base_stem = if base_stem.is_empty() {
        "Scene"
    } else {
        base_stem
    };

    for i in 0..=1000u32 {
        let name = if i == 0 {
            format!("{base_stem}.scene")
        } else {
            format!("{base_stem} {i}.scene")
        };
        let candidate = scenes_dir.join(name);
        if !candidate.exists() && !meta_path_for_asset(&candidate).exists() {
            return candidate;
        }
    }

    let stamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    scenes_dir.join(format!("Scene-{stamp}.scene"))
}

struct DemoWindowState {
    ui: UiTree,
    layers: DemoLayers,
    palette_previous_focus: Option<fret_core::NodeId>,
    context_menu_previous_focus: Option<fret_core::NodeId>,
    inspector_edit_previous_focus: Option<fret_core::NodeId>,
    unsaved_dialog_previous_focus: Option<fret_core::NodeId>,
    inspector_edit_buffer: Model<String>,
    last_cursor_pos: Option<fret_core::Point>,
}

#[derive(Default)]
struct DemoDriver {
    main_window: Option<fret_core::AppWindowId>,
    viewport_targets: Option<ViewportTargets>,
    background: Option<SceneBackgroundRenderer>,
    logical_windows: HashMap<fret_core::AppWindowId, String>,
    window_placements: HashMap<fret_core::AppWindowId, fret_core::DockWindowPlacementV1>,
    next_floating_index: u32,
    loaded_layout: Option<DockLayoutV1>,
    dock_persist_timer: Option<fret_core::TimerToken>,
    dock_persist_pending: bool,
    selection: Option<Model<DemoSelection>>,
    hierarchy: Option<Model<DemoHierarchy>>,
    world: Option<Model<DemoWorld>>,
    undo: Option<Model<UndoStack>>,
    viewport_tools: Option<Model<ViewportToolManager>>,
    viewport_cameras: HashMap<PanelKey, DemoViewportCamera>,
    camera_persist_timer: Option<fret_core::TimerToken>,
    camera_persist_pending: bool,
    theme_reload_timer: Option<fret_core::TimerToken>,
    theme_file_modified: Option<SystemTime>,
    theme_file_size: Option<u64>,
    theme_file_hash: Option<u64>,
    play_mode: bool,
    play_started_at: Option<Instant>,
    pending_window_ui_kinds: VecDeque<DemoUiKind>,
    asset_drop_registry: AssetDropRegistry,
    last_scene_chrome_rev: Option<(u64, u64)>,
    asset_open_registry: AssetOpenRegistry,
    active_property_edits: HashMap<fret_core::AppWindowId, ActivePropertyEdit>,
}

#[derive(Debug, Clone)]
struct ActivePropertyEdit {
    targets: Vec<u64>,
    path: PropertyPath,
    before: Vec<Option<PropertyValue>>,
    was_dirty: bool,
}

fn load_theme(app: &mut App) {
    let candidates = [
        "./.fret/theme.json",
        "./themes/fret-default-dark.json",
        "./themes/godot-default-dark.json",
        "./themes/hardhacker-dark.json",
    ];
    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        match ThemeConfig::from_slice(&bytes) {
            Ok(cfg) => {
                Theme::global_mut(app).apply_config(&cfg);
                tracing::info!(theme = %cfg.name, path = %path, "loaded theme");
                return;
            }
            Err(err) => {
                tracing::error!(error = %err, path = %path, "failed to parse theme file");
            }
        }
    }
}

fn theme_override_path() -> &'static str {
    "./.fret/theme.json"
}

fn theme_builtin_path(id: &str) -> Option<&'static str> {
    match id {
        "fret_default_dark" => Some("./themes/fret-default-dark.json"),
        "hardhacker_dark" => Some("./themes/hardhacker-dark.json"),
        "godot_default_dark" => Some("./themes/godot-default-dark.json"),
        _ => None,
    }
}

#[derive(Debug)]
enum ThemeApplyOutcome {
    Applied,
    NoChange,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct DemoViewportCamera {
    center: [f32; 2],
    zoom: f32,
    rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoViewportRole {
    Scene,
    Game,
}

fn demo_viewport_role(panel: &PanelKey) -> DemoViewportRole {
    match panel.kind.0.as_str() {
        "core.game" => DemoViewportRole::Game,
        _ => DemoViewportRole::Scene,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewportCamerasFileV1 {
    version: u32,
    cameras: Vec<ViewportCameraEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewportCameraEntryV1 {
    panel: PanelKey,
    camera: DemoViewportCamera,
}

impl Default for DemoViewportCamera {
    fn default() -> Self {
        Self {
            center: [5.0, 5.0],
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

impl DemoViewportCamera {
    const WORLD_SPAN: f32 = 10.0;
    const MIN_ZOOM: f32 = 0.1;
    const MAX_ZOOM: f32 = 32.0;

    fn rotate(v: [f32; 2], angle: f32) -> [f32; 2] {
        let (s, c) = angle.sin_cos();
        [v[0] * c - v[1] * s, v[0] * s + v[1] * c]
    }

    fn world_to_uv(&self, pos: [f32; 3]) -> (f32, f32) {
        let dx = pos[0] - self.center[0];
        let dy = pos[1] - self.center[1];
        let r = Self::rotate([dx, dy], -self.rotation);
        let view = [r[0] * self.zoom, r[1] * self.zoom];
        let u = 0.5 + view[0] / Self::WORLD_SPAN;
        let v = 0.5 - view[1] / Self::WORLD_SPAN;
        (u, v)
    }

    fn uv_to_world_xy(&self, uv: (f32, f32)) -> [f32; 2] {
        let view_x = (uv.0 - 0.5) * Self::WORLD_SPAN;
        let view_y = (0.5 - uv.1) * Self::WORLD_SPAN;
        let r = [view_x / self.zoom, view_y / self.zoom];
        let d = Self::rotate(r, self.rotation);
        [self.center[0] + d[0], self.center[1] + d[1]]
    }

    fn pan_by_uv_delta(&mut self, du: f32, dv: f32) {
        let view = [du * Self::WORLD_SPAN, -dv * Self::WORLD_SPAN];
        let r = [view[0] / self.zoom, view[1] / self.zoom];
        let d = Self::rotate(r, self.rotation);
        self.center[0] -= d[0];
        self.center[1] -= d[1];
    }

    fn orbit_by_uv_delta(&mut self, du: f32) {
        let radians_per_u = std::f32::consts::PI * 1.25;
        self.rotation = (self.rotation + du * radians_per_u) % (std::f32::consts::TAU);
    }

    fn zoom_at_uv(&mut self, uv: (f32, f32), wheel_y: f32) {
        let before = self.uv_to_world_xy(uv);
        let zoom_mul = (wheel_y * 0.002).exp();
        self.zoom = (self.zoom * zoom_mul).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let after = self.uv_to_world_xy(uv);
        self.center[0] += before[0] - after[0];
        self.center[1] += before[1] - after[1];
    }
}

impl DemoDriver {
    fn layout_path() -> &'static Path {
        Path::new("./.fret/layout.json")
    }

    fn layout_presets_dir() -> &'static Path {
        Path::new("./.fret/layout-presets")
    }

    fn last_layout_preset_path() -> PathBuf {
        Self::layout_presets_dir().join("last.json")
    }

    fn keymap_path() -> &'static Path {
        Path::new("./.fret/keymap.json")
    }

    fn viewport_cameras_path() -> &'static Path {
        Path::new("./.fret/viewport_cameras.json")
    }

    fn viewport_camera(&self, panel: &PanelKey) -> DemoViewportCamera {
        self.viewport_cameras
            .get(panel)
            .copied()
            .unwrap_or_default()
    }

    fn viewport_camera_mut(&mut self, panel: PanelKey) -> &mut DemoViewportCamera {
        self.viewport_cameras
            .entry(panel)
            .or_insert_with(DemoViewportCamera::default)
    }

    fn install_asset_drop_rules(&mut self) {
        self.asset_drop_registry = AssetDropRegistry::default();

        self.asset_drop_registry.add_rule(AssetDropRule {
            matches: Box::new(|cx| {
                cx.kind == Some(fret_editor::ProjectEntryKind::File)
                    && matches!(cx.target, AssetDropTarget::Hierarchy { .. })
            }),
            apply: Box::new(|cx| {
                let AssetDropTarget::Hierarchy { parent } = &cx.target else {
                    return AssetDropDecision::Ignored;
                };
                let name = cx
                    .path
                    .as_deref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("Asset {}", cx.guid.0));

                let Some(new_id) = cx.driver.spawn_asset_entity(cx.app, name, *parent, None) else {
                    return AssetDropDecision::Ignored;
                };
                cx.driver.select_entity(cx.app, new_id);
                AssetDropDecision::Handled
            }),
        });

        self.asset_drop_registry.add_rule(AssetDropRule {
            matches: Box::new(|cx| {
                cx.kind == Some(fret_editor::ProjectEntryKind::File)
                    && matches!(cx.target, AssetDropTarget::SceneViewport { .. })
            }),
            apply: Box::new(|cx| {
                let AssetDropTarget::SceneViewport { panel, uv, .. } = &cx.target else {
                    return AssetDropDecision::Ignored;
                };
                if demo_viewport_role(panel) != DemoViewportRole::Scene {
                    return AssetDropDecision::Ignored;
                }

                let name = cx
                    .path
                    .as_deref()
                    .and_then(|p| p.file_stem())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("Asset {}", cx.guid.0));

                let cam = cx.driver.viewport_camera(panel);
                let [x, y] = cam.uv_to_world_xy(*uv);
                let Some(new_id) = cx
                    .driver
                    .spawn_asset_entity(cx.app, name, None, Some([x, y]))
                else {
                    return AssetDropDecision::Ignored;
                };
                cx.driver.select_entity(cx.app, new_id);
                AssetDropDecision::Handled
            }),
        });

        self.asset_drop_registry.add_rule(AssetDropRule {
            matches: Box::new(|cx| {
                cx.kind == Some(fret_editor::ProjectEntryKind::File)
                    && cx.extension == Some("scene")
                    && matches!(cx.target, AssetDropTarget::SceneViewport { .. })
            }),
            apply: Box::new(|cx| {
                let AssetDropTarget::SceneViewport { panel, .. } = &cx.target else {
                    return AssetDropDecision::Ignored;
                };
                if demo_viewport_role(panel) != DemoViewportRole::Scene {
                    return AssetDropDecision::Ignored;
                }

                if cx
                    .driver
                    .open_scene_by_guid_or_prompt(cx.app, cx.window, cx.guid)
                {
                    AssetDropDecision::Handled
                } else {
                    AssetDropDecision::Ignored
                }
            }),
        });
    }

    fn install_asset_open_rules(&mut self) {
        self.asset_open_registry = AssetOpenRegistry::default();

        self.asset_open_registry.add_rule(AssetOpenRule {
            matches: Box::new(|cx| {
                cx.kind == Some(fret_editor::ProjectEntryKind::File)
                    && cx
                        .extension
                        .is_some_and(|e| matches!(e, "txt" | "md" | "json" | "wgsl"))
            }),
            apply: Box::new(|cx| {
                let Some(path) = cx.path.as_deref() else {
                    return AssetOpenDecision::Ignored;
                };
                let bytes = std::fs::read(path).unwrap_or_default();
                let text = String::from_utf8_lossy(&bytes).to_string();
                let title = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Text".to_string());

                cx.app
                    .with_global_mut(TextProbeService::default, |s, _app| {
                        s.set(Some(title.clone()), text);
                    });

                let key = PanelKey::new("core.text_probe");
                if let Some(dock) = cx.app.global_mut::<DockManager>() {
                    if let Some(p) = dock.panels.get_mut(&key) {
                        p.title = format!("Text Probe — {title}");
                    }
                }

                let preferred = [Some(cx.window), cx.driver.main_window]
                    .into_iter()
                    .flatten();
                DockManager::request_activate_panel(cx.app, cx.window, preferred, key.clone());
                AssetOpenDecision::Handled
            }),
        });

        self.asset_open_registry.add_rule(AssetOpenRule {
            matches: Box::new(|cx| {
                cx.kind == Some(fret_editor::ProjectEntryKind::File)
                    && cx.extension == Some("scene")
            }),
            apply: Box::new(|cx| {
                let ok = cx
                    .driver
                    .open_scene_by_guid_or_prompt(cx.app, cx.window, cx.guid);
                if !ok {
                    return AssetOpenDecision::Ignored;
                }

                let key = PanelKey::new("core.scene");
                let preferred = [Some(cx.window), cx.driver.main_window]
                    .into_iter()
                    .flatten();
                DockManager::request_activate_panel(cx.app, cx.window, preferred, key.clone());

                AssetOpenDecision::Handled
            }),
        });
    }

    fn open_asset_by_guid(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        guid: fret_editor::AssetGuid,
    ) -> bool {
        let mut reg = std::mem::take(&mut self.asset_open_registry);
        let decision = reg.handle(self, app, window, guid);
        self.asset_open_registry = reg;
        decision == AssetOpenDecision::Handled
    }

    fn mark_scene_dirty(&mut self, app: &mut App) {
        let has_scene = app
            .global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .is_some();
        if !has_scene {
            return;
        }
        app.with_global_mut(SceneDocumentService::default, |s, _app| {
            s.set_dirty(true);
        });
    }

    fn is_scene_dirty(app: &App) -> bool {
        app.global::<SceneDocumentService>()
            .is_some_and(|s| s.dirty())
    }

    fn has_current_scene(app: &App) -> bool {
        app.global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .is_some()
    }

    fn prompt_unsaved(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        action: UnsavedContinuation,
    ) {
        app.with_global_mut(UnsavedChangesService::default, |s, _app| {
            s.set_pending(window, action);
        });
        app.push_effect(Effect::Command {
            window: Some(window),
            command: CommandId::from("unsaved_dialog.open"),
        });
    }

    fn open_scene_by_guid_or_prompt(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        guid: fret_editor::AssetGuid,
    ) -> bool {
        if Self::is_scene_dirty(app) && Self::has_current_scene(app) {
            self.prompt_unsaved(app, window, UnsavedContinuation::OpenScene { guid });
            return true;
        }
        self.open_scene_by_guid(app, guid)
    }

    fn new_scene_or_prompt(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        if Self::is_scene_dirty(app) && Self::has_current_scene(app) {
            self.prompt_unsaved(app, window, UnsavedContinuation::NewScene);
            return true;
        }
        self.create_and_open_new_scene(app)
    }

    fn create_and_open_new_scene(&mut self, app: &mut App) -> bool {
        let Some(guid) = self.create_new_scene_asset(app) else {
            return false;
        };
        self.open_scene_by_guid(app, guid)
    }

    fn create_new_scene_asset(&mut self, app: &mut App) -> Option<fret_editor::AssetGuid> {
        let Some(project) = app.global_mut::<ProjectService>() else {
            return None;
        };

        let scenes_dir = project.assets_root().join("Scenes");
        if let Err(err) = std::fs::create_dir_all(&scenes_dir) {
            tracing::error!(
                error = %err,
                path = %scenes_dir.to_string_lossy(),
                "failed to create Scenes directory"
            );
            return None;
        }

        let path = unique_scene_path(&scenes_dir, "New Scene");
        let scene_title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Scene".to_string());

        let mut entity = DemoWorld::default().entity_snapshot(1);
        entity.name = scene_title;

        let file = scene_document::DemoSceneFileV1 {
            version: 1,
            roots: vec![scene_document::DemoSceneNodeV1 {
                id: 1,
                entity,
                children: Vec::new(),
            }],
        };
        let bytes = scene_document::write_scene_pretty(&file);
        if let Err(err) = std::fs::write(&path, bytes) {
            tracing::error!(
                error = %err,
                path = %path.to_string_lossy(),
                "failed to write new scene file"
            );
            return None;
        }

        if let Err(err) = project.rescan() {
            tracing::error!(error = %err, "failed to rescan project after creating new scene");
            return None;
        }

        project.guid_for_path(&path)
    }

    fn handle_window_close_requested(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut DemoWindowState,
    ) {
        let dirty = Self::is_scene_dirty(app) && Self::has_current_scene(app);
        let key_scene = PanelKey::new("core.scene");
        let contains_scene_panel = app.global::<DockManager>().is_some_and(|dock| {
            dock.graph
                .collect_panels_in_window(window)
                .iter()
                .any(|p| p == &key_scene)
        });
        let is_main = Some(window) == self.main_window;

        if dirty && (is_main || contains_scene_panel) {
            self.prompt_unsaved(app, window, UnsavedContinuation::CloseWindow { window });
            if !state.ui.is_layer_visible(state.layers.modal) {
                state.unsaved_dialog_previous_focus = state.ui.focus();
                state.ui.set_layer_visible(state.layers.modal, true);
                state.ui.set_focus(None);
            }
            app.request_redraw(window);
            return;
        }

        app.push_effect(Effect::Window(WindowRequest::Close(window)));
    }

    fn sync_scene_chrome(&mut self, app: &mut App) {
        let scene_rev = app
            .global::<CurrentSceneService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let doc_rev = app
            .global::<SceneDocumentService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let next = (scene_rev, doc_rev);
        if self.last_scene_chrome_rev == Some(next) {
            return;
        }
        self.last_scene_chrome_rev = Some(next);

        let guid = app.global::<CurrentSceneService>().and_then(|s| s.guid());
        let dirty = app
            .global::<SceneDocumentService>()
            .is_some_and(|s| s.dirty());

        let title = guid
            .and_then(|guid| {
                let project = app.global::<ProjectService>()?;
                let id = project.id_for_guid(guid)?;
                let path = project.path_for_id(id)?;
                let stem = path.file_stem()?.to_string_lossy().to_string();
                Some(stem)
            })
            .unwrap_or_else(|| "Scene".to_string());
        let suffix = if dirty { " *" } else { "" };

        let key_scene = PanelKey::new("core.scene");
        if let Some(dock) = app.global_mut::<DockManager>() {
            if let Some(p) = dock.panels.get_mut(&key_scene) {
                p.title = format!("Scene — {title}{suffix}");
            }
        }
    }

    fn open_scene_by_guid(&mut self, app: &mut App, guid: fret_editor::AssetGuid) -> bool {
        let Some(project) = app.global::<ProjectService>() else {
            return false;
        };
        let Some(id) = project.id_for_guid(guid) else {
            return false;
        };
        if project.kind_for_id(id) != Some(fret_editor::ProjectEntryKind::File) {
            return false;
        }
        let Some(path) = project.path_for_id(id).map(|p| p.to_path_buf()) else {
            return false;
        };
        if path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            != "scene"
        {
            return false;
        }
        let fallback_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Scene".to_string());

        let bytes = std::fs::read(&path).unwrap_or_default();
        let file = scene_document::parse_scene_bytes(&bytes).unwrap_or_else(|| {
            let mut entity = DemoWorld::default().entity_snapshot(1);
            entity.name = fallback_name;
            scene_document::DemoSceneFileV1 {
                version: 1,
                roots: vec![scene_document::DemoSceneNodeV1 {
                    id: 1,
                    entity,
                    children: Vec::new(),
                }],
            }
        });
        let snapshot = file.to_snapshot();
        self.apply_scene_snapshot(app, guid, snapshot);
        true
    }

    fn capture_scene_snapshot(&self, app: &App) -> Option<SceneSnapshot> {
        let hierarchy = self.hierarchy.and_then(|m| m.get(app)).cloned()?;
        let world = self.world.and_then(|m| m.get(app)).cloned()?;
        Some(SceneSnapshot { hierarchy, world })
    }

    fn save_current_scene(&mut self, app: &mut App) -> bool {
        let guid = app.global::<CurrentSceneService>().and_then(|s| s.guid());
        let Some(guid) = guid else {
            return false;
        };
        let Some(project) = app.global::<ProjectService>() else {
            return false;
        };
        let Some(id) = project.id_for_guid(guid) else {
            return false;
        };
        let Some(path) = project.path_for_id(id).map(|p| p.to_path_buf()) else {
            return false;
        };
        if path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            != "scene"
        {
            return false;
        }

        let Some(snapshot) = self.capture_scene_snapshot(app) else {
            return false;
        };
        let file = scene_document::DemoSceneFileV1::from_snapshot(&snapshot);
        let bytes = scene_document::write_scene_pretty(&file);
        if let Err(err) = std::fs::write(&path, bytes) {
            tracing::error!(error = %err, path = %path.to_string_lossy(), "failed to save scene");
            return false;
        }

        app.with_global_mut(SceneDocumentService::default, |s, _app| {
            s.set_dirty(false);
        });
        true
    }

    fn save_current_scene_as(&mut self, app: &mut App) -> bool {
        let Some(snapshot) = self.capture_scene_snapshot(app) else {
            return false;
        };

        let base_stem = app
            .global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .and_then(|guid| {
                let project = app.global::<ProjectService>()?;
                let id = project.id_for_guid(guid)?;
                let path = project.path_for_id(id)?;
                path.file_stem().map(|s| s.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "Scene".to_string());
        let base_stem = format!("{base_stem} Copy");

        let Some(project) = app.global_mut::<ProjectService>() else {
            return false;
        };
        let scenes_dir = project.assets_root().join("Scenes");
        if let Err(err) = std::fs::create_dir_all(&scenes_dir) {
            tracing::error!(
                error = %err,
                path = %scenes_dir.to_string_lossy(),
                "failed to create Scenes directory"
            );
            return false;
        }

        let path = unique_scene_path(&scenes_dir, &base_stem);
        let file = scene_document::DemoSceneFileV1::from_snapshot(&snapshot);
        let bytes = scene_document::write_scene_pretty(&file);
        if let Err(err) = std::fs::write(&path, bytes) {
            tracing::error!(
                error = %err,
                path = %path.to_string_lossy(),
                "failed to write scene file (save as)"
            );
            return false;
        }

        if let Err(err) = project.rescan() {
            tracing::error!(error = %err, "failed to rescan project after save as");
            return false;
        }

        let Some(guid) = project.guid_for_path(&path) else {
            tracing::error!(path = %path.to_string_lossy(), "save as produced file without guid");
            return false;
        };

        app.with_global_mut(CurrentSceneService::default, |s, _app| {
            s.set_guid(Some(guid));
        });
        app.with_global_mut(SceneDocumentService::default, |s, _app| {
            s.set_dirty(false);
        });
        app.with_global_mut(ProjectSelectionService::default, |s, _app| {
            s.set_selected_guid(Some(guid));
        });

        true
    }

    fn apply_scene_snapshot(
        &mut self,
        app: &mut App,
        guid: fret_editor::AssetGuid,
        snapshot: SceneSnapshot,
    ) {
        if let Some(hierarchy) = self.hierarchy {
            let _ = hierarchy.update(app, |h, _cx| {
                h.roots = snapshot.hierarchy.roots;
            });
        }
        if let Some(world) = self.world {
            let _ = world.update(app, |w, _cx| {
                *w = snapshot.world;
            });
        }
        if let Some(selection) = self.selection {
            let _ = selection.update(app, |s, _cx| {
                s.lead_entity = None;
                s.selected_entities.clear();
            });
        }
        if let Some(undo) = self.undo {
            let _ = undo.update(app, |u, _cx| {
                *u = UndoStack::default();
            });
        }

        app.with_global_mut(CurrentSceneService::default, |s, _app| {
            s.set_guid(Some(guid));
        });
        app.with_global_mut(SceneDocumentService::default, |s, _app| {
            s.set_dirty(false);
        });
        app.with_global_mut(ProjectSelectionService::default, |s, _app| {
            s.set_selected_guid(Some(guid));
        });

        for &w in self.logical_windows.keys() {
            app.request_redraw(w);
        }
    }

    fn spawn_asset_entity(
        &mut self,
        app: &mut App,
        name: String,
        parent: Option<u64>,
        position_xy: Option<[f32; 2]>,
    ) -> Option<u64> {
        let hierarchy = self.hierarchy?;
        let world = self.world?;

        let new_id = hierarchy
            .update(app, |h, _cx| h.create_entity(parent, name.clone()))
            .ok()?;

        let _ = world.update(app, |w, _cx| {
            let e = w.entity_mut(new_id);
            e.name = name;
            if let Some([x, y]) = position_xy {
                e.transform.position = [x, y, 0.0];
            }
        });

        self.mark_scene_dirty(app);
        Some(new_id)
    }

    fn select_entity(&mut self, app: &mut App, id: u64) {
        let Some(selection) = self.selection else {
            return;
        };

        let _ = selection.update(app, |s, _cx| {
            s.lead_entity = Some(id);
            s.selected_entities = vec![id];
        });

        app.with_global_mut(ProjectSelectionService::default, |s, _app| {
            s.set_selected_guid(None);
        });
    }

    fn play_time_seconds(&self) -> f32 {
        if !self.play_mode {
            return 0.0;
        }
        self.play_started_at
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0)
    }

    fn load_layout_file() -> Option<DockLayoutV1> {
        let path = Self::layout_path();
        let file = File::open(path).ok()?;
        serde_json::from_reader(file).ok()
    }

    fn load_keymap_file() -> Result<Keymap, fret_app::KeymapError> {
        Keymap::from_file(Self::keymap_path())
    }

    fn load_viewport_cameras_file() -> Option<ViewportCamerasFileV1> {
        let file = File::open(Self::viewport_cameras_path()).ok()?;
        serde_json::from_reader(file).ok()
    }

    fn save_layout_file(layout: &DockLayoutV1) -> std::io::Result<()> {
        if let Some(parent) = Self::layout_path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(Self::layout_path())?;
        serde_json::to_writer_pretty(file, layout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn save_layout_to_path(path: &Path, layout: &DockLayoutV1) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, layout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn load_layout_from_path(path: &Path) -> Option<DockLayoutV1> {
        let file = File::open(path).ok()?;
        serde_json::from_reader(file).ok()
    }

    fn bump_next_floating_index_from_layout(&mut self, layout: &DockLayoutV1) {
        let mut max_seen: u32 = 0;
        for w in &layout.windows {
            let Some(suffix) = w.logical_window_id.strip_prefix("floating-") else {
                continue;
            };
            let Ok(n) = suffix.parse::<u32>() else {
                continue;
            };
            max_seen = max_seen.max(n);
        }
        self.next_floating_index = self.next_floating_index.max(max_seen.saturating_add(1));
    }

    fn save_viewport_cameras_file(file_v1: &ViewportCamerasFileV1) -> std::io::Result<()> {
        if let Some(parent) = Self::viewport_cameras_path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(Self::viewport_cameras_path())?;
        serde_json::to_writer_pretty(file, file_v1)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    fn persist_layout_now(&mut self, app: &mut App) {
        let Some(dock) = app.global::<DockManager>() else {
            return;
        };
        let windows = self.window_list_for_export(dock);
        let layout = dock
            .graph
            .export_layout_v1_with_placement(&windows, |w| self.window_placements.get(&w).cloned());
        if let Err(e) = Self::save_layout_file(&layout) {
            tracing::error!(error = ?e, "failed to save layout.json");
        }
    }

    fn save_last_layout_preset(&mut self, app: &mut App) -> bool {
        let Some(dock) = app.global::<DockManager>() else {
            return false;
        };
        let windows = self.window_list_for_export(dock);
        let layout = dock
            .graph
            .export_layout_v1_with_placement(&windows, |w| self.window_placements.get(&w).cloned());

        let path = Self::last_layout_preset_path();
        if let Err(err) = Self::save_layout_to_path(&path, &layout) {
            tracing::error!(error = %err, path = %path.to_string_lossy(), "failed to save layout preset");
            return false;
        }
        true
    }

    fn apply_layout_v1(&mut self, app: &mut App, layout: DockLayoutV1) -> bool {
        let Some(main_window) = self.main_window else {
            return false;
        };

        let theme = Theme::global(app).snapshot();
        let missing_color = Color {
            a: 1.0,
            ..theme.colors.hover_background
        };

        let windows_to_close: Vec<fret_core::AppWindowId> = self
            .logical_windows
            .keys()
            .copied()
            .filter(|w| *w != main_window)
            .collect();

        {
            let Some(dock) = app.global_mut::<DockManager>() else {
                return false;
            };

            dock.graph = fret_core::DockGraph::new();

            Self::ensure_layout_panels(dock, &layout, missing_color);

            let Some(main_entry) = layout
                .windows
                .iter()
                .find(|w| w.logical_window_id == "main")
            else {
                return false;
            };

            let Some(root) = dock
                .graph
                .import_subtree_from_layout_v1(&layout, main_entry.root)
            else {
                return false;
            };
            dock.graph.set_window_root(main_window, root);

            for w in &windows_to_close {
                dock.graph.remove_window_root(*w);
                dock.clear_viewport_layout_for_window(*w);
            }
        }

        for w in windows_to_close {
            self.logical_windows.remove(&w);
            self.window_placements.remove(&w);
            app.push_effect(Effect::Window(WindowRequest::Close(w)));
        }

        self.loaded_layout = Some(layout.clone());
        self.bump_next_floating_index_from_layout(&layout);

        if let Err(err) = Self::save_layout_file(&layout) {
            tracing::error!(error = %err, "failed to save active layout.json");
        }

        for entry in &layout.windows {
            if entry.logical_window_id == "main" {
                continue;
            }
            app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                kind: CreateWindowKind::DockRestore {
                    logical_window_id: entry.logical_window_id.clone(),
                },
                anchor: None,
            })));
        }

        app.push_effect(Effect::UiInvalidateLayout {
            window: main_window,
        });
        app.request_redraw(main_window);
        true
    }

    fn load_last_layout_preset(&mut self, app: &mut App) -> bool {
        let path = Self::last_layout_preset_path();
        let Some(layout) = Self::load_layout_from_path(&path) else {
            return false;
        };
        self.apply_layout_v1(app, layout)
    }

    fn persist_viewport_cameras_now(&mut self) {
        let mut cameras: Vec<ViewportCameraEntryV1> = self
            .viewport_cameras
            .iter()
            .map(|(panel, camera)| ViewportCameraEntryV1 {
                panel: panel.clone(),
                camera: *camera,
            })
            .collect();
        cameras.sort_by(|a, b| a.panel.kind.0.cmp(&b.panel.kind.0));

        let file_v1 = ViewportCamerasFileV1 {
            version: 1,
            cameras,
        };
        if let Err(e) = Self::save_viewport_cameras_file(&file_v1) {
            tracing::error!(error = ?e, "failed to save viewport_cameras.json");
        }
    }

    fn schedule_layout_persist(&mut self, app: &mut App) {
        let Some(main) = self.main_window else {
            return;
        };
        let token = match self.dock_persist_timer {
            Some(t) => t,
            None => {
                let t = app.next_timer_token();
                self.dock_persist_timer = Some(t);
                t
            }
        };
        self.dock_persist_pending = true;
        app.push_effect(Effect::CancelTimer { token });
        app.push_effect(Effect::SetTimer {
            window: Some(main),
            token,
            after: Duration::from_millis(500),
            repeat: None,
        });
    }

    fn reset_layout_to_default(&mut self, app: &mut App) -> bool {
        let Some(main_window) = self.main_window else {
            return false;
        };

        // Close all non-main dock windows.
        let windows_to_close: Vec<fret_core::AppWindowId> = self
            .logical_windows
            .keys()
            .copied()
            .filter(|w| *w != main_window)
            .collect();

        {
            let Some(dock) = app.global_mut::<DockManager>() else {
                return false;
            };

            let key_scene = PanelKey::new("core.scene");
            let key_game = PanelKey::new("core.game");
            let key_text_probe = PanelKey::new("core.text_probe");
            let key_project = PanelKey::new("core.project");
            let key_inspector = PanelKey::new("core.inspector");
            let key_hierarchy = PanelKey::new("core.hierarchy");

            // Rebuild a clean dock graph (panels remain registered).
            dock.graph = fret_core::DockGraph::new();

            let tabs_left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_hierarchy, key_project],
                active: 0,
            });
            let tabs_scene = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_scene, key_game],
                active: 0,
            });
            let tabs_inspector = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_inspector, key_text_probe],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Vertical,
                children: vec![tabs_scene, tabs_inspector],
                fractions: vec![0.72, 0.28],
            });
            let root_dock = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![tabs_left, right],
                fractions: vec![0.26, 0.74],
            });
            dock.graph.set_window_root(main_window, root_dock);

            for w in &windows_to_close {
                dock.graph.remove_window_root(*w);
                dock.clear_viewport_layout_for_window(*w);
            }
        }

        for w in windows_to_close {
            self.logical_windows.remove(&w);
            self.window_placements.remove(&w);
            app.push_effect(Effect::Window(WindowRequest::Close(w)));
        }

        self.loaded_layout = None;

        app.push_effect(Effect::UiInvalidateLayout {
            window: main_window,
        });
        app.request_redraw(main_window);

        self.persist_layout_now(app);
        self.loaded_layout = Self::load_layout_file();

        true
    }

    fn schedule_camera_persist(&mut self, app: &mut App) {
        let token = match self.camera_persist_timer {
            Some(t) => t,
            None => {
                let t = app.next_timer_token();
                self.camera_persist_timer = Some(t);
                t
            }
        };
        self.camera_persist_pending = true;
        app.push_effect(Effect::CancelTimer { token });
        app.push_effect(Effect::SetTimer {
            window: self.main_window,
            token,
            after: Duration::from_millis(500),
            repeat: None,
        });
    }

    fn ensure_theme_hot_reload(&mut self, app: &mut App, window: fret_core::AppWindowId) {
        if self.theme_reload_timer.is_some() {
            return;
        }

        let token = app.next_timer_token();
        self.theme_reload_timer = Some(token);
        app.push_effect(Effect::SetTimer {
            window: Some(window),
            token,
            after: Duration::from_millis(250),
            repeat: Some(Duration::from_millis(250)),
        });
    }

    fn apply_theme_bytes(&mut self, app: &mut App, path: &str, bytes: &[u8]) -> ThemeApplyOutcome {
        let cfg = match ThemeConfig::from_slice(bytes) {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::error!(error = %err, path = %path, "failed to parse theme file");
                return ThemeApplyOutcome::Failed;
            }
        };

        let before = Theme::global(app).revision();
        Theme::global_mut(app).apply_config(&cfg);
        let after = Theme::global(app).revision();
        if after == before {
            return ThemeApplyOutcome::NoChange;
        }

        tracing::info!(theme = %cfg.name, path = %path, "applied theme");
        for &w in self.logical_windows.keys() {
            app.push_effect(Effect::UiInvalidateLayout { window: w });
            app.request_redraw(w);
        }
        ThemeApplyOutcome::Applied
    }

    fn write_theme_override(&self, bytes: &[u8]) -> bool {
        let path = theme_override_path();
        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                tracing::error!(error = %err, path = %path, "failed to create theme override dir");
                return false;
            }
        }
        if let Err(err) = std::fs::write(path, bytes) {
            tracing::error!(error = %err, path = %path, "failed to write theme override file");
            return false;
        }
        true
    }

    fn reset_theme_override(&mut self) {
        self.theme_file_modified = None;
        self.theme_file_size = None;
        self.theme_file_hash = None;
        let _ = std::fs::remove_file(theme_override_path());
    }

    fn reload_theme_if_changed(&mut self, app: &mut App) {
        let path = theme_override_path();
        let Ok(meta) = std::fs::metadata(path) else {
            return;
        };

        let modified = meta.modified().ok();
        let size = meta.len();
        if self.theme_file_modified == modified && self.theme_file_size == Some(size) {
            return;
        }

        let Ok(bytes) = std::fs::read(path) else {
            return;
        };

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();

        self.theme_file_modified = modified;
        self.theme_file_size = Some(size);

        if self.theme_file_hash == Some(hash) {
            return;
        }
        self.theme_file_hash = Some(hash);

        match ThemeConfig::from_slice(&bytes) {
            Ok(cfg) => {
                let before = Theme::global(app).revision();
                {
                    let theme = Theme::global_mut(app);
                    theme.apply_config(&cfg);
                }
                let after = Theme::global(app).revision();
                if after != before {
                    tracing::info!(theme = %cfg.name, path = %path, "reloaded theme");
                    for &w in self.logical_windows.keys() {
                        app.push_effect(Effect::UiInvalidateLayout { window: w });
                        app.request_redraw(w);
                    }
                }
            }
            Err(err) => {
                tracing::error!(error = %err, path = %path, "failed to parse theme file");
            }
        }
    }

    fn ensure_main_tabs(
        dock: &mut DockManager,
        main: fret_core::AppWindowId,
    ) -> fret_core::DockNodeId {
        dock.graph.first_tabs_in_window(main).unwrap_or_else(|| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: Vec::new(),
                active: 0,
            });
            dock.graph.set_window_root(main, tabs);
            tabs
        })
    }

    fn next_floating_logical_id(&mut self) -> String {
        let n = self.next_floating_index.max(1);
        self.next_floating_index = n.saturating_add(1);
        format!("floating-{n}")
    }

    fn window_list_for_export(
        &mut self,
        dock: &DockManager,
    ) -> Vec<(fret_core::AppWindowId, String)> {
        let mut out: Vec<(fret_core::AppWindowId, String)> = Vec::new();
        for (&window, logical) in &self.logical_windows {
            if dock.graph.window_root(window).is_some() {
                out.push((window, logical.clone()));
            }
        }
        out
    }

    fn ensure_layout_panels(dock: &mut DockManager, layout: &DockLayoutV1, missing_color: Color) {
        for node in &layout.nodes {
            if let DockLayoutNodeV1::Tabs { tabs, .. } = node {
                for key in tabs {
                    dock.ensure_panel(key, || DockPanel {
                        title: format!("Missing: {}", key.kind.0),
                        color: missing_color,
                        viewport: None,
                    });
                }
            }
        }
    }

    fn selection_model(&self) -> Option<Model<DemoSelection>> {
        self.selection
    }

    fn set_selection(&mut self, app: &mut App, lead: Option<u64>, mut selected: Vec<u64>) {
        selected.sort_unstable();
        selected.dedup();

        app.with_global_mut(ProjectSelectionService::default, |s, _app| {
            s.set_selected_guid(None);
        });

        let Some(model) = self.selection_model() else {
            return;
        };
        let _ = model.update(app, |s, _cx| {
            s.lead_entity = lead;
            s.selected_entities = selected;
        });

        for &w in self.logical_windows.keys() {
            app.request_redraw(w);
        }
    }

    fn apply_selection_delta(
        &mut self,
        app: &mut App,
        lead: Option<u64>,
        delta: Vec<u64>,
        modifiers: fret_core::Modifiers,
    ) {
        let Some(model) = self.selection_model() else {
            return;
        };
        let cur = model.get(app).cloned().unwrap_or_default();
        let mut selected = cur.selected_entities;

        if modifiers.ctrl || modifiers.meta {
            use std::collections::HashSet;
            let mut set: HashSet<u64> = selected.into_iter().collect();
            for id in delta {
                if set.contains(&id) {
                    set.remove(&id);
                } else {
                    set.insert(id);
                }
            }
            selected = set.into_iter().collect();
        } else if modifiers.shift {
            selected.extend(delta);
        } else {
            selected = delta;
        }

        selected.sort_unstable();
        selected.dedup();

        let lead = lead.filter(|id| selected.binary_search(id).is_ok());
        let lead = lead.or_else(|| selected.last().copied());

        self.set_selection(app, lead, selected);
    }

    fn sync_viewport_selection_overlay_for_window(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) {
        let lead = self
            .selection_model()
            .and_then(|m| m.get(app))
            .and_then(|s| s.lead_entity);

        let lead_pos = lead.and_then(|id| {
            let world = self.world.and_then(|m| m.get(app))?;
            Some(world.position(id))
        });

        let tool_mode = self
            .viewport_tools
            .and_then(|m| m.get(app))
            .map(|t| t.active)
            .unwrap_or_default();

        let gizmo_highlight =
            self.viewport_tools
                .and_then(|m| m.get(app))
                .and_then(|t| match &t.interaction {
                    Some(ViewportInteraction::TranslateGizmo(g)) => match g.constraint {
                        TranslateAxisConstraint::Free => {
                            Some(fret_ui::dock::ViewportGizmoPart::Handle)
                        }
                        TranslateAxisConstraint::X => Some(fret_ui::dock::ViewportGizmoPart::X),
                        TranslateAxisConstraint::Y => Some(fret_ui::dock::ViewportGizmoPart::Y),
                    },
                    _ => None,
                });
        let rotate_gizmo_state = self.viewport_tools.and_then(|m| m.get(app)).map(|t| {
            (
                matches!(t.interaction, Some(ViewportInteraction::RotateGizmo(_))),
                t.hover_rotate,
            )
        });

        let theme = Theme::global(app).snapshot();
        let selection_fill = theme.colors.viewport_selection_fill;
        let selection_stroke = theme.colors.viewport_selection_stroke;

        let Some(dock) = app.global_mut::<DockManager>() else {
            return;
        };
        if dock.graph.window_root(window).is_none() {
            return;
        }
        for panel_key in dock.graph.collect_panels_in_window(window) {
            let Some(panel) = dock.panels.get(&panel_key) else {
                continue;
            };
            let Some(vp) = panel.viewport else {
                continue;
            };

            if demo_viewport_role(&panel_key) == DemoViewportRole::Game {
                dock.set_viewport_marquee(window, vp.target, None);
                dock.set_viewport_drag_line(window, vp.target, None);
                dock.set_viewport_selection_rect(window, vp.target, None);
                dock.set_viewport_marker(window, vp.target, None);
                dock.set_viewport_gizmo(window, vp.target, None);
                dock.set_viewport_rotate_gizmo(window, vp.target, None);
                continue;
            }

            let camera = self.viewport_camera(&panel_key);
            let marker_uv_from_world = lead_pos.map(|pos| camera.world_to_uv(pos));
            let marker_uv = marker_uv_from_world.or_else(|| lead.and_then(viewport_grid_marker_uv));

            let marker = marker_uv.map(|uv| fret_ui::dock::ViewportMarker {
                uv,
                color: theme.colors.viewport_marker,
            });

            let selection_rect = if let Some(center_uv) = marker_uv_from_world {
                Some(viewport_selection_rect_around_uv(
                    center_uv,
                    vp.target_px_size,
                    8.0,
                    selection_fill,
                    selection_stroke,
                ))
            } else {
                lead.and_then(viewport_grid_cell_uv_rect)
                    .map(|(min_uv, max_uv)| fret_ui::dock::ViewportSelectionRect {
                        min_uv,
                        max_uv,
                        fill: selection_fill,
                        stroke: selection_stroke,
                    })
            };

            let gizmo = match tool_mode {
                ViewportToolMode::Move => marker_uv.map(|center_uv| fret_ui::dock::ViewportGizmo {
                    center_uv,
                    axis_len_px: fret_core::geometry::Px(80.0),
                    highlight: gizmo_highlight,
                }),
                _ => None,
            };
            let rotate_gizmo = match tool_mode {
                ViewportToolMode::Rotate => marker_uv.map(|center_uv| {
                    let highlight = rotate_gizmo_state.is_some_and(|(active, hover)| {
                        active || hover == Some((window, vp.target))
                    });
                    fret_ui::dock::ViewportRotateGizmo {
                        center_uv,
                        radius_px: fret_core::geometry::Px(56.0),
                        highlight,
                    }
                }),
                _ => None,
            };

            dock.set_viewport_selection_rect(window, vp.target, selection_rect);
            dock.set_viewport_marker(window, vp.target, marker);
            dock.set_viewport_gizmo(window, vp.target, gizmo);
            dock.set_viewport_rotate_gizmo(window, vp.target, rotate_gizmo);
        }
    }

    fn handle_asset_drop_requests(&mut self, app: &mut App) -> bool {
        let requests = app.with_global_mut(AssetDropService::default, |s, _app| s.drain());
        if requests.is_empty() {
            return false;
        }

        let mut registry = std::mem::take(&mut self.asset_drop_registry);
        let mut handled_any = false;
        for req in requests {
            let decision = registry.handle(self, app, req);
            handled_any |= decision == AssetDropDecision::Handled;
        }
        self.asset_drop_registry = registry;
        handled_any
    }
}

fn demo_pick_entity_by_uv(
    world: &DemoWorld,
    camera: DemoViewportCamera,
    uv: (f32, f32),
    target_px_size: Option<(u32, u32)>,
) -> Option<u64> {
    const GRID_W: u64 = 64;
    const GRID_H: u64 = 36;
    const PICK_RADIUS_PX: f32 = 24.0;

    let (u, v) = uv;
    let (tw, th) = target_px_size.unwrap_or((1024, 768));
    let tw = tw.max(1) as f32;
    let th = th.max(1) as f32;

    let mut best: Option<(u64, f32)> = None;
    for id in 1..=(GRID_W * GRID_H) {
        let (eu, ev) = camera.world_to_uv(world.position(id));
        let dx = (eu - u) * tw;
        let dy = (ev - v) * th;
        let d2 = dx * dx + dy * dy;
        match best {
            None => best = Some((id, d2)),
            Some((_, best_d2)) if d2 < best_d2 => best = Some((id, d2)),
            _ => {}
        }
    }

    let (id, d2) = best?;
    if d2 <= PICK_RADIUS_PX * PICK_RADIUS_PX {
        Some(id)
    } else {
        None
    }
}

fn demo_pick_entities_in_uv_rect(
    world: &DemoWorld,
    camera: DemoViewportCamera,
    a_uv: (f32, f32),
    b_uv: (f32, f32),
) -> Vec<u64> {
    const GRID_W: u64 = 64;
    const GRID_H: u64 = 36;

    let (u0, v0) = (a_uv.0.min(b_uv.0), a_uv.1.min(b_uv.1));
    let (u1, v1) = (a_uv.0.max(b_uv.0), a_uv.1.max(b_uv.1));

    let mut out: Vec<u64> = Vec::new();
    for id in 1..=(GRID_W * GRID_H) {
        let (eu, ev) = camera.world_to_uv(world.position(id));
        if eu >= u0 && eu <= u1 && ev >= v0 && ev <= v1 {
            out.push(id);
            if out.len() >= 2048 {
                break;
            }
        }
    }
    out
}

fn viewport_selection_rect_around_uv(
    center_uv: (f32, f32),
    target_px_size: (u32, u32),
    half_size_px: f32,
    fill: Color,
    stroke: Color,
) -> fret_ui::dock::ViewportSelectionRect {
    let (tw, th) = target_px_size;
    let tw = (tw.max(1) as f32).max(1.0);
    let th = (th.max(1) as f32).max(1.0);
    let (u, v) = center_uv;
    let du = half_size_px / tw;
    let dv = half_size_px / th;
    fret_ui::dock::ViewportSelectionRect {
        min_uv: ((u - du).clamp(0.0, 1.0), (v - dv).clamp(0.0, 1.0)),
        max_uv: ((u + du).clamp(0.0, 1.0), (v + dv).clamp(0.0, 1.0)),
        fill,
        stroke,
    }
}

fn viewport_grid_marker_uv(id: u64) -> Option<(f32, f32)> {
    if id == 0 {
        return None;
    }
    let grid_w: u64 = 64;
    let grid_h: u64 = 36;

    let idx = id.saturating_sub(1);
    let x = idx % grid_w;
    let y = idx / grid_w;
    if y >= grid_h {
        return None;
    }

    let u = (x as f32 + 0.5) / grid_w as f32;
    let v = (y as f32 + 0.5) / grid_h as f32;
    Some((u, v))
}

fn viewport_gizmo_hit_test_px(
    center_uv: (f32, f32),
    target_px_size: (u32, u32),
    cursor_target_px: (u32, u32),
    axis_len_px: f32,
    thickness_px: f32,
    handle_px: f32,
) -> Option<TranslateAxisConstraint> {
    let (tw, th) = target_px_size;
    let tw = tw.max(1) as f32;
    let th = th.max(1) as f32;
    let (u, v) = center_uv;
    let cx = u * tw;
    let cy = v * th;

    let (x, y) = (cursor_target_px.0 as f32, cursor_target_px.1 as f32);
    let dx = x - cx;
    let dy = y - cy;

    let half_handle = handle_px * 0.5;
    let on_handle = dx.abs() <= half_handle && dy.abs() <= half_handle;

    let on_x_axis = dx >= 0.0 && dx <= axis_len_px && dy.abs() <= thickness_px;
    let on_y_axis = dy <= 0.0 && dy >= -axis_len_px && dx.abs() <= thickness_px;

    if on_handle {
        return Some(TranslateAxisConstraint::Free);
    }
    if on_x_axis {
        return Some(TranslateAxisConstraint::X);
    }
    if on_y_axis {
        return Some(TranslateAxisConstraint::Y);
    }
    None
}

fn viewport_rotate_gizmo_hit_test_px(
    center_uv: (f32, f32),
    target_px_size: (u32, u32),
    cursor_target_px: (u32, u32),
    radius_px: f32,
    thickness_px: f32,
) -> bool {
    let (tw, th) = target_px_size;
    let tw = tw.max(1) as f32;
    let th = th.max(1) as f32;
    let (u, v) = center_uv;
    let cx = u * tw;
    let cy = v * th;

    let (x, y) = (cursor_target_px.0 as f32, cursor_target_px.1 as f32);
    let dx = x - cx;
    let dy = y - cy;
    let d = (dx * dx + dy * dy).sqrt();
    (d - radius_px).abs() <= thickness_px.max(1.0)
}

fn normalize_angle_rad(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

fn viewport_grid_cell_uv_rect(id: u64) -> Option<((f32, f32), (f32, f32))> {
    if id == 0 {
        return None;
    }
    let grid_w: u64 = 64;
    let grid_h: u64 = 36;

    let idx = id.saturating_sub(1);
    let x = idx % grid_w;
    let y = idx / grid_w;
    if y >= grid_h {
        return None;
    }

    let u0 = x as f32 / grid_w as f32;
    let v0 = y as f32 / grid_h as f32;
    let u1 = (x as f32 + 1.0) / grid_w as f32;
    let v1 = (y as f32 + 1.0) / grid_h as f32;
    Some(((u0, v0), (u1, v1)))
}

impl WinitDriver for DemoDriver {
    type WindowState = DemoWindowState;

    fn gpu_ready(&mut self, _app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        let size = (512u32, 512u32);
        self.background = Some(SceneBackgroundRenderer::new(
            &context.device,
            ViewportTarget::FORMAT,
        ));
        let mut targets = ViewportTargets::default();
        targets.insert(
            PanelKey::new("core.scene"),
            ViewportTarget::new(
                &context.device,
                renderer,
                "fret-demo scene render target",
                size,
            ),
        );
        targets.insert(
            PanelKey::new("core.game"),
            ViewportTarget::new(
                &context.device,
                renderer,
                "fret-demo game render target",
                size,
            ),
        );
        self.viewport_targets = Some(targets);
    }

    fn record_engine_frame(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        _state: &mut Self::WindowState,
        context: &WgpuContext,
        _renderer: &mut Renderer,
        scale_factor: f32,
        _tick_id: fret_core::TickId,
        _frame_id: fret_core::FrameId,
    ) -> EngineFrameUpdate {
        let Some(bg) = self.background.as_ref() else {
            return EngineFrameUpdate::default();
        };
        let Some(targets) = self.viewport_targets.as_ref() else {
            return EngineFrameUpdate::default();
        };
        let panels: Vec<PanelKey> = targets.panel_keys().cloned().collect();

        let play_time = self.play_time_seconds();
        let mut wants_raf = false;

        let mut target_updates: Vec<RenderTargetUpdate> = Vec::new();
        let mut cmds: Vec<wgpu::CommandBuffer> = Vec::new();

        for panel in panels {
            let camera = self.viewport_camera(&panel);
            let Some(target_id) = self
                .viewport_targets
                .as_ref()
                .and_then(|t| t.get(&panel).map(|t| t.target))
            else {
                continue;
            };
            let content = app
                .global::<DockManager>()
                .and_then(|dock| dock.viewport_content_rect(window, target_id));
            let Some(content) = content else {
                continue;
            };

            let desired_px = ViewportTarget::desired_px_from_content(content, scale_factor);
            let Some(target) = self
                .viewport_targets
                .as_mut()
                .and_then(|t| t.get_mut(&panel))
            else {
                continue;
            };

            if let Some(update) = target.resize(&context.device, desired_px) {
                target_updates.push(update);
                if let Some(dock) = app.global_mut::<DockManager>() {
                    dock.update_viewport_target_px_size(target.target, desired_px);
                }
                app.request_redraw(window);
            }

            let role = demo_viewport_role(&panel);
            let time = if role == DemoViewportRole::Game && self.play_mode {
                wants_raf = true;
                play_time
            } else {
                0.0
            };

            cmds.push(bg.record_commands(
                &context.device,
                &context.queue,
                &target.view,
                target.target_px_size,
                SceneCameraParams {
                    center: camera.center,
                    zoom: camera.zoom,
                    rotation: camera.rotation,
                    world_span: DemoViewportCamera::WORLD_SPAN,
                    time,
                },
            ));
        }

        if wants_raf {
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        EngineFrameUpdate {
            target_updates,
            command_buffers: cmds,
        }
    }

    fn init(&mut self, app: &mut App, main_window: fret_core::AppWindowId) {
        self.main_window = Some(main_window);
        self.logical_windows.insert(main_window, "main".to_string());

        if self.viewport_cameras.is_empty() {
            if let Some(file) = Self::load_viewport_cameras_file() {
                if file.version == 1 {
                    self.viewport_cameras = file
                        .cameras
                        .into_iter()
                        .map(|e| (e.panel, e.camera))
                        .collect();
                }
            }
        }

        app.set_global(AssetDropService::default());
        app.set_global(CurrentSceneService::default());
        app.set_global(SceneDocumentService::default());
        app.set_global(TextProbeService::default());
        app.with_global_mut(TextProbeService::default, |s, _app| {
            s.set(None, TEXT_PROBE_DEFAULT.to_string());
        });
        app.set_global(UnsavedChangesService::default());
        app.set_global(InspectorEditService::default());
        app.set_global(PropertyEditService::default());
        app.set_global(DebugHudService::default());
        app.set_global(DebugInspectorService::default());
        load_theme(app);
        self.ensure_theme_hot_reload(app, main_window);
        self.install_asset_drop_rules();
        self.install_asset_open_rules();

        self.last_scene_chrome_rev = None;

        app.commands_mut().register(
            CommandId::from("command_palette.toggle"),
            CommandMeta::new("Toggle Command Palette")
                .with_description("Opens/closes the command palette overlay")
                .with_category("View")
                .with_keywords(["palette", "command", "search"]),
        );
        app.commands_mut().register(
            CommandId::from("command_palette.close"),
            CommandMeta::new("Close Command Palette")
                .with_description("Closes the command palette overlay")
                .with_category("View")
                .with_keywords(["palette", "command"]),
        );

        app.commands_mut().register(
            CommandId::from("demo.play.toggle"),
            CommandMeta::new("Toggle Play Mode")
                .with_description("Toggles play mode for the Game viewport (animation preview)")
                .with_category("Game")
                .with_keywords(["play", "run", "game"]),
        );

        app.commands_mut().register(
            CommandId::from("debug.hud.toggle"),
            CommandMeta::new("Toggle UI Debug HUD")
                .with_description("Toggles the UI debug HUD overlay (layout/paint timings)")
                .with_category("View")
                .with_keywords(["debug", "hud", "ui", "profiling"]),
        );
        app.commands_mut().register(
            CommandId::from("debug.inspector.toggle"),
            CommandMeta::new("Toggle UI Inspector Overlay")
                .with_description("Toggles the UI inspector overlay (hit-test + bounds outlines)")
                .with_category("View")
                .with_keywords(["debug", "inspector", "ui", "hit-test"]),
        );

        app.commands_mut().register(
            CommandId::from("theme.reload"),
            CommandMeta::new("Reload Theme")
                .with_description("Reloads theme from .fret/theme.json or built-in fallbacks")
                .with_category("View")
                .with_keywords(["theme", "style", "reload"]),
        );
        app.commands_mut().register(
            CommandId::from("theme.reset_override"),
            CommandMeta::new("Reset Theme Override")
                .with_description("Deletes .fret/theme.json and reloads the default theme")
                .with_category("View")
                .with_keywords(["theme", "style", "reset"]),
        );
        app.commands_mut().register(
            CommandId::from("theme.set.fret_default_dark"),
            CommandMeta::new("Theme: Fret Default (Dark)")
                .with_description("Applies the built-in Fret default dark theme")
                .with_category("View")
                .with_keywords(["theme", "style", "fret"]),
        );
        app.commands_mut().register(
            CommandId::from("theme.set.hardhacker_dark"),
            CommandMeta::new("Theme: HardHacker-inspired Dark")
                .with_description("Applies the built-in HardHacker-inspired dark theme")
                .with_category("View")
                .with_keywords(["theme", "style", "hardhacker"]),
        );
        app.commands_mut().register(
            CommandId::from("theme.set.godot_default_dark"),
            CommandMeta::new("Theme: Godot Default (Dark)")
                .with_description("Applies the built-in Godot default dark theme mapping")
                .with_category("View")
                .with_keywords(["theme", "style", "godot"]),
        );

        app.commands_mut().register(
            CommandId::from("project.refresh"),
            CommandMeta::new("Refresh Project")
                .with_description(
                    "Rescans the demo project Assets folder and regenerates missing .meta files",
                )
                .with_category("Project")
                .with_keywords(["project", "assets", "refresh", "scan"]),
        );
        app.commands_mut().register(
            CommandId::from("project.rename_selected"),
            CommandMeta::new("Rename Selected Asset")
                .with_description(
                    "Renames the selected asset/folder and moves its .meta to preserve GUID",
                )
                .with_category("Project")
                .with_keywords(["project", "assets", "rename", "meta", "guid"]),
        );
        app.commands_mut().register(
            CommandId::from("project.move_selected_to_moved"),
            CommandMeta::new("Move Selected To Moved/")
                .with_description(
                    "Moves the selected asset/folder to Assets/Moved and moves its .meta to preserve GUID",
                )
                .with_category("Project")
                .with_keywords(["project", "assets", "move", "meta", "guid"]),
        );

        app.commands_mut().register(
            CommandId::from("asset.open_selected"),
            CommandMeta::new("Open Asset")
                .with_description("Opens the selected asset using the editor action registry")
                .with_category("Project")
                .with_keywords(["asset", "open"]),
        );

        app.commands_mut().register(
            CommandId::from("unsaved_dialog.open"),
            CommandMeta::new("Internal: Open Unsaved Changes Dialog")
                .with_description("Internal: opens the unsaved changes confirmation dialog")
                .with_category("Internal")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("unsaved_dialog.save"),
            CommandMeta::new("Internal: Save (Unsaved Dialog)")
                .with_description("Internal: saves and continues the pending unsaved action")
                .with_category("Internal")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("unsaved_dialog.discard"),
            CommandMeta::new("Internal: Discard (Unsaved Dialog)")
                .with_description(
                    "Internal: discards changes and continues the pending unsaved action",
                )
                .with_category("Internal")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("unsaved_dialog.cancel"),
            CommandMeta::new("Internal: Cancel (Unsaved Dialog)")
                .with_description("Internal: cancels the pending unsaved action")
                .with_category("Internal")
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("scene.open_selected"),
            CommandMeta::new("Open Scene")
                .with_description("Opens the selected .scene asset as the current scene document")
                .with_category("Scene")
                .with_keywords(["scene", "open", "load"]),
        );
        app.commands_mut().register(
            CommandId::from("scene.new"),
            CommandMeta::new("New Scene")
                .with_description("Creates a new .scene asset and opens it as the current scene")
                .with_category("Scene")
                .with_keywords(["scene", "new", "create"]),
        );
        app.commands_mut().register(
            CommandId::from("scene.save"),
            CommandMeta::new("Save Scene")
                .with_description("Saves the current scene document to disk")
                .with_category("Scene")
                .with_keywords(["scene", "save"]),
        );
        app.commands_mut().register(
            CommandId::from("scene.save_as"),
            CommandMeta::new("Save Scene As")
                .with_description("Saves the current scene document as a new .scene asset")
                .with_category("Scene")
                .with_keywords(["scene", "save", "save as", "copy"]),
        );

        app.commands_mut().register(
            CommandId::from("context_menu.open"),
            CommandMeta::new("Open Context Menu")
                .with_description("Internal: opens a context menu overlay")
                .with_category("View")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("context_menu.close"),
            CommandMeta::new("Close Context Menu")
                .with_description("Closes the context menu overlay")
                .with_category("View")
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("inspector_edit.open"),
            CommandMeta::new("Open Inspector Edit")
                .with_description("Internal: opens the inspector value editor popup")
                .with_category("Inspector")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("inspector_edit.close"),
            CommandMeta::new("Close Inspector Edit")
                .with_description("Internal: closes the inspector value editor popup")
                .with_category("Inspector")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("inspector_edit.commit"),
            CommandMeta::new("Commit Inspector Edit")
                .with_description("Internal: commits the inspector value editor popup")
                .with_category("Inspector")
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("property_edit.commit"),
            CommandMeta::new("Commit Property Edit")
                .with_description("Internal: commits a property edit request")
                .with_category("Inspector")
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("tree_view.expand"),
            CommandMeta::new("Expand")
                .with_description("Expand the selected tree node")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.collapse"),
            CommandMeta::new("Collapse")
                .with_description("Collapse the selected tree node")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.expand_all"),
            CommandMeta::new("Expand All")
                .with_description("Expand all tree nodes")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("tree_view.collapse_all"),
            CommandMeta::new("Collapse All")
                .with_description("Collapse all tree nodes")
                .with_category("Hierarchy")
                .with_scope(CommandScope::Widget),
        );

        app.commands_mut().register(
            CommandId::from("virtual_list.copy_label"),
            CommandMeta::new("Copy Label")
                .with_description("Copies the label of the selected list row")
                .with_category("List")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("virtual_list.clear_selection"),
            CommandMeta::new("Clear Selection")
                .with_description("Clears the list selection")
                .with_category("List")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("viewport.copy_uv"),
            CommandMeta::new("Copy Viewport UV")
                .with_description("Copies the last viewport cursor UV to clipboard")
                .with_category("Viewport")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("viewport.copy_target_px"),
            CommandMeta::new("Copy Viewport Target Px")
                .with_description("Copies the last viewport cursor target pixel coordinates")
                .with_category("Viewport")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );

        app.commands_mut().register(
            CommandId::from("viewport.tool.select"),
            CommandMeta::new("Viewport Tool: Select")
                .with_description("Switches the active viewport tool to Select")
                .with_category("Viewport"),
        );
        app.commands_mut().register(
            CommandId::from("viewport.tool.move"),
            CommandMeta::new("Viewport Tool: Move")
                .with_description("Switches the active viewport tool to Move/Translate")
                .with_category("Viewport"),
        );
        app.commands_mut().register(
            CommandId::from("viewport.tool.rotate"),
            CommandMeta::new("Viewport Tool: Rotate")
                .with_description("Switches the active viewport tool to Rotate")
                .with_category("Viewport"),
        );

        app.commands_mut().register(
            CommandId::from("dock.tab.float"),
            CommandMeta::new("Float Tab")
                .with_description("Floats the current dock tab into a new window")
                .with_category("Dock")
                .with_scope(CommandScope::Widget)
                .with_when(WhenExpr::parse("ui.window_tear_off").expect("valid when expr"))
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("dock.tab.move_left"),
            CommandMeta::new("Move Tab Left")
                .with_description("Moves the current dock tab left")
                .with_category("Dock")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("dock.tab.move_right"),
            CommandMeta::new("Move Tab Right")
                .with_description("Moves the current dock tab right")
                .with_category("Dock")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("dock.tab.close"),
            CommandMeta::new("Close Tab")
                .with_description("Closes the current dock tab")
                .with_category("Dock")
                .with_scope(CommandScope::Widget)
                .hidden(),
        );
        app.commands_mut().register(
            CommandId::from("dock.layout.reset_default"),
            CommandMeta::new("Reset Layout")
                .with_description("Resets the dock layout back to the default layout")
                .with_category("Dock")
                .with_scope(CommandScope::App)
                .with_keywords(["dock", "layout", "reset", "default"]),
        );
        app.commands_mut().register(
            CommandId::from("dock.layout.preset.save_last"),
            CommandMeta::new("Save Layout Preset (Last)")
                .with_description("Saves the current dock layout as the 'last' preset")
                .with_category("Dock")
                .with_scope(CommandScope::App)
                .with_keywords(["dock", "layout", "preset", "save"]),
        );
        app.commands_mut().register(
            CommandId::from("dock.layout.preset.load_last"),
            CommandMeta::new("Load Layout Preset (Last)")
                .with_description("Loads the 'last' layout preset")
                .with_category("Dock")
                .with_scope(CommandScope::App)
                .with_keywords(["dock", "layout", "preset", "load"]),
        );

        app.commands_mut().register(
            CommandId::from("demo.toggle_modal"),
            CommandMeta::new("Toggle Modal Overlay")
                .with_description("Demo-only: toggles the modal overlay layer")
                .with_category("Demo"),
        );
        app.commands_mut().register(
            CommandId::from("demo.toggle_dnd_overlay"),
            CommandMeta::new("Toggle DnD Overlay")
                .with_description("Demo-only: toggles the external drag overlay layer")
                .with_category("Demo"),
        );
        app.commands_mut().register(
            CommandId::from("text.clear"),
            CommandMeta::new("Clear Text Input")
                .with_description("Clears the focused text input")
                .with_category("Edit")
                .with_keywords(["text", "input"])
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.select_all"),
            CommandMeta::new("Select All")
                .with_description("Select all text in the focused text input")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.copy"),
            CommandMeta::new("Copy")
                .with_description("Copy selected text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.cut"),
            CommandMeta::new("Cut")
                .with_description("Cut selected text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );
        app.commands_mut().register(
            CommandId::from("text.paste"),
            CommandMeta::new("Paste")
                .with_description("Paste clipboard text")
                .with_category("Edit")
                .with_scope(CommandScope::Widget),
        );

        app.commands_mut().register(
            CommandId::from("edit.undo"),
            CommandMeta::new("Undo")
                .with_description("Undo the last editor action")
                .with_category("Edit")
                .with_scope(CommandScope::App),
        );
        app.commands_mut().register(
            CommandId::from("edit.redo"),
            CommandMeta::new("Redo")
                .with_description("Redo the last undone editor action")
                .with_category("Edit")
                .with_scope(CommandScope::App),
        );

        for (id, title, desc) in [
            (
                "text.move_left",
                "Move Left",
                "Move caret left by one character",
            ),
            (
                "text.move_right",
                "Move Right",
                "Move caret right by one character",
            ),
            (
                "text.move_word_left",
                "Move Word Left",
                "Move caret left by one word",
            ),
            (
                "text.move_word_right",
                "Move Word Right",
                "Move caret right by one word",
            ),
            ("text.move_home", "Move Home", "Move caret to the start"),
            ("text.move_end", "Move End", "Move caret to the end"),
            ("text.move_up", "Move Up", "Move caret up by one line"),
            ("text.move_down", "Move Down", "Move caret down by one line"),
            (
                "text.select_left",
                "Select Left",
                "Extend selection left by one character",
            ),
            (
                "text.select_right",
                "Select Right",
                "Extend selection right by one character",
            ),
            (
                "text.select_word_left",
                "Select Word Left",
                "Extend selection left by one word",
            ),
            (
                "text.select_word_right",
                "Select Word Right",
                "Extend selection right by one word",
            ),
            (
                "text.select_home",
                "Select Home",
                "Extend selection to the start",
            ),
            (
                "text.select_end",
                "Select End",
                "Extend selection to the end",
            ),
            (
                "text.select_up",
                "Select Up",
                "Extend selection up by one line",
            ),
            (
                "text.select_down",
                "Select Down",
                "Extend selection down by one line",
            ),
            (
                "text.delete_backward",
                "Delete Backward",
                "Delete backward (or delete selection)",
            ),
            (
                "text.delete_forward",
                "Delete Forward",
                "Delete forward (or delete selection)",
            ),
            (
                "text.delete_word_backward",
                "Delete Word Backward",
                "Delete backward by one word (or delete selection)",
            ),
            (
                "text.delete_word_forward",
                "Delete Word Forward",
                "Delete forward by one word (or delete selection)",
            ),
        ] {
            let repeatable = id.starts_with("text.move")
                || id.starts_with("text.select")
                || id.starts_with("text.delete");
            let mut meta = CommandMeta::new(title)
                .with_description(desc)
                .with_category("Edit")
                .with_scope(CommandScope::Widget);
            if repeatable {
                meta = meta.repeatable();
            }
            app.commands_mut().register(CommandId::from(id), meta);
        }
        app.commands_mut().register(
            CommandId::from("focus.next"),
            CommandMeta::new("Focus Next")
                .with_description("Move focus to the next focusable control")
                .with_category("View"),
        );
        app.commands_mut().register(
            CommandId::from("focus.previous"),
            CommandMeta::new("Focus Previous")
                .with_description("Move focus to the previous focusable control")
                .with_category("View"),
        );

        let default_keymap = Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("focus.next".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("focus.previous".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyP".into(),
                    },
                },
                BindingV1 {
                    command: Some("demo.toggle_modal".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "F1".into(),
                    },
                },
                BindingV1 {
                    command: Some("demo.toggle_dnd_overlay".into()),
                    platform: Some("all".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "F2".into(),
                    },
                },
                BindingV1 {
                    command: Some("viewport.tool.select".into()),
                    platform: Some("all".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "KeyQ".into(),
                    },
                },
                BindingV1 {
                    command: Some("viewport.tool.move".into()),
                    platform: Some("all".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "KeyW".into(),
                    },
                },
                BindingV1 {
                    command: Some("viewport.tool.rotate".into()),
                    platform: Some("all".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "KeyE".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("macos".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("macos".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("windows".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("windows".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("linux".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("linux".into()),
                    when: Some("!focus.is_text_input && !ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.new".into()),
                    platform: Some("macos".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyN".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save".into()),
                    platform: Some("macos".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save_as".into()),
                    platform: Some("macos".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into(), "shift".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.new".into()),
                    platform: Some("windows".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyN".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save".into()),
                    platform: Some("windows".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save_as".into()),
                    platform: Some("windows".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.new".into()),
                    platform: Some("linux".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyN".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save".into()),
                    platform: Some("linux".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("scene.save_as".into()),
                    platform: Some("linux".into()),
                    when: Some("!ui.has_modal".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyS".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.clear".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyL".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["meta".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_left".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_right".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_home".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Home".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_end".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "End".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_up".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowUp".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_down".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "ArrowDown".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_left".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_right".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_up".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowUp".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_down".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "ArrowDown".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_home".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Home".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_end".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "End".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_backward".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_forward".into()),
                    platform: Some("all".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("macos".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["alt".into()],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("windows".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Delete".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_left".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.move_word_right".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_left".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowLeft".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_word_right".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "ArrowRight".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_backward".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Backspace".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.delete_word_forward".into()),
                    platform: Some("linux".into()),
                    when: Some("focus.is_text_input".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "Delete".into(),
                    },
                },
            ],
        })
        .expect("default keymap must parse");

        let mut merged = default_keymap;
        match Self::load_keymap_file() {
            Ok(user) => merged.extend(user),
            Err(e) => {
                tracing::info!(error = ?e, path = %Self::keymap_path().display(), "no user keymap loaded");
            }
        }
        app.set_global(KeymapService { keymap: merged });

        let theme = Theme::global(app).snapshot();

        let mut dock = DockManager::default();
        let key_scene = PanelKey::new("core.scene");
        dock.insert_panel(
            key_scene.clone(),
            DockPanel {
                title: "Scene".to_string(),
                color: theme.colors.panel_background,
                viewport: self
                    .viewport_targets
                    .as_ref()
                    .and_then(|t| t.get(&key_scene))
                    .map(|t| ViewportPanel {
                        target: t.target,
                        target_px_size: t.target_px_size,
                        fit: fret_core::ViewportFit::Contain,
                        context_menu_enabled: true,
                    }),
            },
        );
        let key_game = PanelKey::new("core.game");
        dock.insert_panel(
            key_game.clone(),
            DockPanel {
                title: "Game".to_string(),
                color: theme.colors.panel_background,
                viewport: self
                    .viewport_targets
                    .as_ref()
                    .and_then(|t| t.get(&key_game))
                    .map(|t| ViewportPanel {
                        target: t.target,
                        target_px_size: t.target_px_size,
                        fit: fret_core::ViewportFit::Contain,
                        context_menu_enabled: false,
                    }),
            },
        );
        let key_text_probe = PanelKey::new("core.text_probe");
        dock.insert_panel(
            key_text_probe.clone(),
            DockPanel {
                title: "Text Probe".to_string(),
                color: theme.colors.panel_background,
                viewport: None,
            },
        );
        let key_project = PanelKey::new("core.project");
        dock.insert_panel(
            key_project.clone(),
            DockPanel {
                title: "Project".to_string(),
                color: theme.colors.panel_background,
                viewport: None,
            },
        );
        let key_inspector = PanelKey::new("core.inspector");
        dock.insert_panel(
            key_inspector.clone(),
            DockPanel {
                title: "Inspector".to_string(),
                color: theme.colors.panel_background,
                viewport: None,
            },
        );
        let key_hierarchy = PanelKey::new("core.hierarchy");
        dock.insert_panel(
            key_hierarchy.clone(),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: theme.colors.panel_background,
                viewport: None,
            },
        );

        if let Some(layout) = Self::load_layout_file() {
            let missing_color = Color {
                a: 1.0,
                ..theme.colors.hover_background
            };
            Self::ensure_layout_panels(&mut dock, &layout, missing_color);
            self.loaded_layout = Some(layout.clone());
            self.bump_next_floating_index_from_layout(&layout);

            let mut imported_main_root = false;
            if let Some(main_entry) = layout
                .windows
                .iter()
                .find(|w| w.logical_window_id == "main")
            {
                if let Some(root) = dock
                    .graph
                    .import_subtree_from_layout_v1(&layout, main_entry.root)
                {
                    dock.graph.set_window_root(main_window, root);
                    imported_main_root = true;
                }
            }

            // If we have a layout file but fail to import a main root (e.g. version drift or
            // corrupted file), fall back to the default layout instead of leaving the main window
            // blank.
            let mut allow_restore_windows = imported_main_root;
            if !imported_main_root || dock.graph.window_root(main_window).is_none() {
                tracing::warn!(
                    "layout.json present but main root could not be restored; falling back to default layout"
                );
                allow_restore_windows = false;
                let tabs_left = dock.graph.insert_node(DockNode::Tabs {
                    tabs: vec![key_hierarchy.clone(), key_project.clone()],
                    active: 0,
                });
                let tabs_scene = dock.graph.insert_node(DockNode::Tabs {
                    tabs: vec![key_scene.clone(), key_game.clone()],
                    active: 0,
                });
                let tabs_inspector = dock.graph.insert_node(DockNode::Tabs {
                    tabs: vec![key_inspector.clone(), key_text_probe.clone()],
                    active: 0,
                });
                let right = dock.graph.insert_node(DockNode::Split {
                    axis: Axis::Vertical,
                    children: vec![tabs_scene, tabs_inspector],
                    fractions: vec![0.72, 0.28],
                });
                let root_dock = dock.graph.insert_node(DockNode::Split {
                    axis: Axis::Horizontal,
                    children: vec![tabs_left, right],
                    fractions: vec![0.26, 0.74],
                });
                dock.graph.set_window_root(main_window, root_dock);
            }

            let probe_present = dock
                .graph
                .collect_panels_in_window(main_window)
                .iter()
                .any(|p| p == &key_text_probe);
            if !probe_present {
                if let Some(tabs) = dock.graph.first_tabs_in_window(main_window) {
                    if let Some(DockNode::Tabs { tabs: list, .. }) = dock.graph.node_mut(tabs) {
                        if !list.contains(&key_text_probe) {
                            list.push(key_text_probe.clone());
                        }
                    }
                }
            }

            let game_present = dock
                .graph
                .collect_panels_in_window(main_window)
                .iter()
                .any(|p| p == &key_game);
            if !game_present {
                if let Some(tabs) = dock.graph.first_tabs_in_window(main_window) {
                    if let Some(DockNode::Tabs { tabs: list, .. }) = dock.graph.node_mut(tabs) {
                        if !list.contains(&key_game) {
                            list.push(key_game.clone());
                        }
                    }
                }
            }

            let project_present = dock
                .graph
                .collect_panels_in_window(main_window)
                .iter()
                .any(|p| p == &key_project);
            if !project_present {
                if let Some(tabs) = dock.graph.first_tabs_in_window(main_window) {
                    if let Some(DockNode::Tabs { tabs: list, .. }) = dock.graph.node_mut(tabs) {
                        if !list.contains(&key_project) {
                            list.push(key_project.clone());
                        }
                    }
                }
            }

            if allow_restore_windows {
                for w in &layout.windows {
                    if w.logical_window_id == "main" {
                        continue;
                    }
                    app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                        kind: CreateWindowKind::DockRestore {
                            logical_window_id: w.logical_window_id.clone(),
                        },
                        anchor: None,
                    })));
                }
            }
        } else {
            let tabs_left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_hierarchy, key_project],
                active: 0,
            });
            let tabs_scene = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_scene, key_game],
                active: 0,
            });
            let tabs_inspector = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![key_inspector, key_text_probe],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Vertical,
                children: vec![tabs_scene, tabs_inspector],
                fractions: vec![0.72, 0.28],
            });
            let root_dock = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![tabs_left, right],
                fractions: vec![0.26, 0.74],
            });
            dock.graph.set_window_root(main_window, root_dock);
        }

        app.set_global(dock);

        if app.global::<ProjectService>().is_none() {
            let mut project = ProjectService::default();
            if let Err(err) = project.ensure_demo_assets_exist() {
                tracing::error!(error = %err, "failed to create demo project assets");
            }
            if let Err(err) = project.rescan() {
                tracing::error!(error = %err, "failed to scan demo project assets");
            }
            app.set_global(project);
        }

        if app.global::<DemoPlayState>().is_none() {
            app.set_global(DemoPlayState::default());
        }

        if self.selection.is_none() {
            self.selection = Some(app.models_mut().insert(DemoSelection::default()));
        }
        if self.hierarchy.is_none() {
            self.hierarchy = Some(app.models_mut().insert(DemoHierarchy::default()));
        }
        if self.world.is_none() {
            self.world = Some(app.models_mut().insert(DemoWorld::default()));
        }
        if self.undo.is_none() {
            self.undo = Some(app.models_mut().insert(UndoStack::default()));
        }
        if self.viewport_tools.is_none() {
            self.viewport_tools = Some(app.models_mut().insert(ViewportToolManager::default()));
        }
    }

    fn create_window_state(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
    ) -> Self::WindowState {
        let fallback_kind = if self.main_window.is_some() && Some(window) != self.main_window {
            DemoUiKind::DockFloating
        } else {
            DemoUiKind::Main
        };
        let ui_kind = self
            .pending_window_ui_kinds
            .pop_front()
            .unwrap_or(fallback_kind);

        let selection = match self.selection {
            Some(model) => model,
            None => {
                let model = app.models_mut().insert(DemoSelection::default());
                self.selection = Some(model);
                model
            }
        };
        let hierarchy = match self.hierarchy {
            Some(model) => model,
            None => {
                let model = app.models_mut().insert(DemoHierarchy::default());
                self.hierarchy = Some(model);
                model
            }
        };
        let undo = match self.undo {
            Some(model) => model,
            None => {
                let model = app.models_mut().insert(UndoStack::default());
                self.undo = Some(model);
                model
            }
        };
        let world = match self.world {
            Some(model) => model,
            None => {
                let model = app.models_mut().insert(DemoWorld::default());
                self.world = Some(model);
                model
            }
        };
        if self.undo.is_none() {
            self.undo = Some(app.models_mut().insert(UndoStack::default()));
        }
        if self.viewport_tools.is_none() {
            self.viewport_tools = Some(app.models_mut().insert(ViewportToolManager::default()));
        }
        let inspector_edit_buffer = app.models_mut().insert(String::new());
        let viewport_tools = self
            .viewport_tools
            .unwrap_or_else(|| app.models_mut().insert(ViewportToolManager::default()));
        self.viewport_tools = Some(viewport_tools);
        let (mut ui, layers) = build_demo_ui(
            window,
            ui_kind,
            DemoUiConfig::default(),
            inspector_edit_buffer,
            viewport_tools,
        );

        let key_hierarchy = PanelKey::new("core.hierarchy");
        let key_project = PanelKey::new("core.project");
        let key_inspector = PanelKey::new("core.inspector");
        let key_scene = PanelKey::new("core.scene");
        let key_game = PanelKey::new("core.game");
        let key_text_probe = PanelKey::new("core.text_probe");

        let hierarchy_node = ui.create_node(HierarchyPanel::new(selection, hierarchy, undo));
        let project_node = ui.create_node(ProjectPanel::new());
        let inspector_node = ui.create_node(InspectorPanel::new(selection, world));
        let scene_drop_node = ui.create_node(viewport_asset_drop::ViewportAssetDropPanel::new(
            key_scene.clone(),
        ));
        let game_drop_node = ui.create_node(viewport_asset_drop::ViewportAssetDropPanel::new(
            key_game.clone(),
        ));
        let text_probe_node = ui.create_node(TextProbePanel::new(TEXT_PROBE_DEFAULT));
        ui.add_child(layers.dockspace_node, hierarchy_node);
        ui.add_child(layers.dockspace_node, project_node);
        ui.add_child(layers.dockspace_node, inspector_node);
        ui.add_child(layers.dockspace_node, scene_drop_node);
        ui.add_child(layers.dockspace_node, game_drop_node);
        ui.add_child(layers.dockspace_node, text_probe_node);

        app.with_global_mut(DockPanelContentService::default, |s, _app| {
            s.set(window, key_hierarchy, hierarchy_node);
            s.set(window, key_project, project_node);
            s.set(window, key_inspector, inspector_node);
            s.set(window, key_scene, scene_drop_node);
            s.set(window, key_game, game_drop_node);
            s.set(window, key_text_probe, text_probe_node);
        });
        Self::WindowState {
            ui,
            layers,
            palette_previous_focus: None,
            context_menu_previous_focus: None,
            inspector_edit_previous_focus: None,
            unsaved_dialog_previous_focus: None,
            inspector_edit_buffer,
            last_cursor_pos: None,
        }
    }

    fn invalidate_ui_layout(
        &mut self,
        _app: &mut App,
        _window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
    ) {
        state
            .ui
            .invalidate(state.layers.dockspace_node, Invalidation::Layout);
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        _window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        text: &mut dyn fret_core::TextService,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        event: &fret_core::Event,
    ) {
        if let fret_core::Event::Pointer(pe) = event {
            state.last_cursor_pos = Some(match pe {
                fret_core::PointerEvent::Down { position, .. }
                | fret_core::PointerEvent::Up { position, .. }
                | fret_core::PointerEvent::Move { position, .. }
                | fret_core::PointerEvent::Wheel { position, .. } => *position,
            });
        }

        if matches!(event, fret_core::Event::WindowCloseRequested) {
            self.handle_window_close_requested(app, window, state);
            return;
        }

        if let fret_core::Event::ExternalDrag(drag) = event {
            tracing::info!(window = ?window, ?drag, "external drag event received");
            match &drag.kind {
                fret_core::ExternalDragKind::EnterFiles(_)
                | fret_core::ExternalDragKind::OverFiles(_) => {
                    if !state.ui.is_layer_visible(state.layers.external_dnd) {
                        state.ui.set_layer_visible(state.layers.external_dnd, true);
                        app.request_redraw(window);
                    }
                }
                fret_core::ExternalDragKind::DropFiles(files) => {
                    if state.ui.is_layer_visible(state.layers.external_dnd) {
                        state.ui.set_layer_visible(state.layers.external_dnd, false);
                        app.request_redraw(window);
                    }
                    app.push_effect(Effect::ExternalDropReadAll {
                        window,
                        token: files.token,
                    });
                }
                fret_core::ExternalDragKind::Leave => {
                    if state.ui.is_layer_visible(state.layers.external_dnd) {
                        state.ui.set_layer_visible(state.layers.external_dnd, false);
                        app.request_redraw(window);
                    }
                }
            }
        }

        if let fret_core::Event::ExternalDropData(payload) = event {
            for err in &payload.errors {
                tracing::warn!(name = %err.name, message = %err.message, "external drop read error");
            }

            let mut imported: Vec<fret_editor::AssetGuid> = Vec::new();
            if let Some(project) = app.global_mut::<ProjectService>() {
                let items: Vec<(String, Vec<u8>)> = payload
                    .files
                    .iter()
                    .map(|f| (f.name.clone(), f.bytes.clone()))
                    .collect();
                match project.import_files_from_bytes(items) {
                    Ok(guids) => imported = guids,
                    Err(err) => {
                        tracing::error!(error = %err, "project import failed");
                    }
                }
                if let Err(err) = project.rescan() {
                    tracing::error!(error = %err, "failed to rescan project after import");
                }
            }

            app.push_effect(Effect::ExternalDropRelease {
                token: payload.token,
            });

            if let Some(last) = imported.last().copied() {
                app.with_global_mut(ProjectSelectionService::default, |s, _app| {
                    s.set_selected_guid(Some(last));
                });
            }

            for &w in self.logical_windows.keys() {
                app.request_redraw(w);
            }
        }

        match event {
            fret_core::Event::Timer { token } => {
                if Some(*token) == self.dock_persist_timer && self.dock_persist_pending {
                    self.dock_persist_pending = false;
                    self.persist_layout_now(app);
                }
                if Some(*token) == self.camera_persist_timer && self.camera_persist_pending {
                    self.camera_persist_pending = false;
                    self.persist_viewport_cameras_now();
                }
                if Some(*token) == self.theme_reload_timer {
                    self.reload_theme_if_changed(app);
                }
            }
            fret_core::Event::WindowResized { width, height } => {
                let entry = self.window_placements.entry(window).or_insert(
                    fret_core::DockWindowPlacementV1 {
                        width: 640,
                        height: 480,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                );
                entry.width = width.0.max(1.0).round() as u32;
                entry.height = height.0.max(1.0).round() as u32;
                self.schedule_layout_persist(app);
            }
            fret_core::Event::WindowMoved { x, y } => {
                let entry = self.window_placements.entry(window).or_insert(
                    fret_core::DockWindowPlacementV1 {
                        width: 640,
                        height: 480,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                );
                entry.x = Some(*x);
                entry.y = Some(*y);
                self.schedule_layout_persist(app);
            }
            _ => {}
        }

        if let fret_core::Event::Pointer(pe) = event {
            if let fret_core::PointerEvent::Down { .. } = pe {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    // Command palette uses its own backdrop to dismiss; avoid demo-only right-click modal.
                    state.ui.dispatch_event(app, text, event);
                    return;
                }
                if state.ui.is_layer_visible(state.layers.modal) {
                    state.ui.set_layer_visible(state.layers.modal, false);
                    app.request_redraw(window);
                    return;
                }
            }
        }

        if let fret_core::Event::KeyDown { key, modifiers, .. } = event {
            if *key == fret_core::KeyCode::Escape
                && !modifiers.ctrl
                && !modifiers.alt
                && !modifiers.meta
            {
                if let Some(tool) = self.viewport_tools {
                    enum CancelViewportInteraction {
                        OverlayOnly(fret_core::AppWindowId, RenderTargetId),
                        TranslateGizmo(
                            fret_core::AppWindowId,
                            RenderTargetId,
                            Vec<(u64, [f32; 3])>,
                        ),
                        RotateGizmo(fret_core::AppWindowId, RenderTargetId, Vec<(u64, f32)>),
                    }

                    let mut cancel: Option<CancelViewportInteraction> = None;
                    let _ = tool.update(app, |t, _cx| {
                        cancel = match t.interaction.take() {
                            Some(ViewportInteraction::MarqueeSelect(m)) => {
                                Some(CancelViewportInteraction::OverlayOnly(m.window, m.target))
                            }
                            Some(ViewportInteraction::PanOrbit(m)) => {
                                Some(CancelViewportInteraction::OverlayOnly(m.window, m.target))
                            }
                            Some(ViewportInteraction::TranslateGizmo(m)) => {
                                Some(CancelViewportInteraction::TranslateGizmo(
                                    m.window,
                                    m.target,
                                    m.start_positions,
                                ))
                            }
                            Some(ViewportInteraction::RotateGizmo(m)) => {
                                Some(CancelViewportInteraction::RotateGizmo(
                                    m.window,
                                    m.target,
                                    m.start_rotations,
                                ))
                            }
                            other => {
                                t.interaction = other;
                                None
                            }
                        };
                    });

                    let Some(cancel) = cancel else {
                        return;
                    };

                    if let Some(undo) = self.undo {
                        let _ = undo.update(app, |s, _cx| s.cancel_active());
                    }

                    let (w, target, rollback_positions, rollback_rotations) = match cancel {
                        CancelViewportInteraction::OverlayOnly(w, target) => {
                            (w, target, None, None)
                        }
                        CancelViewportInteraction::TranslateGizmo(w, target, start_positions) => {
                            (w, target, Some(start_positions), None)
                        }
                        CancelViewportInteraction::RotateGizmo(w, target, start_rotations) => {
                            (w, target, None, Some(start_rotations))
                        }
                    };

                    if let Some(start_positions) = rollback_positions {
                        if let Some(world) = self.world {
                            let _ = world.update(app, |w, _cx| {
                                for (id, start) in &start_positions {
                                    let e = w.entity_mut(*id);
                                    e.transform.position = *start;
                                }
                            });
                        }
                    }
                    if let Some(start_rotations) = rollback_rotations {
                        if let Some(world) = self.world {
                            let _ = world.update(app, |w, _cx| {
                                for (id, start) in &start_rotations {
                                    w.entity_mut(*id).transform.rotation_y = *start;
                                }
                            });
                        }
                    }

                    if let Some(dock) = app.global_mut::<DockManager>() {
                        dock.set_viewport_marquee(w, target, None);
                        dock.set_viewport_drag_line(w, target, None);
                    }
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                    return;
                }
            }
        }
        state.ui.dispatch_event(app, text, event);

        if self.handle_asset_drop_requests(app) {
            for &w in self.logical_windows.keys() {
                app.request_redraw(w);
            }
        }
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        text: &mut dyn fret_core::TextService,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, text, &command) {
            return;
        }

        match command.as_str() {
            "debug.hud.toggle" => {
                let enabled =
                    app.with_global_mut(DebugHudService::default, |hud, _app| hud.toggle());
                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
                tracing::info!(enabled, "toggled ui debug hud");
            }
            "debug.inspector.toggle" => {
                let enabled =
                    app.with_global_mut(DebugInspectorService::default, |s, _app| s.toggle());
                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
                tracing::info!(enabled, "toggled ui inspector");
            }
            "command_palette.toggle" => {
                let vis = state.ui.is_layer_visible(state.layers.command_palette);
                if vis {
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, false);
                    if let Some(prev) = state.palette_previous_focus.take() {
                        state.ui.set_focus(Some(prev));
                    }
                } else {
                    state.palette_previous_focus = state.ui.focus();
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, true);
                    state.ui.set_focus(Some(state.layers.command_palette_node));
                }
                app.request_redraw(window);
            }
            "demo.play.toggle" => {
                self.play_mode = !self.play_mode;
                self.play_started_at = self.play_mode.then(Instant::now);
                app.with_global_mut(DemoPlayState::default, |s, _app| {
                    s.playing = self.play_mode;
                });
                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
            }
            "theme.reload" => {
                load_theme(app);
                for &w in self.logical_windows.keys() {
                    app.push_effect(Effect::UiInvalidateLayout { window: w });
                    app.request_redraw(w);
                }
            }
            "theme.reset_override" => {
                self.reset_theme_override();
                load_theme(app);
                for &w in self.logical_windows.keys() {
                    app.push_effect(Effect::UiInvalidateLayout { window: w });
                    app.request_redraw(w);
                }
            }
            "theme.set.fret_default_dark"
            | "theme.set.hardhacker_dark"
            | "theme.set.godot_default_dark" => {
                let id = command
                    .as_str()
                    .strip_prefix("theme.set.")
                    .unwrap_or_default();
                let Some(src_path) = theme_builtin_path(id) else {
                    return;
                };

                let Ok(bytes) = std::fs::read(src_path) else {
                    tracing::error!(path = %src_path, "failed to read built-in theme file");
                    return;
                };

                // Persist as the active theme override so future launches keep the selection.
                let _ = self.write_theme_override(&bytes);

                let _ = self.apply_theme_bytes(app, src_path, &bytes);
            }
            "dock.tab.close" | "dock.tab.float" | "dock.tab.move_left" | "dock.tab.move_right" => {
                let allow_tear_off = app
                    .global::<PlatformCapabilities>()
                    .cloned()
                    .unwrap_or_default()
                    .ui
                    .window_tear_off;

                let Some(dock) = app.global_mut::<DockManager>() else {
                    return;
                };
                let Some((dock_window, tabs_node, panel, position)) =
                    dock.take_dock_tab_context_menu()
                else {
                    return;
                };

                let op = match command.as_str() {
                    "dock.tab.close" => DockOp::ClosePanel {
                        window: dock_window,
                        panel,
                    },
                    "dock.tab.float" => {
                        if !allow_tear_off {
                            return;
                        }
                        DockOp::RequestFloatPanelToNewWindow {
                            source_window: dock_window,
                            panel,
                            anchor: Some(WindowAnchor {
                                window: dock_window,
                                position,
                            }),
                        }
                    }
                    "dock.tab.move_left" | "dock.tab.move_right" => {
                        let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(tabs_node) else {
                            return;
                        };
                        let Some(index) = tabs.iter().position(|p| p == &panel) else {
                            return;
                        };

                        if command.as_str() == "dock.tab.move_left" {
                            if index == 0 {
                                return;
                            }
                            DockOp::MovePanel {
                                source_window: dock_window,
                                panel,
                                target_window: dock_window,
                                target_tabs: tabs_node,
                                zone: DropZone::Center,
                                insert_index: Some(index - 1),
                            }
                        } else {
                            if index + 1 >= tabs.len() {
                                return;
                            }
                            DockOp::MovePanel {
                                source_window: dock_window,
                                panel,
                                target_window: dock_window,
                                target_tabs: tabs_node,
                                zone: DropZone::Center,
                                insert_index: Some(index + 2),
                            }
                        }
                    }
                    _ => return,
                };

                self.dock_op(app, op);
            }
            "dock.layout.reset_default" => {
                if self.reset_layout_to_default(app) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "dock.layout.preset.save_last" => {
                if self.save_last_layout_preset(app) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "dock.layout.preset.load_last" => {
                if self.load_last_layout_preset(app) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "project.refresh" => {
                if let Some(project) = app.global_mut::<ProjectService>() {
                    if let Err(err) = project.ensure_demo_assets_exist() {
                        tracing::error!(error = %err, "failed to create demo project assets");
                    }
                    if let Err(err) = project.rescan() {
                        tracing::error!(error = %err, "failed to scan demo project assets");
                    }
                }
                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
            }
            "project.rename_selected" => {
                let guid = app
                    .global::<ProjectSelectionService>()
                    .and_then(|s| s.selected_guid());
                let Some(guid) = guid else {
                    return;
                };

                if let Some(project) = app.global_mut::<ProjectService>() {
                    let Some(id) = project.id_for_guid(guid) else {
                        return;
                    };
                    let Some(path) = project.path_for_id(id).map(|p| p.to_path_buf()) else {
                        return;
                    };
                    if path == project.assets_root() {
                        return;
                    }

                    let Some(new_name) = next_project_rename_name(&path) else {
                        tracing::error!(path = %path.to_string_lossy(), "failed to pick rename candidate");
                        return;
                    };

                    if let Err(err) = project.rename_entry(id, &new_name) {
                        tracing::error!(
                            error = %err,
                            from = %path.to_string_lossy(),
                            to = %new_name,
                            "project rename failed"
                        );
                        return;
                    }
                    if let Err(err) = project.rescan() {
                        tracing::error!(error = %err, "failed to rescan project after rename");
                        return;
                    }
                }

                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
            }
            "project.move_selected_to_moved" => {
                let guid = app
                    .global::<ProjectSelectionService>()
                    .and_then(|s| s.selected_guid());
                let Some(guid) = guid else {
                    return;
                };

                if let Some(project) = app.global_mut::<ProjectService>() {
                    let Some(id) = project.id_for_guid(guid) else {
                        return;
                    };
                    let Some(path) = project.path_for_id(id).map(|p| p.to_path_buf()) else {
                        return;
                    };
                    if path == project.assets_root() {
                        return;
                    }

                    if let Err(err) = project.move_entry_to_folder(id, "Moved") {
                        tracing::error!(
                            error = %err,
                            from = %path.to_string_lossy(),
                            "project move failed"
                        );
                        return;
                    }
                    if let Err(err) = project.rescan() {
                        tracing::error!(error = %err, "failed to rescan project after move");
                        return;
                    }
                }

                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
            }
            "inspector_edit.open" => {
                let Some(request) = app
                    .global::<InspectorEditService>()
                    .and_then(|s| s.get(window))
                    .cloned()
                else {
                    return;
                };

                let _ = state.inspector_edit_buffer.update(app, |buf, _cx| {
                    *buf = request.initial_text;
                });

                state.inspector_edit_previous_focus = state.ui.focus();
                state
                    .ui
                    .set_layer_visible(state.layers.inspector_edit, true);
                state
                    .ui
                    .set_focus(Some(state.layers.inspector_edit_input_node));
                app.request_redraw(window);
            }
            "inspector_edit.close" => {
                if state.ui.is_layer_visible(state.layers.inspector_edit) {
                    state
                        .ui
                        .set_layer_visible(state.layers.inspector_edit, false);
                }

                app.global_mut::<InspectorEditService>()
                    .map(|s| s.clear(window));

                if let Some(prev) = state.inspector_edit_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }
                app.request_redraw(window);
            }
            "inspector_edit.commit" => {
                let Some(request) = app
                    .global::<InspectorEditService>()
                    .and_then(|s| s.get(window))
                    .cloned()
                else {
                    return;
                };

                let input = state
                    .inspector_edit_buffer
                    .get(app)
                    .cloned()
                    .unwrap_or_default();

                let Some(value) = parse_value(request.kind, input.as_str()) else {
                    app.with_global_mut(InspectorEditService::default, |s, _app| {
                        let msg = match request.kind {
                            InspectorEditKind::String => "Invalid value",
                            InspectorEditKind::F32 => "Invalid number",
                            InspectorEditKind::Vec3 => "Invalid vec3 (expected: x, y, z)",
                        };
                        s.set_error(window, msg);
                    });
                    app.request_redraw(window);
                    return;
                };
                app.with_global_mut(PropertyEditService::default, |s, _app| {
                    s.set(
                        window,
                        PropertyEditRequest {
                            targets: request.targets,
                            path: request.path,
                            value,
                            kind: PropertyEditKind::Commit,
                        },
                    );
                });
                app.push_effect(Effect::Command {
                    window: Some(window),
                    command: CommandId::from("property_edit.commit"),
                });

                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }

                state
                    .ui
                    .set_layer_visible(state.layers.inspector_edit, false);
                app.global_mut::<InspectorEditService>()
                    .map(|s| s.clear(window));
                if let Some(prev) = state.inspector_edit_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }
                app.request_redraw(window);
            }
            "property_edit.commit" => {
                let Some(request) = app
                    .global_mut::<PropertyEditService>()
                    .and_then(|s| s.take(window))
                else {
                    return;
                };

                let current_before: Vec<Option<PropertyValue>> = self
                    .world
                    .and_then(|world| world.get(app))
                    .map(|w| {
                        request
                            .targets
                            .iter()
                            .map(|&id| w.get_property(id, &request.path))
                            .collect()
                    })
                    .unwrap_or_default();

                match request.kind {
                    PropertyEditKind::Begin => {
                        let was_dirty = Self::is_scene_dirty(app);
                        self.active_property_edits.insert(
                            window,
                            ActivePropertyEdit {
                                targets: request.targets.clone(),
                                path: request.path.clone(),
                                before: current_before,
                                was_dirty,
                            },
                        );

                        if let Some(world) = self.world {
                            let after = request.value.clone();
                            let _ = world.update(app, |w, _cx| {
                                w.apply_property_value(&request.targets, &request.path, after);
                            });
                        }
                        self.mark_scene_dirty(app);

                        for &w in self.logical_windows.keys() {
                            app.request_redraw(w);
                        }
                    }
                    PropertyEditKind::Update => {
                        if self.active_property_edits.get(&window).is_none() {
                            let was_dirty = Self::is_scene_dirty(app);
                            self.active_property_edits.insert(
                                window,
                                ActivePropertyEdit {
                                    targets: request.targets.clone(),
                                    path: request.path.clone(),
                                    before: current_before,
                                    was_dirty,
                                },
                            );
                        }

                        if let Some(world) = self.world {
                            let after = request.value.clone();
                            let _ = world.update(app, |w, _cx| {
                                w.apply_property_value(&request.targets, &request.path, after);
                            });
                        }
                        self.mark_scene_dirty(app);

                        for &w in self.logical_windows.keys() {
                            app.request_redraw(w);
                        }
                    }
                    PropertyEditKind::Commit => {
                        let active = self.active_property_edits.remove(&window);
                        let before = active
                            .as_ref()
                            .filter(|a| a.targets == request.targets && a.path == request.path)
                            .map(|a| a.before.clone())
                            .unwrap_or(current_before);
                        let after = request.value.clone();

                        if let Some(world) = self.world {
                            let _ = world.update(app, |w, _cx| {
                                w.apply_property_value(
                                    &request.targets,
                                    &request.path,
                                    after.clone(),
                                );
                            });
                        }

                        if let Some(stack) = self.undo {
                            let cmd = EditCommand::SetProperties {
                                targets: request.targets,
                                path: request.path,
                                before,
                                after,
                            };
                            let _ = stack.update(app, |s, _cx| s.push(cmd));
                        }
                        self.mark_scene_dirty(app);

                        for &w in self.logical_windows.keys() {
                            app.request_redraw(w);
                        }
                    }
                    PropertyEditKind::Cancel => {
                        let active = self.active_property_edits.remove(&window);
                        let Some(active) = active else {
                            return;
                        };

                        if let Some(world) = self.world {
                            let _ = world.update(app, |w, _cx| {
                                for (id, before) in
                                    active.targets.iter().copied().zip(active.before.iter())
                                {
                                    let Some(value) = before.clone() else {
                                        continue;
                                    };
                                    let _ = w.set_property(id, &active.path, value);
                                }
                            });
                        }

                        app.with_global_mut(SceneDocumentService::default, |s, _app| {
                            s.set_dirty(active.was_dirty);
                        });

                        for &w in self.logical_windows.keys() {
                            app.request_redraw(w);
                        }
                    }
                }
            }
            "edit.undo" => {
                let mut cmd: Option<EditCommand> = None;
                if let Some(stack) = self.undo {
                    let _ = stack.update(app, |s, _cx| {
                        cmd = s.pop_undo();
                    });
                }
                if let Some(cmd) = cmd {
                    match &cmd {
                        EditCommand::HierarchyMove { .. } => {
                            if let (Some(hierarchy), Some(selection)) =
                                (self.hierarchy, self.selection)
                            {
                                cmd.undo_in_app(app, hierarchy, selection);
                            }
                        }
                        _ => {
                            if let Some(world) = self.world {
                                let _ = world.update(app, |w, _cx| {
                                    cmd.undo(w);
                                });
                            }
                        }
                    }
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "edit.redo" => {
                let mut cmd: Option<EditCommand> = None;
                if let Some(stack) = self.undo {
                    let _ = stack.update(app, |s, _cx| {
                        cmd = s.pop_redo();
                    });
                }
                if let Some(cmd) = cmd {
                    match &cmd {
                        EditCommand::HierarchyMove { .. } => {
                            if let (Some(hierarchy), Some(selection)) =
                                (self.hierarchy, self.selection)
                            {
                                cmd.apply_in_app(app, hierarchy, selection);
                            }
                        }
                        _ => {
                            if let Some(world) = self.world {
                                let _ = world.update(app, |w, _cx| {
                                    cmd.apply(w);
                                });
                            }
                        }
                    }
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "viewport.tool.select" => {
                if let Some(tool) = self.viewport_tools {
                    let _ = tool.update(app, |t, _cx| {
                        t.active = ViewportToolMode::Select;
                        t.interaction = None;
                    });
                }
                app.request_redraw(window);
            }
            "viewport.tool.move" => {
                if let Some(tool) = self.viewport_tools {
                    let _ = tool.update(app, |t, _cx| {
                        t.active = ViewportToolMode::Move;
                        t.interaction = None;
                    });
                }
                app.request_redraw(window);
            }
            "viewport.tool.rotate" => {
                if let Some(tool) = self.viewport_tools {
                    let _ = tool.update(app, |t, _cx| {
                        t.active = ViewportToolMode::Rotate;
                        t.interaction = None;
                    });
                }
                app.request_redraw(window);
            }
            "command_palette.close" => {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    state
                        .ui
                        .set_layer_visible(state.layers.command_palette, false);
                    if let Some(prev) = state.palette_previous_focus.take() {
                        state.ui.set_focus(Some(prev));
                    }
                    app.request_redraw(window);
                }
            }
            "context_menu.open" => {
                if state.ui.is_layer_visible(state.layers.command_palette) {
                    return;
                }

                let has_request = app
                    .global::<ContextMenuService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return;
                }

                state.context_menu_previous_focus = state.ui.focus();
                state.ui.set_layer_visible(state.layers.context_menu, true);
                state.ui.set_focus(Some(state.layers.context_menu_node));
                app.request_redraw(window);
            }
            "context_menu.close" => {
                if state.ui.is_layer_visible(state.layers.context_menu) {
                    state.ui.set_layer_visible(state.layers.context_menu, false);
                }

                app.with_global_mut(ContextMenuService::default, |service, app| {
                    let action = service.take_pending_action(window);
                    service.clear(window);
                    if let Some(command) = action {
                        app.push_effect(Effect::Command {
                            window: Some(window),
                            command,
                        });
                    }
                });

                if let Some(prev) = state.context_menu_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
            }
            "demo.toggle_modal" => {
                let vis = state.ui.is_layer_visible(state.layers.modal);
                state.ui.set_layer_visible(state.layers.modal, !vis);
                app.request_redraw(window);
            }
            "demo.toggle_dnd_overlay" => {
                let vis = state.ui.is_layer_visible(state.layers.external_dnd);
                state.ui.set_layer_visible(state.layers.external_dnd, !vis);
                app.request_redraw(window);
            }
            "unsaved_dialog.open" => {
                let pending_for_window = app
                    .global::<UnsavedChangesService>()
                    .and_then(|s| s.pending())
                    .is_some_and(|(w, _)| w == window);
                if !pending_for_window {
                    return;
                }
                if !state.ui.is_layer_visible(state.layers.modal) {
                    state.unsaved_dialog_previous_focus = state.ui.focus();
                    state.ui.set_layer_visible(state.layers.modal, true);
                    state.ui.set_focus(None);
                    app.request_redraw(window);
                }
            }
            "unsaved_dialog.cancel" => {
                if state.ui.is_layer_visible(state.layers.modal) {
                    state.ui.set_layer_visible(state.layers.modal, false);
                }
                app.with_global_mut(UnsavedChangesService::default, |s, _app| {
                    s.clear();
                });
                if let Some(prev) = state.unsaved_dialog_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }
                app.request_redraw(window);
            }
            "unsaved_dialog.save" | "unsaved_dialog.discard" => {
                let pending = app
                    .global::<UnsavedChangesService>()
                    .and_then(|s| s.pending());
                let Some((pending_window, action)) = pending else {
                    return;
                };
                if pending_window != window {
                    return;
                }

                let continue_after = if command.as_str() == "unsaved_dialog.save" {
                    self.save_current_scene(app)
                } else {
                    app.with_global_mut(SceneDocumentService::default, |s, _app| {
                        s.set_dirty(false);
                    });
                    true
                };
                if !continue_after {
                    return;
                }

                if state.ui.is_layer_visible(state.layers.modal) {
                    state.ui.set_layer_visible(state.layers.modal, false);
                }
                app.with_global_mut(UnsavedChangesService::default, |s, _app| {
                    s.clear();
                });
                if let Some(prev) = state.unsaved_dialog_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }

                match action {
                    UnsavedContinuation::NewScene => {
                        let ok = self.create_and_open_new_scene(app);
                        if ok {
                            let key = PanelKey::new("core.scene");
                            let preferred = [Some(window), self.main_window].into_iter().flatten();
                            DockManager::request_activate_panel(
                                app,
                                window,
                                preferred,
                                key.clone(),
                            );
                        }
                    }
                    UnsavedContinuation::OpenScene { guid } => {
                        let _ = self.open_scene_by_guid(app, guid);
                        let key = PanelKey::new("core.scene");
                        let preferred = [Some(window), self.main_window].into_iter().flatten();
                        DockManager::request_activate_panel(app, window, preferred, key.clone());
                    }
                    UnsavedContinuation::CloseWindow { window: w } => {
                        app.push_effect(Effect::Window(WindowRequest::Close(w)));
                    }
                }

                for &w in self.logical_windows.keys() {
                    app.request_redraw(w);
                }
            }
            "asset.open_selected" => {
                let guid = app
                    .global::<ProjectSelectionService>()
                    .and_then(|s| s.selected_guid());
                let Some(guid) = guid else {
                    return;
                };
                if self.open_asset_by_guid(app, window, guid) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "scene.open_selected" => {
                let guid = app
                    .global::<ProjectSelectionService>()
                    .and_then(|s| s.selected_guid());
                let Some(guid) = guid else {
                    return;
                };
                if self.open_asset_by_guid(app, window, guid) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "scene.new" => {
                if self.new_scene_or_prompt(app, window) {
                    let key = PanelKey::new("core.scene");
                    let preferred = [Some(window), self.main_window].into_iter().flatten();
                    DockManager::request_activate_panel(app, window, preferred, key.clone());

                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "scene.save" => {
                if self.save_current_scene(app) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            "scene.save_as" => {
                if self.save_current_scene_as(app) {
                    for &w in self.logical_windows.keys() {
                        app.request_redraw(w);
                    }
                }
            }
            _ => {}
        }
    }

    fn dock_op(&mut self, app: &mut App, op: DockOp) {
        match &op {
            DockOp::RequestFloatPanelToNewWindow {
                source_window,
                panel,
                anchor,
            } => {
                let caps = app
                    .global::<PlatformCapabilities>()
                    .cloned()
                    .unwrap_or_default();
                if !caps.ui.window_tear_off {
                    return;
                }
                app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                    kind: CreateWindowKind::DockFloating {
                        source_window: *source_window,
                        panel: panel.clone(),
                    },
                    anchor: *anchor,
                })));
                return;
            }
            DockOp::FloatPanelToWindow { .. } => {
                let caps = app
                    .global::<PlatformCapabilities>()
                    .cloned()
                    .unwrap_or_default();
                if !caps.ui.window_tear_off {
                    return;
                }
            }
            _ => {}
        }

        let mut close_if_empty: Option<fret_core::AppWindowId> = None;
        let mut redraw: Vec<fret_core::AppWindowId> = Vec::new();

        {
            let Some(dock) = app.global_mut::<DockManager>() else {
                return;
            };

            let _ = dock.graph.apply_op(&op);

            if let DockOp::MovePanel { source_window, .. } = &op {
                if dock
                    .graph
                    .collect_panels_in_window(*source_window)
                    .is_empty()
                    && Some(*source_window) != self.main_window
                {
                    close_if_empty = Some(*source_window);
                }
            }
            if let DockOp::FloatPanelToWindow { source_window, .. } = &op {
                if dock
                    .graph
                    .collect_panels_in_window(*source_window)
                    .is_empty()
                    && Some(*source_window) != self.main_window
                {
                    close_if_empty = Some(*source_window);
                }
            }
            if let DockOp::ClosePanel { window, .. } = &op {
                if dock.graph.collect_panels_in_window(*window).is_empty()
                    && Some(*window) != self.main_window
                {
                    close_if_empty = Some(*window);
                }
            }

            for (&w, _) in &self.logical_windows {
                if dock.graph.window_root(w).is_some() {
                    redraw.push(w);
                }
            }
        }

        if let Some(window) = close_if_empty {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
        for w in redraw {
            app.push_effect(Effect::UiInvalidateLayout { window: w });
        }
        self.schedule_layout_persist(app);
    }

    fn viewport_input(&mut self, app: &mut App, event: fret_core::ViewportInputEvent) {
        if let Some(tool) = self.viewport_tools {
            let window = event.window;
            let target = event.target;
            let selection_model = self.selection;
            let world_model = self.world;
            let undo_model = self.undo;

            let mut pending_selection: Option<(Option<u64>, Vec<u64>, fret_core::Modifiers)> = None;

            let mut panel_key: Option<PanelKey> = None;
            let mut target_px_size: Option<(u32, u32)> = None;
            if let Some(dock) = app.global::<DockManager>() {
                for pk in dock.graph.collect_panels_in_window(window) {
                    let Some(panel) = dock.panels.get(&pk) else {
                        continue;
                    };
                    let Some(vp) = panel.viewport else {
                        continue;
                    };
                    if vp.target == target {
                        panel_key = Some(pk);
                        target_px_size = Some(vp.target_px_size);
                        break;
                    }
                }
            }

            let allow_edit = panel_key
                .as_ref()
                .map(|p| demo_viewport_role(p) == DemoViewportRole::Scene)
                .unwrap_or(true);

            let mut marquee_update: Option<Option<ViewportMarquee>> = None;
            let mut drag_line_update: Option<Option<fret_ui::dock::ViewportDragLine>> = None;
            let mut request_redraw = false;
            let mut request_animation_frame = false;

            let handled = match event.kind {
                fret_core::ViewportInputKind::PointerDown { button, modifiers } => match button {
                    fret_core::MouseButton::Left => 'handled: {
                        if modifiers.alt {
                            let start_uv = event.uv;
                            let mut started = false;
                            let _ = tool.update(app, |t, _cx| {
                                if t.interaction.is_some() {
                                    return;
                                }
                                t.interaction =
                                    Some(ViewportInteraction::PanOrbit(PanOrbitInteraction {
                                        window,
                                        target,
                                        kind: PanOrbitKind::Orbit,
                                        button: fret_core::MouseButton::Left,
                                        start_modifiers: modifiers,
                                        start_uv,
                                        last_uv: start_uv,
                                        current_uv: start_uv,
                                        start_target_px: event.target_px,
                                        last_target_px: event.target_px,
                                        current_target_px: event.target_px,
                                        dragging: false,
                                    }));
                                started = true;
                            });
                            break 'handled started;
                        }

                        if !allow_edit {
                            break 'handled false;
                        }

                        let start_uv = event.uv;

                        let selection = selection_model
                            .and_then(|m| m.get(app))
                            .cloned()
                            .unwrap_or_default();
                        let lead = selection.lead_entity;
                        let selected = selection.selected_entities;

                        let camera = panel_key
                            .as_ref()
                            .map(|p| self.viewport_camera(p))
                            .unwrap_or_default();
                        let center_uv = lead
                            .and_then(|id| {
                                let world = world_model.and_then(|m| m.get(app))?;
                                Some(camera.world_to_uv(world.position(id)))
                            })
                            .or_else(|| lead.and_then(viewport_grid_marker_uv));

                        let translate_hit = center_uv.and_then(|center_uv| {
                            if let Some(size) = target_px_size {
                                viewport_gizmo_hit_test_px(
                                    center_uv,
                                    size,
                                    event.target_px,
                                    80.0,
                                    6.0,
                                    14.0,
                                )
                            } else {
                                let (u, v) = center_uv;
                                let du = (start_uv.0 - u).abs();
                                let dv = (start_uv.1 - v).abs();
                                (du <= 0.02 && dv <= 0.02).then_some(TranslateAxisConstraint::Free)
                            }
                        });
                        let rotate_hit = center_uv.is_some_and(|center_uv| {
                            if let Some(size) = target_px_size {
                                viewport_rotate_gizmo_hit_test_px(
                                    center_uv,
                                    size,
                                    event.target_px,
                                    56.0,
                                    8.0,
                                )
                            } else {
                                let (u, v) = center_uv;
                                let du = (start_uv.0 - u).abs();
                                let dv = (start_uv.1 - v).abs();
                                du <= 0.06 && dv <= 0.06
                            }
                        });

                        let Some(cur_tool) = tool.get(app) else {
                            break 'handled false;
                        };
                        let active = cur_tool.active;
                        if cur_tool.interaction.is_some() {
                            break 'handled false;
                        }

                        if active == ViewportToolMode::Move
                            && translate_hit.is_some()
                            && !selected.is_empty()
                        {
                            let mut start_positions: Vec<(u64, [f32; 3])> = Vec::new();
                            if let Some(world) = world_model {
                                if let Some(w) = world.get(app) {
                                    for &id in &selected {
                                        start_positions.push((id, w.position(id)));
                                    }
                                }
                            }

                            if !start_positions.is_empty() {
                                let targets: Vec<u64> =
                                    start_positions.iter().map(|(id, _)| *id).collect();
                                let before: Vec<[f32; 3]> =
                                    start_positions.iter().map(|(_, pos)| *pos).collect();

                                if let Some(stack) = undo_model {
                                    let _ = stack.update(app, |s, _cx| {
                                        s.begin_viewport_translate(targets.clone(), before)
                                    });
                                }

                                let _ = tool.update(app, |t, _cx| {
                                    t.interaction = Some(ViewportInteraction::TranslateGizmo(
                                        TranslateGizmoInteraction {
                                            window,
                                            target,
                                            start_modifiers: modifiers,
                                            start_uv,
                                            current_uv: start_uv,
                                            start_target_px: event.target_px,
                                            current_target_px: event.target_px,
                                            dragging: false,
                                            constraint: translate_hit
                                                .unwrap_or(TranslateAxisConstraint::Free),
                                            targets,
                                            start_positions,
                                        },
                                    ));
                                });
                                request_animation_frame = true;
                                break 'handled true;
                            }
                        }

                        if active == ViewportToolMode::Rotate && rotate_hit && !selected.is_empty()
                        {
                            let mut start_rotations: Vec<(u64, f32)> = Vec::new();
                            if let Some(world) = world_model {
                                if let Some(w) = world.get(app) {
                                    for &id in &selected {
                                        start_rotations.push((id, w.rotation_y(id)));
                                    }
                                }
                            }

                            if !start_rotations.is_empty() {
                                let (center_target_px, start_angle_rad, use_target_px) =
                                    if let (Some(size), Some(center_uv)) =
                                        (target_px_size, center_uv)
                                    {
                                        let (tw, th) = size;
                                        let center_target_px = (
                                            center_uv.0 * tw.max(1) as f32,
                                            center_uv.1 * th.max(1) as f32,
                                        );
                                        let dx = event.target_px.0 as f32 - center_target_px.0;
                                        let dy = event.target_px.1 as f32 - center_target_px.1;
                                        (center_target_px, dy.atan2(dx), true)
                                    } else {
                                        let center_uv = center_uv.unwrap_or((0.5, 0.5));
                                        let dx = start_uv.0 - center_uv.0;
                                        let dy = start_uv.1 - center_uv.1;
                                        ((0.0, 0.0), dy.atan2(dx), false)
                                    };

                                let targets: Vec<u64> =
                                    start_rotations.iter().map(|(id, _)| *id).collect();
                                let before: Vec<f32> =
                                    start_rotations.iter().map(|(_, rot)| *rot).collect();

                                if let Some(stack) = undo_model {
                                    let _ = stack.update(app, |s, _cx| {
                                        s.begin_viewport_rotate(targets.clone(), before)
                                    });
                                }

                                let _ = tool.update(app, |t, _cx| {
                                    t.interaction = Some(ViewportInteraction::RotateGizmo(
                                        RotateGizmoInteraction {
                                            window,
                                            target,
                                            start_modifiers: modifiers,
                                            center_uv: center_uv.unwrap_or((0.5, 0.5)),
                                            start_uv,
                                            current_uv: start_uv,
                                            start_target_px: event.target_px,
                                            current_target_px: event.target_px,
                                            center_target_px,
                                            start_angle_rad,
                                            use_target_px,
                                            dragging: false,
                                            targets,
                                            start_rotations,
                                        },
                                    ));
                                });
                                request_animation_frame = true;
                                break 'handled true;
                            }
                        }

                        let _ = tool.update(app, |t, _cx| {
                            t.interaction = Some(ViewportInteraction::MarqueeSelect(
                                MarqueeSelectInteraction {
                                    window,
                                    target,
                                    start_modifiers: modifiers,
                                    start_uv,
                                    current_uv: start_uv,
                                    start_target_px: event.target_px,
                                    current_target_px: event.target_px,
                                },
                            ));
                        });

                        marquee_update = Some(Some(ViewportMarquee {
                            a_uv: start_uv,
                            b_uv: start_uv,
                        }));
                        request_animation_frame = true;
                        break 'handled true;
                    }
                    fret_core::MouseButton::Right | fret_core::MouseButton::Middle => {
                        let kind = if button == fret_core::MouseButton::Right {
                            PanOrbitKind::Orbit
                        } else {
                            PanOrbitKind::Pan
                        };

                        let start_uv = event.uv;
                        let mut started = false;
                        let _ = tool.update(app, |t, _cx| {
                            if t.interaction.is_some() {
                                return;
                            }
                            t.interaction =
                                Some(ViewportInteraction::PanOrbit(PanOrbitInteraction {
                                    window,
                                    target,
                                    kind,
                                    button,
                                    start_modifiers: modifiers,
                                    start_uv,
                                    last_uv: start_uv,
                                    current_uv: start_uv,
                                    start_target_px: event.target_px,
                                    last_target_px: event.target_px,
                                    current_target_px: event.target_px,
                                    dragging: false,
                                }));
                            started = true;
                        });
                        started
                    }
                    _ => false,
                },
                fret_core::ViewportInputKind::PointerMove { buttons, modifiers } => 'mv: {
                    if !buttons.left && !buttons.right && !buttons.middle {
                        if !allow_edit {
                            let mut changed = false;
                            let _ = tool.update(app, |t, _cx| {
                                if t.hover_rotate.is_some() {
                                    t.hover_rotate = None;
                                    changed = true;
                                }
                            });
                            if changed {
                                request_redraw = true;
                            }
                            break 'mv false;
                        }

                        let selection = selection_model
                            .and_then(|m| m.get(app))
                            .cloned()
                            .unwrap_or_default();
                        let lead = selection.lead_entity;
                        let selected = selection.selected_entities;

                        let active = tool.get(app).map(|t| t.active).unwrap_or_default();
                        let can_hover = active == ViewportToolMode::Rotate
                            && !selected.is_empty()
                            && tool.get(app).is_some_and(|t| t.interaction.is_none());

                        let mut hover: Option<(fret_core::AppWindowId, RenderTargetId)> = None;
                        if can_hover {
                            let camera = panel_key
                                .as_ref()
                                .map(|p| self.viewport_camera(p))
                                .unwrap_or_default();
                            let center_uv = lead
                                .and_then(|id| {
                                    let world = world_model.and_then(|m| m.get(app))?;
                                    Some(camera.world_to_uv(world.position(id)))
                                })
                                .or_else(|| lead.and_then(viewport_grid_marker_uv));

                            if let (Some(center_uv), Some(size)) = (center_uv, target_px_size) {
                                if viewport_rotate_gizmo_hit_test_px(
                                    center_uv,
                                    size,
                                    event.target_px,
                                    56.0,
                                    8.0,
                                ) {
                                    hover = Some((window, target));
                                }
                            }
                        }

                        let mut changed = false;
                        let _ = tool.update(app, |t, _cx| {
                            let next = if can_hover { hover } else { None };
                            if t.hover_rotate != next {
                                t.hover_rotate = next;
                                changed = true;
                            }
                        });
                        if changed {
                            request_redraw = true;
                        }
                    }

                    if buttons.left {
                        let current_uv = event.uv;
                        let mut next_marquee: Option<MarqueeSelectInteraction> = None;
                        let mut next_gizmo: Option<(
                            bool,
                            TranslateAxisConstraint,
                            Vec<u64>,
                            Vec<(u64, [f32; 3])>,
                            f32,
                            f32,
                        )> = None;
                        let mut next_rotate: Option<(
                            bool,
                            Vec<u64>,
                            Vec<(u64, f32)>,
                            (f32, f32),
                            f32,
                            bool,
                            (u32, u32),
                            (f32, f32),
                            (f32, f32),
                        )> = None;

                        let _ = tool.update(app, |t, _cx| match t.interaction.as_mut() {
                            Some(ViewportInteraction::MarqueeSelect(m))
                                if m.window == window && m.target == target =>
                            {
                                m.current_uv = current_uv;
                                m.current_target_px = event.target_px;
                                next_marquee = Some(*m);
                            }
                            Some(ViewportInteraction::TranslateGizmo(m))
                                if m.window == window && m.target == target =>
                            {
                                m.current_uv = current_uv;
                                m.current_target_px = event.target_px;

                                let dx = m.start_target_px.0.abs_diff(m.current_target_px.0);
                                let dy = m.start_target_px.1.abs_diff(m.current_target_px.1);
                                if !m.dragging && (dx > 3 || dy > 3) {
                                    m.dragging = true;
                                }

                                let du = m.current_uv.0 - m.start_uv.0;
                                let dv = m.current_uv.1 - m.start_uv.1;
                                next_gizmo = Some((
                                    m.dragging,
                                    m.constraint,
                                    m.targets.clone(),
                                    m.start_positions.clone(),
                                    du,
                                    dv,
                                ));
                            }
                            Some(ViewportInteraction::RotateGizmo(m))
                                if m.window == window && m.target == target =>
                            {
                                m.current_uv = current_uv;
                                m.current_target_px = event.target_px;

                                let dx = m.start_target_px.0.abs_diff(m.current_target_px.0);
                                let dy = m.start_target_px.1.abs_diff(m.current_target_px.1);
                                if !m.dragging && (dx > 3 || dy > 3) {
                                    m.dragging = true;
                                }

                                next_rotate = Some((
                                    m.dragging,
                                    m.targets.clone(),
                                    m.start_rotations.clone(),
                                    m.center_target_px,
                                    m.start_angle_rad,
                                    m.use_target_px,
                                    m.current_target_px,
                                    m.center_uv,
                                    m.current_uv,
                                ));
                            }
                            _ => {}
                        });

                        if let Some(m) = next_marquee {
                            marquee_update = Some(Some(ViewportMarquee {
                                a_uv: m.start_uv,
                                b_uv: m.current_uv,
                            }));
                            request_animation_frame = true;
                            break 'mv true;
                        }

                        if let Some((
                            dragging,
                            constraint,
                            targets,
                            start_positions,
                            du_uv,
                            dv_uv,
                        )) = next_gizmo
                        {
                            if !dragging {
                                break 'mv true;
                            }

                            let camera = panel_key
                                .as_ref()
                                .map(|p| self.viewport_camera(p))
                                .unwrap_or_default();
                            let mut view_dx = du_uv * DemoViewportCamera::WORLD_SPAN;
                            let mut view_dy = -dv_uv * DemoViewportCamera::WORLD_SPAN;
                            match constraint {
                                TranslateAxisConstraint::Free => {}
                                TranslateAxisConstraint::X => view_dy = 0.0,
                                TranslateAxisConstraint::Y => view_dx = 0.0,
                            }

                            let r = [view_dx / camera.zoom, view_dy / camera.zoom];
                            let d = DemoViewportCamera::rotate(r, camera.rotation);
                            let (du, dv) = (d[0], d[1]);

                            let snap_step = if modifiers.shift { Some(0.25) } else { None };

                            let mut after: Vec<[f32; 3]> = Vec::with_capacity(targets.len());
                            for (_id, start) in &start_positions {
                                let mut x = start[0] + du;
                                let mut y = start[1] + dv;

                                if let Some(step) = snap_step {
                                    if step > 0.0 {
                                        if constraint == TranslateAxisConstraint::Free
                                            || constraint == TranslateAxisConstraint::X
                                        {
                                            x = (x / step).round() * step;
                                        }
                                        if constraint == TranslateAxisConstraint::Free
                                            || constraint == TranslateAxisConstraint::Y
                                        {
                                            y = (y / step).round() * step;
                                        }
                                    }
                                }

                                after.push([x, y, start[2]]);
                            }

                            if let Some(world) = world_model {
                                let _ = world.update(app, |w, _cx| {
                                    for ((id, _start), pos) in
                                        start_positions.iter().zip(after.iter().copied())
                                    {
                                        w.entity_mut(*id).transform.position = pos;
                                    }
                                });
                            }

                            if let Some(stack) = undo_model {
                                let _ = stack.update(app, |s, _cx| {
                                    s.update_viewport_translate(targets, after)
                                });
                            }

                            request_animation_frame = true;
                            pending_selection = None;
                            break 'mv true;
                        }

                        if let Some((
                            dragging,
                            targets,
                            start_rotations,
                            center_target_px,
                            start_angle_rad,
                            use_target_px,
                            current_target_px,
                            center_uv,
                            current_uv,
                        )) = next_rotate
                        {
                            if !dragging {
                                break 'mv true;
                            }

                            let current_angle = if use_target_px {
                                let dx = current_target_px.0 as f32 - center_target_px.0;
                                let dy = current_target_px.1 as f32 - center_target_px.1;
                                dy.atan2(dx)
                            } else {
                                let dx = current_uv.0 - center_uv.0;
                                let dy = current_uv.1 - center_uv.1;
                                dy.atan2(dx)
                            };
                            let mut delta_deg =
                                normalize_angle_rad(current_angle - start_angle_rad).to_degrees();
                            if modifiers.shift {
                                let step = 15.0_f32;
                                if step > 0.0 {
                                    delta_deg = (delta_deg / step).round() * step;
                                }
                            }

                            let mut after: Vec<f32> = Vec::with_capacity(targets.len());
                            for (_id, start) in &start_rotations {
                                after.push(*start + delta_deg);
                            }

                            if let Some(world) = world_model {
                                let _ = world.update(app, |w, _cx| {
                                    for ((id, _start), rot) in
                                        start_rotations.iter().zip(after.iter().copied())
                                    {
                                        w.entity_mut(*id).transform.rotation_y = rot;
                                    }
                                });
                            }

                            if let Some(stack) = undo_model {
                                let _ = stack
                                    .update(app, |s, _cx| s.update_viewport_rotate(targets, after));
                            }

                            request_animation_frame = true;
                            pending_selection = None;
                            break 'mv true;
                        }
                    }

                    if buttons.right || buttons.middle || buttons.left {
                        let current_uv = event.uv;
                        let mut next: Option<PanOrbitInteraction> = None;
                        let mut camera_step: Option<(PanOrbitKind, f32, f32)> = None;
                        let _ = tool.update(app, |t, _cx| {
                            let Some(ViewportInteraction::PanOrbit(m)) = t.interaction.as_mut()
                            else {
                                return;
                            };

                            if m.window == window && m.target == target {
                                let want = match m.button {
                                    fret_core::MouseButton::Left => {
                                        buttons.left && !buttons.right && !buttons.middle
                                    }
                                    fret_core::MouseButton::Right => {
                                        buttons.right && !buttons.middle
                                    }
                                    fret_core::MouseButton::Middle => {
                                        buttons.middle && !buttons.right
                                    }
                                    _ => false,
                                };

                                if !want {
                                    return;
                                }

                                m.current_uv = current_uv;
                                m.current_target_px = event.target_px;

                                let dx = m.start_target_px.0.abs_diff(m.current_target_px.0);
                                let dy = m.start_target_px.1.abs_diff(m.current_target_px.1);
                                if !m.dragging && (dx > 3 || dy > 3) {
                                    m.dragging = true;
                                }

                                if m.dragging {
                                    let du = m.current_uv.0 - m.last_uv.0;
                                    let dv = m.current_uv.1 - m.last_uv.1;
                                    m.last_uv = m.current_uv;
                                    m.last_target_px = m.current_target_px;
                                    camera_step = Some((m.kind, du, dv));
                                }

                                next = Some(*m);
                            }
                        });

                        let Some(m) = next else {
                            break 'mv false;
                        };
                        if !m.dragging {
                            break 'mv true;
                        }

                        if let Some((kind, du, dv)) = camera_step {
                            if let Some(panel) = panel_key.clone() {
                                let cam = self.viewport_camera_mut(panel);
                                match kind {
                                    PanOrbitKind::Pan => cam.pan_by_uv_delta(du, dv),
                                    PanOrbitKind::Orbit => cam.orbit_by_uv_delta(du),
                                }
                                self.schedule_camera_persist(app);
                            }
                        }

                        let theme = Theme::global(app).snapshot();
                        let color = match m.kind {
                            PanOrbitKind::Orbit => theme.colors.viewport_drag_line_orbit,
                            PanOrbitKind::Pan => theme.colors.viewport_drag_line_pan,
                        };

                        drag_line_update = Some(Some(fret_ui::dock::ViewportDragLine {
                            a_uv: m.start_uv,
                            b_uv: m.current_uv,
                            color,
                        }));
                        request_animation_frame = true;
                        break 'mv true;
                    }

                    false
                }
                fret_core::ViewportInputKind::Wheel { delta, modifiers } => {
                    let mut wheel_y = delta.y.0;
                    if modifiers.shift {
                        wheel_y *= 4.0;
                    }
                    if let Some(panel) = panel_key.clone() {
                        self.viewport_camera_mut(panel)
                            .zoom_at_uv(event.uv, wheel_y);
                        self.schedule_camera_persist(app);
                    }
                    request_redraw = true;
                    true
                }
                fret_core::ViewportInputKind::PointerUp { button, .. } => match button {
                    fret_core::MouseButton::Left => 'up_left: {
                        let mut commit: Option<MarqueeSelectInteraction> = None;
                        let mut ended_pan_orbit: Option<PanOrbitInteraction> = None;
                        let mut ended_translate_dragging: Option<bool> = None;
                        let mut ended_rotate_dragging: Option<bool> = None;
                        let _ = tool.update(app, |t, _cx| match t.interaction.take() {
                            Some(ViewportInteraction::MarqueeSelect(m))
                                if m.window == window && m.target == target =>
                            {
                                commit = Some(m);
                            }
                            Some(ViewportInteraction::PanOrbit(m))
                                if m.window == window && m.target == target =>
                            {
                                ended_pan_orbit = Some(m);
                            }
                            Some(ViewportInteraction::TranslateGizmo(m))
                                if m.window == window && m.target == target =>
                            {
                                ended_translate_dragging = Some(m.dragging);
                            }
                            Some(ViewportInteraction::RotateGizmo(m))
                                if m.window == window && m.target == target =>
                            {
                                ended_rotate_dragging = Some(m.dragging);
                            }
                            other => t.interaction = other,
                        });

                        marquee_update = Some(None);
                        drag_line_update = Some(None);
                        request_redraw = true;

                        if let Some(dragging) = ended_translate_dragging {
                            if let Some(stack) = undo_model {
                                let _ = stack.update(app, |s, _cx| {
                                    if dragging {
                                        s.commit_active();
                                    } else {
                                        s.cancel_active();
                                    }
                                });
                            }
                            if dragging {
                                self.mark_scene_dirty(app);
                            }
                            break 'up_left true;
                        }

                        if let Some(dragging) = ended_rotate_dragging {
                            if let Some(stack) = undo_model {
                                let _ = stack.update(app, |s, _cx| {
                                    if dragging {
                                        s.commit_active();
                                    } else {
                                        s.cancel_active();
                                    }
                                });
                            }
                            if dragging {
                                self.mark_scene_dirty(app);
                            }
                            break 'up_left true;
                        }

                        if let Some(m) = ended_pan_orbit {
                            if button == m.button {
                                break 'up_left true;
                            }
                        }

                        let Some(m) = commit else {
                            break 'up_left true;
                        };

                        let camera = panel_key
                            .as_ref()
                            .map(|p| self.viewport_camera(p))
                            .unwrap_or_default();

                        let dx = m.start_target_px.0.abs_diff(m.current_target_px.0);
                        let dy = m.start_target_px.1.abs_diff(m.current_target_px.1);

                        let (lead, ids) = if let Some(world) = world_model.and_then(|m| m.get(app))
                        {
                            if dx <= 3 && dy <= 3 {
                                match demo_pick_entity_by_uv(
                                    world,
                                    camera,
                                    m.current_uv,
                                    target_px_size,
                                ) {
                                    Some(id) => (Some(id), vec![id]),
                                    None => (None, Vec::new()),
                                }
                            } else {
                                let ids = demo_pick_entities_in_uv_rect(
                                    world,
                                    camera,
                                    m.start_uv,
                                    m.current_uv,
                                );
                                let lead = ids.last().copied();
                                (lead, ids)
                            }
                        } else if dx <= 3 && dy <= 3 {
                            let grid_w: u64 = 64;
                            let grid_h: u64 = 36;
                            let x = ((m.current_uv.0 * grid_w as f32).floor() as u64)
                                .min(grid_w.saturating_sub(1));
                            let y = ((m.current_uv.1 * grid_h as f32).floor() as u64)
                                .min(grid_h.saturating_sub(1));
                            let id = 1 + y * grid_w + x;
                            (Some(id), vec![id])
                        } else {
                            let (u0, v0) = (
                                m.start_uv.0.min(m.current_uv.0),
                                m.start_uv.1.min(m.current_uv.1),
                            );
                            let (u1, v1) = (
                                m.start_uv.0.max(m.current_uv.0),
                                m.start_uv.1.max(m.current_uv.1),
                            );
                            let grid_w: u64 = 64;
                            let grid_h: u64 = 36;
                            let x0 =
                                ((u0 * grid_w as f32).floor() as u64).min(grid_w.saturating_sub(1));
                            let x1 =
                                ((u1 * grid_w as f32).floor() as u64).min(grid_w.saturating_sub(1));
                            let y0 =
                                ((v0 * grid_h as f32).floor() as u64).min(grid_h.saturating_sub(1));
                            let y1 =
                                ((v1 * grid_h as f32).floor() as u64).min(grid_h.saturating_sub(1));

                            let mut out: Vec<u64> = Vec::new();
                            for y in y0..=y1 {
                                for x in x0..=x1 {
                                    out.push(1 + y * grid_w + x);
                                    if out.len() >= 2048 {
                                        break;
                                    }
                                }
                                if out.len() >= 2048 {
                                    break;
                                }
                            }
                            let lead = out.last().copied();
                            (lead, out)
                        };

                        pending_selection = Some((lead, ids, m.start_modifiers));
                        true
                    }
                    fret_core::MouseButton::Right | fret_core::MouseButton::Middle => 'up_other: {
                        let mut end: Option<PanOrbitInteraction> = None;
                        let _ = tool.update(app, |t, _cx| {
                            end = match t.interaction.take() {
                                Some(ViewportInteraction::PanOrbit(m))
                                    if m.window == window && m.target == target =>
                                {
                                    Some(m)
                                }
                                other => {
                                    t.interaction = other;
                                    None
                                }
                            };
                        });

                        let Some(m) = end else {
                            break 'up_other false;
                        };
                        if button != m.button {
                            break 'up_other false;
                        }

                        drag_line_update = Some(None);
                        request_redraw = true;
                        true
                    }
                    _ => false,
                },
            };

            if let Some(dock) = app.global_mut::<DockManager>() {
                if let Some(update) = marquee_update {
                    dock.set_viewport_marquee(window, target, update);
                }
                if let Some(update) = drag_line_update {
                    dock.set_viewport_drag_line(window, target, update);
                }
            }

            if request_redraw {
                app.request_redraw(window);
            }
            if request_animation_frame {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }

            if handled {
                if let Some((lead, ids, modifiers)) = pending_selection.take() {
                    self.apply_selection_delta(app, lead, ids, modifiers);
                }
                return;
            }
        }

        match event.kind {
            fret_core::ViewportInputKind::PointerDown { .. } => {
                println!("viewport_input: {event:?}");
            }
            fret_core::ViewportInputKind::PointerUp { .. }
            | fret_core::ViewportInputKind::Wheel { .. } => {
                println!("viewport_input: {event:?}");
            }
            fret_core::ViewportInputKind::PointerMove { .. } => {}
        }
    }

    fn render(
        &mut self,
        app: &mut App,
        window: fret_core::AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        text: &mut dyn fret_core::TextService,
        scene: &mut Scene,
    ) {
        scene.clear();
        self.sync_scene_chrome(app);
        self.sync_viewport_selection_overlay_for_window(app, window);

        let hud_enabled = app
            .global::<DebugHudService>()
            .is_some_and(|hud| hud.enabled());
        state.ui.set_debug_enabled(hud_enabled);
        if state.ui.is_layer_visible(state.layers.debug_hud) != hud_enabled {
            state
                .ui
                .set_layer_visible(state.layers.debug_hud, hud_enabled);
        }

        let inspector_enabled = app
            .global::<DebugInspectorService>()
            .is_some_and(|s| s.enabled());
        if state.ui.is_layer_visible(state.layers.debug_inspector) != inspector_enabled {
            state
                .ui
                .set_layer_visible(state.layers.debug_inspector, inspector_enabled);
        }

        state.ui.layout_all(app, text, bounds, scale_factor);

        if inspector_enabled {
            let cursor = state.last_cursor_pos;
            let mut snapshot = DebugInspectorSnapshot::default();
            snapshot.frame_id = app.frame_id();
            snapshot.cursor = cursor;
            snapshot.focus = state.ui.focus();
            snapshot.captured = state.ui.captured();

            let hit_color = Color {
                r: 0.1,
                g: 0.8,
                b: 1.0,
                a: 0.95,
            };
            let focus_color = Color {
                r: 0.2,
                g: 0.95,
                b: 0.2,
                a: 0.95,
            };
            let capture_color = Color {
                r: 1.0,
                g: 0.6,
                b: 0.15,
                a: 0.95,
            };
            let barrier_color = Color {
                r: 0.8,
                g: 0.35,
                b: 1.0,
                a: 0.95,
            };

            let push_outline = |node: fret_core::NodeId, color: Color, out: &mut Vec<_>| {
                if let Some(rect) = state.ui.debug_node_bounds(node) {
                    out.push(DebugInspectorOutline { rect, color });
                }
            };

            if let Some(pos) = cursor {
                let hit = state.ui.debug_hit_test(pos);
                snapshot.hit = hit.hit;
                snapshot.barrier_root = hit.barrier_root;
                snapshot.active_layer_roots = hit.active_layer_roots.clone();
                if let Some(node) = hit.barrier_root {
                    push_outline(node, barrier_color, &mut snapshot.outlines);
                }

                if let Some(node) = hit.hit {
                    let path = state.ui.debug_node_path(node);
                    let denom = (path.len().saturating_sub(1)).max(1) as f32;
                    for (i, id) in path.into_iter().enumerate() {
                        let t = i as f32 / denom;
                        let a = 0.15 + t * 0.75;
                        push_outline(id, Color { a, ..hit_color }, &mut snapshot.outlines);
                    }
                }
            }

            snapshot.layers = state.ui.debug_layers_in_paint_order();

            if let Some(node) = snapshot.focus {
                push_outline(node, focus_color, &mut snapshot.outlines);
            }
            if let Some(node) = snapshot.captured {
                push_outline(node, capture_color, &mut snapshot.outlines);
            }

            app.global_mut::<DebugInspectorService>()
                .map(|s| s.set_snapshot(window, snapshot));
        }

        state.ui.paint_all(app, text, bounds, scene, scale_factor);

        if hud_enabled {
            let stats = state.ui.debug_stats();
            app.global_mut::<DebugHudService>()
                .map(|hud| hud.set_stats(window, stats));
        }
    }

    fn window_create_spec(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match &request.kind {
            CreateWindowKind::DockFloating { panel, .. } => {
                self.pending_window_ui_kinds
                    .push_back(DemoUiKind::DockFloating);
                let title = app
                    .global::<DockManager>()
                    .and_then(|dock| dock.panel(panel))
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|| "Floating".to_string());
                Some(WindowCreateSpec::new(
                    format!("fret-demo - {title}"),
                    winit::dpi::LogicalSize::new(640.0, 480.0),
                ))
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                self.pending_window_ui_kinds
                    .push_back(DemoUiKind::DockFloating);
                let mut spec = WindowCreateSpec::new(
                    format!("fret-demo - {logical_window_id}"),
                    winit::dpi::LogicalSize::new(640.0, 480.0),
                );

                if let Some(layout) = self.loaded_layout.as_ref() {
                    if let Some(entry) = layout
                        .windows
                        .iter()
                        .find(|w| w.logical_window_id == *logical_window_id)
                    {
                        if let Some(p) = entry.placement.as_ref() {
                            spec.size =
                                winit::dpi::LogicalSize::new(p.width as f64, p.height as f64);
                            if let (Some(x), Some(y)) = (p.x, p.y) {
                                spec.position = Some(winit::dpi::Position::Logical(
                                    winit::dpi::LogicalPosition::new(x as f64, y as f64),
                                ));
                            }
                        }
                    }
                }

                Some(spec)
            }
        }
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &CreateWindowRequest,
        new_window: fret_core::AppWindowId,
    ) {
        match &request.kind {
            CreateWindowKind::DockFloating {
                source_window,
                panel,
            } => {
                self.dock_op(
                    app,
                    DockOp::FloatPanelToWindow {
                        source_window: *source_window,
                        panel: panel.clone(),
                        new_window,
                    },
                );

                if !self.logical_windows.contains_key(&new_window) {
                    let id = self.next_floating_logical_id();
                    self.logical_windows.insert(new_window, id);
                }

                app.request_redraw(*source_window);
                app.request_redraw(new_window);
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                self.logical_windows
                    .insert(new_window, logical_window_id.clone());

                let Some(layout) = self.loaded_layout.as_ref() else {
                    return;
                };
                let Some(entry) = layout
                    .windows
                    .iter()
                    .find(|w| w.logical_window_id == *logical_window_id)
                else {
                    return;
                };

                let Some(dock) = app.global_mut::<DockManager>() else {
                    return;
                };
                if let Some(root) = dock.graph.import_subtree_from_layout_v1(layout, entry.root) {
                    dock.graph.set_window_root(new_window, root);
                    app.push_effect(Effect::UiInvalidateLayout { window: new_window });
                    app.request_redraw(new_window);
                }
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: fret_core::AppWindowId) -> bool {
        let Some(main) = self.main_window else {
            return true;
        };
        if window == main {
            self.persist_layout_now(app);
            return true;
        }

        let mut windows_to_invalidate: Vec<fret_core::AppWindowId> = Vec::new();
        {
            let Some(dock) = app.global_mut::<DockManager>() else {
                return true;
            };

            let target_tabs = Self::ensure_main_tabs(dock, main);
            let _ = dock.graph.apply_op(&DockOp::MergeWindowInto {
                source_window: window,
                target_window: main,
                target_tabs,
            });
            self.logical_windows.remove(&window);

            for &w in self.logical_windows.keys() {
                if dock.graph.window_root(w).is_some() {
                    windows_to_invalidate.push(w);
                }
            }
            if dock.graph.window_root(main).is_some() {
                windows_to_invalidate.push(main);
            }
        }

        for w in windows_to_invalidate {
            app.push_effect(Effect::UiInvalidateLayout { window: w });
        }
        self.schedule_layout_persist(app);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{DockLayoutNodeV1, DockLayoutWindowV1};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_temp_path(stem: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("fret-demo-tests-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);

        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        dir.push(format!("{stem}-{n}.json"));
        dir
    }

    #[test]
    fn bump_next_floating_index_from_layout_tracks_max() {
        let mut driver = DemoDriver::default();
        driver.next_floating_index = 1;

        let layout = DockLayoutV1::new_v1(
            vec![DockLayoutWindowV1 {
                logical_window_id: "floating-3".to_string(),
                root: 0,
                placement: None,
            }],
            Vec::new(),
        );

        driver.bump_next_floating_index_from_layout(&layout);
        assert_eq!(driver.next_floating_index, 4);

        let layout = DockLayoutV1::new_v1(
            vec![DockLayoutWindowV1 {
                logical_window_id: "floating-abc".to_string(),
                root: 0,
                placement: None,
            }],
            Vec::new(),
        );

        driver.bump_next_floating_index_from_layout(&layout);
        assert_eq!(driver.next_floating_index, 4);
    }

    #[test]
    fn save_and_load_layout_roundtrips() {
        let path = unique_temp_path("layout");

        let layout = DockLayoutV1::new_v1(
            vec![DockLayoutWindowV1 {
                logical_window_id: "main".to_string(),
                root: 1,
                placement: None,
            }],
            vec![DockLayoutNodeV1::Tabs {
                id: 1,
                tabs: vec![PanelKey::new("core.hierarchy")],
                active: 0,
            }],
        );

        DemoDriver::save_layout_to_path(&path, &layout).unwrap();
        let loaded = DemoDriver::load_layout_from_path(&path).unwrap();

        let orig = serde_json::to_value(&layout).unwrap();
        let got = serde_json::to_value(&loaded).unwrap();
        assert_eq!(orig, got);

        let _ = std::fs::remove_file(&path);
    }
}

fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_platform=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new()?;
    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo".to_string(),
        ..Default::default()
    };

    if let Some(layout) = DemoDriver::load_layout_file() {
        if let Some(main_entry) = layout
            .windows
            .iter()
            .find(|w| w.logical_window_id == "main")
        {
            if let Some(p) = main_entry.placement.as_ref() {
                config.main_window_size =
                    winit::dpi::LogicalSize::new(p.width as f64, p.height as f64);
                if let (Some(x), Some(y)) = (p.x, p.y) {
                    config.main_window_position = Some(winit::dpi::Position::Logical(
                        winit::dpi::LogicalPosition::new(x as f64, y as f64),
                    ));
                }
            }
        }
    }

    let mut app = App::new();
    let mut caps = PlatformCapabilities::default();
    if std::env::var_os("FRET_FORCE_SINGLE_WINDOW").is_some() {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = false;
    }
    app.set_global(caps);
    let driver = DemoDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
