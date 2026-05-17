use super::backdrop_source_group;
use super::clip_path;
use super::composite_group;
use super::context::RenderPlanCompilerCtx;
use super::draw_scope::DrawScopeStack;
use super::effect_chain::EffectChainBudgetStats;
use super::effect_scope;
use super::effects;
use crate::renderer::{
    BlurQualitySnapshot, EffectDegradationSnapshot, EffectMarker, EffectMarkerKind, PlanTarget,
    SceneEncoding, ScissorRect,
};

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
        draw_scopes: &mut DrawScopeStack,
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
            } => self.compile_effect_scope_push(
                plan,
                draw_scopes,
                draw_ix,
                EffectScopePushMarker {
                    scissor,
                    uniform_index,
                    mode,
                    chain,
                    quality,
                },
                args,
            ),
            EffectMarkerKind::Pop => {
                self.compile_effect_scope_pop(plan, draw_scopes, draw_ix, args);
            }
            EffectMarkerKind::ClipPathPush {
                scissor,
                uniform_index,
                mask_draw_index,
            } => self.compile_clip_path_push(
                plan,
                draw_scopes,
                encoding,
                draw_ix,
                ClipPathPushMarker {
                    scissor,
                    uniform_index,
                    mask_draw_index,
                },
                args,
            ),
            EffectMarkerKind::ClipPathPop => {
                self.clip_path_dispatch_state.compile_pop(plan, draw_scopes);
            }
            EffectMarkerKind::BackdropSourceGroupPush {
                scissor,
                pyramid,
                quality,
            } => self.compile_backdrop_source_group_push(
                plan,
                draw_scopes,
                BackdropSourceGroupPushMarker {
                    scissor,
                    pyramid,
                    quality,
                },
                args,
            ),
            EffectMarkerKind::BackdropSourceGroupPop => {
                self.backdrop_source_group_state.compile_pop();
            }
            EffectMarkerKind::CompositeGroupPush {
                scissor,
                uniform_index,
                mode,
                quality,
                opacity,
            } => self.compile_composite_group_push(
                plan,
                draw_scopes,
                draw_ix,
                CompositeGroupPushMarker {
                    scissor,
                    uniform_index,
                    mode,
                    quality,
                    opacity,
                },
                args,
            ),
            EffectMarkerKind::CompositeGroupPop => {
                self.composite_group_state.compile_pop(plan, draw_scopes);
            }
        }
    }

    fn compile_effect_scope_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        draw_ix: usize,
        marker: EffectScopePushMarker,
        args: MarkerDispatchCtx,
    ) {
        let shared_inputs = effect_scope_push_inputs(
            &self.clip_path_dispatch_state,
            &self.backdrop_source_group_state,
        );
        self.effect_scope_state.compile_push(
            plan,
            draw_scopes,
            draw_ix,
            effect_scope::EffectScopePushCtx {
                scissor: marker.scissor,
                uniform_index: marker.uniform_index,
                mode: marker.mode,
                chain: marker.chain,
                quality: marker.quality,
                viewport_size: args.viewport_size,
                format: args.format,
                clear: args.clear,
                scale_factor: args.scale_factor,
                intermediate_budget_bytes: args.intermediate_budget_bytes,
                clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
                clip_path_active_mask_targets: shared_inputs.clip_path_active_mask_targets,
                backdrop_source_group: shared_inputs.backdrop_source_group,
                backdrop_source_group_reserved_targets: shared_inputs
                    .backdrop_source_group_targets
                    .reserved_targets,
                backdrop_source_group_in_use_bytes: shared_inputs
                    .backdrop_source_group_targets
                    .in_use_bytes,
            },
        );
    }

    fn compile_effect_scope_pop(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        draw_ix: usize,
        args: MarkerDispatchCtx,
    ) {
        let shared_inputs = clip_mask_and_backdrop_target_inputs(
            &self.clip_path_dispatch_state,
            &self.backdrop_source_group_state,
        );
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
                backdrop_source_group_reserved_targets: shared_inputs
                    .backdrop_source_group_targets
                    .reserved_targets,
                backdrop_source_group_in_use_bytes: shared_inputs
                    .backdrop_source_group_targets
                    .in_use_bytes,
            },
        );
    }

    fn compile_clip_path_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        encoding: &SceneEncoding,
        draw_ix: usize,
        marker: ClipPathPushMarker,
        args: MarkerDispatchCtx,
    ) {
        let backdrop_targets = backdrop_source_group_targets(&self.backdrop_source_group_state);
        self.clip_path_dispatch_state.compile_push(
            plan,
            draw_scopes,
            encoding,
            draw_ix,
            clip_path::ClipPathPushCtx {
                scissor: marker.scissor,
                uniform_index: marker.uniform_index,
                mask_draw_index: marker.mask_draw_index,
                viewport_size: args.viewport_size,
                scissor_sized_intermediates: args.scissor_sized_intermediates,
                format: args.format,
                intermediate_budget_bytes: args.intermediate_budget_bytes,
                backdrop_source_group_reserved_targets: backdrop_targets.reserved_targets,
                backdrop_source_group_in_use_bytes: backdrop_targets.in_use_bytes,
            },
        );
    }

    fn compile_backdrop_source_group_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        marker: BackdropSourceGroupPushMarker,
        args: MarkerDispatchCtx,
    ) {
        let clip_path_mask_in_use_bytes = self.clip_path_dispatch_state.mask_in_use_bytes();
        self.backdrop_source_group_state.compile_push(
            plan,
            draw_scopes,
            self.effect_scope_state
                .backdrop_source_group_degradations_mut(),
            backdrop_source_group::BackdropSourceGroupPushCtx {
                scissor: marker.scissor,
                pyramid: marker.pyramid,
                quality: marker.quality,
                scale_factor: args.scale_factor,
                viewport_size: args.viewport_size,
                format: args.format,
                intermediate_budget_bytes: args.intermediate_budget_bytes,
                clip_path_mask_in_use_bytes,
            },
        );
    }

    fn compile_composite_group_push(
        &mut self,
        plan: &mut RenderPlanCompilerCtx,
        draw_scopes: &mut DrawScopeStack,
        draw_ix: usize,
        marker: CompositeGroupPushMarker,
        args: MarkerDispatchCtx,
    ) {
        let shared_inputs = clip_mask_and_backdrop_target_inputs(
            &self.clip_path_dispatch_state,
            &self.backdrop_source_group_state,
        );
        self.composite_group_state.compile_push(
            plan,
            draw_scopes,
            draw_ix,
            composite_group::CompositeGroupPushCtx {
                scissor: marker.scissor,
                uniform_index: marker.uniform_index,
                mode: marker.mode,
                quality: marker.quality,
                opacity: marker.opacity,
                viewport_size: args.viewport_size,
                scissor_sized_intermediates: args.scissor_sized_intermediates,
                format: args.format,
                intermediate_budget_bytes: args.intermediate_budget_bytes,
                clip_path_mask_in_use_bytes: shared_inputs.clip_path_mask_in_use_bytes,
                backdrop_source_group_reserved_targets: shared_inputs
                    .backdrop_source_group_targets
                    .reserved_targets,
                backdrop_source_group_in_use_bytes: shared_inputs
                    .backdrop_source_group_targets
                    .in_use_bytes,
            },
        );
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

#[derive(Clone, Copy, Debug)]
struct EffectScopePushMarker {
    scissor: ScissorRect,
    uniform_index: u32,
    mode: fret_core::EffectMode,
    chain: fret_core::EffectChain,
    quality: fret_core::EffectQuality,
}

#[derive(Clone, Copy, Debug)]
struct ClipPathPushMarker {
    scissor: ScissorRect,
    uniform_index: u32,
    mask_draw_index: u32,
}

#[derive(Clone, Copy, Debug)]
struct BackdropSourceGroupPushMarker {
    scissor: ScissorRect,
    pyramid: Option<fret_core::scene::CustomEffectPyramidRequestV1>,
    quality: fret_core::EffectQuality,
}

#[derive(Clone, Copy, Debug)]
struct CompositeGroupPushMarker {
    scissor: ScissorRect,
    uniform_index: u32,
    mode: fret_core::BlendMode,
    quality: fret_core::EffectQuality,
    opacity: f32,
}

#[derive(Clone, Copy, Debug)]
struct BackdropSourceGroupTargetInputs<'a> {
    reserved_targets: &'a [PlanTarget],
    in_use_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
struct EffectScopePushInputs<'a> {
    clip_path_mask_in_use_bytes: u64,
    clip_path_active_mask_targets: clip_path::ActiveMaskTargets,
    backdrop_source_group_targets: BackdropSourceGroupTargetInputs<'a>,
    backdrop_source_group: Option<effects::BackdropSourceGroupCtx>,
}

#[derive(Clone, Copy, Debug)]
struct ClipMaskAndBackdropTargetInputs<'a> {
    clip_path_mask_in_use_bytes: u64,
    backdrop_source_group_targets: BackdropSourceGroupTargetInputs<'a>,
}

fn backdrop_source_group_targets(
    backdrop_source_group_state: &backdrop_source_group::BackdropSourceGroupDispatchState,
) -> BackdropSourceGroupTargetInputs<'_> {
    BackdropSourceGroupTargetInputs {
        reserved_targets: backdrop_source_group_state.reserved_targets(),
        in_use_bytes: backdrop_source_group_state.in_use_bytes(),
    }
}

fn effect_scope_push_inputs<'a>(
    clip_path_dispatch_state: &clip_path::ClipPathDispatchState,
    backdrop_source_group_state: &'a backdrop_source_group::BackdropSourceGroupDispatchState,
) -> EffectScopePushInputs<'a> {
    EffectScopePushInputs {
        clip_path_mask_in_use_bytes: clip_path_dispatch_state.mask_in_use_bytes(),
        clip_path_active_mask_targets: clip_path_dispatch_state.active_mask_targets(),
        backdrop_source_group_targets: backdrop_source_group_targets(backdrop_source_group_state),
        backdrop_source_group: backdrop_source_group_state.effect_ctx(),
    }
}

fn clip_mask_and_backdrop_target_inputs<'a>(
    clip_path_dispatch_state: &clip_path::ClipPathDispatchState,
    backdrop_source_group_state: &'a backdrop_source_group::BackdropSourceGroupDispatchState,
) -> ClipMaskAndBackdropTargetInputs<'a> {
    ClipMaskAndBackdropTargetInputs {
        clip_path_mask_in_use_bytes: clip_path_dispatch_state.mask_in_use_bytes(),
        backdrop_source_group_targets: backdrop_source_group_targets(backdrop_source_group_state),
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
