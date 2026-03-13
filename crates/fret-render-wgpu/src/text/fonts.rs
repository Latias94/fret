use super::{TextFontFamilyConfig, TextSystem};

impl TextSystem {
    pub(super) fn finish_initial_font_bootstrap(&mut self) {
        let _ = self.apply_font_families_inner(&self.fallback_policy.font_family_config.clone());
        self.fallback_policy.recompute_key(&self.parley_shaper);
        self.recompute_font_stack_key();
    }

    /// Returns a sorted list of available font family names.
    ///
    /// This is intended for settings/UI pickers. The result is best-effort and platform-dependent.
    pub fn all_font_names(&mut self) -> Vec<String> {
        self.parley_shaper.all_font_names()
    }

    pub fn all_font_catalog_entries(&mut self) -> Vec<super::FontCatalogEntryMetadata> {
        self.parley_shaper.all_font_catalog_entries()
    }

    pub fn font_stack_key(&self) -> u64 {
        self.font_stack_key
    }

    /// Sets the default locale for text shaping and font fallback selection.
    ///
    /// This participates in `font_stack_key` and clears text caches when changed.
    pub fn set_text_locale(&mut self, locale_bcp47: Option<&str>) -> bool {
        let parsed = locale_bcp47
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .filter(|v| parley::swash::text::Language::parse(v).is_some())
            .map(|v| v.to_string());

        if self.fallback_policy.locale_bcp47 == parsed {
            return false;
        }

        self.fallback_policy.locale_bcp47 = parsed.clone();
        let _ = self.parley_shaper.set_default_locale(parsed);

        self.font_db_revision = self.font_db_revision.saturating_add(1);
        self.fallback_policy.refresh_derived(&self.parley_shaper);
        self.fallback_policy.recompute_key(&self.parley_shaper);
        self.recompute_font_stack_key();
        self.reset_caches_for_font_change();
        true
    }

    /// Adds font bytes (TTF/OTF/TTC) to the font database.
    ///
    /// Returns the number of newly loaded faces. When this returns non-zero, all cached text blobs
    /// and atlas entries are cleared to avoid reusing stale shaping/rasterization results.
    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        let fonts: Vec<Vec<u8>> = fonts.into_iter().collect();

        let added = self.parley_shaper.add_fonts(fonts);
        if added > 0 {
            let _ =
                self.apply_font_families_inner(&self.fallback_policy.font_family_config.clone());
            self.fallback_policy.recompute_key(&self.parley_shaper);
            self.font_db_revision = self.font_db_revision.saturating_add(1);
            self.recompute_font_stack_key();
            self.reset_caches_for_font_change();
        }

        added
    }

    /// Best-effort rescan of system-installed fonts (native-only).
    ///
    /// When this returns `true`, the text system bumps `font_stack_key` and clears all cached text
    /// blobs and atlas entries, ensuring shaping/rasterization results cannot be reused across the
    /// rescan boundary.
    pub fn rescan_system_fonts(&mut self) -> bool {
        let Some(seed) = self.system_font_rescan_seed() else {
            return false;
        };

        let result = seed.run();
        self.apply_system_font_rescan_result(result)
    }

    pub fn system_font_rescan_seed(&self) -> Option<super::SystemFontRescanSeed> {
        self.parley_shaper.system_font_rescan_seed()
    }

    pub fn apply_system_font_rescan_result(
        &mut self,
        result: super::SystemFontRescanResult,
    ) -> bool {
        let changed = self.parley_shaper.apply_system_font_rescan_result(result);
        if !changed {
            return false;
        }

        // Re-apply the current font-family policy and generic injections after swapping the
        // underlying fontique collection (rescan replaces the collection entirely).
        //
        // This keeps selection/fallback behavior stable across rescan boundaries and prevents
        // stale injected FamilyIds from hanging around.
        self.generic_injections.clear();
        let _ = self.apply_font_families_inner(&self.fallback_policy.font_family_config.clone());

        self.font_db_revision = self.font_db_revision.saturating_add(1);
        self.fallback_policy.recompute_key(&self.parley_shaper);
        self.recompute_font_stack_key();
        self.reset_caches_for_font_change();
        true
    }

    fn reset_caches_for_font_change(&mut self) {
        self.frame_perf.cache_resets = self.frame_perf.cache_resets.saturating_add(1);
        self.blob_state.clear();
        self.shape_cache.clear();
        self.measure.clear();
        self.mask_atlas.reset();
        self.color_atlas.reset();
        self.subpixel_atlas.reset();
        self.pin_state.clear();
        self.face_cache.clear();
    }

    pub fn set_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        let (stacks_changed, suffix_changed, mode_changed) = self.apply_font_families_inner(config);
        if !stacks_changed && !suffix_changed && !mode_changed {
            return false;
        }

        self.font_db_revision = self.font_db_revision.saturating_add(1);
        self.fallback_policy.recompute_key(&self.parley_shaper);
        self.recompute_font_stack_key();
        self.reset_caches_for_font_change();
        true
    }

    pub(super) fn apply_font_families_inner(
        &mut self,
        config: &TextFontFamilyConfig,
    ) -> (bool, bool, bool) {
        fret_render_text::font_stack::apply_font_families_inner(
            &mut self.parley_shaper,
            &mut self.fallback_policy,
            &mut self.generic_injections,
            config,
        )
    }

    pub(super) fn recompute_font_stack_key(&mut self) {
        use std::hash::{Hash as _, Hasher as _};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "fret.text.font_stack_key.v1".hash(&mut hasher);
        self.font_db_revision.hash(&mut hasher);
        self.fallback_policy.fallback_policy_key.hash(&mut hasher);
        let key = hasher.finish();
        self.font_stack_key = if key == 0 { 1 } else { key };
    }
}
