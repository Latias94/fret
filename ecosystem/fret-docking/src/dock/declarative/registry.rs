use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, Edges, NodeId, PanelKey};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::{ElementContext, UiHost};

use super::super::manager::DockManager;
use super::super::services::DockPanelContentService;

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

/// Options for declarative docking host authoring.
///
/// This is the public, managed-surface-backed entry point for panel-root hosting. It intentionally
/// does not expose the retained `Widget` API. The legacy retained dock-space node remains for the
/// interaction-heavy adapter until the M1 exit work moves event/command/chrome policy onto the
/// declarative path.
#[derive(Debug, Clone)]
pub struct DockSpaceElementOptions {
    /// Layout for the managed docking host element.
    pub layout: LayoutStyle,
    /// Optional diagnostics / automation test id stamped onto the host element.
    pub test_id: Option<&'static str>,
    /// Allow dragging from an already multi-window dock graph to request another OS-window tear-off.
    pub allow_multi_window_tear_off: bool,
}

impl Default for DockSpaceElementOptions {
    fn default() -> Self {
        Self {
            layout: fill_layout(),
            test_id: None,
            allow_multi_window_tear_off: false,
        }
    }
}

/// A declarative panel root bound to a dock graph panel key.
pub struct DockPanelElement {
    pub panel: PanelKey,
    pub element: AnyElement,
}

impl DockPanelElement {
    pub fn new(panel: PanelKey, element: AnyElement) -> Self {
        Self { panel, element }
    }
}

/// Convenience constructor for [`DockPanelElement`].
pub fn dock_panel_element(panel: PanelKey, element: AnyElement) -> DockPanelElement {
    DockPanelElement::new(panel, element)
}

/// App-owned declarative registry for panel UI content.
///
/// This returns declarative panel roots owned by the app, keeping panel UI authoring outside the
/// docking mechanism while still letting the dock host place active roots.
pub trait DockPanelElementRegistry<H: UiHost>: Send + Sync + 'static {
    fn render_panel(
        &self,
        cx: &mut ElementContext<'_, H>,
        window: AppWindowId,
        panel: &PanelKey,
    ) -> Option<AnyElement>;
}

/// Stores a declarative dock panel registry in the host globals.
pub struct DockPanelElementRegistryService<H: UiHost> {
    registry: Option<Arc<dyn DockPanelElementRegistry<H>>>,
}

impl<H: UiHost> Default for DockPanelElementRegistryService<H> {
    fn default() -> Self {
        Self { registry: None }
    }
}

impl<H: UiHost> DockPanelElementRegistryService<H> {
    pub fn set(&mut self, registry: Arc<dyn DockPanelElementRegistry<H>>) {
        self.registry = Some(registry);
    }

    pub fn clear(&mut self) {
        self.registry = None;
    }

    pub fn registry(&self) -> Option<Arc<dyn DockPanelElementRegistry<H>>> {
        self.registry.clone()
    }
}

fn sort_panel_keys(a: &PanelKey, b: &PanelKey) -> std::cmp::Ordering {
    a.kind.0.cmp(&b.kind.0).then_with(|| {
        a.instance
            .as_deref()
            .unwrap_or("")
            .cmp(b.instance.as_deref().unwrap_or(""))
    })
}

pub(super) fn collect_panels_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Vec<(PanelKey, bool)> {
    let Some(dock) = app.global::<DockManager>() else {
        return Vec::new();
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
}

pub(super) fn missing_panel_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    panel: &PanelKey,
) -> AnyElement {
    let theme = cx.theme().snapshot();
    let padding = theme.metric_token("metric.padding.md");
    let background = theme.color_token("background");
    cx.container(
        ContainerProps {
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
    )
}

pub(super) fn bind_panel_children<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    panels: &[PanelKey],
    children: &[NodeId],
) {
    app.with_global_mut(DockPanelContentService::default, |content, _app| {
        content.replace_window(window, panels.iter().cloned().zip(children.iter().copied()));
    });
}

pub(super) fn panel_nodes_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> HashMap<PanelKey, NodeId> {
    app.global::<DockPanelContentService>()
        .map(|content| content.panel_nodes(window).into_iter().collect())
        .unwrap_or_default()
}
