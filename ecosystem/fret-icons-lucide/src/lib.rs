//! A vendored Lucide SVG icon pack for Fret demos/components.
//!
//! This crate registers a curated subset of Lucide SVG icons into [`fret_icons::IconRegistry`].
//! Most users will use the higher-level install hooks exposed by the ecosystem `fret` crate.

use fret_icons::{
    IconId, IconPackImportModel, IconPackMetadata, IconPackRegistration, IconRegistry, ids,
};
use rust_embed::RustEmbed;
use std::{borrow::Cow, sync::Arc};

pub mod generated_ids;

pub const PACK_METADATA: IconPackMetadata = IconPackMetadata {
    pack_id: "fret-icons-lucide",
    vendor_namespace: "lucide",
    import_model: IconPackImportModel::Vendored,
};

pub const VENDOR_PACK: IconPackRegistration =
    IconPackRegistration::new(PACK_METADATA, register_vendor_icons);

#[cfg(feature = "semantic-ui")]
pub const UI_SEMANTIC_ALIAS_PACK: IconPackRegistration =
    IconPackRegistration::new(PACK_METADATA, register_ui_semantic_aliases);

pub const PACK: IconPackRegistration = IconPackRegistration::new(PACK_METADATA, register_icons);

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
struct Assets;

pub fn register_icons(reg: &mut IconRegistry) {
    register_vendor_icons(reg);

    #[cfg(feature = "semantic-ui")]
    register_ui_semantic_aliases(reg);
}

/// Register Lucide vendor icon IDs (`lucide.*`) into an [`IconRegistry`].
pub fn register_vendor_icons(reg: &mut IconRegistry) {
    register_curated(reg);

    // Legacy Lucide IDs (lucide-react exports `MoreHorizontal`, older packs ship `more-horizontal.svg`).
    // Newer Lucide versions standardize on `ellipsis` / `ellipsis-vertical`.
    let _ = reg.alias_if_missing(
        IconId::new("lucide.more-horizontal"),
        IconId::new("lucide.ellipsis"),
    );
    let _ = reg.alias_if_missing(
        IconId::new("lucide.more-vertical"),
        IconId::new("lucide.ellipsis-vertical"),
    );
}

/// Register semantic `ui.*` aliases for this icon pack.
#[cfg(feature = "semantic-ui")]
pub fn register_ui_semantic_aliases(reg: &mut IconRegistry) {
    semantic_ui::register(reg);
}

#[cfg(feature = "app-integration")]
pub mod advanced;
#[cfg(feature = "app-integration")]
pub mod app;

fn register_curated(reg: &mut IconRegistry) {
    for line in include_str!("../icon-list.txt").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let icon_name = line.strip_suffix(".svg").unwrap_or(line);
        register_vendor_icon(reg, icon_name);
    }
}

fn register_vendor_icon(reg: &mut IconRegistry, icon_name: &str) {
    let path = format!("icons/{icon_name}.svg");
    let Some(file) = Assets::get(&path) else {
        return;
    };

    let bytes: Arc<[u8]> = match file.data {
        Cow::Borrowed(b) => Arc::from(b),
        Cow::Owned(v) => Arc::from(v),
    };

    let _ = reg.register_svg_bytes(IconId::new(format!("lucide.{icon_name}")), bytes);
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    pub fn register(reg: &mut IconRegistry) {
        let _ = reg.alias_if_missing(
            ids::ui::ALERT_TRIANGLE,
            IconId::new("lucide.triangle-alert"),
        );
        let _ = reg.alias_if_missing(ids::ui::ARROW_LEFT, IconId::new("lucide.arrow-left"));
        let _ = reg.alias_if_missing(ids::ui::ARROW_RIGHT, IconId::new("lucide.arrow-right"));
        let _ = reg.alias_if_missing(ids::ui::BOOK, IconId::new("lucide.book"));
        let _ = reg.alias_if_missing(ids::ui::CHECK, IconId::new("lucide.check"));
        let _ = reg.alias_if_missing(ids::ui::COPY, IconId::new("lucide.copy"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_LEFT, IconId::new("lucide.chevron-left"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_DOWN, IconId::new("lucide.chevron-down"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_RIGHT, IconId::new("lucide.chevron-right"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_UP, IconId::new("lucide.chevron-up"));
        let _ = reg.alias_if_missing(
            ids::ui::CHEVRONS_UP_DOWN,
            IconId::new("lucide.chevrons-up-down"),
        );
        let _ = reg.alias_if_missing(ids::ui::CLOSE, IconId::new("lucide.x"));
        let _ = reg.alias_if_missing(ids::ui::FILE, IconId::new("lucide.file"));
        let _ = reg.alias_if_missing(ids::ui::EYE, IconId::new("lucide.eye"));
        let _ = reg.alias_if_missing(ids::ui::EYE_OFF, IconId::new("lucide.eye-off"));
        // Lucide v0.4x ships `git-commit-horizontal` / `git-commit-vertical` (no `git-commit`).
        let _ = reg.alias_if_missing(
            ids::ui::GIT_COMMIT,
            IconId::new("lucide.git-commit-vertical"),
        );
        let _ = reg.alias_if_missing(ids::ui::FOLDER, IconId::new("lucide.folder"));
        let _ = reg.alias_if_missing(ids::ui::FOLDER_OPEN, IconId::new("lucide.folder-open"));
        let _ = reg.alias_if_missing(ids::ui::LOADER, IconId::new("lucide.loader-circle"));
        let _ = reg.alias_if_missing(ids::ui::MORE_HORIZONTAL, IconId::new("lucide.ellipsis"));
        let _ = reg.alias_if_missing(ids::ui::MINUS, IconId::new("lucide.minus"));
        let _ = reg.alias_if_missing(ids::ui::PLUS, IconId::new("lucide.plus"));
        let _ = reg.alias_if_missing(ids::ui::SEARCH, IconId::new("lucide.search"));
        let _ = reg.alias_if_missing(ids::ui::RESET, IconId::new("lucide.rotate-ccw"));
        let _ = reg.alias_if_missing(ids::ui::SETTINGS, IconId::new("lucide.settings"));
        let _ = reg.alias_if_missing(ids::ui::PLAY, IconId::new("lucide.play"));
        let _ = reg.alias_if_missing(ids::ui::SLASH, IconId::new("lucide.slash"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_FAILED, IconId::new("lucide.circle-x"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_PENDING, IconId::new("lucide.circle"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_RUNNING, IconId::new("lucide.circle-dot"));
        let _ = reg.alias_if_missing(
            ids::ui::STATUS_SUCCEEDED,
            IconId::new("lucide.circle-check"),
        );
        let _ = reg.alias_if_missing(ids::ui::TOOL, IconId::new("lucide.wrench"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const LIB_RS: &str = include_str!("lib.rs");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");
    const README: &str = include_str!("../README.md");

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn lucide_more_horizontal_alias_resolves() {
        let mut reg = IconRegistry::default();
        register_vendor_icons(&mut reg);

        reg.resolve(&IconId::new("lucide.ellipsis"))
            .expect("expected lucide.ellipsis to resolve");
        reg.resolve(&IconId::new("lucide.more-horizontal"))
            .expect("expected lucide.more-horizontal legacy alias to resolve");
    }

    #[test]
    fn pack_metadata_stays_explicit_and_stable() {
        assert_eq!(PACK_METADATA.pack_id, "fret-icons-lucide");
        assert_eq!(PACK_METADATA.vendor_namespace, "lucide");
        assert_eq!(PACK_METADATA.import_model, IconPackImportModel::Vendored);
    }

    #[test]
    fn app_integration_stays_under_explicit_app_module() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(public_surface.contains("pub const PACK_METADATA: IconPackMetadata"));
        assert!(public_surface.contains("pub const VENDOR_PACK: IconPackRegistration"));
        assert!(public_surface.contains("pub const PACK: IconPackRegistration"));
        assert!(!public_surface.contains("pub use app::"));
        assert!(!public_surface.contains("pub use advanced::"));
        assert!(!public_surface.contains("pub fn install("));
        assert!(!public_surface.contains("pub fn install_with_ui_services("));
        assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(APP_RS.contains("let frozen = icons.freeze().unwrap_or_else(|errors|"));
        assert!(APP_RS.contains("panic_on_icon_registry_freeze_failure("));
        assert!(APP_RS.contains("panic_on_icon_pack_metadata_conflict("));
        assert!(!APP_RS.contains("install_with_ui_services"));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
    }

    #[test]
    fn readme_keeps_app_installation_and_alias_policy_explicit() {
        assert!(README.contains("`register_vendor_icons(...)` / `register_icons(...)`"));
        assert!(README.contains("`PACK_METADATA` / `PACK`"));
        assert!(README.contains("`fret_icons_lucide::app::install(...)`"));
        assert!(README.contains("`fret_icons_lucide::advanced::install_with_ui_services(...)`"));
        assert!(README.contains("semantic `IconId` / `ui.*` ids"));
        assert!(README.contains("first-writer-wins (`alias_if_missing(...)`)"));
        assert!(README.contains("without mutating `lucide.*` vendor ids"));
        assert!(README.contains("one installer/bundle surface"));
        assert!(README.contains("`app-integration`"));
    }

    #[cfg(feature = "app-integration")]
    #[test]
    fn app_install_records_pack_metadata_and_freezes_registry() {
        let mut app = fret_app::App::new();
        crate::app::install(&mut app);

        let installed = app
            .global::<fret_icons::InstalledIconPacks>()
            .expect("expected installed icon pack metadata to be recorded");
        assert!(installed.contains(PACK_METADATA.pack_id));

        let frozen = app
            .global::<fret_icons::FrozenIconRegistry>()
            .expect("expected app install to refresh the frozen icon registry");
        assert!(frozen.resolve(&IconId::new("lucide.search")).is_some());
    }
}
