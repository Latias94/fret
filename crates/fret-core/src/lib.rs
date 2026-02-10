//! Portable, backend-agnostic core vocabulary for the Fret workspace.
//!
//! This crate is intentionally dependency-light. It must remain usable by:
//!
//! - core/runtime crates (`fret-runtime`, `fret-ui`),
//! - ecosystem crates (policy/component layers),
//! - and backends/runners (native + web),
//!
//! without pulling in backend bindings (`winit`, `wgpu`, `web-sys`) or forcing an async runtime.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-core/README.md`.

pub mod cursor;
pub mod dock;
pub mod file_dialog;
pub mod geometry;
pub mod ids;
pub mod image;
pub mod input;
pub mod layout_direction;
pub mod materials;
pub mod panels;
pub mod render_text;
pub mod scene;
pub mod semantics;
pub mod services;
pub mod streaming;
pub mod svg;
pub mod text;
pub mod time;
pub mod utf;
pub mod vector_path;
pub mod viewport;
pub mod window;

// -----------------------------------------------------------------------------
// Stable re-exports (core vocabulary surface)
// -----------------------------------------------------------------------------
pub use cursor::CursorIcon;
pub use dock::layout::{
    DOCK_LAYOUT_VERSION, DockLayout, DockLayoutBuilder, DockLayoutFloatingWindow, DockLayoutNode,
    DockLayoutValidationError, DockLayoutValidationErrorKind, DockLayoutWindow, DockRect,
    DockWindowPlacement, EditorDockLayoutSpec,
};
pub use dock::op::{DockOp, SplitFractionsUpdate};
pub use dock::{
    Axis, DockFloatingWindow, DockGraph, DockNode, DockOpApplyError, DockOpApplyErrorKind, DropZone,
};
pub use file_dialog::{
    FileDialogDataEvent, FileDialogFilter, FileDialogOptions, FileDialogSelection,
};
pub use geometry::{Corners, Edges, Point, Px, Rect, RectPx, Size, Transform2D};
pub use ids::{
    AppWindowId, ClipboardToken, DockNodeId, ExternalDropToken, FileDialogToken, FontId, FrameId,
    ImageId, ImageUpdateToken, ImageUploadToken, MaterialId, NodeId, PathId, PointerId,
    RenderTargetId, SvgId, TextBlobId, TimerToken, ViewId,
};
pub use image::{
    AlphaMode, ChromaSiting, ColorPrimaries, ColorRange, ImageColorInfo, ImageColorSpace,
    ImageEncoding, TransferFunction, YuvMatrix,
};
pub use input::{
    Event, ExternalDragEvent, ExternalDragFile, ExternalDragFiles, ExternalDragKind,
    ExternalDropDataEvent, ExternalDropFileData, ExternalDropReadError, ExternalDropReadLimits,
    ImageUpdateDropReason, ImeEvent, InternalDragEvent, InternalDragKind, KeyCode, Modifiers,
    MouseButton, PointerCancelEvent, PointerCancelReason, PointerEvent, PointerType,
    keycode_to_ascii_lowercase,
};
pub use input::{MouseButtons, ViewportInputEvent, ViewportInputGeometry, ViewportInputKind};
pub use layout_direction::LayoutDirection;
pub use materials::{MaterialDescriptor, MaterialKind, MaterialRegistrationError, MaterialService};
pub use panels::{PanelKey, PanelKind};
pub use render_text::{RendererGlyphAtlasPerfSnapshot, RendererTextPerfSnapshot};
pub use scene::{
    Color, ColorSpace, DitherMode, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep,
    GradientStop, LinearGradient, MAX_STOPS, MaterialParams, Paint, RadialGradient, Scene, SceneOp,
    SceneRecording, SceneValidationError, SceneValidationErrorKind, TileMode, UvRect,
};
pub use semantics::{
    SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRole, SemanticsRoot,
    SemanticsSnapshot,
};
pub use services::UiServices;
pub use streaming::StreamingUploadPerfSnapshot;
pub use svg::{SvgFit, SvgService};
pub use text::{
    AttributedText, CaretAffinity, DecorationLineStyle, FontWeight, HitTestResult,
    StrikethroughStyle, TextConstraints, TextFontFamilyConfig, TextInput, TextInputRef,
    TextMetrics, TextOverflow, TextPaintStyle, TextService, TextShapingStyle, TextSlant, TextSpan,
    TextStyle, TextWrap, UnderlineStyle,
};
pub use vector_path::{
    FillRule, FillStyle, PathCommand, PathConstraints, PathMetrics, PathService, PathStyle,
    StrokeStyle,
};
pub use viewport::{ViewportFit, ViewportMapped, ViewportMapping};
pub use window::{WindowAnchor, WindowLogicalPosition, WindowMetricsService};
