use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct ViewCacheFlags {
    pub(super) enabled: bool,
    pub(super) parent_layout_dependency: ViewCacheParentLayoutDependency,
    /// Whether the cache root's own box is layout-definite (i.e. it does not size-to-content).
    ///
    /// This is used to decide whether layout/hit-test invalidations can be truncated at the cache
    /// root when view caching is active. Auto-sized cache roots must allow invalidations to reach
    /// ancestors so the root can be placed before running contained relayouts.
    pub(super) layout_definite: bool,
}

impl ViewCacheFlags {
    pub(super) fn from_contain_layout_when_bounds_known(
        enabled: bool,
        contain_layout_when_bounds_known: bool,
        layout_definite: bool,
    ) -> Self {
        Self {
            enabled,
            parent_layout_dependency:
                ViewCacheParentLayoutDependency::from_contain_layout_when_bounds_known(
                    contain_layout_when_bounds_known,
                ),
            layout_definite,
        }
    }

    pub(super) fn layout_contained_when_bounds_known(self) -> bool {
        self.parent_layout_dependency == ViewCacheParentLayoutDependency::ContainedWhenBoundsKnown
    }

    #[cfg(test)]
    pub(crate) fn test_set_layout_contained_when_bounds_known(&mut self, value: bool) {
        self.parent_layout_dependency =
            ViewCacheParentLayoutDependency::from_contain_layout_when_bounds_known(value);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum ViewCacheParentLayoutDependency {
    #[default]
    ParentDependent,
    ContainedWhenBoundsKnown,
}

impl ViewCacheParentLayoutDependency {
    fn from_contain_layout_when_bounds_known(value: bool) -> Self {
        if value {
            Self::ContainedWhenBoundsKnown
        } else {
            Self::ParentDependent
        }
    }

    pub(super) fn as_debug_str(self) -> &'static str {
        match self {
            Self::ParentDependent => "parent_dependent",
            Self::ContainedWhenBoundsKnown => "contained_when_bounds_known",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChildrenWritePolicy {
    #[default]
    Standard,
    Barrier,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeMeasureCacheKey {
    pub(super) known_w_bits: Option<u32>,
    pub(super) known_h_bits: Option<u32>,
    pub(super) avail_w: (u8, u32),
    pub(super) avail_h: (u8, u32),
    pub(super) scale_bits: u32,
    pub(super) text_style_present: bool,
    pub(super) text_style_fingerprint: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NodeMeasureCache {
    pub(super) key: NodeMeasureCacheKey,
    pub(super) size: Size,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TextWrapNoneMeasureCache {
    pub(super) fingerprint: u64,
    pub(super) size: Size,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TextWrappedMeasureCache {
    pub(super) fingerprint: u64,
    pub(super) constraints_max_width: Option<Px>,
    pub(super) measured_size: Size,
    pub(super) clamped_size: Size,
}

#[derive(Debug, Default, Clone)]
pub(super) struct TextWrappedMeasureCaches {
    entries: Vec<TextWrappedMeasureCache>,
}

impl TextWrappedMeasureCaches {
    const MAX_ENTRIES: usize = 8;

    pub(super) fn get(
        &self,
        fingerprint: u64,
        constraints_max_width: Option<Px>,
    ) -> Option<TextWrappedMeasureCache> {
        self.entries.iter().rev().copied().find(|cache| {
            cache.fingerprint == fingerprint && cache.constraints_max_width == constraints_max_width
        })
    }

    pub(super) fn latest_for_fingerprint(
        &self,
        fingerprint: u64,
    ) -> Option<TextWrappedMeasureCache> {
        self.entries
            .iter()
            .rev()
            .copied()
            .find(|cache| cache.fingerprint == fingerprint)
    }

    pub(super) fn insert(&mut self, entry: TextWrappedMeasureCache) {
        self.entries
            .retain(|cache| cache.fingerprint == entry.fingerprint);

        if let Some(index) = self
            .entries
            .iter()
            .position(|cache| cache.constraints_max_width == entry.constraints_max_width)
        {
            self.entries.remove(index);
        }

        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }
}

pub(super) struct Node<H: UiHost> {
    pub(super) widget: Option<Box<dyn Widget<H>>>,
    pub(super) element: Option<GlobalElementId>,
    pub(super) parent: Option<NodeId>,
    pub(super) children: Vec<NodeId>,
    pub(super) children_write_policy: ChildrenWritePolicy,
    pub(super) bounds: Rect,
    pub(super) bounds_written_paint_pass: u64,
    pub(super) measured_size: Size,
    pub(super) paint_geometry_fingerprint: u64,
    pub(super) inherited_text_style_fingerprint: Option<u64>,
    pub(super) paint_passthrough: Option<NodePaintPassthrough>,
    pub(super) measure_cache: Option<NodeMeasureCache>,
    pub(super) text_wrap_none_measure_cache: Option<TextWrapNoneMeasureCache>,
    pub(super) text_wrapped_measure_cache: Option<TextWrappedMeasureCaches>,
    pub(super) invalidation: InvalidationFlags,
    pub(super) semantics_dirty: bool,
    pub(super) subtree_semantics_dirty_count: u32,
    pub(super) subtree_layout_dirty_count: u32,
    pub(super) layout_dirty_children_suppressed: bool,
    pub(super) paint_invalidated_by_hit_test_only: bool,
    pub(super) interaction_cache: Option<prepaint::InteractionCacheEntry>,
    pub(super) prepaint_hit_test: Option<PrepaintHitTestCache>,
    pub(super) widget_prepaint_enabled: bool,
    pub(super) view_cache: ViewCacheFlags,
    pub(super) view_cache_needs_rerender: bool,
    pub(super) text_boundary_mode_override: Option<fret_runtime::TextBoundaryMode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrapped_cache_entry(fingerprint: u64, width: f32, height: f32) -> TextWrappedMeasureCache {
        TextWrappedMeasureCache {
            fingerprint,
            constraints_max_width: Some(Px(width)),
            measured_size: Size::new(Px(width), Px(height)),
            clamped_size: Size::new(Px(width), Px(height)),
        }
    }

    #[test]
    fn text_wrapped_measure_cache_keeps_multiple_widths_for_same_fingerprint() {
        let mut caches = TextWrappedMeasureCaches::default();
        caches.insert(wrapped_cache_entry(7, 100.0, 20.0));
        caches.insert(wrapped_cache_entry(7, 200.0, 30.0));

        assert_eq!(
            caches
                .get(7, Some(Px(100.0)))
                .map(|entry| entry.measured_size.height),
            Some(Px(20.0))
        );
        assert_eq!(
            caches
                .get(7, Some(Px(200.0)))
                .map(|entry| entry.measured_size.height),
            Some(Px(30.0))
        );
    }

    #[test]
    fn text_wrapped_measure_cache_replaces_width_and_evicts_oldest() {
        let mut caches = TextWrappedMeasureCaches::default();
        caches.insert(wrapped_cache_entry(7, 100.0, 20.0));
        caches.insert(wrapped_cache_entry(7, 100.0, 24.0));

        assert_eq!(caches.entries.len(), 1);
        assert_eq!(
            caches
                .get(7, Some(Px(100.0)))
                .map(|entry| entry.measured_size.height),
            Some(Px(24.0))
        );

        for index in 0..TextWrappedMeasureCaches::MAX_ENTRIES {
            caches.insert(wrapped_cache_entry(7, 200.0 + index as f32, 40.0));
        }

        assert_eq!(caches.entries.len(), TextWrappedMeasureCaches::MAX_ENTRIES);
        assert!(caches.get(7, Some(Px(100.0))).is_none());
    }

    #[test]
    fn text_wrapped_measure_cache_drops_stale_fingerprints() {
        let mut caches = TextWrappedMeasureCaches::default();
        caches.insert(wrapped_cache_entry(7, 100.0, 20.0));
        caches.insert(wrapped_cache_entry(8, 100.0, 30.0));

        assert!(caches.get(7, Some(Px(100.0))).is_none());
        assert_eq!(
            caches
                .latest_for_fingerprint(8)
                .map(|entry| entry.measured_size.height),
            Some(Px(30.0))
        );
    }
}

#[derive(Debug, Clone)]
pub(super) struct HitTestPathCache {
    pub(super) layer_root: NodeId,
    pub(super) path: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PrepaintHitTestCache {
    pub(super) render_transform_inv: Option<Transform2D>,
    pub(super) children_render_transform_inv: Option<Transform2D>,
    pub(super) clips_hit_test: bool,
    pub(super) clip_hit_test_corner_radii: Option<Corners>,
    pub(super) is_focusable: bool,
    pub(super) focus_traversal_children: bool,
    pub(super) can_scroll_descendant_into_view: bool,
}

impl<H: UiHost> Node<H> {
    pub(super) fn new(widget: impl Widget<H> + 'static) -> Self {
        Self {
            widget: Some(Box::new(widget)),
            element: None,
            parent: None,
            children: Vec::new(),
            children_write_policy: ChildrenWritePolicy::Standard,
            bounds: Rect::default(),
            bounds_written_paint_pass: 0,
            measured_size: Size::default(),
            paint_geometry_fingerprint: 0,
            inherited_text_style_fingerprint: None,
            paint_passthrough: None,
            measure_cache: None,
            text_wrap_none_measure_cache: None,
            text_wrapped_measure_cache: None,
            invalidation: InvalidationFlags {
                layout: true,
                paint: true,
                hit_test: true,
            },
            semantics_dirty: true,
            subtree_semantics_dirty_count: 1,
            subtree_layout_dirty_count: 1,
            layout_dirty_children_suppressed: false,
            paint_invalidated_by_hit_test_only: false,
            interaction_cache: None,
            prepaint_hit_test: None,
            widget_prepaint_enabled: false,
            view_cache: ViewCacheFlags::default(),
            view_cache_needs_rerender: false,
            text_boundary_mode_override: None,
        }
    }

    #[cfg(test)]
    pub(super) fn new_for_element(
        element: GlobalElementId,
        widget: impl Widget<H> + 'static,
    ) -> Self {
        Self {
            element: Some(element),
            ..Self::new(widget)
        }
    }
}
