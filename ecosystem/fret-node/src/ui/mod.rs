//! Fret UI integration for the node graph editor.
//!
//! This module is behind the `fret-ui` feature.
//!
//! The supported authoring surface is binding/controller/declarative composition; legacy retained
//! canvas widget authoring has been removed from this crate.

pub mod binding;
mod canvas;
pub mod commands;
pub mod controller;
pub mod declarative;
pub mod edge_types;
pub mod geometry_overrides;
pub mod internals;
pub mod measured;
pub mod paint_overrides;
mod portal_commands;
pub mod portal_layout;
pub mod presenter;
pub mod presets;
pub mod registry;
pub mod skin;
pub mod style;
mod viewport_helper;
mod viewport_options;

mod editors;
mod overlays;
mod screen_space_placement;

pub use binding::NodeGraphSurfaceBinding;
pub use canvas::{NodeResizeHandle, NodeResizeHandleSet};
pub use commands::register_node_graph_commands;
pub use controller::{
    NodeGraphController, NodeGraphControllerError, NodeGraphEdgeUpdate,
    NodeGraphNodeConnectionsQuery, NodeGraphNodeUpdate, NodeGraphPortConnectionsQuery,
};
pub use declarative::{
    NodeGraphDeclarativeEdgeLabelRenderer, NodeGraphDeclarativeInsertNodePickerCandidateProvider,
    NodeGraphDeclarativeInsertNodePickerCandidateProviderRef,
    NodeGraphDeclarativeInsertNodePickerOpenOutcome,
    NodeGraphDeclarativeInsertNodePickerOverlayBinding,
    NodeGraphDeclarativeInsertNodePickerPlanError, NodeGraphDeclarativeInsertNodePickerRequest,
    NodeGraphDeclarativeInsertNodePickerSession, NodeGraphDeclarativeInsertNodePickerState,
    NodeGraphDeclarativeInsertNodePickerStateRef, NodeGraphDeclarativeInteractionContext,
    NodeGraphDeclarativeInteractionHook, NodeGraphDeclarativeInteractionHookRef,
    NodeGraphDeclarativeInteractionOutcome, NodeGraphDeclarativePortalCommandHandler,
    NodeGraphDeclarativePortalCommandHandlerRef, NodeGraphDeclarativePortalRenderer,
    NodeGraphDeclarativeSurfaceRenderers, NodeGraphDiagnosticsConfig,
    NodeGraphEdgeLabelHitTestMode, NodeGraphEdgeLabelLayout, NodeGraphSurfaceProps,
    NodeGraphVisibleSubsetPortalConfig, PortalCommandOutcome, PortalTextCommand,
    PortalTextStepMode, node_graph_surface, node_graph_surface_in,
    node_graph_surface_with_edge_label_renderer, node_graph_surface_with_edge_label_renderer_in,
    node_graph_surface_with_insert_node_picker, node_graph_surface_with_insert_node_picker_in,
    node_graph_surface_with_portal_renderer, node_graph_surface_with_portal_renderer_in,
    node_graph_surface_with_renderers, node_graph_surface_with_renderers_in,
    parse_portal_text_command, portal_cancel_text_command, portal_step_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
pub use edge_types::{
    EdgeCustomPath, EdgePathInput, EdgeTypeKey, NodeGraphEdgeTypes, NodeGraphEdgeTypesRef,
};
pub use editors::{
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
};
pub use geometry_overrides::{
    EdgeGeometryOverrideV1, NodeGeometryOverrideV1, NodeGraphGeometryOverrides,
    NodeGraphGeometryOverridesMap, NodeGraphGeometryOverridesRef,
};
pub use internals::{
    NodeGraphCanvasTransform, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};
pub use measured::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter,
};
pub use paint_overrides::{
    EdgePaintOverrideV1, NodeGraphPaintOverrides, NodeGraphPaintOverridesMap,
    NodeGraphPaintOverridesRef, NodePaintOverrideV1,
};
pub use portal_layout::NodeGraphPortalNodeLayout;
pub use presenter::{
    DefaultNodeGraphPresenter, EdgeMarker, EdgeMarkerKind, EdgeRenderHint, EdgeRouteKind,
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter,
    NodeResizeConstraintsPx, PortAnchorHint, RegistryNodeGraphPresenter,
};
pub use presets::{NodeGraphPresetFamily, NodeGraphPresetSkinV1};
pub use registry::{NodeGraphNodeRenderer, NodeGraphNodeTypes};
pub use skin::{
    CanvasChromeHint, EdgeChromeHint, InteractionChromeHint, NodeChromeHint, NodeGraphSkin,
    NodeGraphSkinRef, NodeRingHint, NodeShadowHint, NoopNodeGraphSkin, PortChromeHint,
    PortShapeHint, WireGlowHint, WireHighlightHint, WireOutlineHint,
};
pub use style::{NodeGraphColorMode, NodeGraphStyle};
pub use viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};
