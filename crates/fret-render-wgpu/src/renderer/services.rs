use super::shaders::{
    custom_effect_mask_shader_source, custom_effect_masked_shader_source,
    custom_effect_unmasked_shader_source, custom_effect_v2_mask_shader_source,
    custom_effect_v2_masked_shader_source, custom_effect_v2_unmasked_shader_source,
    custom_effect_v3_mask_shader_source, custom_effect_v3_masked_shader_source,
    custom_effect_v3_unmasked_shader_source,
};
use super::*;
use std::collections::hash_map::Entry;

const MAX_CUSTOM_EFFECT_WGSL_BYTES: usize = 64 * 1024;

fn validate_custom_effect_wgsl_triplet(
    user_source: &str,
    wgsl_unmasked: &str,
    wgsl_masked: &str,
    wgsl_mask: &str,
    parse_failed_msg: &'static str,
    validation_failed_msg: &'static str,
) -> Result<(), fret_core::CustomEffectRegistrationError> {
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    if user_source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
        return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
    }

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    for (label, src) in [
        ("unmasked", wgsl_unmasked),
        ("masked", wgsl_masked),
        ("mask", wgsl_mask),
    ] {
        let module = naga::front::wgsl::parse_str(src).map_err(|err| {
            tracing::warn!(?err, %label, "{parse_failed_msg}");
            fret_core::CustomEffectRegistrationError::InvalidSource
        })?;
        validator.validate(&module).map_err(|err| {
            tracing::warn!(?err, %label, "{validation_failed_msg}");
            fret_core::CustomEffectRegistrationError::InvalidSource
        })?;
    }

    Ok(())
}

fn build_and_validate_custom_effect_wgsl_with_sources(
    user_source: &str,
    unmasked_source: fn(&str) -> String,
    masked_source: fn(&str) -> String,
    mask_source: fn(&str) -> String,
    parse_failed_msg: &'static str,
    validation_failed_msg: &'static str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    if user_source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
        return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
    }

    let wgsl_unmasked = unmasked_source(user_source);
    let wgsl_masked = masked_source(user_source);
    let wgsl_mask = mask_source(user_source);

    validate_custom_effect_wgsl_triplet(
        user_source,
        &wgsl_unmasked,
        &wgsl_masked,
        &wgsl_mask,
        parse_failed_msg,
        validation_failed_msg,
    )?;

    Ok((wgsl_unmasked, wgsl_masked, wgsl_mask))
}

fn build_and_validate_custom_effect_wgsl_v1(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        user_source,
        custom_effect_unmasked_shader_source,
        custom_effect_masked_shader_source,
        custom_effect_mask_shader_source,
        "custom effect v1 wgsl parse failed",
        "custom effect v1 wgsl validation failed",
    )
}

fn build_and_validate_custom_effect_wgsl_v2(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        user_source,
        custom_effect_v2_unmasked_shader_source,
        custom_effect_v2_masked_shader_source,
        custom_effect_v2_mask_shader_source,
        "custom effect v2 wgsl parse failed",
        "custom effect v2 wgsl validation failed",
    )
}

fn build_and_validate_custom_effect_wgsl_v3(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        user_source,
        custom_effect_v3_unmasked_shader_source,
        custom_effect_v3_masked_shader_source,
        custom_effect_v3_mask_shader_source,
        "custom effect v3 wgsl parse failed",
        "custom effect v3 wgsl validation failed",
    )
}

impl fret_core::TextService for Renderer {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        match input {
            fret_core::TextInput::Plain { text, style } => {
                self.text_system.prepare(text.as_ref(), style, constraints)
            }
            fret_core::TextInput::Attributed { text, base, spans } => {
                let rich = fret_core::AttributedText::new(text.clone(), spans.clone());
                self.text_system
                    .prepare_attributed(&rich, base, constraints)
            }
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                self.text_system.prepare(
                    input.text(),
                    &fret_core::TextStyle::default(),
                    constraints,
                )
            }
        }
    }

    fn measure(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> fret_core::TextMetrics {
        match input {
            fret_core::TextInput::Plain { text, style } => {
                self.text_system.measure(text.as_ref(), style, constraints)
            }
            fret_core::TextInput::Attributed { text, base, spans } => {
                let rich = fret_core::AttributedText::new(text.clone(), spans.clone());
                self.text_system
                    .measure_attributed(&rich, base, constraints)
            }
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                self.text_system.measure(
                    input.text(),
                    &fret_core::TextStyle::default(),
                    constraints,
                )
            }
        }
    }

    fn caret_x(&mut self, blob: fret_core::TextBlobId, index: usize) -> fret_core::Px {
        self.text_system
            .caret_x(blob, index)
            .unwrap_or(fret_core::Px(0.0))
    }

    fn hit_test_x(&mut self, blob: fret_core::TextBlobId, x: fret_core::Px) -> usize {
        self.text_system.hit_test_x(blob, x).unwrap_or(0)
    }

    fn selection_rects(
        &mut self,
        blob: fret_core::TextBlobId,
        range: (usize, usize),
        out: &mut Vec<fret_core::Rect>,
    ) {
        let _ = self.text_system.selection_rects(blob, range, out);
    }

    fn first_line_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextLineMetrics> {
        self.text_system.first_line_metrics(blob)
    }

    fn first_line_ink_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextInkMetrics> {
        self.text_system.first_line_ink_metrics(blob)
    }

    fn last_line_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextLineMetrics> {
        self.text_system.last_line_metrics(blob)
    }

    fn last_line_ink_metrics(
        &mut self,
        blob: fret_core::TextBlobId,
    ) -> Option<fret_core::TextInkMetrics> {
        self.text_system.last_line_ink_metrics(blob)
    }

    fn selection_rects_clipped(
        &mut self,
        blob: fret_core::TextBlobId,
        range: (usize, usize),
        clip: fret_core::Rect,
        out: &mut Vec<fret_core::Rect>,
    ) {
        let _ = self
            .text_system
            .selection_rects_clipped(blob, range, clip, out);
    }

    fn caret_stops(&mut self, blob: fret_core::TextBlobId, out: &mut Vec<(usize, fret_core::Px)>) {
        out.clear();
        if let Some(stops) = self.text_system.caret_stops(blob) {
            out.extend_from_slice(stops);
        }
    }

    fn caret_rect(
        &mut self,
        blob: fret_core::TextBlobId,
        index: usize,
        affinity: fret_core::CaretAffinity,
    ) -> fret_core::Rect {
        self.text_system
            .caret_rect(blob, index, affinity)
            .unwrap_or_default()
    }

    fn hit_test_point(
        &mut self,
        blob: fret_core::TextBlobId,
        point: fret_core::Point,
    ) -> fret_core::HitTestResult {
        self.text_system
            .hit_test_point(blob, point)
            .unwrap_or(fret_core::HitTestResult {
                index: 0,
                affinity: fret_core::CaretAffinity::Downstream,
            })
    }

    fn release(&mut self, blob: fret_core::TextBlobId) {
        self.text_system.release(blob);
    }
}

impl fret_core::PathService for Renderer {
    fn prepare(
        &mut self,
        commands: &[fret_core::PathCommand],
        style: fret_core::PathStyle,
        constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        let key = path_cache_key(commands, style, constraints);
        let epoch = self.bump_path_cache_epoch();

        match self.path_cache.entry(key) {
            Entry::Occupied(mut e) => {
                let entry = e.get_mut();
                entry.refs = entry.refs.saturating_add(1);
                entry.last_used_epoch = epoch;
                let id = entry.id;

                if let Some(prepared) = self.paths.get(id) {
                    return (id, prepared.metrics);
                }

                // Cache entry is stale (should be rare). Rebuild it.
                e.remove();
            }
            Entry::Vacant(_) => {}
        }

        let metrics = metrics_from_path_commands(commands, style);
        let (triangles, stroke_s01_mode) = tessellate_path_commands(commands, style, constraints);
        let id = self.paths.insert(PreparedPath {
            metrics,
            triangles,
            stroke_s01_mode,
            cache_key: key,
        });
        self.path_cache.insert(
            key,
            CachedPathEntry {
                id,
                refs: 1,
                last_used_epoch: epoch,
            },
        );
        self.prune_path_cache();
        (id, metrics)
    }

    fn release(&mut self, path: fret_core::PathId) {
        let Some(cache_key) = self.paths.get(path).map(|p| p.cache_key) else {
            return;
        };

        if let Some(entry) = self.path_cache.get_mut(&cache_key)
            && entry.refs > 0
        {
            entry.refs -= 1;
        }

        self.prune_path_cache();
    }
}

impl fret_core::SvgService for Renderer {
    fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
        let h = hash_bytes(bytes);
        if let Some(ids) = self.svg_hash_index.get(&h) {
            for &id in ids {
                let Some(existing) = self.svgs.get(id) else {
                    continue;
                };
                if existing.bytes.as_ref() == bytes {
                    if let Some(entry) = self.svgs.get_mut(id) {
                        entry.refs = entry.refs.saturating_add(1);
                    }
                    return id;
                }
            }
        }

        let id = self.svgs.insert(super::types::SvgEntry {
            bytes: Arc::<[u8]>::from(bytes),
            refs: 1,
        });
        self.svg_hash_index.entry(h).or_default().push(id);
        id
    }

    fn unregister_svg(&mut self, svg: fret_core::SvgId) -> bool {
        let Some(refs) = self.svgs.get(svg).map(|e| e.refs) else {
            return false;
        };

        if refs > 1 {
            if let Some(entry) = self.svgs.get_mut(svg) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return true;
        }

        let Some(bytes) = self.svgs.remove(svg).map(|e| e.bytes) else {
            return false;
        };

        let h = hash_bytes(&bytes);
        if let Some(list) = self.svg_hash_index.get_mut(&h) {
            list.retain(|id| *id != svg);
            if list.is_empty() {
                self.svg_hash_index.remove(&h);
            }
        }

        // Drop any cached rasterizations for this SVG.
        let mut keys_to_remove: Vec<SvgRasterKey> = Vec::new();
        for k in self.svg_rasters.keys() {
            if k.svg == svg {
                keys_to_remove.push(*k);
            }
        }
        for k in keys_to_remove {
            if let Some(entry) = self.svg_rasters.remove(&k) {
                self.drop_svg_raster_entry(entry);
            }
        }

        true
    }
}

fn catalog_texture_material_supported(
    allowed_usages: wgpu::TextureUsages,
    flags: wgpu::TextureFormatFeatureFlags,
) -> bool {
    let ok_usages = allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
    let ok_filterable = flags.contains(wgpu::TextureFormatFeatureFlags::FILTERABLE);
    ok_usages && ok_filterable
}

impl fret_core::MaterialService for Renderer {
    fn register_material(
        &mut self,
        desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        // v1: all baseline material kinds are supported by the quad shader on wgpu backends.
        //
        // v2: sampled materials are capability-gated by a fixed, renderer-owned catalog texture
        // binding shape (ADR 0242).
        match desc.binding {
            fret_core::MaterialBindingShape::ParamsOnly => {}
            fret_core::MaterialBindingShape::ParamsPlusCatalogTexture { .. } => {
                let f = self
                    .adapter
                    .get_texture_format_features(wgpu::TextureFormat::Rgba8Unorm);
                if !catalog_texture_material_supported(f.allowed_usages, f.flags) {
                    return Err(fret_core::MaterialRegistrationError::Unsupported);
                }
            }
        }

        match self.materials_by_desc.entry(desc) {
            Entry::Occupied(e) => {
                let id = *e.get();
                if let Some(entry) = self.materials.get_mut(id) {
                    entry.refs = entry.refs.saturating_add(1);
                }
                Ok(id)
            }
            Entry::Vacant(e) => {
                let id = self
                    .materials
                    .insert(super::MaterialEntry { desc, refs: 1 });
                e.insert(id);
                self.materials_generation = self.materials_generation.wrapping_add(1);
                Ok(id)
            }
        }
    }

    fn unregister_material(&mut self, id: fret_core::MaterialId) -> bool {
        let Some(refs) = self.materials.get(id).map(|e| e.refs) else {
            return false;
        };

        if refs > 1 {
            if let Some(entry) = self.materials.get_mut(id) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return true;
        }

        let Some(entry) = self.materials.remove(id) else {
            return false;
        };

        self.materials_by_desc.remove(&entry.desc);
        self.materials_generation = self.materials_generation.wrapping_add(1);
        true
    }
}

impl fret_core::CustomEffectService for Renderer {
    fn register_custom_effect_v1(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV1,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        let abi = CustomEffectAbi::V1;
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        if desc.source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
            return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
        }

        let h = mix_u64(hash_bytes(desc.source.as_bytes()), 1);
        if let Some(ids) = self.custom_effect_hash_index.get(&h) {
            for &id in ids {
                let Some(existing) = self.custom_effects.get(id) else {
                    continue;
                };
                if existing.abi != abi {
                    continue;
                }
                if existing.raw_source.as_ref() == desc.source.as_str() {
                    if let Some(entry) = self.custom_effects.get_mut(id) {
                        entry.refs = entry.refs.saturating_add(1);
                    }
                    return Ok(id);
                }
            }
        }

        let (wgsl_unmasked, wgsl_masked, wgsl_mask) =
            build_and_validate_custom_effect_wgsl_v1(&desc.source)?;

        let raw_source: Arc<str> = Arc::from(desc.source);
        let id = self.custom_effects.insert(super::CustomEffectEntry {
            abi,
            raw_source: raw_source.clone(),
            wgsl_unmasked: Arc::from(wgsl_unmasked),
            wgsl_masked: Arc::from(wgsl_masked),
            wgsl_mask: Arc::from(wgsl_mask),
            refs: 1,
        });
        self.custom_effect_hash_index.entry(h).or_default().push(id);
        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        Ok(id)
    }

    fn register_custom_effect_v2(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV2,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        let abi = CustomEffectAbi::V2;
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        // V2 requires a filterable sampled input texture (the user image input). Gate the feature
        // on a conservative and widely supported format so apps can reliably probe support.
        let fmt = wgpu::TextureFormat::Rgba8Unorm;
        let f = self.adapter.get_texture_format_features(fmt);
        if !f
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !f
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
        {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        if desc.source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
            return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
        }

        let h = mix_u64(hash_bytes(desc.source.as_bytes()), 2);
        if let Some(ids) = self.custom_effect_hash_index.get(&h) {
            for &id in ids {
                let Some(existing) = self.custom_effects.get(id) else {
                    continue;
                };
                if existing.abi != abi {
                    continue;
                }
                if existing.raw_source.as_ref() == desc.source.as_str() {
                    if let Some(entry) = self.custom_effects.get_mut(id) {
                        entry.refs = entry.refs.saturating_add(1);
                    }
                    return Ok(id);
                }
            }
        }

        let (wgsl_unmasked, wgsl_masked, wgsl_mask) =
            build_and_validate_custom_effect_wgsl_v2(&desc.source)?;

        let raw_source: Arc<str> = Arc::from(desc.source);
        let id = self.custom_effects.insert(super::CustomEffectEntry {
            abi,
            raw_source: raw_source.clone(),
            wgsl_unmasked: Arc::from(wgsl_unmasked),
            wgsl_masked: Arc::from(wgsl_masked),
            wgsl_mask: Arc::from(wgsl_mask),
            refs: 1,
        });
        self.custom_effect_hash_index.entry(h).or_default().push(id);
        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        Ok(id)
    }

    fn register_custom_effect_v3(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV3,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        let abi = CustomEffectAbi::V3;
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        // V3 currently keeps the same conservative requirement as V2 for user image inputs:
        // a filterable sampled texture (RGBA8). Backends that cannot provide it should report
        // `Unsupported` deterministically.
        let fmt = wgpu::TextureFormat::Rgba8Unorm;
        let f = self.adapter.get_texture_format_features(fmt);
        if !f
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !f
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
        {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        if desc.source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
            return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
        }

        let h = mix_u64(hash_bytes(desc.source.as_bytes()), 3);
        if let Some(ids) = self.custom_effect_hash_index.get(&h) {
            for &id in ids {
                let Some(existing) = self.custom_effects.get(id) else {
                    continue;
                };
                if existing.abi != abi {
                    continue;
                }
                if existing.raw_source.as_ref() == desc.source.as_str() {
                    if let Some(entry) = self.custom_effects.get_mut(id) {
                        entry.refs = entry.refs.saturating_add(1);
                    }
                    return Ok(id);
                }
            }
        }

        let (wgsl_unmasked, wgsl_masked, wgsl_mask) =
            build_and_validate_custom_effect_wgsl_v3(&desc.source)?;

        let raw_source: Arc<str> = Arc::from(desc.source);
        let id = self.custom_effects.insert(super::CustomEffectEntry {
            abi,
            raw_source: raw_source.clone(),
            wgsl_unmasked: Arc::from(wgsl_unmasked),
            wgsl_masked: Arc::from(wgsl_masked),
            wgsl_mask: Arc::from(wgsl_mask),
            refs: 1,
        });
        self.custom_effect_hash_index.entry(h).or_default().push(id);
        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        Ok(id)
    }

    fn unregister_custom_effect(&mut self, id: fret_core::EffectId) -> bool {
        let Some(refs) = self.custom_effects.get(id).map(|e| e.refs) else {
            return false;
        };

        if refs > 1 {
            if let Some(entry) = self.custom_effects.get_mut(id) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return true;
        }

        let Some(entry) = self.custom_effects.remove(id) else {
            return false;
        };

        let h = mix_u64(
            hash_bytes(entry.raw_source.as_bytes()),
            match entry.abi {
                CustomEffectAbi::V1 => 1,
                CustomEffectAbi::V2 => 2,
                CustomEffectAbi::V3 => 3,
            },
        );
        if let Some(list) = self.custom_effect_hash_index.get_mut(&h) {
            list.retain(|x| *x != id);
            if list.is_empty() {
                self.custom_effect_hash_index.remove(&h);
            }
        }

        // Drop any cached pipelines for this effect.
        self.pipelines.custom_effect_pipelines.remove(&id);
        self.pipelines.custom_effect_v2_pipelines.remove(&id);
        self.pipelines.custom_effect_v3_pipelines.remove(&id);

        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::CustomEffectService as _;

    #[test]
    fn sampled_material_registration_is_capability_gated() {
        let required_usages = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let required_flags = wgpu::TextureFormatFeatureFlags::FILTERABLE;

        assert!(!catalog_texture_material_supported(
            wgpu::TextureUsages::COPY_DST,
            required_flags
        ));
        assert!(!catalog_texture_material_supported(
            wgpu::TextureUsages::TEXTURE_BINDING,
            required_flags
        ));
        assert!(!catalog_texture_material_supported(
            required_usages,
            wgpu::TextureFormatFeatureFlags::empty()
        ));
        assert!(catalog_texture_material_supported(
            required_usages,
            required_flags
        ));
    }

    #[test]
    fn custom_effect_wgsl_size_limit_is_enforced() {
        let src = "x".repeat(MAX_CUSTOM_EFFECT_WGSL_BYTES + 1);
        assert!(matches!(
            build_and_validate_custom_effect_wgsl_v1(&src),
            Err(fret_core::CustomEffectRegistrationError::InvalidSource)
        ));
    }

    #[test]
    fn custom_effect_wgsl_empty_is_rejected() {
        assert!(matches!(
            build_and_validate_custom_effect_wgsl_v1(""),
            Err(fret_core::CustomEffectRegistrationError::InvalidSource)
        ));
    }

    #[test]
    fn custom_effect_wgsl_minimal_program_validates() {
        let src = r#"
 fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
   return src;
 }
 "#;
        build_and_validate_custom_effect_wgsl_v1(src).expect("expected valid custom effect WGSL");
    }

    #[test]
    fn custom_effect_v2_wgsl_minimal_program_validates() {
        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;
        build_and_validate_custom_effect_wgsl_v2(src)
            .expect("expected valid custom effect v2 WGSL");
    }

    #[test]
    fn custom_effect_v3_wgsl_minimal_program_validates() {
        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;
        build_and_validate_custom_effect_wgsl_v3(src)
            .expect("expected valid custom effect v3 WGSL");
    }

    #[test]
    fn unregister_custom_effect_evicts_custom_effect_pipelines() {
        let ctx = match pollster::block_on(crate::WgpuContext::new()) {
            Ok(ctx) => ctx,
            Err(_err) => return,
        };
        let mut renderer = crate::Renderer::new(&ctx.adapter, &ctx.device);

        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

        let id_v1 = renderer
            .register_custom_effect_v1(fret_core::CustomEffectDescriptorV1::wgsl_utf8(src))
            .expect("custom effect v1 registration must succeed on wgpu backends");
        let id_v2 = renderer
            .register_custom_effect_v2(fret_core::CustomEffectDescriptorV2::wgsl_utf8(src))
            .expect("custom effect v2 registration must succeed on wgpu backends");
        let id_v3 = renderer
            .register_custom_effect_v3(fret_core::CustomEffectDescriptorV3::wgsl_utf8(src))
            .expect("custom effect v3 registration must succeed on wgpu backends");

        renderer.ensure_custom_effect_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v1,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_pipelines
                .contains_key(&id_v1),
            "expected v1 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v1));
        assert!(
            !renderer
                .pipelines
                .custom_effect_pipelines
                .contains_key(&id_v1),
            "expected v1 pipelines to be evicted when the effect is unregistered"
        );

        renderer.ensure_custom_effect_v2_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v2,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_v2_pipelines
                .contains_key(&id_v2),
            "expected v2 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v2));
        assert!(
            !renderer
                .pipelines
                .custom_effect_v2_pipelines
                .contains_key(&id_v2),
            "expected v2 pipelines to be evicted when the effect is unregistered"
        );

        renderer.ensure_custom_effect_v3_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v3,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_v3_pipelines
                .contains_key(&id_v3),
            "expected v3 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v3));
        assert!(
            !renderer
                .pipelines
                .custom_effect_v3_pipelines
                .contains_key(&id_v3),
            "expected v3 pipelines to be evicted when the effect is unregistered"
        );
    }
}
