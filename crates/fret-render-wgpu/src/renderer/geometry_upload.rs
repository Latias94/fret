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

#[derive(Clone, Copy)]
struct ResidentGeometryUploadStreams<'a> {
    quad_instances: &'a [QuadInstance],
    path_paints: &'a [PaintGpu],
    text_paints: &'a [PaintGpu],
    viewport_vertices: &'a [ViewportVertex],
    text_glyph_instances: &'a [TextGlyphInstance],
    text_vertices: &'a [TextVertex],
    path_vertices: &'a [PathVertex],
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
    slots: Vec<Option<Vec<ResidentGeometryUploadStreamSignature>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResidentGeometryUploadStreamSignature {
    layout_fingerprint: u64,
    content_fingerprint: u64,
    range: RenderPlanStreamRange,
}

#[derive(Default)]
struct ResidentGeometryUploadFrameSignatures {
    quad_instances: Vec<ResidentGeometryUploadStreamSignature>,
    path_paints: Vec<ResidentGeometryUploadStreamSignature>,
    text_paints: Vec<ResidentGeometryUploadStreamSignature>,
    viewport_vertices: Vec<ResidentGeometryUploadStreamSignature>,
    text_glyph_instances: Vec<ResidentGeometryUploadStreamSignature>,
    text_vertices: Vec<ResidentGeometryUploadStreamSignature>,
    path_vertices: Vec<ResidentGeometryUploadStreamSignature>,
}

#[derive(Default)]
struct ResidentGeometryUploadStreamAccumulator {
    ranges: Vec<ResidentGeometryUploadStreamRangeSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResidentGeometryUploadStreamRangeSignature {
    layout_fingerprint: u64,
    range: RenderPlanStreamRange,
}

impl ResidentGeometryUploadState {
    fn record_frame(
        &mut self,
        plan: &RenderPlan,
        payload_alignment: &SceneChunkPayloadPlanAlignment,
        frame_perf: Option<&mut RenderPerfStats>,
        slots: ResidentGeometryUploadSlots,
        streams: ResidentGeometryUploadStreams<'_>,
        invalidations: ResidentGeometryUploadInvalidations,
    ) {
        let mut upload = frame_perf.map(|frame_perf| &mut frame_perf.geometry_upload);
        let eligible_candidates = plan
            .segments
            .iter()
            .filter(|segment| segment.scene_chunk_candidate.eligible)
            .count();
        if eligible_candidates == 0 {
            if let Some(upload) = upload.as_deref_mut() {
                upload.record_resident_full_upload_fallback_no_candidate();
            }
            return;
        }

        let alignment_stats = payload_alignment.stats;
        let missing_payloads = alignment_stats.payload_plan_candidates_without_payload;
        if missing_payloads > 0 {
            if let Some(upload) = upload.as_deref_mut() {
                upload.record_resident_full_upload_fallback_missing_payload(missing_payloads);
            }
        }

        let blocked_reassembly = alignment_stats
            .payload_reassembly_blocked_by_shape_mismatch
            .saturating_add(
                alignment_stats.payload_reassembly_blocked_by_stream_fingerprint_mismatch,
            )
            .saturating_add(alignment_stats.payload_reassembly_blocked_by_non_quad_draws)
            .saturating_add(alignment_stats.payload_reassembly_blocked_by_side_tables)
            .saturating_add(alignment_stats.payload_reassembly_blocked_by_material_state);
        if blocked_reassembly > 0 {
            if let Some(upload) = upload.as_deref_mut() {
                upload.record_resident_full_upload_fallback_reassembly_blocked(blocked_reassembly);
            }
        }

        if payload_alignment.reassembly_plan.is_empty() {
            return;
        }

        let signatures = ResidentGeometryUploadFrameSignatures::from_plan(
            plan,
            &payload_alignment.reassembly_plan,
            streams,
        );
        self.quad_instances.record_frame(
            slots.quad_instances,
            invalidations.quad_instances,
            signatures.quad_instances,
            streams.quad_instances.len(),
            estimate_range_bytes::<QuadInstance>,
            upload.as_deref_mut(),
        );
        self.path_paints.record_frame(
            slots.path_paints,
            invalidations.path_paints,
            signatures.path_paints,
            streams.path_paints.len(),
            estimate_range_bytes::<PaintGpu>,
            upload.as_deref_mut(),
        );
        self.text_paints.record_frame(
            slots.text_paints,
            invalidations.text_paints,
            signatures.text_paints,
            streams.text_paints.len(),
            estimate_range_bytes::<PaintGpu>,
            upload.as_deref_mut(),
        );
        self.viewport_vertices.record_frame(
            slots.viewport_vertices,
            invalidations.viewport_vertices,
            signatures.viewport_vertices,
            streams.viewport_vertices.len(),
            estimate_range_bytes::<ViewportVertex>,
            upload.as_deref_mut(),
        );
        self.text_glyph_instances.record_frame(
            slots.text_glyph_instances,
            invalidations.text_glyph_instances,
            signatures.text_glyph_instances,
            streams.text_glyph_instances.len(),
            estimate_range_bytes::<TextGlyphInstance>,
            upload.as_deref_mut(),
        );
        self.text_vertices.record_frame(
            slots.text_vertices,
            invalidations.text_vertices,
            signatures.text_vertices,
            streams.text_vertices.len(),
            estimate_range_bytes::<TextVertex>,
            upload.as_deref_mut(),
        );
        self.path_vertices.record_frame(
            slots.path_vertices,
            invalidations.path_vertices,
            signatures.path_vertices,
            streams.path_vertices.len(),
            estimate_range_bytes::<PathVertex>,
            upload.as_deref_mut(),
        );
    }
}

impl ResidentGeometryUploadFrameSignatures {
    fn from_plan(
        plan: &RenderPlan,
        reassembly_plan: &SceneChunkPayloadReassemblyPlan,
        streams: ResidentGeometryUploadStreams<'_>,
    ) -> Self {
        let mut quad_instances = ResidentGeometryUploadStreamAccumulator::default();
        let mut path_paints = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_paints = ResidentGeometryUploadStreamAccumulator::default();
        let mut viewport_vertices = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_glyph_instances = ResidentGeometryUploadStreamAccumulator::default();
        let mut text_vertices = ResidentGeometryUploadStreamAccumulator::default();
        let mut path_vertices = ResidentGeometryUploadStreamAccumulator::default();

        for segment_index in reassembly_plan.safe_segment_indices() {
            let Some(segment) = plan.segments.get(*segment_index) else {
                continue;
            };
            if !segment.scene_chunk_candidate.eligible {
                continue;
            }
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
            quad_instances: quad_instances.finish(streams.quad_instances),
            path_paints: path_paints.finish(streams.path_paints),
            text_paints: text_paints.finish(streams.text_paints),
            viewport_vertices: viewport_vertices.finish(streams.viewport_vertices),
            text_glyph_instances: text_glyph_instances.finish(streams.text_glyph_instances),
            text_vertices: text_vertices.finish(streams.text_vertices),
            path_vertices: path_vertices.finish(streams.path_vertices),
        }
    }
}

impl ResidentGeometryUploadStreamAccumulator {
    fn include(&mut self, candidate_fingerprint: u64, range: RenderPlanStreamRange) {
        if range.is_empty() {
            return;
        }

        let mut hasher = DefaultHasher::new();
        candidate_fingerprint.hash(&mut hasher);
        range.start.hash(&mut hasher);
        range.end.hash(&mut hasher);
        self.ranges
            .push(ResidentGeometryUploadStreamRangeSignature {
                layout_fingerprint: hasher.finish(),
                range,
            });
    }

    fn finish<T: bytemuck::Pod>(self, values: &[T]) -> Vec<ResidentGeometryUploadStreamSignature> {
        self.ranges
            .into_iter()
            .map(|entry| ResidentGeometryUploadStreamSignature {
                layout_fingerprint: entry.layout_fingerprint,
                content_fingerprint: hash_pod_range(values, entry.range),
                range: entry.range,
            })
            .collect()
    }
}

impl ResidentGeometryUploadStreamState {
    fn record_frame(
        &mut self,
        slot: usize,
        invalidated: bool,
        signatures: Vec<ResidentGeometryUploadStreamSignature>,
        stream_len: usize,
        estimate_bytes: fn(RenderPlanStreamRange) -> u64,
        upload: Option<&mut GeometryUploadPerfSnapshot>,
    ) {
        if signatures.is_empty() {
            return;
        }

        let mut upload = upload;
        if let Some(upload) = upload.as_deref_mut() {
            upload.resident_stream_candidates = upload.resident_stream_candidates.saturating_add(1);
        }
        if invalidated {
            self.slots.clear();
        }
        if self.slots.len() <= slot {
            self.slots.resize(slot + 1, None);
        }

        let covers_entire_stream = resident_stream_signatures_cover_stream(&signatures, stream_len);
        if !covers_entire_stream {
            if let Some(upload) = upload.as_deref_mut() {
                upload.resident_stream_coverage_gaps =
                    upload.resident_stream_coverage_gaps.saturating_add(1);
            }
        }

        match self.slots[slot].as_ref() {
            Some(previous) if previous == &signatures => {
                if let Some(upload) = upload.as_deref_mut() {
                    upload.resident_stream_hits = upload.resident_stream_hits.saturating_add(1);
                }
            }
            Some(previous) if resident_stream_layout_matches(previous, &signatures) => {
                let dirty_bytes =
                    estimate_changed_signature_bytes(previous, &signatures, estimate_bytes);
                if let Some(upload) = upload.as_deref_mut() {
                    upload.record_resident_stream_miss(dirty_bytes);
                    upload.resident_stream_content_mismatches =
                        upload.resident_stream_content_mismatches.saturating_add(1);
                    if covers_entire_stream {
                        let changed_ranges = count_changed_signatures(previous, &signatures) as u64;
                        upload.record_resident_partial_write_dry_run(changed_ranges, dirty_bytes);
                    }
                    upload.resident_full_upload_fallbacks_stream_content_changed = upload
                        .resident_full_upload_fallbacks_stream_content_changed
                        .saturating_add(1);
                }
                self.slots[slot] = Some(signatures);
            }
            Some(_) => {
                if let Some(upload) = upload.as_deref_mut() {
                    upload.record_resident_stream_miss(estimate_signature_bytes(
                        &signatures,
                        estimate_bytes,
                    ));
                    upload.resident_full_upload_fallbacks_stream_layout_changed = upload
                        .resident_full_upload_fallbacks_stream_layout_changed
                        .saturating_add(1);
                }
                self.slots[slot] = Some(signatures);
            }
            None => {
                if let Some(upload) = upload.as_deref_mut() {
                    upload.record_resident_stream_miss(estimate_signature_bytes(
                        &signatures,
                        estimate_bytes,
                    ));
                    if invalidated {
                        upload.resident_full_upload_fallbacks_buffer_resized = upload
                            .resident_full_upload_fallbacks_buffer_resized
                            .saturating_add(1);
                    } else {
                        upload.resident_full_upload_fallbacks_uninitialized = upload
                            .resident_full_upload_fallbacks_uninitialized
                            .saturating_add(1);
                    }
                }
                self.slots[slot] = Some(signatures);
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

    fn record_resident_partial_write_dry_run(&mut self, write_count: u64, dirty_bytes: u64) {
        self.resident_partial_write_dry_run_streams = self
            .resident_partial_write_dry_run_streams
            .saturating_add(1);
        self.resident_partial_write_dry_run_write_count_estimate = self
            .resident_partial_write_dry_run_write_count_estimate
            .saturating_add(write_count);
        self.resident_partial_write_dry_run_bytes_estimate = self
            .resident_partial_write_dry_run_bytes_estimate
            .saturating_add(dirty_bytes);
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

fn resident_stream_layout_matches(
    previous: &[ResidentGeometryUploadStreamSignature],
    current: &[ResidentGeometryUploadStreamSignature],
) -> bool {
    previous.len() == current.len()
        && previous
            .iter()
            .zip(current.iter())
            .all(|(previous, current)| {
                previous.layout_fingerprint == current.layout_fingerprint
                    && previous.range == current.range
            })
}

fn count_changed_signatures(
    previous: &[ResidentGeometryUploadStreamSignature],
    current: &[ResidentGeometryUploadStreamSignature],
) -> usize {
    previous
        .iter()
        .zip(current.iter())
        .filter(|(previous, current)| previous.content_fingerprint != current.content_fingerprint)
        .count()
}

fn estimate_changed_signature_bytes(
    previous: &[ResidentGeometryUploadStreamSignature],
    current: &[ResidentGeometryUploadStreamSignature],
    estimate_bytes: fn(RenderPlanStreamRange) -> u64,
) -> u64 {
    previous
        .iter()
        .zip(current.iter())
        .filter(|(previous, current)| previous.content_fingerprint != current.content_fingerprint)
        .map(|(_, current)| estimate_bytes(current.range))
        .fold(0u64, u64::saturating_add)
}

fn estimate_signature_bytes(
    signatures: &[ResidentGeometryUploadStreamSignature],
    estimate_bytes: fn(RenderPlanStreamRange) -> u64,
) -> u64 {
    signatures
        .iter()
        .map(|signature| estimate_bytes(signature.range))
        .fold(0u64, u64::saturating_add)
}

fn resident_stream_signatures_cover_stream(
    signatures: &[ResidentGeometryUploadStreamSignature],
    stream_len: usize,
) -> bool {
    if signatures.is_empty() || stream_len == 0 {
        return false;
    }

    let Ok(stream_len) = u32::try_from(stream_len) else {
        return false;
    };

    let mut ranges: Vec<RenderPlanStreamRange> =
        signatures.iter().map(|signature| signature.range).collect();
    ranges.sort_by_key(|range| (range.start, range.end));

    let mut covered_end = 0u32;
    for range in ranges {
        if range.start > covered_end || range.end < range.start || range.end > stream_len {
            return false;
        }
        covered_end = covered_end.max(range.end);
    }

    covered_end == stream_len
}

fn hash_pod_range<T: bytemuck::Pod>(values: &[T], range: RenderPlanStreamRange) -> u64 {
    let start = usize::try_from(range.start).unwrap_or(usize::MAX);
    let end = usize::try_from(range.end).unwrap_or(usize::MAX);
    let mut hasher = DefaultHasher::new();
    range.start.hash(&mut hasher);
    range.end.hash(&mut hasher);
    let Some(values) = values.get(start..end) else {
        0u64.hash(&mut hasher);
        return hasher.finish();
    };
    let bytes = bytemuck::cast_slice::<T, u8>(values);
    bytes.len().hash(&mut hasher);
    bytes.hash(&mut hasher);
    hasher.finish()
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
        payload_alignment: &SceneChunkPayloadPlanAlignment,
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

        let slots = ResidentGeometryUploadSlots {
            quad_instances: self.quad_instances.current_slot(),
            path_paints: self.path_paints.current_slot(),
            text_paints: self.text_paints.current_slot(),
            viewport_vertices: self.viewport_vertices.current_slot(),
            text_glyph_instances: self.text_glyph_instances.current_slot(),
            text_vertices: self.text_vertices.current_slot(),
            path_vertices: self.path_vertices.current_slot(),
        };
        let streams = ResidentGeometryUploadStreams {
            quad_instances: instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_glyph_instances,
            text_vertices,
            path_vertices,
        };
        self.resident_uploads.record_frame(
            plan,
            payload_alignment,
            if perf_enabled {
                Some(&mut *frame_perf)
            } else {
                None
            },
            slots,
            streams,
            invalidations,
        );

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

    fn streams_with_quad_instances(
        quad_instances: &[QuadInstance],
    ) -> ResidentGeometryUploadStreams<'_> {
        ResidentGeometryUploadStreams {
            quad_instances,
            path_paints: &[],
            text_paints: &[],
            viewport_vertices: &[],
            text_glyph_instances: &[],
            text_vertices: &[],
            path_vertices: &[],
        }
    }

    fn quad_instances(count: usize) -> Vec<QuadInstance> {
        vec![bytemuck::Zeroable::zeroed(); count]
    }

    fn safe_plan(indices: &[usize]) -> SceneChunkPayloadReassemblyPlan {
        SceneChunkPayloadReassemblyPlan::from_safe_segment_indices(indices.to_vec())
    }

    fn payload_alignment(indices: &[usize]) -> SceneChunkPayloadPlanAlignment {
        payload_alignment_with_stats(indices, SceneChunkEncodingFrameStats::default())
    }

    fn payload_alignment_with_stats(
        indices: &[usize],
        stats: SceneChunkEncodingFrameStats,
    ) -> SceneChunkPayloadPlanAlignment {
        SceneChunkPayloadPlanAlignment {
            stats,
            reassembly_plan: safe_plan(indices),
        }
    }

    fn resident_signature(start: u32, end: u32) -> ResidentGeometryUploadStreamSignature {
        ResidentGeometryUploadStreamSignature {
            layout_fingerprint: u64::from(start) << 32 | u64::from(end),
            content_fingerprint: u64::from(end) << 32 | u64::from(start),
            range: RenderPlanStreamRange::new(start, end),
        }
    }

    fn plan_with_quad_range(start: u32, end: u32, fingerprint: u64) -> RenderPlan {
        plan_with_quad_ranges(&[(start, end, fingerprint)])
    }

    fn plan_with_quad_ranges(ranges: &[(u32, u32, u64)]) -> RenderPlan {
        RenderPlan {
            segments: ranges
                .iter()
                .enumerate()
                .map(|(index, (start, end, fingerprint))| RenderPlanSegment {
                    id: SceneSegmentId(index),
                    draw_range: index..index + 1,
                    start_uniform_index: None,
                    start_uniform_fingerprint: 0,
                    flags: RenderPlanSegmentFlags {
                        has_quad: true,
                        ..Default::default()
                    },
                    scene_chunk_candidate: RenderPlanSceneChunkCandidate {
                        eligible: true,
                        draw_count: 1,
                        fingerprint: *fingerprint,
                    },
                    stream_ranges: RenderPlanSegmentStreamRanges {
                        quad_instances: RenderPlanStreamRange::new(*start, *end),
                        ..Default::default()
                    },
                })
                .collect(),
            passes: Vec::new(),
            compile_stats: RenderPlanCompileStats::default(),
            degradations: Vec::new(),
        }
    }

    #[test]
    fn resident_stream_coverage_requires_complete_stream_ranges() {
        assert!(resident_stream_signatures_cover_stream(
            &[resident_signature(0, 1), resident_signature(1, 3)],
            3
        ));
        assert!(resident_stream_signatures_cover_stream(
            &[resident_signature(0, 2), resident_signature(1, 3)],
            3
        ));
        assert!(!resident_stream_signatures_cover_stream(
            &[resident_signature(1, 3)],
            3
        ));
        assert!(!resident_stream_signatures_cover_stream(
            &[resident_signature(0, 1), resident_signature(2, 3)],
            3
        ));
        assert!(!resident_stream_signatures_cover_stream(
            &[resident_signature(0, 4)],
            3
        ));
        assert!(!resident_stream_signatures_cover_stream(&[], 3));
    }

    #[test]
    fn resident_upload_diagnostics_are_ring_slot_scoped() {
        let plan = plan_with_quad_range(0, 2, 7);
        let quad_instances = quad_instances(2);
        let mut state = ResidentGeometryUploadState::default();

        let alignment = payload_alignment(&[0]);
        let mut first = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut first),
            slots(0),
            streams_with_quad_instances(&quad_instances),
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

        let mut second_slot = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut second_slot),
            slots(1),
            streams_with_quad_instances(&quad_instances),
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

        let mut reused_slot = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut reused_slot),
            slots(0),
            streams_with_quad_instances(&quad_instances),
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
    fn resident_upload_state_warms_without_perf_recorder() {
        let plan = plan_with_quad_range(0, 2, 7);
        let quad_instances = quad_instances(2);
        let alignment = payload_alignment(&[0]);
        let mut state = ResidentGeometryUploadState::default();

        state.record_frame(
            &plan,
            &alignment,
            None,
            slots(0),
            streams_with_quad_instances(&quad_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut observed = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut observed),
            slots(0),
            streams_with_quad_instances(&quad_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        assert_eq!(observed.geometry_upload.resident_stream_candidates, 1);
        assert_eq!(observed.geometry_upload.resident_stream_hits, 1);
        assert_eq!(observed.geometry_upload.resident_stream_misses, 0);
    }

    #[test]
    fn resident_upload_diagnostics_report_fallback_reasons() {
        let plan = plan_with_quad_range(0, 2, 7);
        let quad_instances = quad_instances(2);
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment_with_stats(
            &[],
            SceneChunkEncodingFrameStats {
                payload_plan_candidates_without_payload: 2,
                payload_reassembly_blocked_by_stream_fingerprint_mismatch: 1,
                ..Default::default()
            },
        );
        let mut frame_perf = RenderPerfStats::default();

        state.record_frame(
            &plan,
            &alignment,
            Some(&mut frame_perf),
            slots(0),
            streams_with_quad_instances(&quad_instances),
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
        let quad_instances = quad_instances(3);
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment(&[0]);

        let mut warmup = RenderPerfStats::default();
        state.record_frame(
            &first_plan,
            &alignment,
            Some(&mut warmup),
            slots(0),
            streams_with_quad_instances(&quad_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = RenderPerfStats::default();
        state.record_frame(
            &changed_plan,
            &alignment,
            Some(&mut changed),
            slots(0),
            streams_with_quad_instances(&quad_instances),
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

        let mut resized = RenderPerfStats::default();
        state.record_frame(
            &changed_plan,
            &alignment,
            Some(&mut resized),
            slots(0),
            streams_with_quad_instances(&quad_instances),
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

    #[test]
    fn resident_upload_diagnostics_report_content_change_partial_write_dry_run() {
        let plan = plan_with_quad_range(0, 2, 7);
        let first_instances = quad_instances(2);
        let mut changed_instances = quad_instances(2);
        changed_instances[0].rect[0] = 1.0;
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment(&[0]);

        let mut warmup = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut warmup),
            slots(0),
            streams_with_quad_instances(&first_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut changed),
            slots(0),
            streams_with_quad_instances(&changed_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let dirty_bytes = 2 * std::mem::size_of::<QuadInstance>() as u64;
        assert_eq!(changed.geometry_upload.resident_stream_hits, 0);
        assert_eq!(changed.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            changed.geometry_upload.resident_stream_content_mismatches,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_streams,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_write_count_estimate,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_bytes_estimate,
            dirty_bytes
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_full_upload_fallbacks_stream_content_changed,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_full_upload_fallbacks_stream_layout_changed,
            0
        );
    }

    #[test]
    fn resident_upload_diagnostics_use_exact_safe_segment_indices() {
        let plan = plan_with_quad_ranges(&[(0, 1, 7), (1, 2, 8)]);
        let first_instances = quad_instances(2);
        let mut changed_instances = quad_instances(2);
        changed_instances[0].rect[0] = 1.0;
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment(&[1]);

        let mut warmup = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut warmup),
            slots(0),
            streams_with_quad_instances(&first_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut changed),
            slots(0),
            streams_with_quad_instances(&changed_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        assert_eq!(changed.geometry_upload.resident_stream_candidates, 1);
        assert_eq!(changed.geometry_upload.resident_stream_hits, 1);
        assert_eq!(changed.geometry_upload.resident_stream_misses, 0);
        assert_eq!(changed.geometry_upload.resident_stream_coverage_gaps, 1);
        assert_eq!(
            changed.geometry_upload.resident_stream_content_mismatches,
            0
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_write_count_estimate,
            0
        );
    }

    #[test]
    fn resident_partial_write_dry_run_blocks_incomplete_stream_coverage() {
        let plan = plan_with_quad_ranges(&[(0, 1, 7), (1, 2, 8)]);
        let first_instances = quad_instances(2);
        let mut changed_instances = quad_instances(2);
        changed_instances[1].rect[0] = 1.0;
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment(&[1]);

        let mut warmup = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut warmup),
            slots(0),
            streams_with_quad_instances(&first_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut changed),
            slots(0),
            streams_with_quad_instances(&changed_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        assert_eq!(changed.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            changed.geometry_upload.resident_stream_content_mismatches,
            1
        );
        assert_eq!(changed.geometry_upload.resident_stream_coverage_gaps, 1);
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_streams,
            0
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_write_count_estimate,
            0
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_bytes_estimate,
            0
        );
    }

    #[test]
    fn resident_partial_write_dry_run_counts_changed_ranges() {
        let plan = plan_with_quad_ranges(&[(0, 1, 7), (1, 2, 8)]);
        let first_instances = quad_instances(2);
        let mut changed_instances = quad_instances(2);
        changed_instances[1].rect[0] = 1.0;
        let mut state = ResidentGeometryUploadState::default();
        let alignment = payload_alignment(&[0, 1]);

        let mut warmup = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut warmup),
            slots(0),
            streams_with_quad_instances(&first_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        let mut changed = RenderPerfStats::default();
        state.record_frame(
            &plan,
            &alignment,
            Some(&mut changed),
            slots(0),
            streams_with_quad_instances(&changed_instances),
            ResidentGeometryUploadInvalidations::default(),
        );

        assert_eq!(changed.geometry_upload.resident_stream_misses, 1);
        assert_eq!(
            changed.geometry_upload.resident_stream_content_mismatches,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_write_count_estimate,
            1
        );
        assert_eq!(
            changed
                .geometry_upload
                .resident_partial_write_dry_run_bytes_estimate,
            std::mem::size_of::<QuadInstance>() as u64
        );
    }
}
