use super::super::frame_targets::FrameTargets;
use super::super::intermediate_pool::IntermediatePool;
use super::super::*;

impl Renderer {
    pub(super) fn pick_image_bind_group(
        &self,
        image: fret_core::ImageId,
        sampling: fret_core::scene::ImageSamplingHint,
    ) -> Option<&wgpu::BindGroup> {
        let (linear, nearest) = self.bind_group_caches.get_image_bind_groups(image)?;
        match sampling {
            fret_core::scene::ImageSamplingHint::Nearest => Some(nearest),
            fret_core::scene::ImageSamplingHint::Default
            | fret_core::scene::ImageSamplingHint::Linear => Some(linear),
        }
    }

    pub(super) fn pick_uniform_bind_group_for_mask_image(
        &self,
        mask_image: Option<UniformMaskImageSelection>,
    ) -> &wgpu::BindGroup {
        let Some(sel) = mask_image else {
            return &self.uniform_bind_group;
        };
        let Some((linear, nearest)) = self
            .bind_group_caches
            .get_uniform_mask_image_bind_groups(sel.image)
        else {
            return &self.uniform_bind_group;
        };
        match sel.sampling {
            fret_core::scene::ImageSamplingHint::Nearest => nearest,
            fret_core::scene::ImageSamplingHint::Default
            | fret_core::scene::ImageSamplingHint::Linear => linear,
        }
    }
}

pub(super) fn set_scissor_rect_absolute(
    rp: &mut wgpu::RenderPass<'_>,
    scissor: ScissorRect,
    dst_origin: (u32, u32),
    dst_size: (u32, u32),
) -> bool {
    if scissor.w == 0 || scissor.h == 0 || dst_size.0 == 0 || dst_size.1 == 0 {
        return false;
    }

    let x0 = scissor.x;
    let y0 = scissor.y;
    let x1 = scissor.x.saturating_add(scissor.w);
    let y1 = scissor.y.saturating_add(scissor.h);

    let lx0 = x0.saturating_sub(dst_origin.0).min(dst_size.0);
    let ly0 = y0.saturating_sub(dst_origin.1).min(dst_size.1);
    let lx1 = x1.saturating_sub(dst_origin.0).min(dst_size.0);
    let ly1 = y1.saturating_sub(dst_origin.1).min(dst_size.1);

    let w = lx1.saturating_sub(lx0);
    let h = ly1.saturating_sub(ly0);
    if w == 0 || h == 0 {
        return false;
    }

    rp.set_scissor_rect(lx0, ly0, w, h);
    true
}

pub(super) fn set_scissor_rect_absolute_opt(
    rp: &mut wgpu::RenderPass<'_>,
    scissor: Option<AbsoluteScissorRect>,
    dst_origin: (u32, u32),
    dst_size: (u32, u32),
) -> bool {
    let Some(scissor) = scissor else {
        return false;
    };
    set_scissor_rect_absolute(rp, scissor.0, dst_origin, dst_size)
}

pub(super) fn set_scissor_rect_local(
    rp: &mut wgpu::RenderPass<'_>,
    scissor: LocalScissorRect,
    dst_size: (u32, u32),
) -> bool {
    if dst_size.0 == 0 || dst_size.1 == 0 {
        return false;
    }

    let scissor = scissor.0;
    if scissor.w == 0 || scissor.h == 0 {
        return false;
    }

    let x0 = scissor.x.min(dst_size.0);
    let y0 = scissor.y.min(dst_size.1);
    let x1 = scissor.x.saturating_add(scissor.w).min(dst_size.0);
    let y1 = scissor.y.saturating_add(scissor.h).min(dst_size.1);

    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    if w == 0 || h == 0 {
        return false;
    }

    rp.set_scissor_rect(x0, y0, w, h);
    true
}

pub(super) fn set_scissor_rect_local_opt(
    rp: &mut wgpu::RenderPass<'_>,
    scissor: Option<LocalScissorRect>,
    dst_size: (u32, u32),
) -> bool {
    let Some(scissor) = scissor else {
        return false;
    };
    set_scissor_rect_local(rp, scissor, dst_size)
}

pub(super) fn render_plan_pass_trace_kind(pass: &RenderPlanPass) -> &'static str {
    match pass {
        RenderPlanPass::SceneDrawRange(_) => "scene_draw_range",
        RenderPlanPass::PathMsaaBatch(_) => "path_msaa_batch",
        RenderPlanPass::PathClipMask(_) => "path_clip_mask",
        RenderPlanPass::CompositePremul(_) => "composite_premul",
        RenderPlanPass::ScaleNearest(_) => "scale_nearest",
        RenderPlanPass::Blur(_) => "blur",
        RenderPlanPass::BackdropWarp(_) => "backdrop_warp",
        RenderPlanPass::ColorAdjust(_) => "color_adjust",
        RenderPlanPass::ColorMatrix(_) => "color_matrix",
        RenderPlanPass::AlphaThreshold(_) => "alpha_threshold",
        RenderPlanPass::DropShadow(_) => "drop_shadow",
        RenderPlanPass::FullscreenBlit(_) => "fullscreen_blit",
        RenderPlanPass::ClipMask(_) => "clip_mask",
        RenderPlanPass::ReleaseTarget(_) => "release_target",
    }
}

pub(super) struct RenderPlanPassTraceMeta {
    pub(super) src: Option<PlanTarget>,
    pub(super) dst: Option<PlanTarget>,
    pub(super) load: Option<&'static str>,
    pub(super) scissor: Option<ScissorRect>,
    pub(super) render_origin: Option<(u32, u32)>,
    pub(super) render_size: Option<(u32, u32)>,
}

pub(super) fn render_plan_pass_trace_meta(pass: &RenderPlanPass) -> RenderPlanPassTraceMeta {
    fn load_label(load: wgpu::LoadOp<wgpu::Color>) -> &'static str {
        match load {
            wgpu::LoadOp::Load => "load",
            wgpu::LoadOp::Clear(_) => "clear",
            wgpu::LoadOp::DontCare(_) => "dont_care",
        }
    }

    let (render_origin, render_size) = render_plan_pass_render_space(pass)
        .map(|(origin, size)| (Some(origin), Some(size)))
        .unwrap_or((None, None));

    match pass {
        RenderPlanPass::SceneDrawRange(pass) => RenderPlanPassTraceMeta {
            src: None,
            dst: Some(pass.target),
            load: Some(load_label(pass.load)),
            scissor: None,
            render_origin,
            render_size,
        },
        RenderPlanPass::PathMsaaBatch(pass) => RenderPlanPassTraceMeta {
            src: None,
            dst: Some(pass.target),
            load: Some(load_label(pass.load)),
            scissor: Some(pass.union_scissor.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::PathClipMask(pass) => RenderPlanPassTraceMeta {
            src: None,
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: Some(pass.scissor.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::FullscreenBlit(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::CompositePremul(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::ScaleNearest(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::Blur(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::BackdropWarp(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::ColorAdjust(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::ColorMatrix(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::AlphaThreshold(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::DropShadow(pass) => RenderPlanPassTraceMeta {
            src: Some(pass.src),
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::ClipMask(pass) => RenderPlanPassTraceMeta {
            src: None,
            dst: Some(pass.dst),
            load: Some(load_label(pass.load)),
            scissor: pass.dst_scissor.map(|s| s.0),
            render_origin,
            render_size,
        },
        RenderPlanPass::ReleaseTarget(target) => RenderPlanPassTraceMeta {
            src: None,
            dst: Some(*target),
            load: None,
            scissor: None,
            render_origin,
            render_size,
        },
    }
}

pub(super) fn render_plan_trace_fingerprint(passes: &[RenderPlanPass]) -> u64 {
    fn mix_fnv1a(hash: u64, value: u64) -> u64 {
        (hash ^ value).wrapping_mul(0x100_0000_01B3)
    }

    fn mix_str(mut hash: u64, s: &str) -> u64 {
        for b in s.as_bytes() {
            hash = mix_fnv1a(hash, u64::from(*b));
        }
        hash
    }

    fn pack_scissor(s: Option<ScissorRect>) -> u64 {
        let Some(s) = s else {
            return 0;
        };
        (u64::from(s.x) << 48) ^ (u64::from(s.y) << 32) ^ (u64::from(s.w) << 16) ^ u64::from(s.h)
    }

    fn pack_point(p: Option<(u32, u32)>) -> u64 {
        let Some(p) = p else {
            return 0;
        };
        u64::from(p.0) << 32 | u64::from(p.1)
    }

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    hash = mix_fnv1a(hash, passes.len() as u64);

    for (pass_index, pass) in passes.iter().enumerate() {
        hash = mix_fnv1a(hash, pass_index as u64);
        hash = mix_str(hash, render_plan_pass_trace_kind(pass));

        let meta = render_plan_pass_trace_meta(pass);
        hash = mix_fnv1a(hash, meta.src.map(|t| t as u64 + 1).unwrap_or(0));
        hash = mix_fnv1a(hash, meta.dst.map(|t| t as u64 + 1).unwrap_or(0));
        hash = mix_fnv1a(hash, meta.load.map(|s| mix_str(0, s)).unwrap_or(0));
        hash = mix_fnv1a(hash, pack_scissor(meta.scissor));
        hash = mix_fnv1a(hash, pack_point(meta.render_origin));
        hash = mix_fnv1a(hash, pack_point(meta.render_size));
    }

    hash
}

pub(super) fn render_plan_pass_render_space(
    pass: &RenderPlanPass,
) -> Option<((u32, u32), (u32, u32))> {
    match pass {
        RenderPlanPass::SceneDrawRange(pass) => Some((pass.target_origin, pass.target_size)),
        RenderPlanPass::PathMsaaBatch(pass) => Some((pass.target_origin, pass.target_size)),
        RenderPlanPass::PathClipMask(pass) => Some((pass.dst_origin, pass.dst_size)),
        RenderPlanPass::CompositePremul(pass) => Some((pass.dst_origin, pass.dst_size)),
        RenderPlanPass::ScaleNearest(pass) => Some((pass.dst_origin, pass.dst_size)),
        RenderPlanPass::Blur(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::BackdropWarp(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::ColorAdjust(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::ColorMatrix(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::AlphaThreshold(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::DropShadow(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::FullscreenBlit(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::ClipMask(pass) => Some(((0, 0), pass.dst_size)),
        RenderPlanPass::ReleaseTarget(_) => None,
    }
}

pub(super) fn require_color_src_view(
    frame_targets: &FrameTargets,
    src: PlanTarget,
    src_size: (u32, u32),
    pass_name: &'static str,
) -> Option<wgpu::TextureView> {
    match src {
        PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
            Some(frame_targets.require_target(src, src_size))
        }
        PlanTarget::Output | PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            debug_assert!(false, "{pass_name} src cannot be Output/mask targets");
            None
        }
    }
}

pub(super) fn ensure_color_dst_view_owned(
    frame_targets: &mut FrameTargets,
    pool: &mut IntermediatePool,
    device: &wgpu::Device,
    dst: PlanTarget,
    dst_size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    pass_name: &'static str,
) -> Option<wgpu::TextureView> {
    match dst {
        PlanTarget::Output => None,
        PlanTarget::Intermediate0 | PlanTarget::Intermediate1 | PlanTarget::Intermediate2 => {
            Some(frame_targets.ensure_target(pool, device, dst, dst_size, format, usage))
        }
        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            debug_assert!(false, "{pass_name} dst cannot be mask targets");
            None
        }
    }
}

pub(super) fn require_mask_view(
    frame_targets: &FrameTargets,
    mask_target: PlanTarget,
    mask_size: (u32, u32),
    pass_name: &'static str,
) -> Option<wgpu::TextureView> {
    match mask_target {
        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            Some(frame_targets.require_target(mask_target, mask_size))
        }
        PlanTarget::Output
        | PlanTarget::Intermediate0
        | PlanTarget::Intermediate1
        | PlanTarget::Intermediate2 => {
            debug_assert!(false, "{pass_name} mask target must be Mask[0-2]");
            None
        }
    }
}

pub(super) fn ensure_mask_dst_view(
    frame_targets: &mut FrameTargets,
    pool: &mut IntermediatePool,
    device: &wgpu::Device,
    dst: PlanTarget,
    dst_size: (u32, u32),
    usage: wgpu::TextureUsages,
    pass_name: &'static str,
) -> Option<wgpu::TextureView> {
    match dst {
        PlanTarget::Mask0 | PlanTarget::Mask1 | PlanTarget::Mask2 => {
            Some(frame_targets.ensure_target(
                pool,
                device,
                dst,
                dst_size,
                wgpu::TextureFormat::R8Unorm,
                usage,
            ))
        }
        PlanTarget::Output
        | PlanTarget::Intermediate0
        | PlanTarget::Intermediate1
        | PlanTarget::Intermediate2 => {
            debug_assert!(false, "{pass_name} dst must be Mask[0-2]");
            None
        }
    }
}
