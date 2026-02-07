use super::render_plan::{
    BlurAxis, DebugPostprocess, MaskRef, PlanTarget, RenderPlan, RenderPlanPass, ScaleMode,
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
        },
        EffectMarkerKind::Pop => JsonDumpEffectMarker::Pop { draw_ix: m.draw_ix },
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind")]
enum JsonDumpPass {
    SceneDrawRange {
        segment: usize,
        target: String,
        load: JsonDumpLoadOp,
        draw_range: [usize; 2],
    },
    PathMsaaBatch {
        segment: usize,
        target: String,
        draw_range: [usize; 2],
        union_scissor: JsonDumpScissorRect,
        batch_uniform_index: u32,
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
        dst_scissor: Option<JsonDumpScissorRect>,
        mask_uniform_index: Option<u32>,
        mask: Option<JsonDumpMaskRef>,
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
    ClipMask {
        dst: String,
        dst_size: [u32; 2],
        dst_scissor: Option<JsonDumpScissorRect>,
        uniform_index: u32,
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
            load: encode_load_op(pass.load),
            draw_range: [pass.draw_range.start, pass.draw_range.end],
        },
        RenderPlanPass::PathMsaaBatch(pass) => JsonDumpPass::PathMsaaBatch {
            segment: pass.segment.0,
            target: plan_target_name(pass.target).to_string(),
            draw_range: [pass.draw_range.start, pass.draw_range.end],
            union_scissor: pass.union_scissor.into(),
            batch_uniform_index: pass.batch_uniform_index,
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
            dst_scissor: pass.dst_scissor.map(Into::into),
            mask_uniform_index: pass.mask_uniform_index,
            mask: pass.mask.map(Into::into),
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
        RenderPlanPass::ClipMask(pass) => JsonDumpPass::ClipMask {
            dst: plan_target_name(pass.dst).to_string(),
            dst_size: [pass.dst_size.0, pass.dst_size.1],
            dst_scissor: pass.dst_scissor.map(Into::into),
            uniform_index: pass.uniform_index,
        },
        RenderPlanPass::ReleaseTarget(t) => JsonDumpPass::ReleaseTarget {
            target: plan_target_name(*t).to_string(),
        },
    }
}

#[derive(Debug, serde::Serialize)]
struct JsonDumpCounts {
    total: usize,
    scene: usize,
    path_msaa: usize,
    fullscreen_blit: usize,
    composite_premul: usize,
    scale_nearest: usize,
    blur: usize,
    color_adjust: usize,
    clip_mask: usize,
    release_target: usize,
}

fn pass_counts(plan: &RenderPlan) -> JsonDumpCounts {
    let mut c = JsonDumpCounts {
        total: plan.passes.len(),
        scene: 0,
        path_msaa: 0,
        fullscreen_blit: 0,
        composite_premul: 0,
        scale_nearest: 0,
        blur: 0,
        color_adjust: 0,
        clip_mask: 0,
        release_target: 0,
    };

    for p in &plan.passes {
        match p {
            RenderPlanPass::SceneDrawRange(_) => c.scene += 1,
            RenderPlanPass::PathMsaaBatch(_) => c.path_msaa += 1,
            RenderPlanPass::FullscreenBlit(_) => c.fullscreen_blit += 1,
            RenderPlanPass::CompositePremul(_) => c.composite_premul += 1,
            RenderPlanPass::ScaleNearest(_) => c.scale_nearest += 1,
            RenderPlanPass::Blur(_) => c.blur += 1,
            RenderPlanPass::ColorAdjust(_) => c.color_adjust += 1,
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
    effect_markers: Vec<JsonDumpEffectMarker>,
    pass_counts: JsonDumpCounts,
    passes: Vec<JsonDumpPass>,
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

    let dir = dump_dir_from_env();
    let _ = std::fs::create_dir_all(&dir);

    let dump = RenderPlanJsonDump {
        schema_version: 1,
        frame_index,
        viewport_size: [viewport_size.0, viewport_size.1],
        format: format!("{format:?}"),
        postprocess: encode_debug_postprocess(postprocess),
        ordered_draws_len,
        effect_markers: effect_markers
            .iter()
            .copied()
            .map(encode_effect_marker)
            .collect(),
        pass_counts: pass_counts(plan),
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
