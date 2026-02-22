use super::frame_targets::downsampled_size;
use super::render_plan_effects::{map_scissor_downsample_nearest, map_scissor_to_size};
use super::{SceneEncoding, ScissorRect};
use crate::renderer::estimate_texture_bytes;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneSegmentId(pub(super) usize);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct RenderPlanSegmentFlags {
    pub(super) has_quad: bool,
    pub(super) has_viewport: bool,
    pub(super) has_image: bool,
    pub(super) has_mask: bool,
    pub(super) has_text: bool,
    pub(super) has_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RenderPlanSegment {
    pub(super) id: SceneSegmentId,
    pub(super) draw_range: Range<usize>,
    pub(super) start_uniform_index: Option<u32>,
    pub(super) start_uniform_fingerprint: u64,
    pub(super) flags: RenderPlanSegmentFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct RenderPlanCompileStats {
    pub(super) estimated_peak_intermediate_bytes: u64,
    pub(super) degradation_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderPlanDegradationReason {
    BudgetZero,
    BudgetInsufficient,
    TargetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RenderPlanDegradationKind {
    BackdropEffectNoOp,
    FilterContentDisabled,
    ClipPathDisabled,
    CompositeGroupBlendDegradedToOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RenderPlanDegradation {
    pub(super) draw_ix: usize,
    pub(super) kind: RenderPlanDegradationKind,
    pub(super) reason: RenderPlanDegradationReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanTarget {
    Output,
    Intermediate0,
    Intermediate1,
    Intermediate2,
    Mask0,
    Mask1,
    Mask2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MaskRef {
    pub(super) target: PlanTarget,
    pub(super) size: (u32, u32),
    pub(super) viewport_rect: ScissorRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DebugPostprocess {
    None,
    OffscreenBlit,
    Pixelate {
        scale: u32,
    },
    Blur {
        radius: u32,
        downsample_scale: u32,
        scissor: Option<ScissorRect>,
    },
}

#[derive(Debug)]
pub(super) struct SceneDrawRangePass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) target_origin: (u32, u32),
    pub(super) target_size: (u32, u32),
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
    pub(super) draw_range: Range<usize>,
}

#[derive(Debug)]
pub(super) enum RenderPlanPass {
    SceneDrawRange(SceneDrawRangePass),
    PathMsaaBatch(PathMsaaBatchPass),
    PathClipMask(PathClipMaskPass),
    FullscreenBlit(FullscreenBlitPass),
    CompositePremul(CompositePremulPass),
    ScaleNearest(ScaleNearestPass),
    Blur(BlurPass),
    BackdropWarp(BackdropWarpPass),
    ColorAdjust(ColorAdjustPass),
    ColorMatrix(ColorMatrixPass),
    AlphaThreshold(AlphaThresholdPass),
    DropShadow(DropShadowPass),
    ClipMask(ClipMaskPass),
    ReleaseTarget(PlanTarget),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PathClipMaskPass {
    pub(super) dst: PlanTarget,
    pub(super) dst_origin: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) scissor: ScissorRect,
    pub(super) uniform_index: u32,
    pub(super) first_vertex: u32,
    pub(super) vertex_count: u32,
    pub(super) cache_key: u64,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ClipMaskPass {
    pub(super) dst: PlanTarget,
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) uniform_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlurAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BlurPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) axis: BlurAxis,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BackdropWarpPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) origin_px: (u32, u32),
    pub(super) bounds_size_px: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) strength_px: f32,
    pub(super) scale_px: f32,
    pub(super) phase: f32,
    pub(super) chromatic_aberration_px: f32,
    pub(super) kind: fret_core::scene::BackdropWarpKindV1,
    pub(super) warp_image: Option<fret_core::ImageId>,
    pub(super) warp_uv: fret_core::scene::UvRect,
    pub(super) warp_sampling: fret_core::scene::ImageSamplingHint,
    pub(super) warp_encoding: fret_core::scene::WarpMapEncodingV1,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ColorAdjustPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) saturation: f32,
    pub(super) brightness: f32,
    pub(super) contrast: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ColorMatrixPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) matrix: [f32; 20],
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AlphaThresholdPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) cutoff: f32,
    pub(super) soft: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DropShadowPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) offset_px: (f32, f32),
    pub(super) color: fret_core::scene::Color,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FullscreenBlitPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CompositePremulPass {
    pub(super) src: PlanTarget,
    pub(super) src_origin: (u32, u32),
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_origin: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) blend_mode: fret_core::BlendMode,
    pub(super) opacity: f32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScaleMode {
    Downsample,
    Upscale,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ScaleNearestPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) src_origin: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) dst_origin: (u32, u32),
    pub(super) mask_uniform_index: Option<u32>,
    pub(super) mask: Option<MaskRef>,
    pub(super) mode: ScaleMode,
    pub(super) scale: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone)]
pub(super) struct PathMsaaBatchPass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) target_origin: (u32, u32),
    pub(super) target_size: (u32, u32),
    pub(super) draw_range: Range<usize>,
    pub(super) union_scissor: ScissorRect,
    pub(super) batch_uniform_index: u32,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub(super) segments: Vec<RenderPlanSegment>,
    pub(super) passes: Vec<RenderPlanPass>,
    pub(super) compile_stats: RenderPlanCompileStats,
    pub(super) degradations: Vec<RenderPlanDegradation>,
}

impl RenderPlan {
    pub(super) fn finalize(
        segments: Vec<RenderPlanSegment>,
        passes: Vec<RenderPlanPass>,
        viewport_size: (u32, u32),
        postprocess: DebugPostprocess,
        clear: wgpu::Color,
        format: wgpu::TextureFormat,
        degradations: Vec<RenderPlanDegradation>,
    ) -> Self {
        let mut plan = Self {
            segments,
            passes,
            compile_stats: RenderPlanCompileStats {
                estimated_peak_intermediate_bytes: 0,
                degradation_count: degradations.len() as u64,
            },
            degradations,
        };
        append_postprocess(&mut plan, viewport_size, postprocess, clear);
        insert_early_releases(&mut plan.passes);
        plan.compile_stats.estimated_peak_intermediate_bytes =
            estimate_plan_peak_intermediate_bytes(&plan.passes, format);
        plan.compile_stats.degradation_count = plan.degradations.len() as u64;
        plan
    }

    pub(super) fn compile_for_scene(
        encoding: &SceneEncoding,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        clear: wgpu::Color,
        path_samples: u32,
        postprocess: DebugPostprocess,
        intermediate_budget_bytes: u64,
    ) -> Self {
        super::render_plan_compiler::compile_for_scene(
            encoding,
            viewport_size,
            format,
            clear,
            path_samples,
            postprocess,
            intermediate_budget_bytes,
        )
    }

    #[cfg(debug_assertions)]
    pub(super) fn debug_validate(&self) {
        if let Err(message) = validate_plan_target_lifetimes(&self.passes) {
            panic!("RenderPlan validation failed: {message}");
        }
    }

    #[cfg(not(debug_assertions))]
    pub(super) fn debug_validate(&self) {}
}

#[cfg(debug_assertions)]
fn validate_plan_target_lifetimes(passes: &[RenderPlanPass]) -> Result<(), String> {
    fn slot(t: PlanTarget) -> Option<usize> {
        match t {
            PlanTarget::Intermediate0 => Some(0),
            PlanTarget::Intermediate1 => Some(1),
            PlanTarget::Intermediate2 => Some(2),
            PlanTarget::Mask0 => Some(3),
            PlanTarget::Mask1 => Some(4),
            PlanTarget::Mask2 => Some(5),
            PlanTarget::Output => None,
        }
    }

    fn target_label(t: PlanTarget) -> &'static str {
        match t {
            PlanTarget::Output => "Output",
            PlanTarget::Intermediate0 => "Intermediate0",
            PlanTarget::Intermediate1 => "Intermediate1",
            PlanTarget::Intermediate2 => "Intermediate2",
            PlanTarget::Mask0 => "Mask0",
            PlanTarget::Mask1 => "Mask1",
            PlanTarget::Mask2 => "Mask2",
        }
    }

    let mut live: [bool; 6] = [false; 6];
    let mut initialized: [bool; 6] = [false; 6];

    fn mark_read(
        live: &[bool; 6],
        initialized: &[bool; 6],
        pass_index: usize,
        t: PlanTarget,
    ) -> Result<(), String> {
        let Some(slot) = slot(t) else {
            return Ok(());
        };
        if !live[slot] {
            return Err(format!(
                "pass[{pass_index}] reads {} after release (not live)",
                target_label(t)
            ));
        }
        if !initialized[slot] {
            return Err(format!(
                "pass[{pass_index}] reads {} before initialization",
                target_label(t)
            ));
        }
        Ok(())
    }

    fn mark_write(
        live: &mut [bool; 6],
        initialized: &mut [bool; 6],
        pass_index: usize,
        t: PlanTarget,
        load: Option<wgpu::LoadOp<wgpu::Color>>,
    ) -> Result<(), String> {
        let Some(slot) = slot(t) else {
            return Ok(());
        };

        if let Some(wgpu::LoadOp::Load) = load {
            if !initialized[slot] {
                return Err(format!(
                    "pass[{pass_index}] writes {} with LoadOp::Load before initialization",
                    target_label(t)
                ));
            }
            if !live[slot] {
                return Err(format!(
                    "pass[{pass_index}] writes {} with LoadOp::Load after release (not live)",
                    target_label(t)
                ));
            }
        }

        live[slot] = true;
        // Passes without an explicit LoadOp are assumed to initialize the destination.
        initialized[slot] = true;
        Ok(())
    }

    fn mark_release(
        live: &mut [bool; 6],
        initialized: &mut [bool; 6],
        pass_index: usize,
        t: PlanTarget,
    ) -> Result<(), String> {
        let Some(slot) = slot(t) else {
            return Err(format!(
                "pass[{pass_index}] releases {}, but releasing Output is invalid",
                target_label(t)
            ));
        };
        if !live[slot] {
            return Err(format!(
                "pass[{pass_index}] releases {} when not live",
                target_label(t)
            ));
        }
        live[slot] = false;
        initialized[slot] = false;
        Ok(())
    }

    for (pass_index, pass) in passes.iter().enumerate() {
        match *pass {
            RenderPlanPass::SceneDrawRange(SceneDrawRangePass { target, load, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, target, Some(load))?;
            }
            RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass { target, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, target, None)?;
            }
            RenderPlanPass::PathClipMask(PathClipMaskPass { dst, load, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ClipMask(ClipMaskPass { dst, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, dst, None)?;
            }
            RenderPlanPass::FullscreenBlit(FullscreenBlitPass { src, dst, load, .. }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::CompositePremul(CompositePremulPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ScaleNearest(ScaleNearestPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::Blur(BlurPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::BackdropWarp(BackdropWarpPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ColorAdjust(ColorAdjustPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ColorMatrix(ColorMatrixPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::AlphaThreshold(AlphaThresholdPass {
                src,
                dst,
                mask,
                load,
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::DropShadow(DropShadowPass { src, dst, load, .. }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ReleaseTarget(t) => {
                mark_release(&mut live, &mut initialized, pass_index, t)?;
            }
        }
    }

    Ok(())
}

fn estimate_plan_peak_intermediate_bytes(
    passes: &[RenderPlanPass],
    scene_format: wgpu::TextureFormat,
) -> u64 {
    fn idx(t: PlanTarget) -> usize {
        match t {
            PlanTarget::Output => 0,
            PlanTarget::Intermediate0 => 1,
            PlanTarget::Intermediate1 => 2,
            PlanTarget::Intermediate2 => 3,
            PlanTarget::Mask0 => 4,
            PlanTarget::Mask1 => 5,
            PlanTarget::Mask2 => 6,
        }
    }

    fn target_format(t: PlanTarget, scene_format: wgpu::TextureFormat) -> wgpu::TextureFormat {
        match t {
            PlanTarget::Output
            | PlanTarget::Intermediate0
            | PlanTarget::Intermediate1
            | PlanTarget::Intermediate2 => scene_format,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                wgpu::TextureFormat::R8Unorm
            }
        }
    }

    let mut live: [bool; 7] = [false; 7];
    let mut sizes: [(u32, u32); 7] = [(0, 0); 7];
    let mut peak: u64 = 0;

    fn mark_live(
        live: &mut [bool; 7],
        sizes: &mut [(u32, u32); 7],
        t: PlanTarget,
        size: (u32, u32),
    ) {
        if t == PlanTarget::Output || size.0 == 0 || size.1 == 0 {
            return;
        }
        live[idx(t)] = true;
        sizes[idx(t)] = size;
    }

    for p in passes {
        match *p {
            RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                target,
                target_size,
                ..
            }) => {
                mark_live(&mut live, &mut sizes, target, target_size);
            }
            RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                target,
                target_size,
                ..
            }) => {
                mark_live(&mut live, &mut sizes, target, target_size);
            }
            RenderPlanPass::PathClipMask(PathClipMaskPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::ClipMask(ClipMaskPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::FullscreenBlit(FullscreenBlitPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::CompositePremul(CompositePremulPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::ScaleNearest(ScaleNearestPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::Blur(BlurPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::BackdropWarp(BackdropWarpPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::ColorAdjust(ColorAdjustPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::ColorMatrix(ColorMatrixPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::AlphaThreshold(AlphaThresholdPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::DropShadow(DropShadowPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::ReleaseTarget(t) => {
                live[idx(t)] = false;
            }
        }

        let mut cur: u64 = 0;
        for t in [
            PlanTarget::Intermediate0,
            PlanTarget::Intermediate1,
            PlanTarget::Intermediate2,
            PlanTarget::Mask0,
            PlanTarget::Mask1,
            PlanTarget::Mask2,
        ] {
            if !live[idx(t)] {
                continue;
            }
            cur = cur.saturating_add(estimate_texture_bytes(
                sizes[idx(t)],
                target_format(t, scene_format),
                1,
            ));
        }
        peak = peak.max(cur);
    }

    peak
}

fn insert_early_releases(passes: &mut Vec<RenderPlanPass>) -> u64 {
    let mut last_use: [Option<usize>; 6] = [None, None, None, None, None, None];

    for (idx, pass) in passes.iter().enumerate() {
        let mut mark = |t: PlanTarget| {
            let slot = match t {
                PlanTarget::Intermediate0 => Some(0),
                PlanTarget::Intermediate1 => Some(1),
                PlanTarget::Intermediate2 => Some(2),
                PlanTarget::Mask0 => Some(3),
                PlanTarget::Mask1 => Some(4),
                PlanTarget::Mask2 => Some(5),
                PlanTarget::Output => None,
            };
            if let Some(slot) = slot {
                last_use[slot] = Some(idx);
            }
        };

        match pass {
            RenderPlanPass::SceneDrawRange(p) => mark(p.target),
            RenderPlanPass::PathMsaaBatch(p) => mark(p.target),
            RenderPlanPass::PathClipMask(p) => mark(p.dst),
            RenderPlanPass::FullscreenBlit(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::CompositePremul(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::ScaleNearest(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::Blur(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::BackdropWarp(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::ColorAdjust(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::ColorMatrix(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::AlphaThreshold(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::DropShadow(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::ClipMask(p) => mark(p.dst),
            RenderPlanPass::ReleaseTarget(_target) => {}
        }
    }

    let last0 = last_use[0];
    let last1 = last_use[1];
    let last2 = last_use[2];
    let last_mask0 = last_use[3];
    let last_mask1 = last_use[4];
    let last_mask2 = last_use[5];

    let old = std::mem::take(passes);
    let mut out: Vec<RenderPlanPass> = Vec::with_capacity(old.len() + 4);
    let mut inserted: u64 = 0;

    for (idx, pass) in old.into_iter().enumerate() {
        out.push(pass);

        let mut push_release = |t: PlanTarget| {
            out.push(RenderPlanPass::ReleaseTarget(t));
            inserted = inserted.saturating_add(1);
        };

        if last0 == Some(idx) {
            push_release(PlanTarget::Intermediate0);
        }
        if last1 == Some(idx) {
            push_release(PlanTarget::Intermediate1);
        }
        if last2 == Some(idx) {
            push_release(PlanTarget::Intermediate2);
        }
        if last_mask0 == Some(idx) {
            push_release(PlanTarget::Mask0);
        }
        if last_mask1 == Some(idx) {
            push_release(PlanTarget::Mask1);
        }
        if last_mask2 == Some(idx) {
            push_release(PlanTarget::Mask2);
        }
    }

    *passes = out;
    inserted
}

fn decompose_pixelate_scale(scale: u32) -> Vec<u32> {
    let mut scale = scale.max(1);
    let mut steps = Vec::new();
    while scale >= 4 && scale.is_multiple_of(2) {
        steps.push(2);
        scale /= 2;
    }
    steps.push(scale.max(1));
    steps
}

type DownsampleChainEntry = ((u32, u32), u32);
type DownsampleChainResult = (PlanTarget, (u32, u32), Vec<DownsampleChainEntry>);

fn push_scale_nearest(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    mode: ScaleMode,
    scale: u32,
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes
        .push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
            src,
            dst,
            src_size,
            dst_size,
            src_origin: (0, 0),
            dst_scissor,
            dst_origin: (0, 0),
            mask_uniform_index: None,
            mask: None,
            mode,
            scale,
            load,
        }));
}

fn push_fullscreen_blit(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes
        .push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src,
            dst,
            src_size,
            dst_size,
            dst_scissor,
            load,
        }));
}

fn push_blur(
    plan: &mut RenderPlan,
    src: PlanTarget,
    dst: PlanTarget,
    src_size: (u32, u32),
    dst_size: (u32, u32),
    dst_scissor: Option<ScissorRect>,
    axis: BlurAxis,
    load: wgpu::LoadOp<wgpu::Color>,
) {
    plan.passes.push(RenderPlanPass::Blur(BlurPass {
        src,
        dst,
        src_size,
        dst_size,
        dst_scissor,
        mask_uniform_index: None,
        mask: None,
        axis,
        load,
    }));
}

fn append_downsample_chain(
    plan: &mut RenderPlan,
    mut current_target: PlanTarget,
    mut current_size: (u32, u32),
    steps: &[u32],
    mut dst_a: PlanTarget,
    mut dst_b: PlanTarget,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
) -> DownsampleChainResult {
    let mut stack: Vec<DownsampleChainEntry> = Vec::with_capacity(steps.len());
    for step in steps.iter().copied() {
        let dst_size = downsampled_size(current_size, step);
        let dst_scissor = map_scissor_to_size(scissor_in_full, full_size, dst_size);
        push_scale_nearest(
            plan,
            current_target,
            dst_a,
            current_size,
            dst_size,
            dst_scissor,
            ScaleMode::Downsample,
            step,
            wgpu::LoadOp::Clear(clear),
        );
        stack.push((current_size, step));
        current_target = dst_a;
        current_size = dst_size;
        std::mem::swap(&mut dst_a, &mut dst_b);
    }
    (current_target, current_size, stack)
}

#[derive(Debug, Clone)]
struct DownsampleHalfQuarter {
    half_target: PlanTarget,
    #[allow(dead_code)]
    half_size: (u32, u32),
    quarter_target: PlanTarget,
    quarter_size: (u32, u32),
    stack: Vec<((u32, u32), u32)>,
}

fn append_downsample_half_quarter(
    plan: &mut RenderPlan,
    src_target: PlanTarget,
    src_size: (u32, u32),
    half_target: PlanTarget,
    quarter_target: PlanTarget,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
) -> DownsampleHalfQuarter {
    debug_assert_ne!(src_target, PlanTarget::Output);
    debug_assert_ne!(half_target, PlanTarget::Output);
    debug_assert_ne!(quarter_target, PlanTarget::Output);
    debug_assert_ne!(half_target, quarter_target);

    let half_size = downsampled_size(src_size, 2);
    let half_scissor = map_scissor_to_size(scissor_in_full, full_size, half_size);
    push_scale_nearest(
        plan,
        src_target,
        half_target,
        src_size,
        half_size,
        half_scissor,
        ScaleMode::Downsample,
        2,
        wgpu::LoadOp::Clear(clear),
    );

    let quarter_size = downsampled_size(half_size, 2);
    let quarter_scissor = map_scissor_to_size(scissor_in_full, full_size, quarter_size);
    push_scale_nearest(
        plan,
        half_target,
        quarter_target,
        half_size,
        quarter_size,
        quarter_scissor,
        ScaleMode::Downsample,
        2,
        wgpu::LoadOp::Clear(clear),
    );

    DownsampleHalfQuarter {
        half_target,
        half_size,
        quarter_target,
        quarter_size,
        stack: vec![(src_size, 2), (half_size, 2)],
    }
}

fn append_upsample_chain(
    plan: &mut RenderPlan,
    mut current_target: PlanTarget,
    mut current_size: (u32, u32),
    mut stack: Vec<((u32, u32), u32)>,
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    clear: wgpu::Color,
) -> (PlanTarget, (u32, u32)) {
    while let Some((dst_size, step)) = stack.pop() {
        let dst_target = match current_target {
            PlanTarget::Intermediate1 => PlanTarget::Intermediate2,
            PlanTarget::Intermediate2 => PlanTarget::Intermediate1,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                unreachable!("upsample chain must read from Intermediate1/2")
            }
            PlanTarget::Intermediate0 | PlanTarget::Output => {
                unreachable!("upsample chain must read from Intermediate1/2")
            }
        };
        let dst_scissor = map_scissor_to_size(scissor_in_full, full_size, dst_size);
        push_scale_nearest(
            plan,
            current_target,
            dst_target,
            current_size,
            dst_size,
            dst_scissor,
            ScaleMode::Upscale,
            step,
            wgpu::LoadOp::Clear(clear),
        );
        current_target = dst_target;
        current_size = dst_size;
    }
    (current_target, current_size)
}

fn append_postprocess(
    plan: &mut RenderPlan,
    viewport_size: (u32, u32),
    postprocess: DebugPostprocess,
    clear: wgpu::Color,
) {
    match postprocess {
        DebugPostprocess::None => {}
        DebugPostprocess::OffscreenBlit => {
            push_fullscreen_blit(
                plan,
                PlanTarget::Intermediate0,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                None,
                wgpu::LoadOp::Clear(clear),
            );
        }
        DebugPostprocess::Pixelate { scale } => {
            let steps = decompose_pixelate_scale(scale);
            let (current_target, current_size, stack) =
                if steps.len() >= 2 && steps[0] == 2 && steps[1] == 2 {
                    let half_quarter = append_downsample_half_quarter(
                        plan,
                        PlanTarget::Intermediate0,
                        viewport_size,
                        PlanTarget::Intermediate2,
                        PlanTarget::Intermediate1,
                        None,
                        viewport_size,
                        clear,
                    );

                    let mut stack = half_quarter.stack;
                    let (current_target, current_size, rest_stack) = append_downsample_chain(
                        plan,
                        half_quarter.quarter_target,
                        half_quarter.quarter_size,
                        &steps[2..],
                        half_quarter.half_target,
                        half_quarter.quarter_target,
                        None,
                        viewport_size,
                        clear,
                    );
                    stack.extend(rest_stack);
                    (current_target, current_size, stack)
                } else {
                    let first_step = steps[0];
                    let dst_size = downsampled_size(viewport_size, first_step);
                    push_scale_nearest(
                        plan,
                        PlanTarget::Intermediate0,
                        PlanTarget::Intermediate2,
                        viewport_size,
                        dst_size,
                        None,
                        ScaleMode::Downsample,
                        first_step,
                        wgpu::LoadOp::Clear(clear),
                    );
                    let mut stack = vec![(viewport_size, first_step)];

                    let (current_target, current_size, rest_stack) = append_downsample_chain(
                        plan,
                        PlanTarget::Intermediate2,
                        dst_size,
                        &steps[1..],
                        PlanTarget::Intermediate1,
                        PlanTarget::Intermediate2,
                        None,
                        viewport_size,
                        clear,
                    );
                    stack.extend(rest_stack);
                    (current_target, current_size, stack)
                };
            let (current_target, _current_size) = append_upsample_chain(
                plan,
                current_target,
                current_size,
                stack,
                None,
                viewport_size,
                clear,
            );
            push_fullscreen_blit(
                plan,
                current_target,
                PlanTarget::Output,
                viewport_size,
                viewport_size,
                None,
                wgpu::LoadOp::Clear(clear),
            );
        }
        DebugPostprocess::Blur {
            radius,
            downsample_scale,
            scissor,
        } => {
            let _radius = radius.max(1);
            let downsample_scale = if downsample_scale >= 4 { 4 } else { 2 };
            let use_quarter = downsample_scale == 4;

            let (blur_src, blur_size, scratch) = if use_quarter {
                (
                    PlanTarget::Intermediate1,
                    downsampled_size(viewport_size, 4),
                    PlanTarget::Intermediate2,
                )
            } else {
                (
                    PlanTarget::Intermediate2,
                    downsampled_size(viewport_size, 2),
                    PlanTarget::Intermediate1,
                )
            };

            let down_scissor = map_scissor_downsample_nearest(scissor, downsample_scale, blur_size);
            push_scale_nearest(
                plan,
                PlanTarget::Intermediate0,
                blur_src,
                viewport_size,
                blur_size,
                down_scissor,
                ScaleMode::Downsample,
                downsample_scale,
                wgpu::LoadOp::Clear(clear),
            );

            let blur_scissor = down_scissor;
            push_blur(
                plan,
                blur_src,
                scratch,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Horizontal,
                wgpu::LoadOp::Clear(clear),
            );
            push_blur(
                plan,
                scratch,
                blur_src,
                blur_size,
                blur_size,
                blur_scissor,
                BlurAxis::Vertical,
                wgpu::LoadOp::Clear(clear),
            );

            let final_scissor = map_scissor_to_size(scissor, viewport_size, viewport_size);
            if scissor.is_some() {
                push_fullscreen_blit(
                    plan,
                    PlanTarget::Intermediate0,
                    PlanTarget::Output,
                    viewport_size,
                    viewport_size,
                    None,
                    wgpu::LoadOp::Clear(clear),
                );
                push_scale_nearest(
                    plan,
                    blur_src,
                    PlanTarget::Output,
                    blur_size,
                    viewport_size,
                    final_scissor,
                    ScaleMode::Upscale,
                    downsample_scale,
                    wgpu::LoadOp::Load,
                );
            } else {
                push_scale_nearest(
                    plan,
                    blur_src,
                    PlanTarget::Intermediate0,
                    blur_size,
                    viewport_size,
                    final_scissor,
                    ScaleMode::Upscale,
                    downsample_scale,
                    wgpu::LoadOp::Clear(clear),
                );
                push_fullscreen_blit(
                    plan,
                    PlanTarget::Intermediate0,
                    PlanTarget::Output,
                    viewport_size,
                    viewport_size,
                    final_scissor,
                    wgpu::LoadOp::Clear(clear),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
