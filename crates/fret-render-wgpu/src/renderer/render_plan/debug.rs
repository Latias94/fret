use super::*;
#[cfg(debug_assertions)]
use crate::renderer::frame_targets::downsampled_size;

impl RenderPlan {
    #[cfg(debug_assertions)]
    pub(in crate::renderer) fn debug_validate(&self) {
        if let Err(message) = validate_plan_target_lifetimes(&self.passes) {
            panic!("RenderPlan validation failed: {message}");
        }
        if let Err(message) = validate_plan_scissors(&self.passes) {
            panic!("RenderPlan validation failed: {message}");
        }
    }

    #[cfg(not(debug_assertions))]
    pub(in crate::renderer) fn debug_validate(&self) {}

    #[cfg(debug_assertions)]
    pub(in crate::renderer) fn debug_validate_first_output_write_is_clear(&self) {
        if let Err(message) = validate_plan_first_output_write_is_clear(&self.passes) {
            panic!("RenderPlan validation failed: {message}");
        }
    }

    #[cfg(not(debug_assertions))]
    pub(in crate::renderer) fn debug_validate_first_output_write_is_clear(&self) {}
}

#[cfg(debug_assertions)]
pub(super) fn validate_plan_target_lifetimes(passes: &[RenderPlanPass]) -> Result<(), String> {
    fn slot(t: PlanTarget) -> Option<usize> {
        match t {
            PlanTarget::Intermediate0 => Some(0),
            PlanTarget::Intermediate1 => Some(1),
            PlanTarget::Intermediate2 => Some(2),
            PlanTarget::Intermediate3 => Some(3),
            PlanTarget::Mask0 => Some(4),
            PlanTarget::Mask1 => Some(5),
            PlanTarget::Mask2 => Some(6),
            PlanTarget::Output => None,
        }
    }

    fn target_label(t: PlanTarget) -> &'static str {
        match t {
            PlanTarget::Output => "Output",
            PlanTarget::Intermediate0 => "Intermediate0",
            PlanTarget::Intermediate1 => "Intermediate1",
            PlanTarget::Intermediate2 => "Intermediate2",
            PlanTarget::Intermediate3 => "Intermediate3",
            PlanTarget::Mask0 => "Mask0",
            PlanTarget::Mask1 => "Mask1",
            PlanTarget::Mask2 => "Mask2",
        }
    }

    let mut live: [bool; 7] = [false; 7];
    let mut initialized: [bool; 7] = [false; 7];

    fn mark_read(
        live: &[bool; 7],
        initialized: &[bool; 7],
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
        live: &mut [bool; 7],
        initialized: &mut [bool; 7],
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
        live: &mut [bool; 7],
        initialized: &mut [bool; 7],
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
            RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass { target, load, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, target, Some(load))?;
            }
            RenderPlanPass::PathClipMask(PathClipMaskPass { dst, load, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ClipMask(ClipMaskPass { dst, load, .. }) => {
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
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
            RenderPlanPass::Dither(DitherPass {
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
            RenderPlanPass::Noise(NoisePass {
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
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common:
                    CustomEffectPassCommon {
                        src,
                        dst,
                        mask,
                        load,
                        ..
                    },
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common:
                    CustomEffectPassCommon {
                        src,
                        dst,
                        mask,
                        load,
                        ..
                    },
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                src_raw,
                src_pyramid,
                raw_wanted,
                common:
                    CustomEffectPassCommon {
                        src,
                        dst,
                        mask,
                        load,
                        ..
                    },
                ..
            }) => {
                mark_read(&live, &initialized, pass_index, src)?;
                // Custom V3 always binds both source views; the flags describe shader-requested
                // source semantics for diagnostics and summaries, not resource availability.
                let _ = raw_wanted;
                mark_read(&live, &initialized, pass_index, src_raw)?;
                mark_read(&live, &initialized, pass_index, src_pyramid)?;
                if let Some(mask) = mask {
                    mark_read(&live, &initialized, pass_index, mask.target)?;
                }
                mark_write(&mut live, &mut initialized, pass_index, dst, Some(load))?;
            }
            RenderPlanPass::ReleaseTarget(t) => {
                mark_release(&mut live, &mut initialized, pass_index, t)?;
            }
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
pub(super) fn validate_plan_scissors(passes: &[RenderPlanPass]) -> Result<(), String> {
    fn checked_end(start: u32, len: u32) -> Option<u32> {
        start.checked_add(len)
    }

    fn intersects_absolute(
        scissor: ScissorRect,
        dst_origin: (u32, u32),
        dst_size: (u32, u32),
    ) -> bool {
        if scissor.w == 0 || scissor.h == 0 || dst_size.0 == 0 || dst_size.1 == 0 {
            return false;
        }

        let sx0 = scissor.x;
        let sy0 = scissor.y;
        let Some(sx1) = checked_end(scissor.x, scissor.w) else {
            return false;
        };
        let Some(sy1) = checked_end(scissor.y, scissor.h) else {
            return false;
        };

        let dx0 = dst_origin.0;
        let dy0 = dst_origin.1;
        let Some(dx1) = checked_end(dst_origin.0, dst_size.0) else {
            return false;
        };
        let Some(dy1) = checked_end(dst_origin.1, dst_size.1) else {
            return false;
        };

        let ix0 = sx0.max(dx0);
        let iy0 = sy0.max(dy0);
        let ix1 = sx1.min(dx1);
        let iy1 = sy1.min(dy1);
        ix1 > ix0 && iy1 > iy0
    }

    fn within_local(scissor: ScissorRect, dst_size: (u32, u32)) -> bool {
        if scissor.w == 0 || scissor.h == 0 || dst_size.0 == 0 || dst_size.1 == 0 {
            return false;
        }
        let Some(x1) = checked_end(scissor.x, scissor.w) else {
            return false;
        };
        let Some(y1) = checked_end(scissor.y, scissor.h) else {
            return false;
        };
        x1 <= dst_size.0 && y1 <= dst_size.1
    }

    fn validate_mask_ref(
        pass_index: usize,
        pass_label: &'static str,
        dst_size: (u32, u32),
        mask: MaskRef,
    ) -> Result<(), String> {
        match mask.target {
            PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {}
            _ => {
                return Err(format!(
                    "pass[{pass_index}] {pass_label} mask target is not a mask PlanTarget"
                ));
            }
        }

        if mask.viewport_rect.w == 0 || mask.viewport_rect.h == 0 {
            return Err(format!(
                "pass[{pass_index}] {pass_label} mask viewport_rect is empty"
            ));
        }
        if !within_local(mask.viewport_rect, dst_size) {
            return Err(format!(
                "pass[{pass_index}] {pass_label} mask viewport_rect exceeds destination size"
            ));
        }

        let base = (mask.viewport_rect.w.max(1), mask.viewport_rect.h.max(1));
        let expected = match mask.target {
            PlanTarget::Mask0 => base,
            PlanTarget::Mask1 => downsampled_size(base, 2),
            PlanTarget::Mask2 => downsampled_size(base, 4),
            _ => unreachable!("non-mask targets rejected above"),
        };
        if mask.size != expected {
            return Err(format!(
                "pass[{pass_index}] {pass_label} mask size mismatch (expected {:?}, got {:?})",
                expected, mask.size
            ));
        }

        Ok(())
    }

    fn validate_origin_size(
        pass_index: usize,
        pass_label: &'static str,
        origin: (u32, u32),
        size: (u32, u32),
    ) -> Result<(), String> {
        if checked_end(origin.0, size.0).is_none() || checked_end(origin.1, size.1).is_none() {
            return Err(format!(
                "pass[{pass_index}] {pass_label} origin+size overflows u32"
            ));
        }
        Ok(())
    }

    for (pass_index, pass) in passes.iter().enumerate() {
        match pass {
            RenderPlanPass::SceneDrawRange(pass) => {
                validate_origin_size(
                    pass_index,
                    "SceneDrawRange",
                    pass.target_origin,
                    pass.target_size,
                )?;
            }
            RenderPlanPass::PathClipMask(pass) => {
                validate_origin_size(pass_index, "PathClipMask", pass.dst_origin, pass.dst_size)?;
                if !intersects_absolute(pass.scissor.0, pass.dst_origin, pass.dst_size) {
                    return Err(format!(
                        "pass[{pass_index}] PathClipMask scissor does not intersect destination"
                    ));
                }
            }
            RenderPlanPass::PathMsaaBatch(pass) => {
                validate_origin_size(
                    pass_index,
                    "PathMsaaBatch",
                    pass.target_origin,
                    pass.target_size,
                )?;
                if !intersects_absolute(pass.union_scissor.0, pass.target_origin, pass.target_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] PathMsaaBatch union scissor does not intersect target"
                    ));
                }
            }
            RenderPlanPass::CompositePremul(pass) => {
                validate_origin_size(
                    pass_index,
                    "CompositePremul dst",
                    pass.dst_origin,
                    pass.dst_size,
                )?;
                validate_origin_size(
                    pass_index,
                    "CompositePremul src",
                    pass.src_origin,
                    pass.src_size,
                )?;
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !intersects_absolute(scissor, pass.dst_origin, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] CompositePremul dst_scissor does not intersect destination"
                    ));
                }

                if let Some(mask) = pass.mask {
                    validate_mask_ref(pass_index, "CompositePremul", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::ScaleNearest(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] ScaleNearest dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if !matches!(pass.mode, ScaleMode::Upscale) {
                        return Err(format!(
                            "pass[{pass_index}] ScaleNearest mask requires ScaleMode::Upscale"
                        ));
                    }
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] ScaleNearest mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "ScaleNearest", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::Blur(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] Blur dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] Blur mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "Blur", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::BackdropWarp(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] BackdropWarp dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] BackdropWarp mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "BackdropWarp", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::ColorAdjust(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] ColorAdjust dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] ColorAdjust mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "ColorAdjust", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::ColorMatrix(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] ColorMatrix dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] ColorMatrix mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "ColorMatrix", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::AlphaThreshold(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] AlphaThreshold dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] AlphaThreshold mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "AlphaThreshold", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::Dither(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] Dither dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] Dither mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "Dither", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::Noise(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] Noise dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] Noise mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "Noise", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::DropShadow(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] DropShadow dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.mask {
                    if pass.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] DropShadow mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "DropShadow", pass.dst_size, mask)?;
                }
            }
            RenderPlanPass::CustomEffect(pass) => {
                if let Some(scissor) = pass.common.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.common.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] CustomEffect dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.common.mask {
                    if pass.common.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] CustomEffect mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "CustomEffect", pass.common.dst_size, mask)?;
                }
            }
            RenderPlanPass::CustomEffectV2(pass) => {
                if let Some(scissor) = pass.common.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.common.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] CustomEffectV2 dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.common.mask {
                    if pass.common.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] CustomEffectV2 mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "CustomEffectV2", pass.common.dst_size, mask)?;
                }
            }
            RenderPlanPass::CustomEffectV3(pass) => {
                if let Some(scissor) = pass.common.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.common.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] CustomEffectV3 dst_scissor exceeds destination size"
                    ));
                }
                if let Some(mask) = pass.common.mask {
                    if pass.common.mask_uniform_index.is_none() {
                        return Err(format!(
                            "pass[{pass_index}] CustomEffectV3 mask requires mask_uniform_index"
                        ));
                    }
                    validate_mask_ref(pass_index, "CustomEffectV3", pass.common.dst_size, mask)?;
                }
            }
            RenderPlanPass::FullscreenBlit(pass) => {
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] FullscreenBlit dst_scissor exceeds destination size"
                    ));
                }
            }
            RenderPlanPass::ClipMask(pass) => {
                if !matches!(pass.load, wgpu::LoadOp::Clear(_)) {
                    return Err(format!(
                        "pass[{pass_index}] ClipMask must clear its destination"
                    ));
                }
                if let Some(scissor) = pass.dst_scissor.map(|s| s.0)
                    && !within_local(scissor, pass.dst_size)
                {
                    return Err(format!(
                        "pass[{pass_index}] ClipMask dst_scissor exceeds destination size"
                    ));
                }
            }
            RenderPlanPass::ReleaseTarget(_) => {}
        }
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn validate_plan_first_output_write_is_clear(passes: &[RenderPlanPass]) -> Result<(), String> {
    fn output_write_load(pass: &RenderPlanPass) -> Option<wgpu::LoadOp<wgpu::Color>> {
        match *pass {
            RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                target: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                target: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::CompositePremul(CompositePremulPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::ScaleNearest(ScaleNearestPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::Blur(BlurPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::BackdropWarp(BackdropWarpPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::ColorAdjust(ColorAdjustPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::ColorMatrix(ColorMatrixPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::AlphaThreshold(AlphaThresholdPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::Dither(DitherPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::Noise(NoisePass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::DropShadow(DropShadowPass {
                dst: PlanTarget::Output,
                load,
                ..
            }) => Some(load),
            RenderPlanPass::CustomEffect(CustomEffectPass {
                common:
                    CustomEffectPassCommon {
                        dst: PlanTarget::Output,
                        load,
                        ..
                    },
            }) => Some(load),
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                common:
                    CustomEffectPassCommon {
                        dst: PlanTarget::Output,
                        load,
                        ..
                    },
                ..
            }) => Some(load),
            RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
                common:
                    CustomEffectPassCommon {
                        dst: PlanTarget::Output,
                        load,
                        ..
                    },
                ..
            }) => Some(load),
            _ => None,
        }
    }

    let Some((pass_index, load)) = passes
        .iter()
        .enumerate()
        .find_map(|(ix, p)| output_write_load(p).map(|load| (ix, load)))
    else {
        return Err("plan contains no Output writes".to_string());
    };

    if matches!(load, wgpu::LoadOp::Clear(_)) {
        Ok(())
    } else {
        Err(format!(
            "pass[{pass_index}] first Output write uses LoadOp::Load; prefer LoadOp::Clear for deterministic output"
        ))
    }
}
