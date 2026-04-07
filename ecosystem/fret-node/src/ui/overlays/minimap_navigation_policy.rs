use fret_runtime::Model;
use fret_ui::UiHost;

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::NodeGraphSetViewportOptions;
use crate::ui::controller::NodeGraphController;

/// Navigation wiring knobs for the minimap overlay.
///
/// This is intentionally policy-light: it only affects how viewport updates are emitted.
#[derive(Clone)]
pub enum NodeGraphMiniMapNavigationBinding {
    /// Uses the overlay's default behavior (updates `NodeGraphViewState`, and `NodeGraphStore` when attached).
    Default,
    /// Disables navigation (no viewport updates).
    Disabled,
    /// Routes viewport updates through `NodeGraphController`.
    Controller(NodeGraphController),
}

#[derive(Clone)]
pub struct NodeGraphMiniMapBindings {
    pub navigation: NodeGraphMiniMapNavigationBinding,
}

impl Default for NodeGraphMiniMapBindings {
    fn default() -> Self {
        Self {
            navigation: NodeGraphMiniMapNavigationBinding::Default,
        }
    }
}

pub(super) fn apply_minimap_viewport_update<H: UiHost>(
    host: &mut H,
    navigation: &NodeGraphMiniMapNavigationBinding,
    view_state: &Model<NodeGraphViewState>,
    store: Option<&Model<NodeGraphStore>>,
    pan: CanvasPoint,
    zoom: f32,
) {
    let zoom = normalize_minimap_navigation_zoom(zoom);

    match navigation {
        NodeGraphMiniMapNavigationBinding::Disabled => {}
        NodeGraphMiniMapNavigationBinding::Controller(controller) => {
            if controller.set_viewport_with_options(
                host,
                pan,
                zoom,
                NodeGraphSetViewportOptions::default(),
            ) {
                let _ = controller.sync_view_state_model_from_store(host, view_state);
            }
        }
        NodeGraphMiniMapNavigationBinding::Default => {
            let _ = view_state.update(host, |state, _cx| {
                state.pan = pan;
                state.zoom = zoom;
            });

            if let Some(store) = store {
                let _ = store.update(host, |store, _cx| {
                    store.set_viewport(pan, zoom);
                });
            }
        }
    }
}

pub(super) fn normalize_minimap_navigation_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NodeGraphMiniMapBindings, NodeGraphMiniMapNavigationBinding,
        normalize_minimap_navigation_zoom,
    };

    #[test]
    fn normalize_minimap_navigation_zoom_falls_back_for_non_positive_or_non_finite_values() {
        assert_eq!(normalize_minimap_navigation_zoom(2.5), 2.5);
        assert_eq!(normalize_minimap_navigation_zoom(0.0), 1.0);
        assert_eq!(normalize_minimap_navigation_zoom(-3.0), 1.0);
        assert_eq!(normalize_minimap_navigation_zoom(f32::NAN), 1.0);
        assert_eq!(normalize_minimap_navigation_zoom(f32::INFINITY), 1.0);
    }

    #[test]
    fn minimap_bindings_default_to_store_first_navigation() {
        let bindings = NodeGraphMiniMapBindings::default();
        assert!(matches!(
            bindings.navigation,
            NodeGraphMiniMapNavigationBinding::Default
        ));
    }
}
