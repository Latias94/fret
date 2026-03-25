//! Portable runtime contracts and value types shared across the Fret workspace.
//!
//! This crate intentionally avoids backend bindings (`winit`, `wgpu`, `web-sys`) and must not
//! force a global async runtime (Tokio, etc.).
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-runtime/README.md`.
//!
//! ## Where to start
//!
//! - Commands: [`CommandId`], [`CommandRegistry`], [`CommandMeta`]
//! - Models/state: [`Model`], [`ModelStore`], [`ModelCx`]
//! - Effects: [`Effect`]
//! - Host integration: [`UiHost`], [`GlobalsHost`], [`ModelsHost`]
//! - Portability contracts: [`PlatformCapabilities`]
//!
//! ## Minimal example
//!
//! ```
//! use fret_runtime::{CommandId, CommandMeta, CommandRegistry};
//!
//! let mut commands = CommandRegistry::default();
//! commands.register(CommandId::from("app.quit"), CommandMeta::new("Quit"));
//! ```

pub mod action;
pub mod action_payload;
mod asset_reload;
pub mod asset_resolver;
pub mod capabilities;
pub mod clipboard_diagnostics;
pub mod command;
pub mod command_dispatch_diagnostics;
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
pub mod redraw_request_diagnostics;
pub mod runner_accessibility_diagnostics;
pub mod runner_frame_drive_diagnostics;
pub mod runner_platform_window_receiver_diagnostics;
pub mod runner_present_diagnostics;
pub mod runner_surface_config_diagnostics;
pub mod runner_surface_lifecycle_diagnostics;
pub mod runner_window_lifecycle_diagnostics;
pub mod runner_window_style_diagnostics;
pub mod shortcut_routing_diagnostics;
pub mod strict_runtime;
pub mod text_interaction_settings;
pub mod time;
pub mod ui_host;
pub mod when_expr;
pub mod window_chrome;
pub mod window_command_action_availability;
pub mod window_command_availability;
pub mod window_command_enabled;
pub mod window_command_gating;
pub mod window_global_change_diagnostics;
pub mod window_input_arbitration;
pub mod window_input_context;
pub mod window_key_context_stack;
pub mod window_menu_bar_focus;
pub mod window_metrics;
pub mod window_style;
pub mod window_text_boundary_mode;
pub mod window_text_input_snapshot;

// -----------------------------------------------------------------------------
// Stable re-exports (portable runtime contract surface)
// -----------------------------------------------------------------------------
pub use action::{ActionId, ActionMeta, ActionRegistry, TypedAction};
pub use action_payload::WindowPendingActionPayloadService;
pub use asset_reload::{
    AssetReloadBackendKind, AssetReloadEpoch, AssetReloadFallbackReason, AssetReloadStatus,
    AssetReloadSupport, asset_reload_epoch, asset_reload_status, asset_reload_support,
    bump_asset_reload_epoch, set_asset_reload_status, set_asset_reload_support,
};
pub use asset_resolver::{
    AssetLoadAccessKind, AssetLoadDiagnosticEvent, AssetLoadDiagnosticsSnapshot,
    AssetLoadOutcomeKind, AssetResolverService, AssetRevisionTransitionKind, asset_capabilities,
    asset_resolver, register_asset_resolver, register_bundle_asset_entries,
    register_embedded_asset_entries, resolve_asset_bytes, resolve_asset_locator_bytes,
    resolve_asset_locator_reference, resolve_asset_reference, set_asset_resolver,
};
pub use capabilities::{
    ExecBackgroundWork, ExecCapabilities, ExecTimers, ExecWake, ExternalDragPayloadKind,
    ExternalDragPositionQuality, PlatformCapabilities, ShellCapabilities,
    WindowHoverDetectionQuality, WindowSetOuterPositionQuality, WindowZLevelQuality,
};
pub use clipboard_diagnostics::{
    ClipboardReadDiagnostics, ClipboardWriteDiagnostics, WindowClipboardDiagnosticsStore,
};
pub use command::CommandId;
pub use command_dispatch_diagnostics::{
    CommandDispatchDecisionV1, CommandDispatchSourceKindV1, CommandDispatchSourceV1,
    WindowCommandDispatchDiagnosticsStore, WindowPendingCommandDispatchSourceService,
};
pub use commands::{CommandMeta, CommandRegistry, CommandScope, OsAction};
pub use docking_settings::{
    DockDragInversionModifier, DockDragInversionPolicy, DockDragInversionSettings,
    DockingInteractionSettings,
};
pub use drag::{
    DRAG_KIND_DOCK_PANEL, DRAG_KIND_DOCK_TABS, DragKindId, DragPhase, DragSession, DragSessionId,
    WindowUnderCursorSource,
};
pub use effect::DiagIncomingOpenItem;
pub use effect::{CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
pub use execution::{
    DispatchPriority, Dispatcher, DispatcherHandle, InboxDrain, InboxDrainHost, InboxDrainRegistry,
    Runnable,
};
pub use font_bootstrap::{
    FontCatalogUpdate, FontFamilyDefaultsPolicy, apply_font_catalog_update,
    apply_font_catalog_update_with_metadata,
};
pub use font_catalog::{
    BundledFontBaselineSnapshot, BundledFontBaselineSource, FontCatalog, FontCatalogEntry,
    FontCatalogMetadata, FontVariableAxisInfo, SystemFontRescanState, TextFontStackKey,
};
pub use font_catalog_cache::FontCatalogCache;
pub use fret_core::FrameId;
pub use fret_core::ImageUpdateToken;
pub use fret_core::ImageUploadToken;
pub use fret_core::{
    AlphaMode, ChromaSiting, ClipboardAccessError, ClipboardAccessErrorKind, ClipboardWriteOutcome,
    ColorPrimaries, ColorRange, ImageColorInfo, ImageColorSpace, ImageEncoding, TransferFunction,
    YuvMatrix,
};
pub use fret_core::{
    ClipboardToken, ExternalDropToken, FileDialogToken, IncomingOpenToken, ShareSheetToken,
    TimerToken,
};
pub use fret_core::{IncomingOpenDataEvent, IncomingOpenItem, ShareItem, ShareSheetOutcome};
pub use fret_i18n;
pub use input::{
    DefaultAction, DefaultActionSet, InputContext, InputDispatchPhase, KeyChord, Platform,
    TextBoundaryMode, format_chord, format_sequence,
};
pub use interaction_diagnostics::{
    DockDragDiagnostics, DockDropCandidateRectDiagnostics, DockDropCandidateRectKind,
    DockDropPreviewDiagnostics, DockDropPreviewKindDiagnostics, DockDropResolveDiagnostics,
    DockDropResolveSource, DockDropTargetDiagnostics, DockFloatingDragDiagnostics,
    DockGraphSignatureDiagnostics, DockGraphStatsDiagnostics,
    DockTabStripActiveVisibilityDiagnostics, DockTabStripActiveVisibilityStatusDiagnostics,
    DockingInteractionDiagnostics, ViewportCaptureDiagnostics, WindowInteractionDiagnosticsStore,
    WorkspaceInteractionDiagnostics, WorkspaceTabStripActiveVisibilityDiagnostics,
    WorkspaceTabStripActiveVisibilityStatusDiagnostics, WorkspaceTabStripDragDiagnostics,
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
pub use redraw_request_diagnostics::{
    RedrawRequestCallsiteCount, WindowRedrawRequestAggregateSnapshot,
    WindowRedrawRequestDiagnosticsStore, WindowRedrawRequestWindowSnapshot,
};
pub use runner_accessibility_diagnostics::{
    RunnerAccessibilityDiagnosticsStore, RunnerAccessibilitySnapshot,
};
pub use runner_frame_drive_diagnostics::{
    RunnerFrameDriveAggregateSnapshot, RunnerFrameDriveDiagnosticsStore, RunnerFrameDriveReason,
    RunnerFrameDriveReasonCount, RunnerFrameDriveWindowSnapshot,
};
pub use runner_platform_window_receiver_diagnostics::{
    RunnerPlatformWindowReceiverAtCursorSnapshotV1, RunnerPlatformWindowReceiverAtCursorSourceV1,
    RunnerPlatformWindowReceiverDiagnosticsStore,
};
pub use runner_present_diagnostics::{
    RunnerPresentAggregateSnapshot, RunnerPresentDiagnosticsStore, RunnerPresentWindowSnapshot,
};
pub use runner_surface_config_diagnostics::{
    RunnerSurfaceConfigDiagnosticsStore, RunnerSurfaceConfigWindowSnapshot,
};
pub use runner_surface_lifecycle_diagnostics::{
    RunnerSurfaceLifecycleDiagnosticsStore, RunnerSurfaceLifecycleSnapshot,
};
pub use runner_window_lifecycle_diagnostics::{
    RunnerWindowLifecycleDiagnosticsStore, RunnerWindowLifecycleSnapshot,
};
pub use runner_window_style_diagnostics::{
    RunnerWindowAppearanceV1, RunnerWindowCompositedAlphaSourceV1,
    RunnerWindowHitTestClampReasonV1, RunnerWindowHitTestSourceV1,
    RunnerWindowStyleDiagnosticsStore, RunnerWindowStyleEffectiveSnapshotV1,
    clamp_background_material_request,
};
pub use shortcut_routing_diagnostics::{
    ShortcutRoutingDecision, ShortcutRoutingOutcome, ShortcutRoutingPhase,
    WindowShortcutRoutingDiagnosticsStore,
};
pub use text_interaction_settings::TextInteractionSettings;
pub use time::TickId;
pub use ui_host::{CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost, UiHost};
pub use when_expr::WhenExpr;
pub use window_chrome::WindowResizeDirection;
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
pub use window_global_change_diagnostics::{
    WindowGlobalChangeAggregateSnapshot, WindowGlobalChangeDiagnosticsStore,
    WindowGlobalChangeNameCount, WindowGlobalChangeWindowSnapshot,
};
pub use window_input_arbitration::{WindowInputArbitrationSnapshot, WindowPointerOcclusion};
pub use window_input_context::WindowInputContextService;
pub use window_key_context_stack::WindowKeyContextStackService;
pub use window_menu_bar_focus::WindowMenuBarFocusService;
pub use window_metrics::apply_window_metrics_event;
pub use window_style::{
    ActivationPolicy, TaskbarVisibility, WindowBackgroundMaterialRequest, WindowDecorationsRequest,
    WindowHitTestRegionV1, WindowHitTestRegionsSignatureV1, WindowHitTestRequestV1, WindowOpacity,
    WindowRole, WindowStyleRequest, WindowZLevel, canonicalize_hit_test_regions_v1,
    hit_test_regions_signature_v1,
};
pub use window_text_boundary_mode::{WindowTextBoundaryModeHandle, WindowTextBoundaryModeService};
pub use window_text_input_snapshot::{
    WindowImeSurroundingText, WindowTextInputSnapshot, WindowTextInputSnapshotService,
};
