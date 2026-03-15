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
//!     fn render(&mut self, _ui: &mut AppUi<'_, '_>) -> Ui {
//!         shadcn::Label::new("Fret!").into()
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
//! - enable `state` for grouped selector/query helpers on `AppUi`
//! - enable `router` for `fret::router::{app::install, RouterUiStore, RouterOutlet, router_link, ...}`
//!   plus `RouterUiStore::{back_on_action, forward_on_action}` history bindings
//! - enable `docking` for `fret::docking::{core::*, DockManager, handle_dock_op, ...}`
//! - use `fret::shadcn::{..., app::install, themes::apply_shadcn_new_york, raw::*}` for the
//!   curated default design-system surface; advanced environment / `UiServices` hooks stay on
//!   `fret::shadcn::raw::advanced::*`
//! - use `fret::integration::InstallIntoApp` for reusable app-install bundles; small app-local
//!   composition can also use `.setup((install_a, install_b))` while ordinary app code keeps
//!   passing named installer functions to `.setup(...)` and keeps inline one-off closures or
//!   runtime-captured config on `UiAppBuilder::setup_with(...)`
use crate::advanced::KernelApp;

/// Canonical app-facing window identity alias for the default authoring surface.
pub type WindowId = fret_core::AppWindowId;

/// Re-export the curated default shadcn/ui surface as `shadcn`.
#[cfg(feature = "shadcn")]
pub use fret_ui_shadcn::facade as shadcn;

/// Re-export portable action/command identity types for app code and macros.
pub use fret_runtime::{ActionId, CommandId, TypedAction};

/// Explicit icon identifiers for app and component code that opt into icon-specific authoring.
pub mod icons {
    pub use fret_icons::IconId;
}

/// Explicit style/token nouns for app code that customizes layout or chrome beyond the default lane.
pub mod style {
    pub use fret_ui::Theme;
    pub use fret_ui_kit::{
        ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, ShadowPreset, Size,
        Space,
    };
}

/// Explicit environment and responsive helpers for app code that opts into adaptive UI logic.
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

pub mod actions;
pub mod view;
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

/// Dev-only helpers (hotpatch/dev-state) for iteration workflows.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "devloop"))]
pub mod dev {
    pub use fret_launch::dev_state::{
        DevStateExport, DevStateHook, DevStateHooks, DevStateSnapshot, DevStateWindowKeyRegistry,
    };
}

/// Re-export the kernel facade (desktop builds).
#[cfg(feature = "desktop")]
use fret_framework as kernel;

/// App-facing imports for ordinary Fret application code.
pub mod app {
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
        pub use crate::view::TrackedStateExt as _;
        pub use crate::view::UiCxDataExt as _;
        pub use crate::view::{LocalState, View};
        pub use crate::{AppUi, Ui, UiChild, UiCx, WindowId};
        pub use crate::{actions, workspace_menu};
        pub use fret_core::{Px, SemanticsRole, TextOverflow, TextWrap};
        pub use fret_runtime::CommandId;
        pub use fret_ui::ThemeSnapshot;
        pub use fret_ui_kit::IntoUiElement as _;
        pub use fret_ui_kit::StyledExt as _;
        pub use fret_ui_kit::UiExt as _;
        pub use fret_ui_kit::command::ElementCommandGatingExt as _;
        pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;
        pub use fret_ui_kit::declarative::ElementContextThemeExt as _;
        pub use fret_ui_kit::declarative::UiElementA11yExt as _;
        pub use fret_ui_kit::declarative::UiElementKeyContextExt as _;
        pub use fret_ui_kit::declarative::UiElementTestIdExt as _;
        pub use fret_ui_kit::declarative::icon;
        pub use fret_ui_kit::ui;
        pub use fret_ui_kit::ui::UiElementSinkExt as _;

        #[cfg(feature = "state-query")]
        pub use fret_query::{CancellationToken, QueryError, QueryHandle, QueryKey, QueryPolicy};
        #[cfg(feature = "state-selector")]
        pub use fret_selector::{DepsSignature, ui::DepsBuilder};
    }
}

/// Component-author imports for reusable, portable UI crates.
pub mod component {
    /// Common imports for reusable component crates built on Fret.
    pub mod prelude {
        pub use crate::ComponentCx;
        pub use fret_ui_kit::IntoUiElement as _;
        pub use fret_ui_kit::command::ElementCommandGatingExt as _;
        pub use fret_ui_kit::declarative::prelude::{
            ActionHooksExt, AnyElementSemanticsExt, CollectionSemanticsExt, ElementContextThemeExt,
            GlobalWatchExt, ModelWatchExt, UiElementA11yExt, UiElementKeyContextExt,
            UiElementTestIdExt, accent_color, container_breakpoints, container_query_region,
            container_query_region_with_id, container_width_at_least, contrast_preference,
            forced_colors_active, forced_colors_mode, preferred_color_scheme,
            prefers_dark_color_scheme, prefers_more_contrast, prefers_reduced_motion,
            prefers_reduced_transparency, primary_pointer_can_hover, primary_pointer_is_coarse,
            primary_pointer_type, safe_area_insets, safe_area_insets_or_zero, tailwind,
            text_scale_factor, viewport_aspect_ratio, viewport_breakpoints,
            viewport_height_at_least, viewport_height_breakpoints, viewport_is_landscape,
            viewport_is_portrait, viewport_orientation, viewport_tailwind, viewport_width_at_least,
            window_insets_padding_refinement_or_zero,
        };
        pub use fret_ui_kit::ui;
        pub use fret_ui_kit::ui::UiElementSinkExt as _;
        pub use fret_ui_kit::{
            ChromeRefinement, ColorRef, Corners4, Edges4, IntoUiElement, LayoutRefinement,
            MetricRef, OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence,
            OverlayRequest, OverlayStackEntryKind, Radius, ShadowPreset, Size, Space, UiBuilder,
            UiExt, UiPatchTarget, UiSupportsChrome, UiSupportsLayout, WindowOverlayStackEntry,
            WindowOverlayStackSnapshot, on_activate, on_activate_notify,
            on_activate_request_redraw, on_activate_request_redraw_notify,
        };

        #[cfg(feature = "icons")]
        pub use fret_icons::IconId;
        #[cfg(feature = "icons")]
        pub use fret_ui_kit::declarative::icon;

        pub use fret_core::{Px, SemanticsRole, TextOverflow, TextWrap};
        pub use fret_runtime::{CommandId, Model};
        pub use fret_ui::element::{AnyElement, AnyElementIterExt as _};
        pub use fret_ui::{Invalidation, Theme, UiHost};
    }
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
    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    /// Low-level interop helpers kept off the default crate root.
    pub mod interop {
        pub use crate::interop::embedded_viewport;
        pub use crate::interop::run_native_with_compat_driver;
    }
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
        pub use crate::component::prelude::*;
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
    Runner(#[from] RunnerError),
}

/// Result type used by the `fret` facade.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BootstrapError(#[from] fret_bootstrap::BootstrapError);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RunnerError(#[from] fret_launch::RunnerError);

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
    let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
        app,
        window,
        config.base_color,
        config.scheme,
    );
}

#[cfg(all(test, not(target_arch = "wasm32"), feature = "desktop"))]
mod builder_surface_tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::FretApp;
    use crate::advanced::{
        FretAppAdvancedExt as _, KernelApp, UiAppBuilderAdvancedExt as _, ViewElements,
    };
    use crate::app::App;
    use crate::app::prelude::FretApp as AppPreludeFretApp;
    use crate::view::View;
    use crate::{AppUi, Defaults, Ui, WindowId};
    use fret_app::CreateWindowRequest;
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

    use crate::advanced::KernelApp;
    use fret_core::{AppWindowId, ColorScheme, WindowMetricsService};
    use fret_ui::{Theme, UiTree};

    #[test]
    fn shadcn_auto_theme_middleware_reacts_to_window_metrics() {
        let mut app = KernelApp::new();
        fret_ui_shadcn::app::install(&mut app);

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
    const SHADCN_DECLARATIVE_PROGRESS: &str =
        include_str!("../../../docs/shadcn-declarative-progress.md");
    const AUTHORING_SURFACE_TARGET_INTERFACE_STATE: &str = include_str!(
        "../../../docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md"
    );
    const CRATE_README: &str = include_str!("../README.md");
    const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");
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
        let component_start = LIB_RS
            .find("/// Component-author imports for reusable, portable UI crates.")
            .expect("component module marker should exist in fret facade");
        &LIB_RS[app_start..component_start]
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
        let router_start = LIB_RS
            .find("/// Optional router integration surface for app code.")
            .expect("router module marker should exist in fret facade");
        &LIB_RS[component_start..router_start]
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

    #[test]
    fn readme_prefers_view_entry_and_omits_ui_bridge() {
        assert!(CRATE_README.contains(
            "App authors (default recommendation): `fret::FretApp::new(...).window(...).view::<V>()?`"
        ));
        assert!(CRATE_README.contains("`state`: enable selector/query helpers on `AppUi`"));
        assert!(CRATE_README.contains("`fret::style::{...}`"));
        assert!(CRATE_README.contains("`fret::icons::IconId`"));
        assert!(CRATE_README.contains("`fret::env::{...}`"));
        assert!(!CRATE_README.contains(".run_view::<"));
        assert!(!CRATE_README.contains(".install_app("));
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
    fn readme_and_rustdoc_expose_curated_shadcn_surface() {
        assert!(CRATE_README.contains("`fret::shadcn`"));
        assert!(CRATE_README.contains("`shadcn::app::install(...)`"));
        assert!(CRATE_README.contains("`shadcn::themes::apply_shadcn_new_york(...)`"));
        assert!(CRATE_README.contains("`shadcn::raw::*`"));
        assert!(CRATE_README.contains("`fret::shadcn::raw::advanced::*`"));
        assert!(CRATE_README.contains("`fret_ui_shadcn::advanced::*`"));

        let rustdoc = crate_rustdoc();
        let public_surface = crate_public_surface_source();
        assert!(rustdoc.contains(
            "//! - use `fret::shadcn::{..., app::install, themes::apply_shadcn_new_york, raw::*}`"
        ));
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
        assert!(DOCS_README.contains("`cx.actions().payload::<A>()`"));
        assert!(!DOCS_README.contains("`ecosystem/fret` (`View`, `ViewCx`, `fret::actions!`)"));
        assert!(!DOCS_README.contains("ViewCx::on_payload_action*"));
    }

    #[test]
    fn docs_index_and_first_hour_stay_on_default_app_surface() {
        assert!(DOCS_README.contains("`use fret::app::prelude::*;`"));
        assert!(DOCS_README.contains("`FretApp::new(...).window(...).view::<MyView>()?.run()`"));
        assert!(DOCS_README.contains("`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`"));
        assert!(!DOCS_README.contains("run_view::<"));
        assert!(!DOCS_README.contains("ViewCx::"));

        assert!(FIRST_HOUR.contains("`use fret::app::prelude::*;`"));
        assert!(FIRST_HOUR.contains(
            "`FretApp::new(\"my-simple-todo\").window(\"my-simple-todo\", (...)).view::<TodoView>()?.run()`"
        ));
        assert!(FIRST_HOUR.contains("`fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`"));
        assert!(FIRST_HOUR.contains("`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`"));
        assert!(!FIRST_HOUR.contains("run_view::<"));
        assert!(!FIRST_HOUR.contains("ViewCx::"));
        assert!(!FIRST_HOUR.contains("`fret_ui_shadcn::prelude::*`"));
    }

    #[test]
    fn usage_docs_prefer_grouped_app_ui_actions() {
        assert!(CRATE_USAGE_GUIDE.contains("start with `View` + `AppUi` + typed actions"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().locals::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().models::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().transient::<A>(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::style::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::icons::IconId`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`.on_activate(cx.actions().dispatch::<A>())`"));
        assert!(CRATE_USAGE_GUIDE.contains("`.on_activate(cx.actions().listener(...))`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().selector(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("ViewCx::use_selector"));
        assert!(!CRATE_USAGE_GUIDE.contains("ViewCx::use_query"));
    }

    #[test]
    fn authoring_docs_prefer_grouped_app_ui_data_helpers() {
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().selector(...)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`cx.data().query(...)`"));
        assert!(AUTHORING_GOLDEN_PATH_V2.contains("`.on_activate(cx.actions().dispatch::<A>())`"));
        assert!(
            AUTHORING_GOLDEN_PATH_V2
                .contains("`.on_activate(cx.actions().listener(|host, acx| { ... }))`")
        );
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`cx.use_selector(...)`"));
        assert!(!AUTHORING_GOLDEN_PATH_V2.contains("`cx.use_query(...)`"));
    }

    #[test]
    fn integration_docs_prefer_grouped_query_helpers_for_app_surface() {
        assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async(...)`"));
        assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async_local(...)`"));
        assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().query_async(...)`"));
    }

    #[test]
    fn usage_docs_expose_router_as_explicit_extension_surface() {
        assert!(CRATE_USAGE_GUIDE.contains("enable `fret`'s `router` feature"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::router::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`back_on_action()`"));
        assert!(CRATE_USAGE_GUIDE.contains("`forward_on_action()`"));
        assert!(CRATE_USAGE_GUIDE.contains("`cx.on_action_notify::<...>(store.back_on_action())`"));
        assert!(CRATE_USAGE_GUIDE.contains("second default app runtime"));
    }

    #[test]
    fn usage_docs_link_ecosystem_trait_budget_and_anti_plugin_posture() {
        assert!(CRATE_USAGE_GUIDE.contains("## Ecosystem author checklist"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::integration::InstallIntoApp`"));
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
        assert!(CRATE_USAGE_GUIDE.contains("`fret_ui_assets::app::install_with_budgets(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret_ui_assets::advanced::{install_with_ui_services(...), install_with_ui_services_and_budgets(...)}`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret_node::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::router::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`BootstrapBuilder::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`FretApp::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::register_icon_pack(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_lucide_icons()`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret::router::install_app(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_icons_radix::install_app`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_assets::install_app_with_budgets`"));
    }

    #[test]
    fn todo_golden_path_keeps_icon_pack_setup_on_app_install_surface() {
        assert!(TODO_APP_GOLDEN_PATH.contains("`.setup(fret_icons_radix::app::install)`"));
        assert!(!TODO_APP_GOLDEN_PATH.contains(".register_icon_pack("));
        assert!(!TODO_APP_GOLDEN_PATH.contains("IconRegistry"));
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
    fn usage_docs_expose_curated_component_surface() {
        assert!(CRATE_USAGE_GUIDE.contains("`use fret::component::prelude::*;`"));
        assert!(CRATE_USAGE_GUIDE.contains("`ComponentCx`"));
        assert!(CRATE_USAGE_GUIDE.contains("`UiBuilder`/`UiPatchTarget`/`IntoUiElement<H>`"));
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
        assert!(CRATE_USAGE_GUIDE.contains(
            "`fret_ui_shadcn::advanced::{sync_theme_from_environment(...), install_with_ui_services(...)}`"
        ));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::raw::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`shadcn::raw::typography::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::app::install(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::themes::apply_shadcn_new_york(...)`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::*`"));
        assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::advanced::*`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::install_app(...)`"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::shadcn_themes::"));
        assert!(!CRATE_USAGE_GUIDE.contains("`fret::shadcn::shadcn_themes::"));
    }

    #[test]
    fn shadcn_docs_keep_advanced_hooks_off_curated_lane() {
        assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`fret_ui_shadcn::advanced::*`"));
        assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`shadcn::advanced::*`"));
        assert!(AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret_ui_shadcn::advanced`"));
        assert!(
            AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret::shadcn::raw::advanced::*`")
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
        assert!(!ACTION_FIRST_MIGRATION_GUIDE.contains("use fret_ui_shadcn as shadcn;"));
        assert!(!SHADCN_SELECT_V4_USAGE.contains("use fret_ui_shadcn::{self as shadcn"));
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
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_locals`"));
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_models`"));
        assert!(!FEARLESS_REFACTORING.contains("`ViewCx::on_action_notify_transient`"));
    }

    #[test]
    fn app_prelude_stays_explicit_instead_of_reexporting_legacy_surface() {
        let app_prelude = app_prelude_source();
        assert!(!app_prelude.contains("pub use crate::prelude::*;"));
        assert!(app_prelude.contains("pub use crate::{"));
        assert!(app_prelude.contains("pub use crate::app::App;"));
        assert!(app_prelude_exports_symbol("App"));
        assert!(app_prelude.contains("AppUi"));
        assert!(!app_prelude_exports_symbol("KernelApp"));
        assert!(app_prelude.contains("UiChild"));
        assert!(app_prelude.contains("WindowId"));
        assert!(app_prelude.contains("pub use fret_runtime::CommandId;"));
        assert!(app_prelude.contains("pub use fret_ui::ThemeSnapshot;"));
        assert!(app_prelude.contains("pub use fret_selector::{DepsSignature, ui::DepsBuilder};"));
        assert!(app_prelude.contains("pub use fret_ui_kit::declarative::icon;"));
        assert!(app_prelude.contains("pub use crate::view::TrackedStateExt as _;"));
        assert!(
            app_prelude.contains("pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;")
        );
        assert!(
            app_prelude.contains("pub use fret_ui_kit::declarative::ElementContextThemeExt as _;")
        );
        assert!(app_prelude.contains("pub use fret_ui_kit::declarative::UiElementA11yExt as _;"));
        assert!(
            app_prelude.contains("pub use fret_ui_kit::declarative::UiElementKeyContextExt as _;")
        );
        assert!(app_prelude.contains("pub use fret_ui_kit::declarative::UiElementTestIdExt as _;"));
        assert!(app_prelude.contains("pub use fret_ui_kit::StyledExt as _;"));
        assert!(app_prelude.contains("pub use fret_ui_kit::UiExt as _;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::declarative::prelude::*;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::IntoUiElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiIntoElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiHostBoundIntoElement;"));
        assert!(!app_prelude.contains("pub use fret_ui_kit::UiChildIntoElement;"));
        assert!(!app_prelude_exports_symbol("TrackedStateExt"));
        assert!(!app_prelude_exports_symbol("AnyElementSemanticsExt"));
        assert!(!app_prelude_exports_symbol("ElementContextThemeExt"));
        assert!(!app_prelude_exports_symbol("UiElementA11yExt"));
        assert!(!app_prelude_exports_symbol("UiElementKeyContextExt"));
        assert!(!app_prelude_exports_symbol("UiElementTestIdExt"));
        assert!(!app_prelude_exports_symbol("StyledExt"));
        assert!(!app_prelude_exports_symbol("UiExt"));
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
        assert!(advanced_prelude_exports_symbol("AppUiRawStateExt"));
        assert!(advanced_prelude_exports_symbol("AppUi"));
        assert!(advanced_prelude_exports_symbol("Ui"));
        assert!(advanced_prelude_exports_symbol("UiCx"));
        assert!(advanced_prelude_exports_symbol("ViewElements"));
        assert!(advanced_prelude_exports_symbol("ElementContext"));
        assert!(advanced_prelude_exports_symbol("UiTree"));
        assert!(advanced_prelude_exports_symbol("UiServices"));
        assert!(advanced_prelude_exports_symbol("TextProps"));
        assert!(!advanced_prelude_exports_symbol("ViewCx"));
        assert!(!advanced_prelude_exports_symbol("Elements"));
        assert!(
            !advanced_prelude
                .contains("pub use crate::view::{LocalState, TrackedStateExt, View, ViewCx};")
        );
        assert!(!advanced_prelude.contains(
            "pub use fret_ui::element::{Elements, HoverRegionProps, Length, SemanticsProps};"
        ));
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
        assert!(ACTIONS_RS.contains(
            "pub use fret_runtime::{ActionId, ActionMeta, ActionRegistry, TypedAction};"
        ));
        assert!(!app_prelude_exports_symbol("ActionMeta"));
        assert!(!app_prelude_exports_symbol("ActionRegistry"));
        assert!(!app_prelude.contains("ActionMeta"));
        assert!(!app_prelude.contains("ActionRegistry"));
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

        assert!(root_header.contains("pub mod icons {"));
        assert!(root_header.contains("pub mod style {"));
        assert!(root_header.contains("pub use fret_icons::IconId;"));
        assert!(root_header.contains("pub use fret_ui::Theme;"));
        assert!(root_header.contains("ChromeRefinement, ColorRef, LayoutRefinement, MetricRef"));
        assert!(root_header.contains("Radius, ShadowPreset, Size,"));
        assert!(root_header.contains("Space,"));
    }

    #[test]
    fn root_surface_exposes_explicit_env_module() {
        let root_header = root_surface_header_source();

        assert!(root_header.contains("pub mod env {"));
        assert!(root_header.contains("accent_color, container_breakpoints, container_query_region,"));
        assert!(root_header.contains("preferred_color_scheme, prefers_dark_color_scheme"));
        assert!(root_header.contains("safe_area_insets,"));
        assert!(root_header.contains("viewport_breakpoints, viewport_height_at_least"));
        assert!(root_header.contains("viewport_tailwind,"));
        assert!(root_header.contains("window_insets_padding_refinement_or_zero,"));
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
        assert!(!app_prelude_exports_symbol("ContainerQueryHysteresis"));
        assert!(!app_prelude_exports_symbol("ViewportQueryHysteresis"));
        assert!(!app_prelude_exports_symbol("ImageMetadata"));
        assert!(!app_prelude_exports_symbol("ImageMetadataStore"));
        assert!(!app_prelude_exports_symbol("ImageSamplingExt"));
        assert!(!app_prelude_exports_symbol("MarginEdge"));
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
    }

    #[test]
    fn component_prelude_is_curated_for_reusable_component_authors() {
        let component_prelude = component_prelude_source();
        assert!(component_prelude.contains("pub use crate::ComponentCx;"));
        assert!(component_prelude.contains("pub use fret_ui_kit::ui;"));
        assert!(component_prelude.contains("pub use fret_ui_kit::{"));
        assert!(component_prelude_exports_symbol("UiBuilder"));
        assert!(component_prelude_exports_symbol("UiPatchTarget"));
        assert!(component_prelude_exports_symbol("IntoUiElement"));
        assert!(component_prelude_exports_symbol("UiExt"));
        assert!(component_prelude_exports_symbol("AnyElement"));
        assert!(component_prelude_exports_symbol("UiHost"));
        assert!(component_prelude_exports_symbol("Invalidation"));
        assert!(component_prelude_exports_symbol("Theme"));
        assert!(component_prelude_exports_symbol("Model"));
        assert!(component_prelude_exports_symbol("CommandId"));
        assert!(component_prelude_exports_symbol("OverlayController"));
        assert!(component_prelude_exports_symbol("OverlayRequest"));
        assert!(component_prelude_exports_symbol("SemanticsRole"));
        assert!(!component_prelude.contains("pub use fret_ui_kit::prelude::*;"));
        assert!(!component_prelude_exports_symbol("UiIntoElement"));
        assert!(!component_prelude_exports_symbol("UiHostBoundIntoElement"));
        assert!(!component_prelude_exports_symbol("UiChildIntoElement"));
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
        assert!(!APP_ENTRY_RS.contains("pub struct App"));
    }
}
