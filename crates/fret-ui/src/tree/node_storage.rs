use super::*;
use std::any::{Any, TypeId};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct ViewCacheFlags {
    pub(super) enabled: bool,
    pub(super) contained_layout: bool,
    /// Whether the cache root's own box is layout-definite (i.e. it does not size-to-content).
    ///
    /// This is used to decide whether layout/hit-test invalidations can be truncated at the cache
    /// root when view caching is active. Auto-sized cache roots must allow invalidations to reach
    /// ancestors so the root can be placed before running contained relayouts.
    pub(super) layout_definite: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeMeasureCacheKey {
    pub(super) known_w_bits: Option<u32>,
    pub(super) known_h_bits: Option<u32>,
    pub(super) avail_w: (u8, u32),
    pub(super) avail_h: (u8, u32),
    pub(super) scale_bits: u32,
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

pub(super) struct Node<H: UiHost> {
    pub(super) widget: Option<Box<dyn Widget<H>>>,
    pub(super) element: Option<GlobalElementId>,
    pub(super) parent: Option<NodeId>,
    pub(super) children: Vec<NodeId>,
    pub(super) bounds: Rect,
    pub(super) bounds_written_paint_pass: u64,
    pub(super) measured_size: Size,
    pub(super) measure_cache: Option<NodeMeasureCache>,
    pub(super) text_wrap_none_measure_cache: Option<TextWrapNoneMeasureCache>,
    pub(super) invalidation: InvalidationFlags,
    pub(super) subtree_layout_dirty_count: u32,
    pub(super) paint_invalidated_by_hit_test_only: bool,
    pub(super) paint_cache: Option<PaintCacheEntry>,
    pub(super) interaction_cache: Option<prepaint::InteractionCacheEntry>,
    pub(super) prepaint_outputs: PrepaintOutputs,
    pub(super) prepaint_hit_test: Option<PrepaintHitTestCache>,
    pub(super) view_cache: ViewCacheFlags,
    pub(super) view_cache_needs_rerender: bool,
    pub(super) text_boundary_mode_override: Option<fret_runtime::TextBoundaryMode>,
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

#[derive(Default)]
pub(super) struct PrepaintOutputs {
    key: Option<PaintCacheKey>,
    values: Vec<(TypeId, Box<dyn Any>)>,
}

impl PrepaintOutputs {
    pub(super) fn begin_frame(&mut self, key: PaintCacheKey) {
        if self.key != Some(key) {
            self.key = Some(key);
            self.values.clear();
        }
    }

    pub(super) fn set<T: Any>(&mut self, value: T) {
        let ty = TypeId::of::<T>();
        if let Some((_, existing)) = self.values.iter_mut().find(|(id, _)| *id == ty) {
            *existing = Box::new(value);
            return;
        }
        self.values.push((ty, Box::new(value)));
    }

    pub(super) fn get<T: Any>(&self) -> Option<&T> {
        let ty = TypeId::of::<T>();
        self.values
            .iter()
            .find(|(id, _)| *id == ty)
            .and_then(|(_, value)| value.downcast_ref::<T>())
    }

    pub(super) fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let ty = TypeId::of::<T>();
        self.values
            .iter_mut()
            .find(|(id, _)| *id == ty)
            .and_then(|(_, value)| value.downcast_mut::<T>())
    }
}

impl<H: UiHost> Node<H> {
    pub(super) fn new(widget: impl Widget<H> + 'static) -> Self {
        Self {
            widget: Some(Box::new(widget)),
            element: None,
            parent: None,
            children: Vec::new(),
            bounds: Rect::default(),
            bounds_written_paint_pass: 0,
            measured_size: Size::default(),
            measure_cache: None,
            text_wrap_none_measure_cache: None,
            invalidation: InvalidationFlags {
                layout: true,
                paint: true,
                hit_test: true,
            },
            subtree_layout_dirty_count: 1,
            paint_invalidated_by_hit_test_only: false,
            paint_cache: None,
            interaction_cache: None,
            prepaint_outputs: PrepaintOutputs::default(),
            prepaint_hit_test: None,
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
