use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use fret_core::{AppWindowId, Edges, NodeId, PanelKey, PanelKind, Rect, UiServices};
use fret_ui::{UiHost, UiTree, declarative};

use super::DockManager;
use super::services::DockPanelContentService;

fn fill_layout() -> fret_ui::element::LayoutStyle {
    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Fill;
    layout
}

/// Render a dock panel root wrapped in a view-cache boundary.
///
/// This is a mechanism-level helper intended for GPUI-style view caching experiments.
pub fn render_cached_panel_root<H: UiHost + 'static>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, H>) -> Vec<fret_ui::element::AnyElement>,
) -> NodeId {
    declarative::render_root(ui, app, services, window, bounds, root_name, |cx| {
        vec![cx.view_cache(
            fret_ui::element::ViewCacheProps {
                layout: fill_layout(),
                contained_layout: true,
                ..Default::default()
            },
            f,
        )]
    })
}

/// Contribution-level dock panel seam keyed by stable `PanelKind`.
///
/// This is intended for reusable panel packs. The app still owns the final registry/service that
/// aggregates panel contributions for a window.
pub trait DockPanelFactory<H: UiHost>: Send + Sync + 'static {
    /// Stable kind handled by this contribution.
    fn panel_kind(&self) -> PanelKind;

    /// Build or render the UI root for the requested panel instance.
    ///
    /// The default story is one factory per `PanelKind`, but factories receive the full
    /// `PanelKey` so singleton and multi-instance panels can share the same contribution seam.
    ///
    /// Return `None` for panels that intentionally have no UI node (for example pure viewport
    /// panels).
    fn build_panel(&self, panel: &PanelKey, cx: &mut DockPanelFactoryCx<'_, H>) -> Option<NodeId>;
}

/// Build-time context passed to `DockPanelFactory`.
pub struct DockPanelFactoryCx<'a, H: UiHost> {
    pub ui: &'a mut UiTree<H>,
    pub app: &'a mut H,
    pub services: &'a mut dyn UiServices,
    pub window: AppWindowId,
    pub bounds: Rect,
}

impl<'a, H: UiHost + 'static> DockPanelFactoryCx<'a, H> {
    /// Convenience helper that preserves the existing retained/root caching story for panel UI.
    pub fn render_cached_panel_root(
        &mut self,
        root_name: &str,
        f: impl FnOnce(&mut fret_ui::ElementContext<'_, H>) -> Vec<fret_ui::element::AnyElement>,
    ) -> NodeId {
        render_cached_panel_root(
            self.ui,
            self.app,
            self.services,
            self.window,
            self.bounds,
            root_name,
            f,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateDockPanelKindError {
    pub kind: PanelKind,
}

impl fmt::Display for DuplicateDockPanelKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "duplicate dock panel factory registration for panel kind `{}`",
            self.kind.0
        )
    }
}

impl std::error::Error for DuplicateDockPanelKindError {}

/// App-owned aggregation point for reusable dock panel factories.
pub struct DockPanelRegistryBuilder<H: UiHost> {
    factories: HashMap<PanelKind, Arc<dyn DockPanelFactory<H>>>,
}

impl<H: UiHost + 'static> Default for DockPanelRegistryBuilder<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost + 'static> DockPanelRegistryBuilder<H> {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn try_register<F>(&mut self, factory: F) -> Result<&mut Self, DuplicateDockPanelKindError>
    where
        F: DockPanelFactory<H>,
    {
        self.try_register_arc(Arc::new(factory))
    }

    pub fn register<F>(&mut self, factory: F) -> &mut Self
    where
        F: DockPanelFactory<H>,
    {
        self.try_register(factory)
            .expect("duplicate dock panel kind registered in DockPanelRegistryBuilder")
    }

    pub fn try_register_arc(
        &mut self,
        factory: Arc<dyn DockPanelFactory<H>>,
    ) -> Result<&mut Self, DuplicateDockPanelKindError> {
        let kind = factory.panel_kind();
        if self.factories.contains_key(&kind) {
            return Err(DuplicateDockPanelKindError { kind });
        }
        self.factories.insert(kind, factory);
        Ok(self)
    }

    pub fn register_arc(&mut self, factory: Arc<dyn DockPanelFactory<H>>) -> &mut Self {
        self.try_register_arc(factory)
            .expect("duplicate dock panel kind registered in DockPanelRegistryBuilder")
    }

    pub fn build(self) -> DockPanelFactoryRegistry<H> {
        DockPanelFactoryRegistry {
            factories: self.factories,
        }
    }

    pub fn build_arc(self) -> Arc<dyn DockPanelRegistry<H>> {
        Arc::new(self.build())
    }
}

/// `DockPanelRegistry` implementation backed by contribution-level `DockPanelFactory` values.
pub struct DockPanelFactoryRegistry<H: UiHost> {
    factories: HashMap<PanelKind, Arc<dyn DockPanelFactory<H>>>,
}

impl<H: UiHost + 'static> DockPanelRegistry<H> for DockPanelFactoryRegistry<H> {
    fn render_panel(
        &self,
        ui: &mut UiTree<H>,
        app: &mut H,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &PanelKey,
    ) -> Option<NodeId> {
        let factory = self.factories.get(&panel.kind)?;
        let mut cx = DockPanelFactoryCx {
            ui,
            app,
            services,
            window,
            bounds,
        };
        factory.build_panel(panel, &mut cx)
    }
}

/// App-owned registry that can render panel UI content for docking.
///
/// The docking graph (`DockGraph`) is the source of truth for which panels exist in a window.
/// The driver should render/bind panel roots every frame (before `layout_all`/`paint_all`) to
/// avoid "programmatic close" flicker where a panel disappears from submission without a `DockOp`.
pub trait DockPanelRegistry<H: UiHost>: Send + Sync + 'static {
    /// Render the panel's UI root for this frame and return the root node to be hosted under the
    /// `DockSpace` node.
    ///
    /// Return `None` for panels that intentionally have no UI node (e.g. pure viewport panels).
    fn render_panel(
        &self,
        ui: &mut UiTree<H>,
        app: &mut H,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &PanelKey,
    ) -> Option<NodeId>;
}

/// Stores a dock panel registry in the host globals.
pub struct DockPanelRegistryService<H: UiHost> {
    registry: Option<Arc<dyn DockPanelRegistry<H>>>,
}

impl<H: UiHost> Default for DockPanelRegistryService<H> {
    fn default() -> Self {
        Self { registry: None }
    }
}

impl<H: UiHost> DockPanelRegistryService<H> {
    pub fn set(&mut self, registry: Arc<dyn DockPanelRegistry<H>>) {
        self.registry = Some(registry);
    }

    pub fn clear(&mut self) {
        self.registry = None;
    }

    pub fn registry(&self) -> Option<Arc<dyn DockPanelRegistry<H>>> {
        self.registry.clone()
    }
}

fn missing_panel_root_name(panel: &PanelKey) -> String {
    match panel.instance.as_ref() {
        Some(instance) => format!("dock.panel.missing.{}:{}", panel.kind.0, instance),
        None => format!("dock.panel.missing.{}", panel.kind.0),
    }
}

fn render_missing_panel<H: UiHost + 'static>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    panel: &PanelKey,
) -> NodeId {
    let root_name = missing_panel_root_name(panel);
    render_cached_panel_root(ui, app, services, window, bounds, &root_name, |cx| {
        let theme = cx.theme().snapshot();
        let padding = theme.metric_token("metric.padding.md");
        let background = theme.color_token("background");
        vec![cx.container(
            fret_ui::element::ContainerProps {
                layout: fill_layout(),
                padding: Edges::all(padding).into(),
                background: Some(background),
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.text(format!("Missing panel UI: {}", panel.kind.0)),
                    cx.text("This panel exists in the dock graph but has no registered UI."),
                ]
            },
        )]
    })
}

fn sort_panel_keys(a: &PanelKey, b: &PanelKey) -> std::cmp::Ordering {
    a.kind.0.cmp(&b.kind.0).then_with(|| {
        a.instance
            .as_deref()
            .unwrap_or("")
            .cmp(b.instance.as_deref().unwrap_or(""))
    })
}

/// Render and bind dock panel UI nodes for a window.
///
/// Call this once per frame **before** `UiTree::layout_all`/`paint_all`.
///
/// This helper:
/// - uses the dock graph (`DockManager`) as the source of truth for which panels exist,
/// - renders panel roots via the installed `DockPanelRegistryService`,
/// - falls back to a generic placeholder if a non-viewport panel has no UI implementation,
/// - skips viewport panels unless the registry returns a UI node (viewport panels may be "pure"
///   render targets with no tree nodes),
/// - updates `DockPanelContentService` for the window,
/// - sets the `DockSpace` node children to the rendered panel roots.
pub fn render_and_bind_dock_panels<H: UiHost + 'static>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    dock_space: NodeId,
) {
    let panels: Vec<(PanelKey, bool)> = {
        let Some(dock) = app.global::<DockManager>() else {
            return;
        };

        let mut panels = dock.graph.collect_panels_in_window(window);
        panels.sort_by(sort_panel_keys);
        panels.dedup();

        panels
            .into_iter()
            .map(|panel| {
                let is_viewport_panel = dock.panel(&panel).and_then(|p| p.viewport).is_some();
                (panel, is_viewport_panel)
            })
            .collect()
    };

    let registry = app
        .global::<DockPanelRegistryService<H>>()
        .and_then(|svc| svc.registry());

    let mut rendered: Vec<(PanelKey, NodeId)> = Vec::new();

    for (panel, is_viewport_panel) in panels {
        let node = registry
            .as_ref()
            .and_then(|registry| registry.render_panel(ui, app, services, window, bounds, &panel));

        let node = match (is_viewport_panel, node) {
            (_, Some(node)) => node,
            (true, None) => continue,
            (false, None) => render_missing_panel(ui, app, services, window, bounds, &panel),
        };

        rendered.push((panel, node));
    }

    app.with_global_mut(DockPanelContentService::default, |svc, _app| {
        svc.replace_window(window, rendered.iter().map(|(k, v)| (k.clone(), *v)));
    });

    let children: Vec<NodeId> = rendered.into_iter().map(|(_, node)| node).collect();
    ui.set_children(dock_space, children);
}
