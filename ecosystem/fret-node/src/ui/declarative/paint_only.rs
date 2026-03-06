use std::sync::Arc;

use fret_canvas::view::{
    DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D, screen_rect_to_canvas_rect,
    wheel_zoom_factor,
};
use fret_canvas::wires as canvas_wires;
use fret_core::scene::{
    ColorSpace, DashPatternV1, GradientStop, LinearGradient, MAX_STOPS, Paint, PaintBindingV1,
    PaintEvalSpaceV1, TileMode,
};
use fret_core::{
    Color, DrawOrder, MouseButton, PathCommand, PathStyle, Point, Px, Rect, StrokeCapV1,
    StrokeJoinV1, StrokeStyleV2,
};
use fret_runtime::Model;
use fret_ui::action::{
    OnKeyDown, OnPinchGesture, OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel,
};
use fret_ui::canvas::{CanvasKey, CanvasPainter};
use fret_ui::element::{
    AnyElement, CanvasProps, ColumnProps, ContainerProps, LayoutQueryRegionProps, LayoutStyle,
    Length, PointerRegionProps, PositionStyle, SemanticsDecoration, SemanticsProps, SpacingEdges,
    SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::core::Graph;
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction, graph_diff, normalize_transaction};
use crate::runtime::store::NodeGraphStore;
use crate::ui::NodeGraphController;
use crate::ui::canvas::{CanvasGeometry, CanvasSpatialDerived};
use crate::ui::declarative::view_reducer::{
    apply_fit_view_to_canvas_rect, apply_pan_by_screen_delta, apply_zoom_about_screen_point,
    view_from_state,
};
use crate::ui::geometry_overrides::NodeGraphGeometryOverridesRef;
use crate::ui::paint_overrides::{NodeGraphPaintOverridesMap, NodeGraphPaintOverridesRef};
use crate::ui::presenter::DefaultNodeGraphPresenter;
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Default, Clone)]
struct PortalBoundsStore {
    /// Last-known bounds for portal node subtrees, mapped into canvas space under the current view.
    nodes_canvas_bounds: std::collections::BTreeMap<crate::core::NodeId, Rect>,
    /// Counter for diagnostics gates (fit-to-portals triggered).
    fit_to_portals_count: u64,
    /// Diagnostics-only: when true, a Ctrl+9 fit-to-portals request is armed and will be applied
    /// once portal bounds arrive via `LayoutQueryRegion` (frame-lagged by contract).
    pending_fit_to_portals: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct PortalDebugFlags {
    /// Diagnostics-only: when true, disable portal hosting and clear `PortalBoundsStore` so overlay
    /// consumers can exercise their fallback paths.
    disable_portals: bool,
}

#[derive(Debug, Default, Clone)]
struct HoverAnchorStore {
    /// Last-known hovered node id (paint-only).
    hovered_id: Option<crate::core::NodeId>,
    /// Best-effort hovered node bounds in canvas space.
    ///
    /// This is independent of portal hosting caps so hover-driven overlays remain stable even when
    /// portals are throttled.
    hovered_canvas_bounds: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
pub struct NodeGraphWheelZoomConfig {
    pub base: f32,
    pub step: f32,
    pub speed: f32,
}

impl Default for NodeGraphWheelZoomConfig {
    fn default() -> Self {
        Self {
            base: DEFAULT_WHEEL_ZOOM_BASE,
            step: DEFAULT_WHEEL_ZOOM_STEP,
            speed: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct NodeGraphSurfacePaintOnlyProps {
    pub graph: Model<Graph>,
    pub view_state: Model<NodeGraphViewState>,
    pub controller: Option<NodeGraphController>,
    pub store: Option<Model<NodeGraphStore>>,

    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,

    /// Optional geometry overrides for UI-only per-entity sizing/hit-testing.
    ///
    /// Changes must bump `revision()` on the provider (see `ui/geometry_overrides.rs`).
    pub geometry_overrides: Option<NodeGraphGeometryOverridesRef>,

    /// Optional paint-only overrides for UI-only per-node/per-edge styling.
    ///
    /// Changes must bump `revision()` on the provider (see `ui/paint_overrides.rs`).
    pub paint_overrides: Option<NodeGraphPaintOverridesRef>,

    /// When true, host a lightweight portal layer that renders node labels as normal element
    /// subtrees positioned in screen space (semantic zoom).
    ///
    /// This is intended as an incremental “world layer” bridge:
    /// - paint pass still owns grid + edges (and node chrome),
    /// - portals exercise declarative layout and bounds queries for visible nodes only.
    pub portals_enabled: bool,
    /// Upper bound on the number of portal subtrees hosted for visible nodes.
    pub portal_max_nodes: usize,

    /// Cull margin in screen pixels around the viewport.
    pub cull_margin_screen_px: f32,

    pub pan_button: MouseButton,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub wheel_zoom: NodeGraphWheelZoomConfig,
    pub pinch_zoom_speed: f32,

    pub test_id: Option<Arc<str>>,
}

impl NodeGraphSurfacePaintOnlyProps {
    pub fn new(graph: Model<Graph>, view_state: Model<NodeGraphViewState>) -> Self {
        let mut pointer_region = PointerRegionProps::default();
        pointer_region.layout.size.width = Length::Fill;
        pointer_region.layout.size.height = Length::Fill;

        Self {
            graph,
            view_state,
            controller: None,
            store: None,
            pointer_region,
            canvas: CanvasProps::default(),
            geometry_overrides: None,
            paint_overrides: None,
            portals_enabled: true,
            portal_max_nodes: 32,
            cull_margin_screen_px: 256.0,
            pan_button: MouseButton::Middle,
            min_zoom: 0.05,
            max_zoom: 64.0,
            wheel_zoom: NodeGraphWheelZoomConfig::default(),
            pinch_zoom_speed: 1.0,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DragState {
    button: MouseButton,
    last_pos: Point,
}

#[derive(Debug, Clone)]
struct MarqueeDragState {
    start_screen: Point,
    current_screen: Point,
    active: bool,
    toggle: bool,
    base_selected_nodes: Arc<[crate::core::NodeId]>,
    preview_selected_nodes: Arc<[crate::core::NodeId]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeDragPhase {
    Armed,
    Active,
    Canceled,
}

#[derive(Debug, Clone)]
struct NodeDragState {
    start_screen: Point,
    current_screen: Point,
    phase: NodeDragPhase,
    nodes_sorted: Arc<[crate::core::NodeId]>,
}

impl NodeDragState {
    fn is_armed(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Armed)
    }

    fn is_active(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Active)
    }

    fn is_canceled(&self) -> bool {
        matches!(self.phase, NodeDragPhase::Canceled)
    }

    fn is_live(&self) -> bool {
        !self.is_canceled()
    }

    fn activate(&mut self, current_screen: Point) -> bool {
        if !self.is_armed() {
            return false;
        }
        self.phase = NodeDragPhase::Active;
        self.current_screen = current_screen;
        true
    }

    fn cancel(&mut self) {
        self.phase = NodeDragPhase::Canceled;
    }

    fn update_active_position(&mut self, current_screen: Point) -> bool {
        if !self.is_active() || self.current_screen == current_screen {
            return false;
        }
        self.current_screen = current_screen;
        true
    }
}

#[derive(Debug, Clone)]
struct PendingSelectionState {
    nodes: Arc<[crate::core::NodeId]>,
    clear_edges: bool,
    clear_groups: bool,
}

#[track_caller]
fn use_uncontrolled_model<T: Clone + 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_value: impl FnOnce() -> T,
) -> Model<T> {
    struct UncontrolledModelState<T> {
        model: Option<Model<T>>,
    }

    impl<T> Default for UncontrolledModelState<T> {
        fn default() -> Self {
            Self { model: None }
        }
    }

    let model = cx.with_state(UncontrolledModelState::<T>::default, |st| st.model.clone());
    if let Some(model) = model {
        return model;
    }

    let model = cx.app.models_mut().insert(default_value());
    cx.with_state(UncontrolledModelState::<T>::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

fn mouse_buttons_contains(buttons: fret_core::MouseButtons, button: MouseButton) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

fn stable_hash_u64(seed: u64, value: &impl std::hash::Hash) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
struct GridPaintCacheState {
    /// Last known bounds for the surface (updated from pointer hooks, and optionally from
    /// `last_bounds_for_element`).
    bounds: Rect,
    key: Option<CanvasKey>,
    rebuilds: u64,
    ops: Option<Arc<Vec<fret_core::SceneOp>>>,
}

impl Default for GridPaintCacheState {
    fn default() -> Self {
        Self {
            bounds: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(0.0), Px(0.0)),
            ),
            key: None,
            rebuilds: 0,
            ops: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridPaintCacheKeyV2 {
    bounds_x_q: i32,
    bounds_y_q: i32,
    bounds_w_q: i32,
    bounds_h_q: i32,
    zoom_q: i32,
    ix0: i32,
    ix1: i32,
    iy0: i32,
    iy1: i32,
    bg_r_bits: u32,
    bg_g_bits: u32,
    bg_b_bits: u32,
    bg_a_bits: u32,
    grid_r_bits: u32,
    grid_g_bits: u32,
    grid_b_bits: u32,
    grid_a_bits: u32,
}

fn quantize_f32(value: f32, scale: f32) -> i32 {
    if !value.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return 0;
    }
    (value * scale)
        .round()
        .clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

fn grid_cache_key(bounds: Rect, view: PanZoom2D, style: &NodeGraphStyle) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0);
    let bg = style.paint.background;
    let grid = style.paint.grid_minor_color;

    // Quantize to avoid cache churn due to float noise while remaining sensitive to real changes.
    // - bounds: device-independent UI px (layout space)
    // - zoom: scale factor (unitless)
    // - grid indices: snapped to step boundaries (avoid per-pixel pan churn)
    let step = 64.0f32;
    let (ix0, ix1, iy0, iy1) = if bounds.size.width.0.is_finite()
        && bounds.size.height.0.is_finite()
        && bounds.size.width.0 > 0.0
        && bounds.size.height.0 > 0.0
        && step.is_finite()
        && step > 0.0
    {
        let tl = view.screen_to_canvas(bounds, bounds.origin);
        let br = view.screen_to_canvas(
            bounds,
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0),
                Px(bounds.origin.y.0 + bounds.size.height.0),
            ),
        );
        let x0 = tl.x.0.min(br.x.0);
        let x1 = tl.x.0.max(br.x.0);
        let y0 = tl.y.0.min(br.y.0);
        let y1 = tl.y.0.max(br.y.0);
        if x0.is_finite() && x1.is_finite() && y0.is_finite() && y1.is_finite() {
            (
                (x0 / step).floor() as i32 - 1,
                (x1 / step).ceil() as i32 + 1,
                (y0 / step).floor() as i32 - 1,
                (y1 / step).ceil() as i32 + 1,
            )
        } else {
            (0, 0, 0, 0)
        }
    } else {
        (0, 0, 0, 0)
    };
    let v = GridPaintCacheKeyV2 {
        bounds_x_q: quantize_f32(bounds.origin.x.0, 8.0),
        bounds_y_q: quantize_f32(bounds.origin.y.0, 8.0),
        bounds_w_q: quantize_f32(bounds.size.width.0, 8.0),
        bounds_h_q: quantize_f32(bounds.size.height.0, 8.0),
        zoom_q: quantize_f32(zoom, 4096.0),
        ix0,
        ix1,
        iy0,
        iy1,
        bg_r_bits: bg.r.to_bits(),
        bg_g_bits: bg.g.to_bits(),
        bg_b_bits: bg.b.to_bits(),
        bg_a_bits: bg.a.to_bits(),
        grid_r_bits: grid.r.to_bits(),
        grid_g_bits: grid.g.to_bits(),
        grid_b_bits: grid.b.to_bits(),
        grid_a_bits: grid.a.to_bits(),
    };

    // Namespace the key so future paint-only variants can coexist safely.
    CanvasKey::from_hash(&("fret-node.grid.paint-only.v2", v))
}

#[derive(Debug, Clone)]
struct DerivedGeometryCacheState {
    key: Option<CanvasKey>,
    rebuilds: u64,
    geom: Option<Arc<CanvasGeometry>>,
    index: Option<Arc<CanvasSpatialDerived>>,
}

impl Default for DerivedGeometryCacheState {
    fn default() -> Self {
        Self {
            key: None,
            rebuilds: 0,
            geom: None,
            index: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DerivedGeometryCacheKeyV2 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    geometry_tokens_fingerprint: u64,
    geometry_overrides_rev: u64,
    cell_size_screen_bits: u32,
    min_cell_size_screen_bits: u32,
    edge_aabb_pad_screen_bits: u32,
    edge_interaction_width_bits: u32,
    wire_width_bits: u32,
}

fn derived_geometry_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order: &[crate::core::NodeId],
    interaction: &crate::io::NodeGraphInteractionState,
    style: &NodeGraphStyle,
    geometry_overrides_rev: u64,
    max_edge_interaction_width_override_px: f32,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();

    let tuning = interaction.spatial_index;
    let max_edge_interaction_width_override_px = if max_edge_interaction_width_override_px
        .is_finite()
        && max_edge_interaction_width_override_px >= 0.0
    {
        max_edge_interaction_width_override_px
    } else {
        0.0
    };
    let edge_aabb_pad_screen_px = tuning
        .edge_aabb_pad_screen_px
        .max(interaction.edge_interaction_width)
        .max(max_edge_interaction_width_override_px)
        .max(style.geometry.wire_width)
        .max(0.0);

    let v = DerivedGeometryCacheKeyV2 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash: stable_hash_u64(1, &draw_order),
        geometry_tokens_fingerprint: style.geometry.fingerprint(),
        geometry_overrides_rev,
        cell_size_screen_bits: tuning.cell_size_screen_px.to_bits(),
        min_cell_size_screen_bits: tuning.min_cell_size_screen_px.to_bits(),
        edge_aabb_pad_screen_bits: edge_aabb_pad_screen_px.to_bits(),
        edge_interaction_width_bits: interaction.edge_interaction_width.to_bits(),
        wire_width_bits: style.geometry.wire_width.to_bits(),
    };

    CanvasKey::from_hash(&("fret-node.derived-geometry.paint-only.v2", v))
}

fn build_debug_grid_ops(
    bounds: Rect,
    view: PanZoom2D,
    style: &NodeGraphStyle,
) -> Arc<Vec<fret_core::SceneOp>> {
    let mut ops = Vec::<fret_core::SceneOp>::new();

    // Background fill (debug baseline).
    ops.push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: fret_core::scene::Paint::Solid(style.paint.background).into(),
        border: fret_core::Edges::default(),
        border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::default(),
    });

    let Some(transform) = view.render_transform(bounds) else {
        return Arc::new(ops);
    };

    // Mirror `CanvasPainter::with_transform` behavior: avoid pushing invalid or identity transforms.
    let push_transform = {
        let is_finite = transform.a.is_finite()
            && transform.b.is_finite()
            && transform.c.is_finite()
            && transform.d.is_finite()
            && transform.tx.is_finite()
            && transform.ty.is_finite();
        is_finite && transform != fret_core::Transform2D::IDENTITY
    };

    if push_transform {
        ops.push(fret_core::SceneOp::PushTransform { transform });
    }

    // Draw a lightweight infinite grid in canvas space.
    // This is intentionally simple (paint-only skeleton) and will be replaced by the node-graph
    // style-driven cached grid tiles in later milestones.
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let step = 64.0f32;
    let thickness = (1.0 / zoom).max(0.25 / zoom);

    let tl = view.screen_to_canvas(bounds, bounds.origin);
    let br = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0),
            Px(bounds.origin.y.0 + bounds.size.height.0),
        ),
    );

    let x0 = tl.x.0.min(br.x.0);
    let x1 = tl.x.0.max(br.x.0);
    let y0 = tl.y.0.min(br.y.0);
    let y1 = tl.y.0.max(br.y.0);

    let ix0 = (x0 / step).floor() as i32 - 1;
    let ix1 = (x1 / step).ceil() as i32 + 1;
    let iy0 = (y0 / step).floor() as i32 - 1;
    let iy1 = (y1 / step).ceil() as i32 + 1;

    // Snap the painted extents to the computed grid indices so small pans do not require
    // rebuilding the cached ops (the transform already accounts for pan/zoom).
    let x0 = ix0 as f32 * step;
    let x1 = ix1 as f32 * step;
    let y0 = iy0 as f32 * step;
    let y1 = iy1 as f32 * step;

    let max_lines = 400i32;
    let x_lines = (ix1 - ix0).clamp(0, max_lines);
    let y_lines = (iy1 - iy0).clamp(0, max_lines);

    let grid_paint: fret_core::scene::PaintBindingV1 =
        fret_core::scene::Paint::Solid(style.paint.grid_minor_color).into();

    for i in 0..x_lines {
        let x = (ix0 + i) as f32 * step;
        let rect = Rect::new(
            Point::new(Px(x - 0.5 * thickness), Px(y0)),
            fret_core::Size::new(Px(thickness), Px(y1 - y0)),
        );
        ops.push(fret_core::SceneOp::Quad {
            order: DrawOrder(1),
            rect,
            background: grid_paint,
            border: fret_core::Edges::default(),
            border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::default(),
        });
    }

    for i in 0..y_lines {
        let y = (iy0 + i) as f32 * step;
        let rect = Rect::new(
            Point::new(Px(x0), Px(y - 0.5 * thickness)),
            fret_core::Size::new(Px(x1 - x0), Px(thickness)),
        );
        ops.push(fret_core::SceneOp::Quad {
            order: DrawOrder(1),
            rect,
            background: grid_paint,
            border: fret_core::Edges::default(),
            border_paint: fret_core::scene::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::default(),
        });
    }

    if push_transform {
        ops.push(fret_core::SceneOp::PopTransform);
    }

    Arc::new(ops)
}

fn paint_debug_grid_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    ops: Option<Arc<Vec<fret_core::SceneOp>>>,
    style: &NodeGraphStyle,
) {
    if let Some(ops) = ops {
        for op in ops.iter().copied() {
            p.scene().push(op);
        }
        return;
    }

    // Fallback path: build-and-paint without storing any cache. This keeps the surface functional
    // in the very first frames before bounds are known to the element.
    let bounds = p.bounds();
    let ops = build_debug_grid_ops(bounds, view, style);
    for op in ops.iter().copied() {
        p.scene().push(op);
    }
}

#[derive(Debug, Clone)]
struct EdgePaintCacheState {
    key: Option<CanvasKey>,
    rebuilds: u64,
    draws: Option<Arc<Vec<EdgePathDraw>>>,
}

impl Default for EdgePaintCacheState {
    fn default() -> Self {
        Self {
            key: None,
            rebuilds: 0,
            draws: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgePaintCacheKeyV3 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    derived_geometry_key: u64,
}

#[derive(Debug, Clone)]
struct EdgePathDraw {
    edge: crate::core::EdgeId,
    from: crate::core::PortId,
    to: crate::core::PortId,
    key: u64,
    commands: Box<[PathCommand]>,
    bbox: Rect,
    color: Color,
}

#[derive(Debug, Clone)]
struct NodePaintCacheState {
    key: Option<CanvasKey>,
    rebuilds: u64,
    draws: Option<Arc<Vec<NodeRectDraw>>>,
}

impl Default for NodePaintCacheState {
    fn default() -> Self {
        Self {
            key: None,
            rebuilds: 0,
            draws: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodePaintCacheKeyV3 {
    graph_rev: u64,
    zoom_q: i32,
    node_origin_x_q: i32,
    node_origin_y_q: i32,
    draw_order_hash: u64,
    derived_geometry_key: u64,
}

#[derive(Debug, Clone)]
struct NodeRectDraw {
    id: crate::core::NodeId,
    rect: Rect,
}

#[derive(Debug, Clone)]
struct PortalLabelInfo {
    id: crate::core::NodeId,
    left: Px,
    top: Px,
    width: Px,
    height: Px,
    label: Arc<str>,
    ports_in: u32,
    ports_out: u32,
    selected: bool,
    hovered: bool,
}

fn rect_contains_point(rect: Rect, p: Point) -> bool {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;
    p.x.0 >= x0 && p.x.0 <= x1 && p.y.0 >= y0 && p.y.0 <= y1
}

fn rect_from_points(a: Point, b: Point) -> Rect {
    let x0 = a.x.0.min(b.x.0);
    let x1 = a.x.0.max(b.x.0);
    let y0 = a.y.0.min(b.y.0);
    let y1 = a.y.0.max(b.y.0);
    Rect::new(
        Point::new(Px(x0), Px(y0)),
        fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    )
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0;
    let ay1 = ay0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0;
    let by1 = by0 + b.size.height.0;

    ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0
}

fn rect_approx_eq(a: Rect, b: Rect, eps: f32) -> bool {
    (a.origin.x.0 - b.origin.x.0).abs() <= eps
        && (a.origin.y.0 - b.origin.y.0).abs() <= eps
        && (a.size.width.0 - b.size.width.0).abs() <= eps
        && (a.size.height.0 - b.size.height.0).abs() <= eps
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    let x0 = a.origin.x.0.min(b.origin.x.0);
    let y0 = a.origin.y.0.min(b.origin.y.0);
    let x1 = (a.origin.x.0 + a.size.width.0).max(b.origin.x.0 + b.size.width.0);
    let y1 = (a.origin.y.0 + a.size.height.0).max(b.origin.y.0 + b.size.height.0);
    Rect::new(
        Point::new(Px(x0), Px(y0)),
        fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    )
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    let ox0 = outer.origin.x.0;
    let oy0 = outer.origin.y.0;
    let ox1 = ox0 + outer.size.width.0;
    let oy1 = oy0 + outer.size.height.0;

    let ix0 = inner.origin.x.0;
    let iy0 = inner.origin.y.0;
    let ix1 = ix0 + inner.size.width.0;
    let iy1 = iy0 + inner.size.height.0;

    ix0 >= ox0 && ix1 <= ox1 && iy0 >= oy0 && iy1 <= oy1
}

fn marquee_rect_screen(m: &MarqueeDragState) -> Rect {
    rect_from_points(m.start_screen, m.current_screen)
}

fn pointer_crossed_threshold(start_screen: Point, current_screen: Point, threshold: f32) -> bool {
    let threshold = threshold.max(0.0);
    let dx = current_screen.x.0 - start_screen.x.0;
    let dy = current_screen.y.0 - start_screen.y.0;
    let dist2 = dx * dx + dy * dy;
    let threshold2 = threshold * threshold;
    dist2 >= threshold2
}

fn node_drag_delta_canvas(view: PanZoom2D, drag: &NodeDragState) -> (f32, f32) {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let dx = (drag.current_screen.x.0 - drag.start_screen.x.0) / zoom;
    let dy = (drag.current_screen.y.0 - drag.start_screen.y.0) / zoom;
    (dx, dy)
}

fn node_drag_commit_delta(view: PanZoom2D, drag: &NodeDragState) -> Option<(f32, f32)> {
    let (dx, dy) = node_drag_delta_canvas(view, drag);
    if !drag.is_active() || !dx.is_finite() || !dy.is_finite() {
        return None;
    }
    if dx.abs() <= 1.0e-9 && dy.abs() <= 1.0e-9 {
        return None;
    }
    Some((dx, dy))
}

fn node_drag_contains(drag: &NodeDragState, id: crate::core::NodeId) -> bool {
    drag.nodes_sorted.binary_search(&id).is_ok()
}

fn build_node_drag_transaction(
    graph: &Graph,
    nodes: &[crate::core::NodeId],
    dx: f32,
    dy: f32,
) -> GraphTransaction {
    let mut tx = GraphTransaction::new();
    for id in nodes.iter().copied() {
        let Some(node) = graph.nodes.get(&id) else {
            continue;
        };
        let from = node.pos;
        let to = crate::core::CanvasPoint {
            x: from.x + dx,
            y: from.y + dy,
        };
        if from != to {
            tx.push(GraphOp::SetNodePos { id, from, to });
        }
    }

    let tx = normalize_transaction(tx);
    if tx.is_empty() {
        return tx;
    }

    let label = if tx.ops.len() == 1 {
        "Move Node"
    } else {
        "Move Nodes"
    };
    tx.with_label(label)
}

fn commit_graph_transaction(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    tx: &GraphTransaction,
) -> bool {
    if tx.is_empty() {
        return true;
    }

    if let Some(controller) = controller {
        return controller
            .dispatch_transaction_and_sync_models_action_host(host, graph, view_state, tx)
            .is_ok();
    }

    if let Some(store) = store {
        let controller = NodeGraphController::new(store.clone());
        return controller
            .dispatch_transaction_and_sync_models_action_host(host, graph, view_state, tx)
            .is_ok();
    }

    let Ok(mut scratch) = host.models_mut().read(graph, |g| g.clone()) else {
        return false;
    };
    if apply_transaction(&mut scratch, tx).is_err() {
        return false;
    }

    host.models_mut()
        .update(graph, |g| {
            *g = scratch;
        })
        .is_ok()
}

fn commit_node_drag_transaction(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    tx: &GraphTransaction,
) -> bool {
    commit_graph_transaction(host, graph, view_state, controller, store, tx)
}

fn build_diag_nudge_visible_node_transaction(graph: &Graph) -> GraphTransaction {
    let mut next = graph.clone();
    for node in next.nodes.values_mut() {
        if node.hidden {
            continue;
        }
        node.pos.x += 1.0;
        break;
    }

    let tx = graph_diff(graph, &next);
    if tx.is_empty() {
        tx
    } else {
        tx.with_label("Diag Nudge Visible Node")
    }
}

fn build_diag_normalize_visible_node_transaction(graph: &Graph) -> GraphTransaction {
    let mut next = graph.clone();
    let first_visible = next
        .nodes
        .iter()
        .find_map(|(id, node)| (!node.hidden).then_some(*id));
    let Some(first_visible) = first_visible else {
        return GraphTransaction::new();
    };

    for (id, node) in &mut next.nodes {
        if *id == first_visible {
            node.hidden = false;
            node.pos.x = 0.0;
            node.pos.y = 0.0;
            node.size = Some(crate::core::CanvasSize {
                width: 220.0,
                height: 140.0,
            });
        } else {
            node.hidden = true;
        }
    }

    let tx = graph_diff(graph, &next);
    if tx.is_empty() {
        tx
    } else {
        tx.with_label("Diag Normalize Visible Node")
    }
}

fn update_view_state_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let Ok(mut next_view_state) = host.models_mut().read(view_state, |state| state.clone()) else {
        return false;
    };
    f(&mut next_view_state);

    if let Some(controller) = controller {
        return controller
            .replace_view_state_and_sync_model_action_host(host, view_state, next_view_state)
            .is_ok();
    }

    if let Some(store) = store {
        let controller = NodeGraphController::new(store.clone());
        return controller
            .replace_view_state_and_sync_model_action_host(host, view_state, next_view_state)
            .is_ok();
    }

    host.models_mut()
        .update(view_state, |state| {
            *state = next_view_state;
        })
        .is_ok()
}

fn update_view_state_ui_host<H: UiHost>(
    host: &mut H,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    f: impl FnOnce(&mut NodeGraphViewState),
) -> bool {
    let Ok(mut next_view_state) = host.models_mut().read(view_state, |state| state.clone()) else {
        return false;
    };
    f(&mut next_view_state);

    if let Some(controller) = controller {
        return controller
            .replace_view_state_and_sync_model(host, view_state, next_view_state)
            .is_ok();
    }

    if let Some(store) = store {
        let controller = NodeGraphController::new(store.clone());
        return controller
            .replace_view_state_and_sync_model(host, view_state, next_view_state)
            .is_ok();
    }

    host.models_mut()
        .update(view_state, |state| {
            *state = next_view_state;
        })
        .is_ok()
}

fn update_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    f: impl FnOnce(
        &mut Vec<crate::core::NodeId>,
        &mut Vec<crate::core::EdgeId>,
        &mut Vec<crate::core::GroupId>,
    ),
) -> bool {
    let Ok(state) = host.models_mut().read(view_state, |state| state.clone()) else {
        return false;
    };
    let mut selected_nodes = state.selected_nodes;
    let mut selected_edges = state.selected_edges;
    let mut selected_groups = state.selected_groups;
    f(
        &mut selected_nodes,
        &mut selected_edges,
        &mut selected_groups,
    );

    if let Some(controller) = controller {
        return controller
            .set_selection_and_sync_view_model_action_host(
                host,
                view_state,
                selected_nodes,
                selected_edges,
                selected_groups,
            )
            .is_ok();
    }

    if let Some(store) = store {
        let controller = NodeGraphController::new(store.clone());
        return controller
            .set_selection_and_sync_view_model_action_host(
                host,
                view_state,
                selected_nodes,
                selected_edges,
                selected_groups,
            )
            .is_ok();
    }

    host.models_mut()
        .update(view_state, |state| {
            state.selected_nodes = selected_nodes;
            state.selected_edges = selected_edges;
            state.selected_groups = selected_groups;
        })
        .is_ok()
}

fn compute_marquee_candidate_nodes(
    rect_canvas: Rect,
    selection_mode: crate::io::NodeGraphSelectionMode,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
) -> Vec<crate::core::NodeId> {
    let mut candidates = Vec::<crate::core::NodeId>::new();
    index.query_nodes_in_rect(rect_canvas, &mut candidates);
    candidates.retain(|id| {
        let Some(node) = geom.nodes.get(id) else {
            return false;
        };
        match selection_mode {
            crate::io::NodeGraphSelectionMode::Full => rect_contains_rect(rect_canvas, node.rect),
            crate::io::NodeGraphSelectionMode::Partial => rects_intersect(rect_canvas, node.rect),
        }
    });
    candidates.sort();
    candidates.dedup();
    candidates
}

fn build_marquee_preview_selected_nodes(
    marquee: &MarqueeDragState,
    rect_canvas: Rect,
    selection_mode: crate::io::NodeGraphSelectionMode,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
) -> Arc<[crate::core::NodeId]> {
    let candidates = compute_marquee_candidate_nodes(rect_canvas, selection_mode, geom, index);
    if !marquee.toggle {
        return Arc::from(candidates.into_boxed_slice());
    }

    let mut selected_nodes = marquee.base_selected_nodes.to_vec();
    for id in candidates {
        if let Some(ix) = selected_nodes.iter().position(|value| *value == id) {
            selected_nodes.remove(ix);
        } else {
            selected_nodes.push(id);
        }
    }
    selected_nodes.sort();
    selected_nodes.dedup();
    Arc::from(selected_nodes.into_boxed_slice())
}

fn build_click_selection_preview_nodes(
    base_selected_nodes: &[crate::core::NodeId],
    hit: crate::core::NodeId,
    multi: bool,
) -> Arc<[crate::core::NodeId]> {
    let mut selected_nodes = base_selected_nodes.to_vec();
    let already_selected = selected_nodes.contains(&hit);
    if multi {
        if let Some(ix) = selected_nodes.iter().position(|id| *id == hit) {
            selected_nodes.remove(ix);
        } else {
            selected_nodes.push(hit);
        }
    } else if !already_selected {
        selected_nodes.clear();
        selected_nodes.push(hit);
    }
    selected_nodes.sort();
    selected_nodes.dedup();
    Arc::from(selected_nodes.into_boxed_slice())
}

fn commit_pending_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    pending: &PendingSelectionState,
) -> bool {
    let nodes = pending.nodes.clone();
    let clear_edges = pending.clear_edges;
    let clear_groups = pending.clear_groups;
    update_selection_action_host(
        host,
        view_state,
        controller,
        store,
        move |selected_nodes, selected_edges, selected_groups| {
            selected_nodes.clear();
            selected_nodes.extend(nodes.iter().copied());
            if clear_edges {
                selected_edges.clear();
            }
            if clear_groups {
                selected_groups.clear();
            }
        },
    )
}

fn commit_marquee_selection_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    marquee: &MarqueeDragState,
) -> bool {
    let pending = PendingSelectionState {
        nodes: marquee.preview_selected_nodes.clone(),
        clear_edges: !marquee.toggle,
        clear_groups: !marquee.toggle,
    };
    commit_pending_selection_action_host(host, view_state, controller, store, &pending)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeDragReleaseOutcome {
    selection_committed: bool,
    drag_committed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftPointerReleaseOutcome {
    None,
    NodeDrag(NodeDragReleaseOutcome),
    PendingSelection { selection_committed: bool },
    Marquee { selection_committed: bool },
}

impl LeftPointerReleaseOutcome {
    fn is_handled(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeInteractionCancelMode {
    Escape,
    PointerCancel,
}

impl DeclarativeInteractionCancelMode {
    fn includes_node_drag(self, state: Option<&NodeDragState>) -> bool {
        match self {
            Self::Escape => state.is_some_and(NodeDragState::is_live),
            Self::PointerCancel => state.is_some(),
        }
    }

    fn clear_node_drag(self, state: &mut Option<NodeDragState>) {
        match self {
            Self::Escape => {
                if let Some(state) = state.as_mut() {
                    state.cancel();
                }
            }
            Self::PointerCancel => *state = None,
        }
    }
}

fn complete_node_drag_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    node_drag: &NodeDragState,
    pending_selection: Option<&PendingSelectionState>,
) -> NodeDragReleaseOutcome {
    let view = host
        .models_mut()
        .read(view_state, |state| view_from_state(state))
        .ok()
        .unwrap_or_default();
    let drag_commit_delta = node_drag_commit_delta(view, node_drag);
    let selection_committed = pending_selection.is_some_and(|pending| {
        commit_pending_selection_action_host(host, view_state, controller, store, pending)
    });
    let drag_committed = if let Some((dx, dy)) = drag_commit_delta {
        host.models_mut()
            .read(graph, |graph| {
                build_node_drag_transaction(graph, node_drag.nodes_sorted.as_ref(), dx, dy)
            })
            .ok()
            .is_some_and(|tx| {
                commit_node_drag_transaction(host, graph, view_state, controller, store, &tx)
            })
    } else {
        false
    };
    NodeDragReleaseOutcome {
        selection_committed,
        drag_committed,
    }
}

fn handle_node_drag_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
) -> Option<LeftPointerReleaseOutcome> {
    let node_drag_value = host
        .models_mut()
        .read(node_drag, |state| state.clone())
        .ok()
        .flatten();
    let Some(node_drag_value) = node_drag_value else {
        return None;
    };

    let pending_selection_value = host
        .models_mut()
        .read(pending_selection, |state| state.clone())
        .ok()
        .flatten();
    let outcome = complete_node_drag_release_action_host(
        host,
        graph,
        view_state,
        controller,
        store,
        &node_drag_value,
        pending_selection_value.as_ref(),
    );
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    Some(LeftPointerReleaseOutcome::NodeDrag(outcome))
}

fn handle_pending_selection_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    pending_selection: &Model<Option<PendingSelectionState>>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
) -> Option<LeftPointerReleaseOutcome> {
    let pending_selection_value = host
        .models_mut()
        .read(pending_selection, |state| state.clone())
        .ok()
        .flatten();
    let Some(pending_selection_value) = pending_selection_value else {
        return None;
    };

    let selection_committed = commit_pending_selection_action_host(
        host,
        view_state,
        controller,
        store,
        &pending_selection_value,
    );
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    Some(LeftPointerReleaseOutcome::PendingSelection {
        selection_committed,
    })
}

fn handle_marquee_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
) -> Option<LeftPointerReleaseOutcome> {
    let marquee_value = host
        .models_mut()
        .read(marquee, |state| state.clone())
        .ok()
        .flatten();
    let Some(marquee_value) = marquee_value else {
        return None;
    };

    let selection_committed = if marquee_value.active || !marquee_value.toggle {
        commit_marquee_selection_action_host(host, view_state, controller, store, &marquee_value)
    } else {
        false
    };
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host.models_mut().update(marquee, |state| *state = None);
    Some(LeftPointerReleaseOutcome::Marquee {
        selection_committed,
    })
}

fn complete_left_pointer_release_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
) -> LeftPointerReleaseOutcome {
    if let Some(release) = handle_node_drag_left_pointer_release_action_host(
        host,
        node_drag,
        pending_selection,
        graph,
        view_state,
        controller,
        store,
    ) {
        return release;
    }

    if let Some(release) = handle_pending_selection_left_pointer_release_action_host(
        host,
        pending_selection,
        view_state,
        controller,
        store,
    ) {
        return release;
    }

    handle_marquee_left_pointer_release_action_host(
        host,
        marquee,
        pending_selection,
        view_state,
        controller,
        store,
    )
    .unwrap_or(LeftPointerReleaseOutcome::None)
}

fn cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    mode: DeclarativeInteractionCancelMode,
) -> bool {
    let drag_active = host
        .models_mut()
        .read(drag, |state| state.is_some())
        .ok()
        .unwrap_or(false);
    let marquee_active = host
        .models_mut()
        .read(marquee, |state| state.is_some())
        .ok()
        .unwrap_or(false);
    let node_drag_active = host
        .models_mut()
        .read(node_drag, |state| mode.includes_node_drag(state.as_ref()))
        .ok()
        .unwrap_or(false);
    let pending_selection_active = host
        .models_mut()
        .read(pending_selection, |state| state.is_some())
        .ok()
        .unwrap_or(false);

    if !drag_active && !marquee_active && !node_drag_active && !pending_selection_active {
        return false;
    }

    let _ = host.models_mut().update(drag, |state| *state = None);
    let _ = host.models_mut().update(marquee, |state| *state = None);
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);
    let _ = host
        .models_mut()
        .update(node_drag, |state| mode.clear_node_drag(state));
    true
}

fn escape_cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
        DeclarativeInteractionCancelMode::Escape,
    )
}

fn pointer_cancel_declarative_interactions_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
        DeclarativeInteractionCancelMode::PointerCancel,
    )
}

fn notify_and_redraw_action_host<H>(host: &mut H, action_cx: fret_ui::action::ActionCx)
where
    H: fret_ui::action::UiActionHost + ?Sized,
{
    host.notify(action_cx);
    host.request_redraw(action_cx.window);
}

fn invalidate_notify_and_redraw_pointer_action_host<H>(
    host: &mut H,
    action_cx: fret_ui::action::ActionCx,
    invalidation: Invalidation,
) where
    H: fret_ui::action::UiPointerActionHost + ?Sized,
{
    host.invalidate(invalidation);
    notify_and_redraw_action_host(host, action_cx);
}

fn finish_declarative_pointer_session_action_host<H>(
    host: &mut H,
    action_cx: fret_ui::action::ActionCx,
) where
    H: fret_ui::action::UiPointerActionHost + ?Sized,
{
    host.release_pointer_capture();
    invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
}

fn handle_declarative_pointer_up_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: fret_ui::action::ActionCx,
    up: fret_ui::action::PointerUpCx,
    pan_button: MouseButton,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
) -> bool {
    if up.button != pan_button {
        if up.button != MouseButton::Left {
            return false;
        }

        let release = complete_left_pointer_release_action_host(
            host,
            node_drag,
            pending_selection,
            marquee,
            graph,
            view_state,
            controller,
            store,
        );
        if !release.is_handled() {
            return false;
        }

        finish_declarative_pointer_session_action_host(host, action_cx);
        return true;
    }

    let _ = host.models_mut().update(drag, |state| *state = None);
    finish_declarative_pointer_session_action_host(host, action_cx);
    true
}

fn handle_declarative_pointer_cancel_action_host(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    action_cx: fret_ui::action::ActionCx,
    _cancel: fret_ui::action::PointerCancelCx,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    let _ = pointer_cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
    );
    finish_declarative_pointer_session_action_host(host, action_cx);
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeDiagViewPreset {
    CenteredSelectionOnDrag,
    OffsetPartialMarquee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeDiagKeyAction {
    NudgeVisibleNode,
    NormalizeVisibleNodeCentered,
    NormalizeVisibleNodeForMarquee,
    ArmFitToPortals,
    DisablePortals,
    EnablePortals,
    TogglePaintOverrides,
}

impl DeclarativeDiagKeyAction {
    fn from_key(enabled: bool, key: fret_core::KeyCode) -> Option<Self> {
        if !enabled {
            return None;
        }

        match key {
            fret_core::KeyCode::Digit3 => Some(Self::NudgeVisibleNode),
            fret_core::KeyCode::Digit4 => Some(Self::NormalizeVisibleNodeCentered),
            fret_core::KeyCode::Digit5 => Some(Self::NormalizeVisibleNodeForMarquee),
            fret_core::KeyCode::Digit9 => Some(Self::ArmFitToPortals),
            fret_core::KeyCode::Digit8 => Some(Self::DisablePortals),
            fret_core::KeyCode::Digit7 => Some(Self::EnablePortals),
            fret_core::KeyCode::Digit6 => Some(Self::TogglePaintOverrides),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeKeyboardZoomAction {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl DeclarativeKeyboardZoomAction {
    fn from_key(key: fret_core::KeyCode) -> Option<Self> {
        match key {
            fret_core::KeyCode::Equal
            | fret_core::KeyCode::NumpadAdd
            | fret_core::KeyCode::Digit1 => Some(Self::ZoomIn),
            fret_core::KeyCode::Minus
            | fret_core::KeyCode::NumpadSubtract
            | fret_core::KeyCode::Digit2 => Some(Self::ZoomOut),
            fret_core::KeyCode::Digit0 | fret_core::KeyCode::Numpad0 => Some(Self::Reset),
            _ => None,
        }
    }
}

fn handle_declarative_escape_key_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    escape_cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
    )
}

fn commit_diag_graph_transaction_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    build_tx: fn(&Graph) -> GraphTransaction,
) -> bool {
    let tx = host.models_mut().read(graph, build_tx).ok();
    if let Some(tx) = tx.as_ref() {
        let _ = commit_graph_transaction(host, graph, view_state, controller, store, tx);
    }
    true
}

fn apply_declarative_diag_view_preset_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    preset: DeclarativeDiagViewPreset,
) -> bool {
    update_view_state_action_host(host, view_state, controller, store, |state| {
        match preset {
            DeclarativeDiagViewPreset::CenteredSelectionOnDrag => {
                state.pan.x = 380.0;
                state.pan.y = 290.0;
                state.zoom = 1.0;
                state.interaction.selection_on_drag = true;
            }
            DeclarativeDiagViewPreset::OffsetPartialMarquee => {
                state.pan.x = 540.0;
                state.pan.y = 290.0;
                state.zoom = 1.0;
                state.interaction.selection_on_drag = true;
                state.interaction.selection_mode = crate::io::NodeGraphSelectionMode::Partial;
            }
        }
        state.selected_nodes.clear();
        state.selected_edges.clear();
        state.selected_groups.clear();
    })
}

fn toggle_diag_paint_overrides_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    diag_paint_overrides: &Arc<NodeGraphPaintOverridesMap>,
    diag_paint_overrides_enabled: &Model<bool>,
) -> bool {
    let enable_next = host
        .models_mut()
        .read(diag_paint_overrides_enabled, |state| !*state)
        .ok()
        .unwrap_or(true);
    let _ = host
        .models_mut()
        .update(diag_paint_overrides_enabled, |state| *state = enable_next);

    let edge_id = host
        .models_mut()
        .read(graph, |graph| graph.edges.keys().next().copied())
        .ok()
        .flatten();

    if let Some(edge_id) = edge_id {
        if enable_next {
            let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
            stops[0] = GradientStop::new(0.0, Color::from_srgb_hex_rgb(0xff_3b_30));
            stops[1] = GradientStop::new(1.0, Color::from_srgb_hex_rgb(0x34_c7_59));
            let gradient = LinearGradient {
                start: Point::new(Px(0.0), Px(0.0)),
                end: Point::new(Px(240.0), Px(0.0)),
                tile_mode: TileMode::Clamp,
                color_space: ColorSpace::Srgb,
                stop_count: 2,
                stops,
            };
            let paint = PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::ViewportPx,
            );
            diag_paint_overrides.set_edge_override(
                edge_id,
                Some(
                    crate::ui::paint_overrides::EdgePaintOverrideV1 {
                        dash: Some(DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0))),
                        stroke_width_mul: Some(1.6),
                        stroke_paint: Some(paint),
                    }
                    .normalized(),
                ),
            );
        } else {
            diag_paint_overrides.set_edge_override(edge_id, None);
        }
    }

    true
}

fn handle_declarative_diag_key_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    action: DeclarativeDiagKeyAction,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    portal_bounds_store: &Model<PortalBoundsStore>,
    portal_debug_flags: &Model<PortalDebugFlags>,
    diag_paint_overrides: &Arc<NodeGraphPaintOverridesMap>,
    diag_paint_overrides_enabled: &Model<bool>,
) -> bool {
    match action {
        DeclarativeDiagKeyAction::NudgeVisibleNode => commit_diag_graph_transaction_action_host(
            host,
            graph,
            view_state,
            controller,
            store,
            build_diag_nudge_visible_node_transaction,
        ),
        DeclarativeDiagKeyAction::NormalizeVisibleNodeCentered => {
            commit_diag_graph_transaction_action_host(
                host,
                graph,
                view_state,
                controller,
                store,
                build_diag_normalize_visible_node_transaction,
            );
            let _ = apply_declarative_diag_view_preset_action_host(
                host,
                view_state,
                controller,
                store,
                DeclarativeDiagViewPreset::CenteredSelectionOnDrag,
            );
            true
        }
        DeclarativeDiagKeyAction::NormalizeVisibleNodeForMarquee => {
            commit_diag_graph_transaction_action_host(
                host,
                graph,
                view_state,
                controller,
                store,
                build_diag_normalize_visible_node_transaction,
            );
            let _ = apply_declarative_diag_view_preset_action_host(
                host,
                view_state,
                controller,
                store,
                DeclarativeDiagViewPreset::OffsetPartialMarquee,
            );
            true
        }
        DeclarativeDiagKeyAction::ArmFitToPortals => {
            let _ = host.models_mut().update(portal_bounds_store, |state| {
                state.pending_fit_to_portals = true;
            });
            true
        }
        DeclarativeDiagKeyAction::DisablePortals => {
            let _ = host.models_mut().update(portal_debug_flags, |state| {
                state.disable_portals = true;
            });
            let _ = host.models_mut().update(portal_bounds_store, |state| {
                state.nodes_canvas_bounds.clear();
                state.pending_fit_to_portals = false;
            });
            true
        }
        DeclarativeDiagKeyAction::EnablePortals => {
            let _ = host.models_mut().update(portal_debug_flags, |state| {
                state.disable_portals = false;
            });
            true
        }
        DeclarativeDiagKeyAction::TogglePaintOverrides => toggle_diag_paint_overrides_action_host(
            host,
            graph,
            diag_paint_overrides,
            diag_paint_overrides_enabled,
        ),
    }
}

fn handle_declarative_keyboard_zoom_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    action: DeclarativeKeyboardZoomAction,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    min_zoom: f32,
    max_zoom: f32,
) -> bool {
    const KB_ZOOM_STEP_MUL: f32 = 1.1;
    update_view_state_action_host(host, view_state, controller, store, |state| {
        let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
        state.zoom = match action {
            DeclarativeKeyboardZoomAction::ZoomIn => {
                (zoom * KB_ZOOM_STEP_MUL).clamp(min_zoom, max_zoom)
            }
            DeclarativeKeyboardZoomAction::ZoomOut => {
                (zoom * (1.0 / KB_ZOOM_STEP_MUL)).clamp(min_zoom, max_zoom)
            }
            DeclarativeKeyboardZoomAction::Reset => 1.0,
        };
    })
}

#[derive(Debug, Clone)]
struct LeftPointerDownSnapshot {
    interaction: crate::io::NodeGraphInteractionConfig,
    base_selection: Vec<crate::core::NodeId>,
    hit: Option<crate::core::NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftPointerDownOutcome {
    HitNode { capture_pointer: bool },
    Marquee,
    EmptySpaceClear,
    Idle,
}

impl LeftPointerDownOutcome {
    fn capture_pointer(self) -> bool {
        matches!(
            self,
            Self::HitNode {
                capture_pointer: true,
            } | Self::Marquee
                | Self::EmptySpaceClear
        )
    }
}

fn begin_pan_pointer_down_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    down: fret_ui::action::PointerDownCx,
) -> bool {
    let _ = host.models_mut().update(marquee, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    let _ = host.models_mut().update(drag, |state| {
        *state = Some(DragState {
            button: down.button,
            last_pos: down.position,
        });
    });
    true
}

fn read_left_pointer_down_snapshot_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    hit_scratch: &Model<Vec<crate::core::NodeId>>,
    down: fret_ui::action::PointerDownCx,
    bounds: Rect,
) -> LeftPointerDownSnapshot {
    let (interaction, base_selection, node_click_distance_screen_px, view) = host
        .models_mut()
        .read(view_state, |state| {
            (
                state.interaction.clone(),
                state.selected_nodes.clone(),
                state.interaction.node_click_distance,
                view_from_state(state),
            )
        })
        .ok()
        .unwrap_or((Default::default(), Vec::new(), 6.0, PanZoom2D::default()));

    let (geom, index) = host
        .models_mut()
        .read(derived_cache, |state| {
            (state.geom.clone(), state.index.clone())
        })
        .ok()
        .unwrap_or((None, None));

    let hit = if let (Some(geom), Some(index)) = (geom.as_deref(), index.as_deref()) {
        host.models_mut()
            .update(hit_scratch, |scratch| {
                hit_test_node_at_point(
                    view,
                    bounds,
                    node_click_distance_screen_px,
                    geom,
                    index,
                    down.position,
                    scratch,
                )
            })
            .ok()
            .flatten()
    } else {
        None
    };

    LeftPointerDownSnapshot {
        interaction,
        base_selection,
        hit,
    }
}

fn begin_left_pointer_down_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    down: fret_ui::action::PointerDownCx,
    snapshot: &LeftPointerDownSnapshot,
) -> LeftPointerDownOutcome {
    let multi = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(down.modifiers);
    let selection_box_armed = snapshot.interaction.selection_on_drag
        || snapshot
            .interaction
            .selection_key
            .is_pressed(down.modifiers);

    if let Some(hit) = snapshot.hit {
        let _ = host.models_mut().update(marquee, |state| *state = None);
        let _ = host.models_mut().update(node_drag, |state| *state = None);
        let _ = host
            .models_mut()
            .update(pending_selection, |state| *state = None);
        let _ = host
            .models_mut()
            .update(hovered, |state| *state = Some(hit));
        if snapshot.interaction.elements_selectable {
            let preview_nodes =
                build_click_selection_preview_nodes(&snapshot.base_selection, hit, multi);
            let _ = host.models_mut().update(pending_selection, |state| {
                *state = Some(PendingSelectionState {
                    nodes: preview_nodes.clone(),
                    clear_edges: false,
                    clear_groups: false,
                });
            });

            if snapshot.interaction.nodes_draggable && !multi {
                let _ = host.models_mut().update(node_drag, |state| {
                    *state = Some(NodeDragState {
                        start_screen: down.position,
                        current_screen: down.position,
                        phase: NodeDragPhase::Armed,
                        nodes_sorted: preview_nodes,
                    });
                });
            }
        }
        return LeftPointerDownOutcome::HitNode {
            capture_pointer: snapshot.interaction.elements_selectable,
        };
    }

    let _ = host.models_mut().update(hovered, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);

    if selection_box_armed && snapshot.interaction.elements_selectable {
        let base_selected_nodes: Arc<[crate::core::NodeId]> = if multi {
            Arc::from(snapshot.base_selection.clone().into_boxed_slice())
        } else {
            Arc::from([])
        };
        let preview_selected_nodes: Arc<[crate::core::NodeId]> = if multi {
            base_selected_nodes.clone()
        } else {
            Arc::from([])
        };
        let _ = host.models_mut().update(marquee, |state| {
            *state = Some(MarqueeDragState {
                start_screen: down.position,
                current_screen: down.position,
                active: false,
                toggle: multi,
                base_selected_nodes,
                preview_selected_nodes,
            });
        });
        return LeftPointerDownOutcome::Marquee;
    }

    if snapshot.interaction.elements_selectable && !multi {
        let _ = host.models_mut().update(pending_selection, |state| {
            *state = Some(PendingSelectionState {
                nodes: Arc::from([]),
                clear_edges: true,
                clear_groups: true,
            });
        });
        return LeftPointerDownOutcome::EmptySpaceClear;
    }

    LeftPointerDownOutcome::Idle
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeDragPointerMoveOutcome {
    capture_pointer: bool,
    needs_layout_redraw: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarqueePointerMoveOutcome {
    ReleaseCaptureRedrawOnly,
    NotifyRedraw,
}

fn handle_node_drag_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    view_state: &Model<NodeGraphViewState>,
    controller: Option<&NodeGraphController>,
    store: Option<&Model<NodeGraphStore>>,
    mv: fret_ui::action::PointerMoveCx,
) -> Option<NodeDragPointerMoveOutcome> {
    let node_drag_value = host
        .models_mut()
        .read(node_drag, |state| state.clone())
        .ok()
        .flatten()?;

    if !mouse_buttons_contains(mv.buttons, MouseButton::Left) {
        return Some(NodeDragPointerMoveOutcome {
            capture_pointer: false,
            needs_layout_redraw: false,
        });
    }

    if node_drag_value.is_canceled() {
        let _ = host.models_mut().update(hovered, |state| *state = None);
        return Some(NodeDragPointerMoveOutcome {
            capture_pointer: false,
            needs_layout_redraw: false,
        });
    }

    let interaction = host
        .models_mut()
        .read(view_state, |state| state.interaction.clone())
        .ok()
        .unwrap_or_default();
    let should_activate = pointer_crossed_threshold(
        node_drag_value.start_screen,
        mv.position,
        interaction.node_drag_threshold,
    );
    let capture_pointer = should_activate && node_drag_value.is_armed();

    if capture_pointer {
        let pending_selection_value = host
            .models_mut()
            .read(pending_selection, |state| state.clone())
            .ok()
            .flatten();
        if let Some(pending_selection_value) = pending_selection_value.as_ref() {
            let _ = commit_pending_selection_action_host(
                host,
                view_state,
                controller,
                store,
                pending_selection_value,
            );
            let _ = host
                .models_mut()
                .update(pending_selection, |state| *state = None);
        }
    }

    let mut needs_layout_redraw = false;
    let _ = host.models_mut().update(node_drag, |state| {
        if let Some(state) = state.as_mut() {
            if should_activate && state.activate(mv.position) {
                needs_layout_redraw = true;
            }
            if state.update_active_position(mv.position) {
                needs_layout_redraw = true;
            }
        }
    });
    let _ = host.models_mut().update(hovered, |state| *state = None);

    Some(NodeDragPointerMoveOutcome {
        capture_pointer,
        needs_layout_redraw,
    })
}

fn handle_marquee_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    mv: fret_ui::action::PointerMoveCx,
    bounds: Rect,
) -> Option<MarqueePointerMoveOutcome> {
    let marquee_value = host
        .models_mut()
        .read(marquee, |state| state.clone())
        .ok()
        .flatten()?;
    let (interaction, view) = host
        .models_mut()
        .read(view_state, |state| {
            (state.interaction.clone(), view_from_state(state))
        })
        .ok()
        .unwrap_or((Default::default(), PanZoom2D::default()));

    if !interaction.elements_selectable {
        let _ = host.models_mut().update(marquee, |state| *state = None);
        return Some(MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly);
    }

    let should_activate = pointer_crossed_threshold(
        marquee_value.start_screen,
        mv.position,
        interaction.node_click_distance,
    );
    let active_now = marquee_value.active || should_activate;

    let _ = host.models_mut().update(marquee, |state| {
        if let Some(state) = state.as_mut() {
            if should_activate {
                state.active = true;
            }
            if state.active {
                state.current_screen = mv.position;
            }
        }
    });

    if active_now {
        let (geom, index) = host
            .models_mut()
            .read(derived_cache, |state| {
                (state.geom.clone(), state.index.clone())
            })
            .ok()
            .unwrap_or((None, None));

        if let (Some(geom), Some(index)) = (geom.as_deref(), index.as_deref()) {
            let start_canvas = view.screen_to_canvas(bounds, marquee_value.start_screen);
            let cur_canvas = view.screen_to_canvas(bounds, mv.position);
            let rect_canvas = rect_from_points(start_canvas, cur_canvas);
            let preview_selected_nodes = build_marquee_preview_selected_nodes(
                &marquee_value,
                rect_canvas,
                interaction.selection_mode,
                geom,
                index,
            );
            let _ = host.models_mut().update(marquee, |state| {
                if let Some(state) = state.as_mut() {
                    state.preview_selected_nodes = preview_selected_nodes.clone();
                }
            });
        }
    }

    let _ = host.models_mut().update(hovered, |state| *state = None);
    Some(MarqueePointerMoveOutcome::NotifyRedraw)
}

fn update_hovered_node_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    hovered: &Model<Option<crate::core::NodeId>>,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    hit_scratch: &Model<Vec<crate::core::NodeId>>,
    mv: fret_ui::action::PointerMoveCx,
    bounds: Rect,
) -> bool {
    let node_click_distance_screen_px = host
        .models_mut()
        .read(view_state, |state| state.interaction.node_click_distance)
        .ok()
        .unwrap_or(6.0);
    let view = host
        .models_mut()
        .read(view_state, |state| view_from_state(state))
        .ok()
        .unwrap_or_default();

    let (geom, index) = host
        .models_mut()
        .read(derived_cache, |state| {
            (state.geom.clone(), state.index.clone())
        })
        .ok()
        .unwrap_or((None, None));

    let hit = if let (Some(geom), Some(index)) = (geom.as_deref(), index.as_deref()) {
        host.models_mut()
            .update(hit_scratch, |scratch| {
                hit_test_node_at_point(
                    view,
                    bounds,
                    node_click_distance_screen_px,
                    geom,
                    index,
                    mv.position,
                    scratch,
                )
            })
            .ok()
            .flatten()
    } else {
        None
    };

    host.models_mut()
        .update(hovered, |state| {
            if *state == hit {
                false
            } else {
                *state = hit;
                true
            }
        })
        .ok()
        .unwrap_or(false)
}

fn hit_test_node_at_point(
    view: PanZoom2D,
    bounds: Rect,
    node_click_distance_screen_px: f32,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    pos_screen: Point,
    scratch: &mut Vec<crate::core::NodeId>,
) -> Option<crate::core::NodeId> {
    let z = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let radius_canvas = (node_click_distance_screen_px.max(0.0) / z).max(0.0);
    let pos_canvas = view.screen_to_canvas(bounds, pos_screen);

    let query = Rect::new(
        Point::new(
            Px(pos_canvas.x.0 - radius_canvas),
            Px(pos_canvas.y.0 - radius_canvas),
        ),
        fret_core::Size::new(Px(2.0 * radius_canvas), Px(2.0 * radius_canvas)),
    );

    scratch.clear();
    index.query_nodes_in_rect(query, scratch);

    let mut best: Option<(u32, crate::core::NodeId)> = None;
    for id in scratch.iter().copied() {
        let Some(node) = geom.nodes.get(&id) else {
            continue;
        };
        if !rect_contains_point(node.rect, pos_canvas) {
            continue;
        }
        let rank = geom.node_rank.get(&id).copied().unwrap_or(0);
        match best {
            None => best = Some((rank, id)),
            Some((cur, _)) if rank >= cur => best = Some((rank, id)),
            _ => {}
        }
    }
    best.map(|(_, id)| id)
}

fn edges_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    derived_geometry_key: u64,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();
    let v = EdgePaintCacheKeyV3 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash,
        derived_geometry_key,
    };
    CanvasKey::from_hash(&("fret-node.edges.paint-only.v3", v))
}

fn nodes_cache_key(
    graph_rev: u64,
    zoom: f32,
    node_origin: crate::io::NodeGraphNodeOrigin,
    draw_order_hash: u64,
    derived_geometry_key: u64,
) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0);
    let origin = node_origin.normalized();
    let v = NodePaintCacheKeyV3 {
        graph_rev,
        zoom_q: quantize_f32(zoom, 4096.0),
        node_origin_x_q: quantize_f32(origin.x, 4096.0),
        node_origin_y_q: quantize_f32(origin.y, 4096.0),
        draw_order_hash,
        derived_geometry_key,
    };
    CanvasKey::from_hash(&("fret-node.nodes.paint-only.v3", v))
}

fn canvas_viewport_rect(bounds: Rect, view: PanZoom2D, margin_screen_px: f32) -> Option<Rect> {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let margin_canvas = (margin_screen_px / zoom).max(0.0);

    let tl = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 - margin_screen_px),
            Px(bounds.origin.y.0 - margin_screen_px),
        ),
    );
    let br = view.screen_to_canvas(
        bounds,
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 + margin_screen_px),
            Px(bounds.origin.y.0 + bounds.size.height.0 + margin_screen_px),
        ),
    );

    // Expand in canvas space as well to compensate for quantization and FP noise.
    let x0 = tl.x.0.min(br.x.0) - margin_canvas;
    let x1 = tl.x.0.max(br.x.0) + margin_canvas;
    let y0 = tl.y.0.min(br.y.0) - margin_canvas;
    let y1 = tl.y.0.max(br.y.0) + margin_canvas;

    if !x0.is_finite() || !x1.is_finite() || !y0.is_finite() || !y1.is_finite() {
        return None;
    }

    Some(Rect::new(
        Point::new(Px(x0), Px(y0)),
        fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    ))
}

fn build_edges_draws_paint_only(
    graph: &Graph,
    graph_rev: u64,
    zoom: f32,
    geom: &CanvasGeometry,
    style: &NodeGraphStyle,
) -> Arc<Vec<EdgePathDraw>> {
    let mut out = Vec::<EdgePathDraw>::new();
    out.reserve(graph.edges.len().min(4096));

    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0).max(1.0e-6);

    for (edge_id, edge) in &graph.edges {
        let Some(p0) = geom.port_center(edge.from) else {
            continue;
        };
        let Some(p1) = geom.port_center(edge.to) else {
            continue;
        };

        let (ctrl1, ctrl2) = canvas_wires::wire_ctrl_points(p0, p1, zoom);

        let min_x = p0.x.0.min(p1.x.0).min(ctrl1.x.0).min(ctrl2.x.0);
        let max_x = p0.x.0.max(p1.x.0).max(ctrl1.x.0).max(ctrl2.x.0);
        let min_y = p0.y.0.min(p1.y.0).min(ctrl1.y.0).min(ctrl2.y.0);
        let max_y = p0.y.0.max(p1.y.0).max(ctrl1.y.0).max(ctrl2.y.0);
        let mut bbox = Rect::new(
            Point::new(Px(min_x), Px(min_y)),
            fret_core::Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
        );
        let pad = style
            .geometry
            .wire_width
            .max(style.paint.wire_interaction_width)
            / zoom;
        let pad = pad.max(0.0);
        bbox = Rect::new(
            Point::new(Px(bbox.origin.x.0 - pad), Px(bbox.origin.y.0 - pad)),
            fret_core::Size::new(
                Px((bbox.size.width.0 + 2.0 * pad).max(0.0)),
                Px((bbox.size.height.0 + 2.0 * pad).max(0.0)),
            ),
        );

        let commands: Box<[PathCommand]> = vec![
            PathCommand::MoveTo(p0),
            PathCommand::CubicTo {
                ctrl1,
                ctrl2,
                to: p1,
            },
        ]
        .into_boxed_slice();

        // Stable hosted path key: edge identity + graph model revision.
        let path_key = CanvasKey::from_hash(&("fret-node.edge-path.v1", graph_rev, edge_id)).0;
        let color = match edge.kind {
            crate::core::EdgeKind::Data => style.paint.wire_color_data,
            crate::core::EdgeKind::Exec => style.paint.wire_color_exec,
        };

        out.push(EdgePathDraw {
            edge: *edge_id,
            from: edge.from,
            to: edge.to,
            key: path_key,
            commands,
            bbox,
            color,
        });
    }

    Arc::new(out)
}

fn scale_dash_pattern_screen_px_to_canvas_units(
    pattern: DashPatternV1,
    zoom: f32,
) -> Option<DashPatternV1> {
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }

    let dash = pattern.dash.0 / zoom;
    let gap = pattern.gap.0 / zoom;
    let phase = pattern.phase.0 / zoom;
    let period = dash + gap;
    if !dash.is_finite() || !gap.is_finite() || !phase.is_finite() || dash <= 0.0 || period <= 0.0 {
        return None;
    }

    Some(DashPatternV1::new(Px(dash), Px(gap), Px(phase)))
}

fn paint_edges_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    margin_screen_px: f32,
    draws: Option<Arc<Vec<EdgePathDraw>>>,
    geom: Option<Arc<CanvasGeometry>>,
    node_drag: Option<&NodeDragState>,
    style_tokens: &NodeGraphStyle,
    paint_overrides: Option<&dyn crate::ui::paint_overrides::NodeGraphPaintOverrides>,
) {
    let Some(draws) = draws else {
        return;
    };
    let geom = geom.unwrap_or_else(|| Arc::new(CanvasGeometry::default()));

    let bounds = p.bounds();
    let Some(cull) = canvas_viewport_rect(bounds, view, margin_screen_px) else {
        return;
    };
    let Some(transform) = view.render_transform(bounds) else {
        return;
    };

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let raster_scale_factor = p.scale_factor() * zoom;
    let base_stroke_width = (style_tokens.geometry.wire_width / zoom).max(0.0);

    p.with_transform(transform, |p| {
        let drag_active = node_drag.is_some_and(NodeDragState::is_active);
        let (ddx, ddy) = node_drag
            .filter(|_| drag_active)
            .map(|d| node_drag_delta_canvas(view, d))
            .unwrap_or((0.0, 0.0));

        for d in draws.iter() {
            let mut paint: fret_core::scene::PaintBindingV1 = d.color.into();
            let mut stroke_width_mul = 1.0_f32;
            let mut dash: Option<DashPatternV1> = None;

            if let Some(o) = paint_overrides
                .and_then(|p| p.edge_paint_override(d.edge))
                .map(|o| o.normalized())
            {
                if let Some(m) = o.stroke_width_mul {
                    if m.is_finite() && m > 0.0 {
                        stroke_width_mul = m;
                    }
                }
                dash = o
                    .dash
                    .and_then(|p| scale_dash_pattern_screen_px_to_canvas_units(p, zoom));
                if let Some(paint_override) = o.stroke_paint {
                    paint = paint_override;
                }
            }

            let stroke_width = (base_stroke_width * stroke_width_mul).max(0.0);
            let style = PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(stroke_width),
                join: StrokeJoinV1::Round,
                cap: StrokeCapV1::Round,
                miter_limit: 4.0,
                dash,
            });

            let mut affected_by_drag = false;
            if drag_active {
                let from_node = geom.ports.get(&d.from).map(|h| h.node);
                let to_node = geom.ports.get(&d.to).map(|h| h.node);
                affected_by_drag = from_node
                    .is_some_and(|id| node_drag.is_some_and(|drag| node_drag_contains(drag, id)))
                    || to_node.is_some_and(|id| {
                        node_drag.is_some_and(|drag| node_drag_contains(drag, id))
                    });
            }

            if affected_by_drag {
                let Some(mut p0) = geom.port_center(d.from) else {
                    continue;
                };
                let Some(mut p1) = geom.port_center(d.to) else {
                    continue;
                };
                if let Some(from) = geom.ports.get(&d.from) {
                    if node_drag.is_some_and(|drag| node_drag_contains(drag, from.node)) {
                        p0 = Point::new(Px(p0.x.0 + ddx), Px(p0.y.0 + ddy));
                    }
                }
                if let Some(to) = geom.ports.get(&d.to) {
                    if node_drag.is_some_and(|drag| node_drag_contains(drag, to.node)) {
                        p1 = Point::new(Px(p1.x.0 + ddx), Px(p1.y.0 + ddy));
                    }
                }

                let (ctrl1, ctrl2) = canvas_wires::wire_ctrl_points(p0, p1, zoom);
                let commands: Box<[PathCommand]> = vec![
                    PathCommand::MoveTo(p0),
                    PathCommand::CubicTo {
                        ctrl1,
                        ctrl2,
                        to: p1,
                    },
                ]
                .into_boxed_slice();

                let min_x = p0.x.0.min(p1.x.0).min(ctrl1.x.0).min(ctrl2.x.0);
                let max_x = p0.x.0.max(p1.x.0).max(ctrl1.x.0).max(ctrl2.x.0);
                let min_y = p0.y.0.min(p1.y.0).min(ctrl1.y.0).min(ctrl2.y.0);
                let max_y = p0.y.0.max(p1.y.0).max(ctrl1.y.0).max(ctrl2.y.0);
                let mut bbox = Rect::new(
                    Point::new(Px(min_x), Px(min_y)),
                    fret_core::Size::new(
                        Px((max_x - min_x).max(0.0)),
                        Px((max_y - min_y).max(0.0)),
                    ),
                );
                let pad = style_tokens
                    .geometry
                    .wire_width
                    .max(style_tokens.paint.wire_interaction_width)
                    / zoom;
                let pad = pad.max(0.0);
                bbox = Rect::new(
                    Point::new(Px(bbox.origin.x.0 - pad), Px(bbox.origin.y.0 - pad)),
                    fret_core::Size::new(
                        Px((bbox.size.width.0 + 2.0 * pad).max(0.0)),
                        Px((bbox.size.height.0 + 2.0 * pad).max(0.0)),
                    ),
                );

                if !rects_intersect(cull, bbox) {
                    continue;
                }

                let key = CanvasKey::from_hash(&(
                    "fret-node.edge-path.drag.v1",
                    d.edge,
                    quantize_f32(ddx, 1024.0),
                    quantize_f32(ddy, 1024.0),
                ))
                .0;
                p.path_paint(
                    key,
                    DrawOrder(2),
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style,
                    paint,
                    raster_scale_factor,
                );
            } else {
                if !rects_intersect(cull, d.bbox) {
                    continue;
                }
                p.path_paint(
                    d.key,
                    DrawOrder(2),
                    Point::new(Px(0.0), Px(0.0)),
                    &d.commands,
                    style,
                    paint,
                    raster_scale_factor,
                );
            }
        }
    });
}

fn build_nodes_draws_paint_only(graph: &Graph, zoom: f32) -> Arc<Vec<NodeRectDraw>> {
    // Paint-only geometry approximation:
    // - Use semantic sizes (`node.size`) when available, otherwise a stable default.
    // - Apply semantic zoom sizing: canvas-space size scales by `1/zoom` so screen size is stable.
    const DEFAULT_NODE_W_LOGICAL: f32 = 220.0;
    const DEFAULT_NODE_H_LOGICAL: f32 = 140.0;

    let mut draws = Vec::<NodeRectDraw>::new();
    draws.reserve(graph.nodes.len().min(4096));

    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0).max(1.0e-6);
    let inv_zoom = 1.0 / zoom;

    for (id, node) in &graph.nodes {
        if node.hidden {
            continue;
        }
        let logical_size = node.size.unwrap_or(crate::core::CanvasSize {
            width: DEFAULT_NODE_W_LOGICAL,
            height: DEFAULT_NODE_H_LOGICAL,
        });
        let w = logical_size.width * inv_zoom;
        let h = logical_size.height * inv_zoom;
        if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
            continue;
        }
        let rect = Rect::new(
            Point::new(Px(node.pos.x), Px(node.pos.y)),
            fret_core::Size::new(Px(w), Px(h)),
        );
        draws.push(NodeRectDraw { id: *id, rect });
    }

    Arc::new(draws)
}

fn paint_nodes_cached(
    p: &mut CanvasPainter<'_>,
    view: PanZoom2D,
    margin_screen_px: f32,
    draws: Option<Arc<Vec<NodeRectDraw>>>,
    style_tokens: &NodeGraphStyle,
    hovered_node: Option<crate::core::NodeId>,
    selected_nodes: &[crate::core::NodeId],
    node_drag: Option<&NodeDragState>,
    paint_overrides: Option<&dyn crate::ui::paint_overrides::NodeGraphPaintOverrides>,
) {
    let Some(draws) = draws else {
        return;
    };

    let bounds = p.bounds();
    let Some(cull) = canvas_viewport_rect(bounds, view, margin_screen_px) else {
        return;
    };
    let Some(transform) = view.render_transform(bounds) else {
        return;
    };

    let fill = fret_core::scene::Paint::Solid(style_tokens.paint.node_background).into();
    let transparent_fill = fret_core::scene::Paint::Solid(Color {
        a: 0.0,
        ..style_tokens.paint.node_background
    })
    .into();
    let border_base = style_tokens.paint.node_border;
    let border_hover = Color {
        a: 0.75,
        ..style_tokens.paint.node_border_selected
    };
    let border_selected = style_tokens.paint.node_border_selected;
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let border_w = (1.0 / zoom).max(0.0);
    let corner_r = (style_tokens.paint.node_corner_radius / zoom).max(0.0);
    let border = fret_core::Edges::all(Px(border_w));
    let no_border = fret_core::Edges::all(Px(0.0));
    let corner_radii = fret_core::Corners::all(Px(corner_r));

    p.with_transform(transform, |p| {
        let drag_active = node_drag.is_some_and(NodeDragState::is_active);
        let (ddx, ddy) = node_drag
            .filter(|_| drag_active)
            .map(|d| node_drag_delta_canvas(view, d))
            .unwrap_or((0.0, 0.0));

        for d in draws.iter() {
            let mut rect = d.rect;
            if drag_active && node_drag.is_some_and(|drag| node_drag_contains(drag, d.id)) {
                rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 + ddx), Px(rect.origin.y.0 + ddy)),
                    rect.size,
                );
            }

            if !rects_intersect(cull, rect) {
                continue;
            }
            let selected = selected_nodes.iter().any(|id| *id == d.id);
            let hovered = hovered_node.is_some_and(|id| id == d.id);
            let border_color = if selected {
                border_selected
            } else if hovered {
                border_hover
            } else {
                border_base
            };
            let mut background = fill;
            let mut border_paint = fret_core::scene::Paint::Solid(border_color).into();

            if let Some(o) = paint_overrides
                .and_then(|p| p.node_paint_override(d.id))
                .map(|o| o.normalized())
            {
                if let Some(paint) = o.body_background {
                    background = paint;
                }
                if let Some(paint) = o.border_paint {
                    border_paint = paint;
                }
            }

            // Paint node background *below* wires, but keep the node border above wires.
            // This avoids the common “wire looks truncated” artifact when wires are drawn behind
            // a solid node quad in the paint-only skeleton.
            p.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background,
                border: no_border,
                border_paint: transparent_fill,
                corner_radii,
            });
            p.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: transparent_fill,
                border,
                border_paint,
                corner_radii,
            });
        }
    });
}

/// Paint-only declarative node-graph surface skeleton.
///
/// Notes:
/// - This is intentionally a minimal “M1 skeleton” surface: pan/zoom + a simple grid.
/// - It does **not** yet host node/edge portals or full editor interaction policy.
/// - Escape cancel clears the local pan-drag state, but cannot currently release pointer capture
///   from a key hook (see workstream contract gap log).
#[track_caller]
pub fn node_graph_surface_paint_only<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphSurfacePaintOnlyProps,
) -> AnyElement {
    let NodeGraphSurfacePaintOnlyProps {
        graph,
        view_state,
        controller,
        store,
        pointer_region,
        canvas,
        geometry_overrides,
        paint_overrides,
        portals_enabled,
        portal_max_nodes,
        cull_margin_screen_px,
        pan_button,
        min_zoom,
        max_zoom,
        wheel_zoom,
        pinch_zoom_speed,
        test_id,
    } = props;

    // Drag state is internal to the surface and stored in element state (uncontrolled model).
    let drag: Model<Option<DragState>> = use_uncontrolled_model(cx, || None);

    // Marquee selection drag state is internal to the surface (paint-only baseline).
    let marquee_drag: Model<Option<MarqueeDragState>> = use_uncontrolled_model(cx, || None);

    // Node dragging preview state (paint-only baseline).
    let node_drag: Model<Option<NodeDragState>> = use_uncontrolled_model(cx, || None);

    // Pending click-selection preview that should only commit on pointer-up / drag activation.
    let pending_selection: Model<Option<PendingSelectionState>> =
        use_uncontrolled_model(cx, || None);

    // Hover state is internal and intentionally not persisted in view state.
    let hovered_node: Model<Option<crate::core::NodeId>> = use_uncontrolled_model(cx, || None);
    // Scratch vec used by hit tests to avoid per-event allocations.
    let hit_scratch: Model<Vec<crate::core::NodeId>> = use_uncontrolled_model(cx, Vec::new);

    // Diagnostics-only: an internal paint overrides map so `diag.script_v2` can toggle a paint-only
    // revision bump without demo-specific command routing.
    let diag_paint_overrides: Model<Arc<NodeGraphPaintOverridesMap>> =
        use_uncontrolled_model(cx, || Arc::new(NodeGraphPaintOverridesMap::default()));
    let diag_paint_overrides_enabled: Model<bool> = use_uncontrolled_model(cx, || false);

    // Grid paint cache is a paint-only implementation detail. It uses a deterministic key so we can
    // gate steady-state behavior via diagnostics (no rebuild churn when view is stable).
    let grid_cache: Model<GridPaintCacheState> =
        use_uncontrolled_model(cx, GridPaintCacheState::default);

    // Derived geometry cache shared by node/edge builders.
    let derived_cache: Model<DerivedGeometryCacheState> =
        use_uncontrolled_model(cx, DerivedGeometryCacheState::default);

    // Edges paint cache is another paint-only implementation detail. Unlike the grid, it stores
    // precomputed path commands rather than prepared `PathId`s so hosted caches can manage
    // lifetime/budgets (ADR 0141 / ADR 0161 direction).
    let edges_cache: Model<EdgePaintCacheState> =
        use_uncontrolled_model(cx, EdgePaintCacheState::default);

    // Node chrome cache (paint-only baseline).
    let nodes_cache: Model<NodePaintCacheState> =
        use_uncontrolled_model(cx, NodePaintCacheState::default);

    // Optional, internal store for hosting portal subtrees and harvesting their last-known bounds.
    // This is intentionally local state (not part of `NodeGraphViewState`) while we iterate on the
    // declarative surface strategy.
    let portal_bounds_store: Model<PortalBoundsStore> =
        use_uncontrolled_model(cx, PortalBoundsStore::default);

    // Diagnostics-only toggles for portal hosting.
    let portal_debug_flags: Model<PortalDebugFlags> =
        use_uncontrolled_model(cx, PortalDebugFlags::default);

    // Hover anchor store used by overlays that should not depend on portal hosting caps.
    let hover_anchor_store: Model<HoverAnchorStore> =
        use_uncontrolled_model(cx, HoverAnchorStore::default);

    // Always observe the graph model so changes can invalidate the surface even when we don't need
    // to clone it on steady-state frames.
    cx.observe_model(&graph, Invalidation::Paint);

    // These models affect portal positioning, so treat them as layout-invalidating in a cached view.
    let drag_value = cx
        .get_model_copied(&drag, Invalidation::Layout)
        .unwrap_or(None);
    let panning = drag_value.is_some();

    let marquee_value = cx
        .get_model_cloned(&marquee_drag, Invalidation::Layout)
        .unwrap_or(None);
    let marquee_active = marquee_value.as_ref().is_some_and(|m| m.active);

    let node_drag_value = cx
        .get_model_cloned(&node_drag, Invalidation::Layout)
        .unwrap_or(None);
    let node_drag_armed = node_drag_value
        .as_ref()
        .is_some_and(NodeDragState::is_armed);
    let node_dragging = node_drag_value
        .as_ref()
        .is_some_and(NodeDragState::is_active);
    let pending_selection_value = cx
        .get_model_cloned(&pending_selection, Invalidation::Layout)
        .unwrap_or(None);

    let view_value = cx
        .get_model_cloned(&view_state, Invalidation::Layout)
        .unwrap_or_default();
    let view_for_paint = view_from_state(&view_value);
    let theme = Theme::global(&*cx.app).snapshot();
    let style_tokens = NodeGraphStyle::from_snapshot(theme.clone());
    let diag_keys_enabled = std::env::var("FRET_DIAG")
        .ok()
        .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0");
    let geometry_overrides = geometry_overrides.as_deref();
    let geometry_overrides_rev = geometry_overrides.map(|o| o.revision()).unwrap_or(0);
    let max_edge_interaction_width_override_px = geometry_overrides
        .map(|o| o.max_edge_interaction_width_override_px())
        .filter(|w| w.is_finite() && *w >= 0.0)
        .unwrap_or(0.0);
    let diag_paint_overrides_value = cx
        .get_model_cloned(&diag_paint_overrides, Invalidation::Paint)
        .unwrap_or_else(|| Arc::new(NodeGraphPaintOverridesMap::default()));
    let diag_paint_overrides_ref: NodeGraphPaintOverridesRef = diag_paint_overrides_value.clone();
    let paint_overrides_ref =
        paint_overrides.or_else(|| diag_keys_enabled.then_some(diag_paint_overrides_ref));
    let paint_overrides = paint_overrides_ref.as_deref();
    let paint_overrides_rev = paint_overrides.map(|o| o.revision()).unwrap_or(0);

    let graph_rev = graph.revision(&*cx.app).unwrap_or(0);
    let draw_order_hash = stable_hash_u64(2, &view_value.draw_order);
    let node_origin = view_value.interaction.node_origin;
    let resolved_interaction = view_value.resolved_interaction_state();

    // Attempt to rebuild the cached grid ops when the view/bounds key changes.
    // Bounds are initially learned from pointer hooks (and optionally from `last_bounds_for_element`).
    let mut grid_cache_value = cx
        .get_model_cloned(&grid_cache, Invalidation::Paint)
        .unwrap_or_default();

    if grid_cache_value.bounds.size.width.0 > 0.0
        && grid_cache_value.bounds.size.height.0 > 0.0
        && grid_cache_value.bounds.size.width.0.is_finite()
        && grid_cache_value.bounds.size.height.0.is_finite()
    {
        let key = grid_cache_key(grid_cache_value.bounds, view_for_paint, &style_tokens);
        if grid_cache_value.key != Some(key) {
            let ops = build_debug_grid_ops(grid_cache_value.bounds, view_for_paint, &style_tokens);
            let _ = cx.app.models_mut().update(&grid_cache, |st| {
                st.key = Some(key);
                st.rebuilds = st.rebuilds.saturating_add(1);
                st.ops = Some(ops.clone());
            });
            grid_cache_value.key = Some(key);
            grid_cache_value.rebuilds = grid_cache_value.rebuilds.saturating_add(1);
            grid_cache_value.ops = Some(ops);
        }
    }

    let grid_cached = grid_cache_value.ops.is_some();

    // Attempt to rebuild derived geometry + spatial index when their key changes.
    let mut derived_cache_value = cx
        .get_model_cloned(&derived_cache, Invalidation::Paint)
        .unwrap_or_default();
    {
        let key = derived_geometry_cache_key(
            graph_rev,
            view_for_paint.zoom,
            node_origin,
            &view_value.draw_order,
            &resolved_interaction,
            &style_tokens,
            geometry_overrides_rev,
            max_edge_interaction_width_override_px,
        );

        if derived_cache_value.key != Some(key) {
            let (geom, index) = cx
                .read_model_ref(&graph, Invalidation::Paint, |graph_value| {
                    let zoom = PanZoom2D::sanitize_zoom(view_for_paint.zoom, 1.0);
                    let z = zoom.max(1.0e-6);

                    let mut presenter = DefaultNodeGraphPresenter::default();
                    let geom = CanvasGeometry::build_with_presenter(
                        graph_value,
                        &view_value.draw_order,
                        &style_tokens,
                        zoom,
                        node_origin,
                        &mut presenter,
                        geometry_overrides,
                    );

                    let tuning = view_value.runtime_tuning.spatial_index;
                    let edge_aabb_pad_screen_px = tuning
                        .edge_aabb_pad_screen_px
                        .max(view_value.interaction.edge_interaction_width)
                        .max(max_edge_interaction_width_override_px)
                        .max(style_tokens.geometry.wire_width)
                        .max(0.0);
                    let cell_size_canvas = (tuning.cell_size_screen_px / z)
                        .max(tuning.min_cell_size_screen_px / z)
                        .max(1.0);
                    let max_hit_pad_canvas = (edge_aabb_pad_screen_px / z).max(0.0);

                    let index = CanvasSpatialDerived::build(
                        graph_value,
                        &geom,
                        zoom,
                        max_hit_pad_canvas,
                        cell_size_canvas,
                    );

                    (Arc::new(geom), Arc::new(index))
                })
                .unwrap_or_else(|_| {
                    (
                        Arc::new(CanvasGeometry::default()),
                        Arc::new(CanvasSpatialDerived::empty()),
                    )
                });

            let _ = cx.app.models_mut().update(&derived_cache, |st| {
                st.key = Some(key);
                st.rebuilds = st.rebuilds.saturating_add(1);
                st.geom = Some(geom.clone());
                st.index = Some(index.clone());
            });

            derived_cache_value.key = Some(key);
            derived_cache_value.rebuilds = derived_cache_value.rebuilds.saturating_add(1);
            derived_cache_value.geom = Some(geom);
            derived_cache_value.index = Some(index);
        }
    }

    let geom_cached = derived_cache_value.geom.is_some();

    // Attempt to rebuild cached node chrome draw data when the graph/zoom key changes.
    let mut nodes_cache_value = cx
        .get_model_cloned(&nodes_cache, Invalidation::Paint)
        .unwrap_or_default();
    {
        let derived_geometry_key = derived_cache_value.key.as_ref().map(|k| k.0).unwrap_or(0);
        let key = nodes_cache_key(
            graph_rev,
            view_for_paint.zoom,
            node_origin,
            draw_order_hash,
            derived_geometry_key,
        );
        if nodes_cache_value.key != Some(key) {
            let draws = if let Some(geom) = derived_cache_value.geom.as_deref() {
                let mut out = Vec::<NodeRectDraw>::new();
                out.reserve(geom.order.len().min(4096));
                for id in geom.order.iter().copied() {
                    let Some(node) = geom.nodes.get(&id) else {
                        continue;
                    };
                    out.push(NodeRectDraw {
                        id,
                        rect: node.rect,
                    });
                }
                Arc::new(out)
            } else {
                cx.read_model_ref(&graph, Invalidation::Paint, |graph_value| {
                    build_nodes_draws_paint_only(graph_value, view_for_paint.zoom)
                })
                .unwrap_or_else(|_| Arc::new(Vec::new()))
            };
            let _ = cx.app.models_mut().update(&nodes_cache, |st| {
                st.key = Some(key);
                st.rebuilds = st.rebuilds.saturating_add(1);
                st.draws = Some(draws.clone());
            });
            nodes_cache_value.key = Some(key);
            nodes_cache_value.rebuilds = nodes_cache_value.rebuilds.saturating_add(1);
            nodes_cache_value.draws = Some(draws);
        }
    }

    let nodes_cached = nodes_cache_value.draws.is_some();

    // Attempt to rebuild cached edges draw data when the graph/zoom key changes.
    let mut edges_cache_value = cx
        .get_model_cloned(&edges_cache, Invalidation::Paint)
        .unwrap_or_default();
    {
        let derived_geometry_key = derived_cache_value.key.as_ref().map(|k| k.0).unwrap_or(0);
        let key = edges_cache_key(
            graph_rev,
            view_for_paint.zoom,
            node_origin,
            draw_order_hash,
            derived_geometry_key,
        );
        if edges_cache_value.key != Some(key) {
            let geom_for_edges = derived_cache_value
                .geom
                .clone()
                .unwrap_or_else(|| Arc::new(CanvasGeometry::default()));
            let draws = cx
                .read_model_ref(&graph, Invalidation::Paint, |graph_value| {
                    build_edges_draws_paint_only(
                        graph_value,
                        graph_rev,
                        view_for_paint.zoom,
                        &geom_for_edges,
                        &style_tokens,
                    )
                })
                .unwrap_or_else(|_| Arc::new(Vec::new()));
            let _ = cx.app.models_mut().update(&edges_cache, |st| {
                st.key = Some(key);
                st.rebuilds = st.rebuilds.saturating_add(1);
                st.draws = Some(draws.clone());
            });
            edges_cache_value.key = Some(key);
            edges_cache_value.rebuilds = edges_cache_value.rebuilds.saturating_add(1);
            edges_cache_value.draws = Some(draws);
        }
    }

    let edges_cached = edges_cache_value.draws.is_some();
    let hovered_node_value = cx
        .get_model_copied(&hovered_node, Invalidation::Paint)
        .unwrap_or(None);
    let hovered = hovered_node_value.is_some();
    let effective_selected_nodes = marquee_value
        .as_ref()
        .filter(|marquee| marquee.active)
        .map(|marquee| marquee.preview_selected_nodes.to_vec())
        .or_else(|| {
            pending_selection_value
                .as_ref()
                .map(|pending| pending.nodes.to_vec())
        })
        .unwrap_or_else(|| view_value.selected_nodes.clone());
    let selected_nodes_len = effective_selected_nodes.len();
    let portal_fit_count = cx
        .app
        .models()
        .read(&portal_bounds_store, |st| st.fit_to_portals_count)
        .unwrap_or(0);
    let portal_fit_pending = cx
        .app
        .models()
        .read(&portal_bounds_store, |st| st.pending_fit_to_portals)
        .unwrap_or(false);
    let portal_union = cx
        .app
        .models()
        .read(&portal_bounds_store, |st| {
            let mut out: Option<Rect> = None;
            for rect in st.nodes_canvas_bounds.values().copied() {
                out = Some(match out {
                    Some(prev) => rect_union(prev, rect),
                    None => rect,
                });
            }
            out
        })
        .ok()
        .flatten();
    let (portal_union_w, portal_union_h) = portal_union
        .map(|r| (r.size.width.0, r.size.height.0))
        .unwrap_or((0.0, 0.0));
    let portal_bounds_entries = cx
        .app
        .models()
        .read(&portal_bounds_store, |st| st.nodes_canvas_bounds.len())
        .unwrap_or(0);
    let portals_disabled = cx
        .get_model_copied(&portal_debug_flags, Invalidation::Paint)
        .unwrap_or_default()
        .disable_portals;

    // Lightweight observability for “wires missing/truncated” reports:
    // estimate how many edges are drawn vs culled in the paint-only surface.
    //
    // Note: This is an approximation based on cached edge draws + best-effort bounds tracking
    // (grid cache bounds). It is intended for diagnostics gating, not as a correctness oracle.
    let (
        edges_paint_total,
        edges_paint_drawn,
        edges_paint_culled,
        edges_paint_dragged,
        edges_paint_missing_ports,
    ) = edges_cache_value
        .draws
        .as_deref()
        .map(|draws| {
            let mut total: u32 = draws.len() as u32;
            let mut drawn: u32 = 0;
            let mut culled: u32 = 0;
            let mut dragged: u32 = 0;
            let mut missing_ports: u32 = 0;

            let bounds = grid_cache_value.bounds;
            let view = PanZoom2D {
                pan: Point::new(Px(view_value.pan.x), Px(view_value.pan.y)),
                zoom: view_value.zoom,
            };
            let Some(cull) = canvas_viewport_rect(bounds, view, cull_margin_screen_px) else {
                return (total, drawn, culled, dragged, missing_ports);
            };

            let drag_active = node_drag_value
                .as_ref()
                .is_some_and(NodeDragState::is_active);
            let geom = derived_cache_value.geom.as_deref();

            for d in draws.iter() {
                let mut affected_by_drag = false;
                if drag_active {
                    let from_node = geom.and_then(|g| g.ports.get(&d.from)).map(|h| h.node);
                    let to_node = geom.and_then(|g| g.ports.get(&d.to)).map(|h| h.node);
                    affected_by_drag = from_node.is_some_and(|id| {
                        node_drag_value
                            .as_ref()
                            .is_some_and(|drag| node_drag_contains(drag, id))
                    }) || to_node.is_some_and(|id| {
                        node_drag_value
                            .as_ref()
                            .is_some_and(|drag| node_drag_contains(drag, id))
                    });
                }

                if affected_by_drag {
                    dragged += 1;
                    let ok_from = geom.and_then(|g| g.port_center(d.from)).is_some();
                    let ok_to = geom.and_then(|g| g.port_center(d.to)).is_some();
                    if ok_from && ok_to {
                        drawn += 1;
                    } else {
                        missing_ports += 1;
                    }
                    continue;
                }

                if !rects_intersect(cull, d.bbox) {
                    culled += 1;
                } else {
                    drawn += 1;
                }
            }

            // Keep `total` consistent even if the vec length exceeds u32 (shouldn't in practice).
            if draws.len() > (u32::MAX as usize) {
                total = u32::MAX;
            }
            (total, drawn, culled, dragged, missing_ports)
        })
        .unwrap_or((0, 0, 0, 0, 0));
    let edges_paint_ok =
        edges_paint_total > 0 && edges_paint_drawn > 0 && edges_paint_missing_ports == 0;
    let semantics_value: Arc<str> = Arc::from(format!(
        "panning {panning}; marquee_active:{marquee_active}; node_drag_armed:{node_drag_armed}; node_dragging:{node_dragging}; hovered_node:{hovered}; selected_nodes:{selected_nodes_len}; grid_cached:{grid_cached}; grid_rebuilds:{}; geom_cached:{geom_cached}; geom_rebuilds:{}; nodes_cached:{nodes_cached}; nodes_rebuilds:{}; edges_cached:{edges_cached}; edges_rebuilds:{}; edges_paint_total:{edges_paint_total}; edges_paint_drawn:{edges_paint_drawn}; edges_paint_culled:{edges_paint_culled}; edges_paint_dragged:{edges_paint_dragged}; edges_paint_missing_ports:{edges_paint_missing_ports}; edges_paint_ok:{edges_paint_ok}; paint_overrides_rev:{paint_overrides_rev}; view_pan:{:.2},{:.2}; view_zoom:{:.4}; portal_fit_count:{portal_fit_count}; portal_fit_pending:{portal_fit_pending}; portal_union_wh:{portal_union_w:.2}x{portal_union_h:.2}; portal_bounds_entries:{portal_bounds_entries}; portals_disabled:{portals_disabled};",
        grid_cache_value.rebuilds,
        derived_cache_value.rebuilds,
        nodes_cache_value.rebuilds,
        edges_cache_value.rebuilds,
        view_value.pan.x,
        view_value.pan.y,
        view_value.zoom
    ));
    let test_id = test_id.unwrap_or_else(|| Arc::<str>::from("node_graph.canvas"));

    cx.semantics_with_id(
        SemanticsProps {
            test_id: Some(test_id),
            value: Some(semantics_value),
            // Make the surface focusable so keyboard actions can route here after pointer-down.
            focusable: true,
            ..Default::default()
        },
        move |cx, element| {
            // Opportunistically learn bounds from the last recorded geometry. This is best-effort:
            // pointer hooks also stamp bounds (more immediate), but this helps keep resize behavior
            // roughly correct during the paint-only milestone.
            if let Some(bounds) = cx.last_bounds_for_element(element) {
                let _ = cx.app.models_mut().update(&grid_cache, |st| {
                    if st.bounds != bounds {
                        st.bounds = bounds;
                    }
                });
            }

            let drag_escape = drag.clone();
            let marquee_escape = marquee_drag.clone();
            let node_drag_escape = node_drag.clone();
            let pending_selection_escape = pending_selection.clone();
            let graph_debug = graph.clone();
            let view_zoom_kb = view_state.clone();
             let controller_zoom_kb = controller.clone();
            let store_zoom_kb = store.clone();
            let portal_bounds_for_fit = portal_bounds_store.clone();
            let portal_debug_for_keys = portal_debug_flags.clone();
            let diag_keys_enabled = diag_keys_enabled;
            let diag_paint_overrides_for_keys = diag_paint_overrides_value.clone();
            let diag_paint_overrides_enabled_for_keys = diag_paint_overrides_enabled.clone();
            let on_key_down_capture: OnKeyDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      key: fret_ui::action::KeyDownCx| {
                    if key.repeat || key.ime_composing {
                        return false;
                    }

                    if key.key == fret_core::KeyCode::Escape {
                        let handled = handle_declarative_escape_key_action_host(
                            host,
                            &drag_escape,
                            &marquee_escape,
                            &node_drag_escape,
                            &pending_selection_escape,
                        );
                        if handled {
                            host.request_redraw(action_cx.window);
                        }
                        return handled;
                    }

                    if !(key.modifiers.ctrl || key.modifiers.meta) {
                        return false;
                    }

                    if let Some(action) =
                        DeclarativeDiagKeyAction::from_key(diag_keys_enabled, key.key)
                    {
                        let handled = handle_declarative_diag_key_action_host(
                            host,
                            action,
                            &graph_debug,
                            &view_zoom_kb,
                            controller_zoom_kb.as_ref(),
                            store_zoom_kb.as_ref(),
                            &portal_bounds_for_fit,
                            &portal_debug_for_keys,
                            &diag_paint_overrides_for_keys,
                            &diag_paint_overrides_enabled_for_keys,
                        );
                        if handled {
                            host.request_redraw(action_cx.window);
                        }
                        return handled;
                    }

                    let Some(action) = DeclarativeKeyboardZoomAction::from_key(key.key) else {
                        return false;
                    };
                    let handled = handle_declarative_keyboard_zoom_action_host(
                        host,
                        action,
                        &view_zoom_kb,
                        controller_zoom_kb.as_ref(),
                        store_zoom_kb.as_ref(),
                        min_zoom,
                        max_zoom,
                    );
                    if handled {
                        host.request_redraw(action_cx.window);
                    }
                    handled
                },
            );
            cx.key_on_key_down_capture_for(element, on_key_down_capture);

            let view_pan_down = view_state.clone();
            let _controller_pan_down = controller.clone();
            let _store_pan_down = store.clone();
            let drag_start = drag.clone();
            let marquee_start = marquee_drag.clone();
            let node_drag_start = node_drag.clone();
            let pending_selection_start = pending_selection.clone();
            let grid_cache_bounds = grid_cache.clone();
            let focus_target = element;
            let derived_cache_for_down = derived_cache.clone();
            let hovered_for_down = hovered_node.clone();
            let hit_scratch_for_down = hit_scratch.clone();
            let on_pointer_down: OnPointerDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    host.request_focus(focus_target);

                    let bounds = host.bounds();
                    let _ = host.models_mut().update(&grid_cache_bounds, |state| {
                        if state.bounds != bounds {
                            state.bounds = bounds;
                        }
                    });

                    if down.button == pan_button {
                        let handled = begin_pan_pointer_down_action_host(
                            host,
                            &drag_start,
                            &marquee_start,
                            &node_drag_start,
                            down,
                        );
                        if handled {
                            host.capture_pointer();
                            notify_and_redraw_action_host(host, action_cx);
                        }
                        return handled;
                    }

                    if down.button != MouseButton::Left {
                        return false;
                    }

                    let snapshot = read_left_pointer_down_snapshot_action_host(
                        host,
                        &view_pan_down,
                        &derived_cache_for_down,
                        &hit_scratch_for_down,
                        down,
                        bounds,
                    );
                    let outcome = begin_left_pointer_down_action_host(
                        host,
                        &marquee_start,
                        &node_drag_start,
                        &pending_selection_start,
                        &hovered_for_down,
                        down,
                        &snapshot,
                    );
                    if outcome.capture_pointer() {
                        host.capture_pointer();
                    }
                    notify_and_redraw_action_host(host, action_cx);
                    true
                },
            );

            let view_pan = view_state.clone();
            let controller_pan = controller.clone();
            let store_pan = store.clone();
            let drag_move = drag.clone();
            let marquee_move = marquee_drag.clone();
            let node_drag_move = node_drag.clone();
            let pending_selection_move = pending_selection.clone();
            let grid_cache_bounds = grid_cache.clone();
            let derived_cache_for_hover = derived_cache.clone();
            let hovered_for_hover = hovered_node.clone();
            let hit_scratch_for_hover = hit_scratch.clone();
            let on_pointer_move: OnPointerMove = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      mv: fret_ui::action::PointerMoveCx| {
                    let bounds = host.bounds();
                    let _ = host.models_mut().update(&grid_cache_bounds, |st| {
                        if st.bounds != bounds {
                            st.bounds = bounds;
                        }
                    });

                    let drag = host.models_mut().read(&drag_move, |st| *st).ok().flatten();
                    let Some(mut drag) = drag else {
                        if let Some(outcome) = handle_node_drag_pointer_move_action_host(
                            host,
                            &node_drag_move,
                            &pending_selection_move,
                            &hovered_for_hover,
                            &view_pan,
                            controller_pan.as_ref(),
                            store_pan.as_ref(),
                            mv,
                        ) {
                            if outcome.capture_pointer {
                                host.capture_pointer();
                            }
                            if outcome.needs_layout_redraw {
                                // Node dragging moves portals (layout) and the canvas chrome (paint).
                                invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
                            }
                            return outcome.needs_layout_redraw;
                        }

                        if let Some(outcome) = handle_marquee_pointer_move_action_host(
                            host,
                            &marquee_move,
                            &hovered_for_hover,
                            &view_pan,
                            &derived_cache_for_hover,
                            mv,
                            bounds,
                        ) {
                            match outcome {
                                MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly => {
                                    host.release_pointer_capture();
                                    host.request_redraw(action_cx.window);
                                }
                                MarqueePointerMoveOutcome::NotifyRedraw => {
                                    notify_and_redraw_action_host(host, action_cx);
                                }
                            }
                            return true;
                        }

                        let changed = update_hovered_node_pointer_move_action_host(
                            host,
                            &hovered_for_hover,
                            &view_pan,
                            &derived_cache_for_hover,
                            &hit_scratch_for_hover,
                            mv,
                            bounds,
                        );
                        if changed {
                            invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Paint);
                        }
                        return changed;
                    };
                    if !mouse_buttons_contains(mv.buttons, drag.button) {
                        return false;
                    }

                    let dx = mv.position.x.0 - drag.last_pos.x.0;
                    let dy = mv.position.y.0 - drag.last_pos.y.0;
                    if !dx.is_finite() || !dy.is_finite() {
                        return false;
                    }

                    let updated = update_view_state_action_host(
                        host,
                        &view_pan,
                        controller_pan.as_ref(),
                        store_pan.as_ref(),
                        |state| {
                            apply_pan_by_screen_delta(state, dx, dy);
                        },
                    );
                    if !updated {
                        return false;
                    }

                    drag.last_pos = mv.position;
                    let _ = host.models_mut().update(&drag_move, |st| {
                        if let Some(st) = st.as_mut() {
                            *st = drag;
                        }
                    });

                    // Panning repositions portals (layout) and changes world mapping (hit-test + paint).
                    invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
                    true
                },
            );

            let drag_end = drag.clone();
            let marquee_end = marquee_drag.clone();
            let node_drag_end = node_drag.clone();
            let pending_selection_end = pending_selection.clone();
            let graph_commit = graph.clone();
            let view_commit = view_state.clone();
            let controller_commit = controller.clone();
            let store_commit = store.clone();
            let on_pointer_up: OnPointerUp = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      up: fret_ui::action::PointerUpCx| {
                    handle_declarative_pointer_up_action_host(
                        host,
                        action_cx,
                        up,
                        pan_button,
                        &drag_end,
                        &marquee_end,
                        &node_drag_end,
                        &pending_selection_end,
                        &graph_commit,
                        &view_commit,
                        controller_commit.as_ref(),
                        store_commit.as_ref(),
                    )
                },
            );

            let drag_cancel = drag.clone();
            let marquee_cancel = marquee_drag.clone();
            let node_drag_cancel = node_drag.clone();
            let pending_selection_cancel = pending_selection.clone();
            let on_pointer_cancel: OnPointerCancel = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      cancel: fret_ui::action::PointerCancelCx| {
                    handle_declarative_pointer_cancel_action_host(
                        host,
                        action_cx,
                        cancel,
                        &drag_cancel,
                        &marquee_cancel,
                        &node_drag_cancel,
                        &pending_selection_cancel,
                    )
                },
            );

            let view_zoom = view_state.clone();
            let controller_wheel = controller.clone();
            let store_wheel = store.clone();
            let grid_cache_bounds = grid_cache.clone();
            let on_wheel: OnWheel = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      wheel: fret_ui::action::WheelCx| {
                    if !(wheel.modifiers.ctrl || wheel.modifiers.meta) {
                        return false;
                    }

                    let Some(factor) = wheel_zoom_factor(
                        wheel.delta.y.0,
                        wheel_zoom.base,
                        wheel_zoom.step,
                        wheel_zoom.speed,
                    ) else {
                        return false;
                    };

                    let bounds = host.bounds();
                    let _ = host.models_mut().update(&grid_cache_bounds, |st| {
                        if st.bounds != bounds {
                            st.bounds = bounds;
                        }
                    });
                    let updated = update_view_state_action_host(
                        host,
                        &view_zoom,
                        controller_wheel.as_ref(),
                        store_wheel.as_ref(),
                        |state| {
                            let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
                            let new_zoom = (zoom * factor).clamp(min_zoom, max_zoom);
                            apply_zoom_about_screen_point(
                                state,
                                bounds,
                                wheel.position,
                                new_zoom,
                                min_zoom,
                                max_zoom,
                            );
                        },
                    );
                    if !updated {
                        return false;
                    }

                    invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
                    true
                },
            );

            let view_pinch = view_state.clone();
            let controller_pinch = controller.clone();
            let store_pinch = store.clone();
            let grid_cache_bounds = grid_cache.clone();
            let on_pinch: OnPinchGesture = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      pinch: fret_ui::action::PinchGestureCx| {
                    if !pinch.delta.is_finite() {
                        return false;
                    }
                    let delta = pinch.delta * pinch_zoom_speed;
                    if delta.abs() <= 1.0e-9 {
                        return false;
                    }

                    let bounds = host.bounds();
                    let _ = host.models_mut().update(&grid_cache_bounds, |st| {
                        if st.bounds != bounds {
                            st.bounds = bounds;
                        }
                    });
                    let updated = update_view_state_action_host(
                        host,
                        &view_pinch,
                        controller_pinch.as_ref(),
                        store_pinch.as_ref(),
                        |state| {
                            let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
                            let factor = (1.0 + delta).max(1.0e-6);
                            let new_zoom = (zoom * factor).clamp(min_zoom, max_zoom);
                            apply_zoom_about_screen_point(
                                state,
                                bounds,
                                pinch.position,
                                new_zoom,
                                min_zoom,
                                max_zoom,
                            );
                        },
                    );
                    if !updated {
                        return false;
                    }

                    invalidate_notify_and_redraw_pointer_action_host(host, action_cx, Invalidation::Layout);
                    true
                },
            );

            vec![cx.pointer_region(pointer_region, move |cx| {
                cx.pointer_region_on_pointer_down(on_pointer_down);
                cx.pointer_region_on_pointer_move(on_pointer_move);
                cx.pointer_region_on_pointer_up(on_pointer_up);
                cx.pointer_region_on_pointer_cancel(on_pointer_cancel);
                cx.pointer_region_on_wheel(on_wheel);
                cx.pointer_region_on_pinch_gesture(on_pinch);

                let view_for_paint = view_for_paint;
                let grid_ops = grid_cache_value.ops.clone();
                let node_draws = nodes_cache_value.draws.clone();
                let edge_draws = edges_cache_value.draws.clone();
                let geom_for_paint = derived_cache_value.geom.clone();
                let style_tokens = style_tokens.clone();
                let hovered_node_value = hovered_node_value;
                let selected_nodes = effective_selected_nodes.clone();
                let marquee_value = marquee_value.clone();
                let node_drag_value = node_drag_value.clone();
                let hover_anchor_store = hover_anchor_store.clone();
                let portals_disabled = portals_disabled;

                // Clone values needed by the paint closure so the originals remain available for
                // portal hosting below.
                let node_draws_for_paint = node_draws.clone();
                let edge_draws_for_paint = edge_draws.clone();
                let style_tokens_for_paint = style_tokens.clone();
                let selected_nodes_for_paint = selected_nodes.clone();
                let node_drag_for_paint = node_drag_value.clone();
                let paint_overrides_for_paint = paint_overrides_ref.clone();
                // Ensure canvas paint-cache invalidation tracks interactive state changes even when
                // the surface rerenders outside the canvas node (e.g. portal layout updates).
                let graph_model_id = graph.id();
                let view_state_model_id = view_state.id();
                let hovered_node_model_id = hovered_node.id();
                let node_drag_model_id = node_drag.clone().id();
                let marquee_drag_model_id = marquee_drag.clone().id();
                let canvas = cx.canvas(canvas, move |p| {
                    // Declare paint dependencies for paint-cache invalidation. Without this, the
                    // canvas node can replay cached ops while portals update, which reads as
                    // "drag not following" (chrome decouples from labels).
                    p.observe_model_id(graph_model_id, Invalidation::Paint);
                    p.observe_model_id(view_state_model_id, Invalidation::Paint);
                    p.observe_model_id(hovered_node_model_id, Invalidation::Paint);
                    p.observe_model_id(node_drag_model_id, Invalidation::Paint);
                    p.observe_model_id(marquee_drag_model_id, Invalidation::Paint);

                    paint_debug_grid_cached(p, view_for_paint, grid_ops.clone(), &style_tokens_for_paint);
                    paint_nodes_cached(
                        p,
                        view_for_paint,
                        cull_margin_screen_px,
                        node_draws_for_paint.clone(),
                        &style_tokens_for_paint,
                        hovered_node_value,
                        selected_nodes_for_paint.as_slice(),
                        node_drag_for_paint.as_ref(),
                        paint_overrides_for_paint.as_deref(),
                    );
                    paint_edges_cached(
                        p,
                        view_for_paint,
                        cull_margin_screen_px,
                        edge_draws_for_paint.clone(),
                        geom_for_paint.clone(),
                        node_drag_for_paint.as_ref(),
                        &style_tokens_for_paint,
                        paint_overrides_for_paint.as_deref(),
                    );
                });

                let mut out: Vec<AnyElement> = Vec::new();
                out.push(canvas);
                let mut overlay_children: Vec<AnyElement> = Vec::new();
                let mut hovered_portal_hosted: bool = false;

                if portals_enabled && portal_max_nodes > 0 && !portals_disabled {
                    // Portals are positioned in screen space (semantic zoom) and gated off from
                    // hit-testing so the surface-level pointer region remains the sole input
                    // router during this milestone.
                    let bounds = grid_cache_value.bounds;
                    let view = view_for_paint;
                    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);

                    if bounds.size.width.0.is_finite()
                        && bounds.size.height.0.is_finite()
                        && bounds.size.width.0 > 0.0
                        && bounds.size.height.0 > 0.0
                    {
                        let cull_canvas = canvas_viewport_rect(bounds, view, cull_margin_screen_px);

                        let drag_active = node_drag_value
                            .as_ref()
                            .is_some_and(NodeDragState::is_active);
                        let (ddx, ddy) = node_drag_value
                            .as_ref()
                            .filter(|_| drag_active)
                            .map(|d| node_drag_delta_canvas(view, d))
                            .unwrap_or((0.0, 0.0));

                        let hovered = hovered_node_value;
                        let selected = selected_nodes.clone();
                        let dragged_nodes = node_drag_value
                            .as_ref()
                            .filter(|d| d.is_active())
                            .map(|d| d.nodes_sorted.clone());

                        let portal_bounds_store = portal_bounds_store.clone();
                        let portal_infos: Vec<PortalLabelInfo> = cx
                            .read_model_ref(&graph, Invalidation::Paint, |graph_value| {
                                let Some(draws) = node_draws.as_deref() else {
                                    return Vec::new();
                                };

                                let mut infos: Vec<PortalLabelInfo> = Vec::new();
                                for d in draws.iter() {
                                    if infos.len() >= portal_max_nodes {
                                        break;
                                    }

                                    if let Some(cull) = cull_canvas
                                        && !rects_intersect(cull, d.rect)
                                    {
                                        continue;
                                    }

                                    let mut origin_canvas = d.rect.origin;
                                    if drag_active
                                        && let Some(nodes) = dragged_nodes.as_deref()
                                        && nodes.binary_search(&d.id).is_ok()
                                    {
                                        origin_canvas = Point::new(
                                            Px(origin_canvas.x.0 + ddx),
                                            Px(origin_canvas.y.0 + ddy),
                                        );
                                    }

                                    let origin_screen =
                                        view.canvas_to_screen(bounds, origin_canvas);
                                    let left = Px(origin_screen.x.0 - bounds.origin.x.0);
                                    let top = Px(origin_screen.y.0 - bounds.origin.y.0);
                                    let width = Px((d.rect.size.width.0 * zoom).max(0.0));

                                    if !left.0.is_finite()
                                        || !top.0.is_finite()
                                        || !width.0.is_finite()
                                    {
                                        continue;
                                    }

                                    let label: Arc<str> = graph_value
                                        .nodes
                                        .get(&d.id)
                                        .map(|n| Arc::<str>::from(n.kind.0.as_str()))
                                        .unwrap_or_else(|| Arc::<str>::from("node"));

                                    let (ports_in, ports_out) = graph_value
                                        .nodes
                                        .get(&d.id)
                                        .map(|n| {
                                            let mut ports_in = 0u32;
                                            let mut ports_out = 0u32;
                                            for pid in n.ports.iter() {
                                                let Some(port) = graph_value.ports.get(pid) else {
                                                    continue;
                                                };
                                                match port.dir {
                                                    crate::core::PortDirection::In => ports_in += 1,
                                                    crate::core::PortDirection::Out => {
                                                        ports_out += 1
                                                    }
                                                }
                                            }
                                            (ports_in, ports_out)
                                        })
                                        .unwrap_or((0, 0));

                                    let selected = selected.iter().any(|id| *id == d.id);
                                    let hovered = hovered.is_some_and(|id| id == d.id);

                                    infos.push(PortalLabelInfo {
                                        id: d.id,
                                        left,
                                        top,
                                        width,
                                        height: Px(38.0),
                                        label,
                                        ports_in,
                                        ports_out,
                                        selected,
                                        hovered,
                                    });
                                }

                                infos
                            })
                            .unwrap_or_default();

                        if !portal_infos.is_empty() {
                            hovered_portal_hosted = hovered_node_value
                                .is_some_and(|id| portal_infos.iter().any(|p| p.id == id));

                            // Best-effort prune for nodes that are no longer hosted this frame. Bounds are
                            // frame-lagged by contract, so removals may trail unmount by one frame.
                            let visible: std::collections::BTreeSet<crate::core::NodeId> =
                                portal_infos.iter().map(|p| p.id).collect();
                            let should_prune = cx
                                .app
                                .models()
                                .read(&portal_bounds_store, |st| {
                                    st.nodes_canvas_bounds
                                        .keys()
                                        .any(|id| !visible.contains(id))
                                })
                                .unwrap_or(false);
                            if should_prune {
                                let _ = cx.app.models_mut().update(&portal_bounds_store, |st| {
                                    st.nodes_canvas_bounds.retain(|id, _| visible.contains(id));
                                });
                                cx.request_frame();
                            }

                            for (ordinal, info) in portal_infos.iter().cloned().enumerate() {
                                let style_tokens = style_tokens.clone();
                                let theme = theme.clone();
                                let portal_bounds_store = portal_bounds_store.clone();
                                overlay_children.push(cx.keyed(
                                    ("fret-node.portal-label.v1", info.id),
                                    move |cx| {
                                        let bounds = bounds;
                                        let view = view;
                                        let mut query = LayoutQueryRegionProps {
                                            name: Some("fret-node.portal.node_label.v1".into()),
                                            ..Default::default()
                                        };
                                        // Make the query region itself the absolutely positioned item.
                                        // Otherwise, the region's bounds would not include its absolute
                                        // descendants, and harvested portal bounds would collapse to 0x0.
                                        query.layout.position = PositionStyle::Absolute;
                                        query.layout.inset.left = Some(info.left).into();
                                        query.layout.inset.top = Some(info.top).into();
                                        query.layout.size.width = Length::Px(info.width);
                                        query.layout.size.height = Length::Px(info.height);

                                        cx.layout_query_region_with_id(query, move |cx, element| {
                                            let visual_bounds = cx
                                                .last_visual_bounds_for_element(element)
                                                .or_else(|| cx.last_bounds_for_element(element));

                                            if let Some(visual_bounds) = visual_bounds {
                                                let canvas_bounds = screen_rect_to_canvas_rect(
                                                    bounds,
                                                    view,
                                                    visual_bounds,
                                                );

                                                let should_update = cx
                                                    .app
                                                    .models()
                                                    .read(&portal_bounds_store, |st| {
                                                        let Some(prev) =
                                                            st.nodes_canvas_bounds.get(&info.id)
                                                        else {
                                                            return true;
                                                        };
                                                        !rect_approx_eq(*prev, canvas_bounds, 0.25)
                                                    })
                                                    .unwrap_or(true);

                                                if should_update {
                                                    let _ = cx.app.models_mut().update(
                                                        &portal_bounds_store,
                                                        |st| {
                                                            st.nodes_canvas_bounds
                                                                .insert(info.id, canvas_bounds);
                                                        },
                                                    );
                                                    cx.request_frame();
                                                }
                                            }

                                            let mut p = ContainerProps::default();
                                            p.layout.size.width = Length::Fill;
                                            p.layout.size.height = Length::Fill;
                                            p.padding =
                                                SpacingEdges::all(SpacingLength::Px(Px(4.0)));
                                            p.snap_to_device_pixels = true;
                                            p.background = Some(Color {
                                                a: if info.selected {
                                                    0.98
                                                } else if info.hovered {
                                                    0.95
                                                } else {
                                                    0.92
                                                },
                                                ..style_tokens.paint.node_background
                                            });
                                            p.border = fret_core::Edges::all(Px(1.0));
                                            p.border_color = Some(Color {
                                                a: 0.35,
                                                ..style_tokens.paint.node_border
                                            });

                                            let header_color = theme.color_token("card-foreground");
                                            let ports_color = theme.color_token("muted-foreground");

                                            let header = {
                                                let mut props = TextProps::new(info.label.clone());
                                                props.color = Some(header_color);
                                                cx.text_props(props).attach_semantics(
                                                    SemanticsDecoration::default().test_id(
                                                        Arc::<str>::from(format!(
                                                            "node_graph.portal.node.{ordinal}.header"
                                                        )),
                                                    ),
                                                )
                                            };
                                            let ports = {
                                                let mut props = TextProps::new(Arc::<str>::from(
                                                    format!(
                                                        "in:{} out:{}",
                                                        info.ports_in, info.ports_out
                                                    ),
                                                ));
                                                props.color = Some(ports_color);
                                                cx.text_props(props).attach_semantics(
                                                    SemanticsDecoration::default().test_id(
                                                        Arc::<str>::from(format!(
                                                            "node_graph.portal.node.{ordinal}.ports"
                                                        )),
                                                    ),
                                                )
                                            };

                                            vec![
                                                cx.container(p, move |cx| {
                                                    let mut col = ColumnProps::default();
                                                    col.layout.size.width = Length::Fill;
                                                    col.layout.size.height = Length::Fill;
                                                    col.gap = SpacingLength::Px(Px(2.0));
                                                    vec![cx.column(col, move |_cx| {
                                                        vec![header, ports]
                                                    })]
                                                })
                                                .attach_semantics(
                                                    SemanticsDecoration::default()
                                                        .test_id(Arc::<str>::from(format!(
                                                            "node_graph.portal.node.{ordinal}"
                                                        )))
                                                        .value(Arc::<str>::from(format!(
                                                            "node_id={}; ports_in={} ports_out={}",
                                                            info.id.0,
                                                            info.ports_in,
                                                            info.ports_out
                                                        ))),
                                                ),
                                            ]
                                        })
                                    },
                                ));
                            }
                        }
                    }
                }

                // Keep a best-effort hovered bounds record independent of portal hosting caps.
                if let Some(hovered_id) = hovered_node_value {
                    if let Some(draws) = node_draws.as_deref()
                        && let Some(draw) = draws.iter().find(|d| d.id == hovered_id)
                    {
                        let mut rect = draw.rect;
                        let drag_active = node_drag_value
                            .as_ref()
                            .is_some_and(NodeDragState::is_active);
                        if drag_active
                            && node_drag_value
                                .as_ref()
                                .is_some_and(|d| node_drag_contains(d, hovered_id))
                        {
                            let (ddx, ddy) =
                                node_drag_delta_canvas(view_for_paint, node_drag_value.as_ref().unwrap());
                            rect.origin = Point::new(Px(rect.origin.x.0 + ddx), Px(rect.origin.y.0 + ddy));
                        }

                        let should_update = cx
                            .app
                            .models()
                            .read(&hover_anchor_store, |st| {
                                if st.hovered_id != Some(hovered_id) {
                                    return true;
                                }
                                let Some(prev) = st.hovered_canvas_bounds else {
                                    return true;
                                };
                                !rect_approx_eq(prev, rect, 0.25)
                            })
                            .unwrap_or(true);

                        if should_update {
                            let _ = cx.app.models_mut().update(&hover_anchor_store, |st| {
                                st.hovered_id = Some(hovered_id);
                                st.hovered_canvas_bounds = Some(rect);
                            });
                            cx.request_frame();
                        }
                    }
                } else {
                    let should_clear = cx
                        .app
                        .models()
                        .read(&hover_anchor_store, |st| st.hovered_id.is_some())
                        .unwrap_or(false);
                    if should_clear {
                        let _ = cx.app.models_mut().update(&hover_anchor_store, |st| {
                            st.hovered_id = None;
                            st.hovered_canvas_bounds = None;
                        });
                        cx.request_frame();
                    }
                }

                // Hover tooltip is an incremental declarative overlay example. It consumes the
                // retained-like hover state computed from paint-only hit tests, but renders the
                // chrome as a normal element subtree (screen-space, semantic zoom).
                //
                // Positioning prefers `PortalBoundsStore` so this milestone demonstrates the full
                // bounds-query loop (host subtree → harvest bounds → consume bounds).
                // Keep this diagnostics-only. When portals are hosted, avoid duplicating portal
                // label content: the tooltip should focus on debug details (anchor source, ids).
                if diag_keys_enabled
                    && !panning
                    && !marquee_active
                    && !node_dragging
                {
                    if let Some(hovered_id) = hovered_node_value {
                        let bounds = grid_cache_value.bounds;
                        let view = view_for_paint;
                        let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);

                        if bounds.size.width.0.is_finite()
                            && bounds.size.height.0.is_finite()
                            && bounds.size.width.0 > 0.0
                            && bounds.size.height.0 > 0.0
                        {
                            let portal_canvas_bounds = if portals_disabled {
                                None
                            } else {
                                cx.app
                                    .models()
                                    .read(&portal_bounds_store, |st| {
                                        st.nodes_canvas_bounds.get(&hovered_id).copied()
                                    })
                                    .ok()
                                    .flatten()
                            };

                            let anchor_canvas_bounds = cx
                                .app
                                .models()
                                .read(&hover_anchor_store, |st| {
                                    if st.hovered_id == Some(hovered_id) {
                                        st.hovered_canvas_bounds
                                    } else {
                                        None
                                    }
                                })
                                .ok()
                                .flatten();

                            let tooltip_origin_screen_width_source: Option<(Point, Px, Arc<str>)> =
                                if let Some(portal_canvas_bounds) = portal_canvas_bounds {
                                    Some((
                                        view.canvas_to_screen(bounds, portal_canvas_bounds.origin),
                                        Px((portal_canvas_bounds.size.width.0 * zoom).max(0.0)),
                                        Arc::<str>::from("portal_bounds_store"),
                                    ))
                                } else if let Some(anchor_canvas_bounds) = anchor_canvas_bounds {
                                    Some((
                                        view.canvas_to_screen(bounds, anchor_canvas_bounds.origin),
                                        Px((anchor_canvas_bounds.size.width.0 * zoom).max(0.0)),
                                        Arc::<str>::from("hover_anchor_store"),
                                    ))
                                } else {
                                    None
                                };

                            if let Some((origin_screen, width, source)) =
                                tooltip_origin_screen_width_source
                            {
                                let left = Px(origin_screen.x.0 - bounds.origin.x.0);
                                let mut top = Px(origin_screen.y.0 - bounds.origin.y.0 - 30.0);
                                if top.0 < 0.0 {
                                    top = Px(origin_screen.y.0 - bounds.origin.y.0 + 6.0);
                                }

                                if left.0.is_finite()
                                    && top.0.is_finite()
                                    && width.0.is_finite()
                                    && width.0 > 0.0
                                {
                                    let hovered_label = cx
                                        .read_model_ref(&graph, Invalidation::Paint, |g| {
                                            g.nodes
                                                .get(&hovered_id)
                                                .map(|n| Arc::<str>::from(n.kind.0.as_str()))
                                        })
                                        .ok()
                                        .flatten()
                                        .unwrap_or_else(|| Arc::<str>::from("node"));

                                    let (ports_in, ports_out) = cx
                                        .read_model_ref(&graph, Invalidation::Paint, |g| {
                                            g.nodes.get(&hovered_id).map(|n| {
                                                let mut ports_in = 0u32;
                                                let mut ports_out = 0u32;
                                                for pid in n.ports.iter() {
                                                    let Some(port) = g.ports.get(pid) else {
                                                        continue;
                                                    };
                                                    match port.dir {
                                                        crate::core::PortDirection::In => {
                                                            ports_in += 1
                                                        }
                                                        crate::core::PortDirection::Out => {
                                                            ports_out += 1
                                                        }
                                                    }
                                                }
                                                (ports_in, ports_out)
                                            })
                                        })
                                        .ok()
                                        .flatten()
                                        .unwrap_or((0, 0));

                                    let style_tokens = style_tokens.clone();
                                    overlay_children.push(cx.keyed(
                                        ("fret-node.portal.tooltip.v1", hovered_id),
                                        move |cx| {
                                            let mut p = ContainerProps::default();
                                            p.layout.position = PositionStyle::Absolute;
                                            p.layout.inset.left = Some(left).into();
                                            p.layout.inset.top = Some(top).into();
                                            p.layout.size.width = Length::Px(width);
                                            p.layout.size.height = Length::Px(Px(30.0));
                                            p.padding =
                                                SpacingEdges::all(SpacingLength::Px(Px(4.0)));
                                            p.snap_to_device_pixels = true;
                                            p.background = Some(Color {
                                                a: 0.26,
                                                ..style_tokens.paint.node_background
                                            });
                                            p.border = fret_core::Edges::all(Px(1.0));
                                            p.border_color = Some(Color {
                                                a: 0.35,
                                                ..style_tokens.paint.node_border
                                            });

                                            let source_for_text = source.clone();
                                            cx.container(p, move |cx| {
                                                let mut col = ColumnProps::default();
                                                col.layout.size.width = Length::Fill;
                                                col.layout.size.height = Length::Fill;
                                                col.gap = SpacingLength::Px(Px(2.0));
                                                vec![cx.column(col, move |cx| {
                                                    let mut lines: Vec<AnyElement> = vec![
                                                        cx.text(Arc::<str>::from(format!(
                                                            "id:{}",
                                                            hovered_id.0
                                                        ))),
                                                        cx.text(Arc::<str>::from(format!(
                                                            "source:{source_for_text}"
                                                        ))),
                                                    ];

                                                    // Only include the label/port summary when it won't read as
                                                    // "duplicated text" over a hosted portal label.
                                                    if !hovered_portal_hosted {
                                                        lines.push(cx.text(hovered_label.clone()));
                                                        lines.push(cx.text(Arc::<str>::from(
                                                            format!("in:{} out:{}", ports_in, ports_out),
                                                        )));
                                                    }
                                                    lines
                                                })]
                                            })
                                            .attach_semantics(
                                                SemanticsDecoration::default()
                                                    .test_id("node_graph.portal.tooltip")
                                                    .value(Arc::<str>::from(format!(
                                                        "source={source}; node_id={}; ports_in={} ports_out={}",
                                                        hovered_id.0, ports_in, ports_out
                                                    ))),
                                            )
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }

                // Diagnostics-only: if Ctrl+9 was pressed before portal bounds arrived, apply the
                // fit request once we have harvested at least one portal rect.
                //
                // This keeps scripted gates deterministic without requiring arbitrary sleeps.
                let pending_fit = cx
                    .app
                    .models()
                    .read(&portal_bounds_store, |st| st.pending_fit_to_portals)
                    .unwrap_or(false);
                if pending_fit && portals_enabled && !portals_disabled {
                    let bounds = grid_cache_value.bounds;
                    let bounds_valid = bounds.size.width.0.is_finite()
                        && bounds.size.height.0.is_finite()
                        && bounds.size.width.0 > 0.0
                        && bounds.size.height.0 > 0.0;
                    let target = cx
                        .app
                        .models()
                        .read(&portal_bounds_store, |st| {
                            let mut out: Option<Rect> = None;
                            for rect in st.nodes_canvas_bounds.values().copied() {
                                out = Some(match out {
                                    Some(prev) => rect_union(prev, rect),
                                    None => rect,
                                });
                            }
                            out
                        })
                        .ok()
                        .flatten();

                    if bounds_valid && let Some(target) = target {
                        let applied = update_view_state_ui_host(
                            cx.app,
                            &view_state,
                            controller.as_ref(),
                            store.as_ref(),
                            |state| {
                                let _ = apply_fit_view_to_canvas_rect(
                                    state, bounds, target, 24.0, min_zoom, max_zoom,
                                );
                            },
                        );

                        if applied {
                            let _ = cx.app.models_mut().update(&portal_bounds_store, |st| {
                                st.fit_to_portals_count = st.fit_to_portals_count.saturating_add(1);
                                st.pending_fit_to_portals = false;
                            });
                            // Ensure the semantics `value` string updates promptly for script gates.
                            cx.request_frame();
                        }
                    }

                    // Keep polling until bounds are available, but only while the request is armed.
                    let still_pending = cx
                        .app
                        .models()
                        .read(&portal_bounds_store, |st| st.pending_fit_to_portals)
                        .unwrap_or(false);
                    if still_pending {
                        cx.request_frame();
                    }
                }

                // Marquee chrome must render above portals, so keep it as a normal overlay element
                // instead of drawing it in the canvas paint pass.
                if let Some(marquee) = marquee_value.as_ref()
                    && marquee.active
                {
                    let bounds = grid_cache_value.bounds;
                    let rect = marquee_rect_screen(marquee);
                    if bounds.size.width.0.is_finite()
                        && bounds.size.height.0.is_finite()
                        && bounds.size.width.0 > 0.0
                        && bounds.size.height.0 > 0.0
                        && rect.size.width.0 > 0.0
                        && rect.size.height.0 > 0.0
                    {
                        // Clamp to bounds to avoid accidental giant rects in scripted input.
                        let x0 = rect.origin.x.0.max(bounds.origin.x.0);
                        let y0 = rect.origin.y.0.max(bounds.origin.y.0);
                        let x1 = (rect.origin.x.0 + rect.size.width.0)
                            .min(bounds.origin.x.0 + bounds.size.width.0);
                        let y1 = (rect.origin.y.0 + rect.size.height.0)
                            .min(bounds.origin.y.0 + bounds.size.height.0);
                        let rect = Rect::new(
                            Point::new(Px(x0), Px(y0)),
                            fret_core::Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
                        );

                        if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                            let left = Px(rect.origin.x.0 - bounds.origin.x.0);
                            let top = Px(rect.origin.y.0 - bounds.origin.y.0);

                            let mut p = ContainerProps::default();
                            p.layout.position = PositionStyle::Absolute;
                            p.layout.inset.left = Some(left).into();
                            p.layout.inset.top = Some(top).into();
                            p.layout.size.width = Length::Px(rect.size.width);
                            p.layout.size.height = Length::Px(rect.size.height);
                            p.background = Some(style_tokens.paint.marquee_fill);
                            p.border = fret_core::Edges::all(Px(style_tokens
                                .paint
                                .marquee_border_width
                                .max(0.0)));
                            p.border_color = Some(style_tokens.paint.marquee_border);
                            p.snap_to_device_pixels = true;

                            overlay_children.push(
                                cx.keyed("fret-node.marquee.overlay.v1", move |cx| {
                                    cx.container(p, |_cx| std::iter::empty())
                                }),
                            );
                        }
                    }
                }

                if !overlay_children.is_empty() {
                    let mut layer = ContainerProps::default();
                    layer.layout = LayoutStyle::default();
                    layer.layout.size.width = Length::Fill;
                    layer.layout.size.height = Length::Fill;
                    layer.layout.position = PositionStyle::Relative;

                    out.push(cx.hit_test_gate(false, move |cx| {
                        vec![cx.container(layer, move |_cx| overlay_children)]
                    }));
                }

                out
            })]
        },
    )
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    use fret_canvas::view::PanZoom2D;
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, MouseButtons, Point, PointerId, PointerType, Px, Rect,
    };
    use fret_runtime::{
        ClipboardToken, DragKindId, DragSession, Effect, Model, ModelStore, ShareSheetToken,
        TickId, TimerToken,
    };
    use fret_ui::action::UiActionHost;

    use super::{
        DeclarativeDiagKeyAction, DeclarativeDiagViewPreset, DeclarativeKeyboardZoomAction,
        DerivedGeometryCacheState, DragState, Invalidation, LeftPointerDownOutcome,
        LeftPointerDownSnapshot, LeftPointerReleaseOutcome, MarqueeDragState,
        MarqueePointerMoveOutcome, NodeDragPhase, NodeDragPointerMoveOutcome,
        NodeDragReleaseOutcome, NodeDragState, PendingSelectionState, PortalBoundsStore,
        PortalDebugFlags, apply_declarative_diag_view_preset_action_host,
        begin_left_pointer_down_action_host, begin_pan_pointer_down_action_host,
        build_click_selection_preview_nodes, build_diag_normalize_visible_node_transaction,
        build_diag_nudge_visible_node_transaction, build_marquee_preview_selected_nodes,
        build_node_drag_transaction, commit_graph_transaction,
        commit_marquee_selection_action_host, commit_node_drag_transaction,
        commit_pending_selection_action_host, complete_left_pointer_release_action_host,
        complete_node_drag_release_action_host, escape_cancel_declarative_interactions_action_host,
        handle_declarative_diag_key_action_host, handle_declarative_keyboard_zoom_action_host,
        handle_declarative_pointer_cancel_action_host, handle_declarative_pointer_up_action_host,
        handle_marquee_left_pointer_release_action_host, handle_marquee_pointer_move_action_host,
        handle_node_drag_left_pointer_release_action_host,
        handle_node_drag_pointer_move_action_host,
        handle_pending_selection_left_pointer_release_action_host, node_drag_commit_delta,
        pointer_cancel_declarative_interactions_action_host, pointer_crossed_threshold,
        update_hovered_node_pointer_move_action_host,
    };
    use crate::core::{
        CanvasPoint, CanvasSize, EdgeId, Graph, GraphId, GroupId, Node, NodeId, NodeKindKey,
    };
    use crate::io::NodeGraphViewState;
    use crate::ops::GraphOp;
    use crate::runtime::callbacks::{
        NodeGraphCommitCallbacks, NodeGraphGestureCallbacks, NodeGraphViewCallbacks,
        SelectionChange, install_callbacks,
    };
    use crate::runtime::changes::NodeGraphChanges;
    use crate::runtime::store::NodeGraphStore;
    use crate::ui::NodeGraphController;
    use crate::ui::paint_overrides::{NodeGraphPaintOverrides, NodeGraphPaintOverridesMap};
    use serde_json::Value;

    #[derive(Default)]
    struct TestActionHostImpl {
        models: ModelStore,
        effects: Vec<Effect>,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        redraw_requests: Vec<AppWindowId>,
        notifications: Vec<fret_ui::action::ActionCx>,
        invalidations: Vec<Invalidation>,
        capture_pointer_count: usize,
        release_pointer_capture_count: usize,
        requested_focus: Vec<fret_ui::GlobalElementId>,
        cursor_icons: Vec<fret_core::CursorIcon>,
        prevented_defaults: Vec<fret_runtime::DefaultAction>,
        bounds: Rect,
    }

    impl UiActionHost for TestActionHostImpl {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraw_requests.push(window);
        }

        fn notify(&mut self, cx: fret_ui::action::ActionCx) {
            self.notifications.push(cx);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            TimerToken(self.next_timer_token)
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            ClipboardToken(self.next_clipboard_token)
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            ShareSheetToken(self.next_share_sheet_token)
        }

        fn record_pending_action_payload(
            &mut self,
            _cx: fret_ui::action::ActionCx,
            _action: &fret_runtime::ActionId,
            _payload: Box<dyn Any + Send + Sync>,
        ) {
        }
    }

    impl fret_ui::action::UiFocusActionHost for TestActionHostImpl {
        fn request_focus(&mut self, target: fret_ui::GlobalElementId) {
            self.requested_focus.push(target);
        }
    }

    impl fret_ui::action::UiDragActionHost for TestActionHostImpl {
        fn begin_drag_with_kind(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
        ) {
        }

        fn begin_cross_window_drag_with_kind(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
        ) {
        }

        fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}
    }

    impl fret_ui::action::UiPointerActionHost for TestActionHostImpl {
        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn capture_pointer(&mut self) {
            self.capture_pointer_count = self.capture_pointer_count.saturating_add(1);
        }

        fn release_pointer_capture(&mut self) {
            self.release_pointer_capture_count =
                self.release_pointer_capture_count.saturating_add(1);
        }

        fn set_cursor_icon(&mut self, icon: fret_core::CursorIcon) {
            self.cursor_icons.push(icon);
        }

        fn prevent_default(&mut self, action: fret_runtime::DefaultAction) {
            self.prevented_defaults.push(action);
        }

        fn invalidate(&mut self, invalidation: Invalidation) {
            self.invalidations.push(invalidation);
        }
    }

    fn test_pointer_move(
        position: Point,
        buttons: MouseButtons,
        modifiers: Modifiers,
    ) -> fret_ui::action::PointerMoveCx {
        fret_ui::action::PointerMoveCx {
            pointer_id: PointerId::default(),
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            velocity_window: None,
            buttons,
            modifiers,
            pointer_type: PointerType::Mouse,
        }
    }

    fn test_pointer_down(
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    ) -> fret_ui::action::PointerDownCx {
        fret_ui::action::PointerDownCx {
            pointer_id: PointerId::default(),
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            button,
            modifiers,
            click_count: 1,
            pointer_type: PointerType::Mouse,
            hit_is_text_input: false,
            hit_is_pressable: false,
            hit_pressable_target: None,
        }
    }

    fn test_action_cx() -> fret_ui::action::ActionCx {
        fret_ui::action::ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(1),
        }
    }

    fn test_pointer_up(
        button: MouseButton,
        position: Point,
        modifiers: Modifiers,
    ) -> fret_ui::action::PointerUpCx {
        fret_ui::action::PointerUpCx {
            pointer_id: PointerId::default(),
            position,
            position_local: position,
            position_window: Some(position),
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            velocity_window: None,
            button,
            modifiers,
            is_click: false,
            click_count: 1,
            pointer_type: PointerType::Mouse,
            down_hit_pressable_target: None,
        }
    }

    fn test_pointer_cancel() -> fret_ui::action::PointerCancelCx {
        fret_ui::action::PointerCancelCx {
            pointer_id: PointerId::default(),
            position: None,
            position_local: None,
            position_window: None,
            tick_id: TickId(0),
            pixels_per_point: 1.0,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }
    }

    fn test_node(pos: CanvasPoint) -> Node {
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos,
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        }
    }

    fn test_marquee_geometry() -> (Graph, crate::ui::canvas::CanvasGeometry, NodeId, NodeId) {
        let mut graph = Graph::new(GraphId::from_u128(91));
        let node_a = NodeId::from_u128(9101);
        let node_b = NodeId::from_u128(9102);
        let mut node_a_value = test_node(CanvasPoint { x: 0.0, y: 0.0 });
        node_a_value.size = Some(CanvasSize {
            width: 100.0,
            height: 60.0,
        });
        let mut node_b_value = test_node(CanvasPoint { x: 140.0, y: 0.0 });
        node_b_value.size = Some(CanvasSize {
            width: 100.0,
            height: 60.0,
        });
        graph.nodes.insert(node_a, node_a_value);
        graph.nodes.insert(node_b, node_b_value);

        let draw_order = vec![node_a, node_b];
        let style = crate::ui::style::NodeGraphStyle::default();
        let mut presenter = crate::ui::presenter::DefaultNodeGraphPresenter::default();
        let geom = crate::ui::canvas::CanvasGeometry::build_with_presenter(
            &graph,
            &draw_order,
            &style,
            1.0,
            crate::io::NodeGraphNodeOrigin::default(),
            &mut presenter,
            None,
        );
        (graph, geom, node_a, node_b)
    }

    #[test]
    fn build_node_drag_transaction_uses_set_node_pos_ops() {
        let mut graph = Graph::new(GraphId::from_u128(1));
        let node_a = NodeId::from_u128(11);
        let node_b = NodeId::from_u128(22);
        let missing = NodeId::from_u128(33);
        graph
            .nodes
            .insert(node_a, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
        graph
            .nodes
            .insert(node_b, test_node(CanvasPoint { x: -5.0, y: 7.5 }));

        let tx = build_node_drag_transaction(&graph, &[node_a, missing, node_b], 12.0, -4.5);

        assert_eq!(tx.label.as_deref(), Some("Move Nodes"));
        assert_eq!(tx.ops.len(), 2);
        assert!(matches!(
            tx.ops[0],
            GraphOp::SetNodePos {
                id,
                from: CanvasPoint { x: 10.0, y: 20.0 },
                to: CanvasPoint { x: 22.0, y: 15.5 },
            } if id == node_a
        ));
        assert!(matches!(
            tx.ops[1],
            GraphOp::SetNodePos {
                id,
                from: CanvasPoint { x: -5.0, y: 7.5 },
                to: CanvasPoint { x: 7.0, y: 3.0 },
            } if id == node_b
        ));
    }

    #[test]
    fn build_node_drag_transaction_returns_empty_for_noops() {
        let mut graph = Graph::new(GraphId::from_u128(2));
        let node = NodeId::from_u128(44);
        graph
            .nodes
            .insert(node, test_node(CanvasPoint { x: 3.0, y: 9.0 }));

        let tx = build_node_drag_transaction(&graph, &[node], 0.0, 0.0);

        assert!(tx.is_empty());
        assert_eq!(tx.label, None);
    }

    #[test]
    fn build_diag_nudge_visible_node_transaction_uses_set_node_pos() {
        let mut graph = Graph::new(GraphId::from_u128(3));
        let hidden = NodeId::from_u128(55);
        let visible = NodeId::from_u128(66);
        let mut hidden_node = test_node(CanvasPoint { x: 1.0, y: 2.0 });
        hidden_node.hidden = true;
        graph.nodes.insert(hidden, hidden_node);
        graph
            .nodes
            .insert(visible, test_node(CanvasPoint { x: 10.0, y: 20.0 }));

        let tx = build_diag_nudge_visible_node_transaction(&graph);

        assert_eq!(tx.label.as_deref(), Some("Diag Nudge Visible Node"));
        assert_eq!(tx.ops.len(), 1);
        assert!(matches!(
            tx.ops[0],
            GraphOp::SetNodePos {
                id,
                from: CanvasPoint { x: 10.0, y: 20.0 },
                to: CanvasPoint { x: 11.0, y: 20.0 },
            } if id == visible
        ));
    }

    #[test]
    fn build_diag_normalize_visible_node_transaction_hides_other_nodes() {
        let mut graph = Graph::new(GraphId::from_u128(4));
        let first = NodeId::from_u128(77);
        let other = NodeId::from_u128(88);
        graph
            .nodes
            .insert(first, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
        graph
            .nodes
            .insert(other, test_node(CanvasPoint { x: -5.0, y: 7.5 }));

        let tx = build_diag_normalize_visible_node_transaction(&graph);

        assert_eq!(tx.label.as_deref(), Some("Diag Normalize Visible Node"));
        assert!(tx.ops.iter().any(|op| matches!(
            op,
            GraphOp::SetNodePos {
                id,
                from: CanvasPoint { x: 10.0, y: 20.0 },
                to: CanvasPoint { x: 0.0, y: 0.0 },
            } if *id == first
        )));
        assert!(tx.ops.iter().any(|op| matches!(
            op,
            GraphOp::SetNodeSize {
                id,
                from,
                to: Some(CanvasSize {
                    width: 220.0,
                    height: 140.0,
                }),
            } if *id == first && from.is_none()
        )));
        assert!(tx.ops.iter().any(|op| matches!(
            op,
            GraphOp::SetNodeHidden {
                id,
                from: false,
                to: true,
            } if *id == other
        )));
    }

    #[test]
    fn commit_graph_transaction_syncs_graph_and_view_models_through_controller() {
        let mut host = TestActionHostImpl::default();
        let mut graph_value = Graph::new(GraphId::from_u128(5));
        let node = NodeId::from_u128(99);
        graph_value
            .nodes
            .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
        let graph = host.models.insert(graph_value.clone());
        let view_state = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store.clone());

        let tx = host
            .models
            .read(&graph, |graph| {
                build_node_drag_transaction(graph, &[node], 5.0, -2.0)
            })
            .expect("build transaction");

        assert!(commit_graph_transaction(
            &mut host,
            &graph,
            &view_state,
            Some(&controller),
            Some(&store),
            &tx,
        ));

        let graph_pos = host
            .models
            .read(&graph, |graph| graph.nodes.get(&node).map(|node| node.pos))
            .ok()
            .flatten()
            .expect("graph node pos");
        let store_pos = host
            .models
            .read(&store, |store| {
                store.graph().nodes.get(&node).map(|node| node.pos)
            })
            .ok()
            .flatten()
            .expect("store node pos");
        let synced_zoom = host
            .models
            .read(&view_state, |state| state.zoom)
            .expect("view-state model readable");

        assert_eq!(graph_pos, CanvasPoint { x: 15.0, y: 18.0 });
        assert_eq!(store_pos, CanvasPoint { x: 15.0, y: 18.0 });
        assert_eq!(synced_zoom, 1.0);
    }

    #[test]
    fn commit_node_drag_transaction_notifies_store_callbacks_through_controller() {
        #[derive(Clone)]
        struct Recorder {
            commits: Rc<RefCell<Vec<(Option<String>, usize)>>>,
        }

        impl NodeGraphCommitCallbacks for Recorder {
            fn on_graph_commit(
                &mut self,
                committed: &crate::ops::GraphTransaction,
                changes: &NodeGraphChanges,
            ) {
                self.commits
                    .borrow_mut()
                    .push((committed.label.clone(), changes.nodes.len()));
            }
        }

        impl NodeGraphViewCallbacks for Recorder {}

        impl NodeGraphGestureCallbacks for Recorder {}

        let mut host = TestActionHostImpl::default();
        let mut graph_value = Graph::new(GraphId::from_u128(6));
        let node = NodeId::from_u128(199);
        graph_value
            .nodes
            .insert(node, test_node(CanvasPoint { x: 10.0, y: 20.0 }));
        let graph = host.models.insert(graph_value.clone());
        let view_state = host.models.insert(NodeGraphViewState::default());
        let store = host.models.insert(NodeGraphStore::new(
            graph_value,
            NodeGraphViewState::default(),
        ));
        let controller = NodeGraphController::new(store.clone());
        let commits: Rc<RefCell<Vec<(Option<String>, usize)>>> = Rc::new(RefCell::new(Vec::new()));
        let _callbacks_token = host
            .models
            .update(&store, |store| {
                install_callbacks(
                    store,
                    Recorder {
                        commits: commits.clone(),
                    },
                )
            })
            .expect("install callbacks");

        let tx = host
            .models
            .read(&graph, |graph| {
                build_node_drag_transaction(graph, &[node], 5.0, -2.0)
            })
            .expect("build transaction");

        assert!(commit_node_drag_transaction(
            &mut host,
            &graph,
            &view_state,
            Some(&controller),
            Some(&store),
            &tx,
        ));

        let callback_commits = commits.borrow();
        assert_eq!(callback_commits.len(), 1);
        assert_eq!(callback_commits[0].0.as_deref(), Some("Move Node"));
        assert_eq!(callback_commits[0].1, 1);
    }

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct DeclarativeCallbackTrace {
        commit_labels: Vec<Option<String>>,
        selection_changes: Vec<SelectionChange>,
    }

    #[derive(Clone)]
    struct DeclarativeCallbackRecorder {
        trace: Rc<RefCell<DeclarativeCallbackTrace>>,
    }

    impl NodeGraphCommitCallbacks for DeclarativeCallbackRecorder {
        fn on_graph_commit(
            &mut self,
            committed: &crate::ops::GraphTransaction,
            _changes: &NodeGraphChanges,
        ) {
            self.trace
                .borrow_mut()
                .commit_labels
                .push(committed.label.clone());
        }
    }

    impl NodeGraphViewCallbacks for DeclarativeCallbackRecorder {
        fn on_selection_change(&mut self, sel: SelectionChange) {
            self.trace.borrow_mut().selection_changes.push(sel);
        }
    }

    impl NodeGraphGestureCallbacks for DeclarativeCallbackRecorder {}

    fn install_declarative_callback_trace(
        host: &mut TestActionHostImpl,
        store: &Model<NodeGraphStore>,
    ) -> Rc<RefCell<DeclarativeCallbackTrace>> {
        let trace: Rc<RefCell<DeclarativeCallbackTrace>> =
            Rc::new(RefCell::new(DeclarativeCallbackTrace::default()));
        let _callbacks_token = host
            .models
            .update(store, |store| {
                install_callbacks(
                    store,
                    DeclarativeCallbackRecorder {
                        trace: trace.clone(),
                    },
                )
            })
            .expect("install callbacks");
        trace
    }

    struct DeclarativeControllerFixture {
        host: TestActionHostImpl,
        graph: Model<Graph>,
        view_state: Model<NodeGraphViewState>,
        store: Model<NodeGraphStore>,
        controller: NodeGraphController,
    }

    impl DeclarativeControllerFixture {
        fn new_two_nodes(
            graph_id: u128,
            node_a: NodeId,
            node_a_pos: CanvasPoint,
            node_b: NodeId,
            node_b_pos: CanvasPoint,
            initial_view: NodeGraphViewState,
        ) -> Self {
            let mut host = TestActionHostImpl::default();
            let mut graph_value = Graph::new(GraphId::from_u128(graph_id));
            graph_value.nodes.insert(node_a, test_node(node_a_pos));
            graph_value.nodes.insert(node_b, test_node(node_b_pos));
            let graph = host.models.insert(graph_value.clone());
            let view_state = host.models.insert(initial_view.clone());
            let store = host
                .models
                .insert(NodeGraphStore::new(graph_value, initial_view));
            let controller = NodeGraphController::new(store.clone());
            Self {
                host,
                graph,
                view_state,
                store,
                controller,
            }
        }

        fn install_trace(&mut self) -> Rc<RefCell<DeclarativeCallbackTrace>> {
            install_declarative_callback_trace(&mut self.host, &self.store)
        }
    }

    fn assert_single_selection_change(
        trace: &Rc<RefCell<DeclarativeCallbackTrace>>,
        expected_nodes: Vec<NodeId>,
    ) {
        let got = trace.borrow();
        assert!(got.commit_labels.is_empty());
        assert_eq!(
            got.selection_changes,
            vec![SelectionChange {
                nodes: expected_nodes,
                edges: Vec::new(),
                groups: Vec::new(),
            }]
        );
    }

    fn assert_pointer_session_finished(
        host: &TestActionHostImpl,
        action_cx: fret_ui::action::ActionCx,
    ) {
        assert_eq!(host.release_pointer_capture_count, 1);
        assert_eq!(host.invalidations, vec![Invalidation::Layout]);
        assert_eq!(host.notifications, vec![action_cx]);
        assert_eq!(host.redraw_requests, vec![action_cx.window]);
    }

    #[test]
    fn commit_pending_selection_action_host_notifies_selection_callbacks_through_controller() {
        let node_a = NodeId::from_u128(9801);
        let node_b = NodeId::from_u128(9802);
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            7,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 40.0, y: 20.0 },
            initial_view,
        );
        let trace = fixture.install_trace();
        let pending = PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        };

        assert!(commit_pending_selection_action_host(
            &mut fixture.host,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
            &pending,
        ));

        assert_single_selection_change(&trace, vec![node_b]);
    }

    #[test]
    fn commit_marquee_selection_action_host_notifies_selection_callbacks_through_controller() {
        let node_a = NodeId::from_u128(9901);
        let node_b = NodeId::from_u128(9902);
        let edge = EdgeId::new();
        let group = GroupId::new();
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            selected_edges: vec![edge],
            selected_groups: vec![group],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            8,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 60.0, y: 20.0 },
            initial_view,
        );
        let trace = fixture.install_trace();
        let marquee = MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: true,
            toggle: false,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([node_b]),
        };

        assert!(commit_marquee_selection_action_host(
            &mut fixture.host,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
            &marquee,
        ));

        assert_single_selection_change(&trace, vec![node_b]);
    }

    #[test]
    fn handle_declarative_pointer_up_action_host_left_release_finishes_pointer_session_when_handled()
     {
        let action_cx = test_action_cx();
        let node_a = NodeId::from_u128(9935);
        let node_b = NodeId::from_u128(9936);
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            120,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 40.0, y: 20.0 },
            initial_view,
        );
        let drag = fixture.host.models.insert(None::<DragState>);
        let marquee = fixture.host.models.insert(None::<MarqueeDragState>);
        let node_drag = fixture.host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(12.0), Px(0.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([node_b]),
        }));
        let pending = fixture.host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        }));
        let trace = fixture.install_trace();

        assert!(handle_declarative_pointer_up_action_host(
            &mut fixture.host,
            action_cx,
            test_pointer_up(
                MouseButton::Left,
                Point::new(Px(12.0), Px(0.0)),
                Modifiers::default(),
            ),
            MouseButton::Middle,
            &drag,
            &marquee,
            &node_drag,
            &pending,
            &fixture.graph,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
        ));
        assert_pointer_session_finished(&fixture.host, action_cx);
        assert_single_selection_change(&trace, vec![node_b]);
    }

    #[test]
    fn handle_declarative_pointer_up_action_host_ignores_non_left_non_pan_buttons() {
        let mut host = TestActionHostImpl::default();
        let action_cx = test_action_cx();
        let drag = host.models.insert(Some(DragState {
            button: MouseButton::Middle,
            last_pos: Point::new(Px(3.0), Px(4.0)),
        }));
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);
        let graph = host.models.insert(Graph::new(GraphId::from_u128(121)));
        let view_state = host.models.insert(NodeGraphViewState::default());

        assert!(!handle_declarative_pointer_up_action_host(
            &mut host,
            action_cx,
            test_pointer_up(
                MouseButton::Right,
                Point::new(Px(0.0), Px(0.0)),
                Modifiers::default(),
            ),
            MouseButton::Middle,
            &drag,
            &marquee,
            &node_drag,
            &pending,
            &graph,
            &view_state,
            None,
            None,
        ));
        assert!(
            host.models
                .read(&drag, |state| state.is_some())
                .expect("drag readable")
        );
        assert_eq!(host.release_pointer_capture_count, 0);
        assert!(host.invalidations.is_empty());
        assert!(host.notifications.is_empty());
        assert!(host.redraw_requests.is_empty());
    }

    #[test]
    fn handle_declarative_pointer_up_action_host_pan_release_clears_drag_and_finishes_session() {
        let mut host = TestActionHostImpl::default();
        let action_cx = test_action_cx();
        let drag = host.models.insert(Some(DragState {
            button: MouseButton::Middle,
            last_pos: Point::new(Px(3.0), Px(4.0)),
        }));
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);
        let graph = host.models.insert(Graph::new(GraphId::from_u128(122)));
        let view_state = host.models.insert(NodeGraphViewState::default());

        assert!(handle_declarative_pointer_up_action_host(
            &mut host,
            action_cx,
            test_pointer_up(
                MouseButton::Middle,
                Point::new(Px(0.0), Px(0.0)),
                Modifiers::default(),
            ),
            MouseButton::Middle,
            &drag,
            &marquee,
            &node_drag,
            &pending,
            &graph,
            &view_state,
            None,
            None,
        ));
        assert!(
            host.models
                .read(&drag, |state| state.is_none())
                .expect("drag readable")
        );
        assert_pointer_session_finished(&host, action_cx);
    }

    #[test]
    fn handle_declarative_pointer_cancel_action_host_finishes_session_even_without_transients() {
        let mut host = TestActionHostImpl::default();
        let action_cx = test_action_cx();
        let drag = host.models.insert(None::<DragState>);
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);

        assert!(handle_declarative_pointer_cancel_action_host(
            &mut host,
            action_cx,
            test_pointer_cancel(),
            &drag,
            &marquee,
            &node_drag,
            &pending,
        ));
        assert_pointer_session_finished(&host, action_cx);
    }

    #[test]
    fn complete_left_pointer_release_action_host_pending_selection_clears_transient_and_notifies_selection()
     {
        let node_a = NodeId::from_u128(9941);
        let node_b = NodeId::from_u128(9942);
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            10,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 40.0, y: 20.0 },
            initial_view,
        );
        let node_drag = fixture.host.models.insert(None::<NodeDragState>);
        let marquee = fixture.host.models.insert(None::<MarqueeDragState>);
        let pending = fixture.host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        }));
        let trace = fixture.install_trace();

        let outcome = complete_left_pointer_release_action_host(
            &mut fixture.host,
            &node_drag,
            &pending,
            &marquee,
            &fixture.graph,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
        );

        assert_eq!(
            outcome,
            LeftPointerReleaseOutcome::PendingSelection {
                selection_committed: true,
            }
        );
        assert!(
            fixture
                .host
                .models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        assert_single_selection_change(&trace, vec![node_b]);
    }

    #[test]
    fn complete_left_pointer_release_action_host_inactive_toggle_marquee_skips_selection_commit() {
        let node_a = NodeId::from_u128(9943);
        let node_b = NodeId::from_u128(9944);
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            11,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 40.0, y: 20.0 },
            initial_view,
        );
        let node_drag = fixture.host.models.insert(None::<NodeDragState>);
        let marquee = fixture.host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(5.0), Px(5.0)),
            active: false,
            toggle: true,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([node_b]),
        }));
        let pending = fixture.host.models.insert(None::<PendingSelectionState>);
        let trace = fixture.install_trace();

        let outcome = complete_left_pointer_release_action_host(
            &mut fixture.host,
            &node_drag,
            &pending,
            &marquee,
            &fixture.graph,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
        );

        assert_eq!(
            outcome,
            LeftPointerReleaseOutcome::Marquee {
                selection_committed: false,
            }
        );
        assert!(
            fixture
                .host
                .models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
        let got = trace.borrow();
        assert!(got.commit_labels.is_empty());
        assert!(got.selection_changes.is_empty());
    }

    #[test]
    fn complete_left_pointer_release_action_host_none_when_no_left_release_state_exists() {
        let mut host = TestActionHostImpl::default();
        let graph = host.models.insert(Graph::new(GraphId::from_u128(12)));
        let view_state = host.models.insert(NodeGraphViewState::default());
        let node_drag = host.models.insert(None::<NodeDragState>);
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);

        let outcome = complete_left_pointer_release_action_host(
            &mut host,
            &node_drag,
            &pending,
            &marquee,
            &graph,
            &view_state,
            None,
            None,
        );

        assert_eq!(outcome, LeftPointerReleaseOutcome::None);
    }

    #[test]
    fn handle_node_drag_left_pointer_release_action_host_clears_drag_and_pending_selection() {
        let node_a = NodeId::from_u128(9945);
        let node_b = NodeId::from_u128(9946);
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let mut fixture = DeclarativeControllerFixture::new_two_nodes(
            13,
            node_a,
            CanvasPoint { x: 0.0, y: 0.0 },
            node_b,
            CanvasPoint { x: 40.0, y: 20.0 },
            initial_view,
        );
        let node_drag = fixture.host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(12.0), Px(0.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([node_b]),
        }));
        let pending = fixture.host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        }));
        let trace = fixture.install_trace();

        let outcome = handle_node_drag_left_pointer_release_action_host(
            &mut fixture.host,
            &node_drag,
            &pending,
            &fixture.graph,
            &fixture.view_state,
            Some(&fixture.controller),
            Some(&fixture.store),
        );

        assert_eq!(
            outcome,
            Some(LeftPointerReleaseOutcome::NodeDrag(
                NodeDragReleaseOutcome {
                    selection_committed: true,
                    drag_committed: false,
                }
            ))
        );
        assert!(
            fixture
                .host
                .models
                .read(&node_drag, |state| state.is_none())
                .expect("node drag readable")
        );
        assert!(
            fixture
                .host
                .models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        assert_single_selection_change(&trace, vec![node_b]);
    }

    #[test]
    fn handle_pending_selection_left_pointer_release_action_host_commits_and_clears_pending_only() {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9947);
        let node_b = NodeId::from_u128(9948);
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        });
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: true,
            clear_groups: true,
        }));

        let outcome = handle_pending_selection_left_pointer_release_action_host(
            &mut host,
            &pending,
            &view_state,
            None,
            None,
        );

        assert_eq!(
            outcome,
            Some(LeftPointerReleaseOutcome::PendingSelection {
                selection_committed: true,
            })
        );
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        assert_eq!(
            host.models
                .read(&view_state, |state| state.selected_nodes.clone())
                .expect("view state readable"),
            vec![node_b]
        );
    }

    #[test]
    fn handle_marquee_left_pointer_release_action_host_clears_pending_and_marquee_without_commit() {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9949);
        let node_b = NodeId::from_u128(9950);
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        });
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(4.0), Px(4.0)),
            active: false,
            toggle: true,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([node_b]),
        }));
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        }));

        let outcome = handle_marquee_left_pointer_release_action_host(
            &mut host,
            &marquee,
            &pending,
            &view_state,
            None,
            None,
        );

        assert_eq!(
            outcome,
            Some(LeftPointerReleaseOutcome::Marquee {
                selection_committed: false,
            })
        );
        assert!(
            host.models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        assert_eq!(
            host.models
                .read(&view_state, |state| state.selected_nodes.clone())
                .expect("view state readable"),
            vec![node_a]
        );
    }

    #[test]
    fn complete_node_drag_release_action_host_selection_only_release_notifies_selection_without_drag_commit()
     {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9951);
        let node_b = NodeId::from_u128(9952);
        let mut graph_value = Graph::new(GraphId::from_u128(9));
        graph_value
            .nodes
            .insert(node_a, test_node(CanvasPoint { x: 0.0, y: 0.0 }));
        graph_value
            .nodes
            .insert(node_b, test_node(CanvasPoint { x: 40.0, y: 20.0 }));
        let graph = host.models.insert(graph_value.clone());
        let initial_view = NodeGraphViewState {
            selected_nodes: vec![node_a],
            ..Default::default()
        };
        let view_state = host.models.insert(initial_view.clone());
        let store = host
            .models
            .insert(NodeGraphStore::new(graph_value, initial_view.clone()));
        let controller = NodeGraphController::new(store.clone());
        let trace = install_declarative_callback_trace(&mut host, &store);
        let node_drag = NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(12.0), Px(0.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([node_b]),
        };
        let pending = PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        };

        let outcome = complete_node_drag_release_action_host(
            &mut host,
            &graph,
            &view_state,
            Some(&controller),
            Some(&store),
            &node_drag,
            Some(&pending),
        );

        assert!(outcome.selection_committed);
        assert!(!outcome.drag_committed);
        let got = trace.borrow();
        assert!(got.commit_labels.is_empty());
        assert_eq!(
            got.selection_changes,
            vec![SelectionChange {
                nodes: vec![node_b],
                edges: Vec::new(),
                groups: Vec::new(),
            }]
        );
    }

    #[test]
    fn escape_cancel_declarative_interactions_action_host_handles_pending_selection_only() {
        let mut host = TestActionHostImpl::default();
        let drag = host.models.insert(None::<DragState>);
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([NodeId::from_u128(9961)]),
            clear_edges: true,
            clear_groups: true,
        }));

        assert!(escape_cancel_declarative_interactions_action_host(
            &mut host, &drag, &marquee, &node_drag, &pending,
        ));
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
    }

    #[test]
    fn begin_pan_pointer_down_action_host_clears_transients_and_starts_drag() {
        let mut host = TestActionHostImpl::default();
        let drag = host.models.insert(None::<DragState>);
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(1.0), Px(2.0)),
            current_screen: Point::new(Px(3.0), Px(4.0)),
            active: false,
            toggle: false,
            base_selected_nodes: Arc::from([]),
            preview_selected_nodes: Arc::from([]),
        }));
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(5.0), Px(6.0)),
            current_screen: Point::new(Px(7.0), Px(8.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([NodeId::from_u128(9966)]),
        }));
        let down = test_pointer_down(
            MouseButton::Middle,
            Point::new(Px(10.0), Px(11.0)),
            Modifiers::default(),
        );

        assert!(begin_pan_pointer_down_action_host(
            &mut host, &drag, &marquee, &node_drag, down,
        ));
        assert!(
            host.models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
        assert!(
            host.models
                .read(&node_drag, |state| state.is_none())
                .expect("node drag readable")
        );
        host.models
            .read(&drag, |state| {
                let state = state.expect("drag armed");
                assert_eq!(state.button, MouseButton::Middle);
                assert_eq!(state.last_pos, Point::new(Px(10.0), Px(11.0)));
            })
            .expect("drag readable");
    }

    #[test]
    fn begin_left_pointer_down_action_host_hit_node_selectable_arms_pending_selection_and_drag() {
        let mut host = TestActionHostImpl::default();
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(1.0), Px(2.0)),
            current_screen: Point::new(Px(3.0), Px(4.0)),
            active: false,
            toggle: false,
            base_selected_nodes: Arc::from([]),
            preview_selected_nodes: Arc::from([]),
        }));
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);
        let hovered = host.models.insert(None::<NodeId>);
        let hit = NodeId::from_u128(9967);
        let snapshot = LeftPointerDownSnapshot {
            interaction: crate::io::NodeGraphInteractionConfig {
                elements_selectable: true,
                nodes_draggable: true,
                ..Default::default()
            },
            base_selection: vec![NodeId::from_u128(9968)],
            hit: Some(hit),
        };
        let down = test_pointer_down(
            MouseButton::Left,
            Point::new(Px(12.0), Px(13.0)),
            Modifiers::default(),
        );

        let outcome = begin_left_pointer_down_action_host(
            &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
        );

        assert_eq!(
            outcome,
            LeftPointerDownOutcome::HitNode {
                capture_pointer: true,
            }
        );
        assert!(outcome.capture_pointer());
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            Some(hit)
        );
        host.models
            .read(&pending, |state| {
                let state = state.as_ref().expect("pending armed");
                assert_eq!(state.nodes.as_ref(), &[hit]);
                assert!(!state.clear_edges);
                assert!(!state.clear_groups);
            })
            .expect("pending readable");
        host.models
            .read(&node_drag, |state| {
                let state = state.as_ref().expect("node drag armed");
                assert_eq!(state.phase, NodeDragPhase::Armed);
                assert_eq!(state.nodes_sorted.as_ref(), &[hit]);
            })
            .expect("node drag readable");
        assert!(
            host.models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
    }

    #[test]
    fn begin_left_pointer_down_action_host_empty_space_arms_marquee() {
        let mut host = TestActionHostImpl::default();
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(1.0), Px(1.0)),
            current_screen: Point::new(Px(2.0), Px(2.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([NodeId::from_u128(9969)]),
        }));
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([NodeId::from_u128(9970)]),
            clear_edges: false,
            clear_groups: false,
        }));
        let hovered = host.models.insert(Some(NodeId::from_u128(9971)));
        let snapshot = LeftPointerDownSnapshot {
            interaction: crate::io::NodeGraphInteractionConfig {
                elements_selectable: true,
                selection_on_drag: true,
                ..Default::default()
            },
            base_selection: vec![NodeId::from_u128(9972)],
            hit: None,
        };
        let down = test_pointer_down(
            MouseButton::Left,
            Point::new(Px(20.0), Px(21.0)),
            Modifiers::default(),
        );

        let outcome = begin_left_pointer_down_action_host(
            &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
        );

        assert_eq!(outcome, LeftPointerDownOutcome::Marquee);
        assert!(outcome.capture_pointer());
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            None
        );
        assert!(
            host.models
                .read(&node_drag, |state| state.is_none())
                .expect("node drag readable")
        );
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        host.models
            .read(&marquee, |state| {
                let state = state.as_ref().expect("marquee armed");
                assert_eq!(state.start_screen, Point::new(Px(20.0), Px(21.0)));
                assert_eq!(state.preview_selected_nodes.len(), 0);
            })
            .expect("marquee readable");
    }

    #[test]
    fn begin_left_pointer_down_action_host_empty_space_clear_arms_pending_clear() {
        let mut host = TestActionHostImpl::default();
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(None::<NodeDragState>);
        let pending = host.models.insert(None::<PendingSelectionState>);
        let hovered = host.models.insert(Some(NodeId::from_u128(9973)));
        let snapshot = LeftPointerDownSnapshot {
            interaction: crate::io::NodeGraphInteractionConfig {
                elements_selectable: true,
                ..Default::default()
            },
            base_selection: Vec::new(),
            hit: None,
        };
        let down = test_pointer_down(
            MouseButton::Left,
            Point::new(Px(30.0), Px(31.0)),
            Modifiers::default(),
        );

        let outcome = begin_left_pointer_down_action_host(
            &mut host, &marquee, &node_drag, &pending, &hovered, down, &snapshot,
        );

        assert_eq!(outcome, LeftPointerDownOutcome::EmptySpaceClear);
        assert!(outcome.capture_pointer());
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            None
        );
        host.models
            .read(&pending, |state| {
                let state = state.as_ref().expect("pending clear armed");
                assert!(state.nodes.is_empty());
                assert!(state.clear_edges);
                assert!(state.clear_groups);
            })
            .expect("pending readable");
    }

    #[test]
    fn handle_node_drag_pointer_move_action_host_activation_commits_pending_selection_and_requests_capture()
     {
        let mut host = TestActionHostImpl::default();
        let view_state = host.models.insert(NodeGraphViewState {
            interaction: crate::io::NodeGraphInteractionConfig {
                node_drag_threshold: 4.0,
                ..Default::default()
            },
            ..Default::default()
        });
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            phase: NodeDragPhase::Armed,
            nodes_sorted: Arc::from([NodeId::from_u128(9974)]),
        }));
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([NodeId::from_u128(9975)]),
            clear_edges: false,
            clear_groups: false,
        }));
        let hovered = host.models.insert(Some(NodeId::from_u128(9976)));
        let mv = test_pointer_move(
            Point::new(Px(10.0), Px(0.0)),
            MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            Modifiers::default(),
        );

        let outcome = handle_node_drag_pointer_move_action_host(
            &mut host,
            &node_drag,
            &pending,
            &hovered,
            &view_state,
            None,
            None,
            mv,
        );

        assert_eq!(
            outcome,
            Some(NodeDragPointerMoveOutcome {
                capture_pointer: true,
                needs_layout_redraw: true,
            })
        );
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            None
        );
        host.models
            .read(&view_state, |state| {
                assert_eq!(state.selected_nodes, vec![NodeId::from_u128(9975)]);
            })
            .expect("view readable");
        host.models
            .read(&node_drag, |state| {
                let state = state.as_ref().expect("node drag readable");
                assert!(state.is_active());
                assert_eq!(state.current_screen, Point::new(Px(10.0), Px(0.0)));
            })
            .expect("node drag readable");
    }

    #[test]
    fn handle_node_drag_pointer_move_action_host_canceled_session_clears_hover_without_redraw() {
        let mut host = TestActionHostImpl::default();
        let view_state = host.models.insert(NodeGraphViewState::default());
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            phase: NodeDragPhase::Canceled,
            nodes_sorted: Arc::from([NodeId::from_u128(9977)]),
        }));
        let pending = host.models.insert(None::<PendingSelectionState>);
        let hovered = host.models.insert(Some(NodeId::from_u128(9978)));
        let mv = test_pointer_move(
            Point::new(Px(2.0), Px(0.0)),
            MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            Modifiers::default(),
        );

        let outcome = handle_node_drag_pointer_move_action_host(
            &mut host,
            &node_drag,
            &pending,
            &hovered,
            &view_state,
            None,
            None,
            mv,
        );

        assert_eq!(
            outcome,
            Some(NodeDragPointerMoveOutcome {
                capture_pointer: false,
                needs_layout_redraw: false,
            })
        );
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            None
        );
    }

    #[test]
    fn handle_marquee_pointer_move_action_host_non_selectable_clears_session_without_touching_hover()
     {
        let mut host = TestActionHostImpl::default();
        let view_state = host.models.insert(NodeGraphViewState {
            interaction: crate::io::NodeGraphInteractionConfig {
                elements_selectable: false,
                ..Default::default()
            },
            ..Default::default()
        });
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: false,
            toggle: false,
            base_selected_nodes: Arc::from([]),
            preview_selected_nodes: Arc::from([]),
        }));
        let hovered = host.models.insert(Some(NodeId::from_u128(9979)));
        let derived_cache = host.models.insert(DerivedGeometryCacheState::default());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(200.0)),
        );
        let mv = test_pointer_move(
            Point::new(Px(10.0), Px(10.0)),
            MouseButtons::default(),
            Modifiers::default(),
        );

        let outcome = handle_marquee_pointer_move_action_host(
            &mut host,
            &marquee,
            &hovered,
            &view_state,
            &derived_cache,
            mv,
            bounds,
        );

        assert_eq!(
            outcome,
            Some(MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly)
        );
        assert!(
            host.models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            Some(NodeId::from_u128(9979))
        );
    }

    #[test]
    fn handle_marquee_pointer_move_action_host_updates_preview_and_clears_hover() {
        let mut host = TestActionHostImpl::default();
        let (graph, geom, node_a, _node_b) = test_marquee_geometry();
        let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
        let derived_cache = host.models.insert(DerivedGeometryCacheState {
            key: None,
            rebuilds: 1,
            geom: Some(Arc::new(geom)),
            index: Some(Arc::new(spatial)),
        });
        let view_state = host.models.insert(NodeGraphViewState {
            interaction: crate::io::NodeGraphInteractionConfig {
                elements_selectable: true,
                selection_mode: crate::io::NodeGraphSelectionMode::Partial,
                ..Default::default()
            },
            ..Default::default()
        });
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: false,
            toggle: false,
            base_selected_nodes: Arc::from([]),
            preview_selected_nodes: Arc::from([]),
        }));
        let hovered = host.models.insert(Some(NodeId::from_u128(9980)));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(200.0)),
        );
        let mv = test_pointer_move(
            Point::new(Px(80.0), Px(40.0)),
            MouseButtons::default(),
            Modifiers::default(),
        );

        let outcome = handle_marquee_pointer_move_action_host(
            &mut host,
            &marquee,
            &hovered,
            &view_state,
            &derived_cache,
            mv,
            bounds,
        );

        assert_eq!(outcome, Some(MarqueePointerMoveOutcome::NotifyRedraw));
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            None
        );
        host.models
            .read(&marquee, |state| {
                let state = state.as_ref().expect("marquee readable");
                assert!(state.active);
                assert_eq!(state.current_screen, Point::new(Px(80.0), Px(40.0)));
                assert_eq!(state.preview_selected_nodes.as_ref(), &[node_a]);
            })
            .expect("marquee readable");
    }

    #[test]
    fn update_hovered_node_pointer_move_action_host_sets_hit_node_from_geometry() {
        let mut host = TestActionHostImpl::default();
        let (graph, geom, _node_a, node_b) = test_marquee_geometry();
        let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
        let derived_cache = host.models.insert(DerivedGeometryCacheState {
            key: None,
            rebuilds: 1,
            geom: Some(Arc::new(geom)),
            index: Some(Arc::new(spatial)),
        });
        let view_state = host.models.insert(NodeGraphViewState::default());
        let hovered = host.models.insert(None::<NodeId>);
        let hit_scratch = host.models.insert(Vec::<NodeId>::new());
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(200.0)),
        );
        let mv = test_pointer_move(
            Point::new(Px(160.0), Px(20.0)),
            MouseButtons::default(),
            Modifiers::default(),
        );

        assert!(update_hovered_node_pointer_move_action_host(
            &mut host,
            &hovered,
            &view_state,
            &derived_cache,
            &hit_scratch,
            mv,
            bounds,
        ));
        assert_eq!(
            host.models
                .read(&hovered, |state| *state)
                .expect("hovered readable"),
            Some(node_b)
        );
    }

    #[test]
    fn declarative_diag_key_action_from_key_gates_on_diag_toggle() {
        assert_eq!(
            DeclarativeDiagKeyAction::from_key(false, fret_core::KeyCode::Digit3),
            None
        );
        assert_eq!(
            DeclarativeDiagKeyAction::from_key(true, fret_core::KeyCode::Digit3),
            Some(DeclarativeDiagKeyAction::NudgeVisibleNode)
        );
        assert_eq!(
            DeclarativeKeyboardZoomAction::from_key(fret_core::KeyCode::Digit0),
            Some(DeclarativeKeyboardZoomAction::Reset)
        );
    }

    #[test]
    fn apply_declarative_diag_view_preset_action_host_offset_partial_marquee_clears_selection() {
        let mut host = TestActionHostImpl::default();
        let view_state = host.models.insert(NodeGraphViewState {
            zoom: 2.5,
            selected_nodes: vec![NodeId::from_u128(9964)],
            selected_edges: vec![EdgeId::new()],
            selected_groups: vec![GroupId::new()],
            ..Default::default()
        });

        assert!(apply_declarative_diag_view_preset_action_host(
            &mut host,
            &view_state,
            None,
            None,
            DeclarativeDiagViewPreset::OffsetPartialMarquee,
        ));
        host.models
            .read(&view_state, |state| {
                assert_eq!(state.pan.x, 540.0);
                assert_eq!(state.pan.y, 290.0);
                assert_eq!(state.zoom, 1.0);
                assert!(state.interaction.selection_on_drag);
                assert_eq!(
                    state.interaction.selection_mode,
                    crate::io::NodeGraphSelectionMode::Partial
                );
                assert!(state.selected_nodes.is_empty());
                assert!(state.selected_edges.is_empty());
                assert!(state.selected_groups.is_empty());
            })
            .expect("view readable");
    }

    #[test]
    fn handle_declarative_diag_key_action_host_disable_portals_clears_pending_fit_and_bounds() {
        let mut host = TestActionHostImpl::default();
        let graph = host.models.insert(Graph::default());
        let view_state = host.models.insert(NodeGraphViewState::default());
        let mut portal_bounds_state = PortalBoundsStore::default();
        portal_bounds_state.pending_fit_to_portals = true;
        portal_bounds_state.nodes_canvas_bounds.insert(
            NodeId::from_u128(9965),
            Rect::new(
                Point::new(Px(1.0), Px(2.0)),
                fret_core::Size::new(Px(3.0), Px(4.0)),
            ),
        );
        let portal_bounds = host.models.insert(portal_bounds_state);
        let portal_debug = host.models.insert(PortalDebugFlags::default());
        let diag_paint_overrides_enabled = host.models.insert(false);
        let diag_paint_overrides = Arc::new(NodeGraphPaintOverridesMap::default());

        assert!(handle_declarative_diag_key_action_host(
            &mut host,
            DeclarativeDiagKeyAction::DisablePortals,
            &graph,
            &view_state,
            None,
            None,
            &portal_bounds,
            &portal_debug,
            &diag_paint_overrides,
            &diag_paint_overrides_enabled,
        ));
        assert!(
            host.models
                .read(&portal_debug, |state| state.disable_portals)
                .expect("portal debug readable")
        );
        host.models
            .read(&portal_bounds, |state| {
                assert!(!state.pending_fit_to_portals);
                assert!(state.nodes_canvas_bounds.is_empty());
            })
            .expect("portal bounds readable");
    }

    #[test]
    fn handle_declarative_keyboard_zoom_action_host_reset_normalizes_zoom() {
        let mut host = TestActionHostImpl::default();
        let view_state = host.models.insert(NodeGraphViewState {
            zoom: 2.5,
            ..Default::default()
        });

        assert!(handle_declarative_keyboard_zoom_action_host(
            &mut host,
            DeclarativeKeyboardZoomAction::Reset,
            &view_state,
            None,
            None,
            0.1,
            8.0,
        ));
        assert_eq!(
            host.models
                .read(&view_state, |state| state.zoom)
                .expect("view readable"),
            1.0
        );
    }

    #[test]
    fn handle_declarative_diag_key_action_host_toggle_paint_overrides_sets_first_edge_override() {
        let mut host = TestActionHostImpl::default();
        let edge_id = EdgeId::new();
        let mut graph_value = Graph::new(GraphId::new());
        graph_value.edges.insert(
            edge_id,
            crate::core::Edge {
                kind: crate::core::EdgeKind::Data,
                from: crate::core::PortId::new(),
                to: crate::core::PortId::new(),
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        let graph = host.models.insert(graph_value);
        let view_state = host.models.insert(NodeGraphViewState::default());
        let portal_bounds = host.models.insert(PortalBoundsStore::default());
        let portal_debug = host.models.insert(PortalDebugFlags::default());
        let diag_paint_overrides_enabled = host.models.insert(false);
        let diag_paint_overrides = Arc::new(NodeGraphPaintOverridesMap::default());

        assert!(handle_declarative_diag_key_action_host(
            &mut host,
            DeclarativeDiagKeyAction::TogglePaintOverrides,
            &graph,
            &view_state,
            None,
            None,
            &portal_bounds,
            &portal_debug,
            &diag_paint_overrides,
            &diag_paint_overrides_enabled,
        ));
        assert!(
            host.models
                .read(&diag_paint_overrides_enabled, |state| *state)
                .expect("flag readable")
        );
        assert!(diag_paint_overrides.edge_paint_override(edge_id).is_some());
    }

    #[test]
    fn escape_cancel_declarative_interactions_action_host_ignores_already_canceled_node_drag() {
        let mut host = TestActionHostImpl::default();
        let drag = host.models.insert(None::<DragState>);
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(4.0), Px(0.0)),
            phase: NodeDragPhase::Canceled,
            nodes_sorted: Arc::from([NodeId::from_u128(9962)]),
        }));
        let pending = host.models.insert(None::<PendingSelectionState>);

        assert!(!escape_cancel_declarative_interactions_action_host(
            &mut host, &drag, &marquee, &node_drag, &pending,
        ));
        assert!(
            host.models
                .read(&node_drag, |state| {
                    state.as_ref().is_some_and(NodeDragState::is_canceled)
                })
                .expect("node drag readable")
        );
    }

    #[test]
    fn pointer_cancel_declarative_interactions_action_host_clears_already_canceled_node_drag() {
        let mut host = TestActionHostImpl::default();
        let drag = host.models.insert(None::<DragState>);
        let marquee = host.models.insert(None::<MarqueeDragState>);
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(4.0), Px(0.0)),
            phase: NodeDragPhase::Canceled,
            nodes_sorted: Arc::from([NodeId::from_u128(9963)]),
        }));
        let pending = host.models.insert(None::<PendingSelectionState>);

        assert!(pointer_cancel_declarative_interactions_action_host(
            &mut host, &drag, &marquee, &node_drag, &pending,
        ));
        assert!(
            host.models
                .read(&node_drag, |state| state.is_none())
                .expect("node drag readable")
        );
    }

    #[test]
    fn pointer_cancel_declarative_interactions_action_host_clears_transients_without_callbacks() {
        let mut host = TestActionHostImpl::default();
        let drag = host.models.insert(Some(DragState {
            button: MouseButton::Left,
            last_pos: Point::new(Px(2.0), Px(3.0)),
        }));
        let marquee = host.models.insert(Some(MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(10.0), Px(10.0)),
            active: true,
            toggle: false,
            base_selected_nodes: Arc::from([]),
            preview_selected_nodes: Arc::from([]),
        }));
        let node_drag = host.models.insert(Some(NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(8.0), Px(0.0)),
            phase: NodeDragPhase::Active,
            nodes_sorted: Arc::from([NodeId::from_u128(9971)]),
        }));
        let pending = host.models.insert(Some(PendingSelectionState {
            nodes: Arc::from([NodeId::from_u128(9972)]),
            clear_edges: false,
            clear_groups: false,
        }));

        assert!(pointer_cancel_declarative_interactions_action_host(
            &mut host, &drag, &marquee, &node_drag, &pending,
        ));
        assert!(
            host.models
                .read(&drag, |state| state.is_none())
                .expect("drag readable")
        );
        assert!(
            host.models
                .read(&marquee, |state| state.is_none())
                .expect("marquee readable")
        );
        assert!(
            host.models
                .read(&node_drag, |state| state.is_none())
                .expect("node drag readable")
        );
        assert!(
            host.models
                .read(&pending, |state| state.is_none())
                .expect("pending readable")
        );
    }

    #[test]
    fn build_click_selection_preview_nodes_single_click_replaces_base_selection() {
        let node_a = NodeId::from_u128(9401);
        let node_b = NodeId::from_u128(9402);

        let preview = build_click_selection_preview_nodes(&[node_a], node_b, false);

        assert_eq!(preview.as_ref(), &[node_b]);
    }

    #[test]
    fn build_click_selection_preview_nodes_multi_click_toggles_hit_membership() {
        let node_a = NodeId::from_u128(9501);
        let node_b = NodeId::from_u128(9502);

        let added = build_click_selection_preview_nodes(&[node_a], node_b, true);
        let removed = build_click_selection_preview_nodes(&[node_a, node_b], node_b, true);

        assert_eq!(added.as_ref(), &[node_a, node_b]);
        assert_eq!(removed.as_ref(), &[node_a]);
    }

    #[test]
    fn commit_pending_selection_action_host_preserves_edges_and_groups_when_not_requested() {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9601);
        let node_b = NodeId::from_u128(9602);
        let edge = EdgeId::new();
        let group = GroupId::new();
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node_a],
            selected_edges: vec![edge],
            selected_groups: vec![group],
            ..Default::default()
        });
        let pending = PendingSelectionState {
            nodes: Arc::from([node_b]),
            clear_edges: false,
            clear_groups: false,
        };

        assert!(commit_pending_selection_action_host(
            &mut host,
            &view_state,
            None,
            None,
            &pending,
        ));

        let selection = host
            .models
            .read(&view_state, |state| {
                (
                    state.selected_nodes.clone(),
                    state.selected_edges.clone(),
                    state.selected_groups.clone(),
                )
            })
            .expect("read view state");
        assert_eq!(selection.0, vec![node_b]);
        assert_eq!(selection.1, vec![edge]);
        assert_eq!(selection.2, vec![group]);
    }

    #[test]
    fn commit_pending_selection_action_host_can_clear_all_selection_kinds() {
        let mut host = TestActionHostImpl::default();
        let node = NodeId::from_u128(9701);
        let edge = EdgeId::new();
        let group = GroupId::new();
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node],
            selected_edges: vec![edge],
            selected_groups: vec![group],
            ..Default::default()
        });
        let pending = PendingSelectionState {
            nodes: Arc::from([]),
            clear_edges: true,
            clear_groups: true,
        };

        assert!(commit_pending_selection_action_host(
            &mut host,
            &view_state,
            None,
            None,
            &pending,
        ));

        let selection = host
            .models
            .read(&view_state, |state| {
                (
                    state.selected_nodes.clone(),
                    state.selected_edges.clone(),
                    state.selected_groups.clone(),
                )
            })
            .expect("read view state");
        assert!(selection.0.is_empty());
        assert!(selection.1.is_empty());
        assert!(selection.2.is_empty());
    }

    fn test_node_drag_state(phase: NodeDragPhase, current_screen: Point) -> NodeDragState {
        NodeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen,
            phase,
            nodes_sorted: Arc::from([NodeId::from_u128(9800)]),
        }
    }

    #[test]
    fn node_drag_phase_activation_crosses_threshold() {
        let mut drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(0.0), Px(0.0)));
        let next = Point::new(Px(6.0), Px(8.0));

        assert!(pointer_crossed_threshold(drag.start_screen, next, 10.0));
        assert!(drag.activate(next));
        assert!(drag.is_active());
        assert_eq!(drag.current_screen, next);
    }

    #[test]
    fn canceled_node_drag_does_not_produce_commit_delta() {
        let view = PanZoom2D::default();
        let mut drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(12.0), Px(0.0)));

        assert!(drag.activate(Point::new(Px(12.0), Px(0.0))));
        drag.cancel();

        assert!(drag.is_canceled());
        assert_eq!(node_drag_commit_delta(view, &drag), None);
    }

    #[test]
    fn active_node_drag_with_non_zero_delta_produces_commit_delta() {
        let view = PanZoom2D {
            pan: Point::new(Px(0.0), Px(0.0)),
            zoom: 2.0,
        };
        let drag = test_node_drag_state(NodeDragPhase::Active, Point::new(Px(8.0), Px(-6.0)));

        assert_eq!(node_drag_commit_delta(view, &drag), Some((4.0, -3.0)));
    }

    #[test]
    fn armed_node_drag_release_keeps_drag_commit_local() {
        let view = PanZoom2D::default();
        let drag = test_node_drag_state(NodeDragPhase::Armed, Point::new(Px(14.0), Px(0.0)));

        assert_eq!(node_drag_commit_delta(view, &drag), None);
    }

    #[test]
    fn build_marquee_preview_selected_nodes_non_toggle_uses_current_candidates() {
        let (graph, geom, node_a, node_b) = test_marquee_geometry();
        let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
        let marquee = MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: true,
            toggle: false,
            base_selected_nodes: Arc::from([node_b]),
            preview_selected_nodes: Arc::from([]),
        };
        let rect = Rect::new(
            Point::new(Px(-10.0), Px(-10.0)),
            fret_core::Size::new(Px(120.0), Px(80.0)),
        );

        let preview = build_marquee_preview_selected_nodes(
            &marquee,
            rect,
            crate::io::NodeGraphSelectionMode::Partial,
            &geom,
            &spatial,
        );

        assert_eq!(preview.as_ref(), &[node_a]);
    }

    #[test]
    fn build_marquee_preview_selected_nodes_toggle_flips_against_base_selection() {
        let (graph, geom, node_a, node_b) = test_marquee_geometry();
        let spatial = crate::ui::canvas::CanvasSpatialDerived::build(&graph, &geom, 1.0, 0.0, 64.0);
        let marquee = MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: true,
            toggle: true,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([]),
        };
        let rect = Rect::new(
            Point::new(Px(-10.0), Px(-10.0)),
            fret_core::Size::new(Px(260.0), Px(80.0)),
        );

        let preview = build_marquee_preview_selected_nodes(
            &marquee,
            rect,
            crate::io::NodeGraphSelectionMode::Partial,
            &geom,
            &spatial,
        );

        assert_eq!(preview.as_ref(), &[node_b]);
    }

    #[test]
    fn commit_marquee_selection_action_host_clears_edges_and_groups_for_non_toggle() {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9201);
        let node_b = NodeId::from_u128(9202);
        let edge = EdgeId::new();
        let group = GroupId::new();
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node_a],
            selected_edges: vec![edge],
            selected_groups: vec![group],
            ..Default::default()
        });
        let marquee = MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: true,
            toggle: false,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([node_b]),
        };

        assert!(commit_marquee_selection_action_host(
            &mut host,
            &view_state,
            None,
            None,
            &marquee,
        ));

        let selection = host
            .models
            .read(&view_state, |state| {
                (
                    state.selected_nodes.clone(),
                    state.selected_edges.clone(),
                    state.selected_groups.clone(),
                )
            })
            .expect("read view state");
        assert_eq!(selection.0, vec![node_b]);
        assert!(selection.1.is_empty());
        assert!(selection.2.is_empty());
    }

    #[test]
    fn commit_marquee_selection_action_host_preserves_edges_and_groups_for_toggle() {
        let mut host = TestActionHostImpl::default();
        let node_a = NodeId::from_u128(9301);
        let node_b = NodeId::from_u128(9302);
        let edge = EdgeId::new();
        let group = GroupId::new();
        let view_state = host.models.insert(NodeGraphViewState {
            selected_nodes: vec![node_a],
            selected_edges: vec![edge],
            selected_groups: vec![group],
            ..Default::default()
        });
        let marquee = MarqueeDragState {
            start_screen: Point::new(Px(0.0), Px(0.0)),
            current_screen: Point::new(Px(0.0), Px(0.0)),
            active: true,
            toggle: true,
            base_selected_nodes: Arc::from([node_a]),
            preview_selected_nodes: Arc::from([node_b]),
        };

        assert!(commit_marquee_selection_action_host(
            &mut host,
            &view_state,
            None,
            None,
            &marquee,
        ));

        let selection = host
            .models
            .read(&view_state, |state| {
                (
                    state.selected_nodes.clone(),
                    state.selected_edges.clone(),
                    state.selected_groups.clone(),
                )
            })
            .expect("read view state");
        assert_eq!(selection.0, vec![node_b]);
        assert_eq!(selection.1, vec![edge]);
        assert_eq!(selection.2, vec![group]);
    }
}
