pub use fret_core::LayoutDirection;

use fret_core::{Edges, Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StickyMode {
    Partial,
    #[default]
    Always,
}

/// Offset configuration inspired by Floating UI's `offset()` middleware.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Offset {
    /// Distance along the placement side axis (the "gap" between anchor and panel).
    pub main_axis: Px,
    /// Distance along the alignment axis (skidding).
    pub cross_axis: Px,
    /// Optional skidding override for aligned placements (Start/End).
    ///
    /// When present and `align != Center`, this overrides `cross_axis` and flips sign for `End`.
    /// For vertical placements (Top/Bottom), the direction is also flipped under RTL.
    pub alignment_axis: Option<Px>,
}

/// Collision/overflow options inspired by Floating UI's `detectOverflow` configuration.
///
/// This is applied to the `outer` boundary before running the placement solver:
///
/// 1) If `boundary` is set, intersect `outer` with it (clipping ancestor style).
/// 2) Inset by `padding` (collision padding).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CollisionOptions {
    pub padding: Edges,
    pub boundary: Option<Rect>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AnchoredPanelOptions {
    pub direction: LayoutDirection,
    pub offset: Offset,
    /// Shift/clamp policy for keeping the floating panel within the collision boundary.
    ///
    /// This is inspired by Floating UI's `shift()` middleware options.
    pub shift: ShiftOptions,
    pub arrow: Option<ArrowOptions>,
    pub collision: CollisionOptions,
    pub sticky: StickyMode,
}

/// Shift configuration inspired by Floating UI's `shift()` middleware.
///
/// - `main_axis` clamps the panel along the placement axis (y for Top/Bottom, x for Left/Right).
/// - `cross_axis` clamps the panel along the alignment axis (x for Top/Bottom, y for Left/Right).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShiftOptions {
    pub main_axis: bool,
    pub cross_axis: bool,
}

impl Default for ShiftOptions {
    fn default() -> Self {
        Self {
            main_axis: true,
            cross_axis: true,
        }
    }
}

/// Arrow positioning options inspired by Floating UI's `arrow()` middleware.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowOptions {
    /// Arrow element size (in the same coordinate space as `outer`/`anchor`/`content`).
    pub size: Size,
    /// Padding between the arrow and the floating element edges (useful for rounded corners).
    pub padding: Edges,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowLayout {
    /// Which side of the floating panel the arrow should attach to.
    pub side: Side,
    /// Offset along the arrow's axis inside the floating panel (x for Top/Bottom, y for Left/Right).
    pub offset: Px,
    /// The alignment-axis translation applied to the panel to keep the arrow pointing at the anchor
    /// when the anchor is too small (Radix/Floating behavior).
    pub alignment_offset: Px,
    /// Signed center delta between the ideal arrow center point and the clamped offset.
    ///
    /// This matches Floating UI's `centerOffset` and is used by Radix to determine whether the arrow
    /// should be hidden (`shouldHideArrow`).
    pub center_offset: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchoredPanelLayout {
    pub rect: Rect,
    pub side: Side,
    pub align: Align,
    pub arrow: Option<ArrowLayout>,
}

/// Debug-only trace capturing key decisions made by the anchored panel placement solver.
///
/// This is intended for diagnostics evidence (e.g. `fretboard diag`) so scripted repros can report
/// why a popover/menu was flipped/clamped/shifted without relying on screenshots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchoredPanelLayoutTrace {
    /// Solver inputs before applying collision options.
    pub outer_input: Rect,
    /// Collision boundary after applying `options.collision` (boundary + padding).
    pub outer_collision: Rect,
    pub anchor: Rect,
    /// Desired (intrinsic) content size.
    pub desired: Size,
    /// Requested side gap (before `options.offset.main_axis` is applied).
    pub side_offset: Px,
    pub preferred_side: Side,
    pub align: Align,
    pub options: AnchoredPanelOptions,
    /// Effective main-axis gap used by the solver (`side_offset + options.offset.main_axis`).
    pub gap: Px,

    /// Candidate rect for the preferred side (pre-shift, pre-arrow).
    pub preferred_rect: Rect,
    /// Candidate rect for the flipped side (pre-shift, pre-arrow).
    pub flipped_rect: Rect,
    /// Whether the preferred side fit without requiring main-axis clamping.
    pub preferred_fits_without_main_clamp: bool,
    /// Whether the flipped side fit without requiring main-axis clamping.
    pub flipped_fits_without_main_clamp: bool,

    pub preferred_available_main_px: f32,
    pub flipped_available_main_px: f32,

    /// The side chosen by the solver before shift/clamp/arrow adjustments.
    pub chosen_side: Side,
    /// The chosen candidate rect before shift/clamp/arrow adjustments.
    pub chosen_rect: Rect,
    /// The rect after applying shift/clamp (sticky/shift policy), before arrow adjustments.
    pub rect_after_shift: Rect,
    /// Signed delta applied by shift/clamp (after_shift.origin - chosen.origin).
    pub shift_delta: fret_core::Point,

    /// Final layout output (after shift + arrow).
    pub layout: AnchoredPanelLayout,
}
