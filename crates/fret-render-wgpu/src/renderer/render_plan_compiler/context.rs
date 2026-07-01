use super::super::*;
use super::draw_scope::DrawScopeStack;
use slotmap::Key;
use std::ops::Range;

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

pub(super) struct RenderPlanCompilerCtx {
    passes: Vec<RenderPlanPass>,
    segments: Vec<RenderPlanSegment>,
    degradations: Vec<RenderPlanDegradation>,
    next_segment_id: usize,
}

impl RenderPlanCompilerCtx {
    pub(super) fn new() -> Self {
        Self {
            passes: Vec::new(),
            segments: Vec::new(),
            degradations: Vec::new(),
            next_segment_id: 0,
        }
    }

    pub(super) fn passes_mut(&mut self) -> &mut Vec<RenderPlanPass> {
        &mut self.passes
    }

    pub(super) fn passes_len(&self) -> usize {
        self.passes.len()
    }

    pub(super) fn push_pass(&mut self, pass: RenderPlanPass) {
        self.passes.push(pass);
    }

    pub(super) fn push_degradation(&mut self, degradation: RenderPlanDegradation) {
        self.degradations.push(degradation);
    }

    pub(super) fn alloc_segment(
        &mut self,
        draw_range: Range<usize>,
        draws: &[OrderedDraw],
        encoding: &SceneEncoding,
    ) -> SceneSegmentId {
        let id = SceneSegmentId(self.next_segment_id);
        self.next_segment_id += 1;

        let start_uniform_index = draws.get(draw_range.start).map(|d| match d {
            OrderedDraw::Quad(d) => d.uniform_index,
            OrderedDraw::VertexColor(d) => d.uniform_index,
            OrderedDraw::Viewport(d) => d.uniform_index,
            OrderedDraw::Image(d) => d.uniform_index,
            OrderedDraw::Mask(d) => d.uniform_index,
            OrderedDraw::Text(d) => d.uniform_index,
            OrderedDraw::Path(d) => d.uniform_index,
        });

        fn mix_fnv1a(mut hash: u64, value: u64) -> u64 {
            hash ^= value;
            hash = hash.wrapping_mul(1099511628211);
            hash
        }

        let start_uniform_fingerprint = if let Some(start_uniform_index) = start_uniform_index
            && let Some(uniform) = encoding.uniforms.get(start_uniform_index as usize)
        {
            let mut hash: u64 = 14695981039346656037;
            hash = mix_fnv1a(hash, u64::from(uniform.clip_head));
            hash = mix_fnv1a(hash, u64::from(uniform.clip_count));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_head));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_count));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_scope_head));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_scope_count));
            hash = mix_fnv1a(hash, u64::from(uniform.output_is_srgb));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_viewport_origin[0].to_bits()));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_viewport_origin[1].to_bits()));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_viewport_size[0].to_bits()));
            hash = mix_fnv1a(hash, u64::from(uniform.mask_viewport_size[1].to_bits()));

            let mask_image = encoding
                .uniform_mask_images
                .get(start_uniform_index as usize)
                .copied()
                .flatten();
            hash = mix_fnv1a(
                hash,
                mask_image.map(|sel| sel.image.data().as_ffi()).unwrap_or(0),
            );
            hash = mix_fnv1a(
                hash,
                mask_image
                    .map(|sel| match sel.sampling {
                        fret_core::scene::ImageSamplingHint::Default => 0,
                        fret_core::scene::ImageSamplingHint::Linear => 1,
                        fret_core::scene::ImageSamplingHint::Nearest => 2,
                    })
                    .unwrap_or(0),
            );
            hash
        } else {
            0
        };

        let mut flags = RenderPlanSegmentFlags::default();
        for draw in draws.get(draw_range.start..draw_range.end).unwrap_or(&[]) {
            match draw {
                OrderedDraw::Quad(_) => flags.has_quad = true,
                OrderedDraw::VertexColor(_) => flags.has_vertex_color = true,
                OrderedDraw::Viewport(_) => flags.has_viewport = true,
                OrderedDraw::Image(_) => flags.has_image = true,
                OrderedDraw::Mask(_) => flags.has_mask = true,
                OrderedDraw::Text(_) => flags.has_text = true,
                OrderedDraw::Path(_) => flags.has_path = true,
            }
        }

        self.segments.push(RenderPlanSegment {
            id,
            scene_chunk_candidate: scene_chunk_candidate(
                draw_range.clone(),
                draws,
                start_uniform_fingerprint,
                flags,
            ),
            stream_ranges: segment_stream_ranges(draw_range.clone(), draws),
            draw_range,
            start_uniform_index,
            start_uniform_fingerprint,
            flags,
        });

        id
    }

    pub(super) fn flush_scene_range(
        &mut self,
        end: usize,
        draw_scopes: &mut DrawScopeStack,
        draws: &[OrderedDraw],
        encoding: &SceneEncoding,
        scene_range_start: &mut usize,
    ) {
        let scope = draw_scopes.current_mut();
        if scope.needs_clear {
            let segment = self.alloc_segment(*scene_range_start..end, draws, encoding);
            self.push_pass(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment,
                target: scope.target,
                target_origin: scope.origin,
                target_size: scope.size,
                load: wgpu::LoadOp::Clear(scope.clear_color),
                draw_range: *scene_range_start..end,
            }));
            scope.needs_clear = false;
        } else if *scene_range_start < end {
            let segment = self.alloc_segment(*scene_range_start..end, draws, encoding);
            self.push_pass(RenderPlanPass::SceneDrawRange(SceneDrawRangePass {
                segment,
                target: scope.target,
                target_origin: scope.origin,
                target_size: scope.size,
                load: wgpu::LoadOp::Load,
                draw_range: *scene_range_start..end,
            }));
        }
        *scene_range_start = end;
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        Vec<RenderPlanSegment>,
        Vec<RenderPlanPass>,
        Vec<RenderPlanDegradation>,
    ) {
        (self.segments, self.passes, self.degradations)
    }
}

fn mix_fnv1a(mut hash: u64, value: u64) -> u64 {
    hash ^= value;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash
}

fn mix_scissor(hash: &mut u64, scissor: ScissorRect) {
    *hash = mix_fnv1a(*hash, u64::from(scissor.x));
    *hash = mix_fnv1a(*hash, u64::from(scissor.y));
    *hash = mix_fnv1a(*hash, u64::from(scissor.w));
    *hash = mix_fnv1a(*hash, u64::from(scissor.h));
}

fn segment_stream_ranges(
    draw_range: Range<usize>,
    draws: &[OrderedDraw],
) -> RenderPlanSegmentStreamRanges {
    let mut ranges = RenderPlanSegmentStreamRanges::default();
    for draw in draws.get(draw_range).unwrap_or(&[]) {
        match draw {
            OrderedDraw::Quad(draw) => ranges.quad_instances.extend(
                draw.first_instance,
                draw.first_instance.saturating_add(draw.instance_count),
            ),
            OrderedDraw::Viewport(draw) => ranges.viewport_vertices.extend(
                draw.first_vertex,
                draw.first_vertex.saturating_add(draw.vertex_count),
            ),
            OrderedDraw::Image(draw) => ranges.viewport_vertices.extend(
                draw.first_vertex,
                draw.first_vertex.saturating_add(draw.vertex_count),
            ),
            OrderedDraw::VertexColor(draw) => ranges.viewport_vertices.extend(
                draw.first_vertex,
                draw.first_vertex.saturating_add(draw.vertex_count),
            ),
            OrderedDraw::Mask(draw) => ranges.text_vertices.extend(
                draw.first_vertex,
                draw.first_vertex.saturating_add(draw.vertex_count),
            ),
            OrderedDraw::Text(draw) => {
                ranges.text_glyph_instances.extend(
                    draw.first_instance,
                    draw.first_instance.saturating_add(draw.instance_count),
                );
                ranges
                    .text_paints
                    .extend(draw.paint_index, draw.paint_index.saturating_add(1));
            }
            OrderedDraw::Path(draw) => {
                ranges.path_vertices.extend(
                    draw.first_vertex,
                    draw.first_vertex.saturating_add(draw.vertex_count),
                );
                ranges
                    .path_paints
                    .extend(draw.paint_index, draw.paint_index.saturating_add(1));
            }
        }
    }
    ranges
}

fn scene_chunk_candidate(
    draw_range: Range<usize>,
    draws: &[OrderedDraw],
    start_uniform_fingerprint: u64,
    flags: RenderPlanSegmentFlags,
) -> RenderPlanSceneChunkCandidate {
    let segment_draws = draws.get(draw_range.clone()).unwrap_or(&[]);
    let draw_count = u32::try_from(segment_draws.len()).unwrap_or(u32::MAX);
    if segment_draws.is_empty() {
        return RenderPlanSceneChunkCandidate {
            eligible: false,
            draw_count: 0,
            fingerprint: 0,
        };
    }

    let mut hash = FNV_OFFSET_BASIS;
    hash = mix_fnv1a(hash, draw_range.start as u64);
    hash = mix_fnv1a(hash, draw_range.end as u64);
    hash = mix_fnv1a(hash, start_uniform_fingerprint);
    hash = mix_fnv1a(hash, flags.diagnostics_mask().into());
    for draw in segment_draws {
        mix_ordered_draw(&mut hash, draw);
    }

    RenderPlanSceneChunkCandidate {
        eligible: true,
        draw_count,
        fingerprint: hash,
    }
}

fn mix_ordered_draw(hash: &mut u64, draw: &OrderedDraw) {
    match draw {
        OrderedDraw::Quad(draw) => {
            *hash = mix_fnv1a(*hash, 1);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_instance));
            *hash = mix_fnv1a(*hash, u64::from(draw.instance_count));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.fill_kind));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.border_kind));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.border_present));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.dash_enabled));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.fill_material_sampled));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.border_material_sampled));
            *hash = mix_fnv1a(*hash, u64::from(draw.pipeline.shadow_mode));
        }
        OrderedDraw::Viewport(draw) => {
            *hash = mix_fnv1a(*hash, 2);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_vertex));
            *hash = mix_fnv1a(*hash, u64::from(draw.vertex_count));
            *hash = mix_fnv1a(*hash, draw.target.data().as_ffi());
        }
        OrderedDraw::Image(draw) => {
            *hash = mix_fnv1a(*hash, 3);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_vertex));
            *hash = mix_fnv1a(*hash, u64::from(draw.vertex_count));
            *hash = mix_fnv1a(*hash, draw.image.data().as_ffi());
            *hash = mix_fnv1a(*hash, image_sampling_hint_id(draw.sampling));
        }
        OrderedDraw::VertexColor(draw) => {
            *hash = mix_fnv1a(*hash, 4);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_vertex));
            *hash = mix_fnv1a(*hash, u64::from(draw.vertex_count));
        }
        OrderedDraw::Mask(draw) => {
            *hash = mix_fnv1a(*hash, 5);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_vertex));
            *hash = mix_fnv1a(*hash, u64::from(draw.vertex_count));
            *hash = mix_fnv1a(*hash, draw.image.data().as_ffi());
            *hash = mix_fnv1a(*hash, image_sampling_hint_id(draw.sampling));
        }
        OrderedDraw::Text(draw) => {
            *hash = mix_fnv1a(*hash, 6);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_instance));
            *hash = mix_fnv1a(*hash, u64::from(draw.instance_count));
            *hash = mix_fnv1a(*hash, text_draw_kind_id(draw.kind));
            *hash = mix_fnv1a(*hash, u64::from(draw.atlas_page));
            *hash = mix_fnv1a(*hash, u64::from(draw.paint_index));
        }
        OrderedDraw::Path(draw) => {
            *hash = mix_fnv1a(*hash, 7);
            mix_scissor(hash, draw.scissor);
            *hash = mix_fnv1a(*hash, u64::from(draw.uniform_index));
            *hash = mix_fnv1a(*hash, u64::from(draw.first_vertex));
            *hash = mix_fnv1a(*hash, u64::from(draw.vertex_count));
            *hash = mix_fnv1a(*hash, u64::from(draw.paint_index));
        }
    }
}

fn image_sampling_hint_id(sampling: fret_core::scene::ImageSamplingHint) -> u64 {
    match sampling {
        fret_core::scene::ImageSamplingHint::Default => 0,
        fret_core::scene::ImageSamplingHint::Linear => 1,
        fret_core::scene::ImageSamplingHint::Nearest => 2,
    }
}

fn text_draw_kind_id(kind: TextDrawKind) -> u64 {
    match kind {
        TextDrawKind::Mask => 0,
        TextDrawKind::MaskOutline => 1,
        TextDrawKind::Color => 2,
        TextDrawKind::Subpixel => 3,
        TextDrawKind::SubpixelOutline => 4,
    }
}
