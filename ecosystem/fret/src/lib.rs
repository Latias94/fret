//! Batteries-included desktop-first entry points for Fret.
//!
//! This crate is intentionally **ecosystem-level**:
//! - it exposes a backend-free app-authoring surface under `app`,
//! - it composes `fret-bootstrap` (golden-path wiring) with a default component surface under
//!   `desktop`,
//! - it enables a practical desktop-first default stack by default,
//! - it remains optional: advanced users can depend on `fret-framework` + `fret-bootstrap` directly.
//! - it is **not** the repository?s canonical example host; runnable lessons stay in app-owned
//!   surfaces such as `apps/fret-cookbook`, `apps/fret-ui-gallery`, and other app shells.
//!
//! ## Getting started (desktop)
//!
//! ```no_run
//! use fret::app::prelude::*;
//!
//! struct HelloView;
//!
//! impl View for HelloView {
//!     fn init(_app: &mut App, _window: WindowId) -> Self {
//!         Self
//!     }
//!
//!     fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
//!         ui::single(cx, shadcn::Label::new("Fret!"))
//!     }
//! }
//!
//! fn main() -> fret::Result<()> {
//!     FretApp::new("hello")
//!         .window("Hello", (560.0, 360.0))
//!         .view::<HelloView>()?
//!         .run()
//! }
//! ```
//!
//! For user-facing demos, add `.window_min_size((...))` when the layout should stay above a
//! readable breakpoint during manual resize.
//! Use `.window_position_logical((...))` / `.window_resize_increments((...))` when startup
//! placement or stepwise resizing is part of the product surface.
//! For multi-window apps that rely on fallback-created auxiliary windows, configure
//! `.with_default_window(...)` and related `with_default_window_*` methods on `UiAppBuilder`.
//!
//! ## Choosing a native entry path
//!
//! `FretApp::new(...)` is available in the backend-free `app` profile as an app-authoring spec.
//! Native window creation, `view::<V>()?`, `UiAppBuilder`, and `.run()` are `desktop` surfaces.
//!
//! - Default app path: `fret::FretApp::new(...).window(...).view::<V>()?`.
//! - App-author hooks: `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?` when a
//!   window needs lifecycle hooks but should still read as a `View` app.
//! - Advanced/manual assembly: `fret::advanced::ui_app(...)`,
//!   `fret::advanced::ui_app_with_hooks(...)`,
//!   `fret::advanced::view::render_root_with_app_ui(...)`,
//!   `fret::advanced::run_native_with_fn_driver(...)`,
//!   `fret::advanced::run_native_with_fn_driver_with_hooks(...)`, and
//!   `fret::advanced::run_native_with_configured_fn_driver(...)`.
//! - Retained-driver interop: `fret::advanced::interop::run_native_with_driver(...)` for advanced
//!   bridge integrations that implement `fret_launch::WinitAppDriver` directly.
//! - Low-level runtime/rendering seams stay explicit under `fret::advanced::kernel::*` and
//!   `fret::advanced::interop::*`.
//!
//! Optional ecosystem extensions stay explicit:
//!
//! - enable `state` for grouped selector/query helpers on `AppUi`; prefer
//!   `cx.data().selector_layout(...)` for LocalState-first derived values, keep
//!   `cx.data().query*(...)` plus `handle.read_layout(cx)` as the default query read path, and use
//!   `cx.data().invalidate_query(...)` / `cx.data().invalidate_query_namespace(...)` when
//!   app-facing query invalidation stays inside `AppUi`; use `cx.data().cancel_query(...)` plus
//!   `cx.data().query_snapshot()` / `cx.data().query_snapshot_entry(...)` when app-facing status
//!   chrome needs explicit query maintenance/diagnostics without reopening raw client shell code;
//!   when app code needs explicit state helper nouns, use `fret::selector::ui::DepsBuilder`,
//!   `fret::selector::DepsSignature`, and
//!   `fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}` instead of expecting
//!   those names from `fret::app::prelude::*`
//! - enable `state-mutation` when app code needs an explicit submit/mutation lane that must not
//!   auto-run from render observation; prefer `cx.data().mutation_async(...)` /
//!   `cx.data().mutation_async_local(...)` plus `handle.submit(...)`, `handle.submit_action(...)`,
//!   or explicit `handle.retry_last(...)` replay instead of teaching click-driven submit flows
//!   through `query_async(...)`
//! - enable `router` for
//!   `fret::router::{app::install, RouterUiStore, router_link, router_outlet_by_leaf_with_test_id, ...}`
//!   plus `fret::router::bind_history_actions(...)` history bindings
//! - depend on `fret-docking` directly for editor-grade docking workflows instead of expecting a
//!   `fret` root feature proxy
//! - enable `imui` for `fret::imui::{prelude::*, kit, editor, docking}` when the app wants an
//!   explicit imgui-style authoring lane; keep those helpers off `fret::app::prelude::*` so the
//!   default app story stays declarative-first
//! - use `fret::assets::{AssetBundleId, AssetLocator, AssetRequest, StaticAssetEntry, ...}`
//!   for logical bundle/embedded assets; prefer `AssetBundleId::app(...)` /
//!   `AssetBundleId::package(...)` over raw global strings; `AssetStartupPlan` +
//!   `AssetStartupMode` are backend-free authoring values, `FretApp::asset_startup(...)` records
//!   them on the app spec, and desktop-only `UiAppBuilder::with_asset_startup(...)` applies them to
//!   a concrete runner builder; when host/bootstrap code intentionally installs
//!   file-backed resolver layers directly, construct
//!   `FileAssetManifestResolver::from_bundle_dir(...)` /
//!   `FileAssetManifestResolver::from_manifest_path(...)` and register the result with
//!   `register_resolver(...)` instead of teaching path-first helpers to widget code, and treat
//!   `AssetLocator::file(...)` / `AssetLocator::url(...)` as capability-gated escape hatches;
//!   when native/dev-only UI helpers still need file reload ergonomics, keep app/widget code on
//!   logical bundle locators and let
//!   `fret::app::ui_assets::image_source_state_from_asset_request(cx, ...)`
//!   or `fret::app::ui_assets::svg_source_state_from_asset_request(cx, ...)`
//!   consume the resolver's bundle/reference bridge instead of constructing raw file-path sources
//!   directly; keep `resolve_image_source_from_host_locator(...)` /
//!   `resolve_svg_source_from_host_locator(...)` as the lower-level UI-ready source seams, and use
//!   `fret::assets::resolve_reference(...)` / `resolve_locator_reference(...)` when a non-UI
//!   integration truly needs the raw external reference
//! - use `fret::shadcn::{Button, Card, ...}` for the curated default design-system surface;
//!   `shadcn::app::install(...)` and `shadcn::themes::apply_shadcn_new_york(...)` are setup lanes
//!   rather than peer discovery lanes; only drop to `fret::shadcn::raw::*` when you intentionally
//!   need the full uncurated recipe surface, and keep advanced environment / `UiServices` hooks on
//!   `fret::shadcn::raw::advanced::*`
//! - use `fret::integration::InstallIntoApp` for reusable app-install bundles; small app-local
//!   composition can also use `.setup((install_a, install_b))` while ordinary app code keeps
//!   passing named installer functions to `.setup(...)` and keeps inline one-off closures or
//!   runtime-captured config on `UiAppBuilder::setup_with(...)`
//!
//! ## Immediate-mode lane (optional)
//!
//! `fret::imui` is the explicit imgui-style lane. Keep `fret::app::prelude::*` as the default
//! declarative-first story, and opt into exact `fret::imui` imports when a view wants
//! immediate-mode control flow. `use fret::imui::prelude::*;` remains available for prototypes.
//!
//! ```ignore
//! use fret::app::prelude::*;
//! use fret::imui::{UiWriter as _, imui_in};
//!
//! struct InspectorView;
//!
//! impl View for InspectorView {
//!     fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
//!         imui_in(cx, |ui| {
//!             if ui.button("Save").clicked() {
//!                 // Trigger an app-facing action or mutate shared state here.
//!             }
//!         })
//!     }
//! }
//! ```
//!
//! Reach for `fret::imui::kit` for policy-heavy widgets, `fret::imui::editor` for editor-grade
//! controls, and `fret::imui::docking` for docking helpers.
/// Canonical app-facing window identity alias for the default authoring surface.
pub type WindowId = fret_core::AppWindowId;

/// Re-export portable action/command identity types for app code and macros.
pub use fret_runtime::{ActionId, CommandId, TypedAction};
/// Re-export the curated default shadcn/ui surface as `shadcn`.
#[cfg(feature = "shadcn")]
pub use fret_ui_shadcn::facade as shadcn;

/// Explicit icon helpers and identifiers for app and component code that opt into icon-specific
/// authoring.
pub mod icons {
    pub use fret_icons::IconId;
    #[cfg(feature = "icons")]
    pub use fret_ui_kit::declarative::icon;
}

/// Explicit accessibility/semantics nouns for app code that needs semantic-role overrides.
pub mod semantics {
    pub use fret_core::SemanticsRole;
    pub use fret_ui::element::SemanticsDecoration;
}

/// Cross-platform time primitives for app code.
pub mod time {
    pub use fret_core::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
}

/// Explicit style/token nouns for app code that customizes layout or chrome beyond the default lane.
pub mod style {
    pub use fret_core::scene::DashPatternV1;
    pub use fret_core::{
        AttributedText, Axis, Color, Corners, DecorationLineStyle, Edges, FontWeight, Px,
        StrikethroughStyle, TextAlign, TextOverflow, TextPaintStyle, TextSpan, TextWrap,
    };
    pub use fret_ui::element::{
        ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow, SizeStyle,
        SpacingLength,
    };
    pub use fret_ui::{Theme, ThemeSnapshot};
    pub use fret_ui_kit::{
        ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, ShadowPreset, Size, Space,
    };
}

/// Explicit scroll state handles for app code that coordinates scrollable surfaces.
pub mod scroll {
    pub use fret_ui::scroll::ScrollHandle;
}

/// Explicit virtual-list mechanism vocabulary for app code that opts into low-level virtualization.
pub mod virtual_list {
    pub use fret_ui::element::{VirtualListKeyCacheMode, VirtualListOptions};
    pub use fret_ui::scroll::VirtualListScrollHandle;
    pub use fret_ui::{ItemKey, ScrollStrategy};
}

/// Explicit environment and responsive helpers/configuration nouns for app or component code that
/// opts into adaptive UI logic.
pub mod env {
    pub use fret_ui_kit::declarative::{
        ContainerQueryHysteresis, ViewportOrientation, ViewportQueryHysteresis, accent_color,
        container_breakpoints, container_query_region, container_query_region_with_id,
        container_width_at_least, contrast_preference, forced_colors_active, forced_colors_mode,
        occlusion_insets, occlusion_insets_or_zero, preferred_color_scheme,
        prefers_dark_color_scheme, prefers_more_contrast, prefers_reduced_motion,
        prefers_reduced_transparency, primary_pointer_can_hover, primary_pointer_is_coarse,
        primary_pointer_type, safe_area_insets, safe_area_insets_or_zero, tailwind,
        text_scale_factor, viewport_aspect_ratio, viewport_breakpoints, viewport_height_at_least,
        viewport_height_breakpoints, viewport_is_landscape, viewport_is_portrait,
        viewport_orientation, viewport_tailwind, viewport_width_at_least,
        window_insets_padding_refinement_or_zero,
    };
}

/// Explicit pointer-region authoring helpers for app code that needs low-level pointer streams.
///
/// Keep this lane explicit instead of adding pointer nouns to `fret::app::prelude::*`: ordinary
/// apps should discover buttons, inputs, toggles, and recipes first; custom interaction surfaces
/// can opt into `fret::pointer`.
pub mod pointer {
    #[doc(hidden)]
    pub use crate::view::AppPointerRegion;
    pub use crate::view::{
        CursorIcon, MouseButton, Point, PointerActionCx, PointerCancel, PointerDown, PointerId,
        PointerMove, PointerRegion, PointerUp, Wheel,
    };
}

/// Explicit async/background-work helpers for app code.
///
/// Keep concrete executor choices out of the default `fret` facade. This lane exposes the runner
/// dispatcher, inbox drainer registration, and LocalState-safe drain callbacks while crates such
/// as `fret-executor` remain optional app/ecosystem choices.
pub mod async_work {
    pub use crate::view::{
        AppAsyncWorkExt, AppInboxCx, InboxLocal, inbox_drain_apply, inbox_local,
    };
    pub use fret_runtime::{DispatchPriority, DispatcherHandle};
}

/// Explicit canvas authoring helpers for app code.
///
/// This lane hides raw `CanvasPainter`, raw pointer action host callbacks, and raw `Model<T>`
/// plumbing while still leaving custom canvas drawing and pan/zoom state explicit.
#[cfg(feature = "canvas")]
pub mod canvas {
    pub use crate::view::{AppCanvasPainter, Canvas, CanvasSurface, PanZoomCanvas};
    pub use fret_canvas::scale::constant_pixel_stroke_width;
    pub use fret_canvas::ui::{PanZoomCanvasPaintCx, PanZoomInputPreset};
    pub use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
    pub use fret_core::scene::{Color, Paint as CanvasPaint, PaintBindingV1};
    pub use fret_core::{
        Corners, DrawOrder, Edges, FillStyle, PathCommand, PathMetrics, PathStyle, Point, Px, Rect,
        SceneOp, Size, StrokeCapV1, StrokeJoinV1, StrokeStyle, StrokeStyleV2, Transform2D,
    };
    pub use fret_ui::canvas::CanvasKey;
}

/// Explicit chart authoring helpers for app code.
///
/// This lane hides raw `ChartCanvasPanelProps`, raw chart output `Model<T>` handles, and raw
/// `ViewCacheProps` while keeping the headless `delinea` chart domain explicit.
#[cfg(feature = "chart")]
pub mod chart {
    pub use crate::view::ChartCanvas;
    pub use delinea;
    pub use delinea::engine::ChartEngine;
    pub use delinea::engine::window::DataWindow;
    pub use fret_chart::{ChartCanvasOutput, ChartInputMap};
}

/// Explicit higher-level adaptive policy vocabulary for app code that wants shared classification
/// or device-shell strategy helpers above raw query reads.
pub mod adaptive {
    pub use fret_ui_kit::adaptive::{
        AdaptiveQuerySource, DeviceAdaptiveClass, DeviceAdaptivePolicy, DeviceAdaptiveSnapshot,
        DeviceShellMode, DeviceShellSwitchPolicy, PanelAdaptiveClass, PanelAdaptivePolicy,
        device_adaptive_class, device_adaptive_snapshot, device_shell_mode, device_shell_switch,
        panel_adaptive_class,
    };
}

/// Explicit child-collection helpers for app code that opts into manual sink-style composition.
pub mod children {
    pub use fret_ui_kit::ui::UiElementSinkExt;
}

/// Explicit activation-helper glue for component or advanced code that intentionally authors raw
/// `on_activate(...)` handlers.
pub mod activate {
    pub use fret_ui_kit::{
        on_activate, on_activate_notify, on_activate_request_redraw,
        on_activate_request_redraw_notify,
    };
}

/// Explicit overlay composition and introspection vocabulary for reusable component code.
///
/// The component prelude keeps only the highest-frequency overlay builder nouns. Lower-level
/// overlay stack snapshots and anchoring helpers stay on this explicit lane so reusable component
/// authors do not meet them via first-contact wildcard imports.
pub mod overlay {
    pub use fret_ui_kit::overlay::*;
    pub use fret_ui_kit::{
        OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence,
        OverlayRequest, OverlayStackEntryKind, WindowOverlayStackEntry, WindowOverlayStackSnapshot,
    };
}

/// Optional immediate-mode authoring lane for apps that want imgui-style control flow without
/// widening the default `fret::app::prelude::*` surface.
#[cfg(feature = "imui")]
pub mod imui {
    pub use fret_imui::{
        ImUi, Response, imui, imui_build, imui_build_in, imui_in, imui_raw, imui_raw_in,
        prelude::UiWriter,
    };
    pub use fret_ui_kit::imui::{
        ImUiFacade, IntoImUiBoolModel, IntoImUiFloatModel, IntoImUiOptionalTextModel,
        IntoImUiTextModel, ResponseExt, UiWriterImUiFacadeExt, UiWriterUiKitExt,
    };

    /// App-facing IMUI text helpers for view-local state.
    ///
    /// These keep cookbook/default IMUI examples on `LocalState<String>` without exposing the raw
    /// `Model<String>` bridge at the call site.
    pub trait AppImUiLocalTextExt<H: fret_ui::UiHost>: UiWriterImUiFacadeExt<H> {
        fn input_text_local(&mut self, local: &crate::view::LocalState<String>) -> ResponseExt {
            self.input_text_local_with_options(
                local,
                fret_ui_kit::imui::InputTextOptions::default(),
            )
        }

        fn input_text_local_with_options(
            &mut self,
            local: &crate::view::LocalState<String>,
            options: fret_ui_kit::imui::InputTextOptions,
        ) -> ResponseExt {
            <Self as UiWriterImUiFacadeExt<H>>::input_text_model_with_options(
                self,
                crate::view::LocalStateRawModelExt::model(local),
                options,
            )
        }
    }

    impl<H, W> AppImUiLocalTextExt<H> for W
    where
        H: fret_ui::UiHost,
        W: UiWriterImUiFacadeExt<H> + ?Sized,
    {
    }

    /// Policy-heavy immediate-mode widgets, responses, and option types from `fret-ui-kit`.
    pub mod kit {
        pub use fret_ui_kit::imui::*;
    }

    /// Editor-grade controls and composites available on the immediate-mode lane.
    pub mod editor {
        pub use fret_ui_editor::imui::*;
        pub use fret_ui_editor::{composites, primitives, theme};

        /// Editor controls plus app-facing `LocalState` constructors.
        pub mod controls {
            use std::sync::Arc;

            pub use fret_ui_editor::controls::*;

            use crate::view::{LocalState, LocalStateRawModelExt as _};
            use fret_ui_editor::primitives::DragValueScalar;

            pub trait NumericInputLocalStateExt<T>
            where
                T: Copy + Default + 'static,
            {
                fn new_local(
                    local: &LocalState<T>,
                    format: NumericFormatFn<T>,
                    parse: NumericParseFn<T>,
                ) -> Self;

                fn from_local_presentation(
                    local: &LocalState<T>,
                    presentation: fret_ui_editor::primitives::NumericPresentation<T>,
                ) -> Self;
            }

            impl<T> NumericInputLocalStateExt<T> for NumericInput<T>
            where
                T: Copy + Default + 'static,
            {
                fn new_local(
                    local: &LocalState<T>,
                    format: NumericFormatFn<T>,
                    parse: NumericParseFn<T>,
                ) -> Self {
                    Self::new(local.clone_model(), format, parse)
                }

                fn from_local_presentation(
                    local: &LocalState<T>,
                    presentation: fret_ui_editor::primitives::NumericPresentation<T>,
                ) -> Self {
                    Self::from_presentation(local.clone_model(), presentation)
                }
            }

            pub trait DragValueLocalStateExt<T>
            where
                T: DragValueScalar + Default,
            {
                fn new_local(
                    local: &LocalState<T>,
                    format: NumericFormatFn<T>,
                    parse: NumericParseFn<T>,
                ) -> Self;

                fn from_local_presentation(
                    local: &LocalState<T>,
                    presentation: fret_ui_editor::primitives::NumericPresentation<T>,
                ) -> Self;
            }

            impl<T> DragValueLocalStateExt<T> for DragValue<T>
            where
                T: DragValueScalar + Default,
            {
                fn new_local(
                    local: &LocalState<T>,
                    format: NumericFormatFn<T>,
                    parse: NumericParseFn<T>,
                ) -> Self {
                    Self::new(local.clone_model(), format, parse)
                }

                fn from_local_presentation(
                    local: &LocalState<T>,
                    presentation: fret_ui_editor::primitives::NumericPresentation<T>,
                ) -> Self {
                    Self::from_presentation(local.clone_model(), presentation)
                }
            }

            pub trait ColorEditLocalStateExt {
                fn new_local(local: &LocalState<fret_core::Color>) -> Self;
            }

            impl ColorEditLocalStateExt for ColorEdit {
                fn new_local(local: &LocalState<fret_core::Color>) -> Self {
                    Self::new(local.clone_model())
                }
            }

            pub trait MiniSearchBoxLocalStateExt {
                fn new_local(local: &LocalState<String>) -> Self;
            }

            impl MiniSearchBoxLocalStateExt for MiniSearchBox {
                fn new_local(local: &LocalState<String>) -> Self {
                    Self::new(local.clone_model())
                }
            }

            pub trait TextAssistFieldLocalStateExt {
                fn new_local(
                    query: &LocalState<String>,
                    dismissed_query: &LocalState<String>,
                    active_item_id: &LocalState<Option<Arc<str>>>,
                    items: Arc<[TextAssistItem]>,
                ) -> Self;
            }

            impl TextAssistFieldLocalStateExt for TextAssistField {
                fn new_local(
                    query: &LocalState<String>,
                    dismissed_query: &LocalState<String>,
                    active_item_id: &LocalState<Option<Arc<str>>>,
                    items: Arc<[TextAssistItem]>,
                ) -> Self {
                    Self::new(
                        query.clone_model(),
                        dismissed_query.clone_model(),
                        active_item_id.clone_model(),
                        items,
                    )
                }
            }
        }
    }

    /// Docking helpers for immediate-mode authoring.
    pub mod docking {
        pub use fret_docking::imui::*;
        pub use fret_docking::{DockHostOptions, DockPanel, DockPanelElementRegistry, DockSurface};
        pub use fret_docking::{DockViewportLayout, DockViewportOverlayHooks, DockingPolicy};
        pub use fret_docking::{ViewportPanel, advanced};
    }

    /// Common imports for immediate-mode authoring on the explicit `fret::imui` lane.
    pub mod prelude {
        pub use crate::imui::{
            AppImUiLocalTextExt as _, ImUi, ImUiFacade, IntoImUiBoolModel, IntoImUiFloatModel,
            IntoImUiOptionalTextModel, IntoImUiTextModel, Response, ResponseExt,
            UiWriterImUiFacadeExt, UiWriterUiKitExt, docking, editor, imui, imui_build,
            imui_build_in, imui_in, imui_raw, imui_raw_in, kit,
        };
        pub use fret_imui::prelude::*;
    }
}

/// Explicit logical asset-contract vocabulary and host registration helpers for app code.
///
/// The portable default story is bundle/embedded locators. Prefer `AssetBundleId::app(...)` and
/// `AssetBundleId::package(...)` over ad-hoc global strings. Native/package-dev builds can also
/// mount scanned bundle directories or explicit file-backed manifests without leaking raw paths
/// into widget code. Raw files and URLs stay explicit, capability-gated escape hatches.
pub mod assets {
    #[cfg(not(target_arch = "wasm32"))]
    pub use fret_assets::FileAssetManifestResolver;
    pub use fret_assets::{
        AssetBundleId, AssetBundleNamespace, AssetCapabilities, AssetExternalReference, AssetKey,
        AssetKindHint, AssetLoadError, AssetLocator, AssetLocatorKind, AssetManifestLoadError,
        AssetMediaType, AssetMemoryKey, AssetRequest, AssetResolver, AssetRevision,
        FILE_ASSET_MANIFEST_KIND_V1, FileAssetManifestBundleV1, FileAssetManifestEntryV1,
        FileAssetManifestV1, ResolvedAssetBytes, ResolvedAssetReference, StaticAssetEntry,
        UrlPassthroughAssetResolver, asset_app_bundle_id, asset_package_bundle_id,
    };
    #[cfg(any(feature = "app", feature = "desktop"))]
    pub use fret_bootstrap::{
        AssetReloadPolicy, AssetStartupMode, AssetStartupPlan, AssetStartupPlanError,
    };
    pub use fret_runtime::AssetResolverService;
    pub use fret_runtime::{
        AssetReloadBackendKind, AssetReloadEpoch, AssetReloadFallbackReason, AssetReloadStatus,
        AssetReloadSupport, asset_reload_epoch, asset_reload_status, asset_reload_support,
        bump_asset_reload_epoch,
    };

    /// Install or replace the primary resolver layer for the current host.
    ///
    /// The primary layer participates in the same ordered host resolver stack as every other
    /// registration. Replacing an existing primary layer keeps that layer's current stack
    /// position, so later registrations can still intentionally override it for the same logical
    /// locator.
    pub use fret_runtime::set_asset_resolver as set_primary_resolver;

    /// Add an additional resolver layer without replacing earlier registrations.
    ///
    /// Host resolver registrations preserve insertion order across primary, layered, and static
    /// entry registrations, so later registrations take precedence over earlier ones for the same
    /// logical locator.
    pub use fret_runtime::register_asset_resolver as register_resolver;

    /// Register static bundle-scoped entries on the current host.
    ///
    /// These entries participate in the same ordered host resolver stack as other registrations,
    /// so a later static registration can override an earlier resolver layer and vice versa.
    pub use fret_runtime::register_bundle_asset_entries as register_bundle_entries;

    /// Register static embedded entries owned by a specific bundle or crate.
    ///
    /// These entries participate in the same ordered host resolver stack as other registrations,
    /// so a later static registration can override an earlier resolver layer and vice versa.
    pub use fret_runtime::register_embedded_asset_entries as register_embedded_entries;

    /// Inspect the composed asset resolver service installed on the current host.
    pub use fret_runtime::asset_resolver as resolver;

    /// Report the current host's aggregated asset capabilities.
    pub use fret_runtime::asset_capabilities as capabilities;

    /// Resolve bytes for a logical asset request through the host-installed resolver chain.
    pub use fret_runtime::resolve_asset_bytes as resolve_bytes;

    /// Resolve bytes for a single locator through the host-installed resolver chain.
    pub use fret_runtime::resolve_asset_locator_bytes as resolve_locator;

    /// Resolve an external file/URL reference for a logical asset request through the
    /// host-installed resolver chain.
    pub use fret_runtime::resolve_asset_reference as resolve_reference;

    /// Resolve an external file/URL reference for a single locator through the host-installed
    /// resolver chain.
    pub use fret_runtime::resolve_asset_locator_reference as resolve_locator_reference;
}

#[derive(Debug, Clone)]
#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "desktop")),
    allow(dead_code)
)]
pub(crate) enum AssetMount {
    BundleEntries {
        bundle: fret_assets::AssetBundleId,
        entries: Vec<fret_assets::StaticAssetEntry>,
    },
    EmbeddedEntries {
        owner: fret_assets::AssetBundleId,
        entries: Vec<fret_assets::StaticAssetEntry>,
    },
    #[cfg(any(feature = "app", feature = "desktop"))]
    Startup {
        bundle: fret_assets::AssetBundleId,
        mode: fret_bootstrap::AssetStartupMode,
        plan: fret_bootstrap::AssetStartupPlan,
    },
    #[cfg(any(feature = "app", feature = "desktop"))]
    ReloadPolicy {
        policy: fret_bootstrap::AssetReloadPolicy,
    },
}

pub mod actions;
#[cfg(feature = "workspace")]
pub mod workspace;

/// Explicit command and keybinding vocabulary for app code.
///
/// Keep command registration, availability, key chords, and shortcut display on this named lane
/// instead of importing `fret_app`, `fret_runtime`, `fret_core`, or `fret_ui` from cookbook/default
/// app code.
pub mod commands {
    pub use fret_app::{
        CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, InputContext,
        KeyChord, KeymapService, Platform, PlatformFilter, format_sequence,
        install_command_default_keybindings_into_keymap,
    };
    pub use fret_core::{KeyCode, Modifiers};
    pub use fret_ui::CommandAvailability;
}

pub mod in_window_menubar;
mod view;

/// Explicit app-integration contracts for reusable ecosystem bundles.
pub mod integration;

mod app_entry;
pub use app_entry::FretApp;

/// Canonical app-facing UI context alias for the default authoring surface.
pub type AppUi<'cx, 'a, H = crate::app::App> = view::AppUi<'cx, 'a, H>;

/// Canonical app-facing render return alias for the default authoring surface.
pub type Ui = fret_ui::element::Elements;

/// Canonical app-facing single-element alias for extracted helper functions.
///
/// Prefer returning concrete `impl UiChild` when possible. Use this explicit alias when a helper
/// must erase heterogeneous element branches without importing raw `fret_ui::element::AnyElement`
/// into default app code.
pub type AppElement = fret_ui::element::AnyElement;

/// Canonical app-facing concrete render-context alias for extracted helper ergonomics.
///
/// Prefer `fret::app::AppRenderContext<'a>` for named helper signatures on the default lane.
/// Reach for `AppRenderCx<'a>` when closure-local or inline helper families materially benefit
/// from a concrete context carrier without reopening the raw `ElementContext<App>` spelling. Use
/// `AppComponentCx<'a>` instead when the helper is component-shaped but intentionally app-hosted.
pub type AppRenderCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;

/// Canonical component-facing context alias for reusable component authoring.
pub type ComponentCx<'a, H> = fret_ui::ElementContext<'a, H>;

/// Canonical app-hosted component/snippet context alias.
///
/// Use this for first-party examples, gallery snippets, and app-local components that deliberately
/// target the default `fret::app::App` host. Use `ComponentCx<'a, H>` when the component should
/// stay host-generic, and `AppRenderCx<'a>` / `AppRenderContext<'a>` for app render helpers.
pub type AppComponentCx<'a> = ComponentCx<'a, crate::app::App>;

/// App-facing child return alias for extracted helper functions on the default surface.
pub trait UiChild: fret_ui_kit::IntoUiElement<crate::app::App> {}

impl<T> UiChild for T where T: fret_ui_kit::IntoUiElement<crate::app::App> {}

/// Runtime defaults applied by the `fret` facade (within the enabled crate features).
///
/// This is an ecosystem-level convenience (not a kernel contract).
#[derive(Debug, Clone, Copy)]
pub struct Defaults {
    /// Enable default diagnostics wiring (tracing + panic hook).
    pub diagnostics: bool,
    /// Enable layered `.fret/*` config file loading (settings/keymap/menubar).
    pub config_files: bool,
    /// Install the default shadcn integration into the app.
    pub shadcn: bool,
    /// Install UI asset caches (images/SVG) with budgets.
    pub ui_assets: bool,
    /// Optional override budgets for UI assets.
    pub ui_assets_budgets: Option<(u64, usize, u64, usize)>,
    /// Install built-in icon packs (controlled by crate features).
    pub icons: bool,
    /// Preload icon SVGs on GPU ready (controlled by crate features).
    pub preload_icon_svgs: bool,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Debug, Clone, Copy)]
pub(crate) enum DesktopDefaultsStage {
    /// Base design-system defaults that app setup may intentionally refine.
    Base,
    /// Runtime defaults that need to observe app setup side effects such as command registration.
    Runtime,
    /// The full legacy default path for callers that do not need staged setup.
    All,
}

impl Defaults {
    /// Recommended desktop-first “batteries included” defaults.
    pub const fn desktop_batteries() -> Self {
        Self {
            diagnostics: true,
            config_files: true,
            shadcn: true,
            ui_assets: true,
            ui_assets_budgets: None,
            icons: true,
            preload_icon_svgs: true,
        }
    }

    /// Recommended desktop-first defaults for app authors.
    ///
    /// These defaults are intended to be smooth and practical without pulling in every optional
    /// integration. In particular, they avoid UI assets caches and GPU-time SVG preloading unless
    /// explicitly enabled.
    pub const fn desktop_app() -> Self {
        Self {
            diagnostics: true,
            config_files: false,
            shadcn: true,
            ui_assets: false,
            ui_assets_budgets: None,
            icons: false,
            preload_icon_svgs: false,
        }
    }

    /// Minimal defaults that avoid filesystem config loading and other batteries.
    pub const fn minimal() -> Self {
        Self {
            diagnostics: false,
            config_files: false,
            shadcn: false,
            ui_assets: false,
            ui_assets_budgets: None,
            icons: false,
            preload_icon_svgs: false,
        }
    }

    pub const fn with_ui_assets_budgets(
        mut self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        self.ui_assets = true;
        self.ui_assets_budgets = Some((
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        ));
        self
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self::desktop_app()
    }
}

/// Interop helpers for embedding foreign UI as isolated surfaces (desktop builds).
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod interop;

/// Re-export the kernel facade (desktop builds).
/// App-facing imports for ordinary Fret application code.
pub mod app;

/// Component-author imports for reusable, portable UI crates.
pub mod component;

/// Optional selector integration surface for app code.
///
/// This keeps the selector story explicit:
/// - grouped default app data stays on `cx.data().selector_layout(...)` for LocalState-first
///   inputs, with raw `cx.data().selector(...)` kept explicit,
/// - `fret-selector` remains the portable derived-state crate,
/// - `fret::selector` keeps selector-core nouns on the explicit lane, while the one app-facing UI
///   dependency builder stays under `fret::selector::ui::DepsBuilder` instead of widening
///   `fret::app::prelude::*`.
#[cfg(feature = "state-selector")]
pub mod selector {
    /// Raw selector-core exports for advanced or fully explicit use.
    pub mod core {
        pub use fret_selector::*;
    }

    /// Raw selector-UI adoption exports for advanced or fully explicit use.
    pub mod ui {
        pub use fret_selector::ui::DepsBuilder;
    }

    pub use fret_selector::{DepsSignature, Selector};
}

/// Optional query integration surface for app code.
///
/// This keeps the query story explicit:
/// - grouped default app data stays on `cx.data().query*` plus
///   `cx.data().invalidate_query*`,
/// - `fret-query` remains the portable async resource crate,
/// - `fret::query` gives app authors one curated import lane for `QueryKey` / `QueryPolicy` /
///   `QueryState`-style nouns without pulling those names into `fret::app::prelude::*`.
#[cfg(feature = "state-query")]
pub mod query {
    /// Raw query-core exports for advanced or fully explicit use.
    pub mod core {
        pub use fret_query::*;
    }

    pub use fret_query::{
        CancellationToken, FutureSpawner, FutureSpawnerHandle, QueryCancelMode, QueryClient,
        QueryClientSnapshot, QueryError, QueryErrorKind, QueryHandle, QueryKey, QueryPolicy,
        QueryRetryOn, QueryRetryPolicy, QueryRetryState, QuerySnapshotEntry, QueryState,
        QueryStatus, with_query_client,
    };
}

/// Optional mutation/submission integration surface for app code.
///
/// This keeps the explicit submit story separate from query observation:
/// - grouped default app data uses `cx.data().mutation_async*` to create handles,
/// - `submit(...)` starts work explicitly,
/// - render-time observation must not replay submit work,
/// - and `fret::mutation` keeps the semantic submit-state nouns off `fret::app::prelude::*`.
#[cfg(feature = "state-mutation")]
pub mod mutation {
    /// Raw mutation-core exports for advanced or fully explicit use.
    pub mod core {
        pub use fret_mutation::*;
    }

    pub use fret_mutation::ui::MutationHandleActionExt;
    pub use fret_mutation::{
        CancellationToken, FutureSpawner, FutureSpawnerHandle, MutationConcurrencyPolicy,
        MutationError, MutationErrorKind, MutationHandle, MutationPolicy, MutationState,
        MutationStatus,
    };
}

/// Optional router integration surface for app code.
///
/// This keeps the router story explicit:
/// - `fret-router` remains the portable matching/history/guard core,
/// - `fret-router-ui` remains the thin adoption layer,
/// - `fret::router` gives app authors one curated import lane for router types, link/outlet
///   helpers, and `RouterUiStore` history action bindings without pulling router types into
///   `fret::app::prelude::*`.
#[cfg(feature = "router")]
pub mod router {
    /// Raw router-core exports for advanced or fully explicit use.
    pub mod core {
        pub use fret_router::*;
    }

    #[cfg(target_arch = "wasm32")]
    pub use fret_router::{HashHistoryAdapter, WebHistoryAdapter};
    pub use fret_router::{
        HistoryAdapter, MemoryHistory, NamespaceInvalidationRule, NavigationAction, PathParam,
        PathPattern, PathPatternError, RouteChangePolicy, RouteCodec, RouteHooks, RouteLocation,
        RouteNode, RoutePrefetchIntent, RouteSearchTable, RouteSearchValidationFailure, RouteTree,
        Router, RouterBuildLocationError, RouterEvent, RouterTransition, RouterUpdate,
        RouterUpdateWithPrefetchIntents, SearchMap, SearchValidationMode,
        collect_invalidated_namespaces, prefetch_intent_query_key,
    };
    pub use fret_router_ui::{
        RouterLeafStatus, RouterLink, RouterLinkContextMenuAction, RouterLinkContextMenuItem,
        RouterOutlet, RouterUiSnapshot, RouterUiStore,
    };

    pub fn router_outlet<'a, Cx, R, T>(
        cx: &mut Cx,
        snapshot: &fret_runtime::Model<RouterUiSnapshot<R>>,
        render: impl FnOnce(&mut crate::AppRenderCx<'_>, &RouterUiSnapshot<R>) -> T,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + 'static,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_outlet(cx.elements(), snapshot, render)
    }

    pub fn router_outlet_with_test_id<'a, Cx, R, T>(
        cx: &mut Cx,
        snapshot: &fret_runtime::Model<RouterUiSnapshot<R>>,
        test_id: impl Into<std::sync::Arc<str>>,
        render: impl FnOnce(&mut crate::AppRenderCx<'_>, &RouterUiSnapshot<R>) -> T,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + 'static,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_outlet_with_test_id(cx.elements(), snapshot, test_id, render)
    }

    pub fn router_outlet_by_leaf_with_test_id<'a, Cx, R, T, N>(
        cx: &mut Cx,
        snapshot: &fret_runtime::Model<RouterUiSnapshot<R>>,
        test_id: impl Into<std::sync::Arc<str>>,
        render: impl FnOnce(&mut crate::AppRenderCx<'_>, &R, &RouterUiSnapshot<R>) -> T,
        not_found: impl FnOnce(&mut crate::AppRenderCx<'_>, &RouterUiSnapshot<R>) -> N,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + 'static,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
        N: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::RouterOutlet::new(snapshot.clone())
            .test_id(test_id)
            .into_element_by_leaf(cx.elements(), render, not_found)
    }

    pub fn router_link<'a, Cx, R, H, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        link: RouterLink,
        children: I,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link(cx.elements(), store, link, children)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn router_link_to<'a, Cx, R, H, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        action: NavigationAction,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
        children: I,
    ) -> Result<fret_ui::element::AnyElement, RouterBuildLocationError>
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link_to(
            cx.elements(),
            store,
            action,
            route,
            params,
            search,
            fragment,
            children,
        )
    }

    pub fn router_link_to_typed_route<'a, Cx, R, H, C, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        action: NavigationAction,
        codec: &C,
        route: &C::Route,
        children: I,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        C: RouteCodec,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link_to_typed_route(
            cx.elements(),
            store,
            action,
            codec,
            route,
            children,
        )
    }

    pub fn router_link_to_typed_route_with_test_id<'a, Cx, R, H, C, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        action: NavigationAction,
        codec: &C,
        route: &C::Route,
        test_id: impl Into<std::sync::Arc<str>>,
        children: I,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        C: RouteCodec,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link_to_typed_route_with_test_id(
            cx.elements(),
            store,
            action,
            codec,
            route,
            test_id,
            children,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn router_link_to_with_test_id<'a, Cx, R, H, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        action: NavigationAction,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
        test_id: impl Into<std::sync::Arc<str>>,
        children: I,
    ) -> Result<fret_ui::element::AnyElement, RouterBuildLocationError>
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link_to_with_test_id(
            cx.elements(),
            store,
            action,
            route,
            params,
            search,
            fragment,
            test_id,
            children,
        )
    }

    pub fn router_link_with_test_id<'a, Cx, R, H, I, T>(
        cx: &mut Cx,
        store: &RouterUiStore<R, H>,
        link: RouterLink,
        test_id: impl Into<std::sync::Arc<str>>,
        children: I,
    ) -> fret_ui::element::AnyElement
    where
        Cx: crate::app::AppRenderContext<'a>,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
        I: IntoIterator<Item = T>,
        T: fret_ui_kit::IntoUiElement<crate::app::App>,
    {
        fret_router_ui::router_link_with_test_id(cx.elements(), store, link, test_id, children)
    }

    /// Bind router back/forward handlers to typed app actions.
    ///
    /// This keeps default app code on the router facade instead of importing the raw
    /// `AppUi::on_action_notify(...)` bridge.
    pub fn bind_history_actions<Back, Forward, R, H>(
        cx: &mut crate::AppUi<'_, '_>,
        store: &RouterUiStore<R, H>,
        _back: Back,
        _forward: Forward,
    ) where
        Back: crate::TypedAction,
        Forward: crate::TypedAction,
        R: Clone + Eq + std::hash::Hash + 'static,
        H: fret_router::HistoryAdapter + 'static,
    {
        use crate::view::AppUiRawActionNotifyExt as _;

        cx.on_action_notify::<Back>(store.back_on_action());
        cx.on_action_notify::<Forward>(store.forward_on_action());
    }

    /// Explicit router app-install helpers for the default app lane.
    pub mod app {
        /// Register recommended router commands on the app surface.
        ///
        /// Use this from `FretApp::setup(...)` so default command keybindings/config layering can
        /// see the router commands before the bootstrap installs baseline keymaps.
        pub fn install(app: &mut crate::app::App) {
            fret_router_ui::app::install(app);
        }
    }
}

/// Explicit advanced/manual-assembly imports for power users and integration code.
pub mod advanced;

#[derive(Debug, thiserror::Error)]
/// Public error type for the `fret` facade.
pub enum Error {
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),
    #[error(transparent)]
    AssetManifest(#[from] AssetManifestError),
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    #[error(transparent)]
    AssetStartup(#[from] fret_bootstrap::AssetStartupPlanError),
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    #[error(transparent)]
    Runner(#[from] RunnerError),
}

/// Result type used by the `fret` facade.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BootstrapError(#[from] fret_bootstrap::BootstrapError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AssetManifestError(#[from] fret_assets::AssetManifestLoadError);

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RunnerError(#[from] fret_launch::RunnerError);

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl Error {
    /// Returns a structured bootstrap failure report when this facade error represents a known
    /// startup/install failure taxonomy case.
    pub fn known_bootstrap_failure_report(
        &self,
    ) -> Option<fret_bootstrap::BootstrapKnownFailureReport> {
        match self {
            Error::Bootstrap(err) => Some(err.0.known_failure_report()),
            Error::AssetManifest(err) => {
                Some(fret_bootstrap::BootstrapKnownFailureReport::from_asset_manifest_error(&err.0))
            }
            Error::AssetStartup(err) => {
                Some(fret_bootstrap::BootstrapKnownFailureReport::from_asset_startup_error(err))
            }
            Error::Runner(_) => None,
        }
    }
}

mod builder;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use builder::{UiAppBuilder, UiAppDriver};

#[cfg(test)]
mod authoring_surface_policy_tests;
