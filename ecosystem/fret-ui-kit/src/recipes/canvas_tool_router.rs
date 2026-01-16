// Re-exported for compatibility.
//
// Canvas-specific recipes now live in `ecosystem/fret-canvas` behind the `fret-canvas/ui` feature.
pub use fret_canvas::ui::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolEventCx, CanvasToolHandlers, CanvasToolId,
    CanvasToolRouterProps, OnCanvasToolPaint, OnCanvasToolPinch, OnCanvasToolPointerDown,
    OnCanvasToolPointerMove, OnCanvasToolPointerUp, OnCanvasToolWheel, canvas_tool_router_panel,
};
