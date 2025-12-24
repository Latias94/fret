use crate::DemoPlayState;
use crate::asset_drop::CurrentSceneService;
use crate::command_palette::{CommandPalette, OverlayBackdrop, OverlayPanelLayout};
use crate::dnd_probe::DndProbe;
use crate::elements_mvp2::ElementsMvp2Demo;
use crate::ime_probe::ImeProbe;
use crate::overlay_layouts::{CenteredOverlayLayout, CornerOverlayLayout};
use crate::scene_document::SceneDocumentService;
use fret_app::Model;
use fret_components_ui::{AppMenuBar, ContextMenu, Popover};
use fret_core::{AppWindowId, Axis, Color, ExternalDragPayloadKind, PlatformCapabilities, Px};
use fret_editor::{InspectorEditHint, InspectorEditLayout};
use fret_editor::{ViewportToolManager, ViewportToolMode};
use fret_ui_app::{
    App, Bar, BoundTextInput, ColoredPanel, Column, DockSpace, FixedPanel, GenericWidget,
    HeaderBody, PanelThemeBackground, Row, Scroll, Split, Stack, Text, TextArea, TextInput,
    Toolbar, ToolbarItem, UiLayerId, UiTree, VirtualList, VirtualListDataSource, VirtualListRow,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct DebugHudService {
    enabled: bool,
    per_window: HashMap<AppWindowId, fret_ui_app::UiDebugFrameStats>,
}

#[derive(Default)]
pub struct DebugInspectorService {
    enabled: bool,
    per_window: HashMap<AppWindowId, DebugInspectorSnapshot>,
}

impl DebugInspectorService {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: DebugInspectorSnapshot) {
        self.per_window.insert(window, snapshot);
    }

    pub fn snapshot(&self, window: AppWindowId) -> Option<&DebugInspectorSnapshot> {
        self.per_window.get(&window)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DebugInspectorSnapshot {
    pub frame_id: fret_core::FrameId,
    pub cursor: Option<fret_core::Point>,
    pub hit: Option<fret_core::NodeId>,
    pub focus: Option<fret_core::NodeId>,
    pub captured: Option<fret_core::NodeId>,
    pub barrier_root: Option<fret_core::NodeId>,
    pub active_layer_roots: Vec<fret_core::NodeId>,
    pub layers: Vec<fret_ui_app::UiDebugLayerInfo>,
    pub outlines: Vec<DebugInspectorOutline>,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugInspectorOutline {
    pub rect: fret_core::Rect,
    pub color: fret_core::Color,
}

impl DebugHudService {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    pub fn set_stats(&mut self, window: AppWindowId, stats: fret_ui_app::UiDebugFrameStats) {
        self.per_window.insert(window, stats);
    }

    pub fn stats(&self, window: AppWindowId) -> Option<fret_ui_app::UiDebugFrameStats> {
        self.per_window.get(&window).copied()
    }
}

#[derive(Debug, Clone)]
struct LazyEntityList {
    count: usize,
}

impl VirtualListDataSource for LazyEntityList {
    type Key = u64;

    fn len(&self) -> usize {
        self.count
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index as u64
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        VirtualListRow::new(Cow::Owned(format!("Entity {index:06}")))
    }

    fn index_of_key(&self, key: Self::Key) -> Option<usize> {
        let index = key as usize;
        if index < self.count {
            Some(index)
        } else {
            None
        }
    }
}

pub struct DemoUiConfig {
    pub split_fraction: f32,
}

impl Default for DemoUiConfig {
    fn default() -> Self {
        Self {
            split_fraction: 0.72,
        }
    }
}

pub struct DemoLayers {
    pub modal: UiLayerId,
    pub external_dnd: UiLayerId,
    pub command_palette: UiLayerId,
    pub command_palette_node: fret_core::NodeId,
    pub popover: UiLayerId,
    pub popover_node: fret_core::NodeId,
    pub context_menu: UiLayerId,
    pub context_menu_node: fret_core::NodeId,
    pub inspector_edit: UiLayerId,
    pub inspector_edit_input_node: fret_core::NodeId,
    pub debug_hud: UiLayerId,
    pub debug_inspector: UiLayerId,
    pub dockspace_node: fret_core::NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoUiKind {
    Main,
    DockFloating,
}

struct DemoToolbar {
    tools: Model<ViewportToolManager>,
    toolbar: Toolbar,
    last_mode: Option<ViewportToolMode>,
    last_playing: Option<bool>,
}

impl DemoToolbar {
    pub fn new(tools: Model<ViewportToolManager>) -> Self {
        Self {
            tools,
            toolbar: Toolbar::new(Vec::new()),
            last_mode: None,
            last_playing: None,
        }
    }

    fn rebuild_items(&mut self, app: &mut fret_app::App) -> bool {
        let mode = self.tools.get(app).map(|t| t.active).unwrap_or_default();
        let playing = app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        if self.last_mode == Some(mode) && self.last_playing == Some(playing) {
            return false;
        }
        self.last_mode = Some(mode);
        self.last_playing = Some(playing);

        let play_label: Arc<str> = if playing { "Stop" } else { "Play" }.into();
        let items = vec![
            ToolbarItem::new("New", "scene.new"),
            ToolbarItem::new("Save", "scene.save"),
            ToolbarItem::new("Save As", "scene.save_as"),
            ToolbarItem::new("Select (Q)", "viewport.tool.select")
                .with_selected(mode == ViewportToolMode::Select),
            ToolbarItem::new("Move (W)", "viewport.tool.move")
                .with_selected(mode == ViewportToolMode::Move),
            ToolbarItem::new("Rotate (E)", "viewport.tool.rotate")
                .with_selected(mode == ViewportToolMode::Rotate),
            ToolbarItem::new(play_label, "demo.play.toggle").with_selected(playing),
            ToolbarItem::new("Save Layout", "dock.layout.preset.save_last"),
            ToolbarItem::new("Load Layout", "dock.layout.preset.load_last"),
            ToolbarItem::new("Reset Layout", "dock.layout.reset_default"),
        ];
        self.toolbar.set_items(items);
        true
    }
}

impl GenericWidget<App> for DemoToolbar {
    fn event(&mut self, cx: &mut fret_ui_app::EventCx<'_>, event: &fret_core::Event) {
        if self.rebuild_items(cx.app) {
            cx.invalidate_self(fret_ui_app::Invalidation::Layout);
            cx.invalidate_self(fret_ui_app::Invalidation::Paint);
            cx.request_redraw();
        }
        self.toolbar.event(cx, event);
    }

    fn layout(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> fret_core::Size {
        let _ = self.rebuild_items(cx.app);
        self.toolbar.layout(cx)
    }

    fn paint(&mut self, cx: &mut fret_ui_app::PaintCx<'_>) {
        self.toolbar.paint(cx);
    }
}

struct DemoTopBarStatus {
    text: String,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<fret_core::TextMetrics>,
    last_key: Option<(u64, u64, bool)>,
    last_scale_factor: Option<f32>,
    style: fret_core::TextStyle,
}

impl DemoTopBarStatus {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            text_blob: None,
            text_metrics: None,
            last_key: None,
            last_scale_factor: None,
            style: fret_core::TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
        }
    }

    fn compute_text(app: &fret_app::App) -> String {
        let playing = app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        let play = if playing { "Play" } else { "Edit" };

        let dirty = app
            .global::<SceneDocumentService>()
            .is_some_and(|s| s.dirty());

        let title = app
            .global::<CurrentSceneService>()
            .and_then(|s| s.guid())
            .and_then(|guid| {
                let project = app.global::<fret_editor::ProjectService>()?;
                let id = project.id_for_guid(guid)?;
                let path = project.path_for_id(id)?;
                path.file_stem().map(|s| s.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "No Scene".to_string());

        let suffix = if dirty { "*" } else { "" };
        format!("{play} | {title}{suffix}")
    }

    fn sync_text(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> bool {
        let playing = cx.app.global::<DemoPlayState>().is_some_and(|s| s.playing);
        let scene_rev = cx
            .app
            .global::<CurrentSceneService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let doc_rev = cx
            .app
            .global::<SceneDocumentService>()
            .map(|s| s.revision())
            .unwrap_or(0);
        let key = (scene_rev, doc_rev, playing);

        if self.last_key == Some(key) && self.last_scale_factor == Some(cx.scale_factor) {
            return false;
        }
        self.last_key = Some(key);
        self.last_scale_factor = Some(cx.scale_factor);

        if let Some(blob) = self.text_blob.take() {
            cx.text.release(blob);
        }
        self.text = Self::compute_text(cx.app);

        let constraints = fret_core::TextConstraints {
            max_width: Some(cx.available.width),
            wrap: fret_core::TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.text_blob = Some(blob);
        self.text_metrics = Some(metrics);
        true
    }
}

impl GenericWidget<App> for DemoTopBarStatus {
    fn event(&mut self, _cx: &mut fret_ui_app::EventCx<'_>, _event: &fret_core::Event) {}

    fn layout(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> fret_core::Size {
        let _ = self.sync_text(cx);
        self.text_metrics.map(|m| m.size).unwrap_or_default()
    }

    fn paint(&mut self, cx: &mut fret_ui_app::PaintCx<'_>) {
        let theme = cx.theme().snapshot();
        let Some(blob) = self.text_blob else {
            return;
        };
        let Some(metrics) = self.text_metrics else {
            return;
        };

        let pad = theme.metrics.padding_md.0.max(0.0);
        let content_w = metrics.size.width.0;
        let x = (cx.bounds.origin.x.0 + cx.bounds.size.width.0 - pad - content_w)
            .max(cx.bounds.origin.x.0 + pad);

        let inner_y =
            cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - metrics.size.height.0) * 0.5);
        let y = inner_y + metrics.baseline.0;

        cx.scene.push(fret_core::SceneOp::Text {
            order: fret_core::DrawOrder(0),
            origin: fret_core::geometry::Point::new(Px(x), Px(y)),
            text: blob,
            color: theme.colors.text_muted,
        });
    }
}

struct DebugHudPanel {
    text: String,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<fret_core::TextMetrics>,
    last_key: Option<(u64, u32, u32, u32, u32, u32, u32, u8, u8, u8)>,
    last_scale_factor: Option<f32>,
    style: fret_core::TextStyle,
}

impl DebugHudPanel {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            text_blob: None,
            text_metrics: None,
            last_key: None,
            last_scale_factor: None,
            style: fret_core::TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
        }
    }

    fn sync_text(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(hud) = cx.app.global::<DebugHudService>() else {
            return false;
        };
        let Some(stats) = hud.stats(window) else {
            return false;
        };

        let caps = cx
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let payload_key: u8 = match caps.dnd.external_payload {
            ExternalDragPayloadKind::None => 0,
            ExternalDragPayloadKind::FilePath => 1,
            ExternalDragPayloadKind::FileToken => 2,
            ExternalDragPayloadKind::Text => 3,
        };

        let key = (
            stats.frame_id.0,
            stats.layout_nodes_visited,
            stats.layout_nodes_performed,
            stats.paint_nodes,
            stats.paint_nodes_performed,
            stats.paint_cache_hits,
            stats.paint_cache_misses,
            caps.ui.multi_window as u8,
            caps.ui.window_tear_off as u8,
            payload_key,
        );
        if self.last_key == Some(key) && self.last_scale_factor == Some(cx.scale_factor) {
            return false;
        }
        self.last_key = Some(key);
        self.last_scale_factor = Some(cx.scale_factor);

        if let Some(blob) = self.text_blob.take() {
            cx.text.release(blob);
        }

        let layout_ms = stats.layout_time.as_secs_f64() * 1000.0;
        let paint_ms = stats.paint_time.as_secs_f64() * 1000.0;

        self.text = format!(
            "UI Debug\nframe: {}\nlayout: {:.2} ms ({} / {})\npaint: {:.2} ms ({} / {} nodes)\npaint cache: {} hit / {} miss ({} ops)\nfocus: {:?}\ncapture: {:?}\n\ncaps.ui.multi_window: {}\ncaps.ui.window_tear_off: {}\ncaps.dnd.external_payload: {}",
            stats.frame_id.0,
            layout_ms,
            stats.layout_nodes_performed,
            stats.layout_nodes_visited,
            paint_ms,
            stats.paint_nodes_performed,
            stats.paint_nodes,
            stats.paint_cache_hits,
            stats.paint_cache_misses,
            stats.paint_cache_replayed_ops,
            stats.focus,
            stats.captured,
            caps.ui.multi_window,
            caps.ui.window_tear_off,
            caps.dnd.external_payload.as_str(),
        );

        let constraints = fret_core::TextConstraints {
            max_width: Some(cx.available.width),
            wrap: fret_core::TextWrap::Word,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.text_blob = Some(blob);
        self.text_metrics = Some(metrics);
        true
    }
}

impl GenericWidget<App> for DebugHudPanel {
    fn event(&mut self, _cx: &mut fret_ui_app::EventCx<'_>, _event: &fret_core::Event) {}

    fn layout(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> fret_core::Size {
        let _ = self.sync_text(cx);
        let size = self.text_metrics.map(|m| m.size).unwrap_or_default();
        let pad = cx.theme().snapshot().metrics.padding_md;
        fret_core::Size::new(size.width + pad + pad, size.height + pad + pad)
    }

    fn paint(&mut self, cx: &mut fret_ui_app::PaintCx<'_>) {
        let theme = cx.theme().snapshot();
        let Some(blob) = self.text_blob else {
            return;
        };
        let Some(metrics) = self.text_metrics else {
            return;
        };

        let pad = theme.metrics.padding_md;
        let bg = theme.colors.panel_background;
        let border = theme.colors.panel_border;

        cx.scene.push(fret_core::SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: fret_core::Edges::all(Px(1.0)),
            border_color: border,
            corner_radii: fret_core::Corners::all(theme.metrics.radius_md),
        });

        let origin = fret_core::Point::new(
            cx.bounds.origin.x + pad,
            cx.bounds.origin.y + pad + metrics.baseline,
        );
        cx.scene.push(fret_core::SceneOp::Text {
            order: fret_core::DrawOrder(1),
            origin,
            text: blob,
            color: theme.colors.text_primary,
        });
    }
}

struct DebugInspectorOverlay {
    text: String,
    text_blob: Option<fret_core::TextBlobId>,
    text_metrics: Option<fret_core::TextMetrics>,
    last_key: Option<(
        Option<(i32, i32)>,
        Option<fret_core::NodeId>,
        Option<fret_core::NodeId>,
        Option<fret_core::NodeId>,
        Option<fret_core::NodeId>,
        usize,
        usize,
    )>,
    last_scale_factor: Option<f32>,
    style: fret_core::TextStyle,
}

impl DebugInspectorOverlay {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            text_blob: None,
            text_metrics: None,
            last_key: None,
            last_scale_factor: None,
            style: fret_core::TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
        }
    }

    fn sync_text(&mut self, cx: &mut fret_ui_app::PaintCx<'_>, snapshot: &DebugInspectorSnapshot) {
        let cursor = snapshot
            .cursor
            .map(|p| (p.x.0.round() as i32, p.y.0.round() as i32));
        let key = (
            cursor,
            snapshot.hit,
            snapshot.focus,
            snapshot.captured,
            snapshot.barrier_root,
            snapshot.active_layer_roots.len(),
            snapshot.layers.len(),
        );
        if self.last_key == Some(key) && self.last_scale_factor == Some(cx.scale_factor) {
            return;
        }
        self.last_key = Some(key);
        self.last_scale_factor = Some(cx.scale_factor);

        if let Some(blob) = self.text_blob.take() {
            cx.text.release(blob);
        }

        let layer_summary = snapshot.layers.iter().filter(|l| l.visible).count();
        let barrier = snapshot.barrier_root;

        self.text = format!(
            "UI Inspector\ncursor: {:?}\nhit: {:?}\nfocus: {:?}\ncapture: {:?}\nbarrier: {:?}\nvisible layers: {}\nactive roots: {}",
            cursor,
            snapshot.hit,
            snapshot.focus,
            snapshot.captured,
            barrier,
            layer_summary,
            snapshot.active_layer_roots.len(),
        );

        let constraints = fret_core::TextConstraints {
            max_width: Some(Px(320.0)),
            wrap: fret_core::TextWrap::Word,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) = cx.text.prepare(&self.text, self.style, constraints);
        self.text_blob = Some(blob);
        self.text_metrics = Some(metrics);
    }
}

impl GenericWidget<App> for DebugInspectorOverlay {
    fn event(&mut self, _cx: &mut fret_ui_app::EventCx<'_>, _event: &fret_core::Event) {}

    fn layout(&mut self, cx: &mut fret_ui_app::LayoutCx<'_>) -> fret_core::Size {
        cx.available
    }

    fn paint(&mut self, cx: &mut fret_ui_app::PaintCx<'_>) {
        let Some(window) = cx.window else {
            return;
        };
        let snapshot = {
            let Some(svc) = cx.app.global::<DebugInspectorService>() else {
                return;
            };
            if !svc.enabled() {
                return;
            }
            svc.snapshot(window).cloned()
        };
        let Some(snapshot) = snapshot else {
            return;
        };

        self.sync_text(cx, &snapshot);

        for outline in &snapshot.outlines {
            cx.scene.push(fret_core::SceneOp::Quad {
                order: fret_core::DrawOrder(100),
                rect: outline.rect,
                background: fret_core::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                border: fret_core::Edges::all(Px(2.0)),
                border_color: outline.color,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }

        let Some(blob) = self.text_blob else {
            return;
        };
        let Some(metrics) = self.text_metrics else {
            return;
        };

        let theme = cx.theme().snapshot();
        let pad = theme.metrics.padding_md;
        let w = metrics.size.width + pad + pad;
        let h = metrics.size.height + pad + pad;

        let origin = snapshot
            .cursor
            .map(|p| fret_core::Point::new(p.x + Px(16.0), p.y + Px(16.0)))
            .unwrap_or_else(|| {
                fret_core::Point::new(cx.bounds.origin.x + Px(12.0), cx.bounds.origin.y + Px(12.0))
            });

        let max_x = (cx.bounds.origin.x.0 + cx.bounds.size.width.0 - w.0).max(cx.bounds.origin.x.0);
        let max_y =
            (cx.bounds.origin.y.0 + cx.bounds.size.height.0 - h.0).max(cx.bounds.origin.y.0);
        let x = origin.x.0.min(max_x).max(cx.bounds.origin.x.0);
        let y = origin.y.0.min(max_y).max(cx.bounds.origin.y.0);

        let rect = fret_core::Rect::new(
            fret_core::Point::new(Px(x), Px(y)),
            fret_core::Size::new(w, h),
        );

        cx.scene.push(fret_core::SceneOp::Quad {
            order: fret_core::DrawOrder(101),
            rect,
            background: theme.colors.panel_background,
            border: fret_core::Edges::all(Px(1.0)),
            border_color: theme.colors.panel_border,
            corner_radii: fret_core::Corners::all(theme.metrics.radius_md),
        });

        cx.scene.push(fret_core::SceneOp::Text {
            order: fret_core::DrawOrder(102),
            origin: fret_core::Point::new(
                rect.origin.x + pad,
                rect.origin.y + pad + metrics.baseline,
            ),
            text: blob,
            color: theme.colors.text_primary,
        });
    }
}

pub fn build_demo_ui(
    window: AppWindowId,
    kind: DemoUiKind,
    config: DemoUiConfig,
    inspector_edit_buffer: Model<String>,
    viewport_tools: Model<ViewportToolManager>,
) -> (UiTree, DemoLayers) {
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(Stack::new());
    ui.set_root(root);

    let bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Surface, 1.0));
    ui.add_child(root, bg);

    let dock = match kind {
        DemoUiKind::Main => {
            let frame = ui.create_node(HeaderBody::auto());
            ui.add_child(root, frame);

            let header = ui.create_node(Column::new());
            ui.add_child(frame, header);

            let menu_bar = fret_app::MenuBar {
                menus: vec![
                    fret_app::Menu {
                        title: Arc::<str>::from("File"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.new"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.save"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("scene.save_as"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Submenu {
                                title: Arc::<str>::from("Layout"),
                                when: None,
                                items: vec![
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.reset_default",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Separator,
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.preset.save_last",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "dock.layout.preset.load_last",
                                        ),
                                        when: None,
                                    },
                                ],
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("command_palette.toggle"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("project.refresh"),
                                when: None,
                            },
                        ],
                    },
                    fret_app::Menu {
                        title: Arc::<str>::from("Edit"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("edit.undo"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("edit.redo"),
                                when: None,
                            },
                        ],
                    },
                    fret_app::Menu {
                        title: Arc::<str>::from("View"),
                        items: vec![
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.select"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.move"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("viewport.tool.rotate"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("debug.hud.toggle"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("debug.inspector.toggle"),
                                when: None,
                            },
                            fret_app::MenuItem::Command {
                                command: fret_app::CommandId::from("debug.semantics.open"),
                                when: None,
                            },
                            fret_app::MenuItem::Separator,
                            fret_app::MenuItem::Submenu {
                                title: Arc::<str>::from("Theme"),
                                when: None,
                                items: vec![
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.fret_default_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.hardhacker_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from(
                                            "theme.set.godot_default_dark",
                                        ),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Separator,
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from("theme.reload"),
                                        when: None,
                                    },
                                    fret_app::MenuItem::Command {
                                        command: fret_app::CommandId::from("theme.reset_override"),
                                        when: None,
                                    },
                                ],
                            },
                        ],
                    },
                ],
            };

            let menu_bar_node = ui.create_node(AppMenuBar::new(menu_bar));
            ui.add_child(header, menu_bar_node);

            let top_bar = ui.create_node(Bar::new(PanelThemeBackground::Panel, 1.0));
            ui.add_child(header, top_bar);

            let top_bar_row = ui.create_node(Row::new());
            ui.add_child(top_bar, top_bar_row);

            let toolbar = ui.create_node(DemoToolbar::new(viewport_tools));
            ui.add_child(top_bar_row, toolbar);

            let status = ui.create_node(DemoTopBarStatus::new());
            ui.add_child(top_bar_row, status);

            let split = ui.create_node(Split::new(Axis::Horizontal, config.split_fraction));
            ui.add_child(frame, split);

            let dock = ui.create_node(DockSpace::new(window));
            ui.add_child(split, dock);

            let scroll = ui.create_node(Scroll::new());
            ui.add_child(split, scroll);

            let column = ui.create_node(Column::new().with_padding(Px(10.0)).with_spacing(Px(8.0)));
            ui.add_child(scroll, column);

            let dnd_probe = ui.create_node(DndProbe::new());
            ui.add_child(column, dnd_probe);

            let text_header =
                ui.create_node(Text::new("Text MVP (labels + single-line TextInput)"));
            ui.add_child(column, text_header);

            let text_input =
                ui.create_node(TextInput::new().with_text("Click here, then type (IME supported)"));
            ui.add_child(column, text_input);

            let text_input2 = ui
                .create_node(TextInput::new().with_text("Another TextInput (Tab to switch focus)"));
            ui.add_child(column, text_input2);

            let ime_probe = ui.create_node(ImeProbe::new());
            ui.add_child(column, ime_probe);

            let multiline_header = ui.create_node(Text::new(
                "Multiline MVP (wrap + hit test + caret rect + selection rects)",
            ));
            ui.add_child(column, multiline_header);

            let multiline = ui.create_node(
                TextArea::new(
                    "Multiline text: click/drag to place caret and select.\n\
This is wrapped text (TextWrap::Word) and exercises:\n\
- TextService::hit_test_point\n\
- TextService::caret_rect\n\
- TextService::selection_rects\n\
\n\
Goal: foundation for Console/Inspector/code editor.",
                )
                .with_min_height(Px(220.0)),
            );
            ui.add_child(column, multiline);

            let editor_header = ui.create_node(Text::new(
                "Editor Shell MVP (Hierarchy → Inspector) is mounted into DockSpace panels",
            ));
            ui.add_child(column, editor_header);

            let list_header = ui.create_node(Text::new(
                "VirtualList MVP (Hierarchy/Project-scale list: scroll + selection + virtualization)",
            ));
            ui.add_child(column, list_header);

            let list_panel = ui.create_node(FixedPanel::new(
                Px(260.0),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            ));
            ui.add_child(column, list_panel);

            let list = ui.create_node(VirtualList::new(LazyEntityList { count: 100_000 }));
            ui.add_child(list_panel, list);

            let elements_demo = ui.create_node(ElementsMvp2Demo::new());
            ui.add_child(column, elements_demo);

            dock
        }
        DemoUiKind::DockFloating => {
            let dock = ui.create_node(DockSpace::new(window));
            ui.add_child(root, dock);
            dock
        }
    };

    let modal_root = ui.create_node(CenteredOverlayLayout::new(Px(520.0), Px(170.0)));
    let modal = ui.push_overlay_root(modal_root, true);
    ui.set_layer_visible(modal, false);

    let modal_backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("unsaved_dialog.cancel"),
    ));
    ui.add_child(modal_root, modal_backdrop);

    let modal_panel = ui.create_node(Stack::new());
    ui.add_child(modal_root, modal_panel);

    let modal_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 1.0));
    ui.add_child(modal_panel, modal_bg);

    let modal_col = ui.create_node(Column::new().with_padding(Px(14.0)).with_spacing(Px(10.0)));
    ui.add_child(modal_panel, modal_col);

    let modal_title = ui.create_node(Text::new("Unsaved changes"));
    ui.add_child(modal_col, modal_title);

    let modal_msg = ui.create_node(Text::new(
        "The current scene has unsaved changes.\nDo you want to save before continuing?",
    ));
    ui.add_child(modal_col, modal_msg);

    let modal_actions = ui.create_node(Toolbar::new(vec![
        ToolbarItem::new("Save", "unsaved_dialog.save"),
        ToolbarItem::new("Don't Save", "unsaved_dialog.discard"),
        ToolbarItem::new("Cancel", "unsaved_dialog.cancel"),
    ]));
    ui.add_child(modal_col, modal_actions);

    let dnd_root = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 0.22));
    let external_dnd = ui.push_overlay_root_ex(dnd_root, false, false);
    ui.set_layer_visible(external_dnd, false);

    let debug_root =
        ui.create_node(CornerOverlayLayout::top_left(Px(360.0), Px(140.0)).with_margin(Px(12.0)));
    let debug_hud = ui.push_overlay_root_ex(debug_root, false, false);
    ui.set_layer_visible(debug_hud, false);

    let debug_panel = ui.create_node(DebugHudPanel::new());
    ui.add_child(debug_root, debug_panel);

    let palette_root =
        ui.create_node(OverlayPanelLayout::new(Px(640.0), Px(360.0)).with_top(Px(64.0)));
    let command_palette = ui.push_overlay_root(palette_root, true);
    ui.set_layer_visible(command_palette, false);

    let backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("command_palette.close"),
    ));
    ui.add_child(palette_root, backdrop);

    let command_palette_node = ui.create_node(CommandPalette::new());
    ui.add_child(palette_root, command_palette_node);

    let context_menu_node = ui.create_node(ContextMenu::new());
    let context_menu = ui.push_overlay_root(context_menu_node, true);
    ui.set_layer_visible(context_menu, false);

    let popover_node = ui.create_node(Popover::new());
    let popover = ui.push_overlay_root(popover_node, true);
    ui.set_layer_visible(popover, false);

    let inspector_root = ui.create_node(InspectorEditLayout::new(Px(420.0), Px(110.0)));
    let inspector_edit = ui.push_overlay_root(inspector_root, true);
    ui.set_layer_visible(inspector_edit, false);

    let inspector_backdrop = ui.create_node(OverlayBackdrop::new(
        PanelThemeBackground::Surface,
        0.55,
        fret_app::CommandId::from("inspector_edit.commit"),
    ));
    ui.add_child(inspector_root, inspector_backdrop);

    let inspector_panel = ui.create_node(Stack::new());
    ui.add_child(inspector_root, inspector_panel);

    let inspector_panel_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 1.0));
    ui.add_child(inspector_panel, inspector_panel_bg);

    let inspector_column =
        ui.create_node(Column::new().with_padding(Px(12.0)).with_spacing(Px(8.0)));
    ui.add_child(inspector_panel, inspector_column);

    let inspector_hint = ui.create_node(InspectorEditHint::new(window));
    ui.add_child(inspector_column, inspector_hint);

    let inspector_edit_input_node = ui.create_node(
        BoundTextInput::new(inspector_edit_buffer)
            .with_submit_command(fret_app::CommandId::from("inspector_edit.commit"))
            .with_cancel_command(fret_app::CommandId::from("inspector_edit.close")),
    );
    ui.add_child(inspector_column, inspector_edit_input_node);

    let inspector_overlay_root = ui.create_node(Stack::new());
    let debug_inspector = ui.push_overlay_root_ex(inspector_overlay_root, false, false);
    ui.set_layer_visible(debug_inspector, false);

    let inspector_overlay = ui.create_node(DebugInspectorOverlay::new());
    ui.add_child(inspector_overlay_root, inspector_overlay);

    (
        ui,
        DemoLayers {
            modal,
            external_dnd,
            command_palette,
            command_palette_node,
            popover,
            popover_node,
            context_menu,
            context_menu_node,
            inspector_edit,
            inspector_edit_input_node,
            debug_hud,
            debug_inspector,
            dockspace_node: dock,
        },
    )
}
