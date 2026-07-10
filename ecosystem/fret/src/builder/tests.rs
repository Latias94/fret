use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::FretApp;
use crate::advanced::{
    FretAppAdvancedExt as _, KernelApp, UiAppBuilderAdvancedExt as _, ViewElements,
};
use crate::app::App;
use crate::app::prelude::FretApp as AppPreludeFretApp;
use crate::view::View;
use crate::{AppUi, Defaults, Error, Ui, WindowId};
use fret_app::CreateWindowRequest;
use fret_assets::{AssetBundleId, AssetRevision, FileAssetManifestResolver, StaticAssetEntry};
use fret_core::{AppWindowId, DockOp, Event, KeyCode, Modifiers, UiServices, ViewportInputEvent};
use fret_runtime::{
    CommandId, CommandMeta, DefaultKeybinding, FrameId, InputContext, KeyChord, KeymapService,
    Platform, PlatformCapabilities, PlatformFilter, TickId,
};
#[cfg(feature = "shadcn")]
use fret_ui::Theme;

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

#[cfg(feature = "shadcn")]
fn install_dark_shadcn_theme(app: &mut App) {
    crate::shadcn::themes::apply_shadcn_new_york(
        app,
        crate::shadcn::themes::ShadcnBaseColor::Slate,
        crate::shadcn::themes::ShadcnColorScheme::Dark,
    );
}

#[cfg(feature = "shadcn")]
fn dark_shadcn_background() -> fret_core::Color {
    let mut app = App::new();
    install_dark_shadcn_theme(&mut app);
    Theme::global(&app).color_token("background")
}

fn install_test_command_with_default_keybinding(app: &mut App) {
    app.commands_mut().register(
        CommandId::from("tests.fret_app_setup_order.command"),
        CommandMeta::new("Setup order test command").with_default_keybindings([
            DefaultKeybinding::single(
                PlatformFilter::All,
                KeyChord::new(
                    KeyCode::KeyK,
                    Modifiers {
                        ctrl: true,
                        alt: true,
                        ..Modifiers::default()
                    },
                ),
            ),
        ]),
    );
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
    _context: &fret_framework::render::WgpuContext,
    _renderer: &mut fret_framework::render::Renderer,
    _dt_s: f32,
    _tick_id: TickId,
    _frame_id: FrameId,
) -> fret_launch::EngineFrameUpdate {
    fret_launch::EngineFrameUpdate::default()
}

fn install_custom_effects(_app: &mut KernelApp, _service: &mut dyn fret_core::CustomEffectService) {
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
        .window_min_size((420.0, 320.0))
        .window_max_size((900.0, 700.0))
        .window_resize_increments((24.0, 16.0))
        .window_position_logical((120, 180))
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
            assert_eq!(
                config.main_window_min_size,
                Some(fret_launch::WindowLogicalSize::new(420.0, 320.0))
            );
            assert_eq!(
                config.main_window_max_size,
                Some(fret_launch::WindowLogicalSize::new(900.0, 700.0))
            );
            assert_eq!(
                config.main_window_resize_increments,
                Some(fret_launch::WindowLogicalSize::new(24.0, 16.0))
            );
            assert_eq!(
                config.main_window_position,
                Some(fret_launch::WindowPosition::Logical(
                    fret_core::WindowLogicalPosition { x: 120, y: 180 }
                ))
            );
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
fn app_builder_default_main_window_can_still_apply_constraints() {
    let _builder = AppPreludeFretApp::new("builder-view-constrained-default-main-window")
        .minimal_defaults()
        .window_min_size((420.0, 560.0))
        .window_resize_increments((32.0, 24.0))
        .window_position_physical((40, 80))
        .window_resizable(false)
        .view::<SmokeView>()
        .expect("view should build")
        .configure(|config| {
            assert_eq!(
                config.main_window_title,
                "builder-view-constrained-default-main-window"
            );
            assert_eq!(config.main_window_size.width, 960.0);
            assert_eq!(config.main_window_size.height, 720.0);
            assert_eq!(
                config.main_window_min_size,
                Some(fret_launch::WindowLogicalSize::new(420.0, 560.0))
            );
            assert_eq!(
                config.main_window_resize_increments,
                Some(fret_launch::WindowLogicalSize::new(32.0, 24.0))
            );
            assert_eq!(
                config.main_window_position,
                Some(fret_launch::WindowPosition::Physical(
                    fret_launch::WindowPhysicalPosition::new(40, 80)
                ))
            );
            assert_eq!(config.main_window_style.resizable, Some(false));
        });
}

#[test]
fn ui_app_builder_can_configure_default_aux_window_surface() {
    let _builder = AppPreludeFretApp::new("builder-view-default-aux-window")
        .minimal_defaults()
        .view::<SmokeView>()
        .expect("view should build")
        .with_default_window("Aux Window", (460.0, 340.0))
        .with_default_window_min_size((320.0, 240.0))
        .with_default_window_max_size((900.0, 700.0))
        .with_default_window_resize_increments((18.0, 18.0))
        .with_default_window_position_logical((90, 120))
        .configure(|config| {
            assert_eq!(config.default_window_title, "Aux Window");
            assert_eq!(config.default_window_size.width, 460.0);
            assert_eq!(config.default_window_size.height, 340.0);
            assert_eq!(
                config.default_window_min_size,
                Some(fret_launch::WindowLogicalSize::new(320.0, 240.0))
            );
            assert_eq!(
                config.default_window_max_size,
                Some(fret_launch::WindowLogicalSize::new(900.0, 700.0))
            );
            assert_eq!(
                config.default_window_resize_increments,
                Some(fret_launch::WindowLogicalSize::new(18.0, 18.0))
            );
            assert_eq!(
                config.default_window_position,
                Some(fret_launch::WindowPosition::Logical(
                    fret_core::WindowLogicalPosition { x: 90, y: 120 }
                ))
            );
        });
}

#[test]
fn file_manifest_resolver_from_bundle_dir_installs_on_host_path() {
    let asset_dir = write_asset_dir_fixture("fret-register-file-bundle-dir");
    let bundle = AssetBundleId::app("builder-register-file-bundle-dir");
    let mut app = App::new();
    let resolver = FileAssetManifestResolver::from_bundle_dir(bundle.clone(), &asset_dir)
        .expect("bundle dir resolver should build");

    crate::assets::register_resolver(&mut app, Arc::new(resolver));

    let resolved = crate::assets::resolve_locator(
        &app,
        crate::assets::AssetLocator::bundle(bundle, "images/logo.png"),
    )
    .expect("registered bundle dir asset should resolve");

    assert_eq!(resolved.bytes.as_ref(), b"builder-dir");
}

#[test]
fn file_manifest_resolver_from_bundle_dir_exposes_external_file_reference_on_host_path() {
    let asset_dir = write_asset_dir_fixture("fret-register-file-bundle-dir-reference");
    let bundle = AssetBundleId::app("builder-register-file-bundle-dir-reference");
    let mut app = App::new();
    let resolver = FileAssetManifestResolver::from_bundle_dir(bundle.clone(), &asset_dir)
        .expect("bundle dir resolver should build");

    crate::assets::register_resolver(&mut app, Arc::new(resolver));

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
fn fret_app_asset_startup_installs_selected_development_manifest_lane_on_builder_path() {
    let manifest_path = write_asset_manifest_fixture();

    let _builder = FretApp::new("builder-view-asset-startup-manifest")
        .asset_startup(
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new()
                .development_manifest(&manifest_path)
                .packaged_entries([StaticAssetEntry::new(
                    "images/logo.png",
                    AssetRevision(1),
                    b"builder-bytes",
                )]),
        )
        .view::<SmokeView>()
        .expect("development manifest startup plan should load on fret app builder path");
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
    let asset_dir = write_asset_dir_fixture("asset-startup-plan-development-bundle-dir-if-native");
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
fn asset_startup_builder_methods_fail_early_for_missing_development_manifests() {
    let missing = std::env::temp_dir().join("definitely-missing-fret-assets.manifest.json");

    let fret_app_err = match FretApp::new("builder-view-missing-asset-startup-manifest")
        .asset_startup(
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new().development_manifest(&missing),
        )
        .view::<SmokeView>()
    {
        Ok(_) => panic!("missing development manifest should fail on fret app builder path"),
        Err(err) => err,
    };
    assert!(matches!(fret_app_err, Error::AssetManifest(_)));

    let ui_builder_err =
        match FretApp::new("builder-view-missing-asset-startup-manifest-ui-builder")
            .view::<SmokeView>()
            .expect("view should build")
            .with_asset_startup(
                AssetBundleId::app("builder-view-missing-asset-startup-manifest-ui-builder"),
                crate::assets::AssetStartupMode::Development,
                crate::assets::AssetStartupPlan::new().development_manifest(&missing),
            ) {
            Ok(_) => panic!("missing development manifest should fail on ui app builder path"),
            Err(err) => err,
        };
    assert!(matches!(ui_builder_err, Error::AssetManifest(_)));
}

#[test]
fn asset_startup_builder_methods_fail_early_for_missing_development_directories() {
    let missing = std::env::temp_dir().join("definitely-missing-fret-assets-dir");

    let fret_app_err = match FretApp::new("builder-view-missing-asset-startup-dir")
        .asset_startup(
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new().development_dir(&missing),
        )
        .view::<SmokeView>()
    {
        Ok(_) => panic!("missing development dir should fail on fret app builder path"),
        Err(err) => err,
    };
    assert!(matches!(fret_app_err, Error::AssetManifest(_)));

    let ui_builder_err = match FretApp::new("builder-view-missing-asset-startup-dir-ui-builder")
        .view::<SmokeView>()
        .expect("view should build")
        .with_asset_startup(
            AssetBundleId::app("builder-view-missing-asset-startup-dir-ui-builder"),
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new().development_dir(&missing),
        ) {
        Ok(_) => panic!("missing development dir should fail on ui app builder path"),
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
fn fret_error_known_bootstrap_failure_report_maps_asset_failures() {
    let missing_manifest =
        std::env::temp_dir().join("definitely-missing-known-bootstrap-assets.manifest.json");
    let manifest_error = match FretApp::new("builder-view-known-bootstrap-manifest")
        .asset_startup(
            crate::assets::AssetStartupMode::Development,
            crate::assets::AssetStartupPlan::new().development_manifest(&missing_manifest),
        )
        .view::<SmokeView>()
    {
        Ok(_) => panic!("missing development manifest should fail on fret app builder path"),
        Err(err) => err,
    };
    let manifest_report = manifest_error
        .known_bootstrap_failure_report()
        .expect("asset manifest failure should map to a known bootstrap report");
    assert_eq!(
        manifest_report.stage,
        fret_bootstrap::BootstrapKnownFailureStage::Builder
    );
    assert_eq!(
        manifest_report.kind,
        fret_bootstrap::BootstrapKnownFailureKind::AssetManifestRead
    );
    assert_eq!(manifest_report.surface, Some("asset_manifest"));
    assert!(
        manifest_report
            .summary
            .contains("failed to read asset manifest")
    );
    assert_eq!(manifest_report.details.len(), 1);

    let startup_error = match FretApp::new("builder-view-known-bootstrap-missing-lane")
        .asset_startup(
            crate::assets::AssetStartupMode::Packaged,
            crate::assets::AssetStartupPlan::new().development_dir("assets"),
        )
        .view::<SmokeView>()
    {
        Ok(_) => panic!("missing packaged lane should fail on fret app builder path"),
        Err(err) => err,
    };
    let startup_report = startup_error
        .known_bootstrap_failure_report()
        .expect("asset startup failure should map to a known bootstrap report");
    assert_eq!(
        startup_report.stage,
        fret_bootstrap::BootstrapKnownFailureStage::Builder
    );
    assert_eq!(
        startup_report.kind,
        fret_bootstrap::BootstrapKnownFailureKind::AssetStartupMissingPackagedLane
    );
    assert_eq!(startup_report.surface, Some("asset_startup"));
    assert_eq!(
        startup_report.summary,
        "asset startup plan is missing a packaged lane"
    );
    assert!(startup_report.details.is_empty());
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
#[cfg(feature = "shadcn")]
fn fret_app_setup_theme_runs_after_base_shadcn_defaults() {
    let expected_background = dark_shadcn_background();
    let _builder = FretApp::new("builder-view-setup-theme-order")
        .setup(install_dark_shadcn_theme)
        .view::<SmokeView>()
        .expect("view should build")
        .setup_with(|app| {
            assert_eq!(
                Theme::global(app).color_token("background"),
                expected_background
            );
        });
}

#[test]
fn fret_app_runtime_defaults_still_observe_setup_registered_commands() {
    let _builder = FretApp::new("builder-view-setup-command-order")
        .setup(install_test_command_with_default_keybinding)
        .view::<SmokeView>()
        .expect("view should build")
        .setup_with(|app| {
            let ctx = InputContext::fallback(Platform::Windows, PlatformCapabilities::default());
            let chord = KeyChord::new(
                KeyCode::KeyK,
                Modifiers {
                    ctrl: true,
                    alt: true,
                    ..Modifiers::default()
                },
            );
            let command = app
                .global::<KeymapService>()
                .and_then(|svc| svc.keymap.resolve(&ctx, chord));

            assert_eq!(
                command.as_ref().map(CommandId::as_str),
                Some("tests.fret_app_setup_order.command")
            );
        });
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
    .with_main_window_min_size((520.0, 360.0))
    .with_main_window_resize_increments((20.0, 20.0))
    .with_main_window_position_physical((300, 220))
    .with_main_window_resizable(false)
    .with_default_window("Advanced Aux Window", (480.0, 320.0))
    .with_default_window_min_size((360.0, 240.0))
    .with_default_window_resize_increments((12.0, 12.0))
    .with_default_window_position_physical((44, 55))
    .setup(install_bundle_fixture)
    .install(install)
    .configure(|config| {
        assert_eq!(config.main_window_title, "Advanced UI App Hooks Smoke");
        assert_eq!(config.main_window_size.width, 720.0);
        assert_eq!(config.main_window_size.height, 420.0);
        assert_eq!(
            config.main_window_min_size,
            Some(fret_launch::WindowLogicalSize::new(520.0, 360.0))
        );
        assert_eq!(
            config.main_window_resize_increments,
            Some(fret_launch::WindowLogicalSize::new(20.0, 20.0))
        );
        assert_eq!(
            config.main_window_position,
            Some(fret_launch::WindowPosition::Physical(
                fret_launch::WindowPhysicalPosition::new(300, 220)
            ))
        );
        assert_eq!(config.main_window_style.resizable, Some(false));
        assert_eq!(config.default_window_title, "Advanced Aux Window");
        assert_eq!(config.default_window_size.width, 480.0);
        assert_eq!(config.default_window_size.height, 320.0);
        assert_eq!(
            config.default_window_min_size,
            Some(fret_launch::WindowLogicalSize::new(360.0, 240.0))
        );
        assert_eq!(
            config.default_window_resize_increments,
            Some(fret_launch::WindowLogicalSize::new(12.0, 12.0))
        );
        assert_eq!(
            config.default_window_position,
            Some(fret_launch::WindowPosition::Physical(
                fret_launch::WindowPhysicalPosition::new(44, 55)
            ))
        );
    });
}
