use super::render_plan::{
    AbsoluteScissorRect, BlurAxis, DebugPostprocess, LocalScissorRect, MaskRef, PlanTarget,
    RenderPlan, RenderPlanDegradation, RenderPlanDegradationKind, RenderPlanDegradationReason,
    RenderPlanPass, ScaleMode,
};
use super::{EffectMarker, EffectMarkerKind, ScissorRect};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
struct JsonDumpScissorRect {
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
struct JsonDumpMaskRef {
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
enum JsonDumpLoadOp {
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
enum JsonDumpDebugPostprocess {
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

fn encode_debug_postprocess(p: DebugPostprocess) -> JsonDumpDebugPostprocess {
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
enum JsonDumpEffectMarker {
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

fn encode_effect_marker(m: EffectMarker) -> JsonDumpEffectMarker {
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
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
enum JsonDumpPass {
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

fn plan_target_name(t: PlanTarget) -> &'static str {
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

fn encode_pass(p: &RenderPlanPass) -> JsonDumpPass {
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
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            effect: format!("{:?}", pass.effect),
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::CustomEffectV2(pass) => JsonDumpPass::CustomEffectV2 {
            src: plan_target_name(pass.src),
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            effect: format!("{:?}", pass.effect),
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
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::CustomEffectV3(pass) => JsonDumpPass::CustomEffectV3 {
            src: plan_target_name(pass.src),
            src_raw: plan_target_name(pass.src_raw),
            src_pyramid: plan_target_name(pass.src_pyramid),
            pyramid_levels: pass.pyramid_levels,
            raw_wanted: pass.raw_wanted,
            pyramid_wanted: pass.pyramid_wanted,
            dst: plan_target_name(pass.dst),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            dst_scissor_space: pass.dst_scissor.map(|_| "dst_local"),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            effect: format!("{:?}", pass.effect),
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
            load: encode_load_op(pass.load),
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

#[derive(Debug, serde::Serialize)]
struct JsonDumpSegmentFlags {
    has_quad: bool,
    has_viewport: bool,
    has_image: bool,
    has_mask: bool,
    has_text: bool,
    has_path: bool,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpSegment {
    id: usize,
    draw_range: [usize; 2],
    start_uniform_index: Option<u32>,
    start_uniform_fingerprint: String,
    flags: JsonDumpSegmentFlags,
    pass_counts: JsonDumpSegmentPassCounts,
}

#[derive(Debug, serde::Serialize, Clone, Copy)]
struct JsonDumpSegmentPassCounts {
    scene_draw_range: usize,
    path_msaa_batch: usize,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpCounts {
    total: usize,
    scene: usize,
    path_msaa: usize,
    path_clip_mask: usize,
    fullscreen_blit: usize,
    composite_premul: usize,
    scale_nearest: usize,
    blur: usize,
    backdrop_warp: usize,
    color_adjust: usize,
    color_matrix: usize,
    alpha_threshold: usize,
    dither: usize,
    noise: usize,
    drop_shadow: usize,
    custom_effect_v1: usize,
    custom_effect_v2: usize,
    custom_effect_v3: usize,
    clip_mask: usize,
    release_target: usize,
}

fn pass_counts(plan: &RenderPlan) -> JsonDumpCounts {
    let mut c = JsonDumpCounts {
        total: plan.passes.len(),
        scene: 0,
        path_msaa: 0,
        path_clip_mask: 0,
        fullscreen_blit: 0,
        composite_premul: 0,
        scale_nearest: 0,
        blur: 0,
        backdrop_warp: 0,
        color_adjust: 0,
        color_matrix: 0,
        alpha_threshold: 0,
        dither: 0,
        noise: 0,
        drop_shadow: 0,
        custom_effect_v1: 0,
        custom_effect_v2: 0,
        custom_effect_v3: 0,
        clip_mask: 0,
        release_target: 0,
    };

    for p in &plan.passes {
        match p {
            RenderPlanPass::SceneDrawRange(_) => c.scene += 1,
            RenderPlanPass::PathMsaaBatch(_) => c.path_msaa += 1,
            RenderPlanPass::PathClipMask(_) => c.path_clip_mask += 1,
            RenderPlanPass::FullscreenBlit(_) => c.fullscreen_blit += 1,
            RenderPlanPass::CompositePremul(_) => c.composite_premul += 1,
            RenderPlanPass::ScaleNearest(_) => c.scale_nearest += 1,
            RenderPlanPass::Blur(_) => c.blur += 1,
            RenderPlanPass::BackdropWarp(_) => c.backdrop_warp += 1,
            RenderPlanPass::ColorAdjust(_) => c.color_adjust += 1,
            RenderPlanPass::ColorMatrix(_) => c.color_matrix += 1,
            RenderPlanPass::AlphaThreshold(_) => c.alpha_threshold += 1,
            RenderPlanPass::Dither(_) => c.dither += 1,
            RenderPlanPass::Noise(_) => c.noise += 1,
            RenderPlanPass::DropShadow(_) => c.drop_shadow += 1,
            RenderPlanPass::CustomEffect(_) => c.custom_effect_v1 += 1,
            RenderPlanPass::CustomEffectV2(_) => c.custom_effect_v2 += 1,
            RenderPlanPass::CustomEffectV3(_) => c.custom_effect_v3 += 1,
            RenderPlanPass::ClipMask(_) => c.clip_mask += 1,
            RenderPlanPass::ReleaseTarget(_) => c.release_target += 1,
        }
    }

    c
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpCustomEffectSummary {
    effect: String,
    abi: &'static str,
    pass_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user0_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user0_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user1_image_some: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user1_image_none: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_requested: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_distinct: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_aliased: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_requested: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pyramid_degraded_to_one: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpTargetUsage {
    target: &'static str,
    max_size: [u32; 2],
    src_uses: usize,
    dst_uses: usize,
    mask_uses: usize,
}

#[cfg(not(target_arch = "wasm32"))]
fn summarize_custom_effects(passes: &[RenderPlanPass]) -> Vec<JsonDumpCustomEffectSummary> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Abi {
        V1,
        V2,
        V3,
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct Acc {
        pass_count: usize,
        input_image_some: usize,
        input_image_none: usize,
        user0_image_some: usize,
        user0_image_none: usize,
        user1_image_some: usize,
        user1_image_none: usize,
        raw_requested: usize,
        raw_distinct: usize,
        raw_aliased: usize,
        pyramid_requested: usize,
        pyramid_degraded_to_one: usize,
    }

    let mut by_effect: HashMap<(fret_core::EffectId, Abi), Acc> = HashMap::new();
    for p in passes {
        match p {
            RenderPlanPass::CustomEffect(p) => {
                let acc = by_effect.entry((p.effect, Abi::V1)).or_default();
                acc.pass_count += 1;
            }
            RenderPlanPass::CustomEffectV2(p) => {
                let acc = by_effect.entry((p.effect, Abi::V2)).or_default();
                acc.pass_count += 1;
                if p.input_image.is_some() {
                    acc.input_image_some += 1;
                } else {
                    acc.input_image_none += 1;
                }
            }
            RenderPlanPass::CustomEffectV3(p) => {
                let acc = by_effect.entry((p.effect, Abi::V3)).or_default();
                acc.pass_count += 1;
                if p.user0_image.is_some() {
                    acc.user0_image_some += 1;
                } else {
                    acc.user0_image_none += 1;
                }
                if p.user1_image.is_some() {
                    acc.user1_image_some += 1;
                } else {
                    acc.user1_image_none += 1;
                }
                if p.raw_wanted {
                    acc.raw_requested += 1;
                    if p.src_raw == p.src {
                        acc.raw_aliased += 1;
                    } else {
                        acc.raw_distinct += 1;
                    }
                }
                if p.pyramid_wanted {
                    acc.pyramid_requested += 1;
                    if p.pyramid_levels <= 1 {
                        acc.pyramid_degraded_to_one += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let mut out: Vec<_> = by_effect
        .into_iter()
        .map(|((effect, abi), acc)| JsonDumpCustomEffectSummary {
            effect: format!("{effect:?}"),
            abi: match abi {
                Abi::V1 => "custom_v1.params_only",
                Abi::V2 => "custom_v2.user_image",
                Abi::V3 => "custom_v3.renderer_sources",
            },
            pass_count: acc.pass_count,
            input_image_some: (abi == Abi::V2).then_some(acc.input_image_some),
            input_image_none: (abi == Abi::V2).then_some(acc.input_image_none),
            user0_image_some: (abi == Abi::V3).then_some(acc.user0_image_some),
            user0_image_none: (abi == Abi::V3).then_some(acc.user0_image_none),
            user1_image_some: (abi == Abi::V3).then_some(acc.user1_image_some),
            user1_image_none: (abi == Abi::V3).then_some(acc.user1_image_none),
            raw_requested: (abi == Abi::V3).then_some(acc.raw_requested),
            raw_distinct: (abi == Abi::V3).then_some(acc.raw_distinct),
            raw_aliased: (abi == Abi::V3).then_some(acc.raw_aliased),
            pyramid_requested: (abi == Abi::V3).then_some(acc.pyramid_requested),
            pyramid_degraded_to_one: (abi == Abi::V3).then_some(acc.pyramid_degraded_to_one),
        })
        .collect();

    out.sort_by(|a, b| (a.abi, &a.effect).cmp(&(b.abi, &b.effect)));
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn plan_target_index(t: PlanTarget) -> usize {
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

#[cfg(not(target_arch = "wasm32"))]
fn bump_usage(
    usage: &mut [Option<JsonDumpTargetUsage>; 8],
    target: PlanTarget,
    kind: &str,
    size: (u32, u32),
) {
    let slot = &mut usage[plan_target_index(target)];
    let entry = slot.get_or_insert_with(|| JsonDumpTargetUsage {
        target: plan_target_name(target),
        max_size: [0, 0],
        src_uses: 0,
        dst_uses: 0,
        mask_uses: 0,
    });

    entry.max_size[0] = entry.max_size[0].max(size.0.max(1));
    entry.max_size[1] = entry.max_size[1].max(size.1.max(1));
    match kind {
        "src" => entry.src_uses += 1,
        "dst" => entry.dst_uses += 1,
        "mask" => entry.mask_uses += 1,
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn summarize_target_usage(passes: &[RenderPlanPass]) -> Vec<JsonDumpTargetUsage> {
    let mut usage: [Option<JsonDumpTargetUsage>; 8] = std::array::from_fn(|_| None);

    for p in passes {
        match p {
            RenderPlanPass::SceneDrawRange(pass) => {
                bump_usage(&mut usage, pass.target, "dst", pass.target_size);
            }
            RenderPlanPass::PathMsaaBatch(pass) => {
                bump_usage(&mut usage, pass.target, "dst", pass.target_size);
            }
            RenderPlanPass::PathClipMask(pass) => {
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::FullscreenBlit(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::CompositePremul(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ScaleNearest(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Blur(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::BackdropWarp(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ColorAdjust(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ColorMatrix(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::AlphaThreshold(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Dither(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::Noise(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::DropShadow(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffect(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffectV2(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::CustomEffectV3(pass) => {
                bump_usage(&mut usage, pass.src, "src", pass.src_size);
                bump_usage(&mut usage, pass.src_raw, "src", pass.src_size);
                bump_usage(&mut usage, pass.src_pyramid, "src", pass.src_size);
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
                if let Some(mask) = pass.mask {
                    bump_usage(&mut usage, mask.target, "mask", mask.size);
                }
            }
            RenderPlanPass::ClipMask(pass) => {
                bump_usage(&mut usage, pass.dst, "dst", pass.dst_size);
            }
            RenderPlanPass::ReleaseTarget(_) => {}
        }
    }

    let mut out: Vec<_> = usage.into_iter().flatten().collect();
    out.sort_by(|a, b| a.target.cmp(b.target));
    out
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::renderer::render_plan::{CustomEffectPass, CustomEffectV2Pass, RenderPlanPass};
    use slotmap::SlotMap;

    #[test]
    fn custom_effect_summaries_include_abi_and_input_counts() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect_v1 = effects.insert(());
        let effect_v2 = effects.insert(());

        let mut images: SlotMap<fret_core::ImageId, ()> = SlotMap::with_key();
        let image = images.insert(());

        let passes = vec![
            RenderPlanPass::CustomEffect(CustomEffectPass {
                src: PlanTarget::Output,
                dst: PlanTarget::Intermediate0,
                src_size: (10, 11),
                dst_size: (12, 13),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect: effect_v1,
                params: fret_core::EffectParamsV1::ZERO,
                load: wgpu::LoadOp::Load,
            }),
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                src: PlanTarget::Intermediate0,
                dst: PlanTarget::Intermediate1,
                src_size: (12, 13),
                dst_size: (20, 21),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect: effect_v2,
                params: fret_core::EffectParamsV1::ZERO,
                input_image: Some(image),
                input_uv: fret_core::scene::UvRect::FULL,
                input_sampling: fret_core::scene::ImageSamplingHint::Linear,
                load: wgpu::LoadOp::Load,
            }),
            RenderPlanPass::CustomEffectV2(CustomEffectV2Pass {
                src: PlanTarget::Intermediate1,
                dst: PlanTarget::Output,
                src_size: (20, 21),
                dst_size: (30, 31),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect: effect_v2,
                params: fret_core::EffectParamsV1::ZERO,
                input_image: None,
                input_uv: fret_core::scene::UvRect::FULL,
                input_sampling: fret_core::scene::ImageSamplingHint::Nearest,
                load: wgpu::LoadOp::Load,
            }),
        ];

        let summary = summarize_custom_effects(&passes);
        assert_eq!(summary.len(), 2);

        let v1 = summary
            .iter()
            .find(|s| s.abi == "custom_v1.params_only")
            .expect("v1 summary");
        assert_eq!(v1.pass_count, 1);
        assert_eq!(v1.input_image_some, None);
        assert_eq!(v1.input_image_none, None);

        let v2 = summary
            .iter()
            .find(|s| s.abi == "custom_v2.user_image")
            .expect("v2 summary");
        assert_eq!(v2.pass_count, 2);
        assert_eq!(v2.input_image_some, Some(1));
        assert_eq!(v2.input_image_none, Some(1));
    }

    #[test]
    fn target_usage_tracks_max_size() {
        let mut effects: SlotMap<fret_core::EffectId, ()> = SlotMap::with_key();
        let effect = effects.insert(());

        let passes = vec![
            RenderPlanPass::CustomEffect(CustomEffectPass {
                src: PlanTarget::Output,
                dst: PlanTarget::Intermediate0,
                src_size: (100, 100),
                dst_size: (10, 11),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect,
                params: fret_core::EffectParamsV1::ZERO,
                load: wgpu::LoadOp::Load,
            }),
            RenderPlanPass::CustomEffect(CustomEffectPass {
                src: PlanTarget::Intermediate0,
                dst: PlanTarget::Intermediate0,
                src_size: (20, 21),
                dst_size: (22, 23),
                dst_scissor: None,
                mask_uniform_index: None,
                mask: None,
                effect,
                params: fret_core::EffectParamsV1::ZERO,
                load: wgpu::LoadOp::Load,
            }),
        ];

        let usage = summarize_target_usage(&passes);
        let i0 = usage
            .iter()
            .find(|u| u.target == "Intermediate0")
            .expect("Intermediate0 usage");
        assert_eq!(i0.max_size, [22, 23]);
        assert!(i0.src_uses >= 1);
        assert!(i0.dst_uses >= 1);
    }
}

#[derive(Debug, serde::Serialize)]
struct RenderPlanJsonDump<'a> {
    schema_version: u32,
    frame_index: u64,
    viewport_size: [u32; 2],
    format: String,
    postprocess: JsonDumpDebugPostprocess,
    ordered_draws_len: usize,
    segments: &'a [JsonDumpSegment],
    effect_markers: &'a [JsonDumpEffectMarker],
    pass_counts: JsonDumpCounts,
    custom_effects: &'a [JsonDumpCustomEffectSummary],
    target_usage: &'a [JsonDumpTargetUsage],
    estimated_peak_intermediate_bytes: u64,
    degradations: &'a [JsonDumpDegradation],
    passes: &'a [JsonDumpPass],
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpDegradation {
    draw_ix: usize,
    kind: &'static str,
    reason: &'static str,
}

#[derive(Debug, Default)]
pub(super) struct RenderPlanJsonDumpScratch {
    segment_pass_counts: Vec<JsonDumpSegmentPassCounts>,
    segments: Vec<JsonDumpSegment>,
    effect_markers: Vec<JsonDumpEffectMarker>,
    custom_effects: Vec<JsonDumpCustomEffectSummary>,
    target_usage: Vec<JsonDumpTargetUsage>,
    degradations: Vec<JsonDumpDegradation>,
    passes: Vec<JsonDumpPass>,
    bytes: Vec<u8>,
}

fn encode_degradation(d: RenderPlanDegradation) -> JsonDumpDegradation {
    let kind = match d.kind {
        RenderPlanDegradationKind::BackdropEffectNoOp => "BackdropEffectNoOp",
        RenderPlanDegradationKind::FilterContentDisabled => "FilterContentDisabled",
        RenderPlanDegradationKind::ClipPathDisabled => "ClipPathDisabled",
        RenderPlanDegradationKind::CompositeGroupBlendDegradedToOver => {
            "CompositeGroupBlendDegradedToOver"
        }
    };
    let reason = match d.reason {
        RenderPlanDegradationReason::BudgetZero => "BudgetZero",
        RenderPlanDegradationReason::BudgetInsufficient => "BudgetInsufficient",
        RenderPlanDegradationReason::TargetExhausted => "TargetExhausted",
    };
    JsonDumpDegradation {
        draw_ix: d.draw_ix,
        kind,
        reason,
    }
}

fn parse_env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn dump_dir_from_env() -> PathBuf {
    std::env::var_os("FRET_RENDERPLAN_DUMP_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".fret").join("renderplan"))
}

#[cfg(not(target_arch = "wasm32"))]
fn should_dump_frame(frame_index: u64) -> bool {
    if std::env::var_os("FRET_RENDERPLAN_DUMP")
        .filter(|v| !v.is_empty())
        .is_none()
    {
        return false;
    }

    if let Some(frame) = parse_env_u64("FRET_RENDERPLAN_DUMP_FRAME") {
        return frame_index == frame;
    }

    let after = parse_env_u64("FRET_RENDERPLAN_DUMP_AFTER_FRAMES").unwrap_or(1);
    if frame_index < after {
        return false;
    }

    if let Some(every) = parse_env_u64("FRET_RENDERPLAN_DUMP_EVERY") {
        return every > 0 && (frame_index - after).is_multiple_of(every);
    }

    // Default: dump exactly once (first eligible frame).
    static DUMPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    !DUMPED.swap(true, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn maybe_dump_render_plan_json(
    plan: &RenderPlan,
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    frame_index: u64,
    postprocess: DebugPostprocess,
    ordered_draws_len: usize,
    effect_markers: &[EffectMarker],
    dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
    if !should_dump_frame(frame_index) {
        return;
    }

    dump_scratch.segment_pass_counts.resize(
        plan.segments.len(),
        JsonDumpSegmentPassCounts {
            scene_draw_range: 0,
            path_msaa_batch: 0,
        },
    );
    dump_scratch
        .segment_pass_counts
        .fill(JsonDumpSegmentPassCounts {
            scene_draw_range: 0,
            path_msaa_batch: 0,
        });
    for p in &plan.passes {
        match p {
            RenderPlanPass::SceneDrawRange(p) => {
                if let Some(c) = dump_scratch.segment_pass_counts.get_mut(p.segment.0) {
                    c.scene_draw_range += 1;
                }
            }
            RenderPlanPass::PathMsaaBatch(p) => {
                if let Some(c) = dump_scratch.segment_pass_counts.get_mut(p.segment.0) {
                    c.path_msaa_batch += 1;
                }
            }
            _ => {}
        }
    }

    dump_scratch.segments.clear();
    dump_scratch.segments.reserve(plan.segments.len());
    for (ix, s) in plan.segments.iter().enumerate() {
        dump_scratch.segments.push(JsonDumpSegment {
            id: s.id.0,
            draw_range: [s.draw_range.start, s.draw_range.end],
            start_uniform_index: s.start_uniform_index,
            start_uniform_fingerprint: format!("0x{:016x}", s.start_uniform_fingerprint),
            flags: JsonDumpSegmentFlags {
                has_quad: s.flags.has_quad,
                has_viewport: s.flags.has_viewport,
                has_image: s.flags.has_image,
                has_mask: s.flags.has_mask,
                has_text: s.flags.has_text,
                has_path: s.flags.has_path,
            },
            pass_counts: dump_scratch.segment_pass_counts.get(ix).copied().unwrap_or(
                JsonDumpSegmentPassCounts {
                    scene_draw_range: 0,
                    path_msaa_batch: 0,
                },
            ),
        });
    }

    dump_scratch.effect_markers.clear();
    dump_scratch.effect_markers.reserve(effect_markers.len());
    for m in effect_markers.iter().copied() {
        dump_scratch.effect_markers.push(encode_effect_marker(m));
    }

    dump_scratch.degradations.clear();
    dump_scratch.degradations.reserve(plan.degradations.len());
    for d in plan.degradations.iter().copied() {
        dump_scratch.degradations.push(encode_degradation(d));
    }

    dump_scratch.passes.clear();
    dump_scratch.passes.reserve(plan.passes.len());
    for p in &plan.passes {
        dump_scratch.passes.push(encode_pass(p));
    }

    dump_scratch.custom_effects = summarize_custom_effects(&plan.passes);
    dump_scratch.target_usage = summarize_target_usage(&plan.passes);

    let dir = dump_dir_from_env();
    let _ = std::fs::create_dir_all(&dir);

    let dump = RenderPlanJsonDump {
        schema_version: 6,
        frame_index,
        viewport_size: [viewport_size.0, viewport_size.1],
        format: format!("{format:?}"),
        postprocess: encode_debug_postprocess(postprocess),
        ordered_draws_len,
        segments: &dump_scratch.segments,
        effect_markers: &dump_scratch.effect_markers,
        pass_counts: pass_counts(plan),
        custom_effects: &dump_scratch.custom_effects,
        target_usage: &dump_scratch.target_usage,
        estimated_peak_intermediate_bytes: plan.compile_stats.estimated_peak_intermediate_bytes,
        degradations: &dump_scratch.degradations,
        passes: &dump_scratch.passes,
    };

    let file = dir.join(format!("renderplan.frame{frame_index}.json"));
    dump_scratch.bytes.clear();
    if serde_json::to_writer_pretty(&mut dump_scratch.bytes, &dump).is_err() {
        return;
    }
    let _ = std::fs::write(&file, &dump_scratch.bytes);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn maybe_dump_render_plan_json(
    _plan: &RenderPlan,
    _viewport_size: (u32, u32),
    _format: wgpu::TextureFormat,
    _frame_index: u64,
    _postprocess: DebugPostprocess,
    _ordered_draws_len: usize,
    _effect_markers: &[EffectMarker],
    _dump_scratch: &mut RenderPlanJsonDumpScratch,
) {
}
