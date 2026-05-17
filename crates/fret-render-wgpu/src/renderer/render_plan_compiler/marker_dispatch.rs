use super::backdrop_source_group;
use super::clip_path;
use super::composite_group;
use super::context::RenderPlanCompilerCtx;
use super::draw_scope::DrawScope;
use super::effect_chain::EffectChainBudgetStats;
use super::effect_scope;
use super::effects;
use crate::renderer::{
    BlurQualitySnapshot, EffectDegradationSnapshot, EffectMarker, EffectMarkerKind, PlanTarget,
    SceneEncoding,
};
use smallvec::SmallVec;

pub(super) struct MarkerDispatchState {
    effect_scope_state: effect_scope::EffectScopeDispatchState,
    composite_group_state: composite_group::CompositeGroupDispatchState,
    clip_path_dispatch_state: clip_path::ClipPathDispatchState,
    backdrop_source_group_state: backdrop_source_group::BackdropSourceGroupDispatchState,
}

impl MarkerDispatchState {
    pub(super) fn new() -> Self {
        Self {
            effect_scope_state: effect_scope::EffectScopeDispatchState::new(),
            composite_group_state: composite_group::CompositeGroupDispatchState::new(),
            clip_path_dispatch_state: clip_path::ClipPathDispatchState::new(),
            backdrop_source_group_state:
                backdrop_source_group::BackdropSourceGroupDispatchState::new(),
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
                let shared_inputs = self.shared_inputs();
                self.effect_scope_state.compile_push(
                    plan,
                    draw_scopes,
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
                        clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
                        clip_path_active_mask_targets: shared_inputs.clip_path_active_mask_targets,
                        backdrop_source_group: shared_inputs.backdrop_source_group_effect_ctx,
                        backdrop_source_group_reserved_targets: &shared_inputs
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: shared_inputs
                            .backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::Pop => {
                let shared_inputs = self.shared_inputs();
                self.effect_scope_state.compile_pop(
                    plan,
                    draw_scopes,
                    draw_ix,
                    effect_scope::EffectScopePopCtx {
                        viewport_size: args.viewport_size,
                        format: args.format,
                        clear: args.clear,
                        scale_factor: args.scale_factor,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
                        backdrop_source_group_reserved_targets: &shared_inputs
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: shared_inputs
                            .backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::ClipPathPush {
                scissor,
                uniform_index,
                mask_draw_index,
            } => {
                let shared_inputs = self.shared_inputs();
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
                        backdrop_source_group_reserved_targets: &shared_inputs
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: shared_inputs
                            .backdrop_source_group_in_use_bytes,
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
                let shared_inputs = self.shared_inputs();
                self.backdrop_source_group_state.compile_push(
                    plan,
                    draw_scopes,
                    self.effect_scope_state
                        .backdrop_source_group_degradations_mut(),
                    backdrop_source_group::BackdropSourceGroupPushCtx {
                        scissor,
                        pyramid,
                        quality,
                        scale_factor: args.scale_factor,
                        viewport_size: args.viewport_size,
                        format: args.format,
                        intermediate_budget_bytes: args.intermediate_budget_bytes,
                        clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
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
                let shared_inputs = self.shared_inputs();
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
                        clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
                        backdrop_source_group_reserved_targets: &shared_inputs
                            .backdrop_source_group_reserved_targets,
                        backdrop_source_group_in_use_bytes: shared_inputs
                            .backdrop_source_group_in_use_bytes,
                    },
                );
            }
            EffectMarkerKind::CompositeGroupPop => {
                self.composite_group_state.compile_pop(plan, draw_scopes);
            }
        }
    }

    fn shared_inputs(&self) -> MarkerSharedDispatchInputs {
        MarkerSharedDispatchInputs {
            clip_path_mask_in_use_bytes: self.clip_path_dispatch_state.mask_in_use_bytes(),
            clip_path_active_mask_targets: self.clip_path_dispatch_state.active_mask_targets(),
            backdrop_source_group_reserved_targets: self
                .backdrop_source_group_state
                .reserved_targets()
                .iter()
                .copied()
                .collect(),
            backdrop_source_group_in_use_bytes: self.backdrop_source_group_state.in_use_bytes(),
            backdrop_source_group_effect_ctx: self.backdrop_source_group_state.effect_ctx(),
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        EffectDegradationSnapshot,
        BlurQualitySnapshot,
        EffectChainBudgetStats,
    ) {
        self.effect_scope_state.into_parts()
    }
}

#[derive(Clone, Debug)]
struct MarkerSharedDispatchInputs {
    clip_path_mask_in_use_bytes: u64,
    clip_path_active_mask_targets: clip_path::ActiveMaskTargets,
    backdrop_source_group_reserved_targets: SmallVec<[PlanTarget; 8]>,
    backdrop_source_group_in_use_bytes: u64,
    backdrop_source_group_effect_ctx: Option<effects::BackdropSourceGroupCtx>,
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
