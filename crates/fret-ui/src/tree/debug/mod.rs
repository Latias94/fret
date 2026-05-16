mod commands;
#[cfg(feature = "diagnostics")]
mod diagnostics;
#[cfg(feature = "diagnostics")]
mod dispatch_snapshot;
mod frame_stats;
mod internal;
mod invalidation;
mod layers;
mod layout;
mod scroll;
mod text;
mod view_cache;
mod virtual_list;

pub use commands::UiDebugCommandAvailabilityHotspot;
pub use frame_stats::UiDebugFrameStats;
pub use invalidation::{
    UiDebugDirtyView, UiDebugGlobalChangeHotspot, UiDebugGlobalChangeUnobserved,
    UiDebugHoverDeclarativeInvalidationHotspot, UiDebugInvalidationDetail,
    UiDebugInvalidationSource, UiDebugInvalidationWalk, UiDebugModelChangeHotspot,
    UiDebugModelChangeUnobserved, UiDebugNotifyRequest,
};
pub use layers::{PointerOcclusion, UiDebugHitTest, UiDebugLayerInfo, UiInputArbitrationSnapshot};
pub use layout::{
    UiDebugLayoutDirtyDescendant, UiDebugLayoutEngineMeasureChildHotspot,
    UiDebugLayoutEngineMeasureHotspot, UiDebugLayoutEngineSolve, UiDebugLayoutHotspot,
    UiDebugLayoutRequestBuildRoot, UiDebugPaintWidgetHotspot, UiDebugWidgetMeasureHotspot,
};
pub use scroll::{
    UiDebugScrollAxis, UiDebugScrollHandleChange, UiDebugScrollHandleChangeKind,
    UiDebugScrollLayoutKindProfile, UiDebugScrollLayoutPassKind, UiDebugScrollLayoutPhaseProfile,
    UiDebugScrollLayoutProfile, UiDebugScrollNodeTelemetry,
    UiDebugScrollOverflowObservationTelemetry, UiDebugScrollbarTelemetry,
};
pub use text::{UiDebugPaintTextPrepareHotspot, UiDebugTextConstraintsSnapshot};
pub use view_cache::{UiDebugBoundaryStats, UiDebugCacheRootReuseReason, UiDebugCacheRootStats};
pub use virtual_list::{
    UiDebugPrepaintAction, UiDebugPrepaintActionKind, UiDebugRetainedVirtualListReconcile,
    UiDebugRetainedVirtualListReconcileKind, UiDebugVirtualListWindow,
    UiDebugVirtualListWindowShiftApplyMode, UiDebugVirtualListWindowShiftKind,
    UiDebugVirtualListWindowShiftReason, UiDebugVirtualListWindowShiftSample,
    UiDebugVirtualListWindowSource,
};

#[cfg(feature = "diagnostics")]
pub use diagnostics::{
    UiDebugOverlayPolicyDecisionWrite, UiDebugParentSeverWrite, UiDebugRemoveSubtreeFrameContext,
    UiDebugRemoveSubtreeOutcome, UiDebugRemoveSubtreeRecord, UiDebugSetChildrenWrite,
    UiDebugSetLayerVisibleWrite,
};
#[cfg(feature = "diagnostics")]
pub use dispatch_snapshot::UiDebugDispatchSnapshotParityReport;
#[cfg(feature = "diagnostics")]
pub use dispatch_snapshot::{UiDebugDispatchSnapshot, UiDebugDispatchSnapshotNode};

pub(in crate::tree) use internal::{
    DebugLayoutStackFrame, DebugPaintStackFrame, DebugWidgetMeasureStackFrame,
};
pub(in crate::tree) use invalidation::UiDebugHoverDeclarativeInvalidationCounts;
pub(in crate::tree) use invalidation::UiDebugLayoutDirtySource;
pub(in crate::tree) use view_cache::DebugViewCacheRootRecord;
pub(crate) use virtual_list::{
    UiDebugVirtualListWindowShiftClassificationInput, classify_virtual_list_window_shift,
    fallback_virtual_list_window_shift_detail,
};
