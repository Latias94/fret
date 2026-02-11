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
        let triangles = tessellate_path_commands(commands, style, constraints);
        let id = self.paths.insert(PreparedPath {
            metrics,
            triangles,
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

impl fret_core::MaterialService for Renderer {
    fn register_material(
        &mut self,
        desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        // v1: all baseline material kinds are supported by the quad shader on wgpu backends.
        //
        // Capability gating hooks can be added once we have a backend that cannot support a given
        // kind under the fixed Tier B binding shape (ADR 0122).
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
        true
    }
}
