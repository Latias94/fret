use super::super::*;

impl Renderer {
    pub(in crate::renderer) fn bump_svg_raster_epoch(&mut self) -> u64 {
        self.svg_raster_epoch = self.svg_raster_epoch.wrapping_add(1);
        self.svg_raster_epoch
    }

    pub(in crate::renderer) fn prune_svg_rasters(&mut self) {
        if self.svg_raster_bytes <= self.svg_raster_budget_bytes {
            return;
        }

        // Best-effort eviction: never evict entries used in the current frame.
        let cur_epoch = self.svg_raster_epoch;

        while self.svg_raster_bytes > self.svg_raster_budget_bytes {
            let mut victim_standalone: Option<(SvgRasterKey, u64)> = None;
            for (k, v) in &self.svg_rasters {
                if v.last_used_epoch == cur_epoch {
                    continue;
                }
                if !matches!(&v.storage, SvgRasterStorage::Standalone { .. }) {
                    continue;
                }
                match victim_standalone {
                    None => victim_standalone = Some((*k, v.last_used_epoch)),
                    Some((_, best_epoch)) => {
                        if v.last_used_epoch < best_epoch {
                            victim_standalone = Some((*k, v.last_used_epoch));
                        }
                    }
                }
            }

            let Some((victim_key, _)) = victim_standalone else {
                // Cache is over budget but all standalone entries were used this frame (or there
                // are no standalone entries). Keep correctness and allow a temporary overshoot.
                break;
            };

            if let Some(entry) = self.svg_rasters.remove(&victim_key) {
                if self.perf_enabled {
                    self.perf_svg_raster_budget_evictions =
                        self.perf_svg_raster_budget_evictions.saturating_add(1);
                }
                self.drop_svg_raster_entry(entry);
            } else {
                break;
            }
        }
    }

    pub(in crate::renderer) fn drop_svg_raster_entry(&mut self, entry: SvgRasterEntry) {
        match entry.storage {
            SvgRasterStorage::Standalone { .. } => {
                self.svg_raster_bytes = self.svg_raster_bytes.saturating_sub(entry.approx_bytes);
                let _ = self.unregister_image(entry.image);
            }
            SvgRasterStorage::MaskAtlas {
                page_index,
                alloc_id,
            } => {
                let Some(page) = self
                    .svg_mask_atlas_pages
                    .get_mut(page_index)
                    .and_then(|p| p.as_mut())
                else {
                    return;
                };
                page.allocator.deallocate(alloc_id);
                page.entries = page.entries.saturating_sub(1);
                if page.entries == 0 {
                    self.evict_svg_mask_atlas_page(page_index);
                }
            }
        }
    }
}
