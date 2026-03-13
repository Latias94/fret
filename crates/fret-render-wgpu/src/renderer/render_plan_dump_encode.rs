use super::render_plan::{
    AbsoluteScissorRect, BlurAxis, DebugPostprocess, LocalScissorRect, MaskRef, PlanTarget,
    RenderPlanPass, ScaleMode,
};
use super::{EffectMarker, EffectMarkerKind, ScissorRect};

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpScissorRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl From<ScissorRect> for JsonDumpScissorRect {
    fn from(r: ScissorRect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        }
    }
}

impl From<AbsoluteScissorRect> for JsonDumpScissorRect {
    fn from(r: AbsoluteScissorRect) -> Self {
        r.0.into()
    }
}

impl From<LocalScissorRect> for JsonDumpScissorRect {
    fn from(r: LocalScissorRect) -> Self {
        r.0.into()
    }
}

#[derive(Debug, serde::Serialize)]
pub(super) struct JsonDumpMaskRef {
    target: &'static str,
    size: [u32; 2],
    viewport_rect: JsonDumpScissorRect,
}

impl From<MaskRef> for JsonDumpMaskRef {
    fn from(m: MaskRef) -> Self {
        Self {
            target: plan_target_name(m.target),
            size: [m.size.0, m.size.1],
            viewport_rect: m.viewport_rect.into(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
pub(super) enum JsonDumpLoadOp {
    Clear { rgba: [f64; 4] },
    Load,
    DontCare,
}

fn encode_load_op(load: wgpu::LoadOp<wgpu::Color>) -> JsonDumpLoadOp {
    match load {
        wgpu::LoadOp::Clear(c) => JsonDumpLoadOp::Clear {
            rgba: [c.r, c.g, c.b, c.a],
        },
        wgpu::LoadOp::Load => JsonDumpLoadOp::Load,
        wgpu::LoadOp::DontCare(_) => JsonDumpLoadOp::DontCare,
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
pub(super) enum JsonDumpDebugPostprocess {
    None,
    OffscreenBlit {
        src: &'static str,
    },
    Pixelate {
        scale: u32,
    },
    Blur {
        radius: u32,
        downsample_scale: u32,
        scissor: Option<JsonDumpScissorRect>,
    },
}

pub(super) fn encode_debug_postprocess(p: DebugPostprocess) -> JsonDumpDebugPostprocess {
    match p {
        DebugPostprocess::None => JsonDumpDebugPostprocess::None,
        DebugPostprocess::OffscreenBlit { src } => JsonDumpDebugPostprocess::OffscreenBlit {
            src: plan_target_name(src),
        },
        DebugPostprocess::Pixelate { scale } => JsonDumpDebugPostprocess::Pixelate { scale },
        DebugPostprocess::Blur {
            radius,
            downsample_scale,
            scissor,
        } => JsonDumpDebugPostprocess::Blur {
            radius,
            downsample_scale,
            scissor: scissor.map(Into::into),
        },
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
pub(super) enum JsonDumpEffectMarker {
    Push {
        draw_ix: usize,
        scissor: JsonDumpScissorRect,
        scissor_space: &'static str,
        uniform_index: u32,
        mode: String,
        quality: String,
        chain: String,
        opacity: Option<f32>,
    },
    Pop {
        draw_ix: usize,
    },
}

pub(super) fn encode_effect_marker(m: EffectMarker) -> JsonDumpEffectMarker {
    match m.kind {
        EffectMarkerKind::Push {
            scissor,
            uniform_index,
            mode,
            chain,
            quality,
        } => JsonDumpEffectMarker::Push {
            draw_ix: m.draw_ix,
            scissor: scissor.into(),
            scissor_space: "absolute",
            uniform_index,
            mode: format!("{mode:?}"),
            quality: format!("{quality:?}"),
            chain: format!("{chain:?}"),
            opacity: None,
        },
        EffectMarkerKind::Pop => JsonDumpEffectMarker::Pop { draw_ix: m.draw_ix },
        EffectMarkerKind::ClipPathPush {
            scissor,
            uniform_index,
            mask_draw_index,
        } => JsonDumpEffectMarker::Push {
            draw_ix: m.draw_ix,
            scissor: scissor.into(),
            scissor_space: "absolute",
            uniform_index,
            mode: "ClipPath".to_string(),
            quality: "N/A".to_string(),
            chain: format!("mask_draw_index={mask_draw_index}"),
            opacity: None,
        },
        EffectMarkerKind::ClipPathPop => JsonDumpEffectMarker::Pop { draw_ix: m.draw_ix },
        EffectMarkerKind::CompositeGroupPush {
            scissor,
            uniform_index,
            mode,
            quality,
            opacity,
        } => JsonDumpEffectMarker::Push {
            draw_ix: m.draw_ix,
            scissor: scissor.into(),
            scissor_space: "absolute",
            uniform_index,
            mode: format!("CompositeGroup({mode:?})"),
            quality: format!("{quality:?}"),
            chain: String::new(),
            opacity: Some(opacity),
        },
        EffectMarkerKind::CompositeGroupPop => JsonDumpEffectMarker::Pop { draw_ix: m.draw_ix },
        EffectMarkerKind::BackdropSourceGroupPush {
            scissor,
            pyramid,
            quality,
        } => JsonDumpEffectMarker::Push {
            draw_ix: m.draw_ix,
            scissor: scissor.into(),
            scissor_space: "absolute",
            uniform_index: 0,
            mode: "BackdropSourceGroup".to_string(),
            quality: format!("{quality:?}"),
            chain: format!("pyramid={pyramid:?}"),
            opacity: None,
        },
        EffectMarkerKind::BackdropSourceGroupPop => {
            JsonDumpEffectMarker::Pop { draw_ix: m.draw_ix }
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
pub(super) enum JsonDumpPass {
    SceneDrawRange {
        segment: usize,
        target: &'static str,
        target_origin: [u32; 2],
        target_size: [u32; 2],
        load: JsonDumpLoadOp,
        draw_range: [usize; 2],
    },
    PathMsaaBatch {
        segment: usize,
        target: &'static str,
        target_origin: [u32; 2],
        target_size: [u32; 2],
        draw_range: [usize; 2],
        union_scissor: JsonDumpScissorRect,
        union_scissor_space: &'static str,
        batch_uniform_index: u32,
        load: JsonDumpLoadOp,
    },
    PathClipMask {
        dst: &'static str,
        dst_origin: [u32; 2],
        dst_size: [u32; 2],
        scissor: JsonDumpScissorRect,
        scissor_space: &'static str,
        uniform_index: u32,
        first_vertex: u32,
        vertex_count: u32,
        load: JsonDumpLoadOp,
    },
    FullscreenBlit {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        encode_output_srgb: bool,
        load: JsonDumpLoadOp,
    },
    CompositePremul {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        src_origin: [u32; 2],
        dst_origin: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        opacity: f32,
        load: JsonDumpLoadOp,
    },
    ScaleNearest {
        mode: &'static str,
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        src_origin: [u32; 2],
        dst_origin: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        scale: u32,
        load: JsonDumpLoadOp,
    },
    Blur {
        axis: &'static str,
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        load: JsonDumpLoadOp,
    },
    BackdropWarp {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        origin_px: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        strength_px: f32,
        scale_px: f32,
        phase: f32,
        chromatic_aberration_px: f32,
        warp_kind: &'static str,
        load: JsonDumpLoadOp,
    },
    ColorAdjust {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        saturation: f32,
        brightness: f32,
        contrast: f32,
        load: JsonDumpLoadOp,
    },
    ColorMatrix {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        matrix: [f32; 20],
        load: JsonDumpLoadOp,
    },
    AlphaThreshold {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        cutoff: f32,
        soft: f32,
        load: JsonDumpLoadOp,
    },
    Dither {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        mode: &'static str,
        load: JsonDumpLoadOp,
    },
    Noise {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        strength: f32,
        scale_px: f32,
        phase: f32,
        load: JsonDumpLoadOp,
    },
    DropShadow {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        offset_px: [f32; 2],
        color: [f32; 4],
        load: JsonDumpLoadOp,
    },
    CustomEffect {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        effect: String,
        load: JsonDumpLoadOp,
    },
    CustomEffectV2 {
        src: &'static str,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        effect: String,
        input_image: Option<String>,
        input_uv: [f32; 4],
        input_sampling: &'static str,
        load: JsonDumpLoadOp,
    },
    CustomEffectV3 {
        src: &'static str,
        src_raw: &'static str,
        src_pyramid: &'static str,
        pyramid_levels: u32,
        pyramid_build_scissor: Option<JsonDumpScissorRect>,
        pyramid_build_scissor_space: Option<&'static str>,
        raw_wanted: bool,
        pyramid_wanted: bool,
        dst: &'static str,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        effect: String,
        user0_image: Option<String>,
        user0_uv: [f32; 4],
        user0_sampling: &'static str,
        user1_image: Option<String>,
        user1_uv: [f32; 4],
        user1_sampling: &'static str,
        load: JsonDumpLoadOp,
    },
    ClipMask {
        dst: &'static str,
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        dst_scissor_space: Option<&'static str>,
        uniform_index: u32,
        load: JsonDumpLoadOp,
    },
    ReleaseTarget {
        target: &'static str,
    },
}

pub(super) fn plan_target_name(t: PlanTarget) -> &'static str {
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

pub(super) fn encode_pass(p: &RenderPlanPass) -> JsonDumpPass {
    match p {
        RenderPlanPass::SceneDrawRange(pass) => JsonDumpPass::SceneDrawRange {
            segment: pass.segment.0,
            target: plan_target_name(pass.target),
            target_origin: [pass.target_origin.0, pass.target_origin.1],
            target_size: [pass.target_size.0, pass.target_size.1],
            load: encode_load_op(pass.load),
            draw_range: [pass.draw_range.start, pass.draw_range.end],
        },
        RenderPlanPass::PathMsaaBatch(pass) => JsonDumpPass::PathMsaaBatch {
            segment: pass.segment.0,
            target: plan_target_name(pass.target),
            target_origin: [pass.target_origin.0, pass.target_origin.1],
            target_size: [pass.target_size.0, pass.target_size.1],
            draw_range: [pass.draw_range.start, pass.draw_range.end],
            union_scissor: pass.union_scissor.into(),
            union_scissor_space: "absolute",
            batch_uniform_index: pass.batch_uniform_index,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::PathClipMask(pass) => JsonDumpPass::PathClipMask {
            dst: plan_target_name(pass.dst),
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            scissor: pass.scissor.into(),
            scissor_space: "absolute",
            uniform_index: pass.uniform_index,
            first_vertex: pass.first_vertex,
            vertex_count: pass.vertex_count,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::FullscreenBlit(pass) => JsonDumpPass::FullscreenBlit {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            encode_output_srgb: pass.encode_output_srgb,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::CompositePremul(pass) => JsonDumpPass::CompositePremul {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            src_origin: [pass.src_origin.0, pass.src_origin.1],
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "absolute"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            opacity: pass.opacity,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ScaleNearest(pass) => JsonDumpPass::ScaleNearest {
            mode: match pass.mode {
                ScaleMode::Downsample => "Downsample",
                ScaleMode::Upscale => "Upscale",
            },
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            src_origin: [pass.src_origin.0, pass.src_origin.1],
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            scale: pass.scale,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::Blur(pass) => JsonDumpPass::Blur {
            axis: match pass.axis {
                BlurAxis::Horizontal => "Horizontal",
                BlurAxis::Vertical => "Vertical",
            },
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::BackdropWarp(pass) => JsonDumpPass::BackdropWarp {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            origin_px: [pass.origin_px.0, pass.origin_px.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            strength_px: pass.strength_px,
            scale_px: pass.scale_px,
            phase: pass.phase,
            chromatic_aberration_px: pass.chromatic_aberration_px,
            warp_kind: match pass.kind {
                fret_core::scene::BackdropWarpKindV1::Wave => "Wave",
                fret_core::scene::BackdropWarpKindV1::LensReserved => "LensReserved",
            },
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ColorAdjust(pass) => JsonDumpPass::ColorAdjust {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            saturation: pass.saturation,
            brightness: pass.brightness,
            contrast: pass.contrast,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ColorMatrix(pass) => JsonDumpPass::ColorMatrix {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            matrix: pass.matrix,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::AlphaThreshold(pass) => JsonDumpPass::AlphaThreshold {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            cutoff: pass.cutoff,
            soft: pass.soft,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::Dither(pass) => JsonDumpPass::Dither {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            mode: match pass.mode {
                fret_core::DitherMode::Bayer4x4 => "Bayer4x4",
            },
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::Noise(pass) => JsonDumpPass::Noise {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            strength: pass.strength,
            scale_px: pass.scale_px,
            phase: pass.phase,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::DropShadow(pass) => JsonDumpPass::DropShadow {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            offset_px: [pass.offset_px.0, pass.offset_px.1],
            color: [pass.color.r, pass.color.g, pass.color.b, pass.color.a],
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::CustomEffect(pass) => JsonDumpPass::CustomEffect {
            src: plan_target_name(pass.common.src),
            dst: plan_target_name(pass.common.dst),
            src_size: [pass.common.src_size.0, pass.common.src_size.1],
            dst_size: [pass.common.dst_size.0, pass.common.dst_size.1],
            dst_scissor: pass.common.dst_scissor.map(Into::into),
            dst_scissor_space: pass.common.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.common.mask_uniform_index,
            mask: pass.common.mask.map(Into::into),
            effect: format!("{:?}", pass.common.effect),
            load: encode_load_op(pass.common.load),
        },
        RenderPlanPass::CustomEffectV2(pass) => JsonDumpPass::CustomEffectV2 {
            src: plan_target_name(pass.common.src),
            dst: plan_target_name(pass.common.dst),
            src_size: [pass.common.src_size.0, pass.common.src_size.1],
            dst_size: [pass.common.dst_size.0, pass.common.dst_size.1],
            dst_scissor: pass.common.dst_scissor.map(Into::into),
            dst_scissor_space: pass.common.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.common.mask_uniform_index,
            mask: pass.common.mask.map(Into::into),
            effect: format!("{:?}", pass.common.effect),
            input_image: pass.input_image.map(|id| format!("{id:?}")),
            input_uv: [
                pass.input_uv.u0,
                pass.input_uv.v0,
                pass.input_uv.u1,
                pass.input_uv.v1,
            ],
            input_sampling: match pass.input_sampling {
                fret_core::scene::ImageSamplingHint::Default => "default",
                fret_core::scene::ImageSamplingHint::Linear => "linear",
                fret_core::scene::ImageSamplingHint::Nearest => "nearest",
            },
            load: encode_load_op(pass.common.load),
        },
        RenderPlanPass::CustomEffectV3(pass) => JsonDumpPass::CustomEffectV3 {
            src: plan_target_name(pass.common.src),
            src_raw: plan_target_name(pass.src_raw),
            src_pyramid: plan_target_name(pass.src_pyramid),
            pyramid_levels: pass.pyramid_levels,
            pyramid_build_scissor: pass.pyramid_build_scissor.map(Into::into),
            pyramid_build_scissor_space: pass.pyramid_build_scissor.map(|_| "dst_local"),
            raw_wanted: pass.raw_wanted,
            pyramid_wanted: pass.pyramid_wanted,
            dst: plan_target_name(pass.common.dst),
            src_size: [pass.common.src_size.0, pass.common.src_size.1],
            dst_size: [pass.common.dst_size.0, pass.common.dst_size.1],
            dst_scissor: pass.common.dst_scissor.map(Into::into),
            dst_scissor_space: pass.common.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.common.mask_uniform_index,
            mask: pass.common.mask.map(Into::into),
            effect: format!("{:?}", pass.common.effect),
            user0_image: pass.user0_image.map(|id| format!("{id:?}")),
            user0_uv: [
                pass.user0_uv.u0,
                pass.user0_uv.v0,
                pass.user0_uv.u1,
                pass.user0_uv.v1,
            ],
            user0_sampling: match pass.user0_sampling {
                fret_core::scene::ImageSamplingHint::Default => "default",
                fret_core::scene::ImageSamplingHint::Linear => "linear",
                fret_core::scene::ImageSamplingHint::Nearest => "nearest",
            },
            user1_image: pass.user1_image.map(|id| format!("{id:?}")),
            user1_uv: [
                pass.user1_uv.u0,
                pass.user1_uv.v0,
                pass.user1_uv.u1,
                pass.user1_uv.v1,
            ],
            user1_sampling: match pass.user1_sampling {
                fret_core::scene::ImageSamplingHint::Default => "default",
                fret_core::scene::ImageSamplingHint::Linear => "linear",
                fret_core::scene::ImageSamplingHint::Nearest => "nearest",
            },
            load: encode_load_op(pass.common.load),
        },
        RenderPlanPass::ClipMask(pass) => JsonDumpPass::ClipMask {
            dst: plan_target_name(pass.dst),
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            uniform_index: pass.uniform_index,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ReleaseTarget(t) => JsonDumpPass::ReleaseTarget {
            target: plan_target_name(*t),
        },
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::renderer::render_plan::{CustomEffectPassCommon, CustomEffectV3Pass};
    use serde_json::Value;
    use slotmap::SlotMap;

    #[test]
    fn encode_custom_effect_v3_pass_keeps_distinct_source_targets() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect = effects.insert(());

        let pass = RenderPlanPass::CustomEffectV3(CustomEffectV3Pass {
            src_raw: PlanTarget::Intermediate1,
            src_pyramid: PlanTarget::Intermediate2,
            pyramid_levels: 3,
            pyramid_build_scissor: Some(LocalScissorRect(ScissorRect {
                x: 4,
                y: 5,
                w: 6,
                h: 7,
            })),
            raw_wanted: true,
            pyramid_wanted: true,
            common: CustomEffectPassCommon {
                src: PlanTarget::Intermediate0,
                dst: PlanTarget::Output,
                src_size: (100, 200),
                dst_size: (300, 400),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect,
                params: fret_core::EffectParamsV1::ZERO,
                load: wgpu::LoadOp::Load,
            },
            user0_image: None,
            user0_uv: fret_core::scene::UvRect::FULL,
            user0_sampling: fret_core::scene::ImageSamplingHint::Default,
            user1_image: None,
            user1_uv: fret_core::scene::UvRect::FULL,
            user1_sampling: fret_core::scene::ImageSamplingHint::Nearest,
        });

        let json = serde_json::to_value(encode_pass(&pass)).expect("serialize encoded pass");
        assert_eq!(
            json.get("kind"),
            Some(&Value::String("CustomEffectV3".to_string()))
        );
        assert_eq!(
            json.get("src"),
            Some(&Value::String("Intermediate0".to_string()))
        );
        assert_eq!(
            json.get("src_raw"),
            Some(&Value::String("Intermediate1".to_string()))
        );
        assert_eq!(
            json.get("src_pyramid"),
            Some(&Value::String("Intermediate2".to_string()))
        );
        assert_eq!(
            json.get("pyramid_levels"),
            Some(&Value::Number(serde_json::Number::from(3)))
        );
        assert_eq!(json.get("raw_wanted"), Some(&Value::Bool(true)));
        assert_eq!(json.get("pyramid_wanted"), Some(&Value::Bool(true)));
        assert!(json.get("pyramid_build_scissor").is_some());
    }
}
