//! Batteries-included desktop-first entry points for Fret.
//!
//! This crate is intentionally **ecosystem-level**:
//! - it composes `fret-bootstrap` (golden-path wiring) with a default component surface,
//! - it enables a practical desktop-first default stack,
//! - it remains optional: advanced users can depend on `fret-framework` + `fret-bootstrap` directly.
//! - it is **not** the repository?s canonical example host; runnable lessons stay in app-owned
//!   surfaces such as `apps/fret-cookbook`, `apps/fret-ui-gallery`, and other app shells.
//!
//! ## Choosing a native entry path
//!
//! - `fret::FretApp::new(...).window(...).view::<V>()?` is the recommended app-author path.
//! - `fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?` is the recommended advanced
//!   app-author path when driver hooks are required.
//! - `fret::advanced::ui_app(...)` and `fret::advanced::ui_app_with_hooks(...)` are the
//!   recommended explicit manual-assembly entry points when you want the golden-path UI app
//!   builder without depending on `fret-bootstrap` directly.
//! - `fret::advanced::run_native_with_fn_driver(...)`,
//!   `fret::advanced::run_native_with_fn_driver_with_hooks(...)`, and
//!   `fret::advanced::run_native_with_configured_fn_driver(...)` are the recommended advanced
//!   escape hatches when you need runner-level customization but still want the `fret`
//!   defaults/bootstrap story.
//! - `fret::advanced::interop::run_native_with_compat_driver(...)` is an advanced low-level
//!   interop path (non-default) for retained/bridge integrations that still implement
//!   `fret_launch::WinitAppDriver` directly.
//! - `fret::advanced::kernel::*` and `fret::advanced::interop::*` keep low-level runtime,
//!   rendering, and viewport/foreign-surface seams explicit on the advanced lane.
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
//! Optional ecosystem extensions stay explicit:
//!
//! - enable `state` for grouped selector/query helpers on `AppUi`; prefer
//!   `cx.data().selector_layout(...)` for LocalState-first derived values, keep
//!   `cx.data().query*(...)` plus `handle.read_layout(cx)` as the default query read path, and use
//!   `cx.data().invalidate_query(...)` / `cx.data().invalidate_query_namespace(...)` when
//!   app-facing query invalidation stays inside `AppUi`; when app code needs explicit state helper
//!   nouns, use `fret::selector::ui::DepsBuilder`, `fret::selector::DepsSignature`, and
//!   `fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}` instead of expecting
//!   those names from `fret::app::prelude::*`
//! - enable `router` for `fret::router::{app::install, RouterUiStore, RouterOutlet, router_link, ...}`
//!   plus `RouterUiStore::{back_on_action, forward_on_action}` history bindings
//! - enable `docking` for `fret::docking::{core::*, DockManager, handle_dock_op, ...}`
//! - use `fret::assets::{AssetBundleId, AssetLocator, AssetRequest, StaticAssetEntry, ...}`
//!   for logical bundle/embedded assets; prefer `AssetBundleId::app(...)` /
//!   `AssetBundleId::package(...)` over raw global strings; on native/package-dev lanes you can
//!   mount a scanned bundle directory via `AssetStartupPlan` + `AssetStartupMode` on the high
//!   lane, or `FretApp::asset_dir(...)`, `UiAppBuilder::with_asset_dir(...)`, and
//!   `register_file_bundle_dir(...)` on the lower-level native/package-dev lane; use
//!   `FretApp::asset_manifest(...)`, `UiAppBuilder::with_asset_manifest(...)`, or
//!   `register_file_manifest(...)` when tooling already emits an explicit manifest artifact; treat
//!   `AssetLocator::file(...)` and `AssetLocator::url(...)` as capability-gated escape hatches;
//!   when native/dev-only UI helpers still need file reload ergonomics, keep app/widget code on
//!   logical bundle locators and let
//!   `fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`
//!   or `fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`
//!   consume the resolver's bundle/reference bridge instead of constructing raw file-path sources
//!   directly; keep `resolve_image_source_from_host_locator(...)` /
//!   `resolve_svg_file_source_from_host_locator(...)` as the lower-level compatibility seam when a
//!   non-UI integration truly needs the bridged source or native file handoff object
//! - enable `editor` for opt-in app-level replay of installed `fret-ui-editor` presets after the
//!   `FretApp` shadcn auto-theme middleware resets the host theme
//! - use `fret::shadcn::{..., app::install, themes::apply_shadcn_new_york, raw::*}` for the
//!   curated default design-system surface; component families live on `shadcn::Button` /
//!   `shadcn::Card`, `shadcn::app::*` and `shadcn::themes::*` are setup lanes rather than peer
//!   discovery lanes, and advanced environment / `UiServices` hooks stay on
//!   `fret::shadcn::raw::advanced::*`
//! - use `fret::integration::InstallIntoApp` for reusable app-install bundles; small app-local
//!   composition can also use `.setup((install_a, install_b))` while ordinary app code keeps
//!   passing named installer functions to `.setup(...)` and keeps inline one-off closures or
//!   runtime-captured config on `UiAppBuilder::setup_with(...)`
use std::path::PathBuf;

use crate::advanced::KernelApp;

/// Canonical app-facing window identity alias for the default authoring surface.
pub type WindowId = fret_core::AppWindowId;

/// Re-export the curated default shadcn/ui surface as `shadcn`.
#[cfg(feature = "shadcn")]
pub use fret_ui_shadcn::facade as shadcn;

/// Re-export portable action/command identity types for app code and macros.
pub use fret_runtime::{ActionId, CommandId, TypedAction};

/// Explicit icon helpers and identifiers for app and component code that opt into icon-specific
/// authoring.
pub mod icons {
    pub use fret_icons::IconId;
    pub use fret_ui_kit::declarative::icon;
}

/// Explicit accessibility/semantics nouns for app code that needs semantic-role overrides.
pub mod semantics {
    pub use fret_core::SemanticsRole;
}

/// Explicit style/token nouns for app code that customizes layout or chrome beyond the default lane.
pub mod style {
    pub use fret_core::{TextOverflow, TextWrap};
    pub use fret_ui::{Theme, ThemeSnapshot};
    pub use fret_ui_kit::{
        ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, ShadowPreset, Size, Space,
    };
}

/// Explicit environment and responsive helpers for app or component code that opts into adaptive
/// UI logic.
pub mod env {
    pub use fret_ui_kit::declarative::{
        accent_color, container_breakpoints, container_query_region,
        container_query_region_with_id, container_width_at_least, contrast_preference,
        forced_colors_active, forced_colors_mode, occlusion_insets, occlusion_insets_or_zero,
        preferred_color_scheme, prefers_dark_color_scheme, prefers_more_contrast,
        prefers_reduced_motion, prefers_reduced_transparency, primary_pointer_can_hover,
        primary_pointer_is_coarse, primary_pointer_type, safe_area_insets,
        safe_area_insets_or_zero, tailwind, text_scale_factor, viewport_aspect_ratio,
        viewport_breakpoints, viewport_height_at_least, viewport_height_breakpoints,
        viewport_is_landscape, viewport_is_portrait, viewport_orientation, viewport_tailwind,
        viewport_width_at_least, window_insets_padding_refinement_or_zero,
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

/// Explicit logical asset-contract vocabulary and host registration helpers for app code.
///
/// The portable default story is bundle/embedded locators. Prefer `AssetBundleId::app(...)` and
/// `AssetBundleId::package(...)` over ad-hoc global strings. Native/package-dev builds can also
/// mount scanned bundle directories or explicit file-backed manifests without leaking raw paths
/// into widget code. Raw files and URLs stay explicit, capability-gated escape hatches.
pub mod assets {
    use std::sync::Arc;

    #[cfg(not(target_arch = "wasm32"))]
    pub use fret_assets::FileAssetManifestResolver;
    pub use fret_assets::{
        AssetBundleId, AssetBundleNamespace, AssetCapabilities, AssetExternalReference, AssetKey,
        AssetKindHint, AssetLoadError, AssetLocator, AssetLocatorKind, AssetManifestLoadError,
        AssetMediaType, AssetMemoryKey, AssetRequest, AssetResolver, AssetRevision,
        FILE_ASSET_MANIFEST_KIND_V1, FileAssetManifestBundleV1, FileAssetManifestEntryV1,
        FileAssetManifestV1, ResolvedAssetBytes, ResolvedAssetReference, StaticAssetEntry,
        asset_app_bundle_id, asset_package_bundle_id,
    };
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
    pub fn set_primary_resolver(
        host: &mut impl fret_runtime::GlobalsHost,
        resolver: Arc<dyn AssetResolver>,
    ) {
        fret_runtime::set_asset_resolver(host, resolver);
    }

    /// Add an additional resolver layer without replacing earlier registrations.
    ///
    /// Host resolver registrations preserve insertion order across primary, layered, and static
    /// entry registrations, so later registrations take precedence over earlier ones for the same
    /// logical locator.
    pub fn register_resolver(
        host: &mut impl fret_runtime::GlobalsHost,
        resolver: Arc<dyn AssetResolver>,
    ) {
        fret_runtime::register_asset_resolver(host, resolver);
    }

    /// Load a native/package-dev file manifest and register it as a layered resolver.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn register_file_manifest(
        host: &mut impl fret_runtime::GlobalsHost,
        manifest_path: impl AsRef<std::path::Path>,
    ) -> Result<(), AssetManifestLoadError> {
        let resolver = FileAssetManifestResolver::from_manifest_path(manifest_path)?;
        register_resolver(host, Arc::new(resolver));
        Ok(())
    }

    /// Scan a native/package-dev directory and register it as a logical bundle resolver layer.
    ///
    /// This is a convenience lane for local development and package-time assembly. Prefer
    /// [`register_file_manifest`] when your tooling already emits a reviewable manifest artifact.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn register_file_bundle_dir(
        host: &mut impl fret_runtime::GlobalsHost,
        bundle: impl Into<AssetBundleId>,
        dir: impl AsRef<std::path::Path>,
    ) -> Result<(), AssetManifestLoadError> {
        let resolver = FileAssetManifestResolver::from_bundle_dir(bundle, dir)?;
        register_resolver(host, Arc::new(resolver));
        Ok(())
    }

    /// Register static bundle-scoped entries on the current host.
    ///
    /// These entries participate in the same ordered host resolver stack as other registrations,
    /// so a later static registration can override an earlier resolver layer and vice versa.
    pub fn register_bundle_entries(
        host: &mut impl fret_runtime::GlobalsHost,
        bundle: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        fret_runtime::register_bundle_asset_entries(host, bundle, entries);
    }

    /// Register static embedded entries owned by a specific bundle or crate.
    ///
    /// These entries participate in the same ordered host resolver stack as other registrations,
    /// so a later static registration can override an earlier resolver layer and vice versa.
    pub fn register_embedded_entries(
        host: &mut impl fret_runtime::GlobalsHost,
        owner: impl Into<AssetBundleId>,
        entries: impl IntoIterator<Item = StaticAssetEntry>,
    ) {
        fret_runtime::register_embedded_asset_entries(host, owner, entries);
    }

    /// Inspect the composed asset resolver service installed on the current host.
    pub fn resolver(host: &impl fret_runtime::GlobalsHost) -> Option<&AssetResolverService> {
        fret_runtime::asset_resolver(host)
    }

    /// Report the current host's aggregated asset capabilities.
    pub fn capabilities(host: &impl fret_runtime::GlobalsHost) -> Option<AssetCapabilities> {
        fret_runtime::asset_capabilities(host)
    }

    /// Resolve bytes for a logical asset request through the host-installed resolver chain.
    pub fn resolve_bytes(
        host: &impl fret_runtime::GlobalsHost,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        fret_runtime::resolve_asset_bytes(host, request)
    }

    /// Resolve bytes for a single locator through the host-installed resolver chain.
    pub fn resolve_locator(
        host: &impl fret_runtime::GlobalsHost,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        fret_runtime::resolve_asset_locator_bytes(host, locator)
    }

    /// Resolve an external file/URL reference for a logical asset request through the
    /// host-installed resolver chain.
    pub fn resolve_reference(
        host: &impl fret_runtime::GlobalsHost,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetReference, AssetLoadError> {
        fret_runtime::resolve_asset_reference(host, request)
    }

    /// Resolve an external file/URL reference for a single locator through the host-installed
    /// resolver chain.
    pub fn resolve_locator_reference(
        host: &impl fret_runtime::GlobalsHost,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetReference, AssetLoadError> {
        fret_runtime::resolve_asset_locator_reference(host, locator)
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Debug, Clone)]
pub(crate) enum AssetMount {
    Dir {
        bundle: fret_assets::AssetBundleId,
        dir: PathBuf,
    },
    Manifest {
        path: PathBuf,
    },
    BundleEntries {
        bundle: fret_assets::AssetBundleId,
        entries: Vec<fret_assets::StaticAssetEntry>,
    },
    EmbeddedEntries {
        owner: fret_assets::AssetBundleId,
        entries: Vec<fret_assets::StaticAssetEntry>,
    },
    Startup {
        bundle: fret_assets::AssetBundleId,
        mode: fret_bootstrap::AssetStartupMode,
        plan: fret_bootstrap::AssetStartupPlan,
    },
    ReloadPolicy {
        policy: fret_bootstrap::AssetReloadPolicy,
    },
}

pub mod actions;
mod view;
pub mod workspace_menu;
pub mod workspace_shell;

/// Explicit app-integration contracts for reusable ecosystem bundles.
pub mod integration;

mod pending_shortcut_overlay;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
mod app_entry;
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use app_entry::FretApp;

/// Canonical app-facing UI context alias for the default authoring surface.
pub type AppUi<'cx, 'a, H = crate::app::App> = view::AppUi<'cx, 'a, H>;

/// Canonical app-facing render return alias for the default authoring surface.
pub type Ui = fret_ui::element::Elements;

/// App-facing helper context alias for extracted child-builder functions on the default surface.
pub type UiCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;

/// Canonical component-facing context alias for reusable component authoring.
pub type ComponentCx<'a, H> = fret_ui::ElementContext<'a, H>;

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
#[cfg(feature = "desktop")]
use fret_framework as kernel;

/// App-facing imports for ordinary Fret application code.
pub mod app {
    /// Canonical app-facing view trait on the explicit app lane.
    pub use crate::view::View;
    /// Explicit helper types/traits for app helper signatures that intentionally name them.
    pub use crate::view::{LocalState, UiCxActionsExt, UiCxDataExt};
    /// Canonical app-facing runtime handle on the default `fret` surface.
    ///
    /// This is the same underlying runtime type as the raw kernel alias exposed on
    /// `fret::advanced::kernel`; prefer this name in ordinary app code and keep the raw alias for
    /// advanced/manual integration seams.
    pub use fret_app::App;

    /// Common imports for app code on the default authoring surface.
    pub mod prelude {
        #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
        pub use crate::FretApp;
        pub use crate::app::App;
        #[cfg(feature = "shadcn")]
        pub use crate::shadcn;
        #[cfg(feature = "state-query")]
        pub use crate::view::QueryHandleReadExt as _;
        pub use crate::view::TrackedStateExt as _;
        pub use crate::view::UiCxActionsExt as _;
        pub use crate::view::UiCxDataExt as _;
        pub use crate::view::View;
        pub use crate::{AppUi, Ui, UiChild, UiCx, WindowId};
        pub use fret_core::Px;
        pub use fret_ui_kit::IntoUiElement as _;
        pub use fret_ui_kit::StyledExt as _;
        pub use fret_ui_kit::UiExt as _;
        pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;
        pub use fret_ui_kit::declarative::UiElementA11yExt as _;
        pub use fret_ui_kit::declarative::UiElementTestIdExt as _;
        pub use fret_ui_kit::ui;
    }

    /// Explicit bridge for app-facing widgets that only expose `on_activate(...)`.
    ///
    /// This intentionally stays off `fret::app::prelude::*` so default app autocomplete remains
    /// focused on native widget action slots. Import `use fret::app::AppActivateExt as _;`
    /// explicitly at call sites that still need activation-only `.action(...)`,
    /// `.action_payload(...)`, or `.listen(...)` sugar.
    pub use crate::view::{AppActivateExt, AppActivateSurface};
}

/// Component-author imports for reusable, portable UI crates.
pub mod component {
    /// Common imports for reusable component crates built on Fret.
    pub mod prelude {
        pub use crate::ComponentCx;
        pub use fret_ui_kit::IntoUiElement as _;
        pub use fret_ui_kit::command::ElementCommandGatingExt as _;
        pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;
        pub use fret_ui_kit::declarative::ElementContextThemeExt as _;
        pub use fret_ui_kit::declarative::GlobalWatchExt as _;
        pub use fret_ui_kit::declarative::ModelWatchExt as _;
        pub use fret_ui_kit::declarative::TrackedModelExt as _;
        pub use fret_ui_kit::declarative::UiElementA11yExt as _;
        pub use fret_ui_kit::declarative::UiElementKeyContextExt as _;
        pub use fret_ui_kit::declarative::UiElementTestIdExt as _;
        pub use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
        pub use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
        pub use fret_ui_kit::ui;
        pub use fret_ui_kit::ui::UiElementSinkExt as _;
        pub use fret_ui_kit::{
            ChromeRefinement, ColorRef, Corners4, Edges4, IntoUiElement, LayoutRefinement,
            MetricRef, OverlayController, OverlayPresence, OverlayRequest, Radius, ShadowPreset,
            Size, Space, UiBuilder, UiExt, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
        };

        #[cfg(feature = "icons")]
        pub use fret_icons::IconId;
        #[cfg(feature = "icons")]
        pub use fret_ui_kit::declarative::icon;

        pub use fret_core::{Px, SemanticsRole, TextOverflow, TextWrap};
        pub use fret_runtime::Model;
        pub use fret_ui::element::{AnyElement, AnyElementIterExt as _};
        pub use fret_ui::{Invalidation, Theme, UiHost};
    }
}

/// Optional selector integration surface for app code.
///
/// This keeps the selector story explicit:
/// - grouped default app data stays on `cx.data().selector_layout(...)` for LocalState-first
///   inputs, with raw `cx.data().selector(...)` kept explicit,
/// - `fret-selector` remains the portable derived-state crate,
/// - `fret::selector` keeps selector-core nouns on the explicit lane, while UI dependency builders
///   stay under `fret::selector::ui::*` instead of widening `fret::app::prelude::*`.
#[cfg(feature = "state-selector")]
pub mod selector {
    /// Raw selector-core exports for advanced or fully explicit use.
    pub mod core {
        pub use fret_selector::*;
    }

    /// Raw selector-UI adoption exports for advanced or fully explicit use.
    pub mod ui {
        pub use fret_selector::ui::*;
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

    /// Raw query-UI adoption exports for advanced or fully explicit use.
    pub mod ui {
        pub use fret_query::ui::*;
    }

    pub use fret_query::{
        CancellationToken, FutureSpawner, FutureSpawnerHandle, QueryCancelMode, QueryClient,
        QueryClientSnapshot, QueryError, QueryErrorKind, QueryHandle, QueryKey, QueryPolicy,
        QueryRetryOn, QueryRetryPolicy, QueryRetryState, QuerySnapshotEntry, QueryState,
        QueryStatus, with_query_client,
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

    /// Raw router-UI adoption exports for advanced or fully explicit use.
    pub mod ui {
        pub use fret_router_ui::*;
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
        RouterOutlet, RouterUiSnapshot, RouterUiStore, register_router_commands, router_link,
        router_link_to, router_link_to_typed_route, router_link_to_typed_route_with_test_id,
        router_link_to_with_test_id, router_link_with_props, router_link_with_test_id,
        router_outlet, router_outlet_with_test_id,
    };

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

/// Optional docking integration surface for advanced app code.
///
/// This keeps the docking story explicit:
/// - docking data contracts remain in `fret-core`,
/// - `fret-docking` remains the policy-heavy UI/runtime adoption layer,
/// - `fret::docking` gives advanced app code one curated import lane without leaking docking types
///   into `fret::app::prelude::*`.
#[cfg(feature = "docking")]
pub mod docking {
    /// Raw docking core contracts for advanced or fully explicit use.
    pub mod core {
        pub use fret_core::dock::*;
        pub use fret_core::{
            DOCK_LAYOUT_VERSION, DockLayout, DockLayoutBuilder, DockLayoutFloatingWindow,
            DockLayoutNode, DockLayoutValidationError, DockLayoutValidationErrorKind,
            DockLayoutWindow, DockNodeId, DockOp, DockRect, DockWindowPlacement,
            EditorDockLayoutSpec, PanelKey, SplitFractionsUpdate,
        };
    }

    /// Raw docking UI/policy exports for advanced or fully explicit use.
    pub mod ui {
        pub use fret_docking::*;
    }

    /// Raw docking runtime integration helpers for advanced or fully explicit use.
    pub mod runtime {
        pub use fret_docking::runtime::*;
    }

    pub use fret_docking::runtime::{recenter_in_window_floatings, request_dock_invalidation};
    pub use fret_docking::{
        ActivatePanelOptions, DockManager, DockPanel, DockPanelFactory, DockPanelFactoryCx,
        DockPanelFactoryRegistry, DockPanelRegistry, DockPanelRegistryBuilder,
        DockPanelRegistryService, DockSpace, DockSpaceMount, DockViewportLayout,
        DockViewportOverlayHooks, DockViewportOverlayHooksService, DockingPolicy,
        DockingPolicyService, DockingRuntime, DuplicateDockPanelKindError, ViewportPanel,
        create_dock_space_node, create_dock_space_node_with_test_id,
        handle_dock_before_close_window, handle_dock_op, handle_dock_window_created,
        mount_dock_space, mount_dock_space_with_test_id, render_and_bind_dock_panels,
        render_cached_panel_root,
    };
}

/// Explicit advanced/manual-assembly imports for power users and integration code.
pub mod advanced {
    /// Low-level view-runtime helpers kept off the default crate root.
    pub mod view {
        pub use crate::view::{
            ViewWindowState, view_init_window, view_record_engine_frame, view_view,
        };
    }

    /// Dev-only helpers (hotpatch/dev-state) for iteration workflows.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "devloop"))]
    pub mod dev {
        pub use fret_launch::dev_state::{
            DevStateExport, DevStateHook, DevStateHooks, DevStateSnapshot,
            DevStateWindowKeyRegistry,
        };
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    /// Low-level interop helpers kept off the default crate root.
    pub mod interop {
        pub use crate::interop::embedded_viewport;
        pub use crate::interop::run_native_with_compat_driver;
    }
    /// Explicit raw action-registration hooks kept on the advanced lane.
    ///
    /// This keeps manual `on_action*` / `on_payload_action*` registration discoverable for
    /// advanced/manual assembly and host-owned integrations while leaving
    /// `fret::app::prelude::*` focused on `cx.actions()`.
    pub use crate::view::AppUiRawActionExt;
    /// Explicit raw-model local-state hooks kept on the advanced lane.
    ///
    /// This keeps `use_state*` discoverable for advanced/manual assembly and intentional
    /// `Model<T>`-centric code while leaving `fret::app::prelude::*` focused on
    /// `LocalState<T>` / `cx.state().local*`.
    pub use crate::view::AppUiRawStateExt;
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::{UiAppBuilder, UiAppDriver};
    pub use fret_app::App as KernelApp;
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use fret_bootstrap::ui_app_driver::ViewElements;
    #[cfg(feature = "desktop")]
    /// Low-level kernel facade kept off the default crate root.
    pub use fret_framework as kernel;

    /// Create a golden-path native UI app builder on the explicit advanced surface.
    ///
    /// This mirrors `fret-bootstrap`'s `ui_app(...)` helper while keeping author-facing code on
    /// the `fret::advanced` surface.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub fn ui_app<S: 'static>(
        root_name: &'static str,
        init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
    ) -> crate::UiAppBuilder<S> {
        ui_app_with_hooks(root_name, init_window, view, |driver| driver)
    }

    /// Create a golden-path native UI app builder on the explicit advanced surface, preserving
    /// the driver hook configuration seam.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub fn ui_app_with_hooks<S: 'static>(
        root_name: &'static str,
        init_window: fn(&mut KernelApp, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, KernelApp>, &mut S) -> ViewElements,
        configure: fn(crate::UiAppDriver<S>) -> crate::UiAppDriver<S>,
    ) -> crate::UiAppBuilder<S> {
        let driver = fret_bootstrap::ui_app_driver::UiAppDriver::new(root_name, init_window, view);
        let driver = configure(crate::UiAppDriver::new(driver))
            .into_inner()
            .into_fn_driver();
        crate::UiAppBuilder::from_bootstrap(fret_bootstrap::BootstrapBuilder::new(
            KernelApp::new(),
            driver,
        ))
    }

    /// Run a native desktop app using the advanced `FnDriver` escape hatch.
    ///
    /// This is the recommended low-level path when the app wants the `fret`
    /// bootstrap/defaults story but needs runner-level customization without teaching
    /// `WinitAppDriver` as the primary model.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub fn run_native_with_fn_driver<D: 'static, S: 'static>(
        config: fret_launch::WinitRunnerConfig,
        app: KernelApp,
        driver_state: D,
        create_window_state: fn(&mut D, &mut KernelApp, fret_core::AppWindowId) -> S,
        handle_event: for<'d, 'cx, 'e> fn(
            &'d mut D,
            fret_launch::WinitEventContext<'cx, S>,
            &'e fret_core::Event,
        ),
        render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
    ) -> crate::Result<()> {
        run_native_with_fn_driver_with_hooks(
            config,
            app,
            driver_state,
            create_window_state,
            handle_event,
            render,
            |_hooks| {},
        )
    }

    /// Run a native desktop app using the advanced `FnDriver` escape hatch, preserving hook
    /// configuration.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub fn run_native_with_fn_driver_with_hooks<D: 'static, S: 'static>(
        config: fret_launch::WinitRunnerConfig,
        app: KernelApp,
        driver_state: D,
        create_window_state: fn(&mut D, &mut KernelApp, fret_core::AppWindowId) -> S,
        handle_event: for<'d, 'cx, 'e> fn(
            &'d mut D,
            fret_launch::WinitEventContext<'cx, S>,
            &'e fret_core::Event,
        ),
        render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
        configure_hooks: impl FnOnce(&mut fret_launch::FnDriverHooks<D, S>),
    ) -> crate::Result<()> {
        let builder = fret_bootstrap::BootstrapBuilder::new_fn_with_hooks(
            app,
            driver_state,
            create_window_state,
            handle_event,
            render,
            configure_hooks,
        )
        .configure(move |c| {
            *c = config;
        });

        let builder =
            crate::apply_desktop_defaults(builder).map_err(crate::BootstrapError::from)?;

        builder.run().map_err(crate::RunnerError::from)?;
        Ok(())
    }

    /// Run a native desktop app using a preconfigured advanced `FnDriver` instance.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub fn run_native_with_configured_fn_driver<D: 'static, S: 'static>(
        config: fret_launch::WinitRunnerConfig,
        app: KernelApp,
        driver: fret_launch::FnDriver<D, S>,
    ) -> crate::Result<()> {
        let builder = fret_bootstrap::BootstrapBuilder::new(app, driver).configure(move |c| {
            *c = config;
        });

        let builder =
            crate::apply_desktop_defaults(builder).map_err(crate::BootstrapError::from)?;

        builder.run().map_err(crate::RunnerError::from)?;
        Ok(())
    }

    /// Advanced builder hooks that intentionally stay off the default `FretApp` surface.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub trait FretAppAdvancedExt: Sized {
        /// Install wiring that needs `UiServices` during bootstrap.
        fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices))
        -> Self;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    impl FretAppAdvancedExt for crate::FretApp {
        fn install(
            self,
            install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices),
        ) -> Self {
            self.install_services(install)
        }
    }

    /// Advanced `UiAppBuilder` hooks that are intentionally excluded from the default app path.
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub trait UiAppBuilderAdvancedExt: Sized {
        /// Install wiring that needs `UiServices` during bootstrap.
        fn install(self, install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices))
        -> Self;

        /// Install custom GPU effects at the renderer boundary (ADR 0299).
        ///
        /// Note: the callback receives the **kernel** app type (`fret_app::App`, re-exported here
        /// as `KernelApp`), not the `fret::FretApp` builder-chain facade.
        fn install_custom_effects(
            self,
            install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
        ) -> Self;

        /// Hook GPU-ready setup on the explicit advanced surface.
        fn on_gpu_ready(
            self,
            f: impl FnOnce(
                &mut KernelApp,
                &crate::kernel::render::WgpuContext,
                &mut crate::kernel::render::Renderer,
            ) + 'static,
        ) -> Self;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    impl<S: 'static> UiAppBuilderAdvancedExt for crate::UiAppBuilder<S> {
        fn install(
            self,
            install: fn(&mut crate::app::App, &mut dyn fret_core::UiServices),
        ) -> Self {
            Self {
                inner: self.inner.install(install),
            }
        }

        fn install_custom_effects(
            self,
            install: fn(&mut KernelApp, &mut dyn fret_core::CustomEffectService),
        ) -> Self {
            Self {
                inner: self.inner.install_custom_effects(install),
            }
        }

        fn on_gpu_ready(
            self,
            f: impl FnOnce(
                &mut KernelApp,
                &crate::kernel::render::WgpuContext,
                &mut crate::kernel::render::Renderer,
            ) + 'static,
        ) -> Self {
            Self {
                inner: self.inner.on_gpu_ready(f),
            }
        }
    }

    /// Common imports for advanced/manual-assembly application code.
    pub mod prelude {
        #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
        pub use crate::advanced::interop::embedded_viewport::{
            EmbeddedViewportForeignUiAppDriverExt, EmbeddedViewportUiAppDriverExt,
        };
        pub use crate::advanced::*;
        #[cfg(feature = "state-query")]
        pub use crate::view::QueryHandleReadExt as _;
        pub use crate::view::UiCxActionsExt as _;
        pub use crate::view::UiCxDataExt as _;
        pub use crate::view::{LocalState, TrackedStateExt, View};
        pub use crate::{AppUi, Ui, UiCx};
        pub use fret_app::Effect;
        pub use fret_core::{AppWindowId, Event, UiServices};
        #[cfg(feature = "icons")]
        pub use fret_icons::IconId;
        pub use fret_runtime::{ActionId, TypedAction};
        pub use fret_ui::element::{HoverRegionProps, Length, SemanticsProps, TextProps};
        pub use fret_ui::{ElementContext, ThemeSnapshot, UiTree};
        pub use fret_ui_kit::declarative::TrackedModelExt as _;
        #[cfg(feature = "icons")]
        pub use fret_ui_kit::declarative::icon;
    }
}

#[derive(Debug, thiserror::Error)]
/// Public error type for the `fret` facade.
pub enum Error {
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),
    #[error(transparent)]
    AssetManifest(#[from] AssetManifestError),
    #[error(transparent)]
    AssetStartup(#[from] fret_bootstrap::AssetStartupPlanError),
    #[error(transparent)]
    Runner(#[from] RunnerError),
}

/// Result type used by the `fret` facade.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BootstrapError(#[from] fret_bootstrap::BootstrapError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct AssetManifestError(#[from] fret_assets::AssetManifestLoadError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RunnerError(#[from] fret_launch::RunnerError);

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn map_bootstrap_asset_builder_error(err: fret_bootstrap::BootstrapError) -> Error {
    match err {
        fret_bootstrap::BootstrapError::AssetManifest(err) => {
            Error::AssetManifest(AssetManifestError::from(err))
        }
        fret_bootstrap::BootstrapError::AssetStartup(err) => Error::AssetStartup(err),
        other => Error::Bootstrap(BootstrapError::from(other)),
    }
}

/// A `UiAppDriver` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppDriver<S> {
    inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S> UiAppDriver<S> {
    pub(crate) fn new(inner: fret_bootstrap::ui_app_driver::UiAppDriver<S>) -> Self {
        Self { inner }
    }

    pub(crate) fn into_inner(self) -> fret_bootstrap::ui_app_driver::UiAppDriver<S> {
        self.inner
    }

    pub fn close_on_window_close_requested(mut self, enabled: bool) -> Self {
        self.inner = self.inner.close_on_window_close_requested(enabled);
        self
    }

    #[cfg(feature = "ui-assets")]
    pub fn drive_ui_assets(mut self, enabled: bool) -> Self {
        self.inner = self.inner.drive_ui_assets(enabled);
        self
    }

    pub fn on_event(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_core::Event,
        ),
    ) -> Self {
        self.inner = self.inner.on_event(f);
        self
    }

    pub fn on_command(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &fret_runtime::CommandId,
        ),
    ) -> Self {
        self.inner = self.inner.on_command(f);
        self
    }

    pub fn on_preferences(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_preferences(f);
        self
    }

    pub fn on_hot_reload_window(
        mut self,
        f: fn(
            &mut KernelApp,
            &mut dyn fret_core::UiServices,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
        ),
    ) -> Self {
        self.inner = self.inner.on_hot_reload_window(f);
        self
    }

    pub fn on_model_changes(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &[fret_app::ModelId],
        ),
    ) -> Self {
        self.inner = self.inner.on_model_changes(f);
        self
    }

    pub fn on_global_changes(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &[std::any::TypeId],
        ),
    ) -> Self {
        self.inner = self.inner.on_global_changes(f);
        self
    }

    pub fn window_create_spec(
        mut self,
        f: fn(
            &mut KernelApp,
            &fret_app::CreateWindowRequest,
        ) -> Option<fret_launch::WindowCreateSpec>,
    ) -> Self {
        self.inner = self.inner.window_create_spec(f);
        self
    }

    pub fn window_created(
        mut self,
        f: fn(&mut KernelApp, &fret_app::CreateWindowRequest, fret_core::AppWindowId),
    ) -> Self {
        self.inner = self.inner.window_created(f);
        self
    }

    pub fn before_close_window(
        mut self,
        f: fn(&mut KernelApp, fret_core::AppWindowId) -> bool,
    ) -> Self {
        self.inner = self.inner.before_close_window(f);
        self
    }

    pub fn handle_global_command(
        mut self,
        f: fn(&mut KernelApp, &mut dyn fret_core::UiServices, fret_runtime::CommandId),
    ) -> Self {
        self.inner = self.inner.handle_global_command(f);
        self
    }

    pub fn viewport_input(mut self, f: fn(&mut KernelApp, fret_core::ViewportInputEvent)) -> Self {
        self.inner = self.inner.viewport_input(f);
        self
    }

    pub fn record_engine_frame(
        mut self,
        f: fn(
            &mut KernelApp,
            fret_core::AppWindowId,
            &mut fret_ui::UiTree<KernelApp>,
            &mut S,
            &crate::kernel::render::WgpuContext,
            &mut crate::kernel::render::Renderer,
            f32,
            fret_runtime::TickId,
            fret_runtime::FrameId,
        ) -> fret_launch::EngineFrameUpdate,
    ) -> Self {
        self.inner = self.inner.record_engine_frame(f);
        self
    }

    pub fn dock_op(mut self, f: fn(&mut KernelApp, fret_core::DockOp)) -> Self {
        self.inner = self.inner.dock_op(f);
        self
    }

    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.inner = self.inner.command_palette(enabled);
        self
    }
}

/// A `UiAppBuilder` wrapper used by `fret` to avoid exposing `fret-bootstrap` types in signatures.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct UiAppBuilder<S> {
    inner: fret_bootstrap::UiAppBootstrapBuilder<S>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl<S: 'static> UiAppBuilder<S> {
    pub(crate) fn from_bootstrap(inner: fret_bootstrap::UiAppBootstrapBuilder<S>) -> Self {
        Self { inner }
    }

    pub fn with_command_default_keybindings(self) -> Self {
        Self {
            inner: self.inner.with_command_default_keybindings(),
        }
    }

    pub fn with_default_config_files(self) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_default_config_files()
                .map_err(BootstrapError::from)?,
        })
    }

    pub fn with_default_config_files_for_root(
        self,
        project_root: impl AsRef<std::path::Path>,
    ) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_default_config_files_for_root(project_root)
                .map_err(BootstrapError::from)?,
        })
    }

    pub fn with_main_window(self, title: impl Into<String>, size: (f64, f64)) -> Self {
        Self {
            inner: self.inner.with_main_window(title, size),
        }
    }

    pub fn configure(self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        Self {
            inner: self.inner.configure(f),
        }
    }

    /// Run one-off app setup inline on the builder path.
    ///
    /// Use this when the setup needs to capture runtime values or is intentionally local to this
    /// call site. Prefer [`setup`](Self::setup) with named installer functions, tuples, or named
    /// [`crate::integration::InstallIntoApp`] bundles for reusable/default app wiring.
    pub fn setup_with(self, f: impl FnOnce(&mut crate::app::App)) -> Self {
        Self {
            inner: self.inner.init_app(f),
        }
    }

    /// Run app setup through the stable installer/bundle seam.
    ///
    /// Prefer this for named installer functions, small app-local tuples, and reusable
    /// [`crate::integration::InstallIntoApp`] bundles. Keep inline closures on
    /// [`setup_with`](Self::setup_with) so the default `.setup(...)` story stays explicit.
    pub fn setup<T>(self, setup: T) -> Self
    where
        T: crate::integration::InstallIntoApp + 'static,
    {
        Self {
            inner: self.inner.init_app(move |app| setup.install_into_app(app)),
        }
    }

    /// Register a native/package-dev asset manifest on the builder path.
    ///
    /// This loads the manifest eagerly so failures stay on the build/configure path instead of
    /// surfacing later during `run()`.
    pub fn with_asset_manifest(self, manifest_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let resolver = std::sync::Arc::new(
            crate::assets::FileAssetManifestResolver::from_manifest_path(manifest_path)
                .map_err(AssetManifestError::from)?,
        );
        Ok(Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_resolver(app, resolver);
            }),
        })
    }

    /// Scan a native/package-dev directory and mount it as one logical bundle on the builder path.
    ///
    /// This eagerly validates the directory so failures stay on the build/configure path instead
    /// of surfacing later during `run()`. Prefer [`with_asset_manifest`](Self::with_asset_manifest)
    /// when you want an explicit manifest artifact that tooling can emit, review, or package.
    pub fn with_asset_dir(
        self,
        bundle: impl Into<crate::assets::AssetBundleId>,
        dir: impl AsRef<std::path::Path>,
    ) -> Result<Self> {
        let resolver = std::sync::Arc::new(
            crate::assets::FileAssetManifestResolver::from_bundle_dir(bundle, dir)
                .map_err(AssetManifestError::from)?,
        );
        Ok(Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_resolver(app, resolver);
            }),
        })
    }

    /// Register static bundle-scoped entries on the builder path.
    ///
    /// This is the packaged/web/mobile-friendly lane for compile-time owned assets such as
    /// generated `include_bytes!` modules. Builder registrations preserve call order, so later
    /// calls can intentionally override earlier ones for the same logical locator.
    pub fn with_bundle_asset_entries(
        self,
        bundle: impl Into<crate::assets::AssetBundleId>,
        entries: impl IntoIterator<Item = crate::assets::StaticAssetEntry>,
    ) -> Self {
        let bundle = bundle.into();
        let entries = entries.into_iter().collect::<Vec<_>>();
        Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_bundle_entries(app, bundle, entries);
            }),
        }
    }

    /// Register static embedded entries on the builder path.
    ///
    /// This keeps compile-time owned embedded bytes on the same ordered startup surface as other
    /// asset registrations instead of forcing callers back to ad-hoc setup hooks.
    pub fn with_embedded_asset_entries(
        self,
        owner: impl Into<crate::assets::AssetBundleId>,
        entries: impl IntoIterator<Item = crate::assets::StaticAssetEntry>,
    ) -> Self {
        let owner = owner.into();
        let entries = entries.into_iter().collect::<Vec<_>>();
        Self {
            inner: self.inner.init_app(move |app| {
                crate::assets::register_embedded_entries(app, owner, entries);
            }),
        }
    }

    /// Apply one explicit development-vs-packaged startup plan on the builder path.
    ///
    /// This higher-level surface keeps the current startup decision on one named value while still
    /// lowering to the same ordered builder registrations as `with_asset_dir(...)`,
    /// `with_asset_manifest(...)`, `with_bundle_asset_entries(...)`, and
    /// `with_embedded_asset_entries(...)`.
    pub fn with_asset_startup(
        self,
        app_bundle: impl Into<crate::assets::AssetBundleId>,
        mode: crate::assets::AssetStartupMode,
        plan: crate::assets::AssetStartupPlan,
    ) -> Result<Self> {
        Ok(Self {
            inner: self
                .inner
                .with_asset_startup(app_bundle.into(), mode, plan)
                .map_err(map_bootstrap_asset_builder_error)?,
        })
    }

    /// Enable development asset reload polling for file-backed startup mounts.
    pub fn with_asset_reload_policy(self, policy: crate::assets::AssetReloadPolicy) -> Self {
        Self {
            inner: self.inner.with_asset_reload_policy(policy),
        }
    }

    #[cfg(feature = "ui-assets")]
    pub fn with_ui_assets_budgets(
        self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        Self {
            inner: self.inner.with_ui_assets_budgets(
                image_budget_bytes,
                image_max_ready_entries,
                svg_budget_bytes,
                svg_max_ready_entries,
            ),
        }
    }

    #[cfg(feature = "preload-icon-svgs")]
    pub fn preload_icon_svgs_on_gpu_ready(self) -> Self {
        Self {
            inner: self.inner.preload_icon_svgs_on_gpu_ready(),
        }
    }

    #[cfg(feature = "diagnostics")]
    pub fn with_default_diagnostics(self) -> Self {
        Self {
            inner: self.inner.with_default_diagnostics(),
        }
    }

    pub fn run(self) -> Result<()> {
        self.inner.run().map_err(RunnerError::from)?;
        Ok(())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn apply_asset_mount<S: 'static>(
    builder: UiAppBuilder<S>,
    mount: AssetMount,
) -> Result<UiAppBuilder<S>> {
    match mount {
        AssetMount::Dir { bundle, dir } => builder.with_asset_dir(bundle, dir),
        AssetMount::Manifest { path } => builder.with_asset_manifest(path),
        AssetMount::BundleEntries { bundle, entries } => {
            Ok(builder.with_bundle_asset_entries(bundle, entries))
        }
        AssetMount::EmbeddedEntries { owner, entries } => {
            Ok(builder.with_embedded_asset_entries(owner, entries))
        }
        AssetMount::Startup { bundle, mode, plan } => {
            builder.with_asset_startup(bundle, mode, plan)
        }
        AssetMount::ReloadPolicy { policy } => Ok(builder.with_asset_reload_policy(policy)),
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn apply_asset_mounts<S: 'static>(
    builder: UiAppBuilder<S>,
    mounts: Vec<AssetMount>,
) -> Result<UiAppBuilder<S>> {
    mounts.into_iter().try_fold(builder, apply_asset_mount)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults_with<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
    defaults: Defaults,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    // Always ensure an i18n backend exists unless the app provides one.
    let builder = builder.init_app(fret_bootstrap::install_default_i18n_backend);
    let _ = defaults;

    #[cfg(feature = "diagnostics")]
    let builder = if defaults.diagnostics {
        builder.with_default_diagnostics()
    } else {
        builder
    };

    #[cfg(feature = "config-files")]
    let builder = if defaults.config_files {
        builder.with_default_config_files()?
    } else {
        builder.with_command_default_keybindings()
    };

    #[cfg(not(feature = "config-files"))]
    let builder = builder.with_command_default_keybindings();

    #[cfg(feature = "shadcn")]
    let builder = if defaults.shadcn {
        builder.install_app(fret_ui_shadcn::app::install)
    } else {
        builder
    };

    #[cfg(feature = "ui-assets")]
    let builder = if defaults.ui_assets {
        let (image_budget_bytes, image_max_ready_entries, svg_budget_bytes, svg_max_ready_entries) =
            defaults
                .ui_assets_budgets
                .unwrap_or((64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096));
        builder.with_ui_assets_budgets(
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        )
    } else {
        builder
    };

    #[cfg(feature = "icons")]
    let builder = if defaults.icons {
        builder.with_lucide_icons()
    } else {
        builder
    };

    #[cfg(feature = "preload-icon-svgs")]
    let builder = if defaults.preload_icon_svgs {
        builder.preload_icon_svgs_on_gpu_ready()
    } else {
        builder
    };

    Ok(builder)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub(crate) fn apply_desktop_defaults<D: fret_launch::WinitAppDriver + 'static>(
    builder: fret_bootstrap::BootstrapBuilder<D>,
) -> std::result::Result<fret_bootstrap::BootstrapBuilder<D>, fret_bootstrap::BootstrapError> {
    apply_desktop_defaults_with(builder, Defaults::default())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "shadcn"))]
fn shadcn_sync_theme_from_environment_on_global_changes<S>(
    app: &mut KernelApp,
    window: fret_core::AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    _st: &mut S,
    changed: &[std::any::TypeId],
) {
    if !changed.contains(&std::any::TypeId::of::<fret_core::WindowMetricsService>()) {
        return;
    }
    let config = app
        .global::<fret_ui_shadcn::app::InstallConfig>()
        .copied()
        .unwrap_or_default();
    #[cfg(feature = "editor")]
    {
        let _ = fret_ui_editor::theme::sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
            app,
            changed,
            |app| {
                let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
                    app,
                    window,
                    config.base_color,
                    config.scheme,
                );
            },
        );
    }
    #[cfg(not(feature = "editor"))]
    {
        let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
            app,
            window,
            config.base_color,
            config.scheme,
        );
    }
}

#[cfg(all(test, not(target_arch = "wasm32"), feature = "desktop"))]
mod builder_surface_tests {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::FretApp;
    use crate::advanced::{
        FretAppAdvancedExt as _, KernelApp, UiAppBuilderAdvancedExt as _, ViewElements,
    };
    use crate::app::App;
    use crate::app::prelude::FretApp as AppPreludeFretApp;
    use crate::view::View;
    use crate::{AppUi, Defaults, Error, Ui, WindowId};
    use fret_app::CreateWindowRequest;
    use fret_assets::{AssetBundleId, AssetRevision, StaticAssetEntry};
    use fret_core::{AppWindowId, DockOp, Event, UiServices, ViewportInputEvent};
    use fret_runtime::{CommandId, FrameId, TickId};

    fn install_bundle_fixture(_app: &mut App) {}

    static INSTALL_INTO_APP_CALLS: AtomicUsize = AtomicUsize::new(0);
    static INSTALL_INTO_APP_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct BundleInstaller;

    impl crate::integration::InstallIntoApp for BundleInstaller {
        fn install_into_app(self, app: &mut App) {
            INSTALL_INTO_APP_CALLS.fetch_add(1, Ordering::SeqCst);
            app.commands_mut();
        }
    }

    fn install_bundle_step_a(_app: &mut App) {
        INSTALL_INTO_APP_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn install_bundle_step_b(_app: &mut App) {
        INSTALL_INTO_APP_CALLS.fetch_add(1, Ordering::SeqCst);
    }

    fn install(_app: &mut App, _services: &mut dyn UiServices) {}

    fn on_view_event(
        _app: &mut KernelApp,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut fret_ui::UiTree<KernelApp>,
        _st: &mut crate::view::ViewWindowState<SmokeView>,
        _event: &Event,
    ) {
    }

    fn on_view_command(
        _app: &mut KernelApp,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut fret_ui::UiTree<KernelApp>,
        _st: &mut crate::view::ViewWindowState<SmokeView>,
        _command: &CommandId,
    ) {
    }

    fn handle_global_command(
        _app: &mut KernelApp,
        _services: &mut dyn UiServices,
        _command: CommandId,
    ) {
    }

    fn window_create_spec(
        _app: &mut KernelApp,
        _request: &CreateWindowRequest,
    ) -> Option<fret_launch::WindowCreateSpec> {
        None
    }

    fn window_created(_app: &mut KernelApp, _request: &CreateWindowRequest, _window: AppWindowId) {}

    fn before_close_window(_app: &mut KernelApp, _window: AppWindowId) -> bool {
        true
    }

    fn viewport_input(_app: &mut KernelApp, _event: ViewportInputEvent) {}

    fn record_view_engine_frame(
        _app: &mut KernelApp,
        _window: AppWindowId,
        _ui: &mut fret_ui::UiTree<KernelApp>,
        _st: &mut crate::view::ViewWindowState<SmokeView>,
        _context: &crate::kernel::render::WgpuContext,
        _renderer: &mut crate::kernel::render::Renderer,
        _dt_s: f32,
        _tick_id: TickId,
        _frame_id: FrameId,
    ) -> fret_launch::EngineFrameUpdate {
        fret_launch::EngineFrameUpdate::default()
    }

    fn install_custom_effects(
        _app: &mut KernelApp,
        _service: &mut dyn fret_core::CustomEffectService,
    ) {
    }

    fn dock_op(_app: &mut KernelApp, _op: DockOp) {}

    fn init_window_state(_app: &mut KernelApp, _window: AppWindowId) -> u8 {
        0
    }

    fn hook_view(_cx: &mut fret_ui::ElementContext<'_, KernelApp>, _st: &mut u8) -> ViewElements {
        ViewElements::default()
    }

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_asset_manifest_fixture() -> PathBuf {
        let root = make_temp_dir("fret-builder-asset-manifest");
        let assets_dir = root.join("assets").join("images");
        std::fs::create_dir_all(&assets_dir).expect("create assets dir");
        std::fs::write(assets_dir.join("logo.txt"), b"builder-manifest").expect("write asset");

        let bundle = AssetBundleId::app("builder-smoke");
        let manifest = format!(
            r#"{{
  "schema_version": 1,
  "kind": "fret_file_asset_manifest",
  "bundles": [
    {{
      "id": "{bundle}",
      "root": "assets",
      "entries": [
        {{
          "key": "images/logo.png",
          "path": "images/logo.txt",
          "media_type": "text/plain"
        }}
      ]
    }}
  ]
}}"#,
            bundle = bundle.as_str()
        );

        let manifest_path = root.join("assets.manifest.json");
        std::fs::write(&manifest_path, manifest).expect("write manifest");
        manifest_path
    }

    fn write_asset_dir_fixture(prefix: &str) -> PathBuf {
        let root = make_temp_dir(prefix);
        let assets_dir = root.join("images");
        std::fs::create_dir_all(&assets_dir).expect("create assets dir");
        std::fs::write(assets_dir.join("logo.png"), b"builder-dir").expect("write asset");
        root
    }

    fn configure_hook_driver(driver: crate::UiAppDriver<u8>) -> crate::UiAppDriver<u8> {
        driver.handle_global_command(handle_global_command)
    }

    struct SmokeView;

    impl View for SmokeView {
        fn init(_app: &mut App, _window: WindowId) -> Self {
            Self
        }

        fn render(&mut self, _cx: &mut AppUi<'_, '_>) -> Ui {
            Ui::default()
        }
    }

    #[test]
    fn app_builder_view_with_hooks_smoke() {
        let _builder = FretApp::new("builder-view-smoke")
            .window("Builder View Smoke", (640.0, 480.0))
            .setup(install_bundle_fixture)
            .install(install)
            .view_with_hooks::<SmokeView>(|driver| {
                driver
                    .on_event(on_view_event)
                    .on_command(on_view_command)
                    .handle_global_command(handle_global_command)
                    .window_create_spec(window_create_spec)
                    .window_created(window_created)
                    .before_close_window(before_close_window)
                    .viewport_input(viewport_input)
                    .record_engine_frame(record_view_engine_frame)
                    .dock_op(dock_op)
            })
            .expect("view_with_hooks should build")
            .configure(|config| {
                assert_eq!(config.main_window_title, "Builder View Smoke");
                assert_eq!(config.main_window_size.width, 640.0);
                assert_eq!(config.main_window_size.height, 480.0);
            })
            .setup_with(|_app| {})
            .install_custom_effects(install_custom_effects)
            .on_gpu_ready(|_app, _context, _renderer| {});
    }

    #[test]
    fn app_builder_view_smoke() {
        let _builder = FretApp::new("builder-view-basic")
            .defaults(Defaults::desktop_app())
            .window("Builder View Basic", (800.0, 600.0))
            .view::<SmokeView>()
            .expect("view should build")
            .configure(|config| {
                assert_eq!(config.main_window_title, "Builder View Basic");
                assert_eq!(config.main_window_size.width, 800.0);
                assert_eq!(config.main_window_size.height, 600.0);
            })
            .setup_with(|_app| {})
            .on_gpu_ready(|_app, _context, _renderer| {});
    }

    #[test]
    fn fret_app_asset_manifest_installs_on_builder_path() {
        let manifest_path = write_asset_manifest_fixture();

        let _builder = FretApp::new("builder-view-asset-manifest")
            .asset_manifest(&manifest_path)
            .view::<SmokeView>()
            .expect("asset manifest should load on fret app builder path");
    }

    #[test]
    fn ui_app_builder_with_asset_manifest_installs_on_builder_path() {
        let manifest_path = write_asset_manifest_fixture();

        let _builder = FretApp::new("builder-view-ui-builder-asset-manifest")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_manifest(&manifest_path)
            .expect("asset manifest should load on ui app builder path");
    }

    #[test]
    fn register_file_bundle_dir_installs_on_host_path() {
        let asset_dir = write_asset_dir_fixture("fret-register-file-bundle-dir");
        let bundle = AssetBundleId::app("builder-register-file-bundle-dir");
        let mut app = App::new();

        crate::assets::register_file_bundle_dir(&mut app, bundle.clone(), &asset_dir)
            .expect("bundle dir should register");

        let resolved = crate::assets::resolve_locator(
            &app,
            crate::assets::AssetLocator::bundle(bundle, "images/logo.png"),
        )
        .expect("registered bundle dir asset should resolve");

        assert_eq!(resolved.bytes.as_ref(), b"builder-dir");
    }

    #[test]
    fn register_file_bundle_dir_exposes_external_file_reference_on_host_path() {
        let asset_dir = write_asset_dir_fixture("fret-register-file-bundle-dir-reference");
        let bundle = AssetBundleId::app("builder-register-file-bundle-dir-reference");
        let mut app = App::new();

        crate::assets::register_file_bundle_dir(&mut app, bundle.clone(), &asset_dir)
            .expect("bundle dir should register");

        let resolved = crate::assets::resolve_locator_reference(
            &app,
            crate::assets::AssetLocator::bundle(bundle, "images/logo.png"),
        )
        .expect("registered bundle dir asset should expose an external reference");

        assert_eq!(
            resolved.reference.as_file_path(),
            Some(asset_dir.join("images/logo.png").as_path())
        );
    }

    #[test]
    fn fret_app_asset_dir_installs_on_builder_path() {
        let asset_dir = write_asset_dir_fixture("fret-builder-asset-dir");

        let _builder = FretApp::new("builder-view-asset-dir")
            .asset_dir(&asset_dir)
            .view::<SmokeView>()
            .expect("asset dir should load on fret app builder path");
    }

    #[test]
    fn ui_app_builder_with_asset_dir_installs_on_builder_path() {
        let asset_dir = write_asset_dir_fixture("fret-ui-builder-asset-dir");

        let _builder = FretApp::new("builder-view-ui-builder-asset-dir")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_dir(
                AssetBundleId::app("builder-view-ui-builder-asset-dir"),
                &asset_dir,
            )
            .expect("asset dir should load on ui app builder path");
    }

    #[test]
    fn fret_app_asset_entries_install_on_builder_path() {
        let _builder = FretApp::new("builder-view-asset-entries")
            .asset_entries([StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"builder-bytes",
            )])
            .view::<SmokeView>()
            .expect("asset entries should load on fret app builder path");
    }

    #[test]
    fn ui_app_builder_with_bundle_asset_entries_installs_on_builder_path() {
        let _builder = FretApp::new("builder-view-ui-builder-asset-entries")
            .view::<SmokeView>()
            .expect("view should build")
            .with_bundle_asset_entries(
                AssetBundleId::app("builder-view-ui-builder-asset-entries"),
                [StaticAssetEntry::new(
                    "images/logo.png",
                    AssetRevision(1),
                    b"builder-bytes",
                )],
            );
    }

    #[test]
    fn ui_app_builder_with_embedded_asset_entries_installs_on_builder_path() {
        let _builder = FretApp::new("builder-view-ui-builder-embedded-entries")
            .view::<SmokeView>()
            .expect("view should build")
            .with_embedded_asset_entries(
                AssetBundleId::package("demo-kit"),
                [
                    StaticAssetEntry::new("icons/search.svg", AssetRevision(1), br#"<svg></svg>"#)
                        .with_media_type("image/svg+xml"),
                ],
            );
    }

    #[test]
    fn fret_app_asset_startup_installs_selected_development_lane_on_builder_path() {
        let asset_dir = write_asset_dir_fixture("fret-builder-asset-startup-dev");

        let _builder = FretApp::new("builder-view-asset-startup-dev")
            .asset_startup(
                crate::assets::AssetStartupMode::Development,
                crate::assets::AssetStartupPlan::new()
                    .development_dir(&asset_dir)
                    .packaged_entries([StaticAssetEntry::new(
                        "images/logo.png",
                        AssetRevision(1),
                        b"builder-bytes",
                    )]),
            )
            .view::<SmokeView>()
            .expect("development asset startup plan should load on fret app builder path");
    }

    #[test]
    fn ui_app_builder_with_asset_startup_installs_selected_packaged_lane_on_builder_path() {
        let _builder = FretApp::new("builder-view-ui-builder-asset-startup-packaged")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_startup(
                AssetBundleId::app("builder-view-ui-builder-asset-startup-packaged"),
                crate::assets::AssetStartupMode::Packaged,
                crate::assets::AssetStartupPlan::new()
                    .development_manifest("assets.manifest.json")
                    .packaged_entries([StaticAssetEntry::new(
                        "images/logo.png",
                        AssetRevision(1),
                        b"builder-bytes",
                    )])
                    .packaged_embedded_entries(
                        AssetBundleId::package("demo-kit"),
                        [StaticAssetEntry::new(
                            "icons/search.svg",
                            AssetRevision(1),
                            br#"<svg></svg>"#,
                        )
                        .with_media_type("image/svg+xml")],
                    ),
            )
            .expect("packaged asset startup plan should load on ui app builder path");
    }

    #[test]
    fn asset_startup_mode_preferred_matches_current_target_defaults() {
        #[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
        assert_eq!(
            crate::assets::AssetStartupMode::preferred(),
            crate::assets::AssetStartupMode::Development
        );

        #[cfg(not(all(not(target_arch = "wasm32"), debug_assertions)))]
        assert_eq!(
            crate::assets::AssetStartupMode::preferred(),
            crate::assets::AssetStartupMode::Packaged
        );
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    #[test]
    fn asset_startup_plan_development_bundle_dir_if_native_is_available_on_fret_reexport() {
        let asset_dir =
            write_asset_dir_fixture("asset-startup-plan-development-bundle-dir-if-native");
        let app_bundle = AssetBundleId::app("asset-startup-plan-development-bundle-dir-if-native");
        let _builder = FretApp::new("asset-startup-plan-development-bundle-dir-if-native")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_startup(
                app_bundle.clone(),
                crate::assets::AssetStartupMode::Development,
                crate::assets::AssetStartupPlan::new()
                    .packaged_entries([StaticAssetEntry::new(
                        "images/logo.png",
                        AssetRevision(1),
                        b"builder-bytes",
                    )])
                    .development_bundle_dir_if_native(app_bundle, &asset_dir),
            )
            .expect("native helper should remain available through fret::assets");
    }

    #[test]
    fn asset_manifest_builder_methods_fail_early_for_missing_files() {
        let missing = std::env::temp_dir().join("definitely-missing-fret-assets.manifest.json");

        let fret_app_err = match FretApp::new("builder-view-missing-asset-manifest")
            .asset_manifest(&missing)
            .view::<SmokeView>()
        {
            Ok(_) => panic!("missing manifest should fail on fret app builder path"),
            Err(err) => err,
        };
        assert!(matches!(fret_app_err, Error::AssetManifest(_)));

        let ui_builder_err = match FretApp::new("builder-view-missing-asset-manifest-ui-builder")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_manifest(&missing)
        {
            Ok(_) => panic!("missing manifest should fail on ui app builder path"),
            Err(err) => err,
        };
        assert!(matches!(ui_builder_err, Error::AssetManifest(_)));
    }

    #[test]
    fn asset_dir_builder_methods_fail_early_for_missing_directories() {
        let missing = std::env::temp_dir().join("definitely-missing-fret-assets-dir");

        let fret_app_err = match FretApp::new("builder-view-missing-asset-dir")
            .asset_dir(&missing)
            .view::<SmokeView>()
        {
            Ok(_) => panic!("missing asset dir should fail on fret app builder path"),
            Err(err) => err,
        };
        assert!(matches!(fret_app_err, Error::AssetManifest(_)));

        let ui_builder_err = match FretApp::new("builder-view-missing-asset-dir-ui-builder")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_dir(
                AssetBundleId::app("builder-view-missing-asset-dir-ui-builder"),
                &missing,
            ) {
            Ok(_) => panic!("missing asset dir should fail on ui app builder path"),
            Err(err) => err,
        };
        assert!(matches!(ui_builder_err, Error::AssetManifest(_)));
    }

    #[test]
    fn asset_startup_builder_methods_fail_when_selected_lane_is_missing() {
        let fret_app_err = match FretApp::new("builder-view-missing-asset-startup-packaged")
            .asset_startup(
                crate::assets::AssetStartupMode::Packaged,
                crate::assets::AssetStartupPlan::new().development_dir("assets"),
            )
            .view::<SmokeView>()
        {
            Ok(_) => panic!("missing packaged lane should fail on fret app builder path"),
            Err(err) => err,
        };
        assert!(matches!(fret_app_err, Error::AssetStartup(_)));

        let ui_builder_err = match FretApp::new("builder-view-missing-asset-startup-dev")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_startup(
                AssetBundleId::app("builder-view-missing-asset-startup-dev"),
                crate::assets::AssetStartupMode::Development,
                crate::assets::AssetStartupPlan::new().packaged_entries([StaticAssetEntry::new(
                    "images/logo.png",
                    AssetRevision(1),
                    b"builder-bytes",
                )]),
            ) {
            Ok(_) => panic!("missing development lane should fail on ui app builder path"),
            Err(err) => err,
        };
        assert!(matches!(ui_builder_err, Error::AssetStartup(_)));
    }

    #[test]
    fn app_builder_view_smoke_uses_default_main_window() {
        let _builder = AppPreludeFretApp::new("builder-view-default-main-window")
            .minimal_defaults()
            .view::<SmokeView>()
            .expect("view should build")
            .configure(|config| {
                assert_eq!(config.main_window_title, "builder-view-default-main-window");
                assert_eq!(config.main_window_size.width, 960.0);
                assert_eq!(config.main_window_size.height, 720.0);
            });
    }

    #[test]
    fn fret_app_setup_accepts_install_into_app_bundles() {
        let _guard = INSTALL_INTO_APP_TEST_LOCK
            .lock()
            .expect("lock should not be poisoned");
        INSTALL_INTO_APP_CALLS.store(0, Ordering::SeqCst);

        let app = FretApp::new("builder-view-bundle-setup").setup(BundleInstaller);
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 0);

        let _builder = app.view::<SmokeView>().expect("view should build");
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn ui_app_builder_setup_accepts_install_into_app_bundles() {
        let _guard = INSTALL_INTO_APP_TEST_LOCK
            .lock()
            .expect("lock should not be poisoned");
        INSTALL_INTO_APP_CALLS.store(0, Ordering::SeqCst);

        let builder = FretApp::new("builder-view-bundle-setup-ui-builder")
            .view::<SmokeView>()
            .expect("view should build");
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 0);

        let _builder = builder.setup(BundleInstaller);
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn fret_app_setup_accepts_small_tuple_composition() {
        let _guard = INSTALL_INTO_APP_TEST_LOCK
            .lock()
            .expect("lock should not be poisoned");
        INSTALL_INTO_APP_CALLS.store(0, Ordering::SeqCst);

        let app = FretApp::new("builder-view-tuple-setup")
            .setup((install_bundle_step_a, install_bundle_step_b));
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 0);

        let _builder = app.view::<SmokeView>().expect("view should build");
        assert_eq!(INSTALL_INTO_APP_CALLS.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn advanced_ui_app_with_hooks_smoke() {
        let _builder = crate::advanced::ui_app_with_hooks(
            "advanced-ui-app-hooks-smoke",
            init_window_state,
            hook_view,
            configure_hook_driver,
        )
        .with_main_window("Advanced UI App Hooks Smoke", (720.0, 420.0))
        .setup(install_bundle_fixture)
        .install(install)
        .configure(|config| {
            assert_eq!(config.main_window_title, "Advanced UI App Hooks Smoke");
            assert_eq!(config.main_window_size.width, 720.0);
            assert_eq!(config.main_window_size.height, 420.0);
        });
    }
}

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    feature = "desktop",
    feature = "shadcn"
))]
mod tests {
    use std::any::TypeId;

    use crate::{advanced::KernelApp, shadcn};
    use fret_core::{AppWindowId, ColorScheme, WindowMetricsService};
    use fret_ui::{Theme, UiTree};

    #[test]
    fn shadcn_auto_theme_middleware_reacts_to_window_metrics() {
        let mut app = KernelApp::new();
        shadcn::app::install(&mut app);

        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        app.with_global_mut(WindowMetricsService::default, |svc, _app| {
            svc.set_color_scheme(window, Some(ColorScheme::Dark));
        });

        let mut ui = UiTree::<KernelApp>::default();
        let mut state = ();

        let before_bg = Theme::global(&app).colors.surface_background;
        let before_rev = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[],
        );

        assert_eq!(Theme::global(&app).revision(), before_rev);
        assert_eq!(Theme::global(&app).colors.surface_background, before_bg);

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_ne!(Theme::global(&app).colors.surface_background, before_bg);
        let rev_after = Theme::global(&app).revision();

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_eq!(Theme::global(&app).revision(), rev_after);
    }

    #[cfg(feature = "editor")]
    #[test]
    fn shadcn_auto_theme_middleware_reapplies_installed_editor_preset_once() {
        let mut app = KernelApp::new();
        shadcn::app::install_with_theme(
            &mut app,
            shadcn::themes::ShadcnBaseColor::Slate,
            shadcn::themes::ShadcnColorScheme::Dark,
        );
        fret_ui_editor::theme::install_editor_theme_preset_v1(
            &mut app,
            fret_ui_editor::theme::EditorThemePresetV1::Default,
        );

        let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
        app.with_global_mut(WindowMetricsService::default, |svc, _app| {
            svc.set_color_scheme(window, Some(ColorScheme::Light));
        });

        let mut ui = UiTree::<KernelApp>::default();
        let mut state = ();
        let editor_field_bg = Theme::global(&app).color_by_key("component.text_field.bg");
        let host_bg_before = Theme::global(&app).colors.surface_background;

        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_ne!(
            Theme::global(&app).colors.surface_background,
            host_bg_before
        );
        assert_eq!(
            Theme::global(&app).color_by_key("component.text_field.bg"),
            editor_field_bg
        );

        let rev_after = Theme::global(&app).revision();
        super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
            &mut app,
            window,
            &mut ui,
            &mut state,
            &[TypeId::of::<WindowMetricsService>()],
        );

        assert_eq!(Theme::global(&app).revision(), rev_after);
    }
}

#[cfg(test)]
mod authoring_surface_policy_tests {
    const APP_ENTRY_RS: &str = include_str!("app_entry.rs");
    const ACTIONS_RS: &str = include_str!("actions.rs");
    const CARGO_TOML: &str = include_str!("../Cargo.toml");
    const INTEROP_RS: &str = include_str!("interop.rs");
    const ROOT_README: &str = include_str!("../../../README.md");
    const DOCS_README: &str = include_str!("../../../docs/README.md");
    const FIRST_HOUR: &str = include_str!("../../../docs/first-hour.md");
    const TODO_APP_GOLDEN_PATH: &str =
        include_str!("../../../docs/examples/todo-app-golden-path.md");
    const AUTHORING_GOLDEN_PATH_V2: &str =
        include_str!("../../../docs/authoring-golden-path-v2.md");
    const COMPONENT_AUTHOR_GUIDE: &str = include_str!("../../../docs/component-author-guide.md");
    const SHADCN_DECLARATIVE_PROGRESS: &str =
        include_str!("../../../docs/shadcn-declarative-progress.md");
    const AUTHORING_SURFACE_TARGET_INTERFACE_STATE: &str = include_str!(
        "../../../docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md"
    );
    const CRATE_README: &str = include_str!("../README.md");
    const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");
    const ECOSYSTEM_INSTALLER_COMPOSITION: &str = include_str!(
        "../../../docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md"
    );
    const INTEGRATING_TOKIO_AND_REQWEST: &str =
        include_str!("../../../docs/integrating-tokio-and-reqwest.md");
    const INTEGRATING_SQLITE_AND_SQLX: &str =
        include_str!("../../../docs/integrating-sqlite-and-sqlx.md");
    const FEARLESS_REFACTORING: &str = include_str!("../../../docs/fearless-refactoring.md");
    const ACTION_FIRST_MIGRATION_GUIDE: &str = include_str!(
        "../../../docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md"
    );
    const SHADCN_SELECT_V4_USAGE: &str = include_str!(
        "../../../docs/workstreams/shadcn-part-surface-alignment-v1/SELECT_V4_USAGE.md"
    );
    const SHADCN_COMBOBOX_V4_USAGE: &str = include_str!(
        "../../../docs/workstreams/shadcn-part-surface-alignment-v1/COMBOBOX_V4_USAGE.md"
    );
    const APP_ENTRY_BUILDER_DESIGN: &str =
        include_str!("../../../docs/workstreams/app-entry-builder-v1/DESIGN.md");
    const APP_ENTRY_BUILDER_TODO: &str =
        include_str!("../../../docs/workstreams/app-entry-builder-v1/TODO.md");
    const AUTHORING_SURFACE_MIGRATION_MATRIX: &str = include_str!(
        "../../../docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MIGRATION_MATRIX.md"
    );
    const LIB_RS: &str = include_str!("lib.rs");
    const VIEW_RS: &str = include_str!("view.rs");

    fn crate_rustdoc() -> String {
        LIB_RS
            .lines()
            .filter(|line| line.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn app_prelude_source() -> &'static str {
        let app_start = LIB_RS
            .find("pub mod app {")
            .expect("app module should exist in fret facade");
        let prelude_start = LIB_RS[app_start..]
            .find("pub mod prelude {")
            .map(|offset| app_start + offset)
            .expect("app prelude should exist in fret facade");
        let app_tail_start = LIB_RS[prelude_start..]
            .find("/// Explicit bridge for app-facing widgets that only expose `on_activate(...)`.")
            .map(|offset| prelude_start + offset)
            .expect("app surface tail marker should exist in fret facade");
        &LIB_RS[prelude_start..app_tail_start]
    }

    fn ui_app_builder_impl_source() -> &'static str {
        let start = LIB_RS
            .find("impl<S: 'static> UiAppBuilder<S> {")
            .expect("UiAppBuilder impl should exist in fret facade");
        let end = LIB_RS
            .find("#[cfg(all(not(target_arch = \"wasm32\"), feature = \"desktop\"))]\npub(crate) fn apply_desktop_defaults_with")
            .expect("UiAppBuilder impl end marker should exist in fret facade");
        &LIB_RS[start..end]
    }

    fn crate_public_surface_source() -> &'static str {
        let tests_start = LIB_RS.find("#[cfg(test)]").unwrap_or(LIB_RS.len());
        &LIB_RS[..tests_start]
    }

    fn root_surface_header_source() -> &'static str {
        let app_start = LIB_RS
            .find("/// App-facing imports for ordinary Fret application code.")
            .expect("app module marker should exist in fret facade");
        &LIB_RS[..app_start]
    }

    fn component_prelude_source() -> &'static str {
        let component_start = LIB_RS
            .find("/// Component-author imports for reusable, portable UI crates.")
            .expect("component module marker should exist in fret facade");
        let selector_start = LIB_RS
            .find("/// Optional selector integration surface for app code.")
            .expect("selector module marker should exist in fret facade");
        &LIB_RS[component_start..selector_start]
    }

    fn selector_surface_source() -> &'static str {
        let selector_start = LIB_RS
            .find("/// Optional selector integration surface for app code.")
            .expect("selector module marker should exist in fret facade");
        let query_start = LIB_RS
            .find("/// Optional query integration surface for app code.")
            .expect("query module marker should exist in fret facade");
        &LIB_RS[selector_start..query_start]
    }

    fn query_surface_source() -> &'static str {
        let query_start = LIB_RS
            .find("/// Optional query integration surface for app code.")
            .expect("query module marker should exist in fret facade");
        let router_start = LIB_RS
            .find("/// Optional router integration surface for app code.")
            .expect("router module marker should exist in fret facade");
        &LIB_RS[query_start..router_start]
    }

    fn advanced_prelude_source() -> &'static str {
        let advanced_start = LIB_RS
            .find("/// Explicit advanced/manual-assembly imports for power users and integration code.")
            .expect("advanced module marker should exist in fret facade");
        let error_start = LIB_RS
            .find("#[derive(Debug, thiserror::Error)]")
            .expect("error type marker should exist in fret facade");
        &LIB_RS[advanced_start..error_start]
    }

    fn app_prelude_exports_symbol(symbol: &str) -> bool {
        app_prelude_source()
            .split(';')
            .filter(|statement| statement.contains("pub use "))
            .any(|statement| statement_exports_symbol(statement, symbol))
    }

    fn advanced_prelude_exports_symbol(symbol: &str) -> bool {
        advanced_prelude_source()
            .split(';')
            .filter(|statement| statement.contains("pub use "))
            .any(|statement| statement_exports_symbol(statement, symbol))
    }

    fn component_prelude_exports_symbol(symbol: &str) -> bool {
        component_prelude_source()
            .split(';')
            .filter(|statement| statement.contains("pub use "))
            .any(|statement| statement_exports_symbol(statement, symbol))
    }

    fn statement_exports_symbol(statement: &str, symbol: &str) -> bool {
        let Some(pub_use_start) = statement.find("pub use ") else {
            return false;
        };
        let statement = &statement[pub_use_start + "pub use ".len()..];

        if let Some((_, items)) = statement.rsplit_once("::{") {
            let items = items.trim_end_matches('}');
            return items
                .split(',')
                .filter_map(exported_symbol_name)
                .any(|exported| exported == symbol);
        }

        exported_symbol_name(statement).is_some_and(|exported| exported == symbol)
    }

    fn exported_symbol_name(item: &str) -> Option<&str> {
        let item = item.trim();
        if item.is_empty() {
            return None;
        }

        if let Some((_, alias)) = item.rsplit_once(" as ") {
            let alias = alias.trim();
            return (alias != "_").then_some(alias);
        }

        let exported = item.rsplit("::").next()?.trim();
        (exported != "_").then_some(exported)
    }

    fn exported_symbol_names(source: &str) -> std::collections::BTreeSet<String> {
        let mut exported = std::collections::BTreeSet::new();

        for statement in source
            .split(';')
            .filter(|statement| statement.contains("pub use "))
        {
            let Some(pub_use_start) = statement.find("pub use ") else {
                continue;
            };
            let statement = &statement[pub_use_start + "pub use ".len()..];

            if let Some((_, items)) = statement.rsplit_once("::{") {
                let items = items.trim_end_matches('}');
                for name in items.split(',').filter_map(exported_symbol_name) {
                    exported.insert(name.to_owned());
                }
                continue;
            }

            if let Some(name) = exported_symbol_name(statement) {
                exported.insert(name.to_owned());
            }
        }

        exported
    }

    fn markdown_table_row<'a>(doc: &'a str, label: &str) -> &'a str {
        doc.lines()
            .find(|line| line.starts_with('|') && line.contains(label))
            .unwrap_or_else(|| panic!("expected markdown table row containing `{label}`"))
    }

    #[test]
    fn readme_prefers_view_entry_and_omits_ui_bridge() {
        assert!(CRATE_README.contains(
            "App authors (default recommendation): `fret::FretApp::new(...).window(...).view::<V>()?`"
        ));
        assert!(CRATE_README.contains("`state`: enable selector/query helpers on `AppUi`"));
        assert!(CRATE_README.contains("`fret::style::{...}`"));
        assert!(CRATE_README.contains("`fret::icons::{icon, IconId}`"));
        assert!(CRATE_README.contains("`fret::semantics::SemanticsRole`"));
        assert!(CRATE_README.contains("`fret::env::{...}`"));
        assert!(CRATE_README.contains("`fret::assets::{...}`"));
        assert!(CRATE_README.contains("`AssetBundleId::app(...)`"));
        assert!(CRATE_README.contains("`AssetBundleId::package(...)`"));
        assert!(CRATE_README.contains("`AssetLocator::bundle(...)`"));
        assert!(CRATE_README.contains("`FretApp::asset_dir(...)`"));
        assert!(CRATE_README.contains("`UiAppBuilder::with_asset_dir(...)`"));
        assert!(CRATE_README.contains("`fret::assets::register_file_bundle_dir(...)`"));
        assert!(CRATE_README.contains("`FretApp::asset_manifest(...)`"));
        assert!(CRATE_README.contains("`UiAppBuilder::with_asset_manifest(...)`"));
        assert!(CRATE_README.contains("`fret::assets::register_file_manifest(...)`"));
        assert!(!CRATE_README.contains(".run_view::<"));
        assert!(!CRATE_README.contains(".install_app("));
        assert!(!CRATE_README.contains("`fret_runtime::register_bundle_asset_entries(...)`"));
        assert!(!CRATE_README.contains("fret::FretApp::new(...).window(...).ui(...)?"));
        assert!(!CRATE_README.contains("currently backed by `ViewCx`"));
    }

    #[test]
    fn root_readme_and_golden_path_prefer_builder_then_run() {
        assert!(ROOT_README.contains("use fret::style::Space;"));
        assert!(ROOT_README.contains(".view::<TodoView>()?"));
        assert!(ROOT_README.contains(".run()"));
        assert!(!ROOT_README.contains(".run_view::<"));

        assert!(TODO_APP_GOLDEN_PATH.contains(".view::<TodoView>()?"));
        assert!(TODO_APP_GOLDEN_PATH.contains(".run()"));
        assert!(TODO_APP_GOLDEN_PATH.contains("fn install_todo_app(app: &mut App) {"));
        assert!(TODO_APP_GOLDEN_PATH.contains(".setup(install_todo_app)"));
        assert!(!TODO_APP_GOLDEN_PATH.contains("fn install_app(app: &mut App) {"));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".run_view::<"));
    }

    #[test]
    fn readme_keeps_advanced_builder_hooks_off_default_surface() {
        assert!(CRATE_README.contains("`fret::advanced::FretAppAdvancedExt::install(...)`"));
        assert!(CRATE_README.contains(
            "`fret::advanced::UiAppBuilderAdvancedExt::{install(...), on_gpu_ready(...), install_custom_effects(...)}`"
        ));
        assert!(!CRATE_README.contains("`UiAppBuilder::on_gpu_ready(...)`"));
        assert!(!CRATE_README.contains("`UiAppBuilder::install_custom_effects(...)`"));
    }

    #[test]
    fn readme_and_rustdoc_quarantine_compat_runner_under_advanced_interop() {
        let public_surface = crate_public_surface_source();
        let advanced_surface = advanced_prelude_source();
        let rustdoc = crate_rustdoc();

        assert!(
            CRATE_README.contains("`fret::advanced::interop::run_native_with_compat_driver(...)`")
        );
        assert!(rustdoc.contains("`fret::advanced::interop::run_native_with_compat_driver(...)`"));
        assert!(!public_surface.contains("pub fn run_native_with_compat_driver("));
        assert!(!public_surface.contains("pub mod interop;"));
        assert!(advanced_surface.contains("pub mod interop {"));
        assert!(
            advanced_surface.contains("pub use crate::interop::run_native_with_compat_driver;")
        );
        assert!(INTEROP_RS.contains("pub fn run_native_with_compat_driver<"));
    }

    #[test]
    fn readme_and_rustdoc_quarantine_fn_driver_helpers_under_advanced() {
        let public_surface = crate_public_surface_source();
        let rustdoc = crate_rustdoc();

        assert!(CRATE_README.contains("`fret::advanced::run_native_with_fn_driver(...)`"));
        assert!(
            CRATE_README.contains("`fret::advanced::run_native_with_fn_driver_with_hooks(...)`")
        );
        assert!(
            CRATE_README.contains("`fret::advanced::run_native_with_configured_fn_driver(...)`")
        );
        assert!(rustdoc.contains("`fret::advanced::run_native_with_fn_driver(...)`"));
        assert!(rustdoc.contains("`fret::advanced::run_native_with_fn_driver_with_hooks(...)`"));
        assert!(rustdoc.contains("`fret::advanced::run_native_with_configured_fn_driver(...)`"));
        assert!(!public_surface.contains("pub fn run_native_with_fn_driver("));
        assert!(!public_surface.contains("pub fn run_native_with_fn_driver_with_hooks("));
        assert!(!public_surface.contains("pub fn run_native_with_configured_fn_driver("));
        assert!(LIB_RS.contains("pub fn run_native_with_fn_driver<D: 'static, S: 'static>("));
        assert!(
            LIB_RS.contains("pub fn run_native_with_fn_driver_with_hooks<D: 'static, S: 'static>(")
        );
        assert!(
            LIB_RS.contains("pub fn run_native_with_configured_fn_driver<D: 'static, S: 'static>(")
        );
    }

    #[test]
    fn readme_and_rustdoc_expose_install_into_app_as_explicit_bundle_seam() {
        assert!(CRATE_README.contains("`fret::integration::InstallIntoApp`"));
        assert!(CRATE_README.contains("`.setup((install_a, install_b))`"));
        assert!(CRATE_README.contains("keep `.setup(...)` on named installer"));
        assert!(CRATE_README.contains("reserve `.setup_with(...)`"));

        let rustdoc = crate_rustdoc();
        let public_surface = crate_public_surface_source();
        assert!(rustdoc.contains("`fret::integration::InstallIntoApp`"));
        assert!(rustdoc.contains("`.setup((install_a, install_b))`"));
        assert!(rustdoc.contains("named installer functions to `.setup(...)`"));
        assert!(rustdoc.contains("`UiAppBuilder::setup_with(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiAppBuilder::setup_with(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("should still avoid `.setup(|app| ...)`"));
        assert!(public_surface.contains("pub mod integration;"));
        assert!(!app_prelude_exports_symbol("InstallIntoApp"));
    }

    #[test]
    fn readme_and_rustdoc_expose_router_as_explicit_optional_surface() {
        assert!(CRATE_README.contains("- `router`: enable the explicit app-level router surface"));
        assert!(
            CRATE_README
                .contains("`fret::router::{app::install, RouterUiStore, RouterOutlet, ...}`")
        );

        let rustdoc = crate_rustdoc();
        let public_surface = crate_public_surface_source();
        assert!(rustdoc.contains(
            "`fret::router::{app::install, RouterUiStore, RouterOutlet, router_link, ...}`"
        ));
        assert!(rustdoc.contains("`RouterUiStore::{back_on_action, forward_on_action}`"));
        assert!(public_surface.contains("pub mod router {"));
        assert!(public_surface.contains("pub mod app {"));
        assert!(public_surface.contains("pub fn install(app: &mut crate::app::App) {"));
        assert!(!public_surface.contains("pub fn install_app(app: &mut crate::app::App) {"));
    }

    #[test]
    fn readme_and_rustdoc_expose_selector_and_query_as_explicit_optional_surfaces() {
        assert!(CRATE_README.contains("`cx.data().selector_layout(...)`"));
        assert!(CRATE_README.contains("raw `cx.data().selector(...)`"));
        assert!(CRATE_README.contains("`handle.read_layout(cx)`"));
        assert!(CRATE_README.contains("`cx.data().invalidate_query(...)`"));
        assert!(CRATE_README.contains("`cx.data().invalidate_query_namespace(...)`"));
        assert!(CRATE_README.contains("`fret::selector::ui::DepsBuilder`"));
        assert!(CRATE_README.contains("`fret::selector::DepsSignature`"));
        assert!(
            CRATE_README
                .contains("`fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}`")
        );

        let rustdoc = crate_rustdoc();
        let selector_surface = selector_surface_source();
        let query_surface = query_surface_source();
        assert!(rustdoc.contains("`fret::selector::ui::DepsBuilder`"));
        assert!(rustdoc.contains("`fret::selector::DepsSignature`"));
        assert!(
            rustdoc.contains("`fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}`")
        );
        assert!(selector_surface.contains("pub mod selector {"));
        assert!(selector_surface.contains("pub mod core {"));
        assert!(selector_surface.contains("pub mod ui {"));
        assert!(!selector_surface.contains("pub use crate::view::LocalDepsBuilderExt;"));
        assert!(selector_surface.contains("pub use fret_selector::{DepsSignature, Selector};"));
        assert!(!selector_surface.contains("pub use fret_selector::ui::DepsBuilder;"));
        assert!(selector_surface.contains("pub use fret_selector::ui::*;"));
        assert!(query_surface.contains("pub mod query {"));
        assert!(query_surface.contains("pub mod core {"));
        assert!(query_surface.contains("pub mod ui {"));
        assert!(query_surface.contains("pub use fret_query::{"));
        assert!(query_surface.contains("QueryKey, QueryPolicy"));
        assert!(query_surface.contains("QueryState,"));
        assert!(!app_prelude_exports_symbol("DepsBuilder"));
        assert!(!app_prelude_exports_symbol("DepsSignature"));
        assert!(!app_prelude_exports_symbol("LocalDepsBuilderExt"));
        assert!(!app_prelude_exports_symbol("QueryKey"));
        assert!(!app_prelude_exports_symbol("QueryPolicy"));
        assert!(!app_prelude_exports_symbol("QueryHandle"));
    }

    #[test]
    fn readme_and_rustdoc_expose_docking_as_explicit_optional_surface() {
        assert!(CRATE_README.contains("- `docking`: enable the explicit advanced docking surface"));
        assert!(
            CRATE_README.contains("`fret::docking::{core::*, DockManager, handle_dock_op, ...}`")
        );

        let rustdoc = crate_rustdoc();
        assert!(rustdoc.contains(
            "//! - enable `docking` for `fret::docking::{core::*, DockManager, handle_dock_op, ...}`"
        ));
        assert!(LIB_RS.contains("pub mod docking {"));
        assert!(
            LIB_RS.contains("/// Raw docking core contracts for advanced or fully explicit use.")
        );
        assert!(LIB_RS.contains(
            "/// Raw docking runtime integration helpers for advanced or fully explicit use."
        ));
    }

    #[test]
    fn readme_and_rustdoc_expose_explicit_assets_surface() {
        assert!(CRATE_README.contains("`fret::assets::{...}`"));
        assert!(CRATE_README.contains("`AssetStartupPlan`"));
        assert!(CRATE_README.contains("`AssetStartupMode`"));
        assert!(CRATE_README.contains("`AssetBundleId::app(...)`"));
        assert!(CRATE_README.contains("`AssetBundleId::package(...)`"));
        assert!(CRATE_README.contains("`AssetLocator::bundle(...)`"));
        assert!(CRATE_README.contains("`register_bundle_entries(...)`"));
        assert!(CRATE_README.contains("`FretApp::asset_dir(...)`"));
        assert!(CRATE_README.contains("`UiAppBuilder::with_asset_dir(...)`"));
        assert!(CRATE_README.contains("`fret::assets::register_file_bundle_dir(...)`"));
        assert!(CRATE_README.contains("`FretApp::asset_manifest(...)`"));
        assert!(CRATE_README.contains("`UiAppBuilder::with_asset_manifest(...)`"));
        assert!(CRATE_README.contains("`fret::assets::register_file_manifest(...)`"));
        assert!(CRATE_README.contains(
            "`fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`"
        ));
        assert!(
            CRATE_README.contains(
                "`fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`"
            )
        );

        let rustdoc = crate_rustdoc();
        let public_surface = crate_public_surface_source();
        assert!(rustdoc.contains(
            "`fret::assets::{AssetBundleId, AssetLocator, AssetRequest, StaticAssetEntry, ...}`"
        ));
        assert!(rustdoc.contains("`AssetStartupPlan`"));
        assert!(rustdoc.contains("`AssetStartupMode`"));
        assert!(rustdoc.contains("`AssetBundleId::app(...)`"));
        assert!(rustdoc.contains("`AssetBundleId::package(...)`"));
        assert!(rustdoc.contains("`FretApp::asset_dir(...)`"));
        assert!(rustdoc.contains("`UiAppBuilder::with_asset_dir(...)`"));
        assert!(rustdoc.contains("`register_file_bundle_dir(...)`"));
        assert!(rustdoc.contains("`FretApp::asset_manifest(...)`"));
        assert!(rustdoc.contains("`UiAppBuilder::with_asset_manifest(...)`"));
        assert!(rustdoc.contains("`register_file_manifest(...)`"));
        assert!(rustdoc.contains("`AssetLocator::file(...)`"));
        assert!(rustdoc.contains("`AssetLocator::url(...)`"));
        assert!(rustdoc.contains(
            "`fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`"
        ));
        assert!(
            rustdoc.contains(
                "`fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`"
            )
        );
        assert!(public_surface.contains("pub mod assets {"));
        assert!(!public_surface.contains("pub use fret_runtime::register_bundle_asset_entries;"));
    }

    #[test]
    fn readme_and_rustdoc_expose_editor_as_opt_in_integration_feature() {
        assert!(CRATE_README.contains(
            "- `editor`: keep installed `fret-ui-editor` presets resilient to `FretApp` shadcn theme resets."
        ));

        let rustdoc = crate_rustdoc();
        assert!(rustdoc.contains(
            "//! - enable `editor` for opt-in app-level replay of installed `fret-ui-editor` presets"
        ));
    }

    #[test]
    fn readme_and_rustdoc_expose_curated_shadcn_surface() {
        assert!(CRATE_README.contains("`fret::shadcn`"));
        assert!(CRATE_README.contains("`shadcn::app::install(...)`"));
        assert!(CRATE_README.contains("`shadcn::themes::apply_shadcn_new_york(...)`"));
        assert!(CRATE_README.contains("`shadcn::raw::*`"));
        assert!(CRATE_README.contains("only first-contact component-family lane"));
        assert!(CRATE_README.contains("`shadcn::app::*` and `shadcn::themes::*` are setup lanes"));
        assert!(CRATE_README.contains("`fret::shadcn::raw::advanced::*`"));
        assert!(CRATE_README.contains("`fret_ui_shadcn::advanced::*`"));

        let rustdoc = crate_rustdoc();
        let public_surface = crate_public_surface_source();
        assert!(rustdoc.contains(
            "//! - use `fret::shadcn::{..., app::install, themes::apply_shadcn_new_york, raw::*}`"
        ));
        assert!(rustdoc.contains("`shadcn::app::*` and `shadcn::themes::*` are setup lanes"));
        assert!(rustdoc.contains("`fret::shadcn::raw::advanced::*`"));
        assert!(public_surface.contains("pub use fret_ui_shadcn::facade as shadcn;"));
        assert!(!public_surface.contains("pub use fret_ui_shadcn as shadcn;"));
    }

    #[test]
    fn crate_docs_only_teach_view_entry() {
        let rustdoc = crate_rustdoc();
        assert!(rustdoc.contains(
            "//! - `fret::FretApp::new(...).window(...).view::<V>()?` is the recommended app-author path."
        ));
        assert!(rustdoc.contains("use fret::app::prelude::*;"));
        assert!(rustdoc.contains("FretApp::new(\"hello\")"));
        assert!(rustdoc.contains("&mut App"));
        assert!(rustdoc.contains("WindowId"));
        assert!(!rustdoc.contains("AppWindowId"));
        assert!(!rustdoc.contains("KernelApp"));
        assert!(rustdoc.contains("AppUi<'_, '_>"));
        assert!(!rustdoc.contains("AppUi<'_, '_, KernelApp>"));
        assert!(!rustdoc.contains(".window(...).ui(...)?"));
    }

    #[test]
    fn repo_docs_prefer_app_ui_language_for_golden_path() {
        assert!(DOCS_README.contains("`ecosystem/fret` (`View`, `AppUi`, `fret::actions!`)"));
        assert!(DOCS_README.contains("`on_payload_action_notify`"));
        assert!(!DOCS_README.contains("`payload_locals::<A>(...)`"));
        assert!(!DOCS_README.contains("`ecosystem/fret` (`View`, `ViewCx`, `fret::actions!`)"));
        assert!(!DOCS_README.contains("ViewCx::on_payload_action*"));
    }

    #[test]
    fn docs_index_and_first_hour_stay_on_default_app_surface() {
        assert!(DOCS_README.contains("`use fret::app::prelude::*;`"));
        assert!(DOCS_README.contains("`FretApp::new(...).window(...).view::<MyView>()?.run()`"));
        assert!(DOCS_README.contains("`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`"));
        assert!(!DOCS_README.contains("`.dispatch::<A>()`"));
        assert!(!DOCS_README.contains("`.dispatch_payload::<A>(...)`"));
        assert!(!DOCS_README.contains(".on_activate(cx.actions().dispatch::<"));
        assert!(!DOCS_README.contains(".on_activate(cx.actions().dispatch_payload::<"));
        assert!(!DOCS_README.contains(".on_activate(cx.actions().listener("));
        assert!(!DOCS_README.contains("run_view::<"));
        assert!(!DOCS_README.contains("ViewCx::"));

        assert!(FIRST_HOUR.contains("`use fret::app::prelude::*;`"));
        assert!(FIRST_HOUR.contains(
            "`FretApp::new(\"my-simple-todo\").window(\"my-simple-todo\", (...)).view::<TodoView>()?.run()`"
        ));
        assert!(FIRST_HOUR.contains("`fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`"));
        assert!(FIRST_HOUR.contains("`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`"));
        assert!(FIRST_HOUR.contains("`.action(...)` / `.action_payload(...)` / `.listen(...)`"));
        assert!(!FIRST_HOUR.contains("`.dispatch::<A>()`"));
        assert!(!FIRST_HOUR.contains("`.dispatch_payload::<A>(...)`"));
        assert!(FIRST_HOUR.contains("`ui::single(cx, page(...))`"));
        assert!(FIRST_HOUR.contains("When observing tracked state in views:"));
        assert!(FIRST_HOUR.contains(
            "Treat explicit `.into_element(cx)` / `AnyElement` seams as advanced helper or interop boundaries"
        ));
        assert!(FIRST_HOUR.contains("use fret::children::UiElementSinkExt as _;"));
        assert!(!FIRST_HOUR.contains("run_view::<"));
        assert!(!FIRST_HOUR.contains("ViewCx::"));
        assert!(!FIRST_HOUR.contains("When observing models (via `cx.watch_model(...)`):"));
        assert!(
            !FIRST_HOUR
                .contains("Convert into `AnyElement` at the boundary via `.into_element(cx)`.")
        );
        assert!(!FIRST_HOUR.contains("cx.watch_model(&models.clicks)"));
        assert!(!FIRST_HOUR.contains("`fret_ui_shadcn::prelude::*`"));
    }

    #[test]
    fn app_entry_workstream_docs_match_the_shipped_builder_surface() {
        assert!(
            APP_ENTRY_BUILDER_DESIGN.contains("`fret::FretApp::new(...).window(...).view::<V>()?`")
        );
        assert!(
            APP_ENTRY_BUILDER_DESIGN
                .contains("`fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?`")
        );
        assert!(APP_ENTRY_BUILDER_DESIGN.contains(
            "`run_view::<V>()` / `run_view_with_hooks::<V>(...)` were also removed from `FretApp`"
        ));
        assert!(
            APP_ENTRY_BUILDER_DESIGN
                .contains("Execution stays on the returned `UiAppBuilder` via `.run()`")
        );
        assert!(
            !APP_ENTRY_BUILDER_DESIGN.contains(
                "- `view::<V>()`\n- `view_with_hooks::<V>(configure)`\n- `run_view::<V>()` / `run_view_with_hooks::<V>(...)`"
            )
        );

        assert!(APP_ENTRY_BUILDER_TODO.contains(
            "- [x] Delete `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from `FretApp` before release."
        ));
        assert!(
            !APP_ENTRY_BUILDER_TODO
                .contains("- [x] `run_view::<V>()` / `run_view_with_hooks::<V>(...)`")
        );
    }

    #[test]
    fn usage_docs_prefer_grouped_app_ui_actions() {
        assert!(CRATE_USAGE_GUIDE.contains("start with `View` + `AppUi` + typed actions"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().locals::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().models::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().transient::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::app::LocalState`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::actions::CommandId`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::style::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::style::ThemeSnapshot`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::icons::{icon, IconId}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::semantics::SemanticsRole`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::children::UiElementSinkExt as _`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::actions::ElementCommandGatingExt as _`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::assets::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan`"));
        assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupMode`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::selector::ui::DepsBuilder`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::selector::DepsSignature`"));
        assert!(
            CRATE_USAGE_GUIDE.contains("`fret::query::{QueryKey, QueryPolicy, QueryState, ...}`")
        );
        assert!(CRATE_USAGE_GUIDE.contains("`AssetBundleId::app(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`AssetBundleId::package(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`AssetLocator::bundle(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`register_bundle_entries(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`FretApp::asset_dir(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_asset_dir(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::assets::register_file_bundle_dir(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`FretApp::asset_manifest(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_asset_manifest(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::assets::register_file_manifest(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains(
            "`fret-ui-assets::ui::ImageSourceElementContextExt::use_image_source_state_from_asset_request(...)`"
        ));
        assert!(
            CRATE_USAGE_GUIDE
                .contains(
                    "`fret-ui-assets::ui::SvgAssetElementContextExt::svg_source_state_from_asset_request(...)`"
                )
        );
        assert!(CRATE_USAGE_GUIDE.contains("`widget.action(act::Save)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`widget.action_payload(act::Remove, payload)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`widget.listen(|host, acx| { ... })`"));
        assert!(CRATE_USAGE_GUIDE.contains("`use fret::app::AppActivateExt as _;`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().action(act::Save)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().action_payload(act::Remove, payload)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().listen(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`widget.dispatch::<A>()`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`widget.dispatch_payload::<A>(payload)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().dispatch::<A>()`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().dispatch_payload::<A>(payload)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiCxActionsExt`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().selector_layout(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("raw `cx.data().selector(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`handle.read_layout(cx)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query_namespace(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("ViewCx::use_selector"));
        assert!(!CRATE_USAGE_GUIDE.contains("ViewCx::use_query"));
    }

    #[test]
    fn authoring_surface_matrix_keeps_builder_setup_and_async_docs_closed() {
        let builder_row = markdown_table_row(
            AUTHORING_SURFACE_MIGRATION_MATRIX,
            "Default builder setup seam",
        );
        assert!(builder_row.contains("| Migrated |"));

        let async_docs_row =
            markdown_table_row(AUTHORING_SURFACE_MIGRATION_MATRIX, "async integration docs");
        assert!(async_docs_row.contains("| Migrated |"));

        let component_row =
            markdown_table_row(AUTHORING_SURFACE_MIGRATION_MATRIX, "Component prelude");
        assert!(component_row.contains("| Migrated |"));

        let advanced_row =
            markdown_table_row(AUTHORING_SURFACE_MIGRATION_MATRIX, "Advanced imports");
        assert!(advanced_row.contains("| Deleted |"));

        let app_activate_bridge_row = markdown_table_row(
            AUTHORING_SURFACE_MIGRATION_MATRIX,
            "`AppActivateExt` bridge",
        );
        assert!(app_activate_bridge_row.contains("| Migrated |"));
    }

    #[test]
    fn usage_and_component_docs_keep_app_activate_surface_narrow() {
        assert!(CRATE_USAGE_GUIDE.contains("`fret::app::AppActivateSurface` / `AppActivateExt`"));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("activation-only widgets that expose the standard `OnActivate` slot")
        );
        assert!(CRATE_USAGE_GUIDE.contains("Typed payload/context"));
        assert!(CRATE_USAGE_GUIDE.contains("callbacks remain component-owned surfaces"));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::Button`"));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::SidebarMenuButton`"));
        assert!(CRATE_USAGE_GUIDE.contains("`WorkflowControlsButton`"));
        assert!(CRATE_USAGE_GUIDE.contains("`ConfirmationAction`"));
        assert!(CRATE_USAGE_GUIDE.contains("native `.action(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`Attachment`"));
        assert!(CRATE_USAGE_GUIDE.contains("`QueueItemAction`"));
        assert!(CRATE_USAGE_GUIDE.contains("`Test`"));
        assert!(CRATE_USAGE_GUIDE.contains("`FileTreeAction`"));
        assert!(CRATE_USAGE_GUIDE.contains("`Suggestion`"));
        assert!(CRATE_USAGE_GUIDE.contains("`MessageBranch`"));
        assert!(CRATE_USAGE_GUIDE.contains("first-party default widget bridge table is"));
        assert!(CRATE_USAGE_GUIDE.contains("intentionally empty"));
        assert!(
            COMPONENT_AUTHOR_GUIDE.contains("typed domain callbacks into `AppActivateSurface`")
        );
        assert!(
            COMPONENT_AUTHOR_GUIDE
                .contains("parallel `AppActionCxSurface` / `AppActionCxExt` family")
        );
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`Attachment`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`QueueItemAction`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`Test`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`FileTreeAction`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`Suggestion`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`MessageBranch`"));
    }

    #[test]
    fn authoring_docs_prefer_grouped_app_ui_data_helpers() {
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().selector_layout(...)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().selector(deps, compute)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().query(...)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`handle.read_layout(cx)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().invalidate_query(...)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`ui::single(cx, child)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`.action(act::Save)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains(".action_payload(act::RemoveTodo, todo.id);"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`.listen(|host, acx| { ... })`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`use fret::app::AppActivateExt as _;`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.actions().action(act::Save)`"));
        assert!(
            AUTHORING_GOLDEN_PATH_V2
                .contains("`cx.actions().action_payload(act::RemoveTodo, todo.id)`")
        );
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`.dispatch::<A>()`"));
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`.dispatch_payload::<A>(payload)`"));
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`cx.use_selector(...)`"));
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`cx.use_query(...)`"));
    }

    #[test]
    fn integration_docs_prefer_grouped_query_helpers_for_app_surface() {
        assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async(...)`"));
        assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async_local(...)`"));
        assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("let state = handle.read_layout(cx);"));
        assert!(
            INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().invalidate_query_namespace(...)`")
        );
        assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().query_async(...)`"));
        assert!(
            INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().invalidate_query_namespace(...)`")
        );
    }

    #[test]
    fn usage_docs_expose_router_as_explicit_extension_surface() {
        assert!(CRATE_USAGE_GUIDE.contains("enable `fret`'s `router` feature"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::router::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`back_on_action()`"));
        assert!(CRATE_USAGE_GUIDE.contains("`forward_on_action()`"));
        assert!(CRATE_USAGE_GUIDE.contains("`use fret::advanced::AppUiRawActionExt as _;`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.on_action_notify::<...>(store.back_on_action())`"));
        assert!(CRATE_USAGE_GUIDE.contains("second default app runtime"));
    }

    #[test]
    fn usage_docs_link_ecosystem_trait_budget_and_anti_plugin_posture() {
        assert!(CRATE_USAGE_GUIDE.contains("## Ecosystem author checklist"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::integration::InstallIntoApp`"));
        assert!(CRATE_USAGE_GUIDE.contains("one installer/bundle surface"));
        assert!(CRATE_USAGE_GUIDE.contains("`RouteCodec`"));
        assert!(CRATE_USAGE_GUIDE.contains("`DockPanelFactory`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret-app::Plugin`"));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("`docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`")
        );
    }

    #[test]
    fn usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems() {
        assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(fret_icons_lucide::app::install)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(fret_icons_radix::app::install)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret_icons_lucide::app::install`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret_icons_radix::app::install`"));
        assert!(CRATE_USAGE_GUIDE.contains("`docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`"));
        assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(MyKitBundle)`"));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("`IconRegistry` mutation plus `register_bundle_entries(...)` manually")
        );
        assert!(
            CRATE_USAGE_GUIDE.contains("`fret_ui_assets::app::configure_caches_with_budgets(...)`")
        );
        assert!(CRATE_USAGE_GUIDE.contains(
            "`fret_ui_assets::advanced::{configure_caches_with_ui_services(...), configure_caches_with_ui_services_and_budgets(...)}`"
        ));
        assert!(CRATE_USAGE_GUIDE.contains("`fret_node::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::router::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`BootstrapBuilder::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`FretApp::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_lucide_icons()`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret::router::install_app(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_icons_radix::install_app`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_assets::install_app_with_budgets`"));
        assert!(CRATE_USAGE_GUIDE.contains("generated `Bundle` / `install(app)` /"));
        assert!(CRATE_USAGE_GUIDE.contains("`mount(builder)` surface is usually enough."));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("settings, theme/bootstrap wiring, or multiple generated asset modules")
        );
        assert!(CRATE_USAGE_GUIDE.contains("wrap those low-level"));
        assert!(
            CRATE_USAGE_GUIDE.contains("generated helpers in one named installer/bundle surface")
        );
        assert!(CRATE_USAGE_GUIDE.contains(
            "Prefer `BundleAsset` when the bytes are part of the crate's public lookup story"
        ));
        assert!(CRATE_USAGE_GUIDE.contains("Use `Embedded`"));
        assert!(CRATE_USAGE_GUIDE.contains("owner-scoped bytes"));
        assert!(CRATE_USAGE_GUIDE.contains("public cross-package contract"));
    }

    #[test]
    fn component_author_docs_keep_transitive_icon_and_asset_registration_on_one_bundle_surface() {
        assert!(COMPONENT_AUTHOR_GUIDE.contains(
            "If your crate depends on an icon pack or ships package-owned images/SVGs/fonts"
        ));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("widget code stays on semantic `IconId`s and logical `AssetLocator::bundle(...)` lookups"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("impl InstallIntoApp for MyKitBundle"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("AssetBundleId::package(\"my-kit\")"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`FretApp::setup(MyKitBundle)`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("generated `--surface fret` asset module"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("can remain the app-facing surface"));
        assert!(
            COMPONENT_AUTHOR_GUIDE
                .contains("wrap those low-level generated helpers in one hand-written named")
        );
        assert!(COMPONENT_AUTHOR_GUIDE.contains("installer/bundle surface"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("Prefer `BundleAsset`"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("default public lookup story"));
        assert!(
            COMPONENT_AUTHOR_GUIDE.contains("Use `Embedded` for lower-level owner-scoped bytes")
        );
        assert!(COMPONENT_AUTHOR_GUIDE.contains("crate's public cross-package"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("lookup contract"));
        assert!(
            ECOSYSTEM_INSTALLER_COMPOSITION.contains("the app composes one installer/bundle value")
        );
        assert!(ECOSYSTEM_INSTALLER_COMPOSITION.contains("The app should not usually do this:"));
        assert!(
            ECOSYSTEM_INSTALLER_COMPOSITION
                .contains("### Generated module vs higher-level installer")
        );
        assert!(
            ECOSYSTEM_INSTALLER_COMPOSITION
                .contains("generated modules own low-level byte publication")
        );
        assert!(ECOSYSTEM_INSTALLER_COMPOSITION.contains("### `BundleAsset` vs `Embedded`"));
        assert!(
            ECOSYSTEM_INSTALLER_COMPOSITION.contains("If you are unsure, choose `BundleAsset`.")
        );
    }

    #[test]
    fn component_author_docs_keep_secondary_lanes_explicit() {
        assert!(COMPONENT_AUTHOR_GUIDE.contains("use fret::component::prelude::*;"));
        assert!(COMPONENT_AUTHOR_GUIDE.contains(
            "use fret::env::{container_breakpoints, safe_area_insets, viewport_breakpoints};"
        ));
        assert!(COMPONENT_AUTHOR_GUIDE.contains(
            "use fret::activate::{on_activate, on_activate_notify, on_activate_request_redraw};"
        ));
        assert!(COMPONENT_AUTHOR_GUIDE.contains("`fret::overlay::*`"));
        assert!(
            COMPONENT_AUTHOR_GUIDE
                .contains("`OverlayController`, `OverlayRequest`, `OverlayPresence`")
        );
    }

    #[test]
    fn todo_golden_path_keeps_icon_pack_setup_on_app_install_surface() {
        assert!(TODO_APP_GOLDEN_PATH.contains("`.setup(fret_icons_radix::app::install)`"));
        assert!(TODO_APP_GOLDEN_PATH.contains("`ui::single(cx, page(...))`"));
        assert!(TODO_APP_GOLDEN_PATH.contains("When observing tracked state in views:"));
        assert!(
            TODO_APP_GOLDEN_PATH
                .contains("selector dependencies now stay on\nthe LocalState-first teaching path")
        );
        assert!(!TODO_APP_GOLDEN_PATH.contains("`.dispatch::<A>()`"));
        assert!(!TODO_APP_GOLDEN_PATH.contains("`.dispatch_payload::<A>(...)`"));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".on_activate(cx.actions().dispatch::<"));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".on_activate(cx.actions().dispatch_payload::<"));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".on_activate(cx.actions().listener("));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".register_icon_pack("));
        assert!(!TODO_APP_GOLDEN_PATH.contains("IconRegistry"));
        assert!(!TODO_APP_GOLDEN_PATH.contains("When observing models in views:"));
        assert!(!TODO_APP_GOLDEN_PATH.contains("model handles cloned off those locals"));
    }

    #[test]
    fn usage_docs_expose_docking_as_explicit_extension_surface() {
        assert!(CRATE_USAGE_GUIDE.contains("| Add docking integration | `[\"docking\"]` |"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::docking::{core::*, ...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("enable `fret`'s `docking` feature"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::docking::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("part of `fret::app::prelude::*`"));
    }

    #[test]
    fn usage_docs_expose_editor_as_opt_in_app_integration() {
        assert!(CRATE_USAGE_GUIDE.contains("| Add editor theming integration | `[\"editor\"]` |"));
        assert!(CRATE_USAGE_GUIDE.contains("installed `fret-ui-editor` presets"));
        assert!(CRATE_USAGE_GUIDE.contains("widgets still come from `fret-ui-editor`"));
    }

    #[test]
    fn usage_docs_expose_curated_component_surface() {
        assert!(CRATE_USAGE_GUIDE.contains("`use fret::component::prelude::*;`"));
        assert!(CRATE_USAGE_GUIDE.contains("`ComponentCx`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiBuilder`/`UiPatchTarget`/`IntoUiElement<H>`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::actions::CommandId`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::activate::{on_activate,"));
        assert!(CRATE_USAGE_GUIDE.contains("`use fret::advanced::prelude::*;`"));
        assert!(CRATE_USAGE_GUIDE.contains("advanced-only"));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("without pulling in `FretApp`, `AppUi`, or runner-facing seams")
        );
    }

    #[test]
    fn usage_docs_expose_shadcn_app_surface_as_explicit_submodule() {
        assert!(
            CRATE_USAGE_GUIDE.contains("`use fret_ui_shadcn::{facade as shadcn, prelude::*};`")
        );
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::themes::apply_shadcn_new_york(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("only first-contact component-family discovery"));
        assert!(
            CRATE_USAGE_GUIDE.contains("`shadcn::app::*` and `shadcn::themes::*` are setup lanes")
        );
        assert!(CRATE_USAGE_GUIDE.contains(
            "`fret_ui_shadcn::advanced::{sync_theme_from_environment(...), install_with_ui_services(...)}`"
        ));
        assert!(
            CRATE_USAGE_GUIDE
                .contains("`fret_ui_shadcn::advanced::*` is an implementation/debug lane")
        );
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::raw::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::raw::typography::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::themes::apply_shadcn_new_york(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::app::*` and"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::themes::*` are setup lanes"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::advanced::*`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::install_app(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::shadcn_themes::"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret::shadcn::shadcn_themes::"));
    }

    #[test]
    fn shadcn_docs_keep_advanced_hooks_off_curated_lane() {
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`widget.action(act::Save)`"));
        assert!(
            SHADCN_DECLARATIVE_PROGRESS.contains("`widget.action_payload(act::Remove, payload)`")
        );
        assert!(
            SHADCN_DECLARATIVE_PROGRESS
                .contains("`fret::app::AppActivateSurface` / `AppActivateExt`")
        );
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`use fret::app::AppActivateExt as _;`"));
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`UiCxActionsExt` / `UiCxDataExt`"));
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("first-party"));
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("bridge table is intentionally empty"));
        assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`.dispatch::<A>()`"));
        assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`.dispatch_payload::<A>(payload)`"));
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`fret_ui_shadcn::advanced::*`"));
        assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`shadcn::advanced::*`"));
        assert!(AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret_ui_shadcn::advanced`"));
        assert!(
            AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret::shadcn::raw::advanced::*`")
        );
        assert!(
            AUTHORING_SURFACE_TARGET_INTERFACE_STATE
                .contains("first-party default widget bridge table is intentionally empty")
        );
    }

    #[test]
    fn workstream_docs_teach_curated_direct_shadcn_imports() {
        assert!(
            ACTION_FIRST_MIGRATION_GUIDE
                .contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};")
        );
        assert!(
            SHADCN_SELECT_V4_USAGE.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};")
        );
        assert!(SHADCN_COMBOBOX_V4_USAGE.contains("use fret_ui_shadcn::facade as shadcn;"));
        assert!(!ACTION_FIRST_MIGRATION_GUIDE.contains("use fret_ui_shadcn as shadcn;"));
        assert!(!SHADCN_SELECT_V4_USAGE.contains("use fret_ui_shadcn::{self as shadcn"));
        assert!(!SHADCN_COMBOBOX_V4_USAGE.contains("use fret_ui_shadcn::{"));
    }

    #[test]
    fn fearless_refactoring_docs_distinguish_default_and_advanced_surfaces() {
        assert!(FEARLESS_REFACTORING.contains(
            "`impl View for MyView { fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui { ... } }`"
        ));
        assert!(
            FEARLESS_REFACTORING
                .contains("`fn(&mut ElementContext<'_, App>, &mut State) -> ViewElements`")
        );
        assert!(
            FEARLESS_REFACTORING.contains("Return `Ui` (the app-facing alias over `Elements`)")
        );
        assert!(FEARLESS_REFACTORING.contains("`cx.actions().locals::<A>(...)`"));
        assert!(FEARLESS_REFACTORING.contains("`cx.actions().models::<A>(...)`"));
        assert!(FEARLESS_REFACTORING.contains("`cx.actions().transient::<A>(...)`"));
        assert!(!FEARLESS_REFACTORING.contains("`.dispatch::<A>()`"));
        assert!(!FEARLESS_REFACTORING.contains("`.dispatch_payload::<A>(payload)`"));
        assert!(!FEARLESS_REFACTORING.contains(".on_activate(cx.actions().dispatch::<"));
        assert!(!FEARLESS_REFACTORING.contains(".on_activate(cx.actions().dispatch_payload::<"));
        assert!(!FEARLESS_REFACTORING.contains(".on_activate(cx.actions().listener("));
        assert!(!FEARLESS_REFACTORING.contains("`payload_locals::<A>(...)`"));
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_locals`"));
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_models`"));
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_transient`"));
    }

    #[test]
    fn app_prelude_stays_explicit_instead_of_reexporting_legacy_surface() {
        let app_prelude = app_prelude_source();
        assert!(!app_prelude.contains("pub use crate::prelude::*;"));
        assert!(LIB_RS.contains("pub use crate::view::{AppActivateExt, AppActivateSurface};"));
        assert!(app_prelude.contains("pub use crate::{"));
        assert!(app_prelude.contains("pub use crate::app::App;"));
        assert!(app_prelude_exports_symbol("App"));
        assert!(app_prelude.contains("AppUi"));
        assert!(!app_prelude_exports_symbol("KernelApp"));
        assert!(app_prelude.contains("UiChild"));
        assert!(app_prelude.contains("WindowId"));
        assert!(app_prelude_exports_symbol("Px"));
        assert!(!app_prelude_exports_symbol("LocalState"));
        assert!(!app_prelude_exports_symbol("CommandId"));
        assert!(!app_prelude_exports_symbol("ThemeSnapshot"));
        assert!(!app_prelude_exports_symbol("actions"));
        assert!(!app_prelude_exports_symbol("workspace_menu"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::declarative::icon;"));
        assert!(!app_prelude.contains("pub use crate::view::AppActivateExt as _;"));
        assert!(app_prelude.contains("pub use crate::view::QueryHandleReadExt as _;"));
        assert!(app_prelude.contains("pub use crate::view::TrackedStateExt as _;"));
        assert!(app_prelude.contains("pub use crate::view::UiCxActionsExt as _;"));
        assert!(app_prelude.contains("pub use crate::view::UiCxDataExt as _;"));
        assert!(
            app_prelude.contains("pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;")
        );
        assert!(app_prelude.contains("pub use fret_ui_kit::declarative::UiElementA11yExt as _;"));
        assert!(app_prelude.contains("pub use fret_ui_kit::declarative::UiElementTestIdExt as _;"));
        assert!(app_prelude.contains("pub use fret_ui_kit::StyledExt as _;"));
        assert!(app_prelude.contains("pub use fret_ui_kit::UiExt as _;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::ui::UiElementSinkExt as _;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::declarative::prelude::*;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::IntoUiElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiIntoElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiHostBoundIntoElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiChildIntoElement;"));
        assert!(!app_prelude_exports_symbol("AppActivateExt"));
        assert!(!app_prelude_exports_symbol("QueryHandleReadExt"));
        assert!(
            !app_prelude.contains("pub use crate::view::{AppActivateExt, AppActivateSurface};")
        );
        assert!(!app_prelude_exports_symbol("TrackedStateExt"));
        assert!(!app_prelude_exports_symbol("AnyElementSemanticsExt"));
        assert!(!app_prelude_exports_symbol("ElementContextThemeExt"));
        assert!(!app_prelude_exports_symbol("UiElementA11yExt"));
        assert!(!app_prelude_exports_symbol("UiElementKeyContextExt"));
        assert!(!app_prelude_exports_symbol("UiElementTestIdExt"));
        assert!(
            !app_prelude.contains("pub use fret_ui_kit::command::ElementCommandGatingExt as _;")
        );
        assert!(
            !app_prelude.contains("pub use fret_ui_kit::declarative::ElementContextThemeExt as _;")
        );
        assert!(
            !app_prelude.contains("pub use fret_ui_kit::declarative::UiElementKeyContextExt as _;")
        );
        assert!(!app_prelude_exports_symbol("StyledExt"));
        assert!(!app_prelude_exports_symbol("UiExt"));
        assert!(!app_prelude_exports_symbol("icon"));
        assert!(!app_prelude_exports_symbol("IconId"));
        assert!(!app_prelude_exports_symbol("Theme"));
        assert!(!app_prelude_exports_symbol("ChromeRefinement"));
        assert!(!app_prelude_exports_symbol("ColorRef"));
        assert!(!app_prelude_exports_symbol("LayoutRefinement"));
        assert!(!app_prelude_exports_symbol("MetricRef"));
        assert!(!app_prelude_exports_symbol("Radius"));
        assert!(!app_prelude_exports_symbol("ShadowPreset"));
        assert!(!app_prelude_exports_symbol("Size"));
        assert!(!app_prelude_exports_symbol("Space"));
        assert!(!app_prelude_exports_symbol("TextOverflow"));
        assert!(!app_prelude_exports_symbol("TextWrap"));
        assert!(!app_prelude_exports_symbol("accent_color"));
        assert!(!app_prelude_exports_symbol("tailwind"));
        assert!(!app_prelude_exports_symbol("container_breakpoints"));
        assert!(!app_prelude_exports_symbol("preferred_color_scheme"));
        assert!(!app_prelude_exports_symbol("safe_area_insets"));
        assert!(!app_prelude_exports_symbol("viewport_breakpoints"));
        assert!(!app_prelude_exports_symbol("viewport_tailwind"));
        assert!(!app_prelude_exports_symbol("on_activate"));
        assert!(!app_prelude_exports_symbol("on_activate_notify"));
        assert!(!app_prelude_exports_symbol("on_activate_request_redraw"));
        assert!(!app_prelude_exports_symbol(
            "on_activate_request_redraw_notify"
        ));
        assert!(!app_prelude_exports_symbol("RouterUiStore"));
        assert!(!app_prelude_exports_symbol("DockManager"));
        assert!(!app_prelude_exports_symbol("DockPanelRegistry"));
        assert!(!app_prelude_exports_symbol("handle_dock_op"));
        assert!(!app_prelude_exports_symbol("InstallConfig"));
    }

    #[test]
    fn app_module_explicitly_exports_activation_surface_and_extension() {
        assert!(LIB_RS.contains("pub use crate::view::{AppActivateExt, AppActivateSurface};"));
    }

    #[test]
    fn app_and_style_modules_expose_explicit_secondary_app_nouns() {
        assert!(LIB_RS.contains("pub use crate::view::LocalState;"));
        assert!(LIB_RS.contains("pub use fret_ui::{Theme, ThemeSnapshot};"));
    }

    #[test]
    fn ui_child_alias_uses_unified_component_conversion_trait() {
        let tests_start = LIB_RS.find("#[cfg(test)]").unwrap_or(LIB_RS.len());
        let public_surface = &LIB_RS[..tests_start];
        assert!(
            public_surface
                .contains("pub trait UiChild: fret_ui_kit::IntoUiElement<crate::app::App>")
        );
        assert!(
            !public_surface
                .contains("pub trait UiChild: fret_ui_kit::UiChildIntoElement<crate::app::App>")
        );
    }

    #[test]
    fn advanced_prelude_reexports_app_facing_view_aliases() {
        let advanced_prelude = advanced_prelude_source();
        assert!(LIB_RS.contains("pub use crate::{AppUi, Ui, UiCx};"));
        assert!(advanced_prelude_exports_symbol("KernelApp"));
        assert!(advanced_prelude_exports_symbol("AppUiRawActionExt"));
        assert!(advanced_prelude_exports_symbol("AppUiRawStateExt"));
        assert!(advanced_prelude_exports_symbol("AppUi"));
        assert!(advanced_prelude_exports_symbol("Ui"));
        assert!(advanced_prelude_exports_symbol("UiCx"));
        assert!(advanced_prelude_exports_symbol("ViewElements"));
        assert!(advanced_prelude_exports_symbol("ElementContext"));
        assert!(advanced_prelude_exports_symbol("UiTree"));
        assert!(advanced_prelude.contains("pub use crate::view::QueryHandleReadExt as _;"));
        assert!(advanced_prelude.contains("pub use crate::view::UiCxActionsExt as _;"));
        assert!(advanced_prelude.contains("pub use crate::view::UiCxDataExt as _;"));
        assert!(
            advanced_prelude.contains("pub use fret_ui_kit::declarative::TrackedModelExt as _;")
        );
        assert!(advanced_prelude_exports_symbol("UiServices"));
        assert!(advanced_prelude_exports_symbol("TextProps"));
        assert!(!advanced_prelude.contains("pub use crate::component::prelude::*;"));
        assert!(!advanced_prelude_exports_symbol("UiBuilder"));
        assert!(!advanced_prelude_exports_symbol("UiPatchTarget"));
        assert!(!advanced_prelude_exports_symbol("IntoUiElement"));
        assert!(!advanced_prelude_exports_symbol("UiHost"));
        assert!(!advanced_prelude_exports_symbol("AnyElement"));
        assert!(!advanced_prelude_exports_symbol("Model"));
        assert!(!advanced_prelude_exports_symbol("TrackedModelExt"));
        assert!(!advanced_prelude_exports_symbol("ViewCx"));
        assert!(!advanced_prelude_exports_symbol("Elements"));
        assert!(
            !advanced_prelude
                .contains("pub use crate::view::{LocalState, TrackedStateExt, View, ViewCx};")
        );
        assert!(!advanced_prelude.contains(
            "pub use fret_ui::element::{Elements, HoverRegionProps, Length, SemanticsProps};"
        ));
        assert!(
            advanced_prelude
                .contains("Explicit raw-model local-state hooks kept on the advanced lane.")
        );
        assert!(advanced_prelude.contains("while leaving `fret::app::prelude::*` focused on"));
    }

    #[test]
    fn retained_advanced_aliases_live_only_on_explicit_advanced_surface() {
        let root_header = root_surface_header_source();
        let advanced_prelude = advanced_prelude_source();
        assert!(!root_header.contains("pub use fret_app::App as KernelApp;"));
        assert!(!root_header.contains("pub use fret_bootstrap::ui_app_driver::ViewElements;"));
        assert!(!root_header.contains("pub use fret_framework as kernel;"));
        assert!(advanced_prelude.contains("pub use fret_app::App as KernelApp;"));
        assert!(advanced_prelude.contains("pub use fret_bootstrap::ui_app_driver::ViewElements;"));
        assert!(advanced_prelude.contains("pub use fret_framework as kernel;"));
        assert!(LIB_RS.contains("pub type AppUi<'cx, 'a, H = crate::app::App>"));
        assert!(
            LIB_RS.contains("pub type UiCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;")
        );
    }

    #[test]
    fn root_surface_omits_low_level_action_registry_aliases() {
        let root_header = root_surface_header_source();
        let app_prelude = app_prelude_source();

        assert!(!root_header.contains("ActionMeta"));
        assert!(!root_header.contains("ActionRegistry"));
        assert!(root_header.contains("pub use fret_runtime::{ActionId, CommandId, TypedAction};"));
        assert!(ACTIONS_RS.contains("pub use fret_ui_kit::command::ElementCommandGatingExt;"));
        assert!(ACTIONS_RS.contains(
            "pub use fret_runtime::{ActionId, ActionMeta, ActionRegistry, CommandId, TypedAction};"
        ));
        assert!(!app_prelude_exports_symbol("ActionMeta"));
        assert!(!app_prelude_exports_symbol("ActionRegistry"));
        assert!(!app_prelude.contains("ActionMeta"));
        assert!(!app_prelude.contains("ActionRegistry"));
        assert!(!app_prelude.contains("ElementCommandGatingExt"));
    }

    #[test]
    fn root_surface_omits_workspace_shell_shortcuts() {
        let root_header = root_surface_header_source();
        let public_surface = crate_public_surface_source();

        assert!(!root_header.contains(
            "pub use workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu};"
        ));
        assert!(public_surface.contains("pub mod workspace_shell;"));
        assert!(!app_prelude_exports_symbol("workspace_shell_model"));
        assert!(!app_prelude_exports_symbol(
            "workspace_shell_model_default_menu"
        ));
    }

    #[test]
    fn root_surface_module_budget_is_curated_and_closed() {
        let root_header = root_surface_header_source();
        let actual = root_header
            .lines()
            .filter_map(|line| {
                let module = line.strip_prefix("pub mod ")?;
                Some(
                    module
                        .trim_end_matches(';')
                        .trim_end_matches('{')
                        .trim()
                        .to_owned(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        let expected = [
            "activate",
            "actions",
            "assets",
            "children",
            "env",
            "icons",
            "integration",
            "overlay",
            "semantics",
            "style",
            "workspace_menu",
            "workspace_shell",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            actual, expected,
            "root-level public modules should stay on the curated explicit-lane budget"
        );
        assert!(!root_header.contains("pub mod view;"));
        assert!(!root_header.contains("pub mod dev {"));
    }

    #[test]
    fn root_surface_direct_pub_use_budget_is_curated_and_closed() {
        let root_header = root_surface_header_source();
        let actual = root_header
            .lines()
            .filter(|line| line.starts_with("pub use "))
            .map(str::trim)
            .collect::<std::collections::BTreeSet<_>>();
        let expected = [
            "pub use app_entry::FretApp;",
            "pub use fret_runtime::{ActionId, CommandId, TypedAction};",
            "pub use fret_ui_shadcn::facade as shadcn;",
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            actual, expected,
            "root-level direct re-exports should stay on the curated budget"
        );
    }

    #[test]
    fn root_surface_omits_icon_registry_and_icon_pack_builder_helpers() {
        let root_header = root_surface_header_source();
        let app_prelude = app_prelude_source();
        let ui_app_builder = ui_app_builder_impl_source();

        assert!(!root_header.contains("pub use fret_icons::IconRegistry;"));
        assert!(!app_prelude_exports_symbol("IconRegistry"));
        assert!(!app_prelude.contains("IconRegistry"));
        assert!(!APP_ENTRY_RS.contains("pub fn register_icon_pack("));
        assert!(!ui_app_builder.contains("pub fn register_icon_pack("));
        assert!(!ui_app_builder.contains("pub fn with_lucide_icons("));
    }

    #[test]
    fn root_surface_exposes_explicit_style_and_icon_modules() {
        let root_header = root_surface_header_source();

        assert!(root_header.contains("pub mod activate {"));
        assert!(root_header.contains("pub mod children {"));
        assert!(root_header.contains("pub mod icons {"));
        assert!(root_header.contains("pub mod semantics {"));
        assert!(root_header.contains("pub mod style {"));
        assert!(root_header.contains("pub use fret_ui_kit::{"));
        assert!(
            root_header.contains("on_activate, on_activate_notify, on_activate_request_redraw,")
        );
        assert!(root_header.contains("on_activate_request_redraw_notify,"));
        assert!(root_header.contains("pub use fret_ui_kit::ui::UiElementSinkExt;"));
        assert!(root_header.contains("pub use fret_icons::IconId;"));
        assert!(root_header.contains("pub use fret_ui_kit::declarative::icon;"));
        assert!(root_header.contains("pub use fret_core::SemanticsRole;"));
        assert!(root_header.contains("pub use fret_core::{TextOverflow, TextWrap};"));
        assert!(root_header.contains("pub use fret_ui::{Theme, ThemeSnapshot};"));
        assert!(root_header.contains("ChromeRefinement, ColorRef, LayoutRefinement, MetricRef"));
        assert!(root_header.contains("Radius, ShadowPreset, Size,"));
        assert!(root_header.contains("Space,"));
    }

    #[test]
    fn root_surface_exposes_explicit_overlay_module() {
        let root_header = root_surface_header_source();

        assert!(root_header.contains("pub mod overlay {"));
        assert!(root_header.contains("pub use fret_ui_kit::overlay::*;"));
        assert!(
            root_header.contains("OverlayArbitrationSnapshot, OverlayController, OverlayKind,")
        );
        assert!(root_header.contains("OverlayPresence,"));
        assert!(root_header.contains("OverlayRequest, OverlayStackEntryKind,"));
        assert!(root_header.contains("WindowOverlayStackEntry,"));
        assert!(root_header.contains("WindowOverlayStackSnapshot,"));
    }

    #[test]
    fn root_surface_exposes_explicit_assets_module() {
        let root_header = root_surface_header_source();

        assert!(root_header.contains("pub mod assets {"));
        assert!(root_header.contains("AssetStartupMode"));
        assert!(root_header.contains("AssetStartupPlan"));
        assert!(root_header.contains("AssetStartupPlanError"));
        assert!(root_header.contains("pub use fret_assets::{"));
        assert!(root_header.contains("AssetBundleId,"));
        assert!(root_header.contains("AssetBundleNamespace,"));
        assert!(root_header.contains("AssetCapabilities,"));
        assert!(root_header.contains("AssetKey,"));
        assert!(root_header.contains("AssetKindHint,"));
        assert!(root_header.contains("AssetExternalReference,"));
        assert!(root_header.contains("AssetLoadError,"));
        assert!(root_header.contains("AssetLocator,"));
        assert!(root_header.contains("AssetManifestLoadError,"));
        assert!(root_header.contains("AssetMediaType,"));
        assert!(root_header.contains("AssetMemoryKey,"));
        assert!(root_header.contains("AssetRequest,"));
        assert!(root_header.contains("AssetResolver,"));
        assert!(root_header.contains("AssetRevision,"));
        assert!(root_header.contains("FILE_ASSET_MANIFEST_KIND_V1"));
        assert!(root_header.contains("FileAssetManifestBundleV1,"));
        assert!(root_header.contains("FileAssetManifestEntryV1,"));
        assert!(root_header.contains("FileAssetManifestV1,"));
        assert!(root_header.contains("ResolvedAssetBytes,"));
        assert!(root_header.contains("ResolvedAssetReference,"));
        assert!(root_header.contains("StaticAssetEntry,"));
        assert!(root_header.contains("asset_package_bundle_id,"));
        assert!(root_header.contains("pub use fret_runtime::AssetResolverService;"));
        assert!(root_header.contains("pub use fret_assets::FileAssetManifestResolver;"));
        assert!(root_header.contains("pub fn set_primary_resolver("));
        assert!(root_header.contains("pub fn register_resolver("));
        assert!(root_header.contains("pub fn register_file_manifest("));
        assert!(root_header.contains("pub fn register_file_bundle_dir("));
        assert!(root_header.contains("pub fn register_bundle_entries("));
        assert!(root_header.contains("pub fn register_embedded_entries("));
        assert!(root_header.contains("pub fn capabilities("));
        assert!(root_header.contains("pub fn resolve_bytes("));
        assert!(root_header.contains("pub fn resolve_locator("));
        assert!(root_header.contains("pub fn resolve_reference("));
        assert!(root_header.contains("pub fn resolve_locator_reference("));
    }

    #[test]
    fn root_surface_exposes_explicit_env_module() {
        let root_header = root_surface_header_source();

        assert!(root_header.contains("pub mod env {"));
        assert!(
            root_header.contains("accent_color, container_breakpoints, container_query_region,")
        );
        assert!(root_header.contains("preferred_color_scheme, prefers_dark_color_scheme"));
        assert!(root_header.contains("safe_area_insets,"));
        assert!(root_header.contains("viewport_breakpoints, viewport_height_at_least"));
        assert!(root_header.contains("viewport_tailwind,"));
        assert!(root_header.contains("window_insets_padding_refinement_or_zero,"));
    }

    #[test]
    fn app_and_advanced_modules_expose_view_runtime_on_explicit_lanes_only() {
        let root_header = root_surface_header_source();
        let advanced_surface = advanced_prelude_source();

        assert!(LIB_RS.contains("pub use crate::view::View;"));
        assert!(!root_header.contains("pub mod view;"));
        assert!(advanced_surface.contains("pub mod view {"));
        assert!(advanced_surface.contains("ViewWindowState, view_init_window,"));
        assert!(advanced_surface.contains("view_record_engine_frame, view_view,"));
    }

    #[test]
    fn advanced_surface_quarantines_devloop_helpers_off_root() {
        let root_header = root_surface_header_source();
        let advanced_surface = advanced_prelude_source();

        assert!(!root_header.contains("pub mod dev {"));
        assert!(advanced_surface.contains("pub mod dev {"));
        assert!(advanced_surface.contains("DevStateExport, DevStateHook, DevStateHooks,"));
        assert!(advanced_surface.contains("DevStateSnapshot,"));
        assert!(advanced_surface.contains("DevStateWindowKeyRegistry,"));
    }

    #[test]
    fn public_surface_exposes_explicit_state_modules() {
        let public_surface = crate_public_surface_source();

        assert!(public_surface.contains("pub mod selector {"));
        assert!(public_surface.contains("pub mod query {"));
        assert!(!public_surface.contains("pub use crate::view::LocalDepsBuilderExt;"));
        assert!(public_surface.contains("pub use fret_selector::{DepsSignature, Selector};"));
        assert!(!public_surface.contains("pub use fret_selector::ui::DepsBuilder;"));
        assert!(public_surface.contains("pub use fret_selector::ui::*;"));
        assert!(public_surface.contains("pub use fret_query::{"));
        assert!(public_surface.contains("CancellationToken, FutureSpawner, FutureSpawnerHandle"));
        assert!(
            public_surface
                .contains("QueryError, QueryErrorKind, QueryHandle, QueryKey, QueryPolicy")
        );
        assert!(public_surface.contains("QueryRetryOn, QueryRetryPolicy, QueryRetryState"));
        assert!(public_surface.contains("QuerySnapshotEntry, QueryState,"));
        assert!(public_surface.contains("QueryStatus, with_query_client,"));
    }

    #[test]
    fn crate_feature_surface_omits_compat_icon_aliases() {
        assert!(CARGO_TOML.contains("icons = ["));
        assert!(!CARGO_TOML.contains("icons-lucide = [\"icons\"]"));
    }

    #[test]
    fn view_runtime_exposes_only_app_ui_as_the_public_context_name() {
        assert!(!VIEW_RS.contains("pub type ViewCx"));
        assert!(
            VIEW_RS.contains("fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui;")
        );
        assert!(VIEW_RS.contains(") -> crate::Ui {"));
    }

    #[test]
    fn app_prelude_omits_low_level_mechanism_types() {
        assert!(!app_prelude_exports_symbol("AppWindowId"));
        assert!(!app_prelude_exports_symbol("AppUiRawActionExt"));
        assert!(!app_prelude_exports_symbol("AppUiRawStateExt"));
        assert!(!app_prelude_exports_symbol("Event"));
        assert!(!app_prelude_exports_symbol("ElementContext"));
        assert!(!app_prelude_exports_symbol("UiTree"));
        assert!(!app_prelude_exports_symbol("UiServices"));
        assert!(!app_prelude_exports_symbol("UiHost"));
        assert!(!app_prelude_exports_symbol("AnyElement"));
        assert!(!app_prelude_exports_symbol("ActionId"));
        assert!(!app_prelude_exports_symbol("TypedAction"));
        assert!(!app_prelude_exports_symbol("RouterUiStore"));
        assert!(!app_prelude_exports_symbol("RouterOutlet"));
        assert!(!app_prelude_exports_symbol("UiBuilder"));
        assert!(!app_prelude_exports_symbol("UiPatchTarget"));
        assert!(!app_prelude_exports_symbol("HoverRegionProps"));
        assert!(!app_prelude_exports_symbol("Length"));
        assert!(!app_prelude_exports_symbol("SemanticsProps"));
        assert!(!app_prelude_exports_symbol("UiElementSinkExt"));
        assert!(!app_prelude_exports_symbol("ContainerQueryHysteresis"));
        assert!(!app_prelude_exports_symbol("ViewportQueryHysteresis"));
        assert!(!app_prelude_exports_symbol("ImageMetadata"));
        assert!(!app_prelude_exports_symbol("ImageMetadataStore"));
        assert!(!app_prelude_exports_symbol("ImageSamplingExt"));
        assert!(!app_prelude_exports_symbol("MarginEdge"));
        assert!(!app_prelude_exports_symbol("SemanticsRole"));
        assert!(!app_prelude_exports_symbol("OverrideSlot"));
        assert!(!app_prelude_exports_symbol("WidgetState"));
        assert!(!app_prelude_exports_symbol("WidgetStateProperty"));
        assert!(!app_prelude_exports_symbol("WidgetStates"));
        assert!(!app_prelude_exports_symbol("merge_override_slot"));
        assert!(!app_prelude_exports_symbol("merge_slot"));
        assert!(!app_prelude_exports_symbol("resolve_override_slot"));
        assert!(!app_prelude_exports_symbol("resolve_override_slot_opt"));
        assert!(!app_prelude_exports_symbol(
            "resolve_override_slot_opt_with"
        ));
        assert!(!app_prelude_exports_symbol("resolve_override_slot_with"));
        assert!(!app_prelude_exports_symbol("resolve_slot"));
        assert!(!app_prelude_exports_symbol("ColorFallback"));
        assert!(!app_prelude_exports_symbol("SignedMetricRef"));
        assert!(!app_prelude_exports_symbol("Corners4"));
        assert!(!app_prelude_exports_symbol("Edges4"));
        assert!(!app_prelude_exports_symbol("ViewportOrientation"));
        assert!(!app_prelude_exports_symbol("AssetBundleId"));
        assert!(!app_prelude_exports_symbol("AssetBundleNamespace"));
        assert!(!app_prelude_exports_symbol("AssetCapabilities"));
        assert!(!app_prelude_exports_symbol("AssetKey"));
        assert!(!app_prelude_exports_symbol("AssetLocator"));
        assert!(!app_prelude_exports_symbol("AssetManifestLoadError"));
        assert!(!app_prelude_exports_symbol("AssetRequest"));
        assert!(!app_prelude_exports_symbol("AssetResolver"));
        assert!(!app_prelude_exports_symbol("AssetRevision"));
        assert!(!app_prelude_exports_symbol("FileAssetManifestBundleV1"));
        assert!(!app_prelude_exports_symbol("FileAssetManifestEntryV1"));
        assert!(!app_prelude_exports_symbol("FileAssetManifestResolver"));
        assert!(!app_prelude_exports_symbol("FileAssetManifestV1"));
        assert!(!app_prelude_exports_symbol("ResolvedAssetBytes"));
        assert!(!app_prelude_exports_symbol("StaticAssetEntry"));
        assert!(!app_prelude_exports_symbol("AssetResolverService"));
        assert!(!app_prelude_exports_symbol("CancellationToken"));
        assert!(!app_prelude_exports_symbol("QueryError"));
        assert!(!app_prelude_exports_symbol("QueryHandle"));
        assert!(!app_prelude_exports_symbol("QueryKey"));
        assert!(!app_prelude_exports_symbol("QueryPolicy"));
        assert!(!app_prelude_exports_symbol("DepsBuilder"));
        assert!(!app_prelude_exports_symbol("DepsSignature"));
        assert!(!app_prelude_exports_symbol("LocalDepsBuilderExt"));
    }

    #[test]
    fn component_prelude_is_curated_for_reusable_component_authors() {
        let component_prelude = component_prelude_source();
        assert!(component_prelude.contains("pub use crate::ComponentCx;"));
        assert!(component_prelude.contains("pub use fret_ui_kit::ui;"));
        assert!(component_prelude.contains("pub use fret_ui_kit::{"));
        assert!(
            component_prelude
                .contains("pub use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;")
        );
        assert!(
            component_prelude
                .contains("pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;")
        );
        assert!(
            component_prelude
                .contains("pub use fret_ui_kit::declarative::UiElementTestIdExt as _;")
        );
        assert!(
            component_prelude.contains("pub use fret_ui_kit::declarative::TrackedModelExt as _;")
        );
        assert!(component_prelude_exports_symbol("UiBuilder"));
        assert!(component_prelude_exports_symbol("UiPatchTarget"));
        assert!(component_prelude_exports_symbol("IntoUiElement"));
        assert!(component_prelude_exports_symbol("UiExt"));
        assert!(component_prelude_exports_symbol("AnyElement"));
        assert!(component_prelude_exports_symbol("UiHost"));
        assert!(component_prelude_exports_symbol("Invalidation"));
        assert!(component_prelude_exports_symbol("Theme"));
        assert!(component_prelude_exports_symbol("Model"));
        assert!(component_prelude_exports_symbol("OverlayController"));
        assert!(component_prelude_exports_symbol("OverlayRequest"));
        assert!(component_prelude_exports_symbol("OverlayPresence"));
        assert!(component_prelude_exports_symbol("SemanticsRole"));
        assert!(!component_prelude.contains("pub use fret_ui_kit::prelude::*;"));
        assert!(!component_prelude_exports_symbol("accent_color"));
        assert!(!component_prelude_exports_symbol("container_breakpoints"));
        assert!(!component_prelude_exports_symbol("safe_area_insets"));
        assert!(!component_prelude_exports_symbol("viewport_breakpoints"));
        assert!(!component_prelude_exports_symbol("viewport_tailwind"));
        assert!(!component_prelude_exports_symbol("ActionHooksExt"));
        assert!(!component_prelude_exports_symbol("AnyElementSemanticsExt"));
        assert!(!component_prelude_exports_symbol("CollectionSemanticsExt"));
        assert!(!component_prelude_exports_symbol("ElementContextThemeExt"));
        assert!(!component_prelude_exports_symbol("GlobalWatchExt"));
        assert!(!component_prelude_exports_symbol("ModelWatchExt"));
        assert!(!component_prelude_exports_symbol("TrackedModelExt"));
        assert!(!component_prelude_exports_symbol("UiElementA11yExt"));
        assert!(!component_prelude_exports_symbol("UiElementKeyContextExt"));
        assert!(!component_prelude_exports_symbol("UiElementTestIdExt"));
        assert!(!component_prelude_exports_symbol("UiIntoElement"));
        assert!(!component_prelude_exports_symbol("UiHostBoundIntoElement"));
        assert!(!component_prelude_exports_symbol("UiChildIntoElement"));
        assert!(!component_prelude_exports_symbol(
            "OverlayArbitrationSnapshot"
        ));
        assert!(!component_prelude_exports_symbol("OverlayKind"));
        assert!(!component_prelude_exports_symbol("OverlayStackEntryKind"));
        assert!(!component_prelude_exports_symbol("WindowOverlayStackEntry"));
        assert!(!component_prelude_exports_symbol(
            "WindowOverlayStackSnapshot"
        ));
        assert!(!component_prelude_exports_symbol("on_activate"));
        assert!(!component_prelude_exports_symbol("on_activate_notify"));
        assert!(!component_prelude_exports_symbol(
            "on_activate_request_redraw"
        ));
        assert!(!component_prelude_exports_symbol(
            "on_activate_request_redraw_notify"
        ));
    }

    #[test]
    fn app_and_component_preludes_only_overlap_on_ui_and_px() {
        let app_symbols = exported_symbol_names(app_prelude_source());
        let component_symbols = exported_symbol_names(component_prelude_source());
        let overlap = app_symbols
            .intersection(&component_symbols)
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(overlap, vec!["Px".to_string(), "ui".to_string()]);
    }

    #[test]
    fn component_prelude_omits_app_runtime_and_recipe_specific_surfaces() {
        assert!(!component_prelude_exports_symbol("FretApp"));
        assert!(!component_prelude_exports_symbol("App"));
        assert!(!component_prelude_exports_symbol("AppUi"));
        assert!(!component_prelude_exports_symbol("Ui"));
        assert!(!component_prelude_exports_symbol("UiCx"));
        assert!(!component_prelude_exports_symbol("WindowId"));
        assert!(!component_prelude_exports_symbol("KernelApp"));
        assert!(!component_prelude_exports_symbol("UiAppBuilder"));
        assert!(!component_prelude_exports_symbol("UiAppDriver"));
        assert!(!component_prelude_exports_symbol("UiServices"));
        assert!(!component_prelude_exports_symbol("AppWindowId"));
        assert!(!component_prelude_exports_symbol("Event"));
        assert!(!component_prelude_exports_symbol("UiTree"));
        assert!(!component_prelude_exports_symbol("ActionId"));
        assert!(!component_prelude_exports_symbol("CommandId"));
        assert!(!component_prelude_exports_symbol("TypedAction"));
        assert!(!component_prelude_exports_symbol("shadcn"));
    }

    #[test]
    fn legacy_root_prelude_is_deleted() {
        assert!(!LIB_RS.contains("pub mod prelude {\n    pub use fret_ui_kit::prelude::*;"));
    }

    #[test]
    fn root_builder_aliases_are_deleted() {
        let lines = LIB_RS.lines().map(str::trim).collect::<Vec<_>>();
        assert!(!lines.contains(&"pub use app_entry::App;"));
        assert!(!lines.contains(&"pub use app_entry::App as AppBuilder;"));
        assert!(!lines.contains(&"pub use app_entry::App as FretApp;"));
        assert!(lines.contains(&"pub use app_entry::FretApp;"));
    }

    #[test]
    fn app_builder_uses_setup_language_on_default_surface() {
        assert!(APP_ENTRY_RS.contains("pub fn setup<") || APP_ENTRY_RS.contains("pub fn setup("));
        assert!(
            APP_ENTRY_RS.contains("pub fn asset_startup(")
                || APP_ENTRY_RS.contains("pub fn asset_startup<")
        );
        assert!(
            APP_ENTRY_RS.contains("pub fn asset_manifest(")
                || APP_ENTRY_RS.contains("pub fn asset_manifest<")
        );
        assert!(
            APP_ENTRY_RS.contains("pub fn asset_dir(")
                || APP_ENTRY_RS.contains("pub fn asset_dir<")
        );
        assert!(APP_ENTRY_RS.contains("pub fn view<") || APP_ENTRY_RS.contains("pub fn view("));
        assert!(
            APP_ENTRY_RS.contains("pub fn view_with_hooks<")
                || APP_ENTRY_RS.contains("pub fn view_with_hooks(")
        );
        assert!(!APP_ENTRY_RS.contains("pub fn install_app("));
        assert!(!APP_ENTRY_RS.contains("pub fn install("));
        assert!(!APP_ENTRY_RS.contains("pub fn register_icon_pack("));
        assert!(!APP_ENTRY_RS.contains("pub fn run_view("));
        assert!(!APP_ENTRY_RS.contains("pub fn run_view_with_hooks("));

        let ui_app_builder = ui_app_builder_impl_source();
        assert!(ui_app_builder.contains("pub fn setup_with("));
        assert!(
            ui_app_builder.contains("pub fn setup<") || ui_app_builder.contains("pub fn setup(")
        );
        assert!(ui_app_builder.contains("pub fn with_asset_startup("));
        assert!(ui_app_builder.contains("pub fn with_asset_dir("));
        assert!(ui_app_builder.contains("pub fn with_asset_manifest("));
        assert!(!ui_app_builder.contains("pub fn init_app("));
        assert!(!ui_app_builder.contains("pub fn install("));
        assert!(!ui_app_builder.contains("pub fn register_icon_pack("));
        assert!(!ui_app_builder.contains("pub fn with_lucide_icons("));
        assert!(!ui_app_builder.contains("pub fn install_custom_effects("));
        assert!(!ui_app_builder.contains("pub fn on_gpu_ready("));

        assert!(LIB_RS.contains("pub trait FretAppAdvancedExt"));
        assert!(LIB_RS.contains("pub trait UiAppBuilderAdvancedExt"));
    }

    #[test]
    fn app_entry_builder_name_is_fret_app_only() {
        assert!(APP_ENTRY_RS.contains("pub struct FretApp"));
        assert!(APP_ENTRY_RS.contains("AssetBundleId::app(self.root_name)"));
        assert!(!APP_ENTRY_RS.contains("pub struct App"));
    }
}
