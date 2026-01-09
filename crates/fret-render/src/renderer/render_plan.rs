use super::frame_targets::downsampled_size;
use super::intermediate_pool::estimate_texture_bytes;
use super::util::union_scissor;
use super::{EffectMarkerKind, OrderedDraw, SceneEncoding, ScissorRect};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SceneSegmentId(pub(super) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanTarget {
    Output,
    Intermediate0,
    Intermediate1,
    Intermediate2,
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
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
    pub(super) draw_range: Range<usize>,
}

#[derive(Debug)]
pub(super) enum RenderPlanPass {
    SceneDrawRange(SceneDrawRangePass),
    PathMsaaBatch(PathMsaaBatchPass),
    FullscreenBlit(FullscreenBlitPass),
    CompositePremul(CompositePremulPass),
    ScaleNearest(ScaleNearestPass),
    Blur(BlurPass),
    ColorAdjust(ColorAdjustPass),
    ReleaseTarget(PlanTarget),
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
    pub(super) axis: BlurAxis,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ColorAdjustPass {
    pub(super) src: PlanTarget,
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) saturation: f32,
    pub(super) brightness: f32,
    pub(super) contrast: f32,
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
    pub(super) dst: PlanTarget,
    pub(super) src_size: (u32, u32),
    pub(super) dst_size: (u32, u32),
    pub(super) dst_scissor: Option<ScissorRect>,
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
    pub(super) dst_scissor: Option<ScissorRect>,
    pub(super) mode: ScaleMode,
    pub(super) scale: u32,
    pub(super) load: wgpu::LoadOp<wgpu::Color>,
}

#[derive(Debug, Clone)]
pub(super) struct PathMsaaBatchPass {
    pub(super) segment: SceneSegmentId,
    pub(super) target: PlanTarget,
    pub(super) draw_range: Range<usize>,
    pub(super) union_scissor: ScissorRect,
    pub(super) batch_uniform_index: u32,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub(super) passes: Vec<RenderPlanPass>,
}

impl RenderPlan {
    pub(super) fn compile_for_scene(
        encoding: &SceneEncoding,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        clear: wgpu::Color,
        path_samples: u32,
        postprocess: DebugPostprocess,
        intermediate_budget_bytes: u64,
    ) -> Self {
        let mut postprocess = postprocess;

        let has_backdrop_supported_effect = encoding.effect_markers.iter().any(|m| {
            let EffectMarkerKind::Push { mode, chain, .. } = m.kind else {
                return false;
            };
            mode == fret_core::EffectMode::Backdrop
                && (effect_chain_has_gaussian_blur(chain) || effect_chain_has_color_adjust(chain))
        });
        let backdrop_effect_enabled = if !has_backdrop_supported_effect {
            false
        } else {
            let blur_ok = choose_effect_blur_downsample_scale(
                viewport_size,
                format,
                intermediate_budget_bytes,
                2,
                fret_core::EffectQuality::Auto,
            )
            .is_some();
            let color_ok = color_adjust_enabled(viewport_size, format, intermediate_budget_bytes);
            blur_ok || color_ok
        };

        let needs_intermediate = backdrop_effect_enabled
            || matches!(
                postprocess,
                DebugPostprocess::OffscreenBlit
                    | DebugPostprocess::Pixelate { .. }
                    | DebugPostprocess::Blur { .. }
            );

        if needs_intermediate && matches!(postprocess, DebugPostprocess::None) {
            postprocess = DebugPostprocess::OffscreenBlit;
        }

        let scene_target = if needs_intermediate {
            PlanTarget::Intermediate0
        } else {
            PlanTarget::Output
        };
        let draws = &encoding.ordered_draws;

        if encoding.effect_markers.is_empty() {
            if path_samples <= 1 {
                let mut plan = Self {
                    passes: vec![RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                        segment: SceneSegmentId(0),
                        target: scene_target,
                        load: wgpu::LoadOp::Clear(clear),
                        draw_range: 0..draws.len(),
                    })],
                };
                append_postprocess(&mut plan, viewport_size, postprocess, clear);
                insert_early_releases(&mut plan.passes);
                return plan;
            }

            let mut passes: Vec<RenderPlanPass> = Vec::new();
            let mut is_first_pass = true;
            let mut scene_range_start: usize = 0;
            let mut cursor: usize = 0;

            while cursor < draws.len() {
                if let OrderedDraw::Path(first) = &draws[cursor] {
                    if is_first_pass {
                        passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                            segment: SceneSegmentId(0),
                            target: scene_target,
                            load: wgpu::LoadOp::Clear(clear),
                            draw_range: scene_range_start..cursor,
                        }));
                        is_first_pass = false;
                    } else if scene_range_start < cursor {
                        passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                            segment: SceneSegmentId(0),
                            target: scene_target,
                            load: wgpu::LoadOp::Load,
                            draw_range: scene_range_start..cursor,
                        }));
                    }

                    let batch_uniform_index = first.uniform_index;
                    let mut union = first.scissor;
                    let mut end = cursor + 1;
                    while end < draws.len() {
                        match &draws[end] {
                            OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                                union = union_scissor(union, d.scissor);
                                end += 1;
                            }
                            _ => break,
                        }
                    }

                    passes.push(RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                        segment: SceneSegmentId(0),
                        target: scene_target,
                        draw_range: cursor..end,
                        union_scissor: union,
                        batch_uniform_index,
                    }));

                    cursor = end;
                    scene_range_start = cursor;
                    continue;
                }

                cursor += 1;
            }

            if is_first_pass {
                passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: scene_target,
                    load: wgpu::LoadOp::Clear(clear),
                    draw_range: 0..draws.len(),
                }));
            } else if scene_range_start < draws.len() {
                passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: scene_target,
                    load: wgpu::LoadOp::Load,
                    draw_range: scene_range_start..draws.len(),
                }));
            }

            let mut plan = Self { passes };
            append_postprocess(&mut plan, viewport_size, postprocess, clear);
            insert_early_releases(&mut plan.passes);
            return plan;
        }

        #[derive(Clone, Copy, Debug)]
        struct DrawScope {
            target: PlanTarget,
            needs_clear: bool,
            clear_color: wgpu::Color,
        }

        #[derive(Clone, Copy, Debug)]
        struct EffectScope {
            mode: fret_core::EffectMode,
            chain: fret_core::EffectChain,
            quality: fret_core::EffectQuality,
            scissor: ScissorRect,
            parent_target: PlanTarget,
            content_target: Option<PlanTarget>,
        }

        let mut passes: Vec<RenderPlanPass> = Vec::new();
        let mut draw_scopes: Vec<DrawScope> = vec![DrawScope {
            target: scene_target,
            needs_clear: true,
            clear_color: clear,
        }];
        let mut effect_scopes: Vec<EffectScope> = Vec::new();

        let mut scene_range_start: usize = 0;
        let mut cursor: usize = 0;
        let mut marker_ix: usize = 0;
        let markers = &encoding.effect_markers;

        let flush_scene_range = |end: usize,
                                 passes: &mut Vec<RenderPlanPass>,
                                 draw_scopes: &mut Vec<DrawScope>,
                                 scene_range_start: &mut usize| {
            let scope = draw_scopes.last_mut().expect("draw scope");
            if scope.needs_clear {
                passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: scope.target,
                    load: wgpu::LoadOp::Clear(scope.clear_color),
                    draw_range: *scene_range_start..end,
                }));
                scope.needs_clear = false;
            } else if *scene_range_start < end {
                passes.push(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                    segment: SceneSegmentId(0),
                    target: scope.target,
                    load: wgpu::LoadOp::Load,
                    draw_range: *scene_range_start..end,
                }));
            }
            *scene_range_start = end;
        };

        let available_scratch_targets =
            |draw_scopes: &[DrawScope], srcdst: PlanTarget| -> Vec<PlanTarget> {
                let mut out: Vec<PlanTarget> = Vec::new();
                for t in [
                    PlanTarget::Intermediate0,
                    PlanTarget::Intermediate1,
                    PlanTarget::Intermediate2,
                ] {
                    if t == srcdst {
                        continue;
                    }
                    if draw_scopes.iter().any(|s| s.target == t) {
                        continue;
                    }
                    out.push(t);
                }
                out
            };

        let apply_chain_in_place = |passes: &mut Vec<RenderPlanPass>,
                                    draw_scopes: &[DrawScope],
                                    srcdst: PlanTarget,
                                    chain: fret_core::EffectChain,
                                    quality: fret_core::EffectQuality,
                                    scissor: ScissorRect| {
            if srcdst == PlanTarget::Output || scissor.w == 0 || scissor.h == 0 {
                return;
            }

            for step in chain.iter() {
                match step {
                    fret_core::EffectStep::GaussianBlur {
                        radius_px: _,
                        downsample,
                    } => {
                        let downsample = if downsample >= 4 { 4 } else { 2 };
                        let scratch = available_scratch_targets(draw_scopes, srcdst);
                        if scratch.len() >= 2 {
                            let Some(downsample_scale) = choose_effect_blur_downsample_scale(
                                viewport_size,
                                format,
                                intermediate_budget_bytes,
                                downsample,
                                quality,
                            ) else {
                                continue;
                            };
                            append_scissored_blur_in_place_two_scratch(
                                passes,
                                srcdst,
                                scratch[0],
                                scratch[1],
                                viewport_size,
                                downsample_scale,
                                scissor,
                                clear,
                            );
                            continue;
                        }

                        let Some(&scratch) = scratch.first() else {
                            continue;
                        };
                        if intermediate_budget_bytes == 0 {
                            continue;
                        }
                        let full = estimate_texture_bytes(viewport_size, format, 1);
                        let required = full.saturating_mul(2);
                        if required > intermediate_budget_bytes {
                            continue;
                        }
                        append_scissored_blur_in_place_single_scratch(
                            passes,
                            srcdst,
                            scratch,
                            viewport_size,
                            scissor,
                            clear,
                        );
                    }
                    fret_core::EffectStep::ColorAdjust {
                        saturation,
                        brightness,
                        contrast,
                    } => {
                        if !color_adjust_enabled(viewport_size, format, intermediate_budget_bytes) {
                            continue;
                        }
                        let scratch = available_scratch_targets(draw_scopes, srcdst);
                        let Some(&scratch) = scratch.first() else {
                            continue;
                        };
                        append_color_adjust_in_place_single_scratch(
                            passes,
                            srcdst,
                            scratch,
                            viewport_size,
                            Some(scissor),
                            saturation,
                            brightness,
                            contrast,
                            clear,
                        );
                    }
                    fret_core::EffectStep::Pixelate { .. }
                    | fret_core::EffectStep::Dither { .. } => {
                        // Not yet implemented in effect chains (debug-only postprocesses exist).
                    }
                }
            }
        };

        while cursor <= draws.len() {
            let next_marker_at = markers
                .get(marker_ix)
                .map(|m| m.draw_ix)
                .unwrap_or(usize::MAX);

            if cursor == next_marker_at || cursor == draws.len() {
                flush_scene_range(
                    cursor,
                    &mut passes,
                    &mut draw_scopes,
                    &mut scene_range_start,
                );

                while marker_ix < markers.len() && markers[marker_ix].draw_ix == cursor {
                    let marker = markers[marker_ix];
                    match marker.kind {
                        EffectMarkerKind::Push {
                            scissor,
                            mode,
                            chain,
                            quality,
                        } => {
                            let parent_target = draw_scopes.last().expect("draw scope").target;
                            match mode {
                                fret_core::EffectMode::Backdrop => {
                                    apply_chain_in_place(
                                        &mut passes,
                                        &draw_scopes,
                                        parent_target,
                                        chain,
                                        quality,
                                        scissor,
                                    );
                                    effect_scopes.push(EffectScope {
                                        mode,
                                        chain,
                                        quality,
                                        scissor,
                                        parent_target,
                                        content_target: None,
                                    });
                                }
                                fret_core::EffectMode::FilterContent => {
                                    let mut content_target: Option<PlanTarget> = None;
                                    for t in [
                                        PlanTarget::Intermediate0,
                                        PlanTarget::Intermediate1,
                                        PlanTarget::Intermediate2,
                                    ] {
                                        if draw_scopes.iter().any(|s| s.target == t) {
                                            continue;
                                        }
                                        content_target = Some(t);
                                        break;
                                    }

                                    if let Some(content_target) = content_target {
                                        draw_scopes.push(DrawScope {
                                            target: content_target,
                                            needs_clear: true,
                                            clear_color: wgpu::Color::TRANSPARENT,
                                        });
                                    }

                                    effect_scopes.push(EffectScope {
                                        mode,
                                        chain,
                                        quality,
                                        scissor,
                                        parent_target,
                                        content_target,
                                    });
                                }
                            }
                        }
                        EffectMarkerKind::Pop => {
                            let Some(scope) = effect_scopes.pop() else {
                                marker_ix += 1;
                                continue;
                            };

                            if scope.mode == fret_core::EffectMode::FilterContent
                                && let Some(content_target) = scope.content_target
                            {
                                debug_assert_eq!(
                                    draw_scopes.last().expect("draw scope").target,
                                    content_target
                                );

                                apply_chain_in_place(
                                    &mut passes,
                                    &draw_scopes,
                                    content_target,
                                    scope.chain,
                                    scope.quality,
                                    scope.scissor,
                                );

                                passes.push(RenderPlanPass::CompositePremul(CompositePremulPass {
                                    src: content_target,
                                    dst: scope.parent_target,
                                    src_size: viewport_size,
                                    dst_size: viewport_size,
                                    dst_scissor: None,
                                    load: wgpu::LoadOp::Load,
                                }));

                                let _ = draw_scopes.pop();
                            }
                        }
                    }

                    marker_ix += 1;
                }

                if cursor == draws.len() {
                    break;
                }

                continue;
            }

            if path_samples > 1 {
                if let OrderedDraw::Path(first) = &draws[cursor] {
                    flush_scene_range(
                        cursor,
                        &mut passes,
                        &mut draw_scopes,
                        &mut scene_range_start,
                    );

                    let batch_uniform_index = first.uniform_index;
                    let mut union = first.scissor;
                    let mut end = cursor + 1;
                    while end < draws.len() && end < next_marker_at {
                        match &draws[end] {
                            OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                                union = union_scissor(union, d.scissor);
                                end += 1;
                            }
                            _ => break,
                        }
                    }

                    let target = draw_scopes.last().expect("draw scope").target;
                    passes.push(RenderPlanPass::PathMsaaBatch(PathMsaaBatchPass {
                        segment: SceneSegmentId(0),
                        target,
                        draw_range: cursor..end,
                        union_scissor: union,
                        batch_uniform_index,
                    }));

                    cursor = end;
                    scene_range_start = cursor;
                    continue;
                }
            }

            cursor += 1;
        }

        let mut plan = Self { passes };
        append_postprocess(&mut plan, viewport_size, postprocess, clear);
        insert_early_releases(&mut plan.passes);
        plan
    }
}

fn effect_chain_has_gaussian_blur(chain: fret_core::EffectChain) -> bool {
    chain
        .iter()
        .any(|step| matches!(step, fret_core::EffectStep::GaussianBlur { .. }))
}

fn effect_chain_has_color_adjust(chain: fret_core::EffectChain) -> bool {
    chain
        .iter()
        .any(|step| matches!(step, fret_core::EffectStep::ColorAdjust { .. }))
}

fn choose_effect_blur_downsample_scale(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
    requested_downsample: u32,
    quality: fret_core::EffectQuality,
) -> Option<u32> {
    if budget_bytes == 0 {
        return None;
    }

    let full = estimate_texture_bytes(viewport_size, format, 1);
    let half = estimate_texture_bytes(downsampled_size(viewport_size, 2), format, 1);
    let quarter = estimate_texture_bytes(downsampled_size(viewport_size, 4), format, 1);

    let required_half = full.saturating_add(half.saturating_mul(2));
    let required_quarter = full.saturating_add(quarter.saturating_mul(2));

    let mut desired = match quality {
        fret_core::EffectQuality::Low => 4,
        fret_core::EffectQuality::Medium | fret_core::EffectQuality::High => 2,
        fret_core::EffectQuality::Auto => requested_downsample,
    };

    desired = if desired >= 4 { 4 } else { 2 };

    if desired == 2 && required_half <= budget_bytes {
        return Some(2);
    }
    if required_quarter <= budget_bytes {
        return Some(4);
    }
    None
}

fn color_adjust_enabled(
    viewport_size: (u32, u32),
    format: wgpu::TextureFormat,
    budget_bytes: u64,
) -> bool {
    if budget_bytes == 0 {
        return false;
    }
    let full = estimate_texture_bytes(viewport_size, format, 1);
    full.saturating_mul(2) <= budget_bytes
}

fn append_scissored_blur_in_place_two_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch_a: PlanTarget,
    scratch_b: PlanTarget,
    full_size: (u32, u32),
    downsample_scale: u32,
    scissor: ScissorRect,
    clear: wgpu::Color,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch_a, PlanTarget::Output);
    debug_assert_ne!(scratch_b, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch_a);
    debug_assert_ne!(srcdst, scratch_b);
    debug_assert_ne!(scratch_a, scratch_b);

    if scissor.w == 0 || scissor.h == 0 {
        return;
    }

    let downsample_scale = if downsample_scale >= 4 { 4 } else { 2 };
    let blur_size = downsampled_size(full_size, downsample_scale);

    let down_scissor = map_scissor_to_size(Some(scissor), full_size, blur_size);
    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: srcdst,
        dst: scratch_a,
        src_size: full_size,
        dst_size: blur_size,
        dst_scissor: down_scissor,
        mode: ScaleMode::Downsample,
        scale: downsample_scale,
        load: wgpu::LoadOp::Clear(clear),
    }));

    let blur_scissor = map_scissor_to_size(Some(scissor), full_size, blur_size);
    passes.push(RenderPlanPass::Blur(BlurPass {
        src: scratch_a,
        dst: scratch_b,
        src_size: blur_size,
        dst_size: blur_size,
        dst_scissor: blur_scissor,
        axis: BlurAxis::Horizontal,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::Blur(BlurPass {
        src: scratch_b,
        dst: scratch_a,
        src_size: blur_size,
        dst_size: blur_size,
        dst_scissor: blur_scissor,
        axis: BlurAxis::Vertical,
        load: wgpu::LoadOp::Clear(clear),
    }));

    let final_scissor = map_scissor_to_size(Some(scissor), full_size, full_size);
    passes.push(RenderPlanPass::ScaleNearest(ScaleNearestPass {
        src: scratch_a,
        dst: srcdst,
        src_size: blur_size,
        dst_size: full_size,
        dst_scissor: final_scissor,
        mode: ScaleMode::Upscale,
        scale: downsample_scale,
        load: wgpu::LoadOp::Load,
    }));
}

fn append_scissored_blur_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: ScissorRect,
    clear: wgpu::Color,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if scissor.w == 0 || scissor.h == 0 {
        return;
    }

    passes.push(RenderPlanPass::Blur(BlurPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: Some(scissor),
        axis: BlurAxis::Horizontal,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::Blur(BlurPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: Some(scissor),
        axis: BlurAxis::Vertical,
        load: wgpu::LoadOp::Load,
    }));
}

fn append_color_adjust_in_place_single_scratch(
    passes: &mut Vec<RenderPlanPass>,
    srcdst: PlanTarget,
    scratch: PlanTarget,
    size: (u32, u32),
    scissor: Option<ScissorRect>,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    clear: wgpu::Color,
) {
    debug_assert_ne!(srcdst, PlanTarget::Output);
    debug_assert_ne!(scratch, PlanTarget::Output);
    debug_assert_ne!(srcdst, scratch);

    if let Some(scissor) = scissor {
        if scissor.w == 0 || scissor.h == 0 {
            return;
        }

        passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
            src: srcdst,
            dst: scratch,
            src_size: size,
            dst_size: size,
            dst_scissor: None,
            load: wgpu::LoadOp::Clear(clear),
        }));
        passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
            src: scratch,
            dst: srcdst,
            src_size: size,
            dst_size: size,
            dst_scissor: Some(scissor),
            saturation,
            brightness,
            contrast,
            load: wgpu::LoadOp::Load,
        }));
        return;
    }

    passes.push(RenderPlanPass::ColorAdjust(ColorAdjustPass {
        src: srcdst,
        dst: scratch,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        saturation,
        brightness,
        contrast,
        load: wgpu::LoadOp::Clear(clear),
    }));
    passes.push(RenderPlanPass::FullscreenBlit(FullscreenBlitPass {
        src: scratch,
        dst: srcdst,
        src_size: size,
        dst_size: size,
        dst_scissor: None,
        load: wgpu::LoadOp::Clear(clear),
    }));
}

fn insert_early_releases(passes: &mut Vec<RenderPlanPass>) -> u64 {
    let mut last_use: [Option<usize>; 3] = [None, None, None];

    for (idx, pass) in passes.iter().enumerate() {
        let mut mark = |t: PlanTarget| {
            let slot = match t {
                PlanTarget::Intermediate0 => Some(0),
                PlanTarget::Intermediate1 => Some(1),
                PlanTarget::Intermediate2 => Some(2),
                PlanTarget::Output => None,
            };
            if let Some(slot) = slot {
                last_use[slot] = Some(idx);
            }
        };

        match pass {
            RenderPlanPass::SceneDrawRange(p) => mark(p.target),
            RenderPlanPass::PathMsaaBatch(p) => mark(p.target),
            RenderPlanPass::FullscreenBlit(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::CompositePremul(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::ScaleNearest(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::Blur(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::ColorAdjust(p) => {
                mark(p.src);
                mark(p.dst);
            }
            RenderPlanPass::ReleaseTarget(_target) => {}
        }
    }

    let last0 = last_use[0];
    let last1 = last_use[1];
    let last2 = last_use[2];

    let old = std::mem::take(passes);
    let mut out: Vec<RenderPlanPass> = Vec::with_capacity(old.len() + 3);
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
    }

    *passes = out;
    inserted
}

fn decompose_pixelate_scale(scale: u32) -> Vec<u32> {
    let mut scale = scale.max(1);
    let mut steps = Vec::new();
    while scale >= 4 && scale % 2 == 0 {
        steps.push(2);
        scale /= 2;
    }
    steps.push(scale.max(1));
    steps
}

fn map_scissor_to_size(
    scissor_in_full: Option<ScissorRect>,
    full_size: (u32, u32),
    dst_size: (u32, u32),
) -> Option<ScissorRect> {
    let scissor = scissor_in_full?;
    if scissor.w == 0 || scissor.h == 0 {
        return None;
    }

    let full_w = full_size.0.max(1) as u64;
    let full_h = full_size.1.max(1) as u64;
    let dst_w = dst_size.0.max(1) as u64;
    let dst_h = dst_size.1.max(1) as u64;

    let x0 = scissor.x as u64;
    let y0 = scissor.y as u64;
    let x1 = x0.saturating_add(scissor.w as u64);
    let y1 = y0.saturating_add(scissor.h as u64);

    let sx0 = (x0 * dst_w) / full_w;
    let sy0 = (y0 * dst_h) / full_h;
    let sx1 = (x1 * dst_w + full_w - 1) / full_w;
    let sy1 = (y1 * dst_h + full_h - 1) / full_h;

    let sx0 = sx0.min(dst_w);
    let sy0 = sy0.min(dst_h);
    let sx1 = sx1.min(dst_w);
    let sy1 = sy1.min(dst_h);

    if sx1 <= sx0 || sy1 <= sy0 {
        return None;
    }

    Some(ScissorRect {
        x: sx0 as u32,
        y: sy0 as u32,
        w: (sx1 - sx0) as u32,
        h: (sy1 - sy0) as u32,
    })
}

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
            dst_scissor,
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
) -> (PlanTarget, (u32, u32), Vec<((u32, u32), u32)>) {
    let mut stack: Vec<((u32, u32), u32)> = Vec::with_capacity(steps.len());
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

            let down_scissor = map_scissor_to_size(scissor, viewport_size, blur_size);
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

            let blur_scissor = map_scissor_to_size(scissor, viewport_size, blur_size);
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
                // For region-limited effects we must preserve the content outside the scissor.
                // Copy the base scene to the output first, then write the blurred region in-place.
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
mod tests {
    use super::*;

    fn strip_releases<'a>(passes: &'a [RenderPlanPass]) -> Vec<&'a RenderPlanPass> {
        passes
            .iter()
            .filter(|p| !matches!(p, RenderPlanPass::ReleaseTarget(_)))
            .collect()
    }

    #[test]
    fn compile_for_scene_none_targets_output() {
        let encoding = SceneEncoding::default();
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            (100, 100),
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::None,
            u64::MAX,
        );

        assert_eq!(plan.passes.len(), 1);
        let RenderPlanPass::SceneDrawRange(pass) = &plan.passes[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(pass.target, PlanTarget::Output);
    }

    #[test]
    fn compile_for_scene_offscreen_blit_adds_fullscreen_blit() {
        let encoding = SceneEncoding::default();
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            (100, 100),
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::OffscreenBlit,
            u64::MAX,
        );

        let core = strip_releases(&plan.passes);
        assert_eq!(core.len(), 2);
        let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);
        let RenderPlanPass::FullscreenBlit(blit) = core[1] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate0);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, (100, 100));
        assert_eq!(blit.dst_size, (100, 100));
        assert_eq!(blit.dst_scissor, None);

        assert!(
            plan.passes
                .iter()
                .any(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0))),
            "expected ReleaseTarget(Intermediate0)"
        );
    }

    #[test]
    fn compile_for_scene_pixelate_adds_scale_chain_then_blit() {
        let encoding = SceneEncoding::default();
        let viewport_size = (128, 64);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Pixelate { scale: 4 },
            u64::MAX,
        );

        let core = strip_releases(&plan.passes);
        assert_eq!(core.len(), 6);

        let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(down0) = core[1] else {
            panic!("expected ScaleNearest downsample pass 0");
        };
        assert_eq!(down0.src, PlanTarget::Intermediate0);
        assert_eq!(down0.dst, PlanTarget::Intermediate2);
        assert_eq!(down0.mode, ScaleMode::Downsample);
        assert_eq!(down0.scale, 2);
        assert_eq!(down0.src_size, viewport_size);
        assert_eq!(down0.dst_size, downsampled_size(viewport_size, 2));
        assert_eq!(down0.dst_scissor, None);

        let release0 = plan
            .passes
            .iter()
            .position(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0)))
            .expect("expected ReleaseTarget(Intermediate0)");
        let down0_idx = plan
            .passes
            .iter()
            .position(|p| {
                matches!(
                    p,
                    RenderPlanPass::ScaleNearest(p)
                        if p.mode == ScaleMode::Downsample
                            && p.src == PlanTarget::Intermediate0
                            && p.dst == PlanTarget::Intermediate2
                )
            })
            .unwrap();
        assert!(release0 > down0_idx);

        let RenderPlanPass::ScaleNearest(down1) = core[2] else {
            panic!("expected ScaleNearest downsample pass 1");
        };
        assert_eq!(down1.src, PlanTarget::Intermediate2);
        assert_eq!(down1.dst, PlanTarget::Intermediate1);
        assert_eq!(down1.mode, ScaleMode::Downsample);
        assert_eq!(down1.scale, 2);
        assert_eq!(down1.src_size, down0.dst_size);
        assert_eq!(down1.dst_size, downsampled_size(down0.dst_size, 2));
        assert_eq!(down1.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(up0) = core[3] else {
            panic!("expected ScaleNearest upscale pass 0");
        };
        assert_eq!(up0.src, PlanTarget::Intermediate1);
        assert_eq!(up0.dst, PlanTarget::Intermediate2);
        assert_eq!(up0.mode, ScaleMode::Upscale);
        assert_eq!(up0.scale, 2);
        assert_eq!(up0.src_size, down1.dst_size);
        assert_eq!(up0.dst_size, down1.src_size);
        assert_eq!(up0.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(up1) = core[4] else {
            panic!("expected ScaleNearest upscale pass 1");
        };
        assert_eq!(up1.src, PlanTarget::Intermediate2);
        assert_eq!(up1.dst, PlanTarget::Intermediate1);
        assert_eq!(up1.mode, ScaleMode::Upscale);
        assert_eq!(up1.scale, 2);
        assert_eq!(up1.src_size, up0.dst_size);
        assert_eq!(up1.dst_size, viewport_size);
        assert_eq!(up1.dst_scissor, None);

        let RenderPlanPass::FullscreenBlit(blit) = core[5] else {
            panic!("expected FullscreenBlit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate1);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, viewport_size);
        assert_eq!(blit.dst_size, viewport_size);
        assert_eq!(blit.dst_scissor, None);
        let releases: Vec<PlanTarget> = plan
            .passes
            .iter()
            .filter_map(|p| match p {
                RenderPlanPass::ReleaseTarget(t) => Some(*t),
                _ => None,
            })
            .collect();
        assert!(releases.contains(&PlanTarget::Intermediate0));
        assert!(releases.contains(&PlanTarget::Intermediate1));
        assert!(releases.contains(&PlanTarget::Intermediate2));
    }

    #[test]
    fn downsample_half_quarter_helper_emits_two_passes() {
        let mut plan = RenderPlan { passes: Vec::new() };
        let info = append_downsample_half_quarter(
            &mut plan,
            PlanTarget::Intermediate0,
            (128, 64),
            PlanTarget::Intermediate2,
            PlanTarget::Intermediate1,
            None,
            (128, 64),
            wgpu::Color::TRANSPARENT,
        );

        assert_eq!(info.half_size, (64, 32));
        assert_eq!(info.quarter_size, (32, 16));
        assert_eq!(info.stack, vec![((128, 64), 2), ((64, 32), 2)]);

        assert_eq!(plan.passes.len(), 2);
        let RenderPlanPass::ScaleNearest(pass0) = &plan.passes[0] else {
            panic!("expected ScaleNearest pass 0");
        };
        assert_eq!(pass0.src, PlanTarget::Intermediate0);
        assert_eq!(pass0.dst, PlanTarget::Intermediate2);
        assert_eq!(pass0.src_size, (128, 64));
        assert_eq!(pass0.dst_size, (64, 32));
        assert_eq!(pass0.mode, ScaleMode::Downsample);
        assert_eq!(pass0.scale, 2);
        assert_eq!(pass0.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(pass1) = &plan.passes[1] else {
            panic!("expected ScaleNearest pass 1");
        };
        assert_eq!(pass1.src, PlanTarget::Intermediate2);
        assert_eq!(pass1.dst, PlanTarget::Intermediate1);
        assert_eq!(pass1.src_size, (64, 32));
        assert_eq!(pass1.dst_size, (32, 16));
        assert_eq!(pass1.mode, ScaleMode::Downsample);
        assert_eq!(pass1.scale, 2);
        assert_eq!(pass1.dst_scissor, None);
    }

    #[test]
    fn compile_for_scene_blur_emits_separable_passes() {
        let encoding = SceneEncoding::default();
        let viewport_size = (128, 64);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Blur {
                radius: 2,
                downsample_scale: 2,
                scissor: None,
            },
            u64::MAX,
        );

        let core = strip_releases(&plan.passes);
        assert_eq!(core.len(), 6);

        let RenderPlanPass::SceneDrawRange(scene) = core[0] else {
            panic!("expected SceneDrawRange pass");
        };
        assert_eq!(scene.target, PlanTarget::Intermediate0);

        let RenderPlanPass::ScaleNearest(down) = core[1] else {
            panic!("expected downsample pass");
        };
        assert_eq!(down.mode, ScaleMode::Downsample);
        assert_eq!(down.src, PlanTarget::Intermediate0);
        assert_eq!(down.dst, PlanTarget::Intermediate2);
        assert_eq!(down.src_size, viewport_size);
        assert_eq!(down.dst_size, (64, 32));

        let release0 = plan
            .passes
            .iter()
            .position(|p| matches!(p, RenderPlanPass::ReleaseTarget(PlanTarget::Intermediate0)))
            .expect("expected ReleaseTarget(Intermediate0)");
        let down0_idx = plan
            .passes
            .iter()
            .position(
                |p| matches!(p, RenderPlanPass::ScaleNearest(p) if p.mode == ScaleMode::Downsample),
            )
            .unwrap();
        assert!(release0 > down0_idx);

        let RenderPlanPass::Blur(blur_h) = core[2] else {
            panic!("expected blur-h pass");
        };
        assert_eq!(blur_h.axis, BlurAxis::Horizontal);
        assert_eq!(blur_h.src, PlanTarget::Intermediate2);
        assert_eq!(blur_h.dst, PlanTarget::Intermediate1);
        assert_eq!(blur_h.src_size, (64, 32));
        assert_eq!(blur_h.dst_size, (64, 32));
        assert_eq!(blur_h.dst_scissor, None);

        let RenderPlanPass::Blur(blur_v) = core[3] else {
            panic!("expected blur-v pass");
        };
        assert_eq!(blur_v.axis, BlurAxis::Vertical);
        assert_eq!(blur_v.src, PlanTarget::Intermediate1);
        assert_eq!(blur_v.dst, PlanTarget::Intermediate2);
        assert_eq!(blur_v.src_size, (64, 32));
        assert_eq!(blur_v.dst_size, (64, 32));
        assert_eq!(blur_v.dst_scissor, None);

        let RenderPlanPass::ScaleNearest(upscale) = core[4] else {
            panic!("expected upscale pass");
        };
        assert_eq!(upscale.src, PlanTarget::Intermediate2);
        assert_eq!(upscale.dst, PlanTarget::Intermediate0);
        assert_eq!(upscale.src_size, (64, 32));
        assert_eq!(upscale.dst_size, viewport_size);
        assert_eq!(upscale.mode, ScaleMode::Upscale);
        assert_eq!(upscale.scale, 2);
        assert_eq!(upscale.dst_scissor, None);

        let RenderPlanPass::FullscreenBlit(blit) = core[5] else {
            panic!("expected blit pass");
        };
        assert_eq!(blit.src, PlanTarget::Intermediate0);
        assert_eq!(blit.dst, PlanTarget::Output);
        assert_eq!(blit.src_size, viewport_size);
        assert_eq!(blit.dst_size, viewport_size);
        assert_eq!(blit.dst_scissor, None);

        let releases: Vec<PlanTarget> = plan
            .passes
            .iter()
            .filter_map(|p| match p {
                RenderPlanPass::ReleaseTarget(t) => Some(*t),
                _ => None,
            })
            .collect();
        assert!(releases.contains(&PlanTarget::Intermediate0));
        assert!(releases.contains(&PlanTarget::Intermediate1));
        assert!(releases.contains(&PlanTarget::Intermediate2));
    }

    #[test]
    fn blur_scissor_is_mapped_per_pass_dst_size() {
        let encoding = SceneEncoding::default();
        let viewport_size = (100, 100);
        let plan = RenderPlan::compile_for_scene(
            &encoding,
            viewport_size,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::Color::TRANSPARENT,
            1,
            DebugPostprocess::Blur {
                radius: 2,
                downsample_scale: 2,
                scissor: Some(ScissorRect {
                    x: 10,
                    y: 10,
                    w: 50,
                    h: 50,
                }),
            },
            u64::MAX,
        );

        // Half target is (50, 50) for 100x100.
        let half = plan
            .passes
            .iter()
            .find_map(|p| match p {
                RenderPlanPass::ScaleNearest(p) if p.mode == ScaleMode::Downsample => Some(*p),
                _ => None,
            })
            .expect("expected half downsample pass");
        assert_eq!(
            half.dst_scissor,
            Some(ScissorRect {
                x: 5,
                y: 5,
                w: 25,
                h: 25
            })
        );
        let blur_h = plan
            .passes
            .iter()
            .find_map(|p| match p {
                RenderPlanPass::Blur(p) if p.axis == BlurAxis::Horizontal => Some(*p),
                _ => None,
            })
            .expect("expected blur-h pass");
        assert_eq!(
            blur_h.dst_scissor,
            Some(ScissorRect {
                x: 5,
                y: 5,
                w: 25,
                h: 25
            })
        );
        let base_blit = plan
            .passes
            .iter()
            .find_map(|p| match p {
                RenderPlanPass::FullscreenBlit(p)
                    if p.src == PlanTarget::Intermediate0 && p.dst == PlanTarget::Output =>
                {
                    Some(*p)
                }
                _ => None,
            })
            .expect("expected base blit pass");
        assert_eq!(base_blit.dst_scissor, None);

        let upscale = plan
            .passes
            .iter()
            .find_map(|p| match p {
                RenderPlanPass::ScaleNearest(p)
                    if p.mode == ScaleMode::Upscale && p.dst == PlanTarget::Output =>
                {
                    Some(*p)
                }
                _ => None,
            })
            .expect("expected upscale-to-output pass");
        assert_eq!(
            upscale.dst_scissor,
            Some(ScissorRect {
                x: 10,
                y: 10,
                w: 50,
                h: 50
            })
        );
    }
}
