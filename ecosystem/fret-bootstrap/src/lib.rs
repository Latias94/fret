#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::question_mark)]
#![allow(clippy::suspicious_open_options)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

//! Opinionated bootstrap utilities for Fret applications.
//!
//! This crate is intentionally *ecosystem-level* (not part of the portable kernel). It composes
//! existing primitives from `fret-launch` and friends to provide a convenient “golden path”
//! startup experience.
//!
//! ## Choosing an entry path
//!
//! - `ui_app(...)` / `ui_app_with_hooks(...)`: recommended author-facing path for general UI apps.
//! - `BootstrapBuilder::new_fn(...)` / `BootstrapBuilder::new_fn_with_hooks(...)`: recommended
//!   advanced path when you need runner-level control but still want the bootstrap/defaults story.
//! - `BootstrapBuilder::new(...)`: generic/compatibility path for existing low-level drivers that
//!   already implement `fret_launch::WinitAppDriver`, or for code that already holds a fully built
//!   driver value.
//!
//! Minimal example (native):
//!
//! ```no_run
//! use fret_app::App;
//! use fret_bootstrap::BootstrapBuilder;
//!
//! # fn event(_d: &mut (), _cx: fret_launch::WinitEventContext<'_, ()>, _e: &fret_core::Event) {}
//! # fn render(_d: &mut (), _cx: fret_launch::WinitRenderContext<'_, ()>) {}
//! #
//! let builder = BootstrapBuilder::new_fn(App::new(), (), |_d, _app, _w| (), event, render)
//!     .with_default_config_files()?
//!     .register_icon_pack(|_icons| {});
//! builder.run()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! UI app “golden path” example (native, requires the `ui-app-driver` feature):
//!
//! ```no_run
//! use fret_bootstrap::BootstrapBuilder;
//!
//! # #[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
//! # fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let builder = fret_bootstrap::ui_app(
//!     "todo",
//!     |_app, _window| (),
//!     |_cx, _state| fret_bootstrap::ui_app_driver::ViewElements::default(),
//! )
//!     .with_default_config_files()?
//!     .register_icon_pack(|_icons| {});
//! builder.run()?;
//! # Ok(())
//! # }
//! ```

use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use fret_app::SettingsFileV1;
#[cfg(not(target_arch = "wasm32"))]
use fret_app::config_files::LayeredConfigPaths;
use fret_app::{App, KeymapFileError, MenuBarFileError, SettingsError, TextInteractionSettings};
use fret_i18n::{I18nLookup, I18nService, LocaleId};
use fret_i18n_fluent::{FluentCatalog, FluentLookup};
#[cfg(not(target_arch = "wasm32"))]
use fret_icons::{
    IconPackRegistration, IconRegistry, InstalledIconPacks, panic_on_icon_pack_metadata_conflict,
    panic_on_icon_registry_freeze_failure,
};

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Keymap(#[from] KeymapFileError),
    #[error(transparent)]
    MenuBar(#[from] MenuBarFileError),
    #[error(transparent)]
    AssetManifest(#[from] fret_assets::AssetManifestLoadError),
    #[error(transparent)]
    AssetStartup(#[from] AssetStartupPlanError),
}

/// Broad bootstrap lifecycle stage for known startup/install failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootstrapKnownFailureStage {
    Builder,
    ExplicitInstall,
}

impl BootstrapKnownFailureStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Builder => "builder",
            Self::ExplicitInstall => "explicit_install",
        }
    }
}

/// Stable taxonomy for bootstrap-level startup/install failures that first-party diagnostics
/// should recognize without pattern-matching ad-hoc panic text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootstrapKnownFailureKind {
    SettingsRead,
    SettingsParse,
    KeymapRead,
    KeymapParse,
    MenuBarRead,
    MenuBarParse,
    AssetManifestRead,
    AssetManifestParse,
    AssetManifestSerialize,
    AssetManifestWrite,
    AssetBundleRootRead,
    AssetManifestInvalid,
    AssetManifestDuplicateBundleKey,
    AssetStartupMissingDevelopmentLane,
    AssetStartupMissingPackagedLane,
    IconInstallRegistryFreezeFailed,
    IconInstallMetadataConflict,
}

impl BootstrapKnownFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SettingsRead => "settings_read",
            Self::SettingsParse => "settings_parse",
            Self::KeymapRead => "keymap_read",
            Self::KeymapParse => "keymap_parse",
            Self::MenuBarRead => "menu_bar_read",
            Self::MenuBarParse => "menu_bar_parse",
            Self::AssetManifestRead => "asset_manifest_read",
            Self::AssetManifestParse => "asset_manifest_parse",
            Self::AssetManifestSerialize => "asset_manifest_serialize",
            Self::AssetManifestWrite => "asset_manifest_write",
            Self::AssetBundleRootRead => "asset_bundle_root_read",
            Self::AssetManifestInvalid => "asset_manifest_invalid",
            Self::AssetManifestDuplicateBundleKey => "asset_manifest_duplicate_bundle_key",
            Self::AssetStartupMissingDevelopmentLane => "asset_startup_missing_development_lane",
            Self::AssetStartupMissingPackagedLane => "asset_startup_missing_packaged_lane",
            Self::IconInstallRegistryFreezeFailed => "icon_install_registry_freeze_failed",
            Self::IconInstallMetadataConflict => "icon_install_metadata_conflict",
        }
    }
}

/// Structured report for bootstrap failures that should stay recognizable across returned errors
/// and panic-only install paths.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BootstrapKnownFailureReport {
    pub stage: BootstrapKnownFailureStage,
    pub kind: BootstrapKnownFailureKind,
    pub surface: Option<&'static str>,
    pub pack_id: Option<&'static str>,
    pub summary: String,
    pub details: Vec<String>,
}

impl BootstrapKnownFailureReport {
    pub fn from_bootstrap_error(error: &BootstrapError) -> Self {
        match error {
            BootstrapError::Settings(error) => match error {
                SettingsError::Read { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::SettingsRead,
                    Some("settings"),
                    format!("failed to read settings file `{path}`"),
                    source,
                ),
                SettingsError::Parse { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::SettingsParse,
                    Some("settings"),
                    format!("failed to parse settings file `{path}`"),
                    source,
                ),
            },
            BootstrapError::Keymap(error) => match error {
                KeymapFileError::Read { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::KeymapRead,
                    Some("keymap"),
                    format!("failed to read keymap file `{path}`"),
                    source,
                ),
                KeymapFileError::Parse { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::KeymapParse,
                    Some("keymap"),
                    format!("failed to parse keymap file `{path}`"),
                    source,
                ),
            },
            BootstrapError::MenuBar(error) => match error {
                MenuBarFileError::Read { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::MenuBarRead,
                    Some("menu_bar"),
                    format!("failed to read menubar file `{path}`"),
                    source,
                ),
                MenuBarFileError::Parse { path, source } => Self::builder_with_source(
                    BootstrapKnownFailureKind::MenuBarParse,
                    Some("menu_bar"),
                    format!("failed to parse menubar file `{path}`"),
                    source,
                ),
            },
            BootstrapError::AssetManifest(error) => Self::from_asset_manifest_error(error),
            BootstrapError::AssetStartup(error) => Self::from_asset_startup_error(error),
        }
    }

    pub fn from_asset_manifest_error(error: &fret_assets::AssetManifestLoadError) -> Self {
        match error {
            fret_assets::AssetManifestLoadError::ReadManifest { path, source } => {
                Self::builder_with_source(
                    BootstrapKnownFailureKind::AssetManifestRead,
                    Some("asset_manifest"),
                    format!("failed to read asset manifest `{}`", path.display()),
                    source,
                )
            }
            fret_assets::AssetManifestLoadError::ParseManifest { path, source } => {
                Self::builder_with_source(
                    BootstrapKnownFailureKind::AssetManifestParse,
                    Some("asset_manifest"),
                    format!("failed to parse asset manifest `{}`", path.display()),
                    source,
                )
            }
            fret_assets::AssetManifestLoadError::SerializeManifest { path, source } => {
                Self::builder_with_source(
                    BootstrapKnownFailureKind::AssetManifestSerialize,
                    Some("asset_manifest"),
                    format!("failed to serialize asset manifest `{}`", path.display()),
                    source,
                )
            }
            fret_assets::AssetManifestLoadError::WriteManifest { path, source } => {
                Self::builder_with_source(
                    BootstrapKnownFailureKind::AssetManifestWrite,
                    Some("asset_manifest"),
                    format!("failed to write asset manifest `{}`", path.display()),
                    source,
                )
            }
            fret_assets::AssetManifestLoadError::ReadBundleRoot { path, source } => {
                Self::builder_with_source(
                    BootstrapKnownFailureKind::AssetBundleRootRead,
                    Some("asset_manifest"),
                    format!("failed to read asset bundle root `{}`", path.display()),
                    source,
                )
            }
            fret_assets::AssetManifestLoadError::InvalidManifest { message } => Self::builder(
                BootstrapKnownFailureKind::AssetManifestInvalid,
                Some("asset_manifest"),
                "invalid asset manifest",
                vec![message.to_string()],
            ),
            fret_assets::AssetManifestLoadError::DuplicateBundleKey { bundle, key } => {
                Self::builder(
                    BootstrapKnownFailureKind::AssetManifestDuplicateBundleKey,
                    Some("asset_manifest"),
                    "duplicate asset manifest entry",
                    vec![
                        format!("bundle: {}", bundle.as_str()),
                        format!("key: {}", key.as_str()),
                    ],
                )
            }
        }
    }

    pub fn from_asset_startup_error(error: &AssetStartupPlanError) -> Self {
        match error {
            AssetStartupPlanError::MissingDevelopmentLane => Self::builder(
                BootstrapKnownFailureKind::AssetStartupMissingDevelopmentLane,
                Some("asset_startup"),
                "asset startup plan is missing a development lane",
                Vec::new(),
            ),
            AssetStartupPlanError::MissingPackagedLane => Self::builder(
                BootstrapKnownFailureKind::AssetStartupMissingPackagedLane,
                Some("asset_startup"),
                "asset startup plan is missing a packaged lane",
                Vec::new(),
            ),
        }
    }

    pub fn from_icon_install_failure(report: &fret_icons::IconInstallFailureReport) -> Self {
        match report.kind {
            fret_icons::IconInstallFailureKind::RegistryFreezeFailed => Self::explicit_install(
                BootstrapKnownFailureKind::IconInstallRegistryFreezeFailed,
                Some(report.surface),
                report.pack_id,
                match report.pack_id {
                    Some(pack_id) => {
                        format!("failed to freeze icon registry for pack `{pack_id}`")
                    }
                    None => {
                        "failed to freeze icon registry during explicit icon install".to_string()
                    }
                },
                report.details.clone(),
            ),
            fret_icons::IconInstallFailureKind::MetadataConflict => Self::explicit_install(
                BootstrapKnownFailureKind::IconInstallMetadataConflict,
                Some(report.surface),
                report.pack_id,
                match report.pack_id {
                    Some(pack_id) => {
                        format!("conflicting installed icon pack metadata for pack `{pack_id}`")
                    }
                    None => "conflicting installed icon pack metadata during explicit icon install"
                        .to_string(),
                },
                report.details.clone(),
            ),
        }
    }

    fn builder(
        kind: BootstrapKnownFailureKind,
        surface: Option<&'static str>,
        summary: impl Into<String>,
        details: Vec<String>,
    ) -> Self {
        Self {
            stage: BootstrapKnownFailureStage::Builder,
            kind,
            surface,
            pack_id: None,
            summary: summary.into(),
            details,
        }
    }

    fn builder_with_source(
        kind: BootstrapKnownFailureKind,
        surface: Option<&'static str>,
        summary: impl Into<String>,
        source: &impl std::fmt::Display,
    ) -> Self {
        Self::builder(kind, surface, summary, vec![format!("source: {source}")])
    }

    fn explicit_install(
        kind: BootstrapKnownFailureKind,
        surface: Option<&'static str>,
        pack_id: Option<&'static str>,
        summary: impl Into<String>,
        details: Vec<String>,
    ) -> Self {
        Self {
            stage: BootstrapKnownFailureStage::ExplicitInstall,
            kind,
            surface,
            pack_id,
            summary: summary.into(),
            details,
        }
    }
}

impl BootstrapError {
    pub fn known_failure_report(&self) -> BootstrapKnownFailureReport {
        BootstrapKnownFailureReport::from_bootstrap_error(self)
    }
}

pub use fret_launch::assets::{
    AssetReloadPolicy, AssetStartupMode, AssetStartupPlan, AssetStartupPlanError,
};

/// Explicit logical asset vocabulary and host registration helpers for `fret-bootstrap` users.
pub mod assets {
    pub use fret_launch::assets::*;
}

/// Install the default shadcn command palette overlay on a bootstrap UI driver.
#[cfg(feature = "ui-app-command-palette-shadcn")]
pub fn with_shadcn_command_palette<S>(
    driver: ui_app_driver::UiAppDriver<S>,
) -> ui_app_driver::UiAppDriver<S> {
    driver.command_palette_overlay(render_shadcn_command_palette_overlay)
}

/// Render the default shadcn command palette overlay.
#[cfg(feature = "ui-app-command-palette-shadcn")]
pub fn render_shadcn_command_palette_overlay(
    cx: &mut fret_ui::ElementContext<'_, App>,
    overlay: ui_app_driver::CommandPaletteOverlayCx,
    out: &mut ui_app_driver::ViewElements,
) {
    use fret_ui_shadcn::facade as shadcn;

    let entries: Vec<shadcn::CommandEntry> = if overlay.open {
        fret_ui_kit::command::command_catalog_entries_from_host_commands_with_options(
            cx,
            fret_ui_kit::command::CommandCatalogOptions::default(),
        )
        .into_iter()
        .map(Into::into)
        .collect()
    } else {
        Vec::new()
    };

    let dialog = shadcn::CommandDialog::new(overlay.models.open, overlay.models.query, Vec::new())
        .entries(entries)
        .a11y_label("Command palette")
        .into_element(cx, |cx| {
            cx.interactivity_gate_props(
                fret_ui::element::InteractivityGateProps {
                    present: false,
                    interactive: false,
                    ..Default::default()
                },
                |_| vec![],
            )
        });
    out.push(dialog);
}

/// Apply `SettingsFileV1` to an `App` and runner config.
///
/// This is a convenience helper for the common pattern:
/// - apply docking interaction settings to `App` globals
/// - apply font family overrides to runner config
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_settings(
    app: &mut App,
    config: &mut fret_launch::WinitRunnerConfig,
    settings: &SettingsFileV1,
) {
    install_default_i18n_backend(app);
    fret_app::settings::apply_settings_globals(app, settings);
    config.text_font_families = settings.fonts.clone();
}

/// Installs a default i18n backend if the app hasn't provided one yet.
///
/// Notes:
/// - This is an ecosystem-level “golden path” convenience, not a portable kernel contract.
/// - Callers can override this by installing their own backend before bootstrapping.
pub fn install_default_i18n_backend(app: &mut App) {
    let mut service = app
        .global::<I18nService>()
        .cloned()
        .unwrap_or_else(I18nService::default);
    if service.lookup().is_some() {
        return;
    }

    service.set_lookup(Some(default_i18n_lookup()));
    app.set_global(service);
}

fn diagnostics_env_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|value| !value.is_empty())
        || std::env::var_os("FRET_DIAG_DIR").is_some_and(|value| !value.is_empty())
}

fn install_default_text_interaction_settings_with(app: &mut App, diagnostics_active: bool) {
    if app.global::<TextInteractionSettings>().is_some() {
        return;
    }

    app.set_global(TextInteractionSettings {
        // Desktop-facing bootstrap paths should feel like a normal product by default, while
        // diagnostics runs stay deterministic unless an app opts back into blinking explicitly.
        caret_blink: !diagnostics_active,
        ..Default::default()
    });
}

/// Installs bootstrap-level text interaction defaults when the app has not provided them yet.
///
/// This intentionally lives in the ecosystem bootstrap layer rather than `fret_app::App::new()`:
/// - low-level apps/tests keep deterministic kernel defaults,
/// - desktop "golden path" apps get editor-like caret blink by default,
/// - diagnostics runs remain stable by disabling blink when `FRET_DIAG*` is active.
pub fn install_default_text_interaction_settings(app: &mut App) {
    install_default_text_interaction_settings_with(app, diagnostics_env_enabled());
}

fn default_i18n_lookup() -> Rc<dyn I18nLookup + 'static> {
    let mut catalog = FluentCatalog::new();
    catalog
        .add_locale_ftl(
            LocaleId::parse("en-US").expect("hardcoded locale en-US must parse"),
            DEFAULT_I18N_FTL_EN_US,
        )
        .expect("en-US i18n resource must load");
    catalog
        .add_locale_ftl(
            LocaleId::parse("zh-CN").expect("hardcoded locale zh-CN must parse"),
            DEFAULT_I18N_FTL_ZH_CN,
        )
        .expect("zh-CN i18n resource must load");

    let lookup = FluentLookup::new(Rc::new(catalog));
    Rc::new(lookup)
}

const DEFAULT_I18N_FTL_EN_US: &str = r#"
core-command-category-app = App
workspace-menu-file = File
workspace-menu-edit = Edit
workspace-menu-view = View
workspace-menu-window = Window

core-command-title-app-command-palette = Command Palette
core-command-title-app-about = About
core-command-title-app-preferences = Preferences...
core-command-title-app-locale-switch-next = Switch Language
core-command-title-app-hide = Hide
core-command-title-app-hide-others = Hide Others
core-command-title-app-show-all = Show All
core-command-title-app-quit = Quit
"#;

const DEFAULT_I18N_FTL_ZH_CN: &str = r#"
core-command-category-app = 应用
workspace-menu-file = 文件
workspace-menu-edit = 编辑
workspace-menu-view = 视图
workspace-menu-window = 窗口

core-command-title-app-command-palette = 命令面板
core-command-title-app-about = 关于
core-command-title-app-preferences = 偏好设置...
core-command-title-app-locale-switch-next = 切换语言
core-command-title-app-hide = 隐藏
core-command-title-app-hide-others = 隐藏其他应用
core-command-title-app-show-all = 显示全部
core-command-title-app-quit = 退出
"#;

/// Builder wrapper around `fret_launch::WinitAppBuilder` with common bootstrapping conveniences.
///
/// Entry guidance:
/// - prefer `ui_app(...)` / `ui_app_with_hooks(...)` for app-author-facing UI code,
/// - prefer `BootstrapBuilder::new_fn(...)` for new advanced integrations,
/// - use `BootstrapBuilder::new(...)` for generic/compatibility low-level driver integration.
#[cfg(not(target_arch = "wasm32"))]
pub struct BootstrapBuilder<D> {
    inner: fret_launch::WinitAppBuilder<D>,
    on_gpu_ready_hooks: Vec<
        Box<dyn FnOnce(&mut App, &fret_render::WgpuContext, &mut fret_render::Renderer) + 'static>,
    >,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: fret_launch::WinitAppDriver + 'static> BootstrapBuilder<D> {
    /// Create a bootstrap builder from an already-constructed low-level driver.
    ///
    /// Prefer `new_fn(...)` for new advanced integrations and `ui_app(...)` for general app code.
    /// This constructor remains useful for compatibility-oriented code that still implements
    /// `fret_launch::WinitAppDriver` directly, or for callers that already have a concrete driver
    /// value and simply want the bootstrap/defaults layer.
    pub fn new(mut app: App, driver: D) -> Self {
        install_default_text_interaction_settings(&mut app);
        Self {
            inner: fret_launch::WinitAppBuilder::new(app, driver),
            on_gpu_ready_hooks: Vec::new(),
        }
    }

    pub fn with_settings_json(mut self, path: impl AsRef<Path>) -> Result<Self, BootstrapError> {
        let path = path.as_ref();
        let Some(settings) = SettingsFileV1::load_json_if_exists(path)? else {
            return Ok(self);
        };

        let settings_for_config = settings.clone();
        self.inner = self.inner.configure(move |config| {
            config.text_font_families = settings_for_config.fonts.clone();
        });

        self.inner = self.inner.init_app(move |app| {
            install_default_i18n_backend(app);
            fret_app::settings::apply_settings_globals(app, &settings);
            fret_app::sync_os_menu_bar(app);
        });

        Ok(self)
    }

    pub fn with_default_settings_json(self) -> Result<Self, BootstrapError> {
        self.with_settings_json(".fret/settings.json")
    }

    pub fn with_layered_settings(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (settings, _report) = fret_app::load_layered_settings(&paths)?;

        let settings_for_config = settings.clone();
        self.inner = self.inner.configure(move |config| {
            config.text_font_families = settings_for_config.fonts.clone();
        });

        self.inner = self.inner.init_app(move |app| {
            install_default_i18n_backend(app);
            fret_app::settings::apply_settings_globals(app, &settings);
            fret_app::sync_os_menu_bar(app);
        });

        Ok(self)
    }

    pub fn with_layered_keymap(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (layered, _report) = fret_app::load_layered_keymap(&paths)?;

        self.inner = self.inner.init_app(move |app| {
            fret_app::apply_layered_keymap(app, layered.clone());
        });

        Ok(self)
    }

    pub fn with_layered_menu_bar(
        mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let paths = LayeredConfigPaths::for_project_root(project_root);
        let (layered, _report) = fret_app::load_layered_menu_bar(&paths)?;

        self.inner = self.inner.init_app(move |app| {
            if let Err(e) = fret_app::apply_layered_menu_bar(app, None, layered.clone()) {
                app.with_global_mut(
                    fret_app::ConfigFilesWatcherStatus::default,
                    |status, _app| {
                        status.note(fret_app::ConfigFilesWatcherTick {
                            reloaded_settings: false,
                            reloaded_keymap: false,
                            reloaded_menu_bar: false,
                            settings_error: None,
                            keymap_error: None,
                            menu_bar_error: Some(e.to_string()),
                            actionable_keymap_conflicts: 0,
                            keymap_conflict_samples: Vec::new(),
                        });
                    },
                );
            }
        });

        Ok(self)
    }

    /// Installs command-provided default keybindings into the app keymap.
    ///
    /// Ordering note: call this before `with_layered_keymap(...)` so user/project keymap files can
    /// override defaults via last-wins resolution.
    pub fn with_command_default_keybindings(mut self) -> Self {
        self.inner = self.inner.init_app(move |app| {
            fret_app::install_command_default_keybindings_into_keymap(app);
        });
        self
    }

    /// Installs a set of plugins into the app-owned registry (ADR 0016).
    ///
    /// Ordering note: for correct keymap layering (ADR 0021), prefer calling this before
    /// `with_layered_keymap(...)` / `with_default_config_files()` so user/project overrides remain
    /// last-wins.
    pub fn with_plugins(mut self, plugins: &[&dyn fret_app::Plugin]) -> Self {
        let plugins: Vec<&dyn fret_app::Plugin> = plugins.to_vec();
        self.inner = self.inner.init_app(move |app| {
            fret_app::install_plugins(app, plugins.iter().copied());
        });
        self
    }

    pub fn with_default_config_files(self) -> Result<Self, BootstrapError> {
        self.with_layered_settings(".")?
            .with_command_default_keybindings()
            .with_layered_keymap(".")?
            .with_layered_menu_bar(".")
    }

    pub fn with_default_config_files_for_root(
        self,
        project_root: impl AsRef<Path>,
    ) -> Result<Self, BootstrapError> {
        let project_root = project_root.as_ref();
        self.with_layered_settings(project_root)?
            .with_command_default_keybindings()
            .with_layered_keymap(project_root)?
            .with_layered_menu_bar(project_root)
    }

    /// Enables polling-based hot reload for layered `settings.json` / `keymap.json` / `menubar.json` files.
    ///
    /// This uses a repeating `Effect::SetTimer` and checks file metadata (mtime/len) on each tick.
    /// It is intended for local dev workflows and stays portable (no platform-specific watcher deps).
    pub fn with_config_files_watcher(self, poll_interval: Duration) -> Self {
        self.with_config_files_watcher_for_root(poll_interval, ".")
    }

    pub fn with_config_files_watcher_for_root(
        mut self,
        poll_interval: Duration,
        project_root: impl AsRef<Path>,
    ) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        self.inner = self.inner.init_app(move |app| {
            fret_app::ConfigFilesWatcher::install(app, poll_interval, &project_root);
        });
        self
    }

    /// Apply one explicit development-vs-packaged startup plan on the builder path.
    pub fn with_asset_startup(
        self,
        app_bundle: impl Into<fret_assets::AssetBundleId>,
        mode: AssetStartupMode,
        plan: AssetStartupPlan,
    ) -> Result<Self, BootstrapError> {
        Ok(Self {
            inner: self
                .inner
                .with_asset_startup(app_bundle, mode, plan)
                .map_err(map_asset_runner_error)?,
            on_gpu_ready_hooks: self.on_gpu_ready_hooks,
        })
    }

    /// Enable development asset reload polling for file-backed startup mounts.
    pub fn with_asset_reload_policy(self, policy: fret_launch::assets::AssetReloadPolicy) -> Self {
        Self {
            inner: self.inner.with_asset_reload_policy(policy),
            on_gpu_ready_hooks: self.on_gpu_ready_hooks,
        }
    }

    /// Configure budgets for UI render asset caches (`ImageAssetCache` / `SvgAssetCache`).
    ///
    /// This is an ecosystem-level convenience; it does not change the core "resource handles" boundary (ADR 0004).
    #[cfg(feature = "ui-assets")]
    pub fn with_ui_assets_budgets(
        mut self,
        image_budget_bytes: u64,
        image_max_ready_entries: usize,
        svg_budget_bytes: u64,
        svg_max_ready_entries: usize,
    ) -> Self {
        let budgets = fret_ui_assets::UiAssetsBudgets {
            image_budget_bytes,
            image_max_ready_entries,
            svg_budget_bytes,
            svg_max_ready_entries,
        };
        self.inner = self.inner.init_app(move |app| {
            fret_ui_assets::app::configure_caches_with_budgets(app, budgets);
        });

        self
    }

    /// Enable the `fret-launch` dev hotpatch trigger by setting environment variables.
    ///
    /// This is intended for local developer workflows; production apps should not rely on it.
    ///
    /// # Safety
    ///
    /// `std::env::set_var` is unsafe on Rust 2024 because mutating the process environment while
    /// other threads may read it can cause undefined behavior on some platforms.
    /// Call this early during startup, before any other threads are spawned.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_env(
        self,
        trigger_path: impl AsRef<Path>,
        poll_interval_ms: u64,
    ) -> Self {
        let trigger_path = trigger_path.as_ref();

        // Safety: the caller must ensure no other threads concurrently read/write the process
        // environment while these variables are being set.
        unsafe {
            std::env::set_var("FRET_HOTPATCH", "1");
            std::env::set_var("FRET_HOTPATCH_TRIGGER_PATH", trigger_path.as_os_str());
            std::env::set_var("FRET_HOTPATCH_POLL_MS", poll_interval_ms.to_string());
        }

        self
    }

    /// Enable the `fret-launch` dev hotpatch trigger using a file-based polling marker.
    ///
    /// This is a clearer name for `enable_hotpatch_env`.
    ///
    /// # Safety
    ///
    /// See `enable_hotpatch_env`.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_file_trigger_env(
        self,
        trigger_path: impl AsRef<Path>,
        poll_interval_ms: u64,
    ) -> Self {
        unsafe { self.enable_hotpatch_env(trigger_path, poll_interval_ms) }
    }

    /// Enable Subsecond hotpatch by connecting to a devserver websocket.
    ///
    /// The runner will listen for Dioxus-style devserver messages and apply incoming Subsecond
    /// jump tables. Once a patch is applied, the runner schedules a safe hot-reload reset on the
    /// next event-loop turn.
    ///
    /// # Safety
    ///
    /// `std::env::set_var` is unsafe on Rust 2024 because mutating the process environment while
    /// other threads may read it can cause undefined behavior on some platforms.
    /// Call this early during startup, before any other threads are spawned.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_subsecond_devserver_env(
        self,
        devserver_ws_endpoint: impl AsRef<str>,
    ) -> Self {
        let endpoint = devserver_ws_endpoint.as_ref();

        unsafe {
            std::env::set_var("FRET_HOTPATCH", "1");
            std::env::set_var("FRET_HOTPATCH_DEVSERVER_WS", endpoint);
        }

        self
    }

    /// Same as `enable_hotpatch_subsecond_devserver_env`, but additionally sets a build-id filter.
    ///
    /// When `build_id` is set, the runner will ignore devserver patches whose `for_build_id` does
    /// not match, which helps avoid cross-process confusion in multi-app workflows.
    ///
    /// # Safety
    ///
    /// See `enable_hotpatch_subsecond_devserver_env`.
    #[cfg(feature = "hotpatch-subsecond")]
    pub unsafe fn enable_hotpatch_subsecond_devserver_env_with_build_id(
        self,
        devserver_ws_endpoint: impl AsRef<str>,
        build_id: u64,
    ) -> Self {
        let builder =
            unsafe { self.enable_hotpatch_subsecond_devserver_env(devserver_ws_endpoint) };

        unsafe {
            std::env::set_var("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
        }

        builder
    }

    /// Register an icon pack through the explicit pack contract.
    ///
    /// Prefer this when a pack crate exports `PACK` / `VENDOR_PACK` style registration values
    /// together with explicit metadata/provenance.
    pub fn register_icon_pack_contract(mut self, pack: IconPackRegistration) -> Self {
        self.inner = self.inner.init_app(move |app| {
            app.with_global_mut(IconRegistry::default, |icons, app| {
                pack.register_into_registry(icons);
                let frozen = icons.freeze().unwrap_or_else(|errors| {
                    panic_on_icon_registry_freeze_failure(
                        "fret_bootstrap.register_icon_pack_contract",
                        Some(pack.metadata.pack_id),
                        errors,
                    )
                });
                app.set_global(frozen);
            });
            app.with_global_mut(InstalledIconPacks::default, |installed, _app| {
                installed.record(pack.metadata).unwrap_or_else(|err| {
                    panic_on_icon_pack_metadata_conflict(
                        "fret_bootstrap.register_icon_pack_contract",
                        err,
                    )
                });
            });
        });
        self
    }

    /// Register an icon pack (e.g. `fret_icons_lucide::register_icons`) into the global `IconRegistry`.
    ///
    /// This is the raw registry-only escape hatch. Prefer [`Self::register_icon_pack_contract`]
    /// when a pack crate exports explicit pack metadata.
    pub fn register_icon_pack(mut self, register: fn(&mut IconRegistry)) -> Self {
        self.inner = self.inner.init_app(move |app| {
            app.with_global_mut(IconRegistry::default, |icons, app| {
                register(icons);
                let frozen = icons.freeze().unwrap_or_else(|errors| {
                    panic_on_icon_registry_freeze_failure(
                        "fret_bootstrap.register_icon_pack",
                        None,
                        errors,
                    )
                });
                app.set_global(frozen);
            });
        });
        self
    }

    /// Install the Lucide icon pack into the global `IconRegistry`.
    ///
    /// Requires enabling the `fret-bootstrap/icons-lucide` feature.
    #[cfg(feature = "icons-lucide")]
    pub fn with_lucide_icons(self) -> Self {
        let builder = self.register_icon_pack_contract(fret_icons_lucide::VENDOR_PACK);

        #[cfg(feature = "icons-ui-semantic-lucide")]
        let builder =
            builder.register_icon_pack_contract(fret_icons_lucide::UI_SEMANTIC_ALIAS_PACK);

        builder
    }

    /// Install the Radix icon pack into the global `IconRegistry`.
    ///
    /// Requires enabling the `fret-bootstrap/icons-radix` feature.
    #[cfg(feature = "icons-radix")]
    pub fn with_radix_icons(self) -> Self {
        let builder = self.register_icon_pack_contract(fret_icons_radix::VENDOR_PACK);

        // If both semantic providers are enabled, prefer Lucide's `ui.*` aliases to keep a stable
        // default (Lucide is the `fret` crate's default icon pack).
        #[cfg(all(
            feature = "icons-ui-semantic-radix",
            not(feature = "icons-ui-semantic-lucide")
        ))]
        let builder = builder.register_icon_pack_contract(fret_icons_radix::UI_SEMANTIC_ALIAS_PACK);

        builder
    }

    /// Pre-register all SVG icons from the global `IconRegistry` during `on_gpu_ready`.
    #[cfg(feature = "preload-icon-svgs")]
    pub fn preload_icon_svgs_on_gpu_ready(mut self) -> Self {
        self.on_gpu_ready_hooks
            .push(Box::new(|app, _context, renderer| {
                let services = renderer as &mut dyn fret_core::UiServices;
                fret_ui_kit::declarative::icon::preload_icon_svgs(app, services);
            }));
        self
    }

    pub fn configure(mut self, f: impl FnOnce(&mut fret_launch::WinitRunnerConfig)) -> Self {
        self.inner = self.inner.configure(f);
        self
    }

    /// Initialize a default tracing subscriber (if one is not already installed).
    ///
    /// Controlled by `RUST_LOG` when set; otherwise uses a conservative default filter suitable for
    /// app development.
    #[cfg(feature = "tracing")]
    pub fn with_default_tracing(self) -> Self {
        init_tracing();
        self
    }

    /// Initialize default diagnostics (tracing + panic logging) for application development.
    #[cfg(feature = "diagnostics")]
    pub fn with_default_diagnostics(self) -> Self {
        init_diagnostics();
        self
    }

    /// Configure the main window title and initial size (logical pixels).
    pub fn with_main_window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        let title = title.into();
        let (width, height) = size;

        let title_for_global = title.clone();
        self.inner = self.inner.init_app(move |app| {
            app.set_global(fret_app::AppDisplayName(Arc::from(
                title_for_global.clone(),
            )));
        });

        self.inner = self.inner.configure(move |config| {
            config.main_window_title = title.clone();
            config.main_window_size.width = width;
            config.main_window_size.height = height;
        });

        self
    }

    /// Configure the minimum logical surface size for the main window.
    pub fn with_main_window_min_size(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.main_window_min_size = Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the maximum logical surface size for the main window.
    pub fn with_main_window_max_size(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.main_window_max_size = Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the surface resize increments for the main window.
    pub fn with_main_window_resize_increments(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.main_window_resize_increments =
                Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the initial logical screen position for the main window.
    pub fn with_main_window_position_logical(mut self, position: (i32, i32)) -> Self {
        let (x, y) = position;
        self.inner = self.inner.configure(move |config| {
            config.main_window_position = Some(fret_launch::WindowPosition::Logical(
                fret_core::WindowLogicalPosition { x, y },
            ));
        });
        self
    }

    /// Configure the initial physical screen position for the main window.
    pub fn with_main_window_position_physical(mut self, position: (i32, i32)) -> Self {
        let (x, y) = position;
        self.inner = self.inner.configure(move |config| {
            config.main_window_position = Some(fret_launch::WindowPosition::Physical(
                fret_launch::WindowPhysicalPosition::new(x, y),
            ));
        });
        self
    }

    /// Configure whether the main window can be resized by the OS chrome.
    pub fn with_main_window_resizable(mut self, resizable: bool) -> Self {
        self.inner = self.inner.configure(move |config| {
            config.main_window_style.resizable = Some(resizable);
        });
        self
    }

    /// Configure the fallback title and initial size for newly created auxiliary windows.
    pub fn with_default_window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        let title = title.into();
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.default_window_title = title.clone();
            config.default_window_size = fret_launch::WindowLogicalSize::new(width, height);
        });
        self
    }

    /// Configure the minimum logical surface size for fallback-created auxiliary windows.
    pub fn with_default_window_min_size(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.default_window_min_size =
                Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the maximum logical surface size for fallback-created auxiliary windows.
    pub fn with_default_window_max_size(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.default_window_max_size =
                Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the surface resize increments for fallback-created auxiliary windows.
    pub fn with_default_window_resize_increments(mut self, size: (f64, f64)) -> Self {
        let (width, height) = size;
        self.inner = self.inner.configure(move |config| {
            config.default_window_resize_increments =
                Some(fret_launch::WindowLogicalSize::new(width, height));
        });
        self
    }

    /// Configure the initial logical screen position for fallback-created auxiliary windows.
    pub fn with_default_window_position_logical(mut self, position: (i32, i32)) -> Self {
        let (x, y) = position;
        self.inner = self.inner.configure(move |config| {
            config.default_window_position = Some(fret_launch::WindowPosition::Logical(
                fret_core::WindowLogicalPosition { x, y },
            ));
        });
        self
    }

    /// Configure the initial physical screen position for fallback-created auxiliary windows.
    pub fn with_default_window_position_physical(mut self, position: (i32, i32)) -> Self {
        let (x, y) = position;
        self.inner = self.inner.configure(move |config| {
            config.default_window_position = Some(fret_launch::WindowPosition::Physical(
                fret_launch::WindowPhysicalPosition::new(x, y),
            ));
        });
        self
    }

    pub fn init_app(mut self, f: impl FnOnce(&mut App)) -> Self {
        self.inner = self.inner.init_app(f);
        self
    }

    /// Register a best-effort UI diagnostics debug extension writer.
    ///
    /// Requires enabling `fret-bootstrap/ui-app-driver` and `fret-bootstrap/diagnostics`.
    #[cfg(all(feature = "ui-app-driver", feature = "diagnostics"))]
    pub fn register_diag_debug_extension(
        mut self,
        key: impl Into<String>,
        writer: impl Fn(&App, fret_core::AppWindowId) -> Option<serde_json::Value> + 'static,
    ) -> Self {
        let key = key.into();
        let writer: crate::ui_diagnostics::UiDebugExtensionWriterV1 = Arc::new(writer);
        self.inner = self.inner.init_app(move |app| {
            crate::ui_diagnostics::register_debug_extension_best_effort(app, key, writer);
        });
        self
    }

    /// Install an ecosystem crate that only needs access to the app state.
    ///
    /// This runs during early initialization (before GPU services exist), which is important for
    /// correct keymap layering semantics (user/project keymaps should remain last-wins).
    pub fn install_app(mut self, install: fn(&mut App)) -> Self {
        self.inner = self.inner.init_app(install);
        self
    }

    /// Install an ecosystem crate at the UI services boundary.
    ///
    /// This runs during `on_gpu_ready`, with `services` backed by the renderer.
    pub fn install(mut self, install: fn(&mut App, &mut dyn fret_core::UiServices)) -> Self {
        self.on_gpu_ready_hooks
            .push(Box::new(move |app, _context, renderer| {
                let services = renderer as &mut dyn fret_core::UiServices;
                install(app, services);
            }));
        self
    }

    /// Install an ecosystem crate at the custom effects boundary (ADR 0299).
    ///
    /// This runs during `on_gpu_ready`, with `effects` backed by the renderer.
    pub fn install_custom_effects(
        mut self,
        install: fn(&mut App, &mut dyn fret_core::CustomEffectService),
    ) -> Self {
        self.on_gpu_ready_hooks
            .push(Box::new(move |app, _context, renderer| {
                let effects = renderer as &mut dyn fret_core::CustomEffectService;
                install(app, effects);
            }));
        self
    }

    pub fn on_main_window_created(
        mut self,
        f: impl FnOnce(&mut App, fret_core::AppWindowId) + 'static,
    ) -> Self {
        self.inner = self.inner.on_main_window_created(f);
        self
    }

    pub fn on_gpu_ready(
        mut self,
        f: impl FnOnce(&mut App, &fret_render::WgpuContext, &mut fret_render::Renderer) + 'static,
    ) -> Self {
        self.on_gpu_ready_hooks.push(Box::new(f));
        self
    }

    pub fn run(self) -> Result<(), fret_launch::RunnerError> {
        self.into_inner().run()
    }

    pub fn into_inner(self) -> fret_launch::WinitAppBuilder<D> {
        let BootstrapBuilder {
            mut inner,
            on_gpu_ready_hooks,
        } = self;

        if on_gpu_ready_hooks.is_empty() {
            return inner;
        }

        inner = inner.on_gpu_ready(move |app, context, renderer| {
            for hook in on_gpu_ready_hooks {
                hook(app, context, renderer);
            }
        });

        inner
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn map_asset_runner_error(err: fret_launch::RunnerError) -> BootstrapError {
    match err {
        fret_launch::RunnerError::AssetManifest(source) => BootstrapError::AssetManifest(source),
        fret_launch::RunnerError::AssetStartup(source) => BootstrapError::AssetStartup(source),
        other => unreachable!(
            "unexpected non-asset runner error while configuring bootstrap assets: {other}"
        ),
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: 'static, S: 'static> BootstrapBuilder<fret_launch::FnDriver<D, S>> {
    /// Create a bootstrap builder directly from `FnDriver` pieces.
    ///
    /// This is the recommended advanced escape hatch when the app wants the `fret-bootstrap`
    /// defaults and builder ergonomics, but does not want to manually construct
    /// `fret_launch::FnDriver` first.
    pub fn new_fn(
        app: App,
        driver_state: D,
        create_window_state: fn(&mut D, &mut App, fret_core::AppWindowId) -> S,
        handle_event: for<'d, 'cx, 'e> fn(
            &'d mut D,
            fret_launch::WinitEventContext<'cx, S>,
            &'e fret_core::Event,
        ),
        render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
    ) -> Self {
        Self::new(
            app,
            fret_launch::FnDriver::new(driver_state, create_window_state, handle_event, render),
        )
    }

    /// Same as [`new_fn`](Self::new_fn), but preserves access to `FnDriverHooks`.
    pub fn new_fn_with_hooks(
        app: App,
        driver_state: D,
        create_window_state: fn(&mut D, &mut App, fret_core::AppWindowId) -> S,
        handle_event: for<'d, 'cx, 'e> fn(
            &'d mut D,
            fret_launch::WinitEventContext<'cx, S>,
            &'e fret_core::Event,
        ),
        render: for<'d, 'cx> fn(&'d mut D, fret_launch::WinitRenderContext<'cx, S>),
        configure_hooks: impl FnOnce(&mut fret_launch::FnDriverHooks<D, S>),
    ) -> Self {
        Self::new(
            app,
            fret_launch::FnDriver::new(driver_state, create_window_state, handle_event, render)
                .with_hooks(configure_hooks),
        )
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod fn_driver_builder_tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use fret_app::App;
    use fret_assets::{AssetBundleId, AssetRevision, StaticAssetEntry};
    use fret_core::AppWindowId;
    use fret_icons::{
        IconId, IconPackImportModel, IconPackMetadata, IconPackRegistration, IconRegistry,
        InstalledIconPackMetadataConflict, ResolveError,
    };
    use fret_launch::{
        FnDriverHooks, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    };

    use super::{
        AssetStartupMode, AssetStartupPlan, BootstrapBuilder, BootstrapError,
        BootstrapKnownFailureKind, BootstrapKnownFailureReport, BootstrapKnownFailureStage,
    };

    struct DriverState;
    struct WindowState;

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn create_window_state(
        _driver: &mut DriverState,
        _app: &mut App,
        _window: AppWindowId,
    ) -> WindowState {
        WindowState
    }

    fn handle_event(
        _driver: &mut DriverState,
        _context: WinitEventContext<'_, WindowState>,
        _event: &fret_core::Event,
    ) {
    }

    fn render(_driver: &mut DriverState, _context: WinitRenderContext<'_, WindowState>) {}

    fn hot_reload_window(
        _driver: &mut DriverState,
        _context: WinitHotReloadContext<'_, WindowState>,
    ) {
    }

    fn make_temp_dir(tag: &str) -> PathBuf {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("fret-bootstrap-{tag}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_asset_dir_fixture(tag: &str) -> PathBuf {
        let dir = make_temp_dir(tag).join("assets");
        std::fs::create_dir_all(dir.join("images")).expect("create images dir");
        std::fs::write(dir.join("images/logo.png"), b"bootstrap-bytes").expect("write asset");
        dir
    }

    #[test]
    fn new_fn_with_hooks_builds() {
        let _builder = BootstrapBuilder::new_fn_with_hooks(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
            |hooks: &mut FnDriverHooks<DriverState, WindowState>| {
                hooks.hot_reload_window = Some(hot_reload_window);
            },
        )
        .configure(|_config| {});
    }

    #[test]
    fn bootstrap_builder_with_asset_startup_installs_selected_packaged_lane() {
        let _builder = BootstrapBuilder::new_fn(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
        )
        .with_asset_startup(
            AssetBundleId::app("bootstrap-asset-startup-packaged"),
            AssetStartupMode::Packaged,
            AssetStartupPlan::new()
                .development_manifest("assets.manifest.json")
                .packaged_entries([StaticAssetEntry::new(
                    "images/logo.png",
                    AssetRevision(1),
                    b"builder-bytes",
                )]),
        )
        .expect("packaged asset startup plan should load on bootstrap builder path");
    }

    #[test]
    fn bootstrap_builder_keeps_asset_startup_as_single_asset_mount_surface() {
        let lib_rs = include_str!("lib.rs");
        let bundle_entries_helper = ["pub fn ", "with_bundle_asset_entries("].concat();
        let embedded_entries_helper = ["pub fn ", "with_embedded_asset_entries("].concat();
        let startup_helper = ["pub fn ", "with_asset_startup("].concat();
        assert!(!lib_rs.contains(&bundle_entries_helper));
        assert!(!lib_rs.contains(&embedded_entries_helper));
        assert!(lib_rs.contains(&startup_helper));
    }

    #[test]
    fn bootstrap_builder_keeps_contract_aware_icon_pack_registration_explicit() {
        let lib_rs = include_str!("lib.rs");
        assert!(lib_rs.contains("pub fn register_icon_pack_contract("));
        assert!(lib_rs.contains("pub fn register_icon_pack("));
        assert!(lib_rs.contains("raw registry-only escape hatch"));
        assert!(lib_rs.contains("fret_icons_lucide::VENDOR_PACK"));
        assert!(lib_rs.contains("fret_icons_radix::VENDOR_PACK"));
    }

    #[test]
    fn register_icon_pack_contract_panics_when_registry_cannot_freeze() {
        fn register_invalid_pack(registry: &mut IconRegistry) {
            let _ = registry.alias(
                IconId::new_static("broken.a"),
                IconId::new_static("broken.b"),
            );
            let _ = registry.alias(
                IconId::new_static("broken.b"),
                IconId::new_static("broken.a"),
            );
        }

        let result = std::panic::catch_unwind(|| {
            let _builder = BootstrapBuilder::new_fn(
                App::new(),
                DriverState,
                create_window_state,
                handle_event,
                render,
            )
            .register_icon_pack_contract(IconPackRegistration::new(
                IconPackMetadata {
                    pack_id: "broken-pack",
                    vendor_namespace: "broken",
                    import_model: IconPackImportModel::Manual,
                },
                register_invalid_pack,
            ));
        });

        assert!(result.is_err(), "invalid icon pack should fail fast");
    }

    #[test]
    fn register_icon_pack_contract_panics_on_metadata_conflict() {
        fn register_first_pack(registry: &mut IconRegistry) {
            let _ = registry.register_svg_static(IconId::new_static("demo.first"), b"<svg/>");
        }

        fn register_second_pack(registry: &mut IconRegistry) {
            let _ = registry.register_svg_static(IconId::new_static("demo.second"), b"<svg/>");
        }

        let result = std::panic::catch_unwind(|| {
            let _builder = BootstrapBuilder::new_fn(
                App::new(),
                DriverState,
                create_window_state,
                handle_event,
                render,
            )
            .register_icon_pack_contract(IconPackRegistration::new(
                IconPackMetadata {
                    pack_id: "demo-pack",
                    vendor_namespace: "demo",
                    import_model: IconPackImportModel::Manual,
                },
                register_first_pack,
            ))
            .register_icon_pack_contract(IconPackRegistration::new(
                IconPackMetadata {
                    pack_id: "demo-pack",
                    vendor_namespace: "other-demo",
                    import_model: IconPackImportModel::Vendored,
                },
                register_second_pack,
            ));
        });

        assert!(
            result.is_err(),
            "conflicting pack metadata should fail fast"
        );
    }

    #[test]
    fn bootstrap_error_known_failure_report_maps_settings_read() {
        let error = BootstrapError::from(fret_app::SettingsError::Read {
            path: "/tmp/settings.json".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing settings"),
        });

        let report = error.known_failure_report();

        assert_eq!(report.stage, BootstrapKnownFailureStage::Builder);
        assert_eq!(report.kind, BootstrapKnownFailureKind::SettingsRead);
        assert_eq!(report.surface, Some("settings"));
        assert_eq!(report.pack_id, None);
        assert_eq!(
            report.summary,
            "failed to read settings file `/tmp/settings.json`"
        );
        assert_eq!(report.details.len(), 1);
        assert!(report.details[0].contains("missing settings"));
    }

    #[test]
    fn bootstrap_error_known_failure_report_maps_keymap_and_menu_bar_parse() {
        let keymap_parse_error =
            fret_runtime::Keymap::from_bytes(br#"{ "keymap_version": "#).unwrap_err();
        let keymap_error = BootstrapError::from(fret_app::KeymapFileError::Parse {
            path: "/tmp/keymap.json".to_string(),
            source: keymap_parse_error,
        });
        let keymap_report = keymap_error.known_failure_report();
        assert_eq!(keymap_report.stage, BootstrapKnownFailureStage::Builder);
        assert_eq!(keymap_report.kind, BootstrapKnownFailureKind::KeymapParse);
        assert_eq!(keymap_report.surface, Some("keymap"));
        assert_eq!(
            keymap_report.summary,
            "failed to parse keymap file `/tmp/keymap.json`"
        );
        assert_eq!(keymap_report.details.len(), 1);
        assert!(keymap_report.details[0].contains("failed to parse keymap json"));

        let menu_bar_parse_error =
            fret_runtime::MenuBarConfig::from_bytes(br#"{ "menu_bar_version": "#).unwrap_err();
        let menu_bar_error = BootstrapError::from(fret_app::MenuBarFileError::Parse {
            path: "/tmp/menubar.json".to_string(),
            source: menu_bar_parse_error,
        });
        let menu_bar_report = menu_bar_error.known_failure_report();
        assert_eq!(menu_bar_report.stage, BootstrapKnownFailureStage::Builder);
        assert_eq!(
            menu_bar_report.kind,
            BootstrapKnownFailureKind::MenuBarParse
        );
        assert_eq!(menu_bar_report.surface, Some("menu_bar"));
        assert_eq!(
            menu_bar_report.summary,
            "failed to parse menubar file `/tmp/menubar.json`"
        );
        assert_eq!(menu_bar_report.details.len(), 1);
        assert!(menu_bar_report.details[0].contains("failed to parse menubar json"));
    }

    #[test]
    fn bootstrap_known_failure_report_maps_icon_install_failure() {
        let registry_freeze_report = fret_icons::IconInstallFailureReport::registry_freeze(
            "demo.app.install",
            Some("demo-pack"),
            &[ResolveError::AliasLoop {
                requested: IconId::new_static("demo.alias"),
                chain: vec![
                    IconId::new_static("demo.alias"),
                    IconId::new_static("demo.target"),
                ],
            }],
        );
        let bootstrap_registry_freeze =
            BootstrapKnownFailureReport::from_icon_install_failure(&registry_freeze_report);
        assert_eq!(
            bootstrap_registry_freeze.stage,
            BootstrapKnownFailureStage::ExplicitInstall
        );
        assert_eq!(
            bootstrap_registry_freeze.kind,
            BootstrapKnownFailureKind::IconInstallRegistryFreezeFailed
        );
        assert_eq!(bootstrap_registry_freeze.surface, Some("demo.app.install"));
        assert_eq!(bootstrap_registry_freeze.pack_id, Some("demo-pack"));
        assert!(
            bootstrap_registry_freeze
                .summary
                .contains("failed to freeze icon registry")
        );
        assert_eq!(bootstrap_registry_freeze.details.len(), 1);

        let metadata_conflict = InstalledIconPackMetadataConflict {
            existing: IconPackMetadata {
                pack_id: "demo-pack",
                vendor_namespace: "demo",
                import_model: IconPackImportModel::Generated,
            },
            attempted: IconPackMetadata {
                pack_id: "demo-pack",
                vendor_namespace: "other-demo",
                import_model: IconPackImportModel::Vendored,
            },
        };
        let metadata_conflict_report = fret_icons::IconInstallFailureReport::metadata_conflict(
            "demo.app.install",
            &metadata_conflict,
        );
        let bootstrap_metadata_conflict =
            BootstrapKnownFailureReport::from_icon_install_failure(&metadata_conflict_report);
        assert_eq!(
            bootstrap_metadata_conflict.stage,
            BootstrapKnownFailureStage::ExplicitInstall
        );
        assert_eq!(
            bootstrap_metadata_conflict.kind,
            BootstrapKnownFailureKind::IconInstallMetadataConflict
        );
        assert_eq!(bootstrap_metadata_conflict.pack_id, Some("demo-pack"));
        assert!(
            bootstrap_metadata_conflict
                .summary
                .contains("conflicting installed icon pack metadata")
        );
        assert_eq!(bootstrap_metadata_conflict.details.len(), 2);
    }

    #[test]
    fn bootstrap_builder_asset_startup_fails_early_for_missing_manifest_files() {
        let missing =
            std::env::temp_dir().join("definitely-missing-fret-bootstrap-assets.manifest.json");
        let err = match BootstrapBuilder::new_fn(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
        )
        .with_asset_startup(
            AssetBundleId::app("bootstrap-missing-asset-startup-manifest"),
            AssetStartupMode::Development,
            AssetStartupPlan::new().development_manifest(&missing),
        ) {
            Ok(_) => panic!("missing manifest should fail on bootstrap asset startup path"),
            Err(err) => err,
        };
        assert!(matches!(err, BootstrapError::AssetManifest(_)));
    }

    #[test]
    fn bootstrap_builder_asset_startup_fails_early_for_missing_directories() {
        let missing = std::env::temp_dir().join("definitely-missing-fret-bootstrap-assets-dir");
        let err = match BootstrapBuilder::new_fn(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
        )
        .with_asset_startup(
            AssetBundleId::app("bootstrap-missing-asset-startup-dir"),
            AssetStartupMode::Development,
            AssetStartupPlan::new().development_dir(&missing),
        ) {
            Ok(_) => panic!("missing asset dir should fail on bootstrap asset startup path"),
            Err(err) => err,
        };
        assert!(matches!(err, BootstrapError::AssetManifest(_)));
    }

    #[test]
    fn bootstrap_builder_asset_startup_fails_when_selected_lane_is_missing() {
        let err = match BootstrapBuilder::new_fn(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
        )
        .with_asset_startup(
            AssetBundleId::app("bootstrap-missing-asset-startup-packaged"),
            AssetStartupMode::Packaged,
            AssetStartupPlan::new().development_dir("assets"),
        ) {
            Ok(_) => panic!("missing packaged lane should fail on bootstrap builder path"),
            Err(err) => err,
        };
        assert!(matches!(
            err,
            BootstrapError::AssetStartup(super::AssetStartupPlanError::MissingPackagedLane)
        ));
    }

    #[test]
    fn asset_startup_mode_preferred_matches_current_target_defaults() {
        #[cfg(debug_assertions)]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Development);

        #[cfg(not(debug_assertions))]
        assert_eq!(AssetStartupMode::preferred(), AssetStartupMode::Packaged);
    }

    #[test]
    fn bootstrap_builder_asset_startup_accepts_development_bundle_dir_if_native() {
        let asset_dir =
            write_asset_dir_fixture("asset-startup-plan-development-bundle-dir-if-native");
        let app_bundle = AssetBundleId::app("asset-startup-plan-development-bundle-dir-if-native");
        let plan = AssetStartupPlan::new()
            .packaged_entries([StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(1),
                b"builder-bytes",
            )])
            .development_bundle_dir_if_native(app_bundle.clone(), &asset_dir);

        let _builder = BootstrapBuilder::new_fn(
            App::new(),
            DriverState,
            create_window_state,
            handle_event,
            render,
        )
        .with_asset_startup(app_bundle, AssetStartupMode::Development, plan)
        .expect("native helper should populate the development lane on bootstrap builder path");
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<D> From<fret_launch::WinitAppBuilder<D>> for BootstrapBuilder<D> {
    fn from(inner: fret_launch::WinitAppBuilder<D>) -> Self {
        Self {
            inner,
            on_gpu_ready_hooks: Vec::new(),
        }
    }
}

#[cfg(feature = "ui-app-driver")]
pub mod ui_app_driver;

#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
mod dev_reload;

#[cfg(feature = "ui-app-driver")]
pub mod hot_literals;

#[cfg(feature = "ui-app-driver")]
pub use hot_literals::{HotLiterals, HotLiteralsFile};

#[cfg(feature = "window-style-profiles")]
pub mod window_style_profiles {
    pub use fret_window_style_profiles::*;
}

#[cfg(all(feature = "ui-app-driver", feature = "diagnostics"))]
pub mod ui_diagnostics;

#[cfg(all(
    feature = "ui-app-driver",
    feature = "diagnostics",
    feature = "diagnostics-ws"
))]
mod ui_diagnostics_ws_bridge;

/// Returns `true` when bootstrap diagnostics consumed the event (ignore/intercept).
///
/// App drivers should prefer this helper over reaching into `ui_diagnostics` directly so builds
/// remain resilient when diagnostics modules are feature-gated.
pub fn maybe_consume_event(
    app: &mut fret_app::App,
    window: fret_core::AppWindowId,
    event: &fret_core::Event,
) -> bool {
    #[cfg(all(feature = "ui-app-driver", feature = "diagnostics"))]
    {
        crate::ui_diagnostics::maybe_consume_event(app, window, event)
    }

    #[cfg(not(all(feature = "ui-app-driver", feature = "diagnostics")))]
    {
        let _ = (app, window, event);
        false
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "diagnostics"))]
pub fn init_diagnostics() {
    init_tracing();
    init_panic_hook();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "tracing"))]
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    const DEFAULT: &str = "info,fret=info,fret_launch=info,fret_render=info";

    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| EnvFilter::try_new(v).ok())
        .unwrap_or_else(|| EnvFilter::try_new(DEFAULT).expect("default tracing filter is valid"));

    #[cfg(feature = "tracy")]
    {
        let tracy_enabled = std::env::var_os("FRET_TRACY").is_some_and(|v| !v.is_empty());
        if tracy_enabled {
            use tracing_subscriber::fmt::format::DefaultFields;

            #[derive(Default)]
            struct FretTracyConfig {
                fmt: DefaultFields,
                callstack_depth: u16,
            }

            impl tracing_tracy::Config for FretTracyConfig {
                type Formatter = DefaultFields;

                fn formatter(&self) -> &Self::Formatter {
                    &self.fmt
                }

                fn stack_depth(&self, metadata: &tracing::Metadata<'_>) -> u16 {
                    if self.callstack_depth == 0 {
                        return 0;
                    }

                    match metadata.name() {
                        "fret.ui.layout"
                        | "fret.ui.paint"
                        | "fret_ui.layout_all"
                        | "fret_ui.paint_all"
                        | "fret.ui.layout_engine.solve"
                        | "fret.ui.paint_cache.replay"
                        | "fret.runner.redraw"
                        | "fret.runner.prepare"
                        | "fret.runner.render"
                        | "fret.runner.record"
                        | "fret.runner.present"
                        | "fret.runner.render_scene"
                        | "ui.cache_root.mount"
                        | "ui.cache_root.reuse"
                        | "ui.cache_root.layout"
                        | "ui.cache_root.paint" => self.callstack_depth,
                        _ => 0,
                    }
                }
            }

            let callstack_depth = std::env::var("FRET_TRACY_CALLSTACK_DEPTH")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(16);
            let callstack_enabled =
                std::env::var_os("FRET_TRACY_CALLSTACK").is_some_and(|v| !v.is_empty());

            let config = FretTracyConfig {
                fmt: DefaultFields::default(),
                callstack_depth: if callstack_enabled {
                    callstack_depth
                } else {
                    0
                },
            };

            let _ = tracing_subscriber::registry()
                .with(filter.clone())
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .compact(),
                )
                .with(tracing_tracy::TracyLayer::new(config))
                .try_init();
            return;
        }
    }

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .compact(),
        )
        .try_init();
}

#[cfg(all(not(target_arch = "wasm32"), feature = "diagnostics"))]
pub fn init_panic_hook() {
    use std::backtrace::Backtrace;
    use std::sync::Once;

    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let default_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |info| {
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            let message = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string());

            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string());

            let bootstrap_known_failure =
                fret_icons::current_icon_install_failure_report_for_diagnostics()
                    .map(|report| BootstrapKnownFailureReport::from_icon_install_failure(&report));
            let backtrace = Backtrace::capture();
            match backtrace.status() {
                std::backtrace::BacktraceStatus::Captured => {
                    if let Some(report) = bootstrap_known_failure.as_ref() {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            known_panic_kind = "bootstrap_known_failure",
                            bootstrap_failure_stage = report.stage.as_str(),
                            bootstrap_failure_kind = report.kind.as_str(),
                            bootstrap_failure_surface = report.surface.unwrap_or("<none>"),
                            bootstrap_failure_pack_id = report.pack_id.unwrap_or("<none>"),
                            bootstrap_failure_summary = %report.summary,
                            bootstrap_failure_details = ?report.details,
                            backtrace = %backtrace,
                            "panic"
                        );
                    } else {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            backtrace = %backtrace,
                            "panic"
                        );
                    }
                }
                std::backtrace::BacktraceStatus::Disabled
                | std::backtrace::BacktraceStatus::Unsupported => {
                    if let Some(report) = bootstrap_known_failure.as_ref() {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            known_panic_kind = "bootstrap_known_failure",
                            bootstrap_failure_stage = report.stage.as_str(),
                            bootstrap_failure_kind = report.kind.as_str(),
                            bootstrap_failure_surface = report.surface.unwrap_or("<none>"),
                            bootstrap_failure_pack_id = report.pack_id.unwrap_or("<none>"),
                            bootstrap_failure_summary = %report.summary,
                            bootstrap_failure_details = ?report.details,
                            "panic (set RUST_BACKTRACE=1 to capture a backtrace)"
                        );
                    } else {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            "panic (set RUST_BACKTRACE=1 to capture a backtrace)"
                        );
                    }
                }
                _ => {
                    if let Some(report) = bootstrap_known_failure.as_ref() {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            known_panic_kind = "bootstrap_known_failure",
                            bootstrap_failure_stage = report.stage.as_str(),
                            bootstrap_failure_kind = report.kind.as_str(),
                            bootstrap_failure_surface = report.surface.unwrap_or("<none>"),
                            bootstrap_failure_pack_id = report.pack_id.unwrap_or("<none>"),
                            bootstrap_failure_summary = %report.summary,
                            bootstrap_failure_details = ?report.details,
                            "panic"
                        );
                    } else {
                        tracing::error!(
                            thread = thread_name,
                            location = location,
                            message = message,
                            "panic"
                        );
                    }
                }
            }

            default_hook(info);
        }));
    });
}

/// Concrete `BootstrapBuilder` type returned by `ui_app` / `ui_app_with_app`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub type UiAppBootstrapBuilder<S> = BootstrapBuilder<
    fret_launch::FnDriver<ui_app_driver::UiAppDriver<S>, ui_app_driver::UiAppWindowState<S>>,
>;

/// Create a “golden path” native UI app builder, using `App::new()` by default and allowing a
/// hook to configure the driver before it is wrapped into `FnDriver`.
///
/// This is the recommended author-facing path for general applications that want the bootstrap
/// defaults without dealing with runner-level driver details.
///
/// Prefer passing a non-capturing closure so it can coerce to a `fn` pointer (hotpatch-friendly).
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_hooks<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
    configure: fn(ui_app_driver::UiAppDriver<S>) -> ui_app_driver::UiAppDriver<S>,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app_and_hooks(App::new(), root_name, init_window, view, configure)
}

/// Create a “golden path” native UI app builder, using `App::new()` by default.
///
/// This is the shortest recommended entry for general applications. It hides the `FnDriver`
/// boilerplate and keeps example code short.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app<S: 'static>(
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app(App::new(), root_name, init_window, view)
}

/// Same as `ui_app`, but allows providing a pre-configured `App`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_app<S: 'static>(
    app: App,
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
) -> UiAppBootstrapBuilder<S> {
    ui_app_with_app_and_hooks(app, root_name, init_window, view, |d| d)
}

/// Same as `ui_app_with_app`, but allows a hook to configure the driver before it is wrapped into
/// `FnDriver`.
#[cfg(all(not(target_arch = "wasm32"), feature = "ui-app-driver"))]
pub fn ui_app_with_app_and_hooks<S: 'static>(
    app: App,
    root_name: &'static str,
    init_window: fn(&mut App, fret_core::AppWindowId) -> S,
    view: for<'a> fn(&mut fret_ui::ElementContext<'a, App>, &mut S) -> ui_app_driver::ViewElements,
    configure: fn(ui_app_driver::UiAppDriver<S>) -> ui_app_driver::UiAppDriver<S>,
) -> UiAppBootstrapBuilder<S> {
    let driver = configure(ui_app_driver::UiAppDriver::new(
        root_name,
        init_window,
        view,
    ))
    .into_fn_driver();
    BootstrapBuilder::new(app, driver)
}

#[cfg(test)]
mod text_interaction_defaults_tests {
    use super::*;

    #[test]
    fn install_default_text_interaction_settings_enables_caret_blink_outside_diagnostics() {
        let mut app = App::new();

        install_default_text_interaction_settings_with(&mut app, false);

        let settings = app
            .global::<TextInteractionSettings>()
            .copied()
            .expect("bootstrap should install text interaction settings");
        assert!(settings.caret_blink);
        assert_eq!(settings.caret_blink_interval_ms, 500);
    }

    #[test]
    fn install_default_text_interaction_settings_disables_caret_blink_for_diagnostics() {
        let mut app = App::new();

        install_default_text_interaction_settings_with(&mut app, true);

        let settings = app
            .global::<TextInteractionSettings>()
            .copied()
            .expect("bootstrap should install text interaction settings");
        assert!(!settings.caret_blink);
    }

    #[test]
    fn install_default_text_interaction_settings_preserves_explicit_override() {
        let mut app = App::new();
        let explicit = TextInteractionSettings {
            linux_primary_selection: true,
            caret_blink: false,
            caret_blink_interval_ms: 777,
            horizontal_autoscroll_margin_px: 21,
            horizontal_autoscroll_max_step_px: 42,
        };
        app.set_global(explicit);

        install_default_text_interaction_settings_with(&mut app, false);

        let settings = app
            .global::<TextInteractionSettings>()
            .copied()
            .expect("explicit settings should still be present");
        assert_eq!(settings, explicit);
    }
}
