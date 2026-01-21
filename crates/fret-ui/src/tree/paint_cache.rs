use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PaintCacheKey {
    width_bits: u32,
    height_bits: u32,
    scale_factor_bits: u32,
    theme_revision: u64,
    child_a_bits: u32,
    child_b_bits: u32,
    child_c_bits: u32,
    child_d_bits: u32,
    child_tx_bits: u32,
    child_ty_bits: u32,
}

impl PaintCacheKey {
    pub(super) fn new(
        bounds: Rect,
        scale_factor: f32,
        theme_revision: u64,
        child_transform: Transform2D,
    ) -> Self {
        Self {
            width_bits: bounds.size.width.0.to_bits(),
            height_bits: bounds.size.height.0.to_bits(),
            scale_factor_bits: scale_factor.to_bits(),
            theme_revision,
            child_a_bits: child_transform.a.to_bits(),
            child_b_bits: child_transform.b.to_bits(),
            child_c_bits: child_transform.c.to_bits(),
            child_d_bits: child_transform.d.to_bits(),
            child_tx_bits: child_transform.tx.to_bits(),
            child_ty_bits: child_transform.ty.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PaintCacheEntry {
    pub(super) generation: u64,
    pub(super) key: PaintCacheKey,
    pub(super) origin: Point,
    pub(super) start: u32,
    pub(super) end: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PaintCachePolicy {
    /// Enable caching unless the UI is in an inspection/probe mode (e.g. picking, semantics).
    #[default]
    Auto,
    /// Always enable caching.
    Enabled,
    /// Always disable caching.
    Disabled,
}

#[derive(Debug, Default)]
pub(super) struct PaintCacheState {
    pub(super) generation: u64,
    pub(super) prev_ops: Vec<SceneOp>,
    pub(super) prev_fingerprint: u64,
    pub(super) source_generation: u64,
    pub(super) target_generation: u64,
    pub(super) hits: u32,
    pub(super) misses: u32,
    pub(super) replayed_ops: u32,
}

impl PaintCacheState {
    pub(super) fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_ops = 0;
    }

    pub(super) fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    pub(super) fn invalidate_recording(&mut self) {
        self.prev_ops.clear();
        self.prev_fingerprint = 0;
        self.generation = self.generation.saturating_add(1);
    }
}
