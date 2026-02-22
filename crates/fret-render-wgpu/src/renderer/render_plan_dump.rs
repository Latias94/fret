use super::render_plan::{
    AbsoluteScissorRect, BlurAxis, DebugPostprocess, LocalScissorRect, MaskRef, PlanTarget,
    RenderPlan, RenderPlanDegradation, RenderPlanDegradationKind, RenderPlanDegradationReason,
    RenderPlanPass, ScaleMode,
};
use super::{EffectMarker, EffectMarkerKind, ScissorRect};

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

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
    target: String,
    size: [u32; 2],
    viewport_rect: JsonDumpScissorRect,
}

impl From<MaskRef> for JsonDumpMaskRef {
    fn from(m: MaskRef) -> Self {
        Self {
            target: plan_target_name(m.target).to_string(),
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
    OffscreenBlit,
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
        DebugPostprocess::OffscreenBlit => JsonDumpDebugPostprocess::OffscreenBlit,
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
        target: String,
        target_origin: [u32; 2],
        target_size: [u32; 2],
        load: JsonDumpLoadOp,
        draw_range: [usize; 2],
    },
    PathMsaaBatch {
        segment: usize,
        target: String,
        target_origin: [u32; 2],
        target_size: [u32; 2],
        draw_range: [usize; 2],
        union_scissor: JsonDumpScissorRect,
        batch_uniform_index: u32,
        load: JsonDumpLoadOp,
    },
    PathClipMask {
        dst: String,
        dst_origin: [u32; 2],
        dst_size: [u32; 2],
        scissor: JsonDumpScissorRect,
        uniform_index: u32,
        first_vertex: u32,
        vertex_count: u32,
        load: JsonDumpLoadOp,
    },
    FullscreenBlit {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        load: JsonDumpLoadOp,
    },
    CompositePremul {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        src_origin: [u32; 2],
        dst_origin: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        opacity: f32,
        load: JsonDumpLoadOp,
    },
    ScaleNearest {
        mode: String,
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        src_origin: [u32; 2],
        dst_origin: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        scale: u32,
        load: JsonDumpLoadOp,
    },
    Blur {
        axis: String,
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        load: JsonDumpLoadOp,
    },
    BackdropWarp {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        origin_px: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        strength_px: f32,
        scale_px: f32,
        phase: f32,
        chromatic_aberration_px: f32,
        warp_kind: String,
        load: JsonDumpLoadOp,
    },
    ColorAdjust {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        saturation: f32,
        brightness: f32,
        contrast: f32,
        load: JsonDumpLoadOp,
    },
    ColorMatrix {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        matrix: [f32; 20],
        load: JsonDumpLoadOp,
    },
    AlphaThreshold {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        cutoff: f32,
        soft: f32,
        load: JsonDumpLoadOp,
    },
    DropShadow {
        src: String,
        dst: String,
        src_size: [u32; 2],
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
        offset_px: [f32; 2],
        color: [f32; 4],
        load: JsonDumpLoadOp,
    },
    ClipMask {
        dst: String,
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        uniform_index: u32,
        load: JsonDumpLoadOp,
    },
    ReleaseTarget {
        target: String,
    },
}

fn plan_target_name(t: PlanTarget) -> &'static str {
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

fn encode_pass(p: &RenderPlanPass) -> JsonDumpPass {
    match p {
        RenderPlanPass::SceneDrawRange(pass) => JsonDumpPass::SceneDrawRange {
            segment: pass.segment.0,
            target: plan_target_name(pass.target).to_string(),
            target_origin: [pass.target_origin.0, pass.target_origin.1],
            target_size: [pass.target_size.0, pass.target_size.1],
            load: encode_load_op(pass.load),
            draw_range: [pass.draw_range.start, pass.draw_range.end],
        },
        RenderPlanPass::PathMsaaBatch(pass) => JsonDumpPass::PathMsaaBatch {
            segment: pass.segment.0,
            target: plan_target_name(pass.target).to_string(),
            target_origin: [pass.target_origin.0, pass.target_origin.1],
            target_size: [pass.target_size.0, pass.target_size.1],
            draw_range: [pass.draw_range.start, pass.draw_range.end],
            union_scissor: pass.union_scissor.into(),
            batch_uniform_index: pass.batch_uniform_index,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::PathClipMask(pass) => JsonDumpPass::PathClipMask {
            dst: plan_target_name(pass.dst).to_string(),
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            scissor: pass.scissor.into(),
            uniform_index: pass.uniform_index,
            first_vertex: pass.first_vertex,
            vertex_count: pass.vertex_count,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::FullscreenBlit(pass) => JsonDumpPass::FullscreenBlit {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::CompositePremul(pass) => JsonDumpPass::CompositePremul {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            src_origin: [pass.src_origin.0, pass.src_origin.1],
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            opacity: pass.opacity,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ScaleNearest(pass) => JsonDumpPass::ScaleNearest {
            mode: match pass.mode {
                ScaleMode::Downsample => "Downsample".to_string(),
                ScaleMode::Upscale => "Upscale".to_string(),
            },
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            src_origin: [pass.src_origin.0, pass.src_origin.1],
            dst_origin: [pass.dst_origin.0, pass.dst_origin.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            scale: pass.scale,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::Blur(pass) => JsonDumpPass::Blur {
            axis: match pass.axis {
                BlurAxis::Horizontal => "Horizontal".to_string(),
                BlurAxis::Vertical => "Vertical".to_string(),
            },
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::BackdropWarp(pass) => JsonDumpPass::BackdropWarp {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            origin_px: [pass.origin_px.0, pass.origin_px.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            strength_px: pass.strength_px,
            scale_px: pass.scale_px,
            phase: pass.phase,
            chromatic_aberration_px: pass.chromatic_aberration_px,
            warp_kind: match pass.kind {
                fret_core::scene::BackdropWarpKindV1::Wave => "Wave".to_string(),
                fret_core::scene::BackdropWarpKindV1::LensReserved => "LensReserved".to_string(),
            },
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ColorAdjust(pass) => JsonDumpPass::ColorAdjust {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            saturation: pass.saturation,
            brightness: pass.brightness,
            contrast: pass.contrast,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ColorMatrix(pass) => JsonDumpPass::ColorMatrix {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            matrix: pass.matrix,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::AlphaThreshold(pass) => JsonDumpPass::AlphaThreshold {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            cutoff: pass.cutoff,
            soft: pass.soft,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::DropShadow(pass) => JsonDumpPass::DropShadow {
            src: plan_target_name(pass.src).to_string(),
            dst: plan_target_name(pass.dst).to_string(),
            src_size: [pass.src_size.0, pass.src_size.1],
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
            offset_px: [pass.offset_px.0, pass.offset_px.1],
            color: [pass.color.r, pass.color.g, pass.color.b, pass.color.a],
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ClipMask(pass) => JsonDumpPass::ClipMask {
            dst: plan_target_name(pass.dst).to_string(),
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            uniform_index: pass.uniform_index,
            load: encode_load_op(pass.load),
        },
        RenderPlanPass::ReleaseTarget(t) => JsonDumpPass::ReleaseTarget {
            target: plan_target_name(*t).to_string(),
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
    drop_shadow: usize,
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
        drop_shadow: 0,
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
            RenderPlanPass::DropShadow(_) => c.drop_shadow += 1,
            RenderPlanPass::ClipMask(_) => c.clip_mask += 1,
            RenderPlanPass::ReleaseTarget(_) => c.release_target += 1,
        }
    }

    c
}

#[derive(Debug, serde::Serialize)]
struct RenderPlanJsonDump {
    schema_version: u32,
    frame_index: u64,
    viewport_size: [u32; 2],
    format: String,
    postprocess: JsonDumpDebugPostprocess,
    ordered_draws_len: usize,
    segments: Vec<JsonDumpSegment>,
    effect_markers: Vec<JsonDumpEffectMarker>,
    pass_counts: JsonDumpCounts,
    estimated_peak_intermediate_bytes: u64,
    degradations: Vec<JsonDumpDegradation>,
    passes: Vec<JsonDumpPass>,
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpDegradation {
    draw_ix: usize,
    kind: String,
    reason: String,
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
        kind: kind.to_string(),
        reason: reason.to_string(),
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
) {
    if !should_dump_frame(frame_index) {
        return;
    }

    let mut segment_pass_counts: Vec<JsonDumpSegmentPassCounts> = vec![
        JsonDumpSegmentPassCounts {
            scene_draw_range: 0,
            path_msaa_batch: 0,
        };
        plan.segments.len()
    ];
    for p in &plan.passes {
        match p {
            RenderPlanPass::SceneDrawRange(p) => {
                if let Some(c) = segment_pass_counts.get_mut(p.segment.0) {
                    c.scene_draw_range += 1;
                }
            }
            RenderPlanPass::PathMsaaBatch(p) => {
                if let Some(c) = segment_pass_counts.get_mut(p.segment.0) {
                    c.path_msaa_batch += 1;
                }
            }
            _ => {}
        }
    }

    let dir = dump_dir_from_env();
    let _ = std::fs::create_dir_all(&dir);

    let dump = RenderPlanJsonDump {
        schema_version: 5,
        frame_index,
        viewport_size: [viewport_size.0, viewport_size.1],
        format: format!("{format:?}"),
        postprocess: encode_debug_postprocess(postprocess),
        ordered_draws_len,
        segments: plan
            .segments
            .iter()
            .enumerate()
            .map(|(ix, s)| JsonDumpSegment {
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
                pass_counts: segment_pass_counts.get(ix).cloned().unwrap_or(
                    JsonDumpSegmentPassCounts {
                        scene_draw_range: 0,
                        path_msaa_batch: 0,
                    },
                ),
            })
            .collect(),
        effect_markers: effect_markers
            .iter()
            .copied()
            .map(encode_effect_marker)
            .collect(),
        pass_counts: pass_counts(plan),
        estimated_peak_intermediate_bytes: plan.compile_stats.estimated_peak_intermediate_bytes,
        degradations: plan
            .degradations
            .iter()
            .copied()
            .map(encode_degradation)
            .collect(),
        passes: plan.passes.iter().map(encode_pass).collect(),
    };

    let file = dir.join(format!("renderplan.frame{frame_index}.json"));
    let Ok(bytes) = serde_json::to_vec_pretty(&dump) else {
        return;
    };
    let _ = std::fs::write(&file, bytes);
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
) {
}
