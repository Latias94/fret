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
    composite_group_scopes: Vec<composite_group::CompositeGroupScope>,
    clip_path_scopes: Vec<clip_path::ClipPathScope>,
    clip_path_mask_in_use_bytes: u64,
    backdrop_source_group_scopes: Vec<backdrop_source_group::BackdropSourceGroupScope>,
    backdrop_source_group_reserved_targets: Vec<super::super::PlanTarget>,
    backdrop_source_group_in_use_bytes: u64,
    effect_chain_budget_stats: EffectChainBudgetStats,
    effect_degradations: EffectDegradationSnapshot,
    effect_blur_quality: BlurQualitySnapshot,
}

impl MarkerDispatchState {
    pub(super) fn new() -> Self {
        Self {
            effect_scopes: Vec::new(),
            composite_group_scopes: Vec::new(),
            clip_path_scopes: Vec::new(),
            clip_path_mask_in_use_bytes: 0,
            backdrop_source_group_scopes: Vec::new(),
            backdrop_source_group_reserved_targets: Vec::new(),
            backdrop_source_group_in_use_bytes: 0,
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
                        clip_path_mask_in_use_bytes: self.clip_path_mask_in_use_bytes,
                        clip_path_scopes: &self.clip_path_scopes,
                        backdrop_source_group_scopes: &self.backdrop_source_group_scopes,
                        backdrop_source_group_reserved_targets: &self
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: self.backdrop_source_group_in_use_bytes,
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
                        clip_path_mask_in_use_bytes: self.clip_path_mask_in_use_bytes,
                        backdrop_source_group_reserved_targets: &self
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: self.backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::ClipPathPush {
                scissor,
                uniform_index,
                mask_draw_index,
            } => {
                clip_path::compile_clip_path_push(
                    plan,
                    draw_scopes,
                    &mut self.clip_path_scopes,
                    &mut self.clip_path_mask_in_use_bytes,
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
                        backdrop_source_group_reserved_targets: &self
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: self.backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::ClipPathPop => {
                clip_path::compile_clip_path_pop(
                    plan,
                    draw_scopes,
                    &mut self.clip_path_scopes,
                    &mut self.clip_path_mask_in_use_bytes,
                );
            }
            EffectMarkerKind::BackdropSourceGroupPush {
                scissor,
                pyramid,
                quality,
            } => {
                backdrop_source_group::compile_backdrop_source_group_push(
                    plan,
                    draw_scopes,
                    &mut self.backdrop_source_group_scopes,
                    &mut self.backdrop_source_group_reserved_targets,
                    &mut self.backdrop_source_group_in_use_bytes,
                    &mut self.effect_degradations.backdrop_source_groups,
                    backdrop_source_group::BackdropSourceGroupPushCtx {
                        scissor,
                        pyramid,
                        quality,
                        scale_factor: args.scale_factor,
                        viewport_size: args.viewport_size,
                        format: args.format,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: self.clip_path_mask_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::BackdropSourceGroupPop => {
                backdrop_source_group::compile_backdrop_source_group_pop(
                    &mut self.backdrop_source_group_scopes,
                    &mut self.backdrop_source_group_reserved_targets,
                    &mut self.backdrop_source_group_in_use_bytes,
                );
            }
            EffectMarkerKind::CompositeGroupPush {
                scissor,
                uniform_index,
                mode,
                quality,
                opacity,
            } => {
                composite_group::compile_composite_group_push(
                    plan,
                    draw_scopes,
                    &mut self.composite_group_scopes,
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
                        clip_path_mask_in_use_bytes: self.clip_path_mask_in_use_bytes,
                        backdrop_source_group_reserved_targets: &self
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: self.backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::CompositeGroupPop => {
                composite_group::compile_composite_group_pop(
                    plan,
                    draw_scopes,
                    &mut self.composite_group_scopes,
                );
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
