use super::backdrop_source_group;
use super::clip_path;
use super::composite_group;
use super::context::RenderPlanCompilerCtx;
use super::draw_scope::DrawScope;
use super::effect_chain::EffectChainBudgetStats;
use super::effect_scope;
use crate::renderer::{
    BlurQualitySnapshot, EffectDegradationSnapshot, EffectMarker, EffectMarkerKind, SceneEncoding,
};

pub(super) struct MarkerDispatchState {
    effect_scopes: Vec<effect_scope::EffectScope>,
    composite_group_state: composite_group::CompositeGroupDispatchState,
    clip_path_dispatch_state: clip_path::ClipPathDispatchState,
    backdrop_source_group_state: backdrop_source_group::BackdropSourceGroupDispatchState,
    effect_chain_budget_stats: EffectChainBudgetStats,
    effect_degradations: EffectDegradationSnapshot,
    effect_blur_quality: BlurQualitySnapshot,
}

impl MarkerDispatchState {
    pub(super) fn new() -> Self {
        Self {
            effect_scopes: Vec::new(),
            composite_group_state: composite_group::CompositeGroupDispatchState::new(),
            clip_path_dispatch_state: clip_path::ClipPathDispatchState::new(),
            backdrop_source_group_state:
                backdrop_source_group::BackdropSourceGroupDispatchState::new(),
            effect_chain_budget_stats: EffectChainBudgetStats::default(),
            effect_degradations: EffectDegradationSnapshot::default(),
            effect_blur_quality: BlurQualitySnapshot::default(),
        }
    }

    pub(super) fn compile_marker(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut Vec<DrawScope>,
        encoding: &SceneEncoding,
        draw_ix: usize,
        marker: EffectMarker,
        args: MarkerDispatchCtx,
    ) {
        match marker.kind {
            EffectMarkerKind::Push {
                scissor,
                uniform_index,
                mode,
                chain,
                quality,
            } => {
                effect_scope::compile_effect_scope_push(
                    plan,
                    draw_scopes,
                    &mut self.effect_scopes,
                    &mut self.effect_chain_budget_stats,
                    &mut self.effect_degradations,
                    &mut self.effect_blur_quality,
                    draw_ix,
                    effect_scope::EffectScopePushCtx {
                        scissor,
                        uniform_index,
                        mode,
                        chain,
                        quality,
                        viewport_size: args.viewport_size,
                        format: args.format,
                        clear: args.clear,
                        scale_factor: args.scale_factor,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: self
                            .clip_path_dispatch_state
                            .mask_in_use_bytes(),
                        clip_path_active_mask_targets: self
                            .clip_path_dispatch_state
                            .active_mask_targets(),
                        backdrop_source_group: self.backdrop_source_group_state.effect_ctx(),
                        backdrop_source_group_reserved_targets: self
                            .backdrop_source_group_state
                            .reserved_targets(),
                        backdrop_source_group_in_use_bytes: self
                            .backdrop_source_group_state
                            .in_use_bytes(),
                    },
                );
            }
            EffectMarkerKind::Pop => {
                effect_scope::compile_effect_scope_pop(
                    plan,
                    draw_scopes,
                    &mut self.effect_scopes,
                    &mut self.effect_chain_budget_stats,
                    &mut self.effect_degradations,
                    &mut self.effect_blur_quality,
                    draw_ix,
                    effect_scope::EffectScopePopCtx {
                        viewport_size: args.viewport_size,
                        format: args.format,
                        clear: args.clear,
                        scale_factor: args.scale_factor,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: self
                            .clip_path_dispatch_state
                            .mask_in_use_bytes(),
                        backdrop_source_group_reserved_targets: self
                            .backdrop_source_group_state
                            .reserved_targets(),
                        backdrop_source_group_in_use_bytes: self
                            .backdrop_source_group_state
                            .in_use_bytes(),
                    },
                );
            }
            EffectMarkerKind::ClipPathPush {
                scissor,
                uniform_index,
                mask_draw_index,
            } => {
                self.clip_path_dispatch_state.compile_push(
                    plan,
                    draw_scopes,
                    encoding,
                    draw_ix,
                    clip_path::ClipPathPushCtx {
                        scissor,
                        uniform_index,
                        mask_draw_index,
                        viewport_size: args.viewport_size,
                        scissor_sized_intermediates: args.scissor_sized_intermediates,
                        format: args.format,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        backdrop_source_group_reserved_targets: self
                            .backdrop_source_group_state
                            .reserved_targets(),
                        backdrop_source_group_in_use_bytes: self
                            .backdrop_source_group_state
                            .in_use_bytes(),
                    },
                );
            }
            EffectMarkerKind::ClipPathPop => {
                self.clip_path_dispatch_state.compile_pop(plan, draw_scopes);
            }
            EffectMarkerKind::BackdropSourceGroupPush {
                scissor,
                pyramid,
                quality,
            } => {
                self.backdrop_source_group_state.compile_push(
                    plan,
                    draw_scopes,
                    &mut self.effect_degradations.backdrop_source_groups,
                    backdrop_source_group::BackdropSourceGroupPushCtx {
                        scissor,
                        pyramid,
                        quality,
                        scale_factor: args.scale_factor,
                        viewport_size: args.viewport_size,
                        format: args.format,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: self
                            .clip_path_dispatch_state
                            .mask_in_use_bytes(),
                    },
                );
            }
            EffectMarkerKind::BackdropSourceGroupPop => {
                self.backdrop_source_group_state.compile_pop();
            }
            EffectMarkerKind::CompositeGroupPush {
                scissor,
                uniform_index,
                mode,
                quality,
                opacity,
            } => {
                self.composite_group_state.compile_push(
                    plan,
                    draw_scopes,
                    draw_ix,
                    composite_group::CompositeGroupPushCtx {
                        scissor,
                        uniform_index,
                        mode,
                        quality,
                        opacity,
                        viewport_size: args.viewport_size,
                        scissor_sized_intermediates: args.scissor_sized_intermediates,
                        format: args.format,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: self
                            .clip_path_dispatch_state
                            .mask_in_use_bytes(),
                        backdrop_source_group_reserved_targets: self
                            .backdrop_source_group_state
                            .reserved_targets(),
                        backdrop_source_group_in_use_bytes: self
                            .backdrop_source_group_state
                            .in_use_bytes(),
                    },
                );
            }
            EffectMarkerKind::CompositeGroupPop => {
                self.composite_group_state.compile_pop(plan, draw_scopes);
            }
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        EffectDegradationSnapshot,
        BlurQualitySnapshot,
        EffectChainBudgetStats,
    ) {
        (
            self.effect_degradations,
            self.effect_blur_quality,
            self.effect_chain_budget_stats,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MarkerDispatchCtx {
    pub(super) viewport_size: (u32, u32),
    pub(super) scissor_sized_intermediates: bool,
    pub(super) format: wgpu::TextureFormat,
    pub(super) clear: wgpu::Color,
    pub(super) scale_factor: f32,
    pub(super) intermediate_budget_bytes: u64,
}
