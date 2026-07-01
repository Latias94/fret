use super::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) struct GeometryUploadState {
    quad_instances: buffers::StorageRingBuffer<QuadInstance>,
    path_paints: buffers::StorageRingBuffer<PaintGpu>,
    text_paints: buffers::StorageRingBuffer<PaintGpu>,
    viewport_vertices: buffers::RingBuffer<ViewportVertex>,
    text_glyph_instances: buffers::RingBuffer<TextGlyphInstance>,
    text_vertices: buffers::RingBuffer<TextVertex>,
    path_vertices: buffers::RingBuffer<PathVertex>,
    resident_uploads: ResidentGeometryUploadState,
}

pub(super) struct FrameGeometryUploads {
    pub(super) quad_instance_bind_group: wgpu::BindGroup,
    pub(super) text_paint_bind_group: wgpu::BindGroup,
    pub(super) path_paint_bind_group: wgpu::BindGroup,
    pub(super) viewport_vertex_buffer: wgpu::Buffer,
    pub(super) text_glyph_instance_buffer: wgpu::Buffer,
    pub(super) text_vertex_buffer: wgpu::Buffer,
    pub(super) path_vertex_buffer: wgpu::Buffer,
}

#[derive(Default)]
struct ResidentGeometryUploadState {
    quad_instances: ResidentGeometryUploadStreamState,
    path_paints: ResidentGeometryUploadStreamState,
    text_paints: ResidentGeometryUploadStreamState,
    viewport_vertices: ResidentGeometryUploadStreamState,
    text_glyph_instances: ResidentGeometryUploadStreamState,
    text_vertices: ResidentGeometryUploadStreamState,
    path_vertices: ResidentGeometryUploadStreamState,
}

#[derive(Debug, Clone, Copy)]
struct ResidentGeometryUploadSlots {
    quad_instances: usize,
    path_paints: usize,
    text_paints: usize,
    viewport_vertices: usize,
    text_glyph_instances: usize,
    text_vertices: usize,
    path_vertices: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct ResidentGeometryUploadInvalidations {
    quad_instances: bool,
    path_paints: bool,
    text_paints: bool,
    viewport_vertices: bool,
    text_glyph_instances: bool,
    text_vertices: bool,
    path_vertices: bool,
}

#[derive(Default)]
struct ResidentGeometryUploadStreamState {
    slots: Vec<Option<ResidentGeometryUploadStreamSignature>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResidentGeometryUploadStreamSignature {
    fingerprint: u64,
    range: RenderPlanStreamRange,
}

#[derive(Default)]
struct ResidentGeometryUploadFrameSignatures {
    quad_instances: Option<ResidentGeometryUploadStreamSignature>,
    path_paints: Option<ResidentGeometryUploadStreamSignature>,
    text_paints: Option<ResidentGeometryUploadStreamSignature>,
    viewport_vertices: Option<ResidentGeometryUploadStreamSignature>,
    text_glyph_instances: Option<ResidentGeometryUploadStreamSignature>,
    text_vertices: Option<ResidentGeometryUploadStreamSignature>,
    path_vertices: Option<ResidentGeometryUploadStreamSignature>,
}

#[derive(Default)]
struct ResidentGeometryUploadStreamAccumulator {
    range: RenderPlanStreamRange,
    fingerprint: u64,
    has_range: bool,
}

impl ResidentGeometryUploadState {
    fn record_frame(
        &mut self,
        plan: &RenderPlan,
        frame_perf: &mut RenderPerfStats,
        slots: ResidentGeometryUploadSlots,
        invalidations: ResidentGeometryUploadInvalidations,
    ) {
        let eligible_candidates = plan
            .segments
            .iter()
            .filter(|segment| segment.scene_chunk_candidate.eligible)
            .count();
        if eligible_candidates == 0 {
            frame_perf
                .geometry_upload
                .record_resident_full_upload_fallback_no_candidate();
            return;
        }

        let missing_payloads =
            frame_perf.scene_chunk_encoding_payload_plan_candidates_without_payload;
        if missing_payloads > 0 {
            frame_perf
                .geometry_upload
                .record_resident_full_upload_fallback_missing_payload(missing_payloads);
        }

        let blocked_reassembly = frame_perf
            .scene_chunk_encoding_payload_reassembly_blocked_by_shape_mismatch
            .saturating_add(
                frame_perf
                    .scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch,
            )
            .saturating_add(
                frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_non_quad_draws,
            )
            .saturating_add(
                frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_side_tables,
            )
            .saturating_add(
                frame_perf.scene_chunk_encoding_payload_reassembly_blocked_by_material_state,
            );
        if blocked_reassembly > 0 {
            frame_perf
                .geometry_upload
                .record_resident_full_upload_fallback_reassembly_blocked(blocked_reassembly);
        }

        let safe_candidates =
            frame_perf.scene_chunk_encoding_payload_reassembly_append_only_matches as usize;
        if safe_candidates == 0 {
            return;
        }

        let signatures = ResidentGeometryUploadFrameSignatures::from_plan(plan, safe_candidates);
        self.quad_instances.record_frame(
            slots.quad_instances,
            invalidations.quad_instances,
            signatures.quad_instances,
            estimate_range_bytes::<QuadInstance>,
            &mut frame_perf.geometry_upload,
        );
        self.path_paints.record_frame(
            slots.path_paints,
            invalidations.path_paints,
            signatures.path_paints,
            estimate_range_bytes::<PaintGpu>,
            &mut frame_perf.geometry_upload,
        );
        self.text_paints.record_frame(
            slots.text_paints,
            invalidations.text_paints,
            signatures.text_paints,
            estimate_range_bytes::<PaintGpu>,
            &mut frame_perf.geometry_upload,
        );
        self.viewport_vertices.record_frame(
            slots.viewport_vertices,
            invalidations.viewport_vertices,
            signatures.viewport_vertices,
            estimate_range_bytes::<ViewportVertex>,
            &mut frame_perf.geometry_upload,
        );
        self.text_glyph_instances.record_frame(
            slots.text_glyph_instances,
            invalidations.text_glyph_instances,
            signatures.text_glyph_instances,
            estimate_range_bytes::<TextGlyphInstance>,
            &mut frame_perf.geometry_upload,
        );
        self.text_vertices.record_frame(
            slots.text_vertices,
            invalidations.text_vertices,
            signatures.text_vertices,
            estimate_range_bytes::<TextVertex>,
            &mut frame_perf.geometry_upload,
        );
        self.path_vertices.record_frame(
            slots.path_vertices,
            invalidations.path_vertices,
            signatures.path_vertices,
            estimate_range_bytes::<PathVertex>,
            &mut frame_perf.geometry_upload,
        );
    }
}

impl ResidentGeometryUploadFrameSignatures {
    fn from_plan(plan: &RenderPlan, safe_candidates: usize) -> Self {
        let mut quad_instances = ResidentGeometryUploadStreamAccumulator::default();
        let mut path_paints = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_paints = ResidentGeometryUploadStreamAccumulator::default();
        let mut viewport_vertices = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_glyph_instances = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_vertices = ResidentGeometryUploadStreamAccumulator::default();
        let mut path_vertices = ResidentGeometryUploadStreamAccumulator::default();

        for segment in plan
            .segments
            .iter()
            .filter(|segment| segment.scene_chunk_candidate.eligible)
            .take(safe_candidates)
        {
            let fingerprint = segment.scene_chunk_candidate.fingerprint;
            let ranges = segment.stream_ranges;
            quad_instances.include(fingerprint, ranges.quad_instances);
            path_paints.include(fingerprint, ranges.path_paints);
            text_paints.include(fingerprint, ranges.text_paints);
            viewport_vertices.include(fingerprint, ranges.viewport_vertices);
            text_glyph_instances.include(fingerprint, ranges.text_glyph_instances);
            text_vertices.include(fingerprint, ranges.text_vertices);
            path_vertices.include(fingerprint, ranges.path_vertices);
        }

        Self {
            quad_instances: quad_instances.finish(),
            path_paints: path_paints.finish(),
            text_paints: text_paints.finish(),
            viewport_vertices: viewport_vertices.finish(),
            text_glyph_instances: text_glyph_instances.finish(),
            text_vertices: text_vertices.finish(),
            path_vertices: path_vertices.finish(),
        }
    }
}

impl ResidentGeometryUploadStreamAccumulator {
    fn include(&mut self, candidate_fingerprint: u64, range: RenderPlanStreamRange) {
        if range.is_empty() {
            return;
        }

        self.has_range = true;
        self.range.extend(range.start, range.end);

        let mut hasher = DefaultHasher::new();
        self.fingerprint.hash(&mut hasher);
        candidate_fingerprint.hash(&mut hasher);
        range.start.hash(&mut hasher);
        range.end.hash(&mut hasher);
        self.fingerprint = hasher.finish();
    }

    fn finish(self) -> Option<ResidentGeometryUploadStreamSignature> {
        self.has_range
            .then_some(ResidentGeometryUploadStreamSignature {
                fingerprint: self.fingerprint,
                range: self.range,
            })
    }
}

impl ResidentGeometryUploadStreamState {
    fn record_frame(
        &mut self,
        slot: usize,
        invalidated: bool,
        signature: Option<ResidentGeometryUploadStreamSignature>,
        estimate_bytes: impl FnOnce(RenderPlanStreamRange) -> u64,
        upload: &mut GeometryUploadPerfSnapshot,
    ) {
        let Some(signature) = signature else {
            return;
        };

        upload.resident_stream_candidates = upload.resident_stream_candidates.saturating_add(1);
        if invalidated {
            self.slots.clear();
        }
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }

        match self.slots[slot] {
            Some(previous) if previous == signature => {
                upload.resident_stream_hits = upload.resident_stream_hits.saturating_add(1);
            }
            Some(_) => {
                upload.record_resident_stream_miss(estimate_bytes(signature.range));
                upload.resident_full_upload_fallbacks_stream_layout_changed = upload
                    .resident_full_upload_fallbacks_stream_layout_changed
                    .saturating_add(1);
                self.slots[slot] = Some(signature);
            }
            None => {
                upload.record_resident_stream_miss(estimate_bytes(signature.range));
                if invalidated {
                    upload.resident_full_upload_fallbacks_buffer_resized = upload
                        .resident_full_upload_fallbacks_buffer_resized
                        .saturating_add(1);
                } else {
                    upload.resident_full_upload_fallbacks_uninitialized = upload
                        .resident_full_upload_fallbacks_uninitialized
                        .saturating_add(1);
                }
                self.slots[slot] = Some(signature);
            }
        }
    }
}

impl GeometryUploadPerfSnapshot {
    fn record_resident_stream_miss(&mut self, dirty_bytes: u64) {
        self.resident_stream_misses = self.resident_stream_misses.saturating_add(1);
        self.resident_dirty_range_bytes_estimate = self
            .resident_dirty_range_bytes_estimate
            .saturating_add(dirty_bytes);
        self.resident_full_upload_fallbacks = self.resident_full_upload_fallbacks.saturating_add(1);
    }

    fn record_resident_full_upload_fallback_no_candidate(&mut self) {
        self.resident_full_upload_fallbacks = self.resident_full_upload_fallbacks.saturating_add(1);
        self.resident_full_upload_fallbacks_no_candidate = self
            .resident_full_upload_fallbacks_no_candidate
            .saturating_add(1);
    }

    fn record_resident_full_upload_fallback_missing_payload(&mut self, count: u64) {
        self.resident_full_upload_fallbacks =
            self.resident_full_upload_fallbacks.saturating_add(count);
        self.resident_full_upload_fallbacks_missing_payload = self
            .resident_full_upload_fallbacks_missing_payload
            .saturating_add(count);
    }

    fn record_resident_full_upload_fallback_reassembly_blocked(&mut self, count: u64) {
        self.resident_full_upload_fallbacks =
            self.resident_full_upload_fallbacks.saturating_add(count);
        self.resident_full_upload_fallbacks_reassembly_blocked = self
            .resident_full_upload_fallbacks_reassembly_blocked
            .saturating_add(count);
    }
}

fn estimate_range_bytes<T>(range: RenderPlanStreamRange) -> u64 {
    u64::from(range.len()).saturating_mul(std::mem::size_of::<T>() as u64)
}

impl GeometryUploadState {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        const FRAMES_IN_FLIGHT: usize = 3;

        let quad_instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad instances bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let instance_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let quad_instances = buffers::StorageRingBuffer::<QuadInstance>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            quad_instance_bind_group_layout,
            "fret quad instances",
            instance_usage,
        );

        let path_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret path paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let paint_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let path_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            path_paint_bind_group_layout,
            "fret path paints",
            paint_usage,
        );

        let text_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret text paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let text_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            text_paint_bind_group_layout,
            "fret text paints",
            paint_usage,
        );

        let vertex_usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let viewport_vertices = buffers::RingBuffer::<ViewportVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            64 * 6,
            "fret viewport vertices",
            vertex_usage,
        );
        let text_glyph_instances = buffers::RingBuffer::<TextGlyphInstance>::new(
            device,
            FRAMES_IN_FLIGHT,
            512 * 6,
            "fret text glyph instances",
            vertex_usage,
        );
        let text_vertices = buffers::RingBuffer::<TextVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            512 * 6,
            "fret text vertices",
            vertex_usage,
        );
        let path_vertices = buffers::RingBuffer::<PathVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            "fret path vertices",
            vertex_usage,
        );

        Self {
            quad_instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_glyph_instances,
            text_vertices,
            path_vertices,
            resident_uploads: ResidentGeometryUploadState::default(),
        }
    }

    pub(super) fn quad_instances_layout(&self) -> &wgpu::BindGroupLayout {
        self.quad_instances.layout()
    }

    pub(super) fn path_paints_layout(&self) -> &wgpu::BindGroupLayout {
        self.path_paints.layout()
    }

    pub(super) fn text_paints_layout(&self) -> &wgpu::BindGroupLayout {
        self.text_paints.layout()
    }

    pub(super) fn upload_frame_geometry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &RenderPlan,
        instances: &[QuadInstance],
        path_paints: &[PaintGpu],
        text_paints: &[PaintGpu],
        viewport_vertices: &[ViewportVertex],
        text_glyph_instances: &[TextGlyphInstance],
        text_vertices: &[TextVertex],
        path_vertices: &[PathVertex],
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) -> FrameGeometryUploads {
        fn record_instance_upload(
            perf_enabled: bool,
            frame_perf: &mut RenderPerfStats,
            bytes: u64,
            upload: impl FnOnce(&mut GeometryUploadPerfSnapshot),
        ) {
            if perf_enabled {
                frame_perf.instance_bytes = frame_perf.instance_bytes.saturating_add(bytes);
                upload(&mut frame_perf.geometry_upload);
            }
        }

        fn record_vertex_upload(
            perf_enabled: bool,
            frame_perf: &mut RenderPerfStats,
            bytes: u64,
            upload: impl FnOnce(&mut GeometryUploadPerfSnapshot),
        ) {
            if perf_enabled {
                frame_perf.vertex_bytes = frame_perf.vertex_bytes.saturating_add(bytes);
                upload(&mut frame_perf.geometry_upload);
            }
        }

        let invalidations = ResidentGeometryUploadInvalidations {
            quad_instances: self.quad_instances.ensure_capacity(device, instances.len()),
            path_paints: self.path_paints.ensure_capacity(device, path_paints.len()),
            text_paints: self.text_paints.ensure_capacity(device, text_paints.len()),
            viewport_vertices: self
                .viewport_vertices
                .ensure_capacity(device, viewport_vertices.len()),
            text_glyph_instances: self
                .text_glyph_instances
                .ensure_capacity(device, text_glyph_instances.len()),
            text_vertices: self
                .text_vertices
                .ensure_capacity(device, text_vertices.len()),
            path_vertices: self
                .path_vertices
                .ensure_capacity(device, path_vertices.len()),
        };

        if perf_enabled {
            let slots = ResidentGeometryUploadSlots {
                quad_instances: self.quad_instances.current_slot(),
                path_paints: self.path_paints.current_slot(),
                text_paints: self.text_paints.current_slot(),
                viewport_vertices: self.viewport_vertices.current_slot(),
                text_glyph_instances: self.text_glyph_instances.current_slot(),
                text_vertices: self.text_vertices.current_slot(),
                path_vertices: self.path_vertices.current_slot(),
            };
            self.resident_uploads
                .record_frame(plan, frame_perf, slots, invalidations);
        }

        let (instance_buffer, quad_instance_bind_group) = self.quad_instances.next_pair();
        if !instances.is_empty() {
            queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(instances));
            let bytes = std::mem::size_of_val(instances) as u64;
            record_instance_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.quad_instance_bytes = upload.quad_instance_bytes.saturating_add(bytes);
                upload.quad_instance_write_count =
                    upload.quad_instance_write_count.saturating_add(1);
            });
        }

        let (path_paint_buffer, path_paint_bind_group) = self.path_paints.next_pair();
        if !path_paints.is_empty() {
            queue.write_buffer(&path_paint_buffer, 0, bytemuck::cast_slice(path_paints));
            let bytes = std::mem::size_of_val(path_paints) as u64;
            record_instance_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.path_paint_bytes = upload.path_paint_bytes.saturating_add(bytes);
                upload.path_paint_write_count = upload.path_paint_write_count.saturating_add(1);
            });
        }

        let (text_paint_buffer, text_paint_bind_group) = self.text_paints.next_pair();
        if !text_paints.is_empty() {
            queue.write_buffer(&text_paint_buffer, 0, bytemuck::cast_slice(text_paints));
            let bytes = std::mem::size_of_val(text_paints) as u64;
            record_instance_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.text_paint_bytes = upload.text_paint_bytes.saturating_add(bytes);
                upload.text_paint_write_count = upload.text_paint_write_count.saturating_add(1);
            });
        }

        let viewport_vertex_buffer = self.viewport_vertices.next_buffer();
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                &viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(viewport_vertices),
            );
            let bytes = std::mem::size_of_val(viewport_vertices) as u64;
            record_vertex_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.viewport_vertex_bytes = upload.viewport_vertex_bytes.saturating_add(bytes);
                upload.viewport_vertex_write_count =
                    upload.viewport_vertex_write_count.saturating_add(1);
            });
        }

        let text_glyph_instance_buffer = self.text_glyph_instances.next_buffer();
        if !text_glyph_instances.is_empty() {
            queue.write_buffer(
                &text_glyph_instance_buffer,
                0,
                bytemuck::cast_slice(text_glyph_instances),
            );
            let bytes = std::mem::size_of_val(text_glyph_instances) as u64;
            record_instance_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.text_glyph_instance_bytes =
                    upload.text_glyph_instance_bytes.saturating_add(bytes);
                upload.text_glyph_instance_write_count =
                    upload.text_glyph_instance_write_count.saturating_add(1);
            });
        }

        let text_vertex_buffer = self.text_vertices.next_buffer();
        if !text_vertices.is_empty() {
            queue.write_buffer(&text_vertex_buffer, 0, bytemuck::cast_slice(text_vertices));
            let bytes = std::mem::size_of_val(text_vertices) as u64;
            record_vertex_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.text_vertex_bytes = upload.text_vertex_bytes.saturating_add(bytes);
                upload.text_vertex_write_count = upload.text_vertex_write_count.saturating_add(1);
            });
        }

        let path_vertex_buffer = self.path_vertices.next_buffer();
        if !path_vertices.is_empty() {
            queue.write_buffer(&path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
            let bytes = std::mem::size_of_val(path_vertices) as u64;
            record_vertex_upload(perf_enabled, frame_perf, bytes, |upload| {
                upload.path_vertex_bytes = upload.path_vertex_bytes.saturating_add(bytes);
                upload.path_vertex_write_count = upload.path_vertex_write_count.saturating_add(1);
            });
        }

        FrameGeometryUploads {
            quad_instance_bind_group,
            text_paint_bind_group,
            path_paint_bind_group,
            viewport_vertex_buffer,
            text_glyph_instance_buffer,
            text_vertex_buffer,
            path_vertex_buffer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(slot: usize) -> ResidentGeometryUploadSlots {
        ResidentGeometryUploadSlots {
            quad_instances: slot,
            path_paints: slot,
            text_paints: slot,
            viewport_vertices: slot,
            text_glyph_instances: slot,
            text_vertices: slot,
            path_vertices: slot,
        }
    }

    fn plan_with_quad_range(start: u32, end: u32, fingerprint: u64) -> RenderPlan {
        RenderPlan {
            segments: vec![RenderPlanSegment {
                id: SceneSegmentId(0),
                draw_range: 0..1,
                start_uniform_index: None,
                start_uniform_fingerprint: 0,
                flags: RenderPlanSegmentFlags {
                    has_quad: true,
                    ..Default::default()
                },
                scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                    eligible: true,
                    draw_count: 1,
                    fingerprint,
                },
                stream_ranges: RenderPlanSegmentStreamRanges {
                    quad_instances: RenderPlanStreamRange::new(start, end),
                    ..Default::default()
                },
            }],
            passes: Vec::new(),
            compile_stats: RenderPlanCompileStats::default(),
            degradations: Vec::new(),
        }
    }

    fn frame_perf_with_safe_candidate() -> RenderPerfStats {
        RenderPerfStats {
            scene_chunk_encoding_payload_reassembly_append_only_matches: 1,
            ..Default::default()
        }
    }

    #[test]
    fn resident_upload_diagnostics_are_ring_slot_scoped() {
        let plan = plan_with_quad_range(0, 2, 7);
        let mut state = ResidentGeometryUploadState::default();

        let mut first = frame_perf_with_safe_candidate();
        state.record_frame(
            &plan,
            &mut first,
            slots(0),
            ResidentGeometryUploadInvalidations::default(),
        );
        assert_eq!(first.geometry_upload.resident_stream_candidates, 1);
        assert_eq!(first.geometry_upload.resident_stream_hits, 0);
        assert_eq!(first.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            first.geometry_upload.resident_dirty_range_bytes_estimate,
            2 * std::mem::size_of::<QuadInstance>() as u64
        );
        assert_eq!(
            first
                .geometry_upload
                .resident_full_upload_fallbacks_uninitialized,
            1
        );

        let mut second_slot = frame_perf_with_safe_candidate();
        state.record_frame(
            &plan,
            &mut second_slot,
            slots(1),
            ResidentGeometryUploadInvalidations::default(),
        );
        assert_eq!(second_slot.geometry_upload.resident_stream_hits, 0);
        assert_eq!(second_slot.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            second_slot
                .geometry_upload
                .resident_full_upload_fallbacks_uninitialized,
            1
        );

        let mut reused_slot = frame_perf_with_safe_candidate();
        state.record_frame(
            &plan,
            &mut reused_slot,
            slots(0),
            ResidentGeometryUploadInvalidations::default(),
        );
        assert_eq!(reused_slot.geometry_upload.resident_stream_candidates, 1);
        assert_eq!(reused_slot.geometry_upload.resident_stream_hits, 1);
        assert_eq!(reused_slot.geometry_upload.resident_stream_misses, 0);
        assert_eq!(
            reused_slot
                .geometry_upload
                .resident_dirty_range_bytes_estimate,
            0
        );
        assert_eq!(
            reused_slot.geometry_upload.resident_full_upload_fallbacks,
            0
        );
    }

    #[test]
    fn resident_upload_diagnostics_report_fallback_reasons() {
        let plan = plan_with_quad_range(0, 2, 7);
        let mut state = ResidentGeometryUploadState::default();
        let mut frame_perf = RenderPerfStats {
            scene_chunk_encoding_payload_plan_candidates_without_payload: 2,
            scene_chunk_encoding_payload_reassembly_blocked_by_stream_fingerprint_mismatch: 1,
            ..Default::default()
        };

        state.record_frame(
            &plan,
            &mut frame_perf,
            slots(0),
            ResidentGeometryUploadInvalidations::default(),
        );

        assert_eq!(frame_perf.geometry_upload.resident_stream_candidates, 0);
        assert_eq!(frame_perf.geometry_upload.resident_full_upload_fallbacks, 3);
        assert_eq!(
            frame_perf
                .geometry_upload
                .resident_full_upload_fallbacks_missing_payload,
            2
        );
        assert_eq!(
            frame_perf
                .geometry_upload
                .resident_full_upload_fallbacks_reassembly_blocked,
            1
        );
    }

    #[test]
    fn resident_upload_diagnostics_report_layout_change_and_buffer_resize() {
        let first_plan = plan_with_quad_range(0, 2, 7);
        let changed_plan = plan_with_quad_range(1, 3, 8);
        let mut state = ResidentGeometryUploadState::default();

        let mut warmup = frame_perf_with_safe_candidate();
        state.record_frame(
            &first_plan,
            &mut warmup,
            slots(0),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = frame_perf_with_safe_candidate();
        state.record_frame(
            &changed_plan,
            &mut changed,
            slots(0),
            ResidentGeometryUploadInvalidations::default(),
        );
        assert_eq!(changed.geometry_upload.resident_stream_hits, 0);
        assert_eq!(changed.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            changed
                .geometry_upload
                .resident_full_upload_fallbacks_stream_layout_changed,
            1
        );

        let mut resized = frame_perf_with_safe_candidate();
        state.record_frame(
            &changed_plan,
            &mut resized,
            slots(0),
            ResidentGeometryUploadInvalidations {
                quad_instances: true,
                ..Default::default()
            },
        );
        assert_eq!(resized.geometry_upload.resident_stream_hits, 0);
        assert_eq!(resized.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            resized
                .geometry_upload
                .resident_full_upload_fallbacks_buffer_resized,
            1
        );
    }
}
