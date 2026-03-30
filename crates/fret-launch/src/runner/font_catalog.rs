use fret_core::TextFontFamilyConfig;
use fret_runtime::fret_i18n::I18nService;
use fret_runtime::{FontCatalogEntry, FontCatalogUpdate, FontFamilyDefaultsPolicy, GlobalsHost};

#[doc(hidden)]
pub trait RendererFontEnvironmentHost {
    fn all_font_catalog_entries_runtime(&mut self) -> Vec<FontCatalogEntry>;
    fn set_text_font_families(&mut self, config: &TextFontFamilyConfig) -> bool;
    fn set_text_locale(&mut self, locale: Option<&str>) -> bool;
    fn text_font_stack_key(&self) -> u64;
}

#[doc(hidden)]
pub trait BundledFontBaselineHost {
    fn add_font_blobs<I>(&mut self, fonts: I) -> usize
    where
        I: IntoIterator<Item = Vec<u8>>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct DefaultBundledFontAssetsRegistered(bool);

impl RendererFontEnvironmentHost for fret_render::Renderer {
    fn all_font_catalog_entries_runtime(&mut self) -> Vec<FontCatalogEntry> {
        self.all_font_catalog_entries()
            .into_iter()
            .map(|entry| {
                let (
                    family,
                    has_variable_axes,
                    known_variable_axes,
                    variable_axes,
                    is_monospace_candidate,
                ) = entry.into_parts();
                FontCatalogEntry {
                    family,
                    has_variable_axes,
                    known_variable_axes,
                    variable_axes: variable_axes
                        .into_iter()
                        .map(|axis| {
                            let (tag, min_bits, max_bits, default_bits) = axis.into_parts();
                            fret_runtime::FontVariableAxisInfo {
                                tag,
                                min_bits,
                                max_bits,
                                default_bits,
                            }
                        })
                        .collect(),
                    is_monospace_candidate,
                }
            })
            .collect()
    }

    fn set_text_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        self.set_text_font_families(config)
    }

    fn set_text_locale(&mut self, locale: Option<&str>) -> bool {
        self.set_text_locale(locale)
    }

    fn text_font_stack_key(&self) -> u64 {
        self.text_font_stack_key()
    }
}

impl BundledFontBaselineHost for fret_render::Renderer {
    fn add_font_blobs<I>(&mut self, fonts: I) -> usize
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        self.add_fonts(fonts)
    }
}

fn preferred_text_locale(app: &impl GlobalsHost) -> Option<String> {
    app.global::<I18nService>()
        .and_then(|service| service.preferred_locales().first())
        .map(|locale| locale.to_string())
}

fn bundled_font_role_name(role: fret_fonts::BundledFontRole) -> &'static str {
    match role {
        fret_fonts::BundledFontRole::UiSans => "UiSans",
        fret_fonts::BundledFontRole::UiSerif => "UiSerif",
        fret_fonts::BundledFontRole::UiMonospace => "UiMonospace",
        fret_fonts::BundledFontRole::EmojiFallback => "EmojiFallback",
        fret_fonts::BundledFontRole::CjkFallback => "CjkFallback",
    }
}

fn bundled_generic_family_name(family: fret_fonts::BundledGenericFamily) -> &'static str {
    match family {
        fret_fonts::BundledGenericFamily::Sans => "Sans",
        fret_fonts::BundledGenericFamily::Serif => "Serif",
        fret_fonts::BundledGenericFamily::Monospace => "Monospace",
    }
}

#[doc(hidden)]
pub fn default_bundled_font_baseline_snapshot() -> fret_runtime::BundledFontBaselineSnapshot {
    let profile = fret_fonts::default_profile();
    fret_runtime::BundledFontBaselineSnapshot::bundled_profile(
        profile.name,
        fret_fonts::bundled_asset_bundle().as_str(),
        profile
            .faces
            .iter()
            .map(|face| face.asset_key.to_string())
            .collect(),
        profile
            .provided_roles
            .iter()
            .map(|role| bundled_font_role_name(*role).to_string())
            .collect(),
        profile
            .guaranteed_generic_families
            .iter()
            .map(|family| bundled_generic_family_name(*family).to_string())
            .collect(),
    )
}

fn ensure_default_bundled_font_assets_registered(app: &mut impl GlobalsHost) -> bool {
    app.with_global_mut(
        DefaultBundledFontAssetsRegistered::default,
        |registered, app| {
            if registered.0 {
                return false;
            }
            fret_runtime::register_bundle_asset_entries(
                app,
                fret_fonts::bundled_asset_bundle(),
                fret_fonts::default_profile().asset_entries(),
            );
            registered.0 = true;
            true
        },
    )
}

fn bundled_profile_font_blobs_from_runtime_assets(
    app: &impl GlobalsHost,
    profile: &fret_fonts::BundledFontProfile,
) -> Vec<Vec<u8>> {
    profile
        .faces
        .iter()
        .map(|face| {
            fret_runtime::resolve_asset_bytes(app, &face.asset_request())
                .unwrap_or_else(|err| {
                    panic!(
                        "bundled startup font asset '{}' failed to resolve after registration: {:?}",
                        face.asset_key, err
                    )
                })
                .bytes
                .as_ref()
                .to_vec()
        })
        .collect()
}

#[doc(hidden)]
pub fn install_default_bundled_font_baseline(
    app: &mut impl GlobalsHost,
    renderer: &mut impl BundledFontBaselineHost,
) -> usize {
    let profile = fret_fonts::default_profile();
    let _ = ensure_default_bundled_font_assets_registered(app);
    let font_blobs = bundled_profile_font_blobs_from_runtime_assets(app, profile);
    let added = renderer.add_font_blobs(font_blobs);
    let _ = publish_bundled_font_baseline_snapshot(app, default_bundled_font_baseline_snapshot());
    added
}

#[doc(hidden)]
#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
pub fn initialize_web_startup_font_environment(
    app: &mut impl GlobalsHost,
    renderer: &mut (impl RendererFontEnvironmentHost + BundledFontBaselineHost),
    config: TextFontFamilyConfig,
) -> FontCatalogUpdate {
    let _ = install_default_bundled_font_baseline(app, renderer);
    initialize_startup_font_environment(
        app,
        renderer,
        config,
        StartupFontEnvironmentMode::WebBundledSync,
    )
}

#[doc(hidden)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn initialize_desktop_startup_font_environment(
    app: &mut impl GlobalsHost,
    renderer: &mut (impl RendererFontEnvironmentHost + BundledFontBaselineHost),
    config: TextFontFamilyConfig,
    startup_async: bool,
) -> FontCatalogUpdate {
    let _ = install_default_bundled_font_baseline(app, renderer);
    initialize_startup_font_environment(
        app,
        renderer,
        config,
        if startup_async {
            StartupFontEnvironmentMode::DesktopAsync
        } else {
            StartupFontEnvironmentMode::DesktopSync
        },
    )
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn startup_font_defaults_policy(mode: StartupFontEnvironmentMode) -> FontFamilyDefaultsPolicy {
    match mode {
        StartupFontEnvironmentMode::DesktopSync | StartupFontEnvironmentMode::DesktopAsync => {
            FontFamilyDefaultsPolicy::None
        }
        StartupFontEnvironmentMode::WebBundledSync => {
            FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub enum StartupFontEnvironmentMode {
    DesktopSync,
    DesktopAsync,
    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    WebBundledSync,
}

#[doc(hidden)]
pub fn publish_renderer_text_stack_key_if_changed(
    app: &mut impl GlobalsHost,
    renderer: &impl RendererFontEnvironmentHost,
) -> bool {
    let new_key = renderer.text_font_stack_key();
    let old_key = app
        .global::<fret_runtime::TextFontStackKey>()
        .map(|key| key.0);
    if old_key != Some(new_key) {
        app.set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(new_key));
        true
    } else {
        false
    }
}

#[doc(hidden)]
pub fn publish_system_font_rescan_state(
    app: &mut impl GlobalsHost,
    in_flight: bool,
    pending: bool,
) -> bool {
    let new_state = fret_runtime::SystemFontRescanState { in_flight, pending };
    let old_state = app.global::<fret_runtime::SystemFontRescanState>().copied();
    if old_state != Some(new_state) {
        app.set_global::<fret_runtime::SystemFontRescanState>(new_state);
        true
    } else {
        false
    }
}

#[doc(hidden)]
pub fn publish_bundled_font_baseline_snapshot(
    app: &mut impl GlobalsHost,
    snapshot: fret_runtime::BundledFontBaselineSnapshot,
) -> bool {
    let old = app
        .global::<fret_runtime::BundledFontBaselineSnapshot>()
        .cloned();
    if old.as_ref() != Some(&snapshot) {
        app.set_global::<fret_runtime::BundledFontBaselineSnapshot>(snapshot);
        true
    } else {
        false
    }
}

#[doc(hidden)]
pub fn sync_renderer_font_families_from_globals(
    app: &mut impl GlobalsHost,
    renderer: &mut impl RendererFontEnvironmentHost,
) -> bool {
    let Some(config) = app.global::<TextFontFamilyConfig>().cloned() else {
        return false;
    };
    if renderer.set_text_font_families(&config) {
        let _ = publish_renderer_text_stack_key_if_changed(app, renderer);
        true
    } else {
        false
    }
}

#[doc(hidden)]
pub fn sync_renderer_locale_from_globals(
    app: &mut impl GlobalsHost,
    renderer: &mut impl RendererFontEnvironmentHost,
) -> bool {
    let locale = preferred_text_locale(app);
    if renderer.set_text_locale(locale.as_deref()) {
        let _ = publish_renderer_text_stack_key_if_changed(app, renderer);
        true
    } else {
        false
    }
}

#[doc(hidden)]
pub fn publish_renderer_font_environment(
    app: &mut impl GlobalsHost,
    renderer: &mut impl RendererFontEnvironmentHost,
    entries: Vec<FontCatalogEntry>,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let update = fret_runtime::apply_font_catalog_update_with_metadata(app, entries, policy);
    let _ = renderer.set_text_font_families(&update.config);
    let locale = preferred_text_locale(app);
    let _ = renderer.set_text_locale(locale.as_deref());
    let _ = publish_renderer_text_stack_key_if_changed(app, renderer);
    update
}

#[doc(hidden)]
pub fn apply_renderer_font_catalog_update(
    app: &mut impl GlobalsHost,
    renderer: &mut impl RendererFontEnvironmentHost,
    policy: FontFamilyDefaultsPolicy,
) -> FontCatalogUpdate {
    let entries = renderer.all_font_catalog_entries_runtime();
    publish_renderer_font_environment(app, renderer, entries, policy)
}

#[doc(hidden)]
pub fn initialize_startup_font_environment(
    app: &mut impl GlobalsHost,
    renderer: &mut impl RendererFontEnvironmentHost,
    config: TextFontFamilyConfig,
    mode: StartupFontEnvironmentMode,
) -> FontCatalogUpdate {
    let policy = startup_font_defaults_policy(mode);
    app.set_global::<TextFontFamilyConfig>(config);
    if matches!(mode, StartupFontEnvironmentMode::DesktopAsync) {
        publish_renderer_font_environment(app, renderer, Vec::new(), policy)
    } else {
        apply_renderer_font_catalog_update(app, renderer, policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::fret_i18n::{I18nService, LocaleId};
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestApp {
        globals: HashMap<TypeId, Box<dyn Any>>,
    }

    impl GlobalsHost for TestApp {
        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value: T = self
                .globals
                .remove(&type_id)
                .and_then(|v| v.downcast::<T>().ok())
                .map(|v| *v)
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[derive(Default)]
    struct TestRenderer {
        steps: Vec<&'static str>,
        last_key: u64,
        last_config: Option<TextFontFamilyConfig>,
        last_locale: Option<Option<String>>,
        entries: Vec<FontCatalogEntry>,
        font_blobs: Vec<Vec<u8>>,
    }

    impl RendererFontEnvironmentHost for TestRenderer {
        fn all_font_catalog_entries_runtime(&mut self) -> Vec<FontCatalogEntry> {
            self.steps.push("entries");
            self.entries.clone()
        }

        fn set_text_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
            self.steps.push("families");
            self.last_config = Some(config.clone());
            self.last_key = 11;
            true
        }

        fn set_text_locale(&mut self, locale: Option<&str>) -> bool {
            self.steps.push("locale");
            self.last_locale = Some(locale.map(ToOwned::to_owned));
            self.last_key = 42;
            true
        }

        fn text_font_stack_key(&self) -> u64 {
            self.last_key
        }
    }

    impl BundledFontBaselineHost for TestRenderer {
        fn add_font_blobs<I>(&mut self, fonts: I) -> usize
        where
            I: IntoIterator<Item = Vec<u8>>,
        {
            let start = self.font_blobs.len();
            self.font_blobs.extend(fonts);
            self.font_blobs.len() - start
        }
    }

    #[test]
    fn publish_renderer_font_environment_sets_key_after_locale_application() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer {
            entries: vec![FontCatalogEntry {
                family: "Inter".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let locale = LocaleId::parse("zh-CN").expect("locale must parse");
        app.set_global::<I18nService>(I18nService::new(vec![locale]));

        let _ = apply_renderer_font_catalog_update(
            &mut app,
            &mut renderer,
            FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );

        assert_eq!(renderer.steps, vec!["entries", "families", "locale"]);
        assert_eq!(renderer.last_locale, Some(Some("zh-CN".to_string())));
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            42,
            "expected the published key to reflect the post-locale renderer state"
        );
    }

    #[test]
    fn web_startup_font_environment_sets_key_after_locale_application() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer {
            entries: vec![FontCatalogEntry {
                family: "Inter".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let locale = LocaleId::parse("zh-CN").expect("locale must parse");
        app.set_global::<I18nService>(I18nService::new(vec![locale]));

        let _ = initialize_startup_font_environment(
            &mut app,
            &mut renderer,
            TextFontFamilyConfig::default(),
            StartupFontEnvironmentMode::WebBundledSync,
        );

        assert_eq!(renderer.steps, vec!["entries", "families", "locale"]);
        assert_eq!(renderer.last_locale, Some(Some("zh-CN".to_string())));
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            42,
            "expected web startup to publish the post-locale renderer key"
        );
    }

    #[test]
    fn web_startup_font_environment_preserves_existing_fields_while_seeding_missing_ones() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let config = TextFontFamilyConfig {
            ui_sans: vec!["Custom Sans".to_string()],
            ..Default::default()
        };

        let update = initialize_startup_font_environment(
            &mut app,
            &mut renderer,
            config.clone(),
            StartupFontEnvironmentMode::WebBundledSync,
        );

        assert_eq!(update.config.ui_sans, config.ui_sans);
        assert!(!update.config.ui_mono.is_empty());
        assert!(!update.config.common_fallback.is_empty());
        assert_eq!(renderer.last_config, Some(update.config));
    }

    #[test]
    fn publish_renderer_font_environment_with_empty_entries_preserves_existing_config() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let existing = TextFontFamilyConfig {
            ui_sans: vec!["Inter".to_string()],
            ..Default::default()
        };
        let locale = LocaleId::parse("en-US").expect("locale must parse");
        app.set_global::<TextFontFamilyConfig>(existing.clone());
        app.set_global::<I18nService>(I18nService::new(vec![locale]));

        let update = publish_renderer_font_environment(
            &mut app,
            &mut renderer,
            Vec::new(),
            FontFamilyDefaultsPolicy::None,
        );

        assert_eq!(update.config, existing);
        assert_eq!(renderer.last_config, Some(existing));
        assert_eq!(renderer.steps, vec!["families", "locale"]);
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            42
        );
    }

    #[test]
    fn desktop_async_startup_font_environment_preserves_config_and_key_order() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let existing = TextFontFamilyConfig {
            ui_sans: vec!["Inter".to_string()],
            ui_mono: vec!["Iosevka".to_string()],
            ..Default::default()
        };
        let locale = LocaleId::parse("en-US").expect("locale must parse");
        app.set_global::<I18nService>(I18nService::new(vec![locale]));

        let update = initialize_startup_font_environment(
            &mut app,
            &mut renderer,
            existing.clone(),
            StartupFontEnvironmentMode::DesktopAsync,
        );

        assert_eq!(update.config, existing);
        assert_eq!(renderer.last_config, Some(existing));
        assert_eq!(renderer.steps, vec!["families", "locale"]);
        assert_eq!(renderer.last_locale, Some(Some("en-US".to_string())));
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            42,
            "expected desktop async startup to publish the post-locale renderer key"
        );
    }

    #[test]
    fn sync_renderer_font_families_from_globals_updates_key() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let config = TextFontFamilyConfig {
            ui_sans: vec!["Inter".to_string()],
            ..Default::default()
        };
        app.set_global::<TextFontFamilyConfig>(config.clone());

        let changed = sync_renderer_font_families_from_globals(&mut app, &mut renderer);

        assert!(changed);
        assert_eq!(renderer.steps, vec!["families"]);
        assert_eq!(renderer.last_config, Some(config));
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            11,
            "expected family sync to publish the renderer's current stack key"
        );
    }

    #[test]
    fn sync_renderer_locale_from_globals_updates_key() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let locale = LocaleId::parse("ja-JP").expect("locale must parse");
        app.set_global::<I18nService>(I18nService::new(vec![locale]));

        let changed = sync_renderer_locale_from_globals(&mut app, &mut renderer);

        assert!(changed);
        assert_eq!(renderer.steps, vec!["locale"]);
        assert_eq!(renderer.last_locale, Some(Some("ja-JP".to_string())));
        assert_eq!(
            app.global::<fret_runtime::TextFontStackKey>()
                .expect("font stack key")
                .0,
            42,
            "expected locale sync to publish the renderer's current stack key"
        );
    }

    #[test]
    fn publish_system_font_rescan_state_updates_runtime_global() {
        let mut app = TestApp::default();

        let changed = publish_system_font_rescan_state(&mut app, true, false);

        assert!(changed);
        assert_eq!(
            app.global::<fret_runtime::SystemFontRescanState>()
                .copied()
                .expect("rescan state"),
            fret_runtime::SystemFontRescanState {
                in_flight: true,
                pending: false,
            }
        );
    }

    #[test]
    fn publish_system_font_rescan_state_is_noop_when_unchanged() {
        let mut app = TestApp::default();
        app.set_global::<fret_runtime::SystemFontRescanState>(
            fret_runtime::SystemFontRescanState {
                in_flight: false,
                pending: true,
            },
        );

        let changed = publish_system_font_rescan_state(&mut app, false, true);

        assert!(!changed);
        assert_eq!(
            app.global::<fret_runtime::SystemFontRescanState>()
                .copied()
                .expect("rescan state"),
            fret_runtime::SystemFontRescanState {
                in_flight: false,
                pending: true,
            }
        );
    }

    #[test]
    fn publish_bundled_font_baseline_snapshot_updates_runtime_global() {
        let mut app = TestApp::default();
        let snapshot = fret_runtime::BundledFontBaselineSnapshot::bundled_profile(
            "default-subset+cjk-lite",
            "pkg:fret-fonts",
            vec!["fonts/Inter-roman-subset.ttf".to_string()],
            vec!["UiSans".to_string(), "UiMonospace".to_string()],
            vec!["Sans".to_string(), "Monospace".to_string()],
        );

        let changed = publish_bundled_font_baseline_snapshot(&mut app, snapshot.clone());

        assert!(changed);
        assert_eq!(
            app.global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("bundled font baseline snapshot"),
            snapshot
        );
    }

    #[test]
    fn publish_bundled_font_baseline_snapshot_is_noop_when_unchanged() {
        let mut app = TestApp::default();
        let snapshot = fret_runtime::BundledFontBaselineSnapshot::none();
        app.set_global::<fret_runtime::BundledFontBaselineSnapshot>(snapshot.clone());

        let changed = publish_bundled_font_baseline_snapshot(&mut app, snapshot.clone());

        assert!(!changed);
        assert_eq!(
            app.global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("bundled font baseline snapshot"),
            snapshot
        );
    }

    #[test]
    fn default_bundled_font_baseline_snapshot_matches_default_profile() {
        let profile = fret_fonts::default_profile();
        let snapshot = default_bundled_font_baseline_snapshot();

        assert_eq!(
            snapshot,
            fret_runtime::BundledFontBaselineSnapshot::bundled_profile(
                profile.name,
                fret_fonts::bundled_asset_bundle().as_str(),
                profile
                    .faces
                    .iter()
                    .map(|face| face.asset_key.to_string())
                    .collect(),
                profile
                    .provided_roles
                    .iter()
                    .map(|role| bundled_font_role_name(*role).to_string())
                    .collect(),
                profile
                    .guaranteed_generic_families
                    .iter()
                    .map(|family| bundled_generic_family_name(*family).to_string())
                    .collect(),
            )
        );
    }

    #[test]
    fn install_default_bundled_font_baseline_adds_fonts_and_publishes_snapshot() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();

        let added = install_default_bundled_font_baseline(&mut app, &mut renderer);

        let expected_fonts =
            fret_fonts::test_support::face_blobs(fret_fonts::default_profile().faces.iter())
                .collect::<Vec<_>>();
        assert_eq!(added, expected_fonts.len());
        assert_eq!(renderer.font_blobs, expected_fonts);
        assert_eq!(
            app.global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("bundled font baseline snapshot"),
            default_bundled_font_baseline_snapshot()
        );
    }

    #[test]
    fn install_default_bundled_font_baseline_registers_profile_assets_in_runtime_resolver() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();
        let face = fret_fonts::default_profile()
            .faces
            .first()
            .copied()
            .expect("default bundled profile should expose at least one face");

        let _ = install_default_bundled_font_baseline(&mut app, &mut renderer);

        let resolved = fret_runtime::resolve_asset_bytes(&app, &face.asset_request())
            .expect("default bundled font face should resolve through the runtime asset resolver");

        assert_eq!(resolved.locator, face.asset_locator());
        assert_eq!(resolved.revision, face.asset_entry().revision);
        assert_eq!(resolved.bytes.as_ref(), face.bytes);
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some(face.media_type)
        );
    }

    #[test]
    fn install_default_bundled_font_baseline_registers_assets_only_once_per_app() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();

        let _ = install_default_bundled_font_baseline(&mut app, &mut renderer);
        let first_layer_count = fret_runtime::asset_resolver(&app)
            .map(|service| service.layered_resolvers().len())
            .expect("asset resolver should exist after baseline install");

        let _ = install_default_bundled_font_baseline(&mut app, &mut renderer);
        let second_layer_count = fret_runtime::asset_resolver(&app)
            .map(|service| service.layered_resolvers().len())
            .expect("asset resolver should remain available after repeated baseline install");

        assert_eq!(first_layer_count, 1);
        assert_eq!(second_layer_count, first_layer_count);
    }

    #[test]
    fn install_default_bundled_font_baseline_resolves_runtime_asset_bytes_for_startup_injection() {
        let mut app = TestApp::default();
        let mut first_renderer = TestRenderer::default();
        let profile = fret_fonts::default_profile();
        let first_face = profile
            .faces
            .first()
            .copied()
            .expect("default bundled profile should expose at least one face");
        let override_bytes = b"override-startup-font-bytes";

        let _ = install_default_bundled_font_baseline(&mut app, &mut first_renderer);

        fret_runtime::register_bundle_asset_entries(
            &mut app,
            fret_fonts::bundled_asset_bundle(),
            [fret_assets::StaticAssetEntry::new(
                first_face.asset_key,
                fret_assets::AssetRevision(999),
                override_bytes,
            )
            .with_media_type(first_face.media_type)],
        );

        let mut second_renderer = TestRenderer::default();
        let added = install_default_bundled_font_baseline(&mut app, &mut second_renderer);

        let mut expected_fonts =
            fret_fonts::test_support::face_blobs(profile.faces.iter()).collect::<Vec<_>>();
        expected_fonts[0] = override_bytes.to_vec();

        assert_eq!(added, expected_fonts.len());
        assert_eq!(second_renderer.font_blobs, expected_fonts);
    }

    #[test]
    fn initialize_web_startup_font_environment_installs_baseline_and_seeds_missing_families() {
        let mut app = TestApp::default();
        let mut renderer = TestRenderer::default();

        let update = initialize_web_startup_font_environment(
            &mut app,
            &mut renderer,
            TextFontFamilyConfig::default(),
        );

        assert_eq!(
            renderer.font_blobs.len(),
            fret_fonts::default_profile().faces.len()
        );
        assert_eq!(
            app.global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("web baseline snapshot"),
            default_bundled_font_baseline_snapshot(),
        );
        assert_eq!(renderer.steps, vec!["entries", "families", "locale"]);
        assert!(
            !update.config.ui_mono.is_empty(),
            "expected web startup to seed missing UI families"
        );
    }

    #[test]
    fn initialize_desktop_startup_font_environment_installs_baseline_for_sync_and_async_modes() {
        let mut sync_app = TestApp::default();
        let mut async_app = TestApp::default();
        let mut sync_renderer = TestRenderer::default();
        let mut async_renderer = TestRenderer::default();
        let sync_update = initialize_desktop_startup_font_environment(
            &mut sync_app,
            &mut sync_renderer,
            TextFontFamilyConfig::default(),
            false,
        );
        let async_update = initialize_desktop_startup_font_environment(
            &mut async_app,
            &mut async_renderer,
            TextFontFamilyConfig::default(),
            true,
        );

        assert_eq!(
            sync_renderer.font_blobs.len(),
            fret_fonts::default_profile().faces.len()
        );
        assert_eq!(
            async_renderer.font_blobs.len(),
            fret_fonts::default_profile().faces.len()
        );
        assert_eq!(
            sync_app
                .global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("desktop sync baseline snapshot"),
            async_app
                .global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("desktop async baseline snapshot"),
        );
        assert_eq!(sync_renderer.steps, vec!["entries", "families", "locale"]);
        assert_eq!(async_renderer.steps, vec!["families", "locale"]);
        assert!(
            sync_update.config.ui_mono.is_empty(),
            "expected desktop sync startup to preserve an empty family config under native policy"
        );
        assert!(
            async_update.config.ui_mono.is_empty(),
            "expected desktop async startup to preserve an empty family config under native policy"
        );
    }

    #[test]
    fn platform_startup_helpers_share_bundled_baseline_but_keep_distinct_defaults_policy() {
        let mut web_app = TestApp::default();
        let mut desktop_app = TestApp::default();
        let mut web_renderer = TestRenderer::default();
        let mut desktop_renderer = TestRenderer::default();

        let web_update = initialize_web_startup_font_environment(
            &mut web_app,
            &mut web_renderer,
            TextFontFamilyConfig::default(),
        );
        let desktop_update = initialize_desktop_startup_font_environment(
            &mut desktop_app,
            &mut desktop_renderer,
            TextFontFamilyConfig::default(),
            false,
        );

        assert_eq!(
            web_app
                .global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("web baseline snapshot"),
            desktop_app
                .global::<fret_runtime::BundledFontBaselineSnapshot>()
                .cloned()
                .expect("desktop baseline snapshot"),
        );
        assert_eq!(web_renderer.font_blobs, desktop_renderer.font_blobs);
        assert!(
            !web_update.config.ui_mono.is_empty(),
            "expected web startup to seed missing UI families"
        );
        assert!(
            desktop_update.config.ui_mono.is_empty(),
            "expected desktop startup to preserve an empty family config under native policy"
        );
    }
}
