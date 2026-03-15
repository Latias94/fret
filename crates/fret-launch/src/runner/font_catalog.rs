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

impl RendererFontEnvironmentHost for fret_render::Renderer {
    fn all_font_catalog_entries_runtime(&mut self) -> Vec<FontCatalogEntry> {
        self.all_font_catalog_entries()
            .into_iter()
            .map(|e| FontCatalogEntry {
                family: e.family,
                has_variable_axes: e.has_variable_axes,
                known_variable_axes: e.known_variable_axes,
                variable_axes: e
                    .variable_axes
                    .into_iter()
                    .map(|a| fret_runtime::FontVariableAxisInfo {
                        tag: a.tag,
                        min_bits: a.min_bits,
                        max_bits: a.max_bits,
                        default_bits: a.default_bits,
                    })
                    .collect(),
                is_monospace_candidate: e.is_monospace_candidate,
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

fn preferred_text_locale(app: &impl GlobalsHost) -> Option<String> {
    app.global::<I18nService>()
        .and_then(|service| service.preferred_locales().first())
        .map(|locale| locale.to_string())
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
}
