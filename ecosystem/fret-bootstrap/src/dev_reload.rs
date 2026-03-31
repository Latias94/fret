use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use fret_app::App;
use fret_assets::{
    AssetBundleId, AssetCapabilities, AssetKindHint, AssetLoadError, AssetLocator, AssetRequest,
    AssetResolver, AssetRevision, ResolvedAssetBytes,
};
use fret_core::AppWindowId;
use fret_runtime::{Effect, TimerToken};
use fret_ui::{Theme, ThemeConfig};

use crate::HotLiterals;

#[derive(Debug, Clone, Default)]
struct FontsManifest {
    fonts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct DevReloadFontAssetBatch {
    resolved: Vec<ResolvedAssetBytes>,
    requests: Vec<AssetRequest>,
    errors: Vec<String>,
}

#[derive(Debug, Default)]
struct DevReloadFontAssetResolver {
    entries: RwLock<HashMap<AssetLocator, ResolvedAssetBytes>>,
}

impl DevReloadFontAssetResolver {
    fn replace_entries(&self, entries: impl IntoIterator<Item = ResolvedAssetBytes>) {
        let next = entries
            .into_iter()
            .map(|resolved| (resolved.locator.clone(), resolved))
            .collect::<HashMap<_, _>>();
        *self
            .entries
            .write()
            .expect("poisoned DevReloadFontAssetResolver entries lock") = next;
    }
}

impl AssetResolver for DevReloadFontAssetResolver {
    fn capabilities(&self) -> AssetCapabilities {
        AssetCapabilities {
            memory: false,
            embedded: false,
            bundle_asset: true,
            file: false,
            url: false,
            file_watch: false,
            system_font_scan: false,
        }
    }

    fn resolve_bytes(&self, request: &AssetRequest) -> Result<ResolvedAssetBytes, AssetLoadError> {
        if !matches!(request.locator, AssetLocator::BundleAsset(_)) {
            return Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            });
        }

        self.entries
            .read()
            .expect("poisoned DevReloadFontAssetResolver entries lock")
            .get(&request.locator)
            .cloned()
            .ok_or(AssetLoadError::NotFound)
    }
}

#[derive(Debug, Clone)]
struct DevReloadFontAssetLayer {
    installed: bool,
    resolver: Arc<DevReloadFontAssetResolver>,
}

impl Default for DevReloadFontAssetLayer {
    fn default() -> Self {
        Self {
            installed: false,
            resolver: Arc::new(DevReloadFontAssetResolver::default()),
        }
    }
}

fn dev_reload_font_bundle() -> AssetBundleId {
    AssetBundleId::package("fret-bootstrap-dev-reload-fonts")
}

fn ensure_dev_reload_font_asset_layer(app: &mut App) -> Arc<DevReloadFontAssetResolver> {
    let mut resolver = None;
    app.with_global_mut(DevReloadFontAssetLayer::default, |layer, app| {
        if !layer.installed {
            fret_runtime::register_asset_resolver(app, layer.resolver.clone());
            layer.installed = true;
        }
        resolver = Some(layer.resolver.clone());
    });
    resolver.expect("dev reload font asset layer must install a resolver")
}

fn stable_dev_reload_font_asset_revision(path: &Path, bytes: &[u8]) -> AssetRevision {
    let mut hasher = blake3::Hasher::new();
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.update(bytes);
    let hash = hasher.finalize();
    let mut revision_bytes = [0u8; 8];
    revision_bytes.copy_from_slice(&hash.as_bytes()[..8]);
    let revision = u64::from_le_bytes(revision_bytes);
    AssetRevision(if revision == 0 { 1 } else { revision })
}

fn dev_reload_font_asset_key(path: &Path) -> String {
    let hash = blake3::hash(path.to_string_lossy().as_bytes())
        .to_hex()
        .to_string();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("font");
    let sanitized = file_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("dev-reload/{}-{}", &hash[..12], sanitized)
}

fn dev_reload_font_media_type(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("ttf") => Some("font/ttf"),
        Some(ext) if ext.eq_ignore_ascii_case("otf") => Some("font/otf"),
        Some(ext) if ext.eq_ignore_ascii_case("ttc") => Some("font/collection"),
        Some(ext) if ext.eq_ignore_ascii_case("woff") => Some("font/woff"),
        Some(ext) if ext.eq_ignore_ascii_case("woff2") => Some("font/woff2"),
        _ => None,
    }
}

fn build_dev_reload_font_asset_batch(
    root: &Path,
    manifest: FontsManifest,
) -> DevReloadFontAssetBatch {
    let mut batch = DevReloadFontAssetBatch::default();

    for path in manifest.fonts {
        let abs = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        match std::fs::read(&abs) {
            Ok(bytes) => {
                let locator =
                    AssetLocator::bundle(dev_reload_font_bundle(), dev_reload_font_asset_key(&abs));
                let request =
                    AssetRequest::new(locator.clone()).with_kind_hint(AssetKindHint::Font);
                let mut resolved = ResolvedAssetBytes::new(
                    locator,
                    stable_dev_reload_font_asset_revision(&abs, &bytes),
                    bytes,
                );
                if let Some(media_type) = dev_reload_font_media_type(&abs) {
                    resolved = resolved.with_media_type(media_type);
                }
                batch.requests.push(request);
                batch.resolved.push(resolved);
            }
            Err(err) => batch
                .errors
                .push(format!("read failed: {}: {err}", abs.display())),
        }
    }

    batch
}

fn parse_fonts_manifest(bytes: &[u8]) -> Result<FontsManifest, String> {
    // Accept either:
    // - `["path/to/font.ttf", ...]`
    // - `{ "fonts": ["path/to/font.ttf", ...] }`
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Raw {
        List(Vec<String>),
        Obj { fonts: Vec<String> },
    }

    let raw: Raw =
        serde_json::from_slice(bytes).map_err(|e| format!("invalid fonts manifest JSON: {e}"))?;

    let list = match raw {
        Raw::List(v) => v,
        Raw::Obj { fonts } => fonts,
    };

    let fonts = list
        .into_iter()
        .filter_map(|s| {
            let s = s.trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(PathBuf::from(s))
            }
        })
        .collect();

    Ok(FontsManifest { fonts })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        len: meta.len(),
        modified: meta.modified().ok(),
    })
}

fn env_flag(name: &str) -> bool {
    let Some(v) = std::env::var_os(name) else {
        return false;
    };
    let v = v.to_string_lossy();
    let s = v.trim().to_ascii_lowercase();
    !(s.is_empty() || s == "0" || s == "false" || s == "off")
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok())
}

fn resolve_path(root: &Path, env_var: &str, default_rel: &str) -> PathBuf {
    if let Ok(raw) = std::env::var(env_var) {
        let raw = raw.trim();
        if !raw.is_empty() {
            let p = PathBuf::from(raw);
            return if p.is_absolute() { p } else { root.join(p) };
        }
    }
    root.join(default_rel)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DevReloadTick {
    pub(crate) reloaded_theme: bool,
    pub(crate) reloaded_literals: bool,
    pub(crate) bumped_asset_reload_epoch: bool,
    pub(crate) reloaded_fonts: bool,
    pub(crate) theme_error: Option<String>,
    pub(crate) literals_error: Option<String>,
    pub(crate) fonts_error: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct DevReloadWatcher {
    token: Option<TimerToken>,
    poll_interval: Duration,
    root: PathBuf,
    theme_path: PathBuf,
    literals_path: PathBuf,
    asset_reload_trigger_path: PathBuf,
    fonts_manifest_path: PathBuf,
    theme_stamp: Option<FileStamp>,
    literals_stamp: Option<FileStamp>,
    asset_reload_trigger_stamp: Option<FileStamp>,
    fonts_manifest_stamp: Option<FileStamp>,
}

impl DevReloadWatcher {
    fn enabled() -> bool {
        // Explicit override: allow disabling even in hotpatch contexts.
        if std::env::var_os("FRET_DEV_RELOAD").is_some() {
            return env_flag("FRET_DEV_RELOAD");
        }

        env_flag("FRET_HOTPATCH") || env_flag("DIOXUS_CLI_ENABLED") || env_flag("FRET_DIAG")
    }

    pub(crate) fn install_if_enabled(app: &mut App) {
        if cfg!(target_arch = "wasm32") {
            return;
        }
        if !Self::enabled() {
            return;
        }

        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let poll_ms = env_u64("FRET_DEV_RELOAD_POLL_MS").unwrap_or(250);
        let poll_interval = Duration::from_millis(poll_ms.max(16));

        let theme_path = resolve_path(&root, "FRET_DEV_RELOAD_THEME_PATH", ".fret/theme.json");
        let literals_path = resolve_path(
            &root,
            "FRET_DEV_RELOAD_LITERALS_PATH",
            ".fret/literals.json",
        );
        let asset_reload_trigger_path = resolve_path(
            &root,
            "FRET_DEV_RELOAD_ASSET_RELOAD_TRIGGER_PATH",
            ".fret/asset_reload.touch",
        );
        let fonts_manifest_path =
            resolve_path(&root, "FRET_DEV_RELOAD_FONTS_PATH", ".fret/fonts.json");

        let existing = app.global::<DevReloadWatcher>().map(|w| {
            (
                w.token,
                w.poll_interval,
                w.root.clone(),
                w.theme_path.clone(),
                w.literals_path.clone(),
                w.asset_reload_trigger_path.clone(),
                w.fonts_manifest_path.clone(),
            )
        });
        if let Some((
            Some(_token),
            prev_interval,
            prev_root,
            prev_theme,
            prev_lits,
            prev_assets,
            prev_fonts,
        )) = existing.as_ref()
            && *prev_interval == poll_interval
            && prev_root == &root
            && prev_theme == &theme_path
            && prev_lits == &literals_path
            && prev_assets == &asset_reload_trigger_path
            && prev_fonts == &fonts_manifest_path
        {
            return;
        }

        if let Some((Some(token), ..)) = existing.as_ref() {
            app.push_effect(Effect::CancelTimer { token: *token });
        }

        let token = app.next_timer_token();
        app.push_effect(Effect::SetTimer {
            window: None,
            token,
            after: poll_interval,
            repeat: Some(poll_interval),
        });

        app.set_global(DevReloadWatcher {
            token: Some(token),
            poll_interval,
            root,
            theme_path,
            literals_path,
            asset_reload_trigger_path,
            fonts_manifest_path,
            theme_stamp: None,
            literals_stamp: None,
            asset_reload_trigger_stamp: None,
            fonts_manifest_stamp: None,
        });
    }

    fn poll_and_apply(&mut self, app: &mut App, window: AppWindowId) -> DevReloadTick {
        let mut tick = DevReloadTick::default();

        let next_theme_stamp = file_stamp(&self.theme_path);
        let theme_changed = next_theme_stamp != self.theme_stamp;
        self.theme_stamp = next_theme_stamp;

        if theme_changed && let Some(_stamp) = self.theme_stamp {
            match std::fs::read(&self.theme_path) {
                Ok(bytes) => match ThemeConfig::from_slice(&bytes) {
                    Ok(cfg) => {
                        Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
                        app.request_redraw(window);
                        tick.reloaded_theme = true;
                    }
                    Err(e) => tick.theme_error = Some(format!("theme parse failed: {e}")),
                },
                Err(e) => tick.theme_error = Some(format!("theme read failed: {e}")),
            }
        }

        let next_literals_stamp = file_stamp(&self.literals_path);
        let literals_changed = next_literals_stamp != self.literals_stamp;
        self.literals_stamp = next_literals_stamp;

        if literals_changed && let Some(_stamp) = self.literals_stamp {
            match std::fs::read(&self.literals_path) {
                Ok(bytes) => match HotLiterals::from_json_slice(&bytes) {
                    Ok(lits) => {
                        app.set_global(lits);
                        app.request_redraw(window);
                        tick.reloaded_literals = true;
                    }
                    Err(e) => tick.literals_error = Some(e),
                },
                Err(e) => tick.literals_error = Some(format!("literals read failed: {e}")),
            }
        }

        let next_assets_stamp = file_stamp(&self.asset_reload_trigger_path);
        let assets_changed = next_assets_stamp != self.asset_reload_trigger_stamp;
        self.asset_reload_trigger_stamp = next_assets_stamp;

        let next_fonts_manifest_stamp = file_stamp(&self.fonts_manifest_path);
        let fonts_manifest_changed = next_fonts_manifest_stamp != self.fonts_manifest_stamp;
        self.fonts_manifest_stamp = next_fonts_manifest_stamp;

        if assets_changed && self.asset_reload_trigger_stamp.is_some() {
            #[cfg(feature = "ui-assets")]
            {
                fret_runtime::bump_asset_reload_epoch(app);
                app.request_redraw(window);
                tick.bumped_asset_reload_epoch = true;
            }
        }

        let should_reload_fonts = fonts_manifest_changed || tick.bumped_asset_reload_epoch;
        if should_reload_fonts && let Some(_stamp) = self.fonts_manifest_stamp {
            match std::fs::read(&self.fonts_manifest_path) {
                Ok(bytes) => match parse_fonts_manifest(&bytes) {
                    Ok(manifest) => {
                        let batch = build_dev_reload_font_asset_batch(&self.root, manifest);
                        let resolver = ensure_dev_reload_font_asset_layer(app);
                        resolver.replace_entries(batch.resolved.clone());

                        if !batch.errors.is_empty() {
                            tick.fonts_error = Some(batch.errors.join("; "));
                        }

                        if !batch.requests.is_empty() {
                            app.push_effect(Effect::TextAddFontAssets {
                                requests: batch.requests,
                            });
                            tick.reloaded_fonts = true;
                        }
                    }
                    Err(e) => tick.fonts_error = Some(e),
                },
                Err(e) => tick.fonts_error = Some(format!("fonts manifest read failed: {e}")),
            }
        }

        tick
    }
}

pub(crate) fn handle_dev_reload_timer(
    app: &mut App,
    window: AppWindowId,
    token: TimerToken,
) -> Option<DevReloadTick> {
    let watcher_token = app.global::<DevReloadWatcher>().and_then(|w| w.token);
    if watcher_token != Some(token) {
        return None;
    }

    let mut out: Option<DevReloadTick> = None;
    app.with_global_mut(DevReloadWatcher::default, |w, app| {
        if w.token != Some(token) {
            return;
        }
        out = Some(w.poll_and_apply(app, window));
    });
    out
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use fret_assets::{AssetKindHint, AssetLocator};

    use super::*;

    fn unique_temp_dir(label: &str) -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "fret-dev-reload-{label}-{}-{nanos}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("temp dir must be creatable");
        dir
    }

    #[test]
    fn build_font_asset_batch_emits_font_bundle_requests() {
        let root = unique_temp_dir("requests");
        let font_path = root.join("demo font.ttf");
        fs::write(&font_path, b"fake-font-bytes").expect("font fixture must be writable");

        let batch = build_dev_reload_font_asset_batch(
            &root,
            FontsManifest {
                fonts: vec![PathBuf::from("demo font.ttf")],
            },
        );

        assert!(batch.errors.is_empty());
        assert_eq!(batch.requests.len(), 1);
        assert_eq!(batch.resolved.len(), 1);
        assert_eq!(batch.requests[0].kind_hint, Some(AssetKindHint::Font));
        assert_eq!(&batch.requests[0].locator, &batch.resolved[0].locator);
        match &batch.requests[0].locator {
            AssetLocator::BundleAsset(locator) => {
                assert_eq!(locator.bundle, dev_reload_font_bundle());
            }
            other => panic!("expected bundle asset request, got {other:?}"),
        }

        fs::remove_dir_all(&root).expect("temp dir must be removable");
    }

    #[test]
    fn build_font_asset_batch_records_read_failures() {
        let root = unique_temp_dir("errors");

        let batch = build_dev_reload_font_asset_batch(
            &root,
            FontsManifest {
                fonts: vec![PathBuf::from("missing.ttf")],
            },
        );

        assert!(batch.requests.is_empty());
        assert!(batch.resolved.is_empty());
        assert_eq!(batch.errors.len(), 1);
        assert!(batch.errors[0].contains("missing.ttf"));

        fs::remove_dir_all(&root).expect("temp dir must be removable");
    }
}
