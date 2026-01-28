use std::sync::Arc;

use fret_core::{AppWindowId, Edges, NodeId, PanelKey, Rect, UiServices};
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
        let padding = theme.metric_required("metric.padding.md");
        let background = theme.color_required("background");
        vec![cx.container(
            fret_ui::element::ContainerProps {
                layout: fill_layout(),
                padding: Edges::all(padding),
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

/// Render and bind all non-viewport panel UI nodes for a window.
///
/// Call this once per frame **before** `UiTree::layout_all`/`paint_all`.
///
/// This helper:
/// - uses the dock graph (`DockManager`) as the source of truth for which panels exist,
/// - renders panel roots via the installed `DockPanelRegistryService`,
/// - falls back to a generic placeholder if a non-viewport panel has no UI implementation,
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
        if is_viewport_panel {
            continue;
        }

        let node = match registry.as_ref() {
            Some(registry) => registry.render_panel(ui, app, services, window, bounds, &panel),
            None => None,
        }
        .unwrap_or_else(|| render_missing_panel(ui, app, services, window, bounds, &panel));

        rendered.push((panel, node));
    }

    app.with_global_mut(DockPanelContentService::default, |svc, _app| {
        svc.replace_window(window, rendered.iter().map(|(k, v)| (k.clone(), *v)));
    });

    let children: Vec<NodeId> = rendered.into_iter().map(|(_, node)| node).collect();
    ui.set_children(dock_space, children);
}
