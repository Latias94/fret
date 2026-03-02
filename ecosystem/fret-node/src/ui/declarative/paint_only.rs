use std::sync::Arc;

use fret_canvas::view::{
    DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, PanZoom2D, screen_rect_to_canvas_rect,
    wheel_zoom_factor,
};
use fret_canvas::wires as canvas_wires;
use fret_core::scene::DashPatternV1;
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
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::core::Graph;
use crate::io::NodeGraphViewState;
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
}

#[derive(Debug, Clone)]
struct NodeDragState {
    start_screen: Point,
    current_screen: Point,
    active: bool,
    canceled: bool,
    nodes_sorted: Arc<[crate::core::NodeId]>,
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
struct GridPaintCacheKeyV1 {
    bounds_x_q: i32,
    bounds_y_q: i32,
    bounds_w_q: i32,
    bounds_h_q: i32,
    pan_x_q: i32,
    pan_y_q: i32,
    zoom_q: i32,
}

fn quantize_f32(value: f32, scale: f32) -> i32 {
    if !value.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return 0;
    }
    (value * scale)
        .round()
        .clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

fn grid_cache_key(bounds: Rect, view: PanZoom2D) -> CanvasKey {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0);

    // Quantize to avoid cache churn due to float noise while remaining sensitive to real changes.
    // - bounds: device-independent UI px (layout space)
    // - pan: canvas units
    // - zoom: scale factor (unitless)
    let v = GridPaintCacheKeyV1 {
        bounds_x_q: quantize_f32(bounds.origin.x.0, 8.0),
        bounds_y_q: quantize_f32(bounds.origin.y.0, 8.0),
        bounds_w_q: quantize_f32(bounds.size.width.0, 8.0),
        bounds_h_q: quantize_f32(bounds.size.height.0, 8.0),
        pan_x_q: quantize_f32(view.pan.x.0, 64.0),
        pan_y_q: quantize_f32(view.pan.y.0, 64.0),
        zoom_q: quantize_f32(zoom, 4096.0),
    };

    // Namespace the key so future paint-only variants can coexist safely.
    CanvasKey::from_hash(&("fret-node.grid.paint-only.v1", v))
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

fn build_debug_grid_ops(bounds: Rect, view: PanZoom2D) -> Arc<Vec<fret_core::SceneOp>> {
    let mut ops = Vec::<fret_core::SceneOp>::new();

    // Background fill (debug baseline).
    ops.push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: fret_core::scene::Paint::Solid(fret_core::Color::from_srgb_hex_rgb(0x0f1218))
            .into(),
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

    let max_lines = 400i32;
    let x_lines = (ix1 - ix0).clamp(0, max_lines);
    let y_lines = (iy1 - iy0).clamp(0, max_lines);

    let grid_paint: fret_core::scene::PaintBindingV1 =
        fret_core::scene::Paint::Solid(fret_core::Color::from_srgb_hex_rgb(0x202833)).into();

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
    let ops = build_debug_grid_ops(bounds, view);
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

fn node_drag_delta_canvas(view: PanZoom2D, drag: &NodeDragState) -> (f32, f32) {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let dx = (drag.current_screen.x.0 - drag.start_screen.x.0) / zoom;
    let dy = (drag.current_screen.y.0 - drag.start_screen.y.0) / zoom;
    (dx, dy)
}

fn node_drag_contains(drag: &NodeDragState, id: crate::core::NodeId) -> bool {
    drag.nodes_sorted.binary_search(&id).is_ok()
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
        let drag_active = node_drag.is_some_and(|d| d.active && !d.canceled);
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
        let drag_active = node_drag.is_some_and(|d| d.active && !d.canceled);
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

    let drag_value = cx
        .get_model_copied(&drag, Invalidation::Paint)
        .unwrap_or(None);
    let panning = drag_value.is_some();

    let marquee_value = cx
        .get_model_cloned(&marquee_drag, Invalidation::Paint)
        .unwrap_or(None);
    let marquee_active = marquee_value.as_ref().is_some_and(|m| m.active);

    let node_drag_value = cx
        .get_model_cloned(&node_drag, Invalidation::Paint)
        .unwrap_or(None);
    let node_drag_armed = node_drag_value.as_ref().is_some_and(|d| !d.canceled);
    let node_dragging = node_drag_value
        .as_ref()
        .is_some_and(|d| d.active && !d.canceled);

    let view_value = cx
        .get_model_cloned(&view_state, Invalidation::Paint)
        .unwrap_or_default();
    let view_for_paint = view_from_state(&view_value);
    let style_tokens = NodeGraphStyle::default();
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
        let key = grid_cache_key(grid_cache_value.bounds, view_for_paint);
        if grid_cache_value.key != Some(key) {
            let ops = build_debug_grid_ops(grid_cache_value.bounds, view_for_paint);
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
            &view_value.interaction,
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

                    let tuning = view_value.interaction.spatial_index;
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
    let selected_nodes_len = view_value.selected_nodes.len();
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
                .is_some_and(|d| d.active && !d.canceled);
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
            let graph_debug = graph.clone();
            let view_zoom_kb = view_state.clone();
            let view_escape = view_state.clone();
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
                        let drag_active = host
                            .models_mut()
                            .read(&drag_escape, |st| *st)
                            .ok()
                            .flatten();
                        let marquee = host
                            .models_mut()
                            .read(&marquee_escape, |st| st.clone())
                            .ok()
                            .flatten();
                        let marquee_active = marquee.is_some();
                        let node_drag_active = host
                            .models_mut()
                            .read(&node_drag_escape, |st| st.is_some())
                            .ok()
                            .unwrap_or(false);

                        if drag_active.is_none() && !marquee_active && !node_drag_active {
                            return false;
                        }

                        let _ = host.models_mut().update(&drag_escape, |st| *st = None);
                        if let Some(marquee) = marquee {
                            let base_selected = marquee.base_selected_nodes.clone();
                            let _ = host.models_mut().update(&view_escape, |state| {
                                state.selected_nodes.clear();
                                state.selected_nodes.extend(base_selected.iter().copied());
                            });
                        }
                        let _ = host.models_mut().update(&marquee_escape, |st| *st = None);
                        let _ = host.models_mut().update(&node_drag_escape, |st| {
                            if let Some(st) = st.as_mut() {
                                st.canceled = true;
                                st.active = false;
                            }
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if !(key.modifiers.ctrl || key.modifiers.meta) {
                        return false;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit3 {
                        // Diagnostics-only: a deterministic graph mutation to validate that
                        // geometry caches rebuild on graph revision changes (without relying on
                        // demo command routing).
                        let _ = host.models_mut().update(&graph_debug, |g| {
                            for node in g.nodes.values_mut() {
                                if node.hidden {
                                    continue;
                                }
                                node.pos.x += 1.0;
                                break;
                            }
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit4 {
                        // Diagnostics-only: normalize the graph/view so scripted hit tests can be
                        // deterministic without relying on demo-specific command routing.
                        //
                        // - Move one visible node to (0,0) with a stable size.
                        // - Hide all other nodes to avoid ambiguity.
                        // - Set a deterministic pan so the node center lands at window center for
                        //   the demo's default 980x720 config.
                        let _ = host.models_mut().update(&graph_debug, |g| {
                            let mut first: Option<crate::core::NodeId> = None;
                            for (id, node) in g.nodes.iter() {
                                if !node.hidden {
                                    first = Some(*id);
                                    break;
                                }
                            }
                            let Some(first) = first else {
                                return;
                            };

                            for (id, node) in g.nodes.iter_mut() {
                                if *id == first {
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
                        });

                        let _ = host.models_mut().update(&view_zoom_kb, |s| {
                            s.pan.x = 380.0;
                            s.pan.y = 290.0;
                            s.zoom = 1.0;
                            // `diag.script_v2` cannot currently synthesize pointer-modifier combos,
                            // so enable selection-on-drag to make marquee tests deterministic.
                            s.interaction.selection_on_drag = true;
                            s.selected_nodes.clear();
                            s.selected_edges.clear();
                            s.selected_groups.clear();
                        });

                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit5 {
                        // Diagnostics-only: place the normalized node slightly away from the canvas
                        // center so scripted marquee selection can start from the center (empty
                        // space) and intersect the node via a deterministic drag.
                        let _ = host.models_mut().update(&graph_debug, |g| {
                            let mut first: Option<crate::core::NodeId> = None;
                            for (id, node) in g.nodes.iter() {
                                if !node.hidden {
                                    first = Some(*id);
                                    break;
                                }
                            }
                            let Some(first) = first else {
                                return;
                            };

                            for (id, node) in g.nodes.iter_mut() {
                                if *id == first {
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
                        });

                        let _ = host.models_mut().update(&view_zoom_kb, |s| {
                            // Shift pan.x so the node no longer covers the canvas center.
                            s.pan.x = 540.0;
                            s.pan.y = 290.0;
                            s.zoom = 1.0;
                            s.interaction.selection_on_drag = true;
                            // Use partial selection so the scripted drag only needs to intersect.
                            s.interaction.selection_mode =
                                crate::io::NodeGraphSelectionMode::Partial;
                            s.selected_nodes.clear();
                            s.selected_edges.clear();
                            s.selected_groups.clear();
                        });

                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit9 {
                        // Portal bounds are frame-lagged by contract and canvas bounds are only
                        // learned after layout. Arm a pending fit request and let the paint pass
                        // apply it deterministically once the prerequisites are available.
                        let _ = host.models_mut().update(&portal_bounds_for_fit, |st| {
                            st.pending_fit_to_portals = true;
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit8 {
                        let _ = host.models_mut().update(&portal_debug_for_keys, |st| {
                            st.disable_portals = true;
                        });
                        // Ensure consumers don't keep using stale portal bounds.
                        let _ = host.models_mut().update(&portal_bounds_for_fit, |st| {
                            st.nodes_canvas_bounds.clear();
                            st.pending_fit_to_portals = false;
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit7 {
                        let _ = host.models_mut().update(&portal_debug_for_keys, |st| {
                            st.disable_portals = false;
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if diag_keys_enabled && key.key == fret_core::KeyCode::Digit6 {
                        let enable_next = host
                            .models_mut()
                            .read(&diag_paint_overrides_enabled_for_keys, |st| *st)
                            .ok()
                            .unwrap_or(false);
                        let enable_next = !enable_next;
                        let _ = host
                            .models_mut()
                            .update(&diag_paint_overrides_enabled_for_keys, |st| {
                                *st = enable_next;
                            });

                        let (edge_id, node_id) = host
                            .models_mut()
                            .read(&graph_debug, |g| {
                                let edge = g.edges.keys().next().copied();
                                let node = g
                                    .nodes
                                    .iter()
                                    .find_map(|(id, n)| (!n.hidden).then_some(*id));
                                (edge, node)
                            })
                            .ok()
                            .unwrap_or((None, None));

                        if let Some(edge_id) = edge_id {
                            if enable_next {
                                diag_paint_overrides_for_keys.set_edge_override(
                                    edge_id,
                                    Some(
                                        crate::ui::paint_overrides::EdgePaintOverrideV1 {
                                            dash: Some(DashPatternV1::new(
                                                Px(8.0),
                                                Px(4.0),
                                                Px(0.0),
                                            )),
                                            stroke_width_mul: Some(1.6),
                                            stroke_paint: Some(
                                                fret_core::scene::Paint::Solid(Color::from_srgb_hex_rgb(
                                                    0xff_3b_30,
                                                ))
                                                .into(),
                                            ),
                                        }
                                        .normalized(),
                                    ),
                                );
                            } else {
                                diag_paint_overrides_for_keys.set_edge_override(edge_id, None);
                            }
                        }

                        if let Some(node_id) = node_id {
                            if enable_next {
                                diag_paint_overrides_for_keys.set_node_override(
                                    node_id,
                                    Some(
                                        crate::ui::paint_overrides::NodePaintOverrideV1 {
                                            body_background: Some(
                                                fret_core::scene::Paint::Solid(Color::from_srgb_hex_rgb(
                                                    0x1c_2b_3a,
                                                ))
                                                .into(),
                                            ),
                                            ..Default::default()
                                        }
                                        .normalized(),
                                    ),
                                );
                            } else {
                                diag_paint_overrides_for_keys.set_node_override(node_id, None);
                            }
                        }

                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    const KB_ZOOM_STEP_MUL: f32 = 1.1;
                    let (factor, reset) = match key.key {
                        fret_core::KeyCode::Equal | fret_core::KeyCode::NumpadAdd => {
                            (KB_ZOOM_STEP_MUL, false)
                        }
                        fret_core::KeyCode::Minus | fret_core::KeyCode::NumpadSubtract => {
                            (1.0 / KB_ZOOM_STEP_MUL, false)
                        }
                        // Diagnostics-friendly fallbacks (only digits are guaranteed to parse in
                        // `diag.script_v2` key synthesis today).
                        fret_core::KeyCode::Digit1 => (KB_ZOOM_STEP_MUL, false),
                        fret_core::KeyCode::Digit2 => (1.0 / KB_ZOOM_STEP_MUL, false),
                        fret_core::KeyCode::Digit0 | fret_core::KeyCode::Numpad0 => (1.0, true),
                        _ => return false,
                    };

                    let _ = host.models_mut().update(&view_zoom_kb, |state| {
                        let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
                        let new_zoom = if reset {
                            1.0
                        } else {
                            (zoom * factor).clamp(min_zoom, max_zoom)
                        };
                        state.zoom = new_zoom;
                    });

                    host.request_redraw(action_cx.window);
                    true
                },
            );
            cx.key_on_key_down_capture_for(element, on_key_down_capture);

            let view_pan_down = view_state.clone();
            let drag_start = drag.clone();
            let marquee_start = marquee_drag.clone();
            let node_drag_start = node_drag.clone();
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
                    let _ = host.models_mut().update(&grid_cache_bounds, |st| {
                        if st.bounds != bounds {
                            st.bounds = bounds;
                        }
                    });

                    if down.button == pan_button {
                        host.capture_pointer();
                        let _ = host.models_mut().update(&marquee_start, |st| *st = None);
                        let _ = host.models_mut().update(&node_drag_start, |st| *st = None);
                        let _ = host.models_mut().update(&drag_start, |st| {
                            *st = Some(DragState {
                                button: down.button,
                                last_pos: down.position,
                            });
                        });
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    if down.button == MouseButton::Left {
                        let (interaction, base_selection, node_click_distance_screen_px, view) =
                            host.models_mut()
                                .read(&view_pan_down, |s| {
                                    (
                                        s.interaction.clone(),
                                        s.selected_nodes.clone(),
                                        s.interaction.node_click_distance,
                                        view_from_state(s),
                                    )
                                })
                                .ok()
                                .unwrap_or((
                                    Default::default(),
                                    Vec::new(),
                                    6.0,
                                    PanZoom2D::default(),
                                ));

                        let (geom, index) = host
                            .models_mut()
                            .read(&derived_cache_for_down, |st| {
                                (st.geom.clone(), st.index.clone())
                            })
                            .ok()
                            .unwrap_or((None, None));

                        let hit = if let (Some(geom), Some(index)) =
                            (geom.as_deref(), index.as_deref())
                        {
                            host.models_mut()
                                .update(&hit_scratch_for_down, |scratch| {
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

                        let multi = interaction.multi_selection_key.is_pressed(down.modifiers);
                        let selection_box_armed = interaction.selection_on_drag
                            || interaction.selection_key.is_pressed(down.modifiers);

                        if let Some(hit) = hit {
                            let _ = host.models_mut().update(&marquee_start, |st| *st = None);
                            let _ = host.models_mut().update(&node_drag_start, |st| *st = None);
                            let _ = host
                                .models_mut()
                                .update(&hovered_for_down, |h| *h = Some(hit));
                            if interaction.elements_selectable {
                                let _ = host.models_mut().update(&view_pan_down, |state| {
                                    let already_selected =
                                        state.selected_nodes.iter().any(|id| *id == hit);
                                    if multi {
                                        if let Some(ix) =
                                            state.selected_nodes.iter().position(|id| *id == hit)
                                        {
                                            state.selected_nodes.remove(ix);
                                        } else {
                                            state.selected_nodes.push(hit);
                                        }
                                    } else if !already_selected {
                                        state.selected_nodes.clear();
                                        state.selected_nodes.push(hit);
                                    }
                                });
                            }

                            if interaction.nodes_draggable
                                && interaction.elements_selectable
                                && !multi
                            {
                                let mut nodes = if base_selection.iter().any(|id| *id == hit) {
                                    base_selection.clone()
                                } else {
                                    vec![hit]
                                };
                                nodes.sort();
                                nodes.dedup();
                                let _ = host.models_mut().update(&node_drag_start, |st| {
                                    *st = Some(NodeDragState {
                                        start_screen: down.position,
                                        current_screen: down.position,
                                        active: false,
                                        canceled: false,
                                        nodes_sorted: Arc::from(nodes.into_boxed_slice()),
                                    });
                                });
                            }

                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        let _ = host.models_mut().update(&hovered_for_down, |h| *h = None);
                        let _ = host.models_mut().update(&node_drag_start, |st| *st = None);

                        if selection_box_armed && interaction.elements_selectable {
                            host.capture_pointer();
                            let base_selected_nodes: Arc<[crate::core::NodeId]> = if multi {
                                Arc::from(base_selection.into_boxed_slice())
                            } else {
                                Arc::from([])
                            };

                            let _ = host.models_mut().update(&marquee_start, |st| {
                                *st = Some(MarqueeDragState {
                                    start_screen: down.position,
                                    current_screen: down.position,
                                    active: false,
                                    toggle: multi,
                                    base_selected_nodes,
                                });
                            });

                            if !multi {
                                let _ = host.models_mut().update(&view_pan_down, |state| {
                                    state.selected_nodes.clear();
                                    state.selected_edges.clear();
                                    state.selected_groups.clear();
                                });
                            }

                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        // Clicking empty space clears selection (unless multi-selection modifier is held).
                        if interaction.elements_selectable && !multi {
                            let _ = host.models_mut().update(&view_pan_down, |state| {
                                state.selected_nodes.clear();
                                state.selected_edges.clear();
                                state.selected_groups.clear();
                            });
                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    false
                },
            );

            let view_pan = view_state.clone();
            let drag_move = drag.clone();
            let marquee_move = marquee_drag.clone();
            let node_drag_move = node_drag.clone();
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
                        let node_drag = host
                            .models_mut()
                            .read(&node_drag_move, |st| st.clone())
                            .ok()
                            .flatten();
                        if let Some(node_drag) = node_drag {
                            if !mouse_buttons_contains(mv.buttons, MouseButton::Left) {
                                return false;
                            }

                            if node_drag.canceled {
                                let _ = host.models_mut().update(&hovered_for_hover, |h| *h = None);
                                return false;
                            }

                            let interaction = host
                                .models_mut()
                                .read(&view_pan, |s| s.interaction.clone())
                                .ok()
                                .unwrap_or_default();
                            let threshold = interaction.node_drag_threshold.max(0.0);
                            let dx = mv.position.x.0 - node_drag.start_screen.x.0;
                            let dy = mv.position.y.0 - node_drag.start_screen.y.0;
                            let dist2 = dx * dx + dy * dy;
                            let threshold2 = threshold * threshold;
                            let should_activate = dist2 >= threshold2;
                            if should_activate && !node_drag.active {
                                host.capture_pointer();
                            }

                            let mut needs_redraw = false;
                            let _ = host.models_mut().update(&node_drag_move, |st| {
                                if let Some(st) = st.as_mut() {
                                    if should_activate && !st.active {
                                        st.active = true;
                                        st.current_screen = mv.position;
                                        needs_redraw = true;
                                    }
                                    if st.active {
                                        if st.current_screen != mv.position {
                                            st.current_screen = mv.position;
                                            needs_redraw = true;
                                        }
                                    }
                                }
                            });

                            let _ = host.models_mut().update(&hovered_for_hover, |h| *h = None);
                            if needs_redraw {
                                host.request_redraw(action_cx.window);
                            }
                            return needs_redraw;
                        }

                        let marquee = host
                            .models_mut()
                            .read(&marquee_move, |st| st.clone())
                            .ok()
                            .flatten();
                        if let Some(marquee) = marquee {
                            let (interaction, view) = host
                                .models_mut()
                                .read(&view_pan, |s| (s.interaction.clone(), view_from_state(s)))
                                .ok()
                                .unwrap_or((Default::default(), PanZoom2D::default()));

                            if !interaction.elements_selectable {
                                let _ = host.models_mut().update(&marquee_move, |st| *st = None);
                                host.release_pointer_capture();
                                host.request_redraw(action_cx.window);
                                return true;
                            }

                            let threshold = interaction.node_click_distance.max(0.0);
                            let dx = mv.position.x.0 - marquee.start_screen.x.0;
                            let dy = mv.position.y.0 - marquee.start_screen.y.0;
                            let dist2 = dx * dx + dy * dy;
                            let threshold2 = threshold * threshold;
                            let should_activate = dist2 >= threshold2;
                            let active_now = marquee.active || should_activate;

                            let _ = host.models_mut().update(&marquee_move, |st| {
                                if let Some(st) = st.as_mut() {
                                    if should_activate {
                                        st.active = true;
                                    }
                                    if st.active {
                                        st.current_screen = mv.position;
                                    }
                                }
                            });

                            if active_now {
                                let (geom, index) = host
                                    .models_mut()
                                    .read(&derived_cache_for_hover, |st| {
                                        (st.geom.clone(), st.index.clone())
                                    })
                                    .ok()
                                    .unwrap_or((None, None));

                                if let (Some(geom), Some(index)) =
                                    (geom.as_deref(), index.as_deref())
                                {
                                    let start_canvas =
                                        view.screen_to_canvas(bounds, marquee.start_screen);
                                    let cur_canvas = view.screen_to_canvas(bounds, mv.position);
                                    let rect_canvas = rect_from_points(start_canvas, cur_canvas);
                                    let selection_mode = interaction.selection_mode;

                                    if marquee.toggle {
                                        let mut candidates = Vec::<crate::core::NodeId>::new();
                                        index.query_nodes_in_rect(rect_canvas, &mut candidates);
                                        candidates.retain(|id| {
                                            let Some(node) = geom.nodes.get(id) else {
                                                return false;
                                            };
                                            match selection_mode {
                                                crate::io::NodeGraphSelectionMode::Full => {
                                                    rect_contains_rect(rect_canvas, node.rect)
                                                }
                                                crate::io::NodeGraphSelectionMode::Partial => {
                                                    rects_intersect(rect_canvas, node.rect)
                                                }
                                            }
                                        });

                                        let base_selected = marquee.base_selected_nodes.clone();
                                        let _ = host.models_mut().update(&view_pan, |state| {
                                            state.selected_nodes.clear();
                                            state
                                                .selected_nodes
                                                .extend(base_selected.iter().copied());
                                            for id in candidates.iter().copied() {
                                                if let Some(ix) = state
                                                    .selected_nodes
                                                    .iter()
                                                    .position(|v| *v == id)
                                                {
                                                    state.selected_nodes.remove(ix);
                                                } else {
                                                    state.selected_nodes.push(id);
                                                }
                                            }
                                        });
                                    } else {
                                        let _ = host.models_mut().update(&view_pan, |state| {
                                            state.selected_nodes.clear();
                                            index.query_nodes_in_rect(
                                                rect_canvas,
                                                &mut state.selected_nodes,
                                            );
                                            state.selected_nodes.retain(|id| {
                                                let Some(node) = geom.nodes.get(id) else {
                                                    return false;
                                                };
                                                match selection_mode {
                                                    crate::io::NodeGraphSelectionMode::Full => {
                                                        rect_contains_rect(rect_canvas, node.rect)
                                                    }
                                                    crate::io::NodeGraphSelectionMode::Partial => {
                                                        rects_intersect(rect_canvas, node.rect)
                                                    }
                                                }
                                            });
                                        });
                                    }
                                }
                            }

                            let _ = host.models_mut().update(&hovered_for_hover, |h| *h = None);
                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        let node_click_distance_screen_px = host
                            .models_mut()
                            .read(&view_pan, |s| s.interaction.node_click_distance)
                            .ok()
                            .unwrap_or(6.0);
                        let view = host
                            .models_mut()
                            .read(&view_pan, |s| view_from_state(s))
                            .ok()
                            .unwrap_or_default();

                        let (geom, index) = host
                            .models_mut()
                            .read(&derived_cache_for_hover, |st| {
                                (st.geom.clone(), st.index.clone())
                            })
                            .ok()
                            .unwrap_or((None, None));

                        let hit = if let (Some(geom), Some(index)) =
                            (geom.as_deref(), index.as_deref())
                        {
                            host.models_mut()
                                .update(&hit_scratch_for_hover, |scratch| {
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

                        let changed = host
                            .models_mut()
                            .update(&hovered_for_hover, |h| {
                                if *h == hit {
                                    false
                                } else {
                                    *h = hit;
                                    true
                                }
                            })
                            .ok()
                            .unwrap_or(false);

                        if changed {
                            host.request_redraw(action_cx.window);
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

                    let _ = host.models_mut().update(&view_pan, |state| {
                        apply_pan_by_screen_delta(state, dx, dy);
                    });

                    drag.last_pos = mv.position;
                    let _ = host.models_mut().update(&drag_move, |st| {
                        if let Some(st) = st.as_mut() {
                            *st = drag;
                        }
                    });

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let drag_end = drag.clone();
            let marquee_end = marquee_drag.clone();
            let node_drag_end = node_drag.clone();
            let graph_commit = graph.clone();
            let view_commit = view_state.clone();
            let on_pointer_up: OnPointerUp = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      up: fret_ui::action::PointerUpCx| {
                    if up.button != pan_button {
                        if up.button != MouseButton::Left {
                            return false;
                        }

                        let node_drag = host
                            .models_mut()
                            .read(&node_drag_end, |st| st.clone())
                            .ok()
                            .flatten();
                        if let Some(node_drag) = node_drag {
                            let view = host
                                .models_mut()
                                .read(&view_commit, |s| view_from_state(s))
                                .ok()
                                .unwrap_or_default();
                            let (dx, dy) = node_drag_delta_canvas(view, &node_drag);
                            let commit = node_drag.active
                                && !node_drag.canceled
                                && dx.is_finite()
                                && dy.is_finite();

                            if commit && (dx.abs() > 1.0e-9 || dy.abs() > 1.0e-9) {
                                let nodes = node_drag.nodes_sorted.clone();
                                let _ = host.models_mut().update(&graph_commit, |g| {
                                    for id in nodes.iter().copied() {
                                        let Some(node) = g.nodes.get_mut(&id) else {
                                            continue;
                                        };
                                        node.pos.x += dx;
                                        node.pos.y += dy;
                                    }
                                });
                            }

                            host.release_pointer_capture();
                            let _ = host.models_mut().update(&node_drag_end, |st| *st = None);
                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        let active = host
                            .models_mut()
                            .read(&marquee_end, |st| st.is_some())
                            .ok()
                            .unwrap_or(false);
                        if !active {
                            return false;
                        }

                        host.release_pointer_capture();
                        let _ = host.models_mut().update(&marquee_end, |st| *st = None);
                        host.request_redraw(action_cx.window);
                        return true;
                    }

                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&drag_end, |st| *st = None);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let drag_cancel = drag.clone();
            let marquee_cancel = marquee_drag.clone();
            let node_drag_cancel = node_drag.clone();
            let view_cancel = view_state.clone();
            let on_pointer_cancel: OnPointerCancel = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      _cancel: fret_ui::action::PointerCancelCx| {
                    let marquee = host
                        .models_mut()
                        .read(&marquee_cancel, |st| st.clone())
                        .ok()
                        .flatten();
                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&drag_cancel, |st| *st = None);
                    if let Some(marquee) = marquee {
                        let base_selected = marquee.base_selected_nodes.clone();
                        let _ = host.models_mut().update(&view_cancel, |state| {
                            state.selected_nodes.clear();
                            state.selected_nodes.extend(base_selected.iter().copied());
                        });
                    }
                    let _ = host.models_mut().update(&marquee_cancel, |st| *st = None);
                    let _ = host.models_mut().update(&node_drag_cancel, |st| *st = None);
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let view_zoom = view_state.clone();
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
                    let _ = host.models_mut().update(&view_zoom, |state| {
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
                    });

                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let view_pinch = view_state.clone();
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
                    let _ = host.models_mut().update(&view_pinch, |state| {
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
                    });

                    host.request_redraw(action_cx.window);
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
                let selected_nodes = view_value.selected_nodes.clone();
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
                let canvas = cx.canvas(canvas, move |p| {
                    paint_debug_grid_cached(p, view_for_paint, grid_ops.clone());
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
                            .is_some_and(|d| d.active && !d.canceled);
                        let (ddx, ddy) = node_drag_value
                            .as_ref()
                            .filter(|_| drag_active)
                            .map(|d| node_drag_delta_canvas(view, d))
                            .unwrap_or((0.0, 0.0));

                        let hovered = hovered_node_value;
                        let selected = selected_nodes.clone();
                        let dragged_nodes = node_drag_value
                            .as_ref()
                            .filter(|d| d.active && !d.canceled)
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
                                                    0.30
                                                } else if info.hovered {
                                                    0.22
                                                } else {
                                                    0.16
                                                },
                                                ..style_tokens.paint.node_background
                                            });
                                            p.border = fret_core::Edges::all(Px(1.0));
                                            p.border_color = Some(Color {
                                                a: 0.35,
                                                ..style_tokens.paint.node_border
                                            });

                                            let header = cx
                                                .text(info.label.clone())
                                                .attach_semantics(
                                                SemanticsDecoration::default().test_id(
                                                    Arc::<str>::from(format!(
                                                        "node_graph.portal.node.{ordinal}.header"
                                                    )),
                                                ),
                                            );
                                            let ports = cx
                                                .text(Arc::<str>::from(format!(
                                                    "in:{} out:{}",
                                                    info.ports_in, info.ports_out
                                                )))
                                                .attach_semantics(
                                                    SemanticsDecoration::default().test_id(Arc::<
                                                        str,
                                                    >::from(
                                                        format!(
                                                            "node_graph.portal.node.{ordinal}.ports"
                                                        ),
                                                    )),
                                                );

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
                            .is_some_and(|d| d.active && !d.canceled);
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
                if !panning && !marquee_active && !node_dragging {
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

                                            cx.container(p, move |cx| {
                                                let mut col = ColumnProps::default();
                                                col.layout.size.width = Length::Fill;
                                                col.layout.size.height = Length::Fill;
                                                col.gap = SpacingLength::Px(Px(2.0));
                                                vec![cx.column(col, move |cx| {
                                                    vec![
                                                        cx.text(hovered_label.clone()),
                                                        cx.text(Arc::<str>::from(format!(
                                                            "in:{} out:{}",
                                                            ports_in, ports_out
                                                        ))),
                                                    ]
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
                        let applied = cx
                            .app
                            .models_mut()
                            .update(&view_state, |state| {
                                apply_fit_view_to_canvas_rect(
                                    state, bounds, target, 24.0, min_zoom, max_zoom,
                                )
                            })
                            .ok()
                            .unwrap_or(false);

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
