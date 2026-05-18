use super::*;
use crate::renderer::{estimate_clip_mask_bytes, estimate_texture_bytes};

pub(super) fn estimate_plan_peak_intermediate_bytes(
    passes: &[RenderPlanPass],
    scene_format: wgpu::TextureFormat,
) -> u64 {
    fn idx(t: PlanTarget) -> usize {
        match t {
            PlanTarget::Output => 0,
            PlanTarget::Intermediate0 => 1,
            PlanTarget::Intermediate1 => 2,
            PlanTarget::Intermediate2 => 3,
            PlanTarget::Intermediate3 => 4,
            PlanTarget::Mask0 => 5,
            PlanTarget::Mask1 => 6,
            PlanTarget::Mask2 => 7,
        }
    }

    fn target_format(t: PlanTarget, scene_format: wgpu::TextureFormat) -> wgpu::TextureFormat {
        match t {
            PlanTarget::Output
            | PlanTarget::Intermediate0
            | PlanTarget::Intermediate1
            | PlanTarget::Intermediate2
            | PlanTarget::Intermediate3 => scene_format,
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                wgpu::TextureFormat::R8Unorm
            }
        }
    }

    let mut live: [bool; 8] = [false; 8];
    let mut sizes: [(u32, u32); 8] = [(0, 0); 8];
    let mut peak: u64 = 0;

    fn mark_live(
        live: &mut [bool; 8],
        sizes: &mut [(u32, u32); 8],
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
            RenderPlanPass::Dither(DitherPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::Noise(NoisePass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::DropShadow(DropShadowPass { dst, dst_size, .. }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common: CustomEffectPassCommon { dst, dst_size, .. },
            }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common: CustomEffectPassCommon { dst, dst_size, .. },
                ..
            }) => {
                mark_live(&mut live, &mut sizes, dst, dst_size);
            }
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                common: CustomEffectPassCommon { dst, dst_size, .. },
                ..
            }) => {
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
            PlanTarget::Intermediate3,
            PlanTarget::Mask0,
            PlanTarget::Mask1,
            PlanTarget::Mask2,
        ] {
            if !live[idx(t)] {
                continue;
            }
            let bytes = match t {
                PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
                    estimate_clip_mask_bytes(sizes[idx(t)])
                }
                _ => estimate_texture_bytes(sizes[idx(t)], target_format(t, scene_format), 1),
            };
            cur = cur.saturating_add(bytes);
        }
        peak = peak.max(cur);
    }

    peak
}

pub(super) fn insert_early_releases(passes: &mut Vec<RenderPlanPass>) -> u64 {
    let mut last_use: [Option<usize>; 7] = [None, None, None, None, None, None, None];

    for (idx, pass) in passes.iter().enumerate() {
        let mut mark = |t: PlanTarget| {
            let slot = match t {
                PlanTarget::Intermediate0 => Some(0),
                PlanTarget::Intermediate1 => Some(1),
                PlanTarget::Intermediate2 => Some(2),
                PlanTarget::Intermediate3 => Some(3),
                PlanTarget::Mask0 => Some(4),
                PlanTarget::Mask1 => Some(5),
                PlanTarget::Mask2 => Some(6),
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
            RenderPlanPass::CustomEffect(p) => {
                mark(p.common.src);
                mark(p.common.dst);
                if let Some(mask) = p.common.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::CustomEffectV2(p) => {
                mark(p.common.src);
                mark(p.common.dst);
                if let Some(mask) = p.common.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::CustomEffectV3(p) => {
                mark(p.common.src);
                mark(p.src_raw);
                mark(p.src_pyramid);
                mark(p.common.dst);
                if let Some(mask) = p.common.mask {
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
            RenderPlanPass::Dither(p) => {
                mark(p.src);
                mark(p.dst);
                if let Some(mask) = p.mask {
                    mark(mask.target);
                }
            }
            RenderPlanPass::Noise(p) => {
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
    let last3 = last_use[3];
    let last_mask0 = last_use[4];
    let last_mask1 = last_use[5];
    let last_mask2 = last_use[6];

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
        if last3 == Some(idx) {
            push_release(PlanTarget::Intermediate3);
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
