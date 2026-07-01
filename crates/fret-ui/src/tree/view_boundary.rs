use super::*;
use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::Arc;

pub trait BoundarySceneFragmentDebug: Any {
    fn boundary_scene_fragment_entry_count(&self) -> usize;

    fn boundary_scene_fragment_chunk_count(&self) -> usize {
        0
    }

    fn boundary_scene_fragment_fingerprint(&self) -> u64 {
        0
    }

    fn append_boundary_scene_fragment_chunks(&self, _out: &mut BoundarySceneChunkManifest) {}
}

#[derive(Debug, Clone)]
pub struct BoundarySceneFragmentChunk {
    chunk: fret_core::SceneChunk,
    local_bounds: Rect,
    scene_origin: Point,
}

impl BoundarySceneFragmentChunk {
    pub fn new(chunk: fret_core::SceneChunk, local_bounds: Rect, scene_origin: Point) -> Self {
        Self {
            chunk,
            local_bounds,
            scene_origin,
        }
    }

    pub fn chunk(&self) -> &fret_core::SceneChunk {
        &self.chunk
    }

    pub fn local_bounds(&self) -> Rect {
        self.local_bounds
    }

    pub fn scene_origin(&self) -> Point {
        self.scene_origin
    }

    pub fn fingerprint(&self) -> u64 {
        self.chunk.fingerprint()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoundarySceneChunkManifest {
    chunks: Vec<BoundarySceneFragmentChunk>,
}

impl BoundarySceneChunkManifest {
    pub fn push(&mut self, chunk: BoundarySceneFragmentChunk) {
        if !chunk.chunk.is_empty() {
            self.chunks.push(chunk);
        }
    }

    pub fn chunks(&self) -> &[BoundarySceneFragmentChunk] {
        &self.chunks
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn fingerprint(&self) -> u64 {
        self.chunks
            .iter()
            .fold(0, |fingerprint, chunk| fingerprint ^ chunk.fingerprint())
    }

    pub fn append_to_scene_chunk_manifest(&self, out: &mut fret_core::SceneChunkManifest) {
        for chunk in &self.chunks {
            out.push(fret_core::SceneChunkManifestEntry::new(
                chunk.chunk().clone(),
                chunk.local_bounds(),
                chunk.scene_origin(),
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BoundaryId(NodeId);

impl BoundaryId {
    pub(crate) fn from_node(node: NodeId) -> Self {
        Self(node)
    }

    pub(super) fn node(self) -> NodeId {
        self.0
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct DirtyViewFrontier {
    views: HashSet<ViewId>,
}

impl DirtyViewFrontier {
    pub(super) fn mark_boundary_node_v1(&mut self, node: NodeId) -> bool {
        self.views.insert(ViewId::from(node))
    }

    pub(super) fn clear_boundary_node_v1(&mut self, node: NodeId) -> bool {
        self.views.remove(&ViewId::from(node))
    }

    #[cfg(test)]
    pub(super) fn contains_boundary_node_v1(&self, node: NodeId) -> bool {
        self.views.contains(&ViewId::from(node))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.views.len()
    }

    pub(super) fn iter_views(&self) -> impl Iterator<Item = ViewId> + '_ {
        self.views.iter().copied()
    }

    pub(super) fn iter_boundary_nodes_v1(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.iter_views().map(NodeId::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ViewBoundaryKind {
    Node,
    ViewCacheRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoundaryParentLayoutDependency {
    ParentDependent,
    ContainedWhenBoundsKnown,
}

impl BoundaryParentLayoutDependency {
    fn as_debug_str(self) -> &'static str {
        match self {
            Self::ParentDependent => "parent_dependent",
            Self::ContainedWhenBoundsKnown => "contained_when_bounds_known",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BoundaryLayoutDependencies {
    pub(super) parent: BoundaryParentLayoutDependency,
    pub(super) layout_definite: bool,
}

impl BoundaryLayoutDependencies {
    fn from_view_cache_flags(flags: ViewCacheFlags) -> Self {
        let parent = if flags.layout_contained_when_bounds_known() {
            BoundaryParentLayoutDependency::ContainedWhenBoundsKnown
        } else {
            BoundaryParentLayoutDependency::ParentDependent
        };
        Self {
            parent,
            layout_definite: flags.layout_definite,
        }
    }

    pub(super) fn allows_contained_relayout(self) -> bool {
        self.parent == BoundaryParentLayoutDependency::ContainedWhenBoundsKnown
            && self.layout_definite
    }
}

pub(super) struct ViewBoundaryState {
    pub(super) id: BoundaryId,
    pub(super) parent: Option<BoundaryId>,
    pub(super) kind: ViewBoundaryKind,
    pub(super) layout_dependencies: BoundaryLayoutDependencies,
    pub(super) frame_products: BoundaryFrameProducts,
}

#[derive(Default)]
pub(super) struct BoundaryFrameProducts {
    pub(super) dirty: BoundaryDirtyState,
    pub(super) prepaint: BoundaryPrepaintState,
    pub(super) hit_test_bounds: bounds_tree::BoundaryHitTestBoundsState,
    pub(super) semantics: BoundarySemanticsState,
    pub(super) interaction_cache: BoundaryInteractionCacheState,
    pub(super) scene_fragment: BoundarySceneFragmentState,
    pub(super) paint_cache: BoundaryPaintCacheState,
}

impl ViewBoundaryState {
    fn new_runtime(id: BoundaryId, parent: Option<BoundaryId>, flags: ViewCacheFlags) -> Self {
        Self {
            id,
            parent,
            kind: if flags.enabled {
                ViewBoundaryKind::ViewCacheRoot
            } else {
                ViewBoundaryKind::Node
            },
            layout_dependencies: BoundaryLayoutDependencies::from_view_cache_flags(flags),
            frame_products: BoundaryFrameProducts::default(),
        }
    }

    fn refresh_runtime(&mut self, parent: Option<BoundaryId>, flags: ViewCacheFlags) {
        self.parent = parent;
        self.kind = if flags.enabled {
            ViewBoundaryKind::ViewCacheRoot
        } else {
            ViewBoundaryKind::Node
        };
        self.layout_dependencies = BoundaryLayoutDependencies::from_view_cache_flags(flags);
    }
}

#[derive(Default)]
pub(in crate::tree) struct BoundaryPaintCacheState {
    entry: Option<PaintCacheEntry>,
}

#[derive(Default)]
pub(in crate::tree) struct BoundaryInteractionCacheState {
    entry: Option<prepaint::InteractionCacheEntry>,
}

#[derive(Default)]
pub(in crate::tree) struct BoundarySemanticsState {
    snapshot: Option<Arc<SemanticsSnapshot>>,
    range: Option<(usize, usize)>,
}

#[derive(Default)]
pub(in crate::tree) struct PaintCacheEntryState {
    entry: Option<PaintCacheEntry>,
}

impl BoundaryInteractionCacheState {
    pub(super) fn entry(&self) -> Option<prepaint::InteractionCacheEntry> {
        self.entry
    }

    pub(super) fn set_entry(&mut self, entry: prepaint::InteractionCacheEntry) {
        self.entry = Some(entry);
    }

    pub(super) fn has_entry(&self) -> bool {
        self.entry.is_some()
    }
}

impl BoundaryPaintCacheState {
    pub(super) fn entry(&self) -> Option<PaintCacheEntry> {
        self.entry
    }

    pub(super) fn set_entry(&mut self, entry: PaintCacheEntry) {
        self.entry = Some(entry);
    }

    pub(super) fn clear(&mut self) {
        self.entry = None;
    }

    pub(super) fn translate_origin(&mut self, delta: Point) {
        if let Some(entry) = &mut self.entry {
            entry.origin = Point::new(entry.origin.x + delta.x, entry.origin.y + delta.y);
        }
    }

    pub(super) fn has_entry(&self) -> bool {
        self.entry.is_some()
    }
}

impl BoundarySemanticsState {
    pub(super) fn set_subtree(
        &mut self,
        snapshot: Arc<SemanticsSnapshot>,
        start: usize,
        end: usize,
    ) {
        if start >= end || end > snapshot.nodes.len() {
            self.clear();
            return;
        }
        self.snapshot = Some(snapshot);
        self.range = Some((start, end));
    }

    pub(super) fn clear(&mut self) {
        self.snapshot = None;
        self.range = None;
    }

    pub(super) fn reuse_subtree(
        &self,
        parent: Option<NodeId>,
        bounds: Rect,
        nodes: &mut Vec<SemanticsNode>,
    ) -> bool {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return false;
        };
        let Some((start, end)) = self.range else {
            return false;
        };
        let Some(range) = snapshot.nodes.get(start..end) else {
            return false;
        };
        let Some(previous_root) = range.first() else {
            return false;
        };
        if previous_root.parent != parent {
            return false;
        }

        if previous_root.bounds == bounds {
            nodes.extend(range.iter().cloned());
            return true;
        }

        if previous_root.bounds.size == bounds.size {
            let dx = bounds.origin.x - previous_root.bounds.origin.x;
            let dy = bounds.origin.y - previous_root.bounds.origin.y;
            nodes.extend(range.iter().cloned().map(|mut reused| {
                reused.bounds.origin =
                    Point::new(reused.bounds.origin.x + dx, reused.bounds.origin.y + dy);
                reused
            }));
            return true;
        }

        false
    }

    pub(super) fn has_subtree(&self) -> bool {
        self.snapshot.is_some() && self.range.is_some()
    }
}

impl PaintCacheEntryState {
    pub(super) fn entry(&self) -> Option<PaintCacheEntry> {
        self.entry
    }

    pub(super) fn set_entry(&mut self, entry: PaintCacheEntry) {
        self.entry = Some(entry);
    }

    pub(super) fn take_entry(&mut self) -> Option<PaintCacheEntry> {
        self.entry.take()
    }

    pub(super) fn translate_origin(&mut self, delta: Point) {
        if let Some(entry) = &mut self.entry {
            entry.origin = Point::new(entry.origin.x + delta.x, entry.origin.y + delta.y);
        }
    }

    #[cfg(test)]
    pub(super) fn has_entry(&self) -> bool {
        self.entry.is_some()
    }
}

#[derive(Default)]
pub(super) struct BoundaryDirtyState {
    reason: Option<(UiDebugInvalidationSource, UiDebugInvalidationDetail)>,
}

impl BoundaryDirtyState {
    pub(super) fn mark(
        &mut self,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        self.reason = Some((source, detail));
    }

    pub(super) fn clear(&mut self) {
        self.reason = None;
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.reason.is_some()
    }

    pub(super) fn reason(&self) -> Option<(UiDebugInvalidationSource, UiDebugInvalidationDetail)> {
        self.reason
    }
}

#[derive(Default)]
pub(super) struct BoundaryPrepaintState {
    outputs: BoundaryTypedOutputs,
}

impl BoundaryPrepaintState {
    pub(super) fn begin_outputs(&mut self, key: PaintCacheKey) {
        self.outputs.begin_frame(key);
    }

    pub(super) fn set_output<T: Any>(&mut self, value: T) {
        self.outputs.set(value);
    }

    pub(super) fn set_output_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.outputs.set_box(ty, value);
    }

    pub(super) fn output<T: Any>(&self) -> Option<&T> {
        self.outputs.get::<T>()
    }

    pub(super) fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.outputs.get_mut::<T>()
    }

    pub(super) fn output_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.outputs.get_any(ty)
    }

    pub(super) fn output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.outputs.get_any_mut(ty)
    }
}

#[derive(Default)]
pub(super) struct BoundarySceneFragmentState {
    outputs: BoundaryTypedOutputs,
    used_entries: usize,
    rejected_entries: usize,
    last_reject_reason: Option<&'static str>,
}

impl BoundarySceneFragmentState {
    pub(super) fn begin_fragment(&mut self, key: PaintCacheKey) {
        if self.outputs.begin_frame(key) {
            self.used_entries = 0;
            self.rejected_entries = 0;
            self.last_reject_reason = None;
        }
    }

    pub(super) fn set_fragment<T: Any>(&mut self, value: T) {
        self.outputs.set(value);
    }

    pub(super) fn set_fragment_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.outputs.set_box(ty, value);
    }

    pub(super) fn set_fragment_with_debug_metadata<T: Any>(
        &mut self,
        value: T,
        metadata: BoundaryTypedOutputDebugMetadata,
    ) {
        self.outputs.set_with_debug_metadata(value, metadata);
    }

    pub(super) fn set_fragment_box_with_debug_metadata(
        &mut self,
        ty: TypeId,
        value: Box<dyn Any>,
        metadata: BoundaryTypedOutputDebugMetadata,
    ) {
        self.outputs
            .set_box_with_debug_metadata(ty, value, metadata);
    }

    pub(super) fn fragment<T: Any>(&self) -> Option<&T> {
        self.outputs.get::<T>()
    }

    pub(super) fn fragment_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.outputs.get_mut::<T>()
    }

    pub(super) fn fragment_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.outputs.get_any(ty)
    }

    pub(super) fn fragment_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.outputs.get_any_mut(ty)
    }

    pub(super) fn slot_count(&self) -> usize {
        self.outputs.len()
    }

    pub(super) fn entry_count(&self) -> usize {
        self.outputs.entry_count()
    }

    pub(super) fn chunk_count(&self) -> usize {
        self.outputs.chunk_count()
    }

    pub(super) fn fingerprint(&self) -> u64 {
        self.outputs.fingerprint()
    }

    pub(super) fn chunk_manifest(&self) -> BoundarySceneChunkManifest {
        self.outputs.chunk_manifest()
    }

    pub(super) fn record_used_entries(&mut self, count: usize) {
        self.used_entries = self.used_entries.saturating_add(count);
    }

    pub(super) fn record_rejected_entries(&mut self, count: usize, reason: &'static str) {
        self.rejected_entries = self.rejected_entries.saturating_add(count);
        self.last_reject_reason = Some(reason);
    }

    pub(super) fn used_entries(&self) -> usize {
        self.used_entries
    }

    pub(super) fn rejected_entries(&self) -> usize {
        self.rejected_entries
    }

    pub(super) fn last_reject_reason(&self) -> Option<&'static str> {
        self.last_reject_reason
    }
}

#[derive(Default)]
struct BoundaryTypedOutputs {
    key: Option<PaintCacheKey>,
    values: Vec<(TypeId, Box<dyn Any>, BoundaryTypedOutputDebugMetadata)>,
}

#[derive(Debug, Default, Clone)]
pub(in crate::tree) struct BoundaryTypedOutputDebugMetadata {
    pub(in crate::tree) entry_count: usize,
    pub(in crate::tree) chunk_count: usize,
    pub(in crate::tree) fingerprint: u64,
    pub(in crate::tree) chunks: BoundarySceneChunkManifest,
}

impl BoundaryTypedOutputs {
    fn begin_frame(&mut self, key: PaintCacheKey) -> bool {
        if self.key != Some(key) {
            self.key = Some(key);
            self.values.clear();
            true
        } else {
            false
        }
    }

    fn set<T: Any>(&mut self, value: T) {
        self.set_box(TypeId::of::<T>(), Box::new(value));
    }

    fn set_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.set_box_with_debug_metadata(ty, value, BoundaryTypedOutputDebugMetadata::default());
    }

    fn set_with_debug_metadata<T: Any>(
        &mut self,
        value: T,
        metadata: BoundaryTypedOutputDebugMetadata,
    ) {
        self.set_box_with_debug_metadata(TypeId::of::<T>(), Box::new(value), metadata);
    }

    fn set_box_with_debug_metadata(
        &mut self,
        ty: TypeId,
        value: Box<dyn Any>,
        metadata: BoundaryTypedOutputDebugMetadata,
    ) {
        if let Some((_, existing, existing_metadata)) =
            self.values.iter_mut().find(|(id, _, _)| *id == ty)
        {
            *existing = value;
            *existing_metadata = metadata;
            return;
        }
        self.values.push((ty, value, metadata));
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.get_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.get_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    fn get_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.values
            .iter()
            .find(|(id, _, _)| *id == ty)
            .map(|(_, value, _)| value.as_ref())
    }

    fn get_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.values
            .iter_mut()
            .find(|(id, _, _)| *id == ty)
            .map(|(_, value, _)| value.as_mut())
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn entry_count(&self) -> usize {
        self.values
            .iter()
            .map(|(_, _, metadata)| metadata.entry_count)
            .sum()
    }

    fn chunk_count(&self) -> usize {
        let manifest = self.chunk_manifest();
        if !manifest.is_empty() {
            return manifest.len();
        }
        self.values
            .iter()
            .map(|(_, _, metadata)| metadata.chunk_count)
            .sum()
    }

    fn fingerprint(&self) -> u64 {
        self.values.iter().fold(0, |fingerprint, (_, _, metadata)| {
            fingerprint ^ metadata.fingerprint
        })
    }

    fn chunk_manifest(&self) -> BoundarySceneChunkManifest {
        let mut manifest = BoundarySceneChunkManifest::default();
        for (_, _, metadata) in &self.values {
            for chunk in metadata.chunks.chunks() {
                manifest.push(chunk.clone());
            }
        }
        manifest
    }
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn ensure_view_boundary_state(
        &mut self,
        node: NodeId,
    ) -> Option<&mut ViewBoundaryState> {
        let flags = self.nodes.get(node)?.view_cache;
        let parent = self.nearest_parent_view_boundary(node);

        if !self.view_boundaries.contains_key(node) {
            self.view_boundaries.insert(
                node,
                ViewBoundaryState::new_runtime(BoundaryId::from_node(node), parent, flags),
            );
        }

        let state = self.view_boundaries.get_mut(node)?;
        state.refresh_runtime(parent, flags);
        if let Some(entry) = self
            .retained_paint_cache_entries
            .remove(node)
            .and_then(|mut state| state.take_entry())
        {
            state.frame_products.paint_cache.set_entry(entry);
        }
        Some(state)
    }

    pub(in crate::tree) fn sync_view_boundary_state_for_node(&mut self, node: NodeId) {
        if self.nodes.get(node).is_none() {
            self.view_boundaries.remove(node);
            self.retained_paint_cache_entries.remove(node);
            return;
        }

        let should_be_runtime_boundary = self
            .nodes
            .get(node)
            .is_some_and(|n| n.view_cache.enabled || n.widget_prepaint_enabled);

        if self.view_boundaries.contains_key(node) || should_be_runtime_boundary {
            let _ = self.ensure_view_boundary_state(node);
        }
    }

    pub(in crate::tree) fn remove_view_boundary_state(&mut self, node: NodeId) {
        self.view_boundaries.remove(node);
        self.retained_paint_cache_entries.remove(node);
        self.dirty_view_frontier.clear_boundary_node_v1(node);
    }

    pub(in crate::tree) fn paint_cache_entry_for_node(
        &self,
        node: NodeId,
    ) -> Option<PaintCacheEntry> {
        self.view_boundaries
            .get(node)
            .and_then(|state| state.frame_products.paint_cache.entry())
            .or_else(|| {
                self.retained_paint_cache_entries
                    .get(node)
                    .and_then(PaintCacheEntryState::entry)
            })
    }

    pub(in crate::tree) fn set_paint_cache_entry_for_node(
        &mut self,
        node: NodeId,
        entry: PaintCacheEntry,
    ) {
        if self.nodes.get(node).is_none() {
            return;
        }
        if let Some(boundary) = self.view_boundaries.get_mut(node) {
            boundary.frame_products.paint_cache.set_entry(entry);
            return;
        }
        if !self.retained_paint_cache_entries.contains_key(node) {
            self.retained_paint_cache_entries
                .insert(node, PaintCacheEntryState::default());
        }
        if let Some(state) = self.retained_paint_cache_entries.get_mut(node) {
            state.set_entry(entry);
        }
    }

    pub(in crate::tree) fn clear_paint_cache_entry_for_node(&mut self, node: NodeId) {
        if let Some(boundary) = self.view_boundaries.get_mut(node) {
            boundary.frame_products.paint_cache.clear();
        } else {
            self.retained_paint_cache_entries.remove(node);
        }
    }

    pub(in crate::tree) fn translate_paint_cache_entry_origin(
        &mut self,
        node: NodeId,
        delta: Point,
    ) {
        if let Some(boundary) = self.view_boundaries.get_mut(node) {
            boundary.frame_products.paint_cache.translate_origin(delta);
        } else if let Some(state) = self.retained_paint_cache_entries.get_mut(node) {
            state.translate_origin(delta);
        }
    }

    pub(in crate::tree) fn interaction_cache_entry_for_boundary(
        &self,
        node: NodeId,
    ) -> Option<prepaint::InteractionCacheEntry> {
        self.view_boundaries
            .get(node)
            .and_then(|state| state.frame_products.interaction_cache.entry())
    }

    pub(in crate::tree) fn set_interaction_cache_entry_for_boundary(
        &mut self,
        node: NodeId,
        entry: prepaint::InteractionCacheEntry,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.frame_products.interaction_cache.set_entry(entry);
    }

    pub(in crate::tree) fn mark_boundary_layout_dirty(
        &mut self,
        node: NodeId,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.frame_products.dirty.mark(source, detail);
        self.dirty_view_frontier.mark_boundary_node_v1(node);
        self.debug_refresh_dirty_frontier_max();
    }

    pub(in crate::tree) fn clear_boundary_layout_dirty(&mut self, node: NodeId) {
        if let Some(boundary) = self.view_boundaries.get_mut(node) {
            boundary.frame_products.dirty.clear();
        }
        self.dirty_view_frontier.clear_boundary_node_v1(node);
        self.debug_refresh_dirty_frontier_max();
    }

    #[cfg(test)]
    pub(in crate::tree) fn boundary_layout_dirty(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.frame_products.dirty.is_dirty())
    }

    pub(in crate::tree) fn boundary_layout_dirty_reason(
        &self,
        node: NodeId,
    ) -> Option<(UiDebugInvalidationSource, UiDebugInvalidationDetail)> {
        self.view_boundaries
            .get(node)
            .and_then(|state| state.frame_products.dirty.reason())
    }

    fn nearest_parent_view_boundary(&self, node: NodeId) -> Option<BoundaryId> {
        let mut current = self.nodes.get(node).and_then(|n| n.parent);
        while let Some(id) = current {
            if self.view_boundaries.contains_key(id)
                || self
                    .nodes
                    .get(id)
                    .is_some_and(|n| n.view_cache.enabled || n.widget_prepaint_enabled)
            {
                return Some(BoundaryId::from_node(id));
            }
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        None
    }

    pub fn debug_boundary_prepaint_owner_for_node(&self, node: NodeId) -> &'static str {
        if self.view_boundaries.contains_key(node) {
            "view_boundary_prepaint_state"
        } else {
            "none"
        }
    }

    pub(in crate::tree) fn boundary_allows_contained_relayout(&self, node: NodeId) -> bool {
        self.view_boundaries.get(node).is_some_and(|state| {
            debug_assert_eq!(state.id.node(), node);
            state.layout_dependencies.allows_contained_relayout()
        })
    }

    pub(in crate::tree) fn debug_boundary_layout_dependency_for_node(
        &self,
        node: NodeId,
    ) -> Option<&'static str> {
        self.view_boundaries
            .get(node)
            .map(|state| state.layout_dependencies.parent.as_debug_str())
    }

    pub fn debug_boundary_stats(&self) -> Vec<UiDebugBoundaryStats> {
        if !self.debug_enabled {
            return Vec::new();
        }

        let mut out: Vec<UiDebugBoundaryStats> = self
            .view_boundaries
            .iter()
            .map(|(node, state)| UiDebugBoundaryStats {
                id: state.id.node(),
                parent: state.parent.map(BoundaryId::node),
                element: self.nodes.get(node).and_then(|n| n.element),
                kind: match state.kind {
                    ViewBoundaryKind::Node => "node",
                    ViewBoundaryKind::ViewCacheRoot => "view_cache_root",
                },
                source: match state.kind {
                    ViewBoundaryKind::Node => "runtime",
                    ViewBoundaryKind::ViewCacheRoot => "view_cache",
                },
                prepaint_owner: "view_boundary_prepaint_state",
                hit_test_bounds_owner: if state.frame_products.hit_test_bounds.has_frame_product() {
                    "view_boundary_hit_test_bounds_state"
                } else {
                    "none"
                },
                semantics_subtree_owner: if state.frame_products.semantics.has_subtree() {
                    "view_boundary_semantics_state"
                } else {
                    "none"
                },
                interaction_cache_owner: if state.frame_products.interaction_cache.has_entry() {
                    "view_boundary_interaction_cache_state"
                } else {
                    "none"
                },
                paint_cache_owner: if state.frame_products.paint_cache.has_entry() {
                    "view_boundary_paint_cache_state"
                } else {
                    "none"
                },
                scene_fragment_owner: if state.frame_products.scene_fragment.slot_count() > 0 {
                    "view_boundary_scene_fragment_state"
                } else {
                    "none"
                },
                scene_fragment_slots: state.frame_products.scene_fragment.slot_count(),
                scene_fragment_entries: state.frame_products.scene_fragment.entry_count(),
                scene_fragment_chunks: state.frame_products.scene_fragment.chunk_count(),
                scene_fragment_fingerprint: state.frame_products.scene_fragment.fingerprint(),
                scene_fragment_used_entries: state.frame_products.scene_fragment.used_entries(),
                scene_fragment_rejected_entries: state
                    .frame_products
                    .scene_fragment
                    .rejected_entries(),
                scene_fragment_reject_reason: state
                    .frame_products
                    .scene_fragment
                    .last_reject_reason(),
                layout_dependency: state.layout_dependencies.parent.as_debug_str(),
                layout_definite: state.layout_dependencies.layout_definite,
                layout_dirty: state.frame_products.dirty.is_dirty(),
                layout_dirty_source: state
                    .frame_products
                    .dirty
                    .reason()
                    .map(|(source, _)| source),
                layout_dirty_detail: state
                    .frame_products
                    .dirty
                    .reason()
                    .map(|(_, detail)| detail),
            })
            .collect();
        out.sort_by_key(|stats| stats.id.data().as_ffi());
        out
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_exists(&self, node: NodeId) -> bool {
        self.view_boundaries.contains_key(node)
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_parent(&self, node: NodeId) -> Option<NodeId> {
        self.view_boundaries
            .get(node)
            .and_then(|state| state.parent)
            .map(BoundaryId::node)
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_prepaint_output<T: Any>(&self, node: NodeId) -> Option<&T> {
        self.view_boundaries
            .get(node)?
            .frame_products
            .prepaint
            .output::<T>()
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_allows_contained_relayout(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.layout_dependencies.allows_contained_relayout())
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_layout_dirty(&self, node: NodeId) -> bool {
        self.boundary_layout_dirty(node)
    }

    #[cfg(test)]
    pub(crate) fn test_dirty_view_frontier_empty(&self) -> bool {
        self.dirty_view_frontier.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn test_dirty_view_frontier_len(&self) -> usize {
        self.dirty_view_frontier.len()
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_paint_cache_has_entry(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.frame_products.paint_cache.has_entry())
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_interaction_cache_has_entry(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.frame_products.interaction_cache.has_entry())
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_semantics_has_subtree(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.frame_products.semantics.has_subtree())
    }

    #[cfg(test)]
    pub(crate) fn test_paint_cache_entry_for_node_has_entry(&self, node: NodeId) -> bool {
        self.paint_cache_entry_for_node(node).is_some()
    }

    #[cfg(test)]
    pub(crate) fn test_retained_paint_cache_entry_store_has_entry(&self, node: NodeId) -> bool {
        self.retained_paint_cache_entries
            .get(node)
            .is_some_and(PaintCacheEntryState::has_entry)
    }

    #[cfg(test)]
    pub(crate) fn test_retained_paint_recording_ops_len(&self) -> usize {
        self.window_paint_replay.retained_recording_ops_len()
    }
}

#[cfg(test)]
mod dirty_view_frontier_tests {
    use super::*;
    use slotmap::Key;

    fn node(id: u64) -> NodeId {
        NodeId::from(slotmap::KeyData::from_ffi(id))
    }

    #[test]
    fn dirty_view_frontier_coalesces_views_and_keeps_v1_node_bridge_explicit() {
        let first = node(1);
        let second = node(2);
        let mut frontier = DirtyViewFrontier::default();

        assert!(frontier.mark_boundary_node_v1(first));
        assert!(!frontier.mark_boundary_node_v1(first));
        assert!(frontier.mark_boundary_node_v1(second));

        assert_eq!(frontier.len(), 2);
        assert!(frontier.contains_boundary_node_v1(first));
        let mut views: Vec<ViewId> = frontier.iter_views().collect();
        views.sort_by_key(|view| view.0.data().as_ffi());
        assert_eq!(views, vec![ViewId::from(first), ViewId::from(second)]);

        let mut bridge_nodes: Vec<NodeId> = frontier.iter_boundary_nodes_v1().collect();
        bridge_nodes.sort_by_key(|id| id.data().as_ffi());
        assert_eq!(bridge_nodes, vec![first, second]);

        assert!(frontier.clear_boundary_node_v1(first));
        assert!(!frontier.clear_boundary_node_v1(first));
        assert_eq!(frontier.len(), 1);
    }
}

#[cfg(test)]
mod boundary_frame_products_tests {
    use super::*;
    use fret_core::{DrawOrder, Edges};

    fn node(id: u64) -> NodeId {
        NodeId::from(slotmap::KeyData::from_ffi(id))
    }

    fn paint_cache_key() -> PaintCacheKey {
        PaintCacheKey::new(
            Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
            1,
            1.0,
            1,
            crate::tree::paint_style::PaintStyleState::default(),
            None,
            Transform2D::IDENTITY,
        )
    }

    #[test]
    fn boundary_frame_products_own_boundary_dirty_prepaint_interaction_scene_and_paint_cache_state()
    {
        let mut state = ViewBoundaryState::new_runtime(
            BoundaryId::from_node(node(1)),
            None,
            ViewCacheFlags {
                enabled: true,
                layout_definite: true,
                ..Default::default()
            },
        );
        let key = paint_cache_key();

        state.frame_products.dirty.mark(
            UiDebugInvalidationSource::Notify,
            UiDebugInvalidationDetail::NotifyCall,
        );
        assert_eq!(
            state.frame_products.dirty.reason(),
            Some((
                UiDebugInvalidationSource::Notify,
                UiDebugInvalidationDetail::NotifyCall,
            ))
        );

        state.frame_products.prepaint.begin_outputs(key);
        state.frame_products.prepaint.set_output(42u32);
        assert_eq!(state.frame_products.prepaint.output::<u32>(), Some(&42));

        state
            .frame_products
            .hit_test_bounds
            .reuse_for_frame(FrameId(1));
        assert!(state.frame_products.hit_test_bounds.has_frame_product());

        state
            .frame_products
            .interaction_cache
            .set_entry(prepaint::InteractionCacheEntry {
                generation: 1,
                key,
                origin: Point::default(),
                start: 0,
                end: 2,
            });
        assert!(state.frame_products.interaction_cache.has_entry());

        let snapshot = Arc::new(SemanticsSnapshot {
            nodes: vec![
                SemanticsNode {
                    id: node(1),
                    parent: None,
                    role: SemanticsRole::Generic,
                    bounds: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
                    flags: Default::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("boundary".to_string()),
                    value: None,
                    extra: Default::default(),
                    text_selection: None,
                    text_composition: None,
                    actions: Default::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                    inline_spans: Vec::new(),
                },
                SemanticsNode {
                    id: node(2),
                    parent: Some(node(1)),
                    role: SemanticsRole::Generic,
                    bounds: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
                    flags: Default::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("child".to_string()),
                    value: None,
                    extra: Default::default(),
                    text_selection: None,
                    text_composition: None,
                    actions: Default::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                    inline_spans: Vec::new(),
                },
            ],
            ..Default::default()
        });
        state
            .frame_products
            .semantics
            .set_subtree(Arc::clone(&snapshot), 0, 2);
        assert!(state.frame_products.semantics.has_subtree());
        let mut reused = Vec::new();
        assert!(state.frame_products.semantics.reuse_subtree(
            None,
            Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
            &mut reused
        ));
        assert_eq!(reused.len(), 2);
        assert_eq!(reused[1].label.as_deref(), Some("child"));

        state.frame_products.scene_fragment.begin_fragment(key);
        let chunk = fret_core::SceneChunk::from_ops(Arc::from([SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
            background: Color::TRANSPARENT.into(),
            border: Edges::all(Px(0.0)),
            border_paint: Color::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        }]));
        let mut chunks = BoundarySceneChunkManifest::default();
        chunks.push(BoundarySceneFragmentChunk::new(
            chunk.clone(),
            Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
            Point::default(),
        ));
        state
            .frame_products
            .scene_fragment
            .set_fragment_with_debug_metadata(
                "fragment",
                BoundaryTypedOutputDebugMetadata {
                    entry_count: 3,
                    chunk_count: 1,
                    fingerprint: 0xF00D,
                    chunks,
                },
            );
        state.frame_products.scene_fragment.record_used_entries(2);
        state
            .frame_products
            .scene_fragment
            .record_rejected_entries(1, "test");
        assert_eq!(
            state.frame_products.scene_fragment.fragment::<&str>(),
            Some(&"fragment")
        );
        assert_eq!(state.frame_products.scene_fragment.entry_count(), 3);
        assert_eq!(state.frame_products.scene_fragment.chunk_count(), 1);
        assert_eq!(state.frame_products.scene_fragment.fingerprint(), 0xF00D);
        let manifest = state.frame_products.scene_fragment.chunk_manifest();
        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest.chunks()[0].fingerprint(), chunk.fingerprint());
        assert_eq!(state.frame_products.scene_fragment.used_entries(), 2);
        assert_eq!(state.frame_products.scene_fragment.rejected_entries(), 1);
        assert_eq!(
            state.frame_products.scene_fragment.last_reject_reason(),
            Some("test")
        );

        state.frame_products.paint_cache.set_entry(PaintCacheEntry {
            generation: 1,
            key,
            origin: Point::default(),
            start: 0,
            end: 1,
            text_blob_start: 0,
            text_blob_end: 0,
        });
        assert!(state.frame_products.paint_cache.has_entry());
    }
}
