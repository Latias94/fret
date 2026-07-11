const APP_ENTRY_RS: &str = include_str!("app_entry.rs");
const ACTIONS_RS: &str = include_str!("actions.rs");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const INTEROP_RS: &str = include_str!("interop.rs");
const ROOT_README: &str = include_str!("../../../README.md");
const DOCS_README: &str = include_str!("../../../docs/README.md");
const FIRST_HOUR: &str = include_str!("../../../docs/first-hour.md");
const TODO_APP_GOLDEN_PATH: &str = include_str!("../../../docs/examples/todo-app-golden-path.md");
const AUTHORING_GOLDEN_PATH: &str = include_str!("../../../docs/authoring-golden-path.md");
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
const SHADCN_SELECT_V4_USAGE: &str =
    include_str!("../../../docs/workstreams/shadcn-part-surface-alignment-v1/SELECT_V4_USAGE.md");
const SHADCN_COMBOBOX_V4_USAGE: &str =
    include_str!("../../../docs/workstreams/shadcn-part-surface-alignment-v1/COMBOBOX_V4_USAGE.md");
const APP_ENTRY_BUILDER_DESIGN: &str =
    include_str!("../../../docs/workstreams/app-entry-builder-v1/DESIGN.md");
const APP_ENTRY_BUILDER_TODO: &str =
    include_str!("../../../docs/workstreams/app-entry-builder-v1/TODO.md");
const AUTHORING_SURFACE_MIGRATION_MATRIX: &str = include_str!(
    "../../../docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MIGRATION_MATRIX.md"
);
const ROOT_RS: &str = include_str!("lib.rs");
const APP_RS: &str = include_str!("app.rs");
const APP_PRELUDE_RS: &str = include_str!("app/prelude.rs");
const COMPONENT_RS: &str = include_str!("component.rs");
const COMPONENT_PRELUDE_RS: &str = include_str!("component/prelude.rs");
const ADVANCED_RS: &str = include_str!("advanced.rs");
const ADVANCED_PRELUDE_RS: &str = include_str!("advanced/prelude.rs");
const ADVANCED_RAW_RS: &str = include_str!("advanced/raw.rs");
const ADVANCED_DRIVER_RS: &str = include_str!("advanced/driver.rs");
const BUILDER_RS: &str = include_str!("builder.rs");
const WORKSPACE_RS: &str = include_str!("workspace.rs");
const VIEW_RS: &str = include_str!("view.rs");
const VIEW_CONTEXT_RS: &str = include_str!("view/context.rs");

fn crate_rustdoc() -> String {
    ROOT_RS
        .lines()
        .filter(|line| line.starts_with("//!"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn module_block_source(source: &'static str, marker: &str) -> &'static str {
    let start = source.find(marker).expect("module marker should exist");
    let open = source[start..]
        .find('{')
        .map(|idx| start + idx)
        .expect("module block should have an opening brace");
    let mut depth = 0usize;
    for (idx, byte) in source[open..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[start..open + idx + 1];
                }
            }
            _ => {}
        }
    }
    panic!("module block should have a closing brace");
}

fn app_prelude_source() -> &'static str {
    APP_PRELUDE_RS
}

fn ui_app_builder_impl_source() -> &'static str {
    BUILDER_RS
}

fn crate_public_surface_source() -> &'static str {
    ROOT_RS
}

fn root_surface_header_source() -> &'static str {
    ROOT_RS
}

fn component_prelude_source() -> &'static str {
    COMPONENT_PRELUDE_RS
}

fn selector_surface_source() -> &'static str {
    module_block_source(ROOT_RS, "pub mod selector {")
}

fn query_surface_source() -> &'static str {
    module_block_source(ROOT_RS, "pub mod query {")
}

fn advanced_prelude_source() -> &'static str {
    ADVANCED_RS
}

fn advanced_common_prelude_source() -> &'static str {
    ADVANCED_PRELUDE_RS
}

fn root_contains(needle: &str) -> bool {
    ROOT_RS.contains(needle)
}

fn app_contains(needle: &str) -> bool {
    APP_RS.contains(needle)
}

fn component_contains(needle: &str) -> bool {
    COMPONENT_RS.contains(needle)
}

fn advanced_raw_contains(needle: &str) -> bool {
    ADVANCED_RAW_RS.contains(needle)
}

fn advanced_driver_contains(needle: &str) -> bool {
    ADVANCED_DRIVER_RS.contains(needle)
}

fn app_prelude_exports_symbol(symbol: &str) -> bool {
    app_prelude_source()
        .split(';')
        .filter(|statement| statement.contains("pub use "))
        .any(|statement| statement_exports_symbol(statement, symbol))
}

fn advanced_common_prelude_exports_symbol(symbol: &str) -> bool {
    advanced_common_prelude_source()
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

fn root_style_exported_symbols() -> std::collections::BTreeSet<String> {
    exported_symbol_names(module_block_source(ROOT_RS, "pub mod style {"))
}

fn expected_root_style_exported_symbols() -> std::collections::BTreeSet<String> {
    [
        "AttributedText",
        "Axis",
        "ChromeRefinement",
        "Color",
        "ColorRef",
        "ContainerProps",
        "Corners",
        "CrossAlign",
        "DashPatternV1",
        "DecorationLineStyle",
        "Edges",
        "FlexProps",
        "FontWeight",
        "LayoutRefinement",
        "LayoutStyle",
        "Length",
        "MainAlign",
        "MetricRef",
        "Overflow",
        "Px",
        "Radius",
        "ShadowPreset",
        "Size",
        "SizeStyle",
        "Space",
        "SpacingLength",
        "StrikethroughStyle",
        "TextAlign",
        "TextOverflow",
        "TextPaintStyle",
        "TextSpan",
        "TextWrap",
        "Theme",
        "ThemeSnapshot",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn pub_use_lines(source: &str) -> std::collections::BTreeSet<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("pub use "))
        .map(str::to_owned)
        .collect()
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
    assert!(CRATE_README.contains("`local.layout_value(cx)` / `local.paint_value(cx)`"));
    assert!(CRATE_README.contains(
        "`local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)`"
    ));
    assert!(CRATE_README.contains("`fret::style::{...}`"));
    assert!(CRATE_README.contains("`fret::icons::{icon, IconId}`"));
    assert!(CRATE_README.contains("`fret::semantics::SemanticsRole`"));
    assert!(CRATE_README.contains("`fret::env::{...}`"));
    assert!(CRATE_README.contains("`fret::assets::{...}`"));
    assert!(CRATE_README.contains("`AssetBundleId::app(...)`"));
    assert!(CRATE_README.contains("`AssetBundleId::package(...)`"));
    assert!(CRATE_README.contains("`AssetLocator::bundle(...)`"));
    assert!(CRATE_README.contains("`FretApp::asset_startup(...)`"));
    assert!(CRATE_README.contains("`UiAppBuilder::with_asset_startup(...)`"));
    assert!(CRATE_README.contains("`FileAssetManifestResolver::from_bundle_dir(...)`"));
    assert!(CRATE_README.contains("`FileAssetManifestResolver::from_manifest_path(...)`"));
    assert!(CRATE_README.contains("`register_resolver(...)`"));
    assert!(!CRATE_README.contains("`FretApp::asset_dir(...)`"));
    assert!(!CRATE_README.contains("`UiAppBuilder::with_asset_dir(...)`"));
    assert!(!CRATE_README.contains("`FretApp::asset_manifest(...)`"));
    assert!(!CRATE_README.contains("`UiAppBuilder::with_asset_manifest(...)`"));
    assert!(!CRATE_README.contains("`fret::assets::register_file_bundle_dir(...)`"));
    assert!(!CRATE_README.contains("`fret::assets::register_file_manifest(...)`"));
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
fn readme_and_rustdoc_quarantine_retained_driver_under_advanced_interop() {
    let public_surface = crate_public_surface_source();
    let advanced_surface = advanced_prelude_source();
    let rustdoc = crate_rustdoc();

    assert!(CRATE_README.contains("`fret::advanced::interop::run_native_with_driver(...)`"));
    assert!(rustdoc.contains("`fret::advanced::interop::run_native_with_driver(...)`"));
    assert!(!public_surface.contains("pub fn run_native_with_driver("));
    assert!(!public_surface.contains("pub mod interop;"));
    assert!(advanced_surface.contains("pub mod interop {"));
    assert!(advanced_surface.contains("pub use crate::interop::run_native_with_driver;"));
    assert!(INTEROP_RS.contains("pub fn run_native_with_driver<"));
}

#[test]
fn readme_and_rustdoc_quarantine_fn_driver_helpers_under_advanced() {
    let public_surface = crate_public_surface_source();
    let rustdoc = crate_rustdoc();

    assert!(CRATE_README.contains("`fret::advanced::run_native_with_fn_driver(...)`"));
    assert!(CRATE_README.contains("`fret::advanced::run_native_with_fn_driver_with_hooks(...)`"));
    assert!(CRATE_README.contains("`fret::advanced::run_native_with_configured_fn_driver(...)`"));
    assert!(rustdoc.contains("`fret::advanced::run_native_with_fn_driver(...)`"));
    assert!(rustdoc.contains("`fret::advanced::run_native_with_fn_driver_with_hooks(...)`"));
    assert!(rustdoc.contains("`fret::advanced::run_native_with_configured_fn_driver(...)`"));
    assert!(!public_surface.contains("pub fn run_native_with_fn_driver("));
    assert!(!public_surface.contains("pub fn run_native_with_fn_driver_with_hooks("));
    assert!(!public_surface.contains("pub fn run_native_with_configured_fn_driver("));
    assert!(advanced_driver_contains(
        "pub fn run_native_with_fn_driver<D: 'static, S: 'static>("
    ));
    assert!(advanced_driver_contains(
        "pub fn run_native_with_fn_driver_with_hooks<D: 'static, S: 'static>("
    ));
    assert!(advanced_driver_contains(
        "pub fn run_native_with_configured_fn_driver<D: 'static, S: 'static>("
    ));
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
    assert!(CRATE_README.contains(
        "`fret::router::{app::install, bind_history_actions, RouterUiStore, router_outlet_by_leaf_with_test_id, ...}`"
    ));

    let rustdoc = crate_rustdoc();
    let public_surface = crate_public_surface_source();
    assert!(rustdoc.contains(
        "`fret::router::{app::install, RouterUiStore, router_link, router_outlet_by_leaf_with_test_id, ...}`"
    ));
    assert!(rustdoc.contains("`fret::router::bind_history_actions(...)`"));
    assert!(public_surface.contains("pub mod router {"));
    assert!(public_surface.contains("pub fn router_link_to_typed_route_with_test_id<'a, Cx"));
    assert!(public_surface.contains("pub fn router_link_with_test_id<'a, Cx"));
    assert!(public_surface.contains("pub fn router_outlet_by_leaf_with_test_id<'a, Cx"));
    assert!(public_surface.contains("Cx: crate::app::AppRenderContext<'a>"));
    assert!(public_surface.contains("fret_router_ui::router_link_to_typed_route_with_test_id("));
    assert!(public_surface.contains("fret_router_ui::router_link_with_test_id("));
    assert!(public_surface.contains("fret_router_ui::RouterOutlet::new(snapshot.clone())"));
    assert!(public_surface.contains("pub fn bind_history_actions<Back, Forward, R, H>("));
    assert!(public_surface.contains("pub mod app {"));
    assert!(public_surface.contains("pub fn install(app: &mut crate::app::App) {"));
    assert!(!public_surface.contains("register_router_commands"));
    assert!(!public_surface.contains("pub fn install_app(app: &mut crate::app::App) {"));
    assert!(!public_surface.contains("pub use fret_router_ui::*;"));
}

#[test]
fn readme_and_rustdoc_expose_imui_as_explicit_optional_surface() {
    assert!(CRATE_README.contains("features = [\"imui\"]"));
    assert!(CRATE_README.contains("## Immediate-mode lane (optional)"));
    assert!(CRATE_README.contains("`fret::imui` is the explicit imgui-style lane."));
    assert!(CRATE_README.contains("use fret::imui::{UiWriter as _, imui_in};"));
    assert!(CRATE_README.contains("`use fret::imui::prelude::*;`"));
    assert!(CRATE_README.contains("if ui.button(\"Save\").clicked() {"));
    assert!(CRATE_README.contains("`fret::imui::kit`"));
    assert!(CRATE_README.contains("`fret::imui::editor`"));
    assert!(CRATE_README.contains("`fret::imui::docking`"));
    assert!(CRATE_README.contains("- `imui`: enable the explicit immediate-mode authoring lane"));

    let rustdoc = crate_rustdoc();
    let public_surface = crate_public_surface_source();
    assert!(rustdoc.contains("## Immediate-mode lane (optional)"));
    assert!(rustdoc.contains("`fret::imui` is the explicit imgui-style lane."));
    assert!(rustdoc.contains("use fret::imui::{UiWriter as _, imui_in};"));
    assert!(rustdoc.contains("`use fret::imui::prelude::*;`"));
    assert!(rustdoc.contains("if ui.button(\"Save\").clicked() {"));
    assert!(rustdoc.contains("Reach for `fret::imui::kit` for policy-heavy widgets,"));
    assert!(rustdoc.contains("`fret::imui::{prelude::*, kit, editor, docking}`"));
    assert!(public_surface.contains("pub mod imui {"));
    assert!(public_surface.contains("prelude::UiWriter"));
    assert!(!app_prelude_exports_symbol("ImUi"));
    assert!(!app_prelude_exports_symbol("ImUiFacade"));
    assert!(!app_prelude_exports_symbol("UiWriterImUiFacadeExt"));
    assert!(!app_prelude_exports_symbol("UiWriterUiKitExt"));
}

#[test]
fn readme_and_rustdoc_expose_selector_and_query_as_explicit_optional_surfaces() {
    assert!(CRATE_README.contains("`cx.data().selector_layout(...)`"));
    assert!(CRATE_README.contains("raw `cx.data().selector(...)`"));
    assert!(CRATE_README.contains("`handle.read_layout(cx)`"));
    assert!(CRATE_README.contains("`cx.data().invalidate_query(...)`"));
    assert!(CRATE_README.contains("`cx.data().invalidate_query_namespace(...)`"));
    assert!(CRATE_README.contains("`cx.data().cancel_query(...)`"));
    assert!(CRATE_README.contains("`cx.data().query_snapshot_entry(...)`"));
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
    assert!(!selector_surface.contains("pub use crate::view::LocalSelectorDepsBuilderExt;"));
    assert!(selector_surface.contains("pub use fret_selector::{DepsSignature, Selector};"));
    assert!(selector_surface.contains("pub use fret_selector::ui::DepsBuilder;"));
    assert!(!selector_surface.contains("pub use fret_selector::ui::*;"));
    assert!(query_surface.contains("pub mod query {"));
    assert!(query_surface.contains("pub mod core {"));
    assert!(!query_surface.contains("pub mod ui {"));
    assert!(query_surface.contains("pub use fret_query::{"));
    assert!(query_surface.contains("QueryKey, QueryPolicy"));
    assert!(query_surface.contains("QueryState,"));
    assert!(!app_prelude_exports_symbol("DepsBuilder"));
    assert!(!app_prelude_exports_symbol("DepsSignature"));
    assert!(!app_prelude_exports_symbol("LocalSelectorDepsBuilderExt"));
    assert!(!app_prelude_exports_symbol("QueryKey"));
    assert!(!app_prelude_exports_symbol("QueryPolicy"));
    assert!(!app_prelude_exports_symbol("QueryHandle"));
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
    assert!(CRATE_README.contains("`FretApp::asset_startup(...)`"));
    assert!(CRATE_README.contains("`UiAppBuilder::with_asset_startup(...)`"));
    assert!(CRATE_README.contains("`FileAssetManifestResolver::from_bundle_dir(...)`"));
    assert!(CRATE_README.contains("`FileAssetManifestResolver::from_manifest_path(...)`"));
    assert!(CRATE_README.contains("`register_resolver(...)`"));
    assert!(!CRATE_README.contains("`FretApp::asset_dir(...)`"));
    assert!(!CRATE_README.contains("`UiAppBuilder::with_asset_dir(...)`"));
    assert!(!CRATE_README.contains("`FretApp::asset_manifest(...)`"));
    assert!(!CRATE_README.contains("`UiAppBuilder::with_asset_manifest(...)`"));
    assert!(
        CRATE_README
            .contains("`fret::app::ui_assets::image_source_state_from_asset_request(cx, ...)`")
    );
    assert!(
        CRATE_README
            .contains("`fret::app::ui_assets::svg_source_state_from_asset_request(cx, ...)`")
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
    assert!(rustdoc.contains("`FretApp::asset_startup(...)`"));
    assert!(rustdoc.contains("`UiAppBuilder::with_asset_startup(...)`"));
    assert!(rustdoc.contains("`FileAssetManifestResolver::from_bundle_dir(...)`"));
    assert!(rustdoc.contains("`FileAssetManifestResolver::from_manifest_path(...)`"));
    assert!(rustdoc.contains("`register_resolver(...)`"));
    assert!(!rustdoc.contains("`register_file_bundle_dir(...)`"));
    assert!(!rustdoc.contains("`register_file_manifest(...)`"));
    assert!(!rustdoc.contains("`FretApp::asset_dir(...)`"));
    assert!(!rustdoc.contains("`UiAppBuilder::with_asset_dir(...)`"));
    assert!(!rustdoc.contains("`FretApp::asset_manifest(...)`"));
    assert!(!rustdoc.contains("`UiAppBuilder::with_asset_manifest(...)`"));
    assert!(rustdoc.contains("`AssetLocator::file(...)`"));
    assert!(rustdoc.contains("`AssetLocator::url(...)`"));
    assert!(
        rustdoc.contains("`fret::app::ui_assets::image_source_state_from_asset_request(cx, ...)`")
    );
    assert!(
        rustdoc.contains("`fret::app::ui_assets::svg_source_state_from_asset_request(cx, ...)`")
    );
    assert!(public_surface.contains("pub mod assets {"));
    assert!(!public_surface.contains("pub use fret_runtime::register_bundle_asset_entries;"));
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
    let curated_lane = "`fret::shadcn::{Button, Card, ...}`";
    let raw_escape_hatch = "`fret::shadcn::raw::*`";
    assert!(rustdoc.contains(curated_lane));
    assert!(rustdoc.contains("`shadcn::app::install(...)`"));
    assert!(rustdoc.contains("`shadcn::themes::apply_shadcn_new_york(...)`"));
    assert!(rustdoc.contains("are setup lanes\n//!   rather than peer discovery lanes"));
    assert!(
        rustdoc
            .find(curated_lane)
            .expect("rustdoc should teach the curated shadcn lane")
            < rustdoc
                .find(raw_escape_hatch)
                .expect("rustdoc should retain raw shadcn as an explicit escape hatch"),
        "rustdoc should teach the curated shadcn lane before the raw escape hatch"
    );
    assert!(!rustdoc.contains(
        "//! - use `fret::shadcn::{..., app::install, themes::apply_shadcn_new_york, raw::*}`"
    ));
    assert!(rustdoc.contains("`fret::shadcn::raw::advanced::*`"));
    assert!(public_surface.contains("pub use fret_ui_shadcn::facade as shadcn;"));
    assert!(!public_surface.contains("pub use fret_ui_shadcn as shadcn;"));
}

#[test]
fn crate_docs_only_teach_view_entry() {
    let rustdoc = crate_rustdoc();
    let getting_started = rustdoc
        .find("//! ## Getting started (desktop)")
        .expect("rustdoc should lead with the default app skeleton");
    let choosing_entry = rustdoc
        .find("//! ## Choosing a native entry path")
        .expect("rustdoc should still include progressive entry-path guidance");
    assert!(getting_started < choosing_entry);
    assert!(
        rustdoc.contains(
            "//! - Default app path: `fret::FretApp::new(...).window(...).view::<V>()?`."
        )
    );
    assert!(rustdoc.contains("//! - Advanced/manual assembly:"));
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
    assert!(FIRST_HOUR.contains("`local.layout_value(cx)` / `local.paint_value(cx)`"));
    assert!(FIRST_HOUR.contains(
        "`local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)`"
    ));
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
        !FIRST_HOUR.contains("Convert into `AnyElement` at the boundary via `.into_element(cx)`.")
    );
    assert!(!FIRST_HOUR.contains("cx.watch_model(&models.clicks)"));
    assert!(!FIRST_HOUR.contains("`fret_ui_shadcn::prelude::*`"));
    assert!(!FIRST_HOUR.contains("let clicks = clicks_state.paint(cx).value_or_default();"));
    assert!(!FIRST_HOUR.contains("let label = label_state.layout(cx).value_or_default();"));
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
    assert!(CRATE_USAGE_GUIDE.contains("start with `View` +"));
    assert!(CRATE_USAGE_GUIDE.contains("`AppUi` + typed actions"));
    assert!(CRATE_USAGE_GUIDE.contains("`app.local_state(value)`"));
    assert!(
        CRATE_USAGE_GUIDE.contains("`cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().models::<A>(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().payload_models::<A>(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().transient::<A>(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.effects().toast_message(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.effects().toast_dismiss_all()`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::async_work::{...}` for"));
    assert!(CRATE_USAGE_GUIDE.contains("`register_inbox_drainer(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`inbox_local(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`inbox_drain_apply(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AppUiRawActionNotifyExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::canvas::{...}` lane"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`Canvas`, `CanvasSurface`, `PanZoomCanvas`, `AppCanvasPainter`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("path/key helpers"));
    assert!(CRATE_USAGE_GUIDE.contains("wheel streams"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_ui::canvas::CanvasPainter`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::chart::{...}` lane"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`ChartCanvas`, `ChartEngine`, `ChartCanvasOutput`, `ChartInputMap`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`fret_chart::ChartCanvasPanelProps`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::app::{LocalState, LocalStateTxn}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`app.local_state_txn(|tx| ...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::commands::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains(
        "`fret::pointer::{PointerRegion, PointerDown, PointerMove, PointerUp, MouseButton, ...}`"
    ));
    assert!(CRATE_USAGE_GUIDE.contains("`UiPointerActionHost`, `PointerRegionProps`,"));
    assert!(CRATE_USAGE_GUIDE.contains("`prevent_focus_on_pointer_down()`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::style::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::style::ThemeSnapshot`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::icons::{icon, IconId}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::semantics::{SemanticsDecoration, SemanticsRole}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::children::UiElementSinkExt as _`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::actions::ElementCommandGatingExt as _`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::assets::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupMode`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::selector::ui::DepsBuilder`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::selector::DepsSignature`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::query::{QueryKey, QueryPolicy, QueryState, ...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetBundleId::app(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetBundleId::package(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetLocator::bundle(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`register_bundle_entries(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`FretApp::asset_startup(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_asset_startup(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`FileAssetManifestResolver::from_bundle_dir(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`FileAssetManifestResolver::from_manifest_path(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`register_resolver(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`fret::assets::register_file_bundle_dir(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`fret::assets::register_file_manifest(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`FretApp::asset_dir(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_asset_dir(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`FretApp::asset_manifest(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`UiAppBuilder::with_asset_manifest(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`BootstrapBuilder::with_asset_startup(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan::development_dir(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan::development_manifest(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan::packaged_bundle_entries(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AssetStartupPlan::packaged_embedded_entries(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("the bootstrap crate also exposes the matching"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`fret::app::ui_assets::image_source_state_from_asset_request(cx, ...)`")
    );
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`fret::app::ui_assets::svg_source_state_from_asset_request(cx, ...)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`widget.action(act::Save)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`widget.action_payload(act::Remove, payload)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`widget.listen(|host, acx| { ... })`"));
    assert!(CRATE_USAGE_GUIDE.contains("`use fret::app::AppActivateExt as _;`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.actions().listen(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().action(act::Save)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().action_payload(act::Remove, payload)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`local.layout_value(cx)` / `local.paint_value(cx)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`tx.value(&local)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`widget.dispatch::<A>()`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`widget.dispatch_payload::<A>(payload)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().dispatch::<A>()`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.actions().dispatch_payload::<A>(payload)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`AppRenderActionsExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_ui_kit::ui::hover_region(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_ui_kit::ui::rich_text(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().selector_layout(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("raw `cx.data().selector(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`handle.read_layout(cx)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query_namespace(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().cancel_query(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_snapshot_entry(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains(
        "`local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)`"
    ));
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

    let component_row = markdown_table_row(AUTHORING_SURFACE_MIGRATION_MATRIX, "Component prelude");
    assert!(component_row.contains("| Migrated |"));

    let advanced_row = markdown_table_row(AUTHORING_SURFACE_MIGRATION_MATRIX, "Advanced imports");
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
    assert!(COMPONENT_AUTHOR_GUIDE.contains("typed domain callbacks into `AppActivateSurface`"));
    assert!(
        COMPONENT_AUTHOR_GUIDE.contains("parallel `AppActionCxSurface` / `AppActionCxExt` family")
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
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().selector_layout(...)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().selector(deps, compute)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().query(...)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`handle.read_layout(cx)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().invalidate_query(...)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().cancel_query(...)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`cx.data().query_snapshot_entry(...)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains(
        "`local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)`"
    ));
    assert!(AUTHORING_GOLDEN_PATH.contains("`ui::single(cx, child)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`.action(act::Save)`"));
    assert!(AUTHORING_GOLDEN_PATH.contains(".action_payload(act::RemoveTodo, todo.id);"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`.listen(|host, acx| { ... })`"));
    assert!(AUTHORING_GOLDEN_PATH.contains("`use fret::app::AppActivateExt as _;`"));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`cx.actions().action(act::Save)`"));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`cx.actions().action_payload("));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`.dispatch::<A>()`"));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`.dispatch_payload::<A>(payload)`"));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`cx.use_selector(...)`"));
    assert!(!AUTHORING_GOLDEN_PATH.contains("`cx.use_query(...)`"));
}

#[test]
fn integration_docs_prefer_grouped_query_helpers_for_app_surface() {
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async(...)`"));
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().query_async_local(...)`"));
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("let state = handle.read_layout(cx);"));
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().invalidate_query_namespace(...)`"));
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().query_async(...)`"));
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().invalidate_query_namespace(...)`"));
    assert!(
        INTEGRATING_SQLITE_AND_SQLX
            .contains("`cx.data().invalidate_query_namespace_after_mutation_success(...)`")
    );
}

#[test]
fn docs_lock_query_reads_vs_mutation_submit_story() {
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("If the flow is click-driven"));
    assert!(
        INTEGRATING_TOKIO_AND_REQWEST.contains("submit work (POST/PUT/DELETE, Save, Run, Sync)")
    );
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`state-mutation`"));
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`cx.data().mutation_async(...)`"));
    assert!(INTEGRATING_TOKIO_AND_REQWEST.contains("`handle.submit(...)`"));
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().mutation_async(...)`"));
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.data().mutation_async_local(...)`"));
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("`cx.actions().mutation_submit(...)`"));
    let sqlite_default_lane = INTEGRATING_SQLITE_AND_SQLX
        .split("## 3) Advanced/manual surfaces")
        .next()
        .expect("SQLite guide should keep an explicit advanced/manual boundary");
    for raw_crate in [
        "fret_app::",
        "fret_runtime::",
        "fret_core::",
        "fret_ui::",
        "fret_canvas::",
        "fret_chart::",
    ] {
        assert!(
            !sqlite_default_lane.contains(raw_crate),
            "default SQLite lane leaked raw crate path `{raw_crate}`"
        );
    }
    assert!(!sqlite_default_lane.contains("handle.submit(models"));
    assert!(sqlite_default_lane.contains("fret::actions!([SaveTodo = \"todo.save\"]);"));
    assert!(sqlite_default_lane.contains(".action(act::SaveTodo)"));
    assert!(
        INTEGRATING_SQLITE_AND_SQLX
            .contains("`cx.data().invalidate_query_namespace_after_mutation_success(...)`")
    );
    assert!(
        INTEGRATING_SQLITE_AND_SQLX
            .contains("`apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`")
    );
    assert!(INTEGRATING_SQLITE_AND_SQLX.contains("let save_state = save_todo.read_layout(cx);"));
    assert!(!INTEGRATING_SQLITE_AND_SQLX.contains("cx.root_state(Option::<Instant>::default"));
    assert!(
        INTEGRATING_SQLITE_AND_SQLX
            .contains("Do not teach a Save/Delete/Sync flow as `query_async(...)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("### `fret-mutation`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().mutation_async(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().mutation_async_local(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`handle.submit(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`handle.submit_action(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`handle.retry_last(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().update_after_mutation_completion(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().take_mutation_completion(...)`"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`cx.data().invalidate_query_namespace_after_mutation_success(...)`")
    );
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`fret::mutation::{MutationPolicy, MutationState, ...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("raw `fret-executor` + `InboxDrainer`"));
}

#[test]
fn usage_docs_expose_router_as_explicit_extension_surface() {
    assert!(CRATE_USAGE_GUIDE.contains("enable `fret`'s `router` feature"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::router::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`router_outlet_by_leaf_with_test_id(...)`"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`bind_history_actions(cx, &store, act::RouterBack, act::RouterForward)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`bind_history_actions(...)`"));
    assert!(
        !CRATE_USAGE_GUIDE.contains("`use fret::advanced::raw::AppUiRawActionNotifyExt as _;`")
    );
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.on_action_notify::<...>(store.back_on_action())`"));
    assert!(CRATE_USAGE_GUIDE.contains("second default app runtime"));
}

#[test]
fn usage_docs_link_ecosystem_trait_budget_and_anti_plugin_posture() {
    assert!(CRATE_USAGE_GUIDE.contains("## Ecosystem author checklist"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::integration::InstallIntoApp`"));
    assert!(CRATE_USAGE_GUIDE.contains("one installer/bundle surface"));
    assert!(CRATE_USAGE_GUIDE.contains("`RouteCodec`"));
    assert!(CRATE_USAGE_GUIDE.contains("`DockPanelElementRegistry`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret-app::Plugin`"));
    assert!(
        CRATE_USAGE_GUIDE.contains("`docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`")
    );
}

#[test]
fn usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems() {
    assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(fret_icons_lucide::app::install)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(fret_icons_radix::app::install)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_icons_lucide::app::install`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_icons_radix::app::install`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fretboard icons import iconify-collection --source ./iconify/lucide.json --crate-name lucide-icons --vendor-namespace lucide`"));
    assert!(CRATE_USAGE_GUIDE.contains("`--semantic-aliases ./semantic-aliases.json`"));
    assert!(CRATE_USAGE_GUIDE.contains("`--presentation-defaults ./presentation-defaults.json`"));
    assert!(CRATE_USAGE_GUIDE.contains(
        "`fretboard icons suggest presentation-defaults --provenance ./iconify/mdi-home.provenance.json --out ./iconify/presentation-defaults.json`"
    ));
    assert!(CRATE_USAGE_GUIDE.contains(
        "`fretboard icons suggest svg-dir-presentation-overrides --source ./icons --out ./presentation-defaults.json --report-out ./presentation-defaults.report.json`"
    ));
    assert!(
        CRATE_USAGE_GUIDE.contains("`--report-out ./iconify/presentation-defaults.report.json`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("\"schema_version\": 1"));
    assert!(CRATE_USAGE_GUIDE.contains("\"semantic_id\": \"ui.search\""));
    assert!(CRATE_USAGE_GUIDE.contains("\"default_render_mode\": \"mask\""));
    assert!(CRATE_USAGE_GUIDE.contains("\"render_mode\": \"original-colors\""));
    assert!(CRATE_USAGE_GUIDE.contains("`target_icon` must use the generated icon name"));
    assert!(CRATE_USAGE_GUIDE.contains("`icon_name` must also use the generated icon name"));
    assert!(CRATE_USAGE_GUIDE.contains("Unlisted icons use `default_render_mode`;"));
    assert!(CRATE_USAGE_GUIDE.contains("Treat that file as advisory and review it"));
    assert!(CRATE_USAGE_GUIDE.contains("optional versioned report records"));
    assert!(CRATE_USAGE_GUIDE.contains("only emits per-icon `original-colors` overrides"));
    assert!(CRATE_USAGE_GUIDE.contains("It does not\n  infer `default_render_mode`"));
    assert!(CRATE_USAGE_GUIDE.contains(
        "Review the emitted config before passing `--presentation-defaults` into `icons import ...`."
    ));
    assert!(CRATE_USAGE_GUIDE.contains("`PACK_METADATA` and a data-first registration value"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`BootstrapBuilder::register_icon_pack_contract(my_icons::PACK)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`"));
    assert!(CRATE_USAGE_GUIDE.contains("`FretApp::setup(MyKitBundle)`"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("`IconRegistry` mutation plus `register_bundle_entries(...)` manually")
    );
    assert!(
        CRATE_USAGE_GUIDE.contains("`fret::app::ui_assets::configure_caches_with_budgets(...)`")
    );
    assert!(CRATE_USAGE_GUIDE.contains(
        "`fret_ui_assets::advanced::{configure_caches_with_ui_services(...), configure_caches_with_ui_services_and_budgets(...)}`"
    ));
    assert!(CRATE_USAGE_GUIDE.contains("`fret_node::app::install(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::router::app::install(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`BootstrapBuilder::register_icon_pack(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`BootstrapBuilder::register_icon_pack_contract(...)`"));
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
    assert!(CRATE_USAGE_GUIDE.contains("generated helpers in one named installer/bundle surface"));
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
    assert!(COMPONENT_AUTHOR_GUIDE.contains(
        "widget code stays on semantic `IconId`s and logical `AssetLocator::bundle(...)` lookups"
    ));
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
    assert!(COMPONENT_AUTHOR_GUIDE.contains("Use `Embedded` for lower-level owner-scoped bytes"));
    assert!(COMPONENT_AUTHOR_GUIDE.contains("crate's public cross-package"));
    assert!(COMPONENT_AUTHOR_GUIDE.contains("lookup contract"));
    assert!(
        ECOSYSTEM_INSTALLER_COMPOSITION.contains("the app composes one installer/bundle value")
    );
    assert!(ECOSYSTEM_INSTALLER_COMPOSITION.contains("The app should not usually do this:"));
    assert!(
        ECOSYSTEM_INSTALLER_COMPOSITION.contains("### Generated module vs higher-level installer")
    );
    assert!(
        ECOSYSTEM_INSTALLER_COMPOSITION
            .contains("generated modules own low-level byte publication")
    );
    assert!(ECOSYSTEM_INSTALLER_COMPOSITION.contains("### `BundleAsset` vs `Embedded`"));
    assert!(ECOSYSTEM_INSTALLER_COMPOSITION.contains("If you are unsure, choose `BundleAsset`."));
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
        COMPONENT_AUTHOR_GUIDE.contains("`OverlayController`, `OverlayRequest`, `OverlayPresence`")
    );
}

#[test]
fn todo_golden_path_keeps_icon_pack_setup_on_app_install_surface() {
    assert!(TODO_APP_GOLDEN_PATH.contains("`.setup(fret_icons_radix::app::install)`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`.register_icon_pack_contract(my_icons::PACK)`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`fretboard icons import iconify-collection --source ./iconify/lucide.json --crate-name lucide-icons --vendor-namespace lucide`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`--semantic-aliases ./semantic-aliases.json`"));
    assert!(
        TODO_APP_GOLDEN_PATH.contains("`--presentation-defaults ./presentation-defaults.json`")
    );
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "`fretboard icons suggest presentation-defaults --provenance ./iconify/mdi-home.provenance.json --out ./iconify/presentation-defaults.json`"
    ));
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "fretboard icons suggest svg-dir-presentation-overrides --source ./icons --out ./presentation-defaults.json --report-out ./presentation-defaults.report.json"
    ));
    assert!(
        TODO_APP_GOLDEN_PATH.contains("`--report-out ./iconify/presentation-defaults.report.json`")
    );
    assert!(TODO_APP_GOLDEN_PATH.contains("\"schema_version\": 1"));
    assert!(TODO_APP_GOLDEN_PATH.contains("\"default_render_mode\": \"mask\""));
    assert!(TODO_APP_GOLDEN_PATH.contains("\"render_mode\": \"original-colors\""));
    assert!(TODO_APP_GOLDEN_PATH.contains("`target_icon` should match the generated icon name."));
    assert!(
        TODO_APP_GOLDEN_PATH.contains("`icon_name` should also match the generated icon name.")
    );
    assert!(TODO_APP_GOLDEN_PATH.contains("Unlisted icons use `default_render_mode`;"));
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "Treat the emitted file as advisory and review it before passing it into `icons import ...`."
    ));
    assert!(TODO_APP_GOLDEN_PATH.contains("optional versioned report keeps"));
    assert!(TODO_APP_GOLDEN_PATH.contains("only suggests `original-colors` overrides"));
    assert!(TODO_APP_GOLDEN_PATH.contains("It does not infer\n`default_render_mode`"));
    assert!(TODO_APP_GOLDEN_PATH.contains(
        "review the emitted files before passing `--presentation-defaults` into\n`icons import ...`."
    ));
    assert!(TODO_APP_GOLDEN_PATH.contains("`my_icons::app::install(...)`"));
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
fn usage_docs_expose_curated_component_surface() {
    assert!(CRATE_USAGE_GUIDE.contains("`use fret::component::prelude::*;`"));
    assert!(CRATE_USAGE_GUIDE.contains("`ComponentCx`"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiBuilder`/`UiPatchTarget`/`IntoUiElement<H>`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::commands::{CommandId, CommandMeta, ...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::adaptive::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::activate::{on_activate,"));
    assert!(CRATE_USAGE_GUIDE.contains("`use fret::advanced::prelude::*;`"));
    assert!(CRATE_USAGE_GUIDE.contains("advanced-only"));
    assert!(
        CRATE_USAGE_GUIDE.contains("without pulling in `FretApp`, `AppUi`, or runner-facing seams")
    );
}

#[test]
fn usage_docs_expose_shadcn_app_surface_as_explicit_submodule() {
    assert!(CRATE_USAGE_GUIDE.contains("`use fret_ui_shadcn::{facade as shadcn, prelude::*};`"));
    assert!(CRATE_USAGE_GUIDE.contains("`shadcn::app::install(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`shadcn::themes::apply_shadcn_new_york(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("component-family discovery lane"));
    assert!(CRATE_USAGE_GUIDE.contains("`shadcn::app::*` and `shadcn::themes::*` are setup lanes"));
    assert!(CRATE_USAGE_GUIDE.contains(
        "`fret_ui_shadcn::advanced::{sync_theme_from_environment(...), install_with_ui_services(...)}`"
    ));
    assert!(
        CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::advanced::*` is an implementation/debug lane")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`shadcn::raw::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`shadcn::typography::*` facade module"));
    assert!(!CRATE_USAGE_GUIDE.contains("`shadcn::raw::typography::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::app::install(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::themes::apply_shadcn_new_york(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::app::*` and"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::themes::*` are setup lanes"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::raw::advanced::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::shadcn::typography::*` facade module"));
    assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::install_app(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`fret_ui_shadcn::shadcn_themes::"));
    assert!(!CRATE_USAGE_GUIDE.contains("`fret::shadcn::shadcn_themes::"));
}

#[test]
fn shadcn_docs_keep_advanced_hooks_off_curated_lane() {
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`widget.action(act::Save)`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`widget.action_payload(act::Remove, payload)`"));
    assert!(
        SHADCN_DECLARATIVE_PROGRESS.contains("`fret::app::AppActivateSurface` / `AppActivateExt`")
    );
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`use fret::app::AppActivateExt as _;`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`AppRenderActionsExt` / `AppRenderDataExt`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`fret_ui_kit::ui::hover_region(...)`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`fret_ui_kit::ui::rich_text(...)`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("first-party"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("bridge table is intentionally empty"));
    assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`.dispatch::<A>()`"));
    assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`.dispatch_payload::<A>(payload)`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`fret_ui_shadcn::advanced::*`"));
    assert!(!SHADCN_DECLARATIVE_PROGRESS.contains("`shadcn::advanced::*`"));
    assert!(AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret_ui_shadcn::advanced`"));
    assert!(AUTHORING_SURFACE_TARGET_INTERFACE_STATE.contains("`fret::shadcn::raw::advanced::*`"));
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
    assert!(SHADCN_SELECT_V4_USAGE.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
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
    assert!(FEARLESS_REFACTORING.contains("Return `Ui` (the app-facing alias over `Elements`)"));
    assert!(FEARLESS_REFACTORING.contains("`cx.actions().locals_with((...)).on::<A>(...)`"));
    assert!(FEARLESS_REFACTORING.contains("`cx.actions().models::<A>(...)`"));
    assert!(FEARLESS_REFACTORING.contains("`cx.actions().payload_models::<A>(...)`"));
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
    assert!(app_contains(
        "pub use crate::view::{AppActivateExt, AppActivateSurface};"
    ));
    assert!(app_prelude.contains("pub use crate::{"));
    assert!(app_prelude.contains("pub use crate::app::App;"));
    assert!(app_prelude.contains("pub use crate::app::AppRenderCx;"));
    assert!(app_prelude.contains("pub use crate::app::text;"));
    assert!(app_prelude.contains("pub use crate::app::ui_assets;"));
    assert!(app_prelude_exports_symbol("App"));
    assert!(app_prelude_exports_symbol("AppRenderCx"));
    assert!(app_prelude_exports_symbol("text"));
    assert!(app_prelude_exports_symbol("ui_assets"));
    assert!(!app_prelude_exports_symbol("AppComponentCx"));
    assert!(!app_prelude_exports_symbol("UiCx"));
    assert!(app_prelude.contains("AppUi"));
    assert!(!app_prelude_exports_symbol("KernelApp"));
    assert!(app_prelude.contains("UiChild"));
    assert!(app_prelude.contains("WindowId"));
    assert!(app_prelude_exports_symbol("Px"));
    assert!(!app_prelude_exports_symbol("LocalState"));
    assert!(!app_prelude_exports_symbol("LocalStateTxn"));
    assert!(!app_prelude_exports_symbol("AppLocalStateExt"));
    assert!(!app_prelude_exports_symbol("AppLocalStateTxnExt"));
    assert!(!app_prelude_exports_symbol("UiActionHostLocalStateTxnExt"));
    assert!(!app_prelude_exports_symbol("CommandId"));
    assert!(!app_prelude_exports_symbol("ThemeSnapshot"));
    assert!(!app_prelude_exports_symbol("actions"));
    assert!(!app_prelude_exports_symbol("workspace_menu"));
    assert!(!app_prelude_exports_symbol("in_window_menubar"));
    assert!(!app_prelude.contains("pub use fret_ui_kit::declarative::icon;"));
    assert!(!app_prelude.contains("pub use crate::view::AppActivateExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::QueryHandleReadLayoutExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::TrackedStateExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::UiActionHostLocalStateTxnExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::AppLocalStateExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::AppLocalStateTxnExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(app_prelude.contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(app_prelude.contains("pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;"));
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
    assert!(!app_prelude_exports_symbol("QueryHandleReadLayoutExt"));
    assert!(!app_prelude.contains("pub use crate::view::{AppActivateExt, AppActivateSurface};"));
    assert!(!app_prelude_exports_symbol("TrackedStateExt"));
    assert!(!app_prelude_exports_symbol("AnyElementSemanticsExt"));
    assert!(!app_prelude_exports_symbol("ElementContextThemeExt"));
    assert!(!app_prelude_exports_symbol("UiElementA11yExt"));
    assert!(!app_prelude_exports_symbol("UiElementKeyContextExt"));
    assert!(!app_prelude_exports_symbol("UiElementTestIdExt"));
    assert!(!app_prelude.contains("pub use fret_ui_kit::command::ElementCommandGatingExt as _;"));
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
    assert!(!app_prelude_exports_symbol("Color"));
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
    assert!(!app_prelude_exports_symbol("SemanticsDecoration"));
    assert!(!app_prelude_exports_symbol("AdaptiveQuerySource"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptiveClass"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptivePolicy"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptiveSnapshot"));
    assert!(!app_prelude_exports_symbol("DeviceShellMode"));
    assert!(!app_prelude_exports_symbol("DeviceShellSwitchPolicy"));
    assert!(!app_prelude_exports_symbol("PanelAdaptiveClass"));
    assert!(!app_prelude_exports_symbol("PanelAdaptivePolicy"));
    assert!(!app_prelude_exports_symbol("device_adaptive_class"));
    assert!(!app_prelude_exports_symbol("device_adaptive_snapshot"));
    assert!(!app_prelude_exports_symbol("device_shell_mode"));
    assert!(!app_prelude_exports_symbol("device_shell_switch"));
    assert!(!app_prelude_exports_symbol("panel_adaptive_class"));
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
    assert!(!app_prelude_exports_symbol("ImUi"));
    assert!(!app_prelude_exports_symbol("ImUiFacade"));
    assert!(!app_prelude_exports_symbol("UiWriterImUiFacadeExt"));
    assert!(!app_prelude_exports_symbol("UiWriterUiKitExt"));
    assert!(!app_prelude_exports_symbol("ResponseExt"));
    assert!(!app_prelude_exports_symbol("editor"));
    assert!(!app_prelude_exports_symbol("docking"));
    assert!(!app_prelude_exports_symbol("handle_dock_op"));
    assert!(!app_prelude_exports_symbol("InstallConfig"));
}

#[test]
fn app_prelude_pub_use_budget_is_curated_and_closed() {
    let app_prelude = app_prelude_source();
    let actual_lines = pub_use_lines(app_prelude);
    let expected_lines = [
        "pub use crate::FretApp;",
        "pub use crate::app::App;",
        "pub use crate::app::AppRenderContext;",
        "pub use crate::app::AppRenderCx;",
        "pub use crate::app::text;",
        "pub use crate::app::ui_assets;",
        "pub use crate::shadcn;",
        "pub use crate::view::AppLocalStateExt as _;",
        "pub use crate::view::AppLocalStateTxnExt as _;",
        "pub use crate::view::AppRenderActionsExt as _;",
        "pub use crate::view::AppRenderDataExt as _;",
        "pub use crate::view::MutationHandleReadLayoutExt as _;",
        "pub use crate::view::QueryHandleReadLayoutExt as _;",
        "pub use crate::view::TrackedStateExt as _;",
        "pub use crate::view::UiActionHostLocalStateTxnExt as _;",
        "pub use crate::view::View;",
        "pub use crate::{AppUi, Ui, UiChild, WindowId};",
        "pub use fret_core::Px;",
        "pub use fret_ui::Invalidation;",
        "pub use fret_ui_kit::IntoUiElement as _;",
        "pub use fret_ui_kit::IntoUiElementInExt as _;",
        "pub use fret_ui_kit::StyledExt as _;",
        "pub use fret_ui_kit::UiExt as _;",
        "pub use fret_ui_kit::declarative::AnyElementSemanticsExt as _;",
        "pub use fret_ui_kit::declarative::UiElementA11yExt as _;",
        "pub use fret_ui_kit::declarative::UiElementTestIdExt as _;",
        "pub use fret_ui_kit::ui;",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_lines, expected_lines,
        "app prelude pub-use statements should stay on the approved Golden Path budget"
    );

    let actual_symbols = exported_symbol_names(app_prelude);
    let expected_symbols = [
        "App",
        "AppRenderContext",
        "AppRenderCx",
        "AppUi",
        "FretApp",
        "Invalidation",
        "Px",
        "Ui",
        "UiChild",
        "View",
        "WindowId",
        "shadcn",
        "text",
        "ui",
        "ui_assets",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual_symbols, expected_symbols,
        "app prelude named exports should stay limited to first-contact app authoring nouns"
    );

    let compact = app_prelude.split_whitespace().collect::<String>();
    assert!(compact.contains("#[cfg(feature=\"shadcn\")]pubusecrate::shadcn;"));
    assert!(compact.contains("#[cfg(feature=\"ui-assets\")]pubusecrate::app::ui_assets;"));
    assert!(compact.contains(
        "#[cfg(feature=\"state-mutation\")]pubusecrate::view::MutationHandleReadLayoutExtas_;"
    ));
    assert!(compact.contains(
        "#[cfg(feature=\"state-query\")]pubusecrate::view::QueryHandleReadLayoutExtas_;"
    ));

    assert!(CRATE_USAGE_GUIDE.contains("`fret::app::prelude::*` is a closed Golden Path budget"));
    assert!(CRATE_USAGE_GUIDE.contains("closed Golden Path budget"));
    assert!(CRATE_USAGE_GUIDE.contains("first-contact app authoring nouns"));
    assert!(CRATE_USAGE_GUIDE.contains("Anonymous extension"));
    assert!(CRATE_USAGE_GUIDE.contains("traits are also part of the budget"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("must stay on explicit modules unless the Golden Path budget is")
    );
}

#[test]
fn app_text_facade_keeps_first_contact_text_off_raw_element_context() {
    let app_source = APP_RS;
    assert!(app_source.contains("pub mod text {"));
    assert!(app_source.contains("pub fn control_readout<'a, Cx, T>("));
    assert!(app_source.contains("pub fn control_label<'a, Cx, T>("));
    assert!(app_source.contains("pub fn paragraph<'a, Cx, T>("));
    assert!(app_source.contains("pub fn compact_paragraph<'a, Cx, T>("));
    assert!(app_source.contains("pub fn list_row_label<'a, Cx, T>("));
    assert!(app_source.contains("pub fn list_row_label_with_foreground<'a, Cx, T>("));
    assert!(app_source.contains("pub fn chrome_title<'a, Cx, T>("));
    assert!(app_source.contains("pub fn button_label<'a, Cx, T>("));
    assert!(app_source.contains("pub fn table_cell<'a, Cx, T>("));
    assert!(app_source.contains("pub fn table_cell_emphasis<'a, Cx, T>("));
    assert!(app_source.contains("pub fn list_row_label_attributed_with_foreground<'a, Cx>("));
    assert!(app_source.contains("pub fn section_chrome_label<'a, Cx, T>("));
    assert!(app_source.contains("pub fn chrome_glyph<'a, Cx, T>("));
    assert!(app_source.contains("pub fn code_label<'a, Cx, T>("));
    assert!(app_source.contains("pub fn code_block<'a, Cx, T>("));
    assert!(app_source.contains(") -> fret_ui::element::AnyElement"));
    assert!(app_source.contains("Cx: crate::app::AppRenderContext<'a>"));
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_control_readout(cx.elements(), text)")
    );
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_control_label(cx.elements(), text)")
    );
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_list_row_label(cx.elements(), text)")
    );
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_chrome_title(cx.elements(), text)")
    );
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_button_label(cx.elements(), text)")
    );
    assert!(
        app_source.contains("fret_ui_kit::declarative::text::text_table_cell(cx.elements(), text)")
    );
    assert!(
        app_source.contains(
            "fret_ui_kit::declarative::text::text_table_cell_emphasis(cx.elements(), text)"
        )
    );
    assert!(app_source.contains(
        "fret_ui_kit::declarative::text::text_list_row_label_attributed(cx.elements(), rich)"
    ));
    assert!(
        app_source
            .contains("fret_ui_kit::declarative::text::text_chrome_glyph(cx.elements(), text)")
    );
    assert!(
        app_source.contains("fret_ui_kit::declarative::text::text_code_block(cx.elements(), text)")
    );
    assert!(app_prelude_exports_symbol("text"));
    assert!(!app_prelude_exports_symbol("AnyElement"));
    assert!(!app_prelude_exports_symbol("ElementContext"));
}

#[test]
fn advanced_text_facade_keeps_manual_text_off_raw_kit_imports() {
    let advanced_surface = advanced_prelude_source();
    let advanced_common_prelude = advanced_common_prelude_source();

    assert!(advanced_surface.contains("pub mod text {"));
    assert!(advanced_surface.contains("pub fn control_readout<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("pub fn compact_paragraph<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("pub fn section_chrome_label<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("pub fn chrome_glyph<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("pub fn code_label<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("pub fn code_block<'a, H, Cx, T>("));
    assert!(advanced_surface.contains("H: fret_ui::UiHost + 'a"));
    assert!(advanced_surface.contains("Cx: fret_ui::ElementContextAccess<'a, H>"));
    assert!(
        advanced_surface
            .contains("fret_ui_kit::declarative::text::text_control_readout(cx.elements(), text)")
    );
    assert!(
        advanced_surface.contains(
            "fret_ui_kit::declarative::text::text_compact_paragraph(cx.elements(), text)"
        )
    );
    assert!(advanced_surface.contains(
        "fret_ui_kit::declarative::text::text_section_chrome_label(cx.elements(), text)"
    ));
    assert!(
        advanced_surface
            .contains("fret_ui_kit::declarative::text::text_chrome_glyph(cx.elements(), text)")
    );
    assert!(
        advanced_surface
            .contains("fret_ui_kit::declarative::text::text_code_label(cx.elements(), text)")
    );
    assert!(
        advanced_surface
            .contains("fret_ui_kit::declarative::text::text_code_block(cx.elements(), text)")
    );
    assert!(
        !advanced_common_prelude.contains("pub use crate::advanced::text"),
        "advanced text helpers should stay on the explicit module, not prelude imports"
    );
    assert!(CRATE_USAGE_GUIDE.contains("manual `KernelApp` or custom-host helpers"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::advanced::text::{control_readout"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("instead of importing `fret_ui_kit::declarative::text` in the app\nexample")
    );
}

#[test]
fn app_ui_assets_facade_keeps_asset_state_off_raw_element_context() {
    assert!(app_contains("pub mod ui_assets {"));
    assert!(app_contains(
        "pub use fret_core::{ImageColorSpace, ImageId};"
    ));
    assert!(app_contains(
        "pub use fret_ui_assets::image_asset_cache::{ImageAssetKey, ImageAssetStats};"
    ));
    assert!(app_contains("pub fn rgba8_image_state<'a, Cx>("));
    assert!(app_contains("pub fn image_source_state<'a, Cx>("));
    assert!(app_contains(
        "pub fn image_source_state_from_asset_request<'a, Cx>("
    ));
    assert!(app_contains(
        "pub fn svg_source_state_from_asset_request<'a, Cx>("
    ));
    assert!(app_contains("pub fn image_stats<'a, Cx>("));
    assert!(app_contains("pub fn svg_stats<'a, Cx>("));
    assert!(app_contains("Cx: crate::app::AppRenderContext<'a>"));
    assert!(app_contains(
        "fret_ui_assets::ui::use_rgba8_image_state_in(cx, width, height, rgba, color_space)"
    ));
    assert!(app_contains(
        "fret_ui_assets::ui::use_image_source_state_in(cx, source)"
    ));
    assert!(app_contains(
        "fret_ui_assets::ui::use_image_source_state_from_asset_request_in(cx, request)"
    ));
    assert!(app_contains(
        "fret_ui_assets::ui::svg_source_state_from_asset_request_in(cx, request)"
    ));
    assert!(app_contains("fret_ui_assets::ui::image_stats_in(cx)"));
    assert!(app_contains("fret_ui_assets::ui::svg_stats_in(cx)"));
    assert!(app_prelude_exports_symbol("ui_assets"));
    assert!(!app_prelude_exports_symbol("ImageSource"));
    assert!(!app_prelude_exports_symbol("ImageSourceState"));
    assert!(!app_prelude_exports_symbol("SvgAssetSourceState"));
}

#[test]
fn component_ui_assets_facade_keeps_snippet_assets_off_raw_extension_traits() {
    assert!(root_contains("pub mod component;"));
    assert!(component_contains("pub mod ui_assets {"));
    assert!(component_contains(
        "pub use fret_core::{ImageColorSpace, ImageId};"
    ));
    assert!(component_contains(
        "pub fn image_source_state<H: fret_ui::UiHost>("
    ));
    assert!(component_contains(
        "pub fn image_source_state_from_asset_request<H: fret_ui::UiHost>("
    ));
    assert!(component_contains(
        "pub fn rgba8_image_state<H: fret_ui::UiHost>("
    ));
    assert!(component_contains(
        "cx: &mut fret_ui::ElementContext<'_, H>"
    ));
    assert!(component_contains(
        "fret_ui_assets::ui::use_image_source_state_in(cx, source)"
    ));
    assert!(component_contains(
        "fret_ui_assets::ui::use_image_source_state_from_asset_request_in(cx, request)"
    ));
    assert!(CRATE_README.contains("`fret::component::ui_assets::*`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::component::ui_assets::*`"));
}

#[test]
fn app_module_explicitly_exports_activation_surface_and_extension() {
    assert!(app_contains(
        "pub use crate::view::{AppActivateExt, AppActivateSurface};"
    ));
}

#[test]
fn app_and_style_modules_expose_explicit_secondary_app_nouns() {
    let public_surface = crate_public_surface_source();
    assert!(root_contains(
        "pub type AppElement = fret_ui::element::AnyElement;"
    ));
    assert!(app_contains("pub use crate::AppElement;"));
    assert!(app_contains(
        "pub fn paragraph_break_words_with_foreground<'a, Cx, T>("
    ));
    assert!(app_contains("pub use crate::AppComponentCx;"));
    assert!(app_contains("pub use crate::AppRenderCx;"));
    assert_eq!(
        root_style_exported_symbols(),
        expected_root_style_exported_symbols(),
        "root style exports should be compared as symbols rather than rustfmt-sensitive text"
    );
    assert!(root_contains("pub mod scroll {"));
    assert!(root_contains("pub use fret_ui::scroll::ScrollHandle;"));
    assert!(root_contains(
        "pub use fret_core::time::{Duration, Instant, SystemTime, UNIX_EPOCH};"
    ));
    for symbol in [
        "AppLocalStateExt",
        "AppLocalStateTxnExt",
        "AppRenderActionsExt",
        "AppRenderContext",
        "AppRenderDataExt",
        "LocalState",
        "LocalStateTxn",
        "RenderContextAccess",
        "TrackedStateExt",
        "UiActionHostLocalStateTxnExt",
        "view_child",
        "view_child_with",
    ] {
        assert!(app_contains(symbol), "app facade should expose `{symbol}`");
    }
    assert!(!app_prelude_exports_symbol("view_child"));
    assert!(!app_prelude_exports_symbol("view_child_with"));
    assert!(!public_surface.contains("pub use crate::view::{UiCxActionsExt, UiCxDataExt};"));
    assert!(root_contains("pub use fret_ui::{Theme, ThemeSnapshot};"));
    assert!(app_contains("pub mod pressable {"));
    assert!(app_contains("pub use fret_ui::element::PressableState;"));
    assert!(app_contains("pub fn command_button<'a, Cx, L, I, T>("));
}

#[test]
fn ui_child_alias_uses_unified_component_conversion_trait() {
    let public_surface = ROOT_RS;
    assert!(public_surface.contains("pub type AppElement = fret_ui::element::AnyElement;"));
    assert!(
        public_surface.contains("pub trait UiChild: fret_ui_kit::IntoUiElement<crate::app::App>")
    );
    assert!(
        !public_surface
            .contains("pub trait UiChild: fret_ui_kit::UiChildIntoElement<crate::app::App>")
    );
}

#[test]
fn advanced_prelude_reexports_app_facing_view_aliases_without_raw_hooks() {
    let advanced_surface = advanced_prelude_source();
    let advanced_prelude = advanced_common_prelude_source();
    assert!(advanced_prelude.contains("pub use crate::AppComponentCx;"));
    assert!(advanced_prelude.contains("pub use crate::{AppUi, Ui};"));
    assert!(advanced_prelude.contains("pub use crate::AppRenderCx;"));
    assert!(advanced_common_prelude_exports_symbol("KernelApp"));
    assert!(!advanced_common_prelude_exports_symbol(
        "AppUiRawActionNotifyExt"
    ));
    assert!(!advanced_common_prelude_exports_symbol("AppUiRawStateExt"));
    assert!(!advanced_common_prelude_exports_symbol("AppUiRawModelExt"));
    assert!(!advanced_common_prelude_exports_symbol(
        "LocalStateRawModelExt"
    ));
    assert!(!advanced_common_prelude_exports_symbol(
        "LocalStateModelStoreExt"
    ));
    assert!(!advanced_common_prelude_exports_symbol(
        "LocalStateElementContextExt"
    ));
    assert!(advanced_surface.contains("pub mod raw;"));
    assert!(advanced_raw_contains(
        "pub use fret_runtime::{Model, ModelStore, ModelUpdateError};"
    ));
    assert!(advanced_raw_contains("pub use fret_ui::UiTree;"));
    assert!(advanced_raw_contains(
        "pub use fret_ui_kit::declarative::TrackedModelExt;"
    ));
    assert!(advanced_raw_contains("pub fn local_state_in<T>("));
    assert!(advanced_raw_contains(
        "pub use crate::view::AppUiRawActionNotifyExt;"
    ));
    assert!(advanced_raw_contains(
        "AppUiRawModelExt, LocalStateElementContextExt, LocalStateModelStoreExt,"
    ));
    assert!(advanced_surface.contains("pub mod driver;"));
    assert!(advanced_driver_contains(
        "pub use crate::{UiAppBuilder, UiAppDriver};"
    ));
    assert!(advanced_prelude.contains("pub use crate::advanced::driver::{"));
    assert!(advanced_common_prelude_exports_symbol("AppComponentCx"));
    assert!(advanced_common_prelude_exports_symbol("AppRenderCx"));
    assert!(advanced_common_prelude_exports_symbol("AppUi"));
    assert!(advanced_common_prelude_exports_symbol("Ui"));
    assert!(!advanced_common_prelude_exports_symbol("UiCx"));
    assert!(advanced_common_prelude_exports_symbol("ViewElements"));
    assert!(advanced_common_prelude_exports_symbol("ElementContext"));
    assert!(!advanced_common_prelude_exports_symbol("UiTree"));
    assert!(advanced_prelude.contains("pub use crate::view::QueryHandleReadLayoutExt as _;"));
    assert!(advanced_prelude.contains("pub use crate::view::AppRenderActionsExt as _;"));
    assert!(advanced_prelude.contains("pub use crate::view::AppRenderDataExt as _;"));
    assert!(!advanced_prelude.contains("TrackedModelExt"));
    assert!(advanced_common_prelude_exports_symbol("UiServices"));
    assert!(advanced_common_prelude_exports_symbol("TextProps"));
    assert!(!advanced_prelude.contains("pub use crate::component::prelude::*;"));
    assert!(!advanced_common_prelude_exports_symbol("UiBuilder"));
    assert!(!advanced_common_prelude_exports_symbol("UiPatchTarget"));
    assert!(!advanced_common_prelude_exports_symbol("IntoUiElement"));
    assert!(!advanced_common_prelude_exports_symbol("UiHost"));
    assert!(!advanced_common_prelude_exports_symbol("AnyElement"));
    assert!(!advanced_common_prelude_exports_symbol("Model"));
    assert!(!advanced_common_prelude_exports_symbol("TrackedModelExt"));
    assert!(!advanced_common_prelude_exports_symbol("ViewCx"));
    assert!(!advanced_common_prelude_exports_symbol("Elements"));
    assert!(
        !advanced_prelude
            .contains("pub use crate::view::{LocalState, TrackedStateExt, View, ViewCx};")
    );
    assert!(!advanced_prelude.contains(
        "pub use fret_ui::element::{Elements, HoverRegionProps, Length, SemanticsProps};"
    ));
    assert!(advanced_surface.contains("Import these from `fret::advanced::raw`"));
    assert!(advanced_surface.contains("They intentionally stay out of `advanced::prelude::*`"));
}

#[test]
fn retained_advanced_aliases_live_only_on_explicit_advanced_surface() {
    let public_surface = crate_public_surface_source();
    let root_header = root_surface_header_source();
    let advanced_prelude = advanced_prelude_source();
    assert!(!root_header.contains("pub use fret_app::App as KernelApp;"));
    assert!(!root_header.contains("pub use fret_bootstrap::ui_app_driver::ViewElements;"));
    assert!(!root_header.contains("pub use fret_framework as kernel;"));
    assert!(advanced_prelude.contains("pub use fret_app::App as KernelApp;"));
    assert!(advanced_driver_contains(
        "pub use fret_bootstrap::ui_app_driver::ViewElements;"
    ));
    assert!(advanced_prelude.contains("pub use fret_framework as kernel;"));
    assert!(root_contains(
        "pub type AppUi<'cx, 'a, H = crate::app::App>"
    ));
    assert!(root_contains(
        "pub type AppRenderCx<'a> = fret_ui::ElementContext<'a, crate::app::App>;"
    ));
    assert!(root_contains(
        "pub type ComponentCx<'a, H> = fret_ui::ElementContext<'a, H>;"
    ));
    assert!(root_contains(
        "pub type AppComponentCx<'a> = ComponentCx<'a, crate::app::App>;"
    ));
    assert!(!public_surface.contains("pub type UiCx<'a>"));
}

#[test]
fn root_surface_omits_low_level_action_registry_aliases() {
    let root_header = root_surface_header_source();
    let app_prelude = app_prelude_source();

    assert!(!root_header.contains("ActionMeta"));
    assert!(!root_header.contains("ActionRegistry"));
    assert!(root_header.contains("pub use fret_runtime::{ActionId, CommandId, TypedAction};"));
    assert!(ACTIONS_RS.contains("pub use fret_ui_kit::command::ElementCommandGatingExt;"));
    assert!(ACTIONS_RS.contains("pub use fret_runtime::{ActionId, CommandId, TypedAction};"));
    assert!(!ACTIONS_RS.contains("ActionMeta"));
    assert!(!ACTIONS_RS.contains("ActionRegistry"));
    assert!(!ACTIONS_RS.contains("pub type OnAction"));
    assert!(!ACTIONS_RS.contains("pub type OnPayloadAction"));
    assert!(!ACTIONS_RS.contains("pub type OnActionAvailability"));
    assert!(!ACTIONS_RS.contains("pub trait TypedActionMeta"));
    assert!(!ACTIONS_RS.contains("pub trait ActionRegistryExt"));
    assert!(!ACTIONS_RS.contains("pub struct ActionHandlerTable"));
    assert!(ACTIONS_RS.contains("pub(crate) struct ActionHandlerTable"));
    assert!(!app_prelude_exports_symbol("ActionMeta"));
    assert!(!app_prelude_exports_symbol("ActionRegistry"));
    assert!(!app_prelude.contains("ActionMeta"));
    assert!(!app_prelude.contains("ActionRegistry"));
    assert!(!app_prelude.contains("ElementCommandGatingExt"));
}

#[test]
fn root_surface_keeps_workspace_on_the_explicit_lane() {
    let root_header = root_surface_header_source();
    let public_surface = crate_public_surface_source();

    assert!(root_header.contains("pub mod workspace;"));
    assert!(!root_header.contains(
        "pub use workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu};"
    ));
    assert!(!public_surface.contains("pub mod workspace_shell;"));
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
        "adaptive",
        "activate",
        "actions",
        "advanced",
        "app",
        "assets",
        "async_work",
        "canvas",
        "chart",
        "children",
        "commands",
        "component",
        "env",
        "imui",
        "icons",
        "integration",
        "mutation",
        "overlay",
        "pointer",
        "query",
        "router",
        "scroll",
        "selector",
        "semantics",
        "style",
        "time",
        "virtual_list",
        "in_window_menubar",
        "workspace",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "root-level public modules should stay on the curated explicit-lane budget"
    );
    assert!(!root_header.contains("pub mod workspace_menu;"));
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
        "pub use builder::{UiAppBuilder, UiAppDriver};",
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
    assert!(root_header.contains("pub mod async_work {"));
    assert!(root_header.contains("pub mod canvas {"));
    assert!(root_header.contains("pub mod chart {"));
    assert!(root_header.contains("pub mod children {"));
    assert!(root_header.contains("pub mod commands {"));
    assert!(root_header.contains("pub mod icons {"));
    assert!(root_header.contains("pub mod pointer {"));
    assert!(root_header.contains("pub mod scroll {"));
    assert!(root_header.contains("pub mod semantics {"));
    assert!(root_header.contains("pub mod style {"));
    assert!(root_header.contains("pub mod virtual_list {"));
    assert!(root_header.contains("pub use fret_ui_kit::{"));
    assert!(root_header.contains("on_activate, on_activate_notify, on_activate_request_redraw,"));
    assert!(root_header.contains("on_activate_request_redraw_notify,"));
    assert!(root_header.contains("pub use fret_ui_kit::ui::UiElementSinkExt;"));
    assert!(root_header.contains("pub use fret_icons::IconId;"));
    assert!(root_header.contains("pub use fret_ui_kit::declarative::icon;"));
    assert!(root_header.contains("AppAsyncWorkExt, AppInboxCx, InboxLocal"));
    assert!(root_header.contains("inbox_drain_apply, inbox_local"));
    assert!(root_header.contains("pub use fret_runtime::{DispatchPriority, DispatcherHandle};"));
    assert!(root_header.contains("AppCanvasPainter, Canvas, CanvasSurface, PanZoomCanvas"));
    assert!(root_header.contains("PanZoom2D, visible_canvas_rect"));
    assert!(root_header.contains("PanZoomCanvasPaintCx, PanZoomInputPreset"));
    assert!(
        root_header
            .contains("pub use fret_core::scene::{Color, Paint as CanvasPaint, PaintBindingV1};")
    );
    assert!(root_header.contains("PathCommand, PathMetrics, PathStyle, Point, Px, Rect,"));
    assert!(root_header.contains("StrokeCapV1, StrokeJoinV1, StrokeStyle, StrokeStyleV2,"));
    assert!(root_header.contains("pub use fret_ui::canvas::CanvasKey;"));
    assert!(root_header.contains("pub use crate::view::ChartCanvas;"));
    assert!(root_header.contains("pub use delinea::engine::ChartEngine;"));
    assert!(root_header.contains("pub use delinea::engine::window::DataWindow;"));
    assert!(root_header.contains("pub use fret_chart::{ChartCanvasOutput, ChartInputMap};"));
    assert!(root_header.contains("CommandMeta, CommandRegistry, CommandScope"));
    assert!(root_header.contains("DefaultKeybinding, InputContext,"));
    assert!(root_header.contains("KeyChord, KeymapService, Platform, PlatformFilter"));
    assert!(root_header.contains("install_command_default_keybindings_into_keymap,"));
    assert!(root_header.contains("pub use fret_core::{KeyCode, Modifiers};"));
    assert!(root_header.contains("pub use fret_ui::CommandAvailability;"));
    assert!(root_header.contains("PointerActionCx, PointerCancel, PointerDown, PointerId,"));
    assert!(root_header.contains("PointerMove, PointerRegion, PointerUp, Wheel,"));
    assert!(root_header.contains("pub use fret_core::SemanticsRole;"));
    assert!(root_header.contains("pub use fret_ui::element::SemanticsDecoration;"));
    assert!(root_header.contains("pub mod time {"));
    assert!(
        root_header
            .contains("pub use fret_core::time::{Duration, Instant, SystemTime, UNIX_EPOCH};")
    );
    assert_eq!(
        root_style_exported_symbols(),
        expected_root_style_exported_symbols(),
        "root style exports should stay on the curated symbol budget"
    );
    assert!(
        root_header
            .contains("pub use fret_ui::element::{VirtualListKeyCacheMode, VirtualListOptions};")
    );
    assert!(root_header.contains("pub use fret_ui::scroll::VirtualListScrollHandle;"));
    assert!(root_header.contains("pub use fret_ui::{ItemKey, ScrollStrategy};"));
    assert!(root_header.contains("pub use fret_ui::scroll::ScrollHandle;"));
}

#[test]
fn root_surface_exposes_explicit_overlay_module() {
    let root_header = root_surface_header_source();

    assert!(root_header.contains("pub mod overlay {"));
    assert!(root_header.contains("pub use fret_ui_kit::overlay::*;"));
    assert!(root_header.contains("OverlayArbitrationSnapshot, OverlayController, OverlayKind,"));
    assert!(root_header.contains("OverlayPresence,"));
    assert!(root_header.contains("OverlayRequest, OverlayStackEntryKind,"));
    assert!(root_header.contains("WindowOverlayStackEntry,"));
    assert!(root_header.contains("WindowOverlayStackSnapshot,"));
}

#[test]
fn root_surface_exposes_explicit_imui_module() {
    let root_header = root_surface_header_source();
    let public_surface = crate_public_surface_source();

    assert!(root_header.contains("#[cfg(feature = \"imui\")]"));
    assert!(root_header.contains("pub mod imui {"));
    assert!(root_header.contains("pub use fret_imui::{"));
    assert!(root_header.contains("pub use fret_ui_kit::imui::{"));
    assert!(public_surface.contains("pub trait AppImUiLocalTextExt"));
    assert!(public_surface.contains("fn input_text_local_with_options("));
    assert!(public_surface.contains("pub mod kit {"));
    assert!(public_surface.contains("pub use fret_ui_kit::imui::*;"));
    assert!(public_surface.contains("pub mod editor {"));
    assert!(public_surface.contains("pub use fret_ui_editor::imui::*;"));
    assert!(public_surface.contains("pub use fret_ui_editor::{composites, primitives, theme};"));
    assert!(public_surface.contains("pub mod controls {"));
    assert!(public_surface.contains("pub use fret_ui_editor::controls::*;"));
    assert!(public_surface.contains("pub trait NumericInputLocalStateExt"));
    assert!(public_surface.contains("pub trait DragValueLocalStateExt"));
    assert!(public_surface.contains("pub trait ColorEditLocalStateExt"));
    assert!(public_surface.contains("pub trait MiniSearchBoxLocalStateExt"));
    assert!(public_surface.contains("pub trait TextAssistFieldLocalStateExt"));
    assert!(public_surface.contains("pub mod docking {"));
    assert!(public_surface.contains("pub use fret_docking::imui::*;"));
    assert!(public_surface.contains("pub mod prelude {"));
    assert!(public_surface.contains("pub use fret_imui::prelude::*;"));
    assert!(public_surface.contains("AppImUiLocalTextExt as _,"));
    assert!(public_surface.contains("IntoImUiBoolModel,"));
    assert!(public_surface.contains("IntoImUiFloatModel,"));
    assert!(public_surface.contains("IntoImUiOptionalTextModel,"));
    assert!(public_surface.contains("IntoImUiTextModel,"));
    assert!(public_surface.contains("docking, editor, imui, imui_build,"));
    assert!(public_surface.contains("imui_build_in, imui_in, imui_raw, imui_raw_in,"));
    assert!(public_surface.contains("kit,"));
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
    assert!(
        root_header.contains("pub use fret_runtime::set_asset_resolver as set_primary_resolver;")
    );
    assert!(
        root_header.contains("pub use fret_runtime::register_asset_resolver as register_resolver;")
    );
    assert!(root_header.contains(
        "pub use fret_runtime::register_bundle_asset_entries as register_bundle_entries;"
    ));
    assert!(root_header.contains(
        "pub use fret_runtime::register_embedded_asset_entries as register_embedded_entries;"
    ));
    assert!(root_header.contains("pub use fret_runtime::asset_capabilities as capabilities;"));
    assert!(root_header.contains("pub use fret_runtime::resolve_asset_bytes as resolve_bytes;"));
    assert!(
        root_header
            .contains("pub use fret_runtime::resolve_asset_locator_bytes as resolve_locator;")
    );
    assert!(
        root_header.contains("pub use fret_runtime::resolve_asset_reference as resolve_reference;")
    );
    assert!(root_header.contains(
        "pub use fret_runtime::resolve_asset_locator_reference as resolve_locator_reference;"
    ));
}

#[test]
fn root_surface_exposes_explicit_env_module() {
    let root_header = root_surface_header_source()
        .split_whitespace()
        .collect::<String>();

    assert!(root_header.contains("pubmodenv{"));
    assert!(
        root_header
            .contains("ContainerQueryHysteresis,ViewportOrientation,ViewportQueryHysteresis,")
    );
    assert!(root_header.contains("preferred_color_scheme,"));
    assert!(root_header.contains("prefers_dark_color_scheme,"));
    assert!(root_header.contains("safe_area_insets,"));
    assert!(root_header.contains("ViewportOrientation,"));
    assert!(root_header.contains("ViewportQueryHysteresis,"));
    assert!(root_header.contains("viewport_aspect_ratio,"));
    assert!(root_header.contains("viewport_breakpoints,viewport_height_at_least"));
    assert!(root_header.contains("viewport_tailwind,"));
    assert!(root_header.contains("window_insets_padding_refinement_or_zero,"));
}

#[test]
fn root_surface_exposes_explicit_adaptive_module() {
    let root_header = root_surface_header_source();

    assert!(root_header.contains("pub mod adaptive {"));
    assert!(root_header.contains("pub use fret_ui_kit::adaptive::{"));
    assert!(root_header.contains("AdaptiveQuerySource, DeviceAdaptiveClass,"));
    assert!(root_header.contains("DeviceAdaptivePolicy,"));
    assert!(root_header.contains("DeviceAdaptiveSnapshot,"));
    assert!(root_header.contains("DeviceShellMode, DeviceShellSwitchPolicy,"));
    assert!(root_header.contains("PanelAdaptiveClass, PanelAdaptivePolicy,"));
    assert!(root_header.contains("device_adaptive_class, device_adaptive_snapshot,"));
    assert!(root_header.contains("device_shell_mode, device_shell_switch,"));
    assert!(root_header.contains("panel_adaptive_class,"));
}

#[test]
fn app_and_advanced_modules_expose_view_runtime_on_explicit_lanes_only() {
    let root_header = root_surface_header_source();
    let advanced_surface = advanced_prelude_source();

    assert!(app_contains("pub use crate::view::View;"));
    assert!(!root_header.contains("pub mod view;"));
    assert!(advanced_surface.contains("pub mod view {"));
    assert!(advanced_surface.contains("AppUiRenderRootState"));
    assert!(advanced_surface.contains("AppRenderDataExt"));
    assert!(advanced_surface.contains("render_root_with_app_ui"));
    assert!(advanced_surface.contains("ViewWindowState,"));
    assert!(advanced_surface.contains("view_init_window,"));
    assert!(advanced_surface.contains("view_view"));
    assert!(advanced_surface.contains("view_record_engine_frame"));
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
    assert!(!public_surface.contains("pub use crate::view::LocalSelectorDepsBuilderExt;"));
    assert!(public_surface.contains("pub use fret_selector::{DepsSignature, Selector};"));
    assert!(public_surface.contains("pub use fret_selector::ui::DepsBuilder;"));
    assert!(!public_surface.contains("pub use fret_selector::ui::*;"));
    assert!(public_surface.contains("pub use fret_query::{"));
    assert!(public_surface.contains("CancellationToken, FutureSpawner, FutureSpawnerHandle"));
    assert!(
        public_surface.contains("QueryError, QueryErrorKind, QueryHandle, QueryKey, QueryPolicy")
    );
    assert!(public_surface.contains("QueryRetryOn, QueryRetryPolicy, QueryRetryState"));
    assert!(public_surface.contains("QuerySnapshotEntry, QueryState,"));
    assert!(public_surface.contains("QueryStatus, with_query_client,"));
    assert!(!public_surface.contains("pub use fret_query::ui::*;"));
    assert!(!public_surface.contains("pub use fret_router_ui::*;"));
}

#[test]
fn crate_feature_surface_omits_compat_icon_aliases() {
    assert!(CARGO_TOML.contains("icons = ["));
    assert!(!CARGO_TOML.contains("icons-lucide = [\"icons\"]"));
}

#[test]
fn ui_assets_feature_stays_cross_platform_and_off_desktop_runner() {
    let manifest = CARGO_TOML.split_whitespace().collect::<String>();

    assert!(manifest.contains("ui-assets=[\"dep:fret-bootstrap\",\"dep:fret-ui-assets\","));
    assert!(manifest.contains("\"fret-ui-assets/app-integration\","));
    assert!(manifest.contains("\"fret-ui-assets/ui\","));
    assert!(manifest.contains("\"fret-bootstrap/ui-assets\",]"));
    assert!(!manifest.contains("ui-assets=[\"desktop\","));
    assert!(!manifest.contains("ui-assets=[\"desktop\""));
    assert!(CRATE_README.contains(
        "- `ui-assets`: cross-platform UI render-asset caches (images/SVG) and default budgets."
    ));
    assert!(!CRATE_README.contains("- `ui-assets`: desktop-bound UI render-asset caches"));
}

#[test]
fn view_runtime_exposes_only_app_ui_as_the_public_context_name() {
    assert!(!VIEW_RS.contains("pub type ViewCx"));
    assert!(
        VIEW_CONTEXT_RS
            .contains("fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui;")
    );
    assert!(VIEW_RS.contains(") -> crate::Ui {"));
}

#[test]
fn app_prelude_omits_low_level_mechanism_types() {
    assert!(!app_prelude_exports_symbol("AppWindowId"));
    assert!(!app_prelude_exports_symbol("AppUiRawActionNotifyExt"));
    assert!(!app_prelude_exports_symbol("AppUiRawStateExt"));
    assert!(!app_prelude_exports_symbol("AppUiRawModelExt"));
    assert!(!app_prelude_exports_symbol("LocalStateRawModelExt"));
    assert!(!app_prelude_exports_symbol("LocalStateModelStoreExt"));
    assert!(!app_prelude_exports_symbol("LocalStateElementContextExt"));
    assert!(!app_prelude_exports_symbol("Event"));
    assert!(!app_prelude_exports_symbol("ElementContext"));
    assert!(!app_prelude_exports_symbol("UiTree"));
    assert!(!app_prelude_exports_symbol("UiServices"));
    assert!(!app_prelude_exports_symbol("UiHost"));
    assert!(!app_prelude_exports_symbol("AnyElement"));
    assert!(!app_prelude_exports_symbol("CommandAvailability"));
    assert!(!app_prelude_exports_symbol("ActionId"));
    assert!(!app_prelude_exports_symbol("CommandMeta"));
    assert!(!app_prelude_exports_symbol("CommandRegistry"));
    assert!(!app_prelude_exports_symbol("CommandScope"));
    assert!(!app_prelude_exports_symbol("DefaultKeybinding"));
    assert!(!app_prelude_exports_symbol("InputContext"));
    assert!(!app_prelude_exports_symbol("KeyChord"));
    assert!(!app_prelude_exports_symbol("KeyCode"));
    assert!(!app_prelude_exports_symbol("KeymapService"));
    assert!(!app_prelude_exports_symbol("Modifiers"));
    assert!(!app_prelude_exports_symbol("TypedAction"));
    assert!(!app_prelude_exports_symbol("RouterUiStore"));
    assert!(!app_prelude_exports_symbol("RouterOutlet"));
    assert!(!app_prelude_exports_symbol("AppElement"));
    assert!(!app_prelude_exports_symbol("UiBuilder"));
    assert!(!app_prelude_exports_symbol("UiPatchTarget"));
    assert!(!app_prelude_exports_symbol("HoverRegionProps"));
    assert!(!app_prelude_exports_symbol("PointerRegion"));
    assert!(!app_prelude_exports_symbol("PointerActionCx"));
    assert!(!app_prelude_exports_symbol("PointerDown"));
    assert!(!app_prelude_exports_symbol("PointerMove"));
    assert!(!app_prelude_exports_symbol("PointerUp"));
    assert!(!app_prelude_exports_symbol("PointerId"));
    assert!(!app_prelude_exports_symbol("PressableA11y"));
    assert!(!app_prelude_exports_symbol("PressableProps"));
    assert!(!app_prelude_exports_symbol("PressableState"));
    assert!(!app_prelude_exports_symbol("MouseButton"));
    assert!(!app_prelude_exports_symbol("CursorIcon"));
    assert!(!app_prelude_exports_symbol("AppAsyncWorkExt"));
    assert!(!app_prelude_exports_symbol("AppInboxCx"));
    assert!(!app_prelude_exports_symbol("InboxLocal"));
    assert!(!app_prelude_exports_symbol("DispatcherHandle"));
    assert!(!app_prelude_exports_symbol("DispatchPriority"));
    assert!(!app_prelude_exports_symbol("AppCanvasPainter"));
    assert!(!app_prelude_exports_symbol("Canvas"));
    assert!(!app_prelude_exports_symbol("CanvasSurface"));
    assert!(!app_prelude_exports_symbol("PanZoomCanvas"));
    assert!(!app_prelude_exports_symbol("PanZoom2D"));
    assert!(!app_prelude_exports_symbol("CanvasPaint"));
    assert!(!app_prelude_exports_symbol("CanvasKey"));
    assert!(!app_prelude_exports_symbol("PathCommand"));
    assert!(!app_prelude_exports_symbol("PathStyle"));
    assert!(!app_prelude_exports_symbol("ChartCanvas"));
    assert!(!app_prelude_exports_symbol("ChartEngine"));
    assert!(!app_prelude_exports_symbol("ChartCanvasOutput"));
    assert!(!app_prelude_exports_symbol("Length"));
    assert!(!app_prelude_exports_symbol("LayoutStyle"));
    assert!(!app_prelude_exports_symbol("ContainerProps"));
    assert!(!app_prelude_exports_symbol("Edges"));
    assert!(!app_prelude_exports_symbol("SemanticsProps"));
    assert!(!app_prelude_exports_symbol("UiElementSinkExt"));
    assert!(!app_prelude_exports_symbol("VirtualListOptions"));
    assert!(!app_prelude_exports_symbol("ScrollHandle"));
    assert!(!app_prelude_exports_symbol("VirtualListScrollHandle"));
    assert!(!app_prelude_exports_symbol("VirtualListKeyCacheMode"));
    assert!(!app_prelude_exports_symbol("ScrollStrategy"));
    assert!(!app_prelude_exports_symbol("ItemKey"));
    assert!(!app_prelude_exports_symbol("AdaptiveQuerySource"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptiveClass"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptivePolicy"));
    assert!(!app_prelude_exports_symbol("DeviceAdaptiveSnapshot"));
    assert!(!app_prelude_exports_symbol("DeviceShellMode"));
    assert!(!app_prelude_exports_symbol("DeviceShellSwitchPolicy"));
    assert!(!app_prelude_exports_symbol("PanelAdaptiveClass"));
    assert!(!app_prelude_exports_symbol("PanelAdaptivePolicy"));
    assert!(!app_prelude_exports_symbol("device_adaptive_class"));
    assert!(!app_prelude_exports_symbol("device_adaptive_snapshot"));
    assert!(!app_prelude_exports_symbol("device_shell_mode"));
    assert!(!app_prelude_exports_symbol("device_shell_switch"));
    assert!(!app_prelude_exports_symbol("panel_adaptive_class"));
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
    assert!(!app_prelude_exports_symbol("LocalSelectorDepsBuilderExt"));
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
        component_prelude.contains("pub use fret_ui_kit::declarative::UiElementTestIdExt as _;")
    );
    assert!(component_prelude.contains("pub use fret_ui_kit::declarative::TrackedModelExt as _;"));
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
    assert!(!component_prelude_exports_symbol("AdaptiveQuerySource"));
    assert!(!component_prelude_exports_symbol("DeviceAdaptiveClass"));
    assert!(!component_prelude_exports_symbol("DeviceAdaptivePolicy"));
    assert!(!component_prelude_exports_symbol("DeviceAdaptiveSnapshot"));
    assert!(!component_prelude_exports_symbol("DeviceShellMode"));
    assert!(!component_prelude_exports_symbol("DeviceShellSwitchPolicy"));
    assert!(!component_prelude_exports_symbol("PanelAdaptiveClass"));
    assert!(!component_prelude_exports_symbol("PanelAdaptivePolicy"));
    assert!(!component_prelude_exports_symbol("device_adaptive_class"));
    assert!(!component_prelude_exports_symbol(
        "device_adaptive_snapshot"
    ));
    assert!(!component_prelude_exports_symbol("device_shell_mode"));
    assert!(!component_prelude_exports_symbol("device_shell_switch"));
    assert!(!component_prelude_exports_symbol("panel_adaptive_class"));
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

    assert_eq!(
        overlap,
        vec![
            "Invalidation".to_string(),
            "Px".to_string(),
            "ui".to_string()
        ]
    );
}

#[test]
fn component_prelude_omits_app_runtime_and_recipe_specific_surfaces() {
    assert!(!component_prelude_exports_symbol("FretApp"));
    assert!(!component_prelude_exports_symbol("App"));
    assert!(!component_prelude_exports_symbol("AppComponentCx"));
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
    let public_surface = crate_public_surface_source();
    assert!(!public_surface.contains("pub mod prelude {\n    pub use fret_ui_kit::prelude::*;"));
}

#[test]
fn root_builder_aliases_are_deleted() {
    let lines = ROOT_RS.lines().map(str::trim).collect::<Vec<_>>();
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
    assert!(APP_ENTRY_RS.contains("pub fn view<") || APP_ENTRY_RS.contains("pub fn view("));
    assert!(
        APP_ENTRY_RS.contains("pub fn view_with_hooks<")
            || APP_ENTRY_RS.contains("pub fn view_with_hooks(")
    );
    assert!(!APP_ENTRY_RS.contains("pub fn install_app("));
    assert!(!APP_ENTRY_RS.contains("pub fn install("));
    assert!(!APP_ENTRY_RS.contains("pub fn asset_manifest("));
    assert!(!APP_ENTRY_RS.contains("pub fn asset_manifest<"));
    assert!(!APP_ENTRY_RS.contains("pub fn asset_dir("));
    assert!(!APP_ENTRY_RS.contains("pub fn asset_dir<"));
    assert!(!APP_ENTRY_RS.contains("pub fn register_icon_pack("));
    assert!(!APP_ENTRY_RS.contains("pub fn run_view("));
    assert!(!APP_ENTRY_RS.contains("pub fn run_view_with_hooks("));

    let ui_app_builder = ui_app_builder_impl_source();
    assert!(ui_app_builder.contains("pub fn setup_with("));
    assert!(ui_app_builder.contains("pub fn setup<") || ui_app_builder.contains("pub fn setup("));
    assert!(ui_app_builder.contains("pub fn with_asset_startup("));
    assert!(!ui_app_builder.contains("pub fn init_app("));
    assert!(!ui_app_builder.contains("pub fn install("));
    assert!(!ui_app_builder.contains("pub fn with_asset_dir("));
    assert!(!ui_app_builder.contains("pub fn with_asset_manifest("));
    assert!(!ui_app_builder.contains("pub fn register_icon_pack("));
    assert!(!ui_app_builder.contains("pub fn with_lucide_icons("));
    assert!(!ui_app_builder.contains("pub fn install_custom_effects("));
    assert!(!ui_app_builder.contains("pub fn on_gpu_ready("));

    assert!(advanced_driver_contains("pub trait FretAppAdvancedExt"));
    assert!(advanced_driver_contains(
        "pub trait UiAppBuilderAdvancedExt"
    ));
}

#[test]
fn app_entry_builder_name_is_fret_app_only() {
    assert!(APP_ENTRY_RS.contains("pub struct FretApp"));
    assert!(APP_ENTRY_RS.contains("AssetBundleId::app(self.root_name)"));
    assert!(!APP_ENTRY_RS.contains("pub struct App"));
}

#[test]
fn workspace_app_public_state_hooks_do_not_smuggle_retained_runtime_seams() {
    let state_trait = module_block_source(WORKSPACE_RS, "pub trait WorkspaceWindowState");
    assert!(state_trait.contains("fn workspace_workbench(&self) -> &WorkspaceWorkbench"));
    assert!(state_trait.contains("fn handle_workspace_command("));
    assert!(state_trait.contains("fn handle_workspace_event("));
    assert!(state_trait.contains("fn handle_workspace_global_changes("));
    for forbidden in [
        "UiTree",
        "UiServices",
        "FnDriver",
        "RenderRootContext",
        "UiFrameCx",
    ] {
        assert!(
            !state_trait.contains(forbidden),
            "WorkspaceWindowState must not expose {forbidden}"
        );
    }

    assert!(WORKSPACE_RS.contains("pub fn ui<S: WorkspaceWindowState + 'static>("));
    assert!(
        WORKSPACE_RS
            .contains(".on_app_command_before_ui(handle_workbench_command_from_context::<S>)")
    );
    assert!(!WORKSPACE_RS.contains("pub fn ui_with_hooks<S:"));
}
