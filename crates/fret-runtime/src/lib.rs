//! Portable runtime contracts and value types shared across the Fret workspace.
//!
//! This crate intentionally avoids backend bindings (`winit`, `wgpu`, `web-sys`) and must not
//! force a global async runtime (Tokio, etc.).
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-runtime/README.md`.

pub mod capabilities;
pub mod command;
pub mod commands;
pub mod docking_settings;
pub mod drag;
pub mod effect;
pub mod execution;
pub mod font_bootstrap;
pub mod font_catalog;
pub mod font_catalog_cache;
#[cfg(test)]
mod font_config_tests;
pub mod input;
pub mod interaction_diagnostics;
pub mod keymap;
pub mod menu;
pub mod model;
pub mod platform_completion;
pub mod platform_text_input;
pub mod shortcut_routing_diagnostics;
pub mod text_interaction_settings;
pub mod time;
pub mod ui_host;
pub mod when_expr;
pub mod window_command_action_availability;
pub mod window_command_availability;
pub mod window_command_enabled;
pub mod window_command_gating;
pub mod window_input_arbitration;
pub mod window_input_context;
pub mod window_menu_bar_focus;
pub mod window_metrics;
pub mod window_text_boundary_mode;
pub mod window_text_input_snapshot;

// -----------------------------------------------------------------------------
// Stable re-exports (portable runtime contract surface)
// -----------------------------------------------------------------------------
pub use capabilities::{
    ExecBackgroundWork, ExecCapabilities, ExecTimers, ExecWake, ExternalDragPayloadKind,
    ExternalDragPositionQuality, PlatformCapabilities, ShellCapabilities,
    WindowHoverDetectionQuality, WindowSetOuterPositionQuality, WindowZLevelQuality,
};
pub use command::CommandId;
pub use commands::{CommandMeta, CommandRegistry, CommandScope, OsAction};
pub use docking_settings::{
    DockDragInversionModifier, DockDragInversionPolicy, DockDragInversionSettings,
    DockingInteractionSettings,
};
pub use drag::{
    DRAG_KIND_DOCK_PANEL, DRAG_KIND_DOCK_TABS, DragKindId, DragPhase, DragSession, DragSessionId,
};
pub use effect::{
    ActivationPolicy, CreateWindowKind, CreateWindowRequest, Effect, TaskbarVisibility,
    WindowRequest, WindowRole, WindowStyleRequest, WindowZLevel,
};
pub use execution::{
    DispatchPriority, Dispatcher, DispatcherHandle, InboxDrain, InboxDrainHost, InboxDrainRegistry,
    Runnable,
};
pub use font_bootstrap::{FontCatalogUpdate, FontFamilyDefaultsPolicy, apply_font_catalog_update};
pub use font_catalog::{FontCatalog, TextFontStackKey};
pub use font_catalog_cache::FontCatalogCache;
pub use fret_core::FrameId;
pub use fret_core::ImageUpdateToken;
pub use fret_core::ImageUploadToken;
pub use fret_core::{
    AlphaMode, ChromaSiting, ColorPrimaries, ColorRange, ImageColorInfo, ImageColorSpace,
    ImageEncoding, TransferFunction, YuvMatrix,
};
pub use fret_core::{ClipboardToken, ExternalDropToken, FileDialogToken, TimerToken};
pub use fret_i18n;
pub use input::{
    DefaultAction, DefaultActionSet, InputContext, InputDispatchPhase, KeyChord, Platform,
    TextBoundaryMode, format_chord, format_sequence,
};
pub use interaction_diagnostics::{
    DockDragDiagnostics, DockDropCandidateRectDiagnostics, DockDropCandidateRectKind,
    DockDropPreviewDiagnostics, DockDropPreviewKindDiagnostics, DockDropResolveDiagnostics,
    DockDropResolveSource, DockDropTargetDiagnostics, DockGraphStatsDiagnostics,
    DockingInteractionDiagnostics, ViewportCaptureDiagnostics, WindowInteractionDiagnosticsStore,
};
pub use keymap::{BindingV1, KeySpecV1, KeymapError, KeymapFileV1};
pub use keymap::{DefaultKeybinding, Keymap, KeymapContinuation, KeymapService, PlatformFilter};
pub use menu::{
    ItemAnchor, ItemSelector, ItemSelectorTyped, Menu, MenuBar, MenuBarConfig, MenuBarError,
    MenuBarFileV1, MenuBarFileV2, MenuBarPatch, MenuBarPatchOp, MenuFileV1, MenuFileV2, MenuItem,
    MenuItemFileV1, MenuItemFileV2, MenuItemToggle, MenuItemToggleKind, MenuRole, MenuTarget,
    SystemMenuType,
};
pub use model::{
    Model, ModelChangedDebugInfo, ModelCreatedDebugInfo, ModelCx, ModelHost, ModelId, ModelStore,
    ModelUpdateError, WeakModel,
};
pub use platform_completion::PlatformCompletion;
pub use platform_text_input::{PlatformTextInputQuery, PlatformTextInputQueryResult, Utf16Range};
pub use shortcut_routing_diagnostics::{
    ShortcutRoutingDecision, ShortcutRoutingOutcome, ShortcutRoutingPhase,
    WindowShortcutRoutingDiagnosticsStore,
};
pub use text_interaction_settings::TextInteractionSettings;
pub use time::TickId;
pub use ui_host::{CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost, UiHost};
pub use when_expr::WhenExpr;
pub use window_command_action_availability::WindowCommandActionAvailabilityService;
pub use window_command_availability::{
    WindowCommandAvailability, WindowCommandAvailabilityService,
};
pub use window_command_enabled::WindowCommandEnabledService;
pub use window_command_gating::{
    WindowCommandGatingHandle, WindowCommandGatingService, WindowCommandGatingSnapshot,
    best_effort_snapshot_for_window, best_effort_snapshot_for_window_with_input_ctx_fallback,
    command_is_enabled_for_window_with_input_ctx_fallback, snapshot_for_window,
    snapshot_for_window_with_input_ctx_fallback,
};
pub use window_input_arbitration::{WindowInputArbitrationSnapshot, WindowPointerOcclusion};
pub use window_input_context::WindowInputContextService;
pub use window_menu_bar_focus::WindowMenuBarFocusService;
pub use window_metrics::apply_window_metrics_event;
pub use window_text_boundary_mode::{WindowTextBoundaryModeHandle, WindowTextBoundaryModeService};
pub use window_text_input_snapshot::{WindowTextInputSnapshot, WindowTextInputSnapshotService};
