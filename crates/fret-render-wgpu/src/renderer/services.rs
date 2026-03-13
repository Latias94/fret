use super::*;
use std::collections::hash_map::Entry;

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
        self.path_state.prepare_path(commands, style, constraints)
    }

    fn release(&mut self, path: fret_core::PathId) {
        self.path_state.release_path(path);
    }
}

impl fret_core::SvgService for Renderer {
    fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
        self.svg_registry_state.register_svg(bytes)
    }

    fn unregister_svg(&mut self, svg: fret_core::SvgId) -> bool {
        match self.svg_registry_state.unregister_svg(svg) {
            svg::SvgRegistryUnregisterOutcome::Missing => return false,
            svg::SvgRegistryUnregisterOutcome::StillReferenced => return true,
            svg::SvgRegistryUnregisterOutcome::Removed => {}
        }

        // Drop any cached rasterizations for this SVG.
        let mut keys_to_remove: Vec<SvgRasterKey> = Vec::new();
        for k in self.svg_raster_state.rasters.keys() {
            if k.svg == svg {
                keys_to_remove.push(*k);
            }
        }
        for k in keys_to_remove {
            if let Some(entry) = self.svg_raster_state.rasters.remove(&k) {
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

        match self.material_effect_state.materials_by_desc.entry(desc) {
            Entry::Occupied(e) => {
                let id = *e.get();
                if let Some(entry) = self.material_effect_state.materials.get_mut(id) {
                    entry.refs = entry.refs.saturating_add(1);
                }
                Ok(id)
            }
            Entry::Vacant(e) => {
                let id = self
                    .material_effect_state
                    .materials
                    .insert(MaterialEntry { desc, refs: 1 });
                e.insert(id);
                self.material_effect_state.materials_generation = self
                    .material_effect_state
                    .materials_generation
                    .wrapping_add(1);
                Ok(id)
            }
        }
    }

    fn unregister_material(&mut self, id: fret_core::MaterialId) -> bool {
        let Some(refs) = self.material_effect_state.materials.get(id).map(|e| e.refs) else {
            return false;
        };

        if refs > 1 {
            if let Some(entry) = self.material_effect_state.materials.get_mut(id) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return true;
        }

        let Some(entry) = self.material_effect_state.materials.remove(id) else {
            return false;
        };

        self.material_effect_state
            .materials_by_desc
            .remove(&entry.desc);
        self.material_effect_state.materials_generation = self
            .material_effect_state
            .materials_generation
            .wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
