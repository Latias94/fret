use crate::UiHost;
use crate::elements::{ElementContext, GlobalElementId};
use crate::overlay_placement::{Align, AnchoredPanelLayout, AnchoredPanelOptions, Side};
use fret_core::{
    AttributedText, CaretAffinity, Color, Corners, Edges, EffectChain, EffectMode, EffectQuality,
    ImageId, NodeId, Px, Rect, RenderTargetId, SemanticsRole, Size, SvgFit, TextOverflow,
    TextStyle, TextWrap, UvRect, ViewportFit,
};
use fret_runtime::{CommandId, Model};
use std::sync::Arc;

use crate::{ResizablePanelGroupStyle, SvgSource, TextAreaStyle, TextInputStyle};

/// Declarative element tree node (ephemeral per frame), keyed by a stable `GlobalElementId`.
///
/// This is the authoring-layer representation described by ADR 0028 / ADR 0039.
#[derive(Debug, Clone)]
pub struct AnyElement {
    pub id: GlobalElementId,
    pub kind: ElementKind,
    pub children: Vec<AnyElement>,
}

impl AnyElement {
    pub fn new(id: GlobalElementId, kind: ElementKind, children: Vec<AnyElement>) -> Self {
        Self { id, kind, children }
    }
}

#[derive(Debug, Clone)]
pub enum ElementKind {
    Container(ContainerProps),
    Semantics(SemanticsProps),
    /// A flex container that also contributes a semantics node with a fixed role.
    ///
    /// This is used by higher-level libraries (e.g. Radix/shadcn ports) to model structural
    /// grouping (`role="group"`) without introducing an extra semantics wrapper layer that would
    /// otherwise be separated from layout.
    SemanticFlex(SemanticFlexProps),
    FocusScope(FocusScopeProps),
    /// A transparent wrapper that gates subtree presence and interactivity.
    ///
    /// This is a mechanism-oriented primitive intended to support Radix-style authoring outcomes
    /// like `forceMount` while still being able to make a subtree non-interactive (click/keyboard)
    /// or fully absent from layout/paint, without deleting the subtree (so per-element state can be
    /// preserved).
    InteractivityGate(InteractivityGateProps),
    Opacity(OpacityProps),
    /// A scoped post-processing effect group wrapper (ADR 0119).
    EffectLayer(EffectLayerProps),
    /// Experimental view-level cache boundary wrapper.
    ///
    /// When enabled by the runtime, this marks a subtree as a cache root for range-replay and
    /// invalidation containment experiments (see `docs/workstreams/gpui-parity-refactor.md`).
    ViewCache(ViewCacheProps),
    VisualTransform(VisualTransformProps),
    RenderTransform(RenderTransformProps),
    FractionalRenderTransform(FractionalRenderTransformProps),
    Anchored(AnchoredProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    /// A focusable, text-input-capable event region primitive.
    ///
    /// Unlike `TextInput` / `TextArea`, this does not own an internal text model. It exists as a
    /// mechanism-only building block for ecosystem text surfaces (e.g. code editors) that need
    /// to receive `Event::TextInput` / `Event::Ime` / clipboard events while owning their own
    /// buffer and rendering pipeline.
    TextInputRegion(TextInputRegionProps),
    /// An internal drag event listener region primitive.
    ///
    /// This is a mechanism-only building block: it does not own policy for any particular drag
    /// kind, and is intended to be used by higher-level layers (workspace, docking, etc.).
    InternalDragRegion(InternalDragRegionProps),
    RovingFlex(RovingFlexProps),
    Stack(StackProps),
    Column(ColumnProps),
    Row(RowProps),
    Spacer(SpacerProps),
    Text(TextProps),
    StyledText(StyledTextProps),
    SelectableText(SelectableTextProps),
    TextInput(TextInputProps),
    TextArea(TextAreaProps),
    ResizablePanelGroup(ResizablePanelGroupProps),
    VirtualList(VirtualListProps),
    Flex(FlexProps),
    Grid(GridProps),
    Image(ImageProps),
    /// A declarative, leaf canvas element for custom scene emission (ADR 0156).
    Canvas(CanvasProps),
    /// Composites an app-owned render target (Tier A; ADR 0007 / ADR 0038 / ADR 0125).
    ViewportSurface(ViewportSurfaceProps),
    SvgIcon(SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    /// An event-only wheel listener that updates an imperative scroll handle.
    ///
    /// Unlike `Scroll`, this element does not translate its children; it only mutates the provided
    /// `ScrollHandle` and invalidates an optional target.
    WheelRegion(WheelRegionProps),
    Scroll(ScrollProps),
    Scrollbar(ScrollbarProps),
}

#[derive(Debug, Clone, Copy)]
pub struct SemanticFlexProps {
    pub role: SemanticsRole,
    pub flex: FlexProps,
}

/// Per-element pointer state for `PointerRegion`.
#[derive(Debug, Default, Clone)]
pub struct PointerRegionState {
    pub last_down: Option<crate::action::PointerDownCx>,
}

/// A pointer event listener region primitive.
///
/// This is a mechanism-only building block: it does not imply click/activation semantics.
#[derive(Debug, Clone, Copy)]
pub struct PointerRegionProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

/// A focusable event region that participates in text input / IME routing.
#[derive(Debug, Clone)]
pub struct TextInputRegionProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub text_boundary_mode_override: Option<fret_runtime::TextBoundaryMode>,
    /// Optional accessibility label for this text input region.
    pub a11y_label: Option<Arc<str>>,
    /// Optional accessibility value text for this text input region.
    ///
    /// When present, selection and composition ranges are interpreted as UTF-8 byte offsets within
    /// this value (ADR 0071).
    pub a11y_value: Option<Arc<str>>,
    /// Optional selection range (anchor, focus) in UTF-8 byte offsets within `a11y_value`.
    pub a11y_text_selection: Option<(u32, u32)>,
    /// Optional IME composition range (start, end) in UTF-8 byte offsets within `a11y_value`.
    pub a11y_text_composition: Option<(u32, u32)>,
}

/// An internal drag event listener region primitive.
///
/// This is a mechanism-only building block for cross-window and internal drag flows.
#[derive(Debug, Clone, Copy)]
pub struct InternalDragRegionProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

impl Default for InternalDragRegionProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
        }
    }
}

impl Default for PointerRegionProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
        }
    }
}

impl Default for TextInputRegionProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            text_boundary_mode_override: None,
            a11y_label: None,
            a11y_value: None,
            a11y_text_selection: None,
            a11y_text_composition: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutStyle {
    pub size: SizeStyle,
    pub flex: FlexItemStyle,
    pub overflow: Overflow,
    pub margin: MarginEdges,
    pub position: PositionStyle,
    pub inset: InsetStyle,
    pub aspect_ratio: Option<f32>,
    pub grid: GridItemStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginEdge {
    Px(Px),
    Auto,
}

impl Default for MarginEdge {
    fn default() -> Self {
        Self::Px(Px(0.0))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarginEdges {
    pub top: MarginEdge,
    pub right: MarginEdge,
    pub bottom: MarginEdge,
    pub left: MarginEdge,
}

impl MarginEdges {
    pub fn all(edge: MarginEdge) -> Self {
        Self {
            top: edge,
            right: edge,
            bottom: edge,
            left: edge,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PositionStyle {
    /// Default flow position; inset offsets are ignored.
    #[default]
    Static,
    /// Inset offsets tweak the final position without affecting siblings.
    Relative,
    /// Removed from flow and positioned via inset offsets.
    Absolute,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InsetStyle {
    pub top: Option<Px>,
    pub right: Option<Px>,
    pub bottom: Option<Px>,
    pub left: Option<Px>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GridItemStyle {
    pub column: GridLine,
    pub row: GridLine,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GridLine {
    pub start: Option<i16>,
    pub span: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Option<Px>,
    pub min_height: Option<Px>,
    pub max_width: Option<Px>,
    pub max_height: Option<Px>,
}

impl Default for SizeStyle {
    fn default() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexItemStyle {
    pub grow: f32,
    pub shrink: f32,
    pub basis: Length,
    pub align_self: Option<CrossAlign>,
}

impl Default for FlexItemStyle {
    fn default() -> Self {
        Self {
            grow: 0.0,
            // Tailwind/DOM default is `flex-shrink: 1`. Recipes should opt out via
            // `LayoutRefinement::flex_shrink_0()` when needed.
            shrink: 1.0,
            basis: Length::Auto,
            align_self: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Length {
    #[default]
    Auto,
    Px(Px),
    Fill,
}

/// A low-opinionated container primitive for declarative authoring.
///
/// This is intentionally small and composable: it provides padding and an optional quad background
/// (including border and corner radii) so component-layer recipes can build shadcn-like widgets
/// via composition.
#[derive(Debug, Clone, Copy)]
pub struct ContainerProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub background: Option<Color>,
    pub shadow: Option<ShadowStyle>,
    pub border: Edges,
    pub border_color: Option<Color>,
    /// Optional focus-visible ring decoration.
    pub focus_ring: Option<RingStyle>,
    /// Optional border-color override applied when focus-visible is active.
    ///
    /// This is primarily used for shadcn-style `focus-visible:border-ring` outcomes without
    /// requiring a dedicated "border state" API at the layout layer.
    pub focus_border_color: Option<Color>,
    /// When true, focus state is derived from any focused descendant (focus-within).
    pub focus_within: bool,
    pub corner_radii: Corners,
    /// When true, snap paint bounds to device pixels (policy-only).
    pub snap_to_device_pixels: bool,
}

impl Default for ContainerProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            focus_ring: None,
            focus_border_color: None,
            focus_within: false,
            corner_radii: Corners::all(Px(0.0)),
            snap_to_device_pixels: false,
        }
    }
}

/// A transparent semantics wrapper for structuring the accessibility tree.
///
/// This is intentionally input-transparent (hit-test passes through) and paint-transparent: it
/// only contributes layout and semantics.
#[derive(Debug, Clone)]
pub struct SemanticsProps {
    pub layout: LayoutStyle,
    pub role: SemanticsRole,
    pub label: Option<Arc<str>>,
    /// Debug/test-only identifier for deterministic automation.
    ///
    /// This MUST NOT be mapped into platform accessibility name/label fields by default.
    pub test_id: Option<Arc<str>>,
    pub value: Option<Arc<str>>,
    /// Whether this semantics wrapper participates in focus traversal.
    ///
    /// Note: this is intentionally separate from pointer hit-testing. `Semantics` remains
    /// input-transparent; use `Pressable` when you need pointer-driven focus.
    pub focusable: bool,
    pub disabled: bool,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<bool>,
    pub active_descendant: Option<NodeId>,
    /// Declarative-only: element ID of a node which labels this node.
    ///
    /// This is an authoring convenience for relationships like `aria-labelledby` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub labelled_by_element: Option<u64>,
    /// Declarative-only: element ID of a node which describes this node.
    ///
    /// This is an authoring convenience for relationships like `aria-describedby` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub described_by_element: Option<u64>,
    /// Declarative-only: element ID of a node which this node controls.
    ///
    /// This is an authoring convenience for relationships like `aria-controls` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub controls_element: Option<u64>,
}

impl Default for SemanticsProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            role: SemanticsRole::Generic,
            label: None,
            test_id: None,
            value: None,
            focusable: false,
            disabled: false,
            selected: false,
            expanded: None,
            checked: None,
            active_descendant: None,
            labelled_by_element: None,
            described_by_element: None,
            controls_element: None,
        }
    }
}

/// A transparent focus-scope wrapper that can trap focus traversal within its subtree.
///
/// This is a small, mechanism-oriented primitive intended to support component-owned focus scopes
/// (ADR 0068). It does not imply modal barriers or pointer blocking; it only affects `focus.next`
/// / `focus.previous` command routing when focus is inside the subtree.
#[derive(Debug, Default, Clone, Copy)]
pub struct FocusScopeProps {
    pub layout: LayoutStyle,
    pub trap_focus: bool,
}

/// Gate subtree presence (layout/paint) and interactivity (hit-testing + focus traversal).
///
/// When `present == false`, the subtree remains mounted but is treated like `display: none`:
/// it does not participate in layout, paint, hit-testing, or focus traversal.
///
/// When `present == true` and `interactive == false`, the subtree is still laid out/painted but is
/// inert for pointer and focus traversal (useful for close animations).
#[derive(Debug, Clone, Copy)]
pub struct InteractivityGateProps {
    pub layout: LayoutStyle,
    pub present: bool,
    pub interactive: bool,
}

impl Default for InteractivityGateProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            present: true,
            interactive: true,
        }
    }
}

/// A paint-only opacity group wrapper (ADR 0019).
///
/// This is intentionally layout-only + paint-only: it does not imply semantics beyond its
/// children, and it is input-transparent (hit-test passes through).
#[derive(Debug, Clone, Copy)]
pub struct OpacityProps {
    pub layout: LayoutStyle,
    pub opacity: f32,
}

impl Default for OpacityProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            opacity: 1.0,
        }
    }
}

/// Scoped post-processing effect wrapper for declarative element subtrees (ADR 0119).
///
/// This emits a `SceneOp::PushEffect/PopEffect` pair around the subtree during painting. The
/// effect's computation bounds are the wrapper's final layout bounds.
#[derive(Debug, Clone, Copy)]
pub struct EffectLayerProps {
    pub layout: LayoutStyle,
    pub mode: EffectMode,
    pub chain: EffectChain,
    pub quality: EffectQuality,
}

impl Default for EffectLayerProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            mode: EffectMode::FilterContent,
            chain: EffectChain::EMPTY,
            quality: EffectQuality::Auto,
        }
    }
}

/// Experimental cache boundary wrapper for declarative element subtrees.
///
/// This is a mechanism-only primitive intended to support GPUI-style view caching experiments
/// without committing to a stable authoring API.
#[derive(Debug, Clone, Copy)]
pub struct ViewCacheProps {
    pub layout: LayoutStyle,
    /// Whether the subtree should be treated as layout-contained by the runtime when view caching is enabled.
    pub contained_layout: bool,
    /// Explicit cache key for view-cache reuse (experimental).
    ///
    /// The runtime will reuse cached output for this view-cache root only when the computed key is
    /// unchanged. This mirrors GPUI's `ViewCacheKey` gating behavior.
    pub cache_key: u64,
}

impl Default for ViewCacheProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            contained_layout: false,
            cache_key: 0,
        }
    }
}

/// Paint-only transform wrapper for declarative element subtrees.
///
/// This applies a `SceneOp::PushTransform` / `PopTransform` around the subtree during painting,
/// without affecting layout, hit-testing, or pointer event coordinates.
///
/// This is intentionally similar to GPUI's `with_transformation(...)` semantics for elements like
/// `Svg`: it is useful for spinners and decorative animations, and is cheap to optimize because it
/// does not require inverse mapping during hit-testing.
#[derive(Debug, Clone, Copy, Default)]
pub struct VisualTransformProps {
    pub layout: LayoutStyle,
    /// A transform expressed in the element's local coordinate space.
    ///
    /// The runtime composes this around the element's bounds origin so that local transforms can be
    /// expressed in px relative to the element (e.g. rotate around `Point(Px(w/2), Px(h/2))`).
    pub transform: fret_core::Transform2D,
}

/// Render transform wrapper for declarative element subtrees.
///
/// This applies `Widget::render_transform(...)` for the subtree rooted at this element:
/// - Paint and hit-testing are both transformed.
/// - Pointer event coordinates are mapped through the inverse transform (when invertible).
/// - Layout bounds remain authoritative (this is not a layout transform).
///
/// This is useful for interactive translations (e.g. drag-to-dismiss surfaces) that must keep input
/// aligned with the rendered output.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderTransformProps {
    pub layout: LayoutStyle,
    pub transform: fret_core::Transform2D,
}

/// Render transform wrapper for declarative element subtrees.
///
/// This is a convenience wrapper for cases where the desired translation is best expressed as a
/// fraction of the element's own laid-out bounds, similar to CSS percentage translate operations.
///
/// This is computed during layout so the first painted frame can use the correct pixel offset.
#[derive(Debug, Clone, Copy, Default)]
pub struct FractionalRenderTransformProps {
    pub layout: LayoutStyle,
    /// Translation in units of the element's own width (e.g. `-1.0` shifts left by one full width).
    pub translate_x_fraction: f32,
    /// Translation in units of the element's own height.
    pub translate_y_fraction: f32,
}

/// Layout-driven anchored placement wrapper for declarative element subtrees (ADR 0104).
///
/// This wrapper computes a placement transform during layout (based on the child's intrinsic
/// size) and applies it via the retained runtime's `Widget::render_transform` hook.
///
/// Unlike `VisualTransformProps`, this affects hit-testing and pointer coordinate mapping.
#[derive(Debug, Clone)]
pub struct AnchoredProps {
    pub layout: LayoutStyle,
    /// Insets applied to the wrapper bounds before placement.
    pub outer_margin: Edges,
    /// Anchor rect in the same coordinate space as the wrapper bounds.
    pub anchor: fret_core::Rect,
    /// Optional anchor element ID to resolve during layout (ADR 0104).
    ///
    /// When set, the layout pass attempts to resolve the element's current-frame bounds and uses
    /// that rect as the anchor. This avoids cross-frame geometry jitter from
    /// `bounds_for_element(...)` / `last_bounds_for_element(...)` queries and better matches GPUI's
    /// layout-driven placement model.
    ///
    /// If the element cannot be resolved (e.g. not mounted yet), `anchor` is used as a fallback.
    pub anchor_element: Option<u64>,
    pub side: Side,
    pub align: Align,
    /// Gap between the anchor and the placed subtree.
    pub side_offset: Px,
    pub options: AnchoredPanelOptions,
    /// Optional output model updated with the computed layout during layout.
    pub layout_out: Option<Model<AnchoredPanelLayout>>,
}

impl Default for AnchoredProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;

        Self {
            layout,
            outer_margin: Edges::all(Px(0.0)),
            anchor: fret_core::Rect::default(),
            anchor_element: None,
            side: Side::Bottom,
            align: Align::Start,
            side_offset: Px(0.0),
            options: AnchoredPanelOptions::default(),
            layout_out: None,
        }
    }
}

/// One `box-shadow` layer (CSS-style) for component-level elevation recipes.
///
/// This is renderer-friendly: runtimes can approximate blur by drawing multiple expanded quads with
/// alpha falloff (ADR 0060) until we have a true blur pipeline.
#[derive(Debug, Clone, Copy)]
pub struct ShadowLayerStyle {
    pub color: Color,
    pub offset_x: Px,
    pub offset_y: Px,
    /// Blur radius in pixels.
    pub blur: Px,
    /// Spread radius in pixels (can be negative).
    pub spread: Px,
}

/// A low-level drop shadow primitive for component-level elevation recipes.
///
/// Many Tailwind/shadcn recipes are multi-layer shadows (e.g. `shadow-md`), so we support up to two
/// layers without forcing heap allocation (keeps `ContainerProps` `Copy`).
#[derive(Debug, Clone, Copy)]
pub struct ShadowStyle {
    pub primary: ShadowLayerStyle,
    pub secondary: Option<ShadowLayerStyle>,
    pub corner_radii: Corners,
}

#[derive(Clone)]
pub struct PressableProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    /// Whether this pressable is a focus traversal stop (Tab order).
    ///
    /// When `false`, the node can still be focused programmatically (e.g. roving focus),
    /// but it is skipped by the default focus traversal.
    pub focusable: bool,
    pub focus_ring: Option<RingStyle>,
    /// Optional override for the bounds used when painting the focus ring.
    ///
    /// Coordinates are **local** to the pressable's origin (i.e. `0,0` is the pressable's top-left),
    /// and are translated into absolute coordinates at paint time.
    ///
    /// This is useful when the pressable is wider than the visual control chrome (e.g. a "row"
    /// pressable that should paint focus ring only around an icon-sized control).
    pub focus_ring_bounds: Option<Rect>,
    pub a11y: PressableA11y,
}

impl std::fmt::Debug for PressableProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("PressableProps");
        out.field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable);

        out.field("focus_ring", &self.focus_ring)
            .field("focus_ring_bounds", &self.focus_ring_bounds)
            .field("a11y", &self.a11y)
            .finish()
    }
}

impl Default for PressableProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            focusable: true,
            focus_ring: None,
            focus_ring_bounds: None,
            a11y: PressableA11y::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct RovingFlexProps {
    pub flex: FlexProps,
    pub roving: RovingFocusProps,
}

impl std::fmt::Debug for RovingFlexProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RovingFlexProps")
            .field("flex", &self.flex)
            .field("roving", &self.roving)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct RovingFocusProps {
    pub enabled: bool,
    pub wrap: bool,
    pub disabled: Arc<[bool]>,
}

impl Default for RovingFocusProps {
    fn default() -> Self {
        Self {
            enabled: true,
            wrap: true,
            disabled: Arc::from([]),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PressableA11y {
    pub role: Option<SemanticsRole>,
    pub label: Option<Arc<str>>,
    /// Debug/test-only identifier for deterministic automation.
    ///
    /// This MUST NOT be mapped into platform accessibility name/label fields by default.
    pub test_id: Option<Arc<str>>,
    /// When true, suppress exposing this pressable to assistive technologies (aria-hidden).
    ///
    /// This is useful for purely visual affordances (e.g. decorative scroll buttons in Radix
    /// Select) that should remain interactive for pointer users but should not appear in the
    /// accessibility tree.
    pub hidden: bool,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<bool>,
    pub active_descendant: Option<NodeId>,
    /// Declarative-only: element ID of a node which labels this node.
    ///
    /// This is an authoring convenience for relationships like `aria-labelledby` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub labelled_by_element: Option<u64>,
    /// Declarative-only: element ID of a node which describes this node.
    ///
    /// This is an authoring convenience for relationships like `aria-describedby` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub described_by_element: Option<u64>,
    /// Declarative-only: element ID of a node which this node controls.
    ///
    /// This is an authoring convenience for relationships like `aria-controls` where the target
    /// is another declarative element. The runtime resolves this into a `NodeId` during semantics
    /// snapshot production.
    pub controls_element: Option<u64>,
    pub pos_in_set: Option<u32>,
    pub set_size: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PressableState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RingPlacement {
    /// Draw the ring inside the element bounds.
    Inset,
    /// Draw the ring outside the element bounds (best effort; may be clipped by parent clips).
    #[default]
    Outset,
}

/// A simple focus ring decoration, intended for component-layer recipes (e.g. shadcn-style
/// focus-visible ring).
///
/// This is intentionally small and renderer-friendly: it maps to one or two `SceneOp::Quad`
/// operations.
#[derive(Debug, Clone, Copy)]
pub struct RingStyle {
    pub placement: RingPlacement,
    pub width: Px,
    pub offset: Px,
    pub color: Color,
    pub offset_color: Option<Color>,
    pub corner_radii: Corners,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StackProps {
    pub layout: LayoutStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RowProps {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MainAlign {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CrossAlign {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub struct SpacerProps {
    pub layout: LayoutStyle,
    pub min: Px,
}

impl Default for SpacerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.flex.grow = 1.0;
        layout.flex.shrink = 1.0;
        layout.flex.basis = Length::Px(Px(0.0));
        Self {
            layout,
            min: Px(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub layout: LayoutStyle,
    pub text: std::sync::Arc<str>,
    pub style: Option<TextStyle>,
    pub color: Option<Color>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

#[derive(Debug, Clone)]
pub struct StyledTextProps {
    pub layout: LayoutStyle,
    pub rich: AttributedText,
    pub style: Option<TextStyle>,
    /// Base color for glyphs without a per-run override.
    pub color: Option<Color>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

#[derive(Debug, Clone)]
pub struct SelectableTextProps {
    pub layout: LayoutStyle,
    pub rich: AttributedText,
    pub style: Option<TextStyle>,
    /// Base color for glyphs without a per-run override.
    pub color: Option<Color>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

#[derive(Debug, Clone)]
pub struct SelectableTextState {
    pub selection_anchor: usize,
    pub caret: usize,
    pub affinity: CaretAffinity,
    pub preferred_x: Option<Px>,
    pub dragging: bool,
    pub last_pointer_pos: Option<fret_core::Point>,
}

impl Default for SelectableTextState {
    fn default() -> Self {
        Self {
            selection_anchor: 0,
            caret: 0,
            affinity: CaretAffinity::Downstream,
            preferred_x: None,
            dragging: false,
            last_pointer_pos: None,
        }
    }
}

#[derive(Clone)]
pub struct TextInputProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub model: Model<String>,
    pub a11y_label: Option<std::sync::Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub test_id: Option<std::sync::Arc<str>>,
    pub placeholder: Option<std::sync::Arc<str>>,
    pub active_descendant: Option<NodeId>,
    pub expanded: Option<bool>,
    pub chrome: TextInputStyle,
    pub text_style: TextStyle,
    pub submit_command: Option<CommandId>,
    pub cancel_command: Option<CommandId>,
}

impl TextInputProps {
    pub fn new(model: Model<String>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            focusable: true,
            model,
            a11y_label: None,
            a11y_role: None,
            test_id: None,
            placeholder: None,
            active_descendant: None,
            expanded: None,
            chrome: TextInputStyle::default(),
            text_style: TextStyle::default(),
            submit_command: None,
            cancel_command: None,
        }
    }
}

impl std::fmt::Debug for TextInputProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInputProps")
            .field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable)
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("a11y_role", &self.a11y_role)
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field(
                "placeholder",
                &self.placeholder.as_ref().map(|s| s.as_ref()),
            )
            .field("expanded", &self.expanded)
            .field("chrome", &self.chrome)
            .field("text_style", &self.text_style)
            .field("submit_command", &self.submit_command)
            .field("cancel_command", &self.cancel_command)
            .finish()
    }
}

#[derive(Clone)]
pub struct TextAreaProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub model: Model<String>,
    pub a11y_label: Option<std::sync::Arc<str>>,
    pub test_id: Option<std::sync::Arc<str>>,
    pub chrome: TextAreaStyle,
    pub text_style: TextStyle,
    pub min_height: Px,
}

impl TextAreaProps {
    pub fn new(model: Model<String>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            enabled: true,
            focusable: true,
            model,
            a11y_label: None,
            test_id: None,
            chrome: TextAreaStyle::default(),
            text_style: TextStyle::default(),
            min_height: Px(80.0),
        }
    }
}

impl std::fmt::Debug for TextAreaProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextAreaProps")
            .field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable)
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("chrome", &self.chrome)
            .field("text_style", &self.text_style)
            .field("min_height", &self.min_height)
            .finish()
    }
}

#[derive(Clone)]
pub struct ResizablePanelGroupProps {
    pub layout: LayoutStyle,
    pub axis: fret_core::Axis,
    pub model: Model<Vec<f32>>,
    pub min_px: Vec<Px>,
    pub enabled: bool,
    pub chrome: ResizablePanelGroupStyle,
}

impl ResizablePanelGroupProps {
    pub fn new(axis: fret_core::Axis, model: Model<Vec<f32>>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            axis,
            model,
            min_px: Vec::new(),
            enabled: true,
            chrome: ResizablePanelGroupStyle::default(),
        }
    }
}

impl std::fmt::Debug for ResizablePanelGroupProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizablePanelGroupProps")
            .field("layout", &self.layout)
            .field("axis", &self.axis)
            .field("model", &"<model>")
            .field("min_px_len", &self.min_px.len())
            .field("enabled", &self.enabled)
            .field("chrome", &self.chrome)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageProps {
    pub layout: LayoutStyle,
    pub image: ImageId,
    pub opacity: f32,
    pub uv: Option<UvRect>,
}

impl ImageProps {
    pub fn new(image: ImageId) -> Self {
        Self {
            layout: LayoutStyle::default(),
            image,
            opacity: 1.0,
            uv: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportSurfaceProps {
    pub layout: LayoutStyle,
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub opacity: f32,
}

impl ViewportSurfaceProps {
    pub fn new(target: RenderTargetId) -> Self {
        Self {
            layout: LayoutStyle::default(),
            target,
            target_px_size: (1, 1),
            fit: ViewportFit::Stretch,
            opacity: 1.0,
        }
    }
}

/// A declarative leaf canvas element.
///
/// Paint handlers are registered via element-local state (not props) so the element tree can
/// remain `Clone + Debug` (see ADR 0156).
#[derive(Debug, Clone, Copy)]
pub struct CanvasProps {
    pub layout: LayoutStyle,
    pub cache_policy: CanvasCachePolicy,
}

/// Cache tuning for a single hosted resource kind (text/path/svg) within a declarative `Canvas`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasCacheTuning {
    /// How long an unused entry may remain cached (in UI frames).
    pub keep_frames: u64,
    /// Hard cap on cached entries for this resource kind.
    pub max_entries: usize,
}

impl CanvasCacheTuning {
    pub const fn transient() -> Self {
        Self {
            keep_frames: 0,
            max_entries: 0,
        }
    }
}

/// Hosted cache policy for declarative `Canvas` resources.
///
/// This is intentionally numeric-only configuration: it does not encode interaction policy or
/// domain semantics (ADR 0156 / ADR 0137).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasCachePolicy {
    pub text: CanvasCacheTuning,
    pub shared_text: CanvasCacheTuning,
    pub path: CanvasCacheTuning,
    pub svg: CanvasCacheTuning,
}

impl CanvasCachePolicy {
    pub const fn smooth_default() -> Self {
        Self {
            // ~1s at 60fps; reduces prepare/release thrash during scroll/pan.
            text: CanvasCacheTuning {
                keep_frames: 60,
                max_entries: 4096,
            },
            // Shared cache (keyed by content/style/constraints) is useful for repeated labels,
            // but should remain bounded and configurable for large, scroll-driven surfaces.
            //
            // Default preserves the previous hard-coded behavior in `CanvasCache`.
            shared_text: CanvasCacheTuning {
                keep_frames: 120,
                max_entries: 4096,
            },
            path: CanvasCacheTuning {
                keep_frames: 60,
                max_entries: 2048,
            },
            svg: CanvasCacheTuning {
                keep_frames: 60,
                max_entries: 256,
            },
        }
    }
}

impl Default for CanvasCachePolicy {
    fn default() -> Self {
        Self::smooth_default()
    }
}

impl Default for CanvasProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            cache_policy: CanvasCachePolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SvgIconProps {
    pub layout: LayoutStyle,
    pub svg: SvgSource,
    pub fit: SvgFit,
    pub color: Color,
    pub opacity: f32,
}

impl SvgIconProps {
    pub fn new(svg: SvgSource) -> Self {
        Self {
            layout: LayoutStyle::default(),
            svg,
            fit: SvgFit::Contain,
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
        }
    }
}

/// A simple loading spinner primitive.
///
/// This is intentionally low-opinionated and renderer-friendly: it paints a ring of small rounded
/// quads with frame-driven alpha modulation (`Effect::RequestAnimationFrame`).
#[derive(Debug, Clone, Copy)]
pub struct SpinnerProps {
    pub layout: LayoutStyle,
    pub color: Option<Color>,
    pub dot_count: u8,
    /// Phase increment per frame, in dot steps. (`0.0` disables animation.)
    pub speed: f32,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Px(Px(16.0));
        layout.size.height = Length::Px(Px(16.0));

        Self {
            layout,
            color: None,
            dot_count: 12,
            speed: 0.2,
        }
    }
}

/// A hover tracking region primitive.
///
/// This is a small substrate building block: it provides a `hovered: bool` signal to component
/// code (via `ElementCx::hover_region(...)`) without imposing click/focus semantics.
#[derive(Debug, Clone, Copy, Default)]
pub struct HoverRegionProps {
    pub layout: LayoutStyle,
}

/// A wheel listener region that mutates a scroll handle without affecting layout.
#[derive(Debug, Clone)]
pub struct WheelRegionProps {
    pub layout: LayoutStyle,
    pub axis: ScrollAxis,
    /// Declarative element id to invalidate when the scroll offset changes.
    pub scroll_target: Option<GlobalElementId>,
    pub scroll_handle: crate::scroll::ScrollHandle,
}

impl Default for WheelRegionProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            axis: ScrollAxis::Y,
            scroll_target: None,
            scroll_handle: crate::scroll::ScrollHandle::default(),
        }
    }
}

impl TextProps {
    pub fn new(text: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            layout: LayoutStyle::default(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }

    pub(crate) fn resolved_text_style(&self, theme: crate::ThemeSnapshot) -> TextStyle {
        crate::text_props::resolve_text_style(theme, self.style.clone())
    }

    pub(crate) fn build_text_input_with_style(&self, style: TextStyle) -> fret_core::TextInput {
        crate::text_props::build_text_input_plain(self.text.clone(), style)
    }

    pub(crate) fn build_text_input(&self, theme: crate::ThemeSnapshot) -> fret_core::TextInput {
        self.build_text_input_with_style(self.resolved_text_style(theme))
    }
}

impl StyledTextProps {
    pub fn new(rich: AttributedText) -> Self {
        Self {
            layout: LayoutStyle::default(),
            rich,
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }

    pub(crate) fn resolved_text_style(&self, theme: crate::ThemeSnapshot) -> TextStyle {
        crate::text_props::resolve_text_style(theme, self.style.clone())
    }

    pub(crate) fn build_text_input_with_style(&self, style: TextStyle) -> fret_core::TextInput {
        crate::text_props::build_text_input_attributed(&self.rich, style)
    }

    pub(crate) fn build_text_input(&self, theme: crate::ThemeSnapshot) -> fret_core::TextInput {
        self.build_text_input_with_style(self.resolved_text_style(theme))
    }
}

impl SelectableTextProps {
    pub fn new(rich: AttributedText) -> Self {
        Self {
            layout: LayoutStyle::default(),
            rich,
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }

    pub(crate) fn resolved_text_style(&self, theme: crate::ThemeSnapshot) -> TextStyle {
        crate::text_props::resolve_text_style(theme, self.style.clone())
    }

    pub(crate) fn build_text_input_with_style(&self, style: TextStyle) -> fret_core::TextInput {
        crate::text_props::build_text_input_attributed(&self.rich, style)
    }

    pub(crate) fn build_text_input(&self, theme: crate::ThemeSnapshot) -> fret_core::TextInput {
        self.build_text_input_with_style(self.resolved_text_style(theme))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexProps {
    pub layout: LayoutStyle,
    pub direction: fret_core::Axis,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
    pub wrap: bool,
}

impl Default for FlexProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridProps {
    pub layout: LayoutStyle,
    pub cols: u16,
    pub rows: Option<u16>,
    pub gap: Px,
    pub padding: Edges,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for GridProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
            cols: 1,
            rows: None,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListProps {
    pub layout: LayoutStyle,
    pub axis: fret_core::Axis,
    pub len: usize,
    pub items_revision: u64,
    pub estimate_row_height: Px,
    pub measure_mode: VirtualListMeasureMode,
    pub key_cache: VirtualListKeyCacheMode,
    pub overscan: usize,
    pub scroll_margin: Px,
    pub gap: Px,
    pub scroll_handle: crate::scroll::VirtualListScrollHandle,
    pub visible_items: Vec<crate::virtual_list::VirtualItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualListMeasureMode {
    /// Performs a measurement pass for all visible items and updates the virtualizer with the
    /// measured sizes. Correct for variable-height items.
    Measured,
    /// Skips the measurement pass and assumes all items have the estimated size.
    /// Intended for fixed-height lists/tables.
    Fixed,
    /// Skips the measurement pass and uses caller-provided per-index row heights.
    ///
    /// This mode is intended for “known-height” virtualization (e.g. fixed-height rows with
    /// occasional deterministic height changes like group headers), where measuring each visible
    /// row would be wasted work.
    ///
    /// Correctness requires that the provided height function matches the rendered row layout.
    Known,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VirtualListKeyCacheMode {
    /// Cache the full `index -> key` mapping so we can:
    /// - restore scroll anchor across reorder
    /// - provide stable keys to measured virtualization
    #[default]
    AllKeys,
    /// Do not cache `index -> key`. Keys are computed on-demand for visible items only.
    ///
    /// This is intended for very large fixed-height lists (e.g. tables) where caching the full
    /// key map can dominate startup time and memory.
    VisibleOnly,
}

#[derive(Clone)]
pub struct VirtualListOptions {
    pub axis: fret_core::Axis,
    pub items_revision: u64,
    pub estimate_row_height: Px,
    pub measure_mode: VirtualListMeasureMode,
    pub key_cache: VirtualListKeyCacheMode,
    pub overscan: usize,
    pub scroll_margin: Px,
    pub gap: Px,
    pub known_row_height_at: Option<Arc<dyn Fn(usize) -> Px + Send + Sync>>,
}

impl VirtualListOptions {
    pub fn new(estimate_row_height: Px, overscan: usize) -> Self {
        Self {
            axis: fret_core::Axis::Vertical,
            items_revision: 0,
            estimate_row_height,
            measure_mode: VirtualListMeasureMode::Measured,
            key_cache: VirtualListKeyCacheMode::AllKeys,
            overscan,
            scroll_margin: Px(0.0),
            gap: Px(0.0),
            known_row_height_at: None,
        }
    }

    pub fn fixed(estimate_row_height: Px, overscan: usize) -> Self {
        Self {
            measure_mode: VirtualListMeasureMode::Fixed,
            ..Self::new(estimate_row_height, overscan)
        }
    }

    pub fn known(
        estimate_row_height: Px,
        overscan: usize,
        height_at: impl Fn(usize) -> Px + Send + Sync + 'static,
    ) -> Self {
        let mut options = Self::new(estimate_row_height, overscan);
        options.measure_mode = VirtualListMeasureMode::Known;
        options.known_row_height_at = Some(Arc::new(height_at));
        options
    }
}

impl std::fmt::Debug for VirtualListOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListOptions")
            .field("axis", &self.axis)
            .field("items_revision", &self.items_revision)
            .field("estimate_row_height", &self.estimate_row_height)
            .field("measure_mode", &self.measure_mode)
            .field("key_cache", &self.key_cache)
            .field("overscan", &self.overscan)
            .field("scroll_margin", &self.scroll_margin)
            .field("gap", &self.gap)
            .field("known_row_height_at", &self.known_row_height_at.is_some())
            .finish()
    }
}

/// Cross-frame element-local state for a virtual list (stored in the element state store).
#[derive(Debug, Default, Clone)]
pub struct VirtualListState {
    pub offset_x: Px,
    pub offset_y: Px,
    pub viewport_w: Px,
    pub viewport_h: Px,
    pub(crate) window_range: Option<crate::virtual_list::VirtualRange>,
    pub(crate) render_window_range: Option<crate::virtual_list::VirtualRange>,
    pub(crate) has_final_viewport: bool,
    pub(crate) deferred_scroll_offset_hint: Option<Px>,
    pub(crate) metrics: crate::virtual_list::VirtualListMetrics,
    pub(crate) items_revision: u64,
    pub(crate) items_len: usize,
    pub(crate) key_cache: VirtualListKeyCacheMode,
    pub(crate) keys: Vec<crate::ItemKey>,
}

#[derive(Debug, Clone)]
pub struct ScrollProps {
    pub layout: LayoutStyle,
    pub axis: ScrollAxis,
    pub scroll_handle: Option<crate::scroll::ScrollHandle>,
    pub intrinsic_measure_mode: ScrollIntrinsicMeasureMode,
    /// When true, the scroll subtree's paint output depends on the scroll offset in a
    /// windowed/virtualized way (e.g. a single `Canvas` that only paints the visible range).
    ///
    /// In this mode, scroll-handle updates must be allowed to invalidate view-cache reuse so the
    /// subtree can re-render and re-run paint handlers for the new visible window.
    ///
    /// This is a mechanism-only switch; policy lives in ecosystem layers.
    pub windowed_paint: bool,
    /// When true (default), scroll containers probe their content with a very large available size
    /// along the scroll axis to measure the full scrollable extent.
    ///
    /// When false, probing uses the viewport constraints, which allows word-wrapping content while
    /// still permitting scrolling for long unbreakable tokens.
    pub probe_unbounded: bool,
}

impl Default for ScrollProps {
    fn default() -> Self {
        let layout = LayoutStyle {
            overflow: Overflow::Clip,
            ..Default::default()
        };
        Self {
            layout,
            axis: ScrollAxis::Y,
            scroll_handle: None,
            intrinsic_measure_mode: ScrollIntrinsicMeasureMode::Content,
            windowed_paint: false,
            probe_unbounded: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollIntrinsicMeasureMode {
    /// Default behavior: scroll measurement probes children (potentially using MaxContent on the
    /// scroll axis when `probe_unbounded` is true).
    Content,
    /// Treat the scroll container as a viewport-sized barrier in intrinsic measurement contexts.
    ///
    /// This avoids recursively measuring large scrollable subtrees (virtualized surfaces, large
    /// tables, code views) during Min/MaxContent measurement passes.
    ///
    /// Note: this affects only `measure()` / intrinsic sizing; final layout under definite
    /// available space is unchanged.
    Viewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    X,
    Y,
    Both,
}

impl ScrollAxis {
    pub fn scroll_x(self) -> bool {
        matches!(self, Self::X | Self::Both)
    }

    pub fn scroll_y(self) -> bool {
        matches!(self, Self::Y | Self::Both)
    }
}

/// Cross-frame element-local state for scroll containers.
#[derive(Debug, Default, Clone)]
pub struct ScrollState {
    pub scroll_handle: crate::scroll::ScrollHandle,
    pub(crate) intrinsic_measure_cache: Option<ScrollIntrinsicMeasureCache>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScrollIntrinsicMeasureCacheKey {
    pub avail_w: u64,
    pub avail_h: u64,
    pub axis: u8,
    pub probe_unbounded: bool,
    pub scale_bits: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScrollIntrinsicMeasureCache {
    pub key: ScrollIntrinsicMeasureCacheKey,
    pub max_child: Size,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarStyle {
    pub thumb: Color,
    pub thumb_hover: Color,
    pub thumb_idle_alpha: f32,
    /// Padding (main axis) reserved at both ends of the scrollbar track.
    ///
    /// This is part of Radix ScrollArea's thumb sizing/offset math. Component libraries should set
    /// this to match the visual padding they apply to the scrollbar container (e.g. shadcn/ui v4
    /// uses `p-px`, so `Px(1.0)`).
    pub track_padding: Px,
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            thumb: Color {
                r: 0.35,
                g: 0.38,
                b: 0.45,
                a: 1.0,
            },
            thumb_hover: Color {
                r: 0.45,
                g: 0.50,
                b: 0.60,
                a: 1.0,
            },
            thumb_idle_alpha: 0.65,
            track_padding: Px(1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollbarAxis {
    #[default]
    Vertical,
    Horizontal,
}

/// A mechanism-only scrollbar primitive.
///
/// Component libraries decide when to show/hide scrollbars and resolve theme tokens into this
/// style. The runtime owns hit-testing, thumb/track interactions, and paints using the resolved
/// style.
#[derive(Debug, Clone, Default)]
pub struct ScrollbarProps {
    pub layout: LayoutStyle,
    pub axis: ScrollbarAxis,
    /// Declarative element id for the associated scroll container, if any.
    ///
    /// When provided, the scrollbar will invalidate the target node's layout/paint when the
    /// scroll handle offset changes (e.g. thumb drag or track paging).
    pub scroll_target: Option<GlobalElementId>,
    pub scroll_handle: crate::scroll::ScrollHandle,
    pub style: ScrollbarStyle,
}

/// Cross-frame element-local state for scrollbars.
#[derive(Debug, Default, Clone)]
pub struct ScrollbarState {
    pub dragging_thumb: bool,
    pub drag_start_pointer: Px,
    pub drag_start_offset: Px,
    pub hovered: bool,
}

/// Authoring conversion boundary (ADR 0039).
pub trait IntoElement {
    fn into_element(self, id: GlobalElementId) -> AnyElement;
}

/// A small owned collection wrapper for element lists.
///
/// This is intended for authoring-facing APIs that want an "iterator-friendly" return type without
/// forcing callers into `Vec<AnyElement>` as the only option.
#[derive(Debug, Clone, Default)]
pub struct Elements(pub Vec<AnyElement>);

impl Elements {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self(children.into_iter().collect())
    }

    pub fn into_vec(self) -> Vec<AnyElement> {
        self.0
    }
}

impl From<Vec<AnyElement>> for Elements {
    fn from(value: Vec<AnyElement>) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<[AnyElement; N]> for Elements {
    fn from(value: [AnyElement; N]) -> Self {
        Self::new(value)
    }
}

impl std::iter::FromIterator<AnyElement> for Elements {
    fn from_iter<T: IntoIterator<Item = AnyElement>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl std::ops::Deref for Elements {
    type Target = Vec<AnyElement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Elements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Elements {
    type Item = AnyElement;
    type IntoIter = std::vec::IntoIter<AnyElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Elements {
    type Item = &'a AnyElement;
    type IntoIter = std::slice::Iter<'a, AnyElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Elements {
    type Item = &'a mut AnyElement;
    type IntoIter = std::slice::IterMut<'a, AnyElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

/// Authoring helper for collecting iterator-produced child elements.
///
/// This exists to reduce boilerplate after switching common `children: Vec<AnyElement>` APIs to
/// accept `IntoIterator<Item = AnyElement>` (e.g. `ElementContext::{row,column}`), where the target
/// collection type is no longer implied by the callee.
///
/// Example:
/// `let children = (0..10).map(|i| cx.text(format!("row-{i}"))).elements();`
pub trait AnyElementIterExt: Iterator<Item = AnyElement> + Sized {
    fn elements(self) -> Vec<AnyElement> {
        self.collect()
    }

    fn elements_owned(self) -> Elements {
        self.collect::<Elements>()
    }
}

impl<T> AnyElementIterExt for T where T: Iterator<Item = AnyElement> + Sized {}

impl IntoElement for AnyElement {
    fn into_element(self, _id: GlobalElementId) -> AnyElement {
        self
    }
}

impl IntoElement for TextProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Text(self), Vec::new())
    }
}

impl IntoElement for StyledTextProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::StyledText(self), Vec::new())
    }
}

impl IntoElement for SelectableTextProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::SelectableText(self), Vec::new())
    }
}

impl IntoElement for ImageProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Image(self), Vec::new())
    }
}

impl IntoElement for ViewportSurfaceProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::ViewportSurface(self), Vec::new())
    }
}

impl IntoElement for SvgIconProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::SvgIcon(self), Vec::new())
    }
}

impl IntoElement for ScrollProps {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        AnyElement::new(id, ElementKind::Scroll(self), Vec::new())
    }
}

impl IntoElement for std::sync::Arc<str> {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        TextProps::new(self).into_element(id)
    }
}

impl IntoElement for &'static str {
    fn into_element(self, id: GlobalElementId) -> AnyElement {
        TextProps::new(self).into_element(id)
    }
}

/// Stateful view authoring layer (ADR 0039).
pub trait Render {
    fn render<H: UiHost>(&mut self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

/// Stateless component authoring layer (ADR 0039).
pub trait RenderOnce {
    fn render_once<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}
