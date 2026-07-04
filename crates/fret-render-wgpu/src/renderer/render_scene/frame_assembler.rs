use super::super::*;

#[derive(Default)]
pub(in crate::renderer) struct FrameAssembler {
    scene_chunk_encoding_state: SceneChunkEncodingState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::renderer) enum FrameAssemblyUnsupportedReason {
    UnsupportedLaunch {
        stream_class: Option<ChunkLaunchStreamClass>,
        reason: ChunkLaunchUnsupportedReason,
    },
    RequestedStreamClassMismatch {
        requested: ChunkLaunchStreamClass,
        supported: ChunkLaunchStreamClass,
    },
    EntryStreamClassMismatch {
        entry_index: u32,
        requested: ChunkLaunchStreamClass,
        actual: ChunkLaunchStreamClass,
    },
    MissingPayload {
        entry_index: u32,
    },
    PayloadReassemblyBlocked {
        entry_index: u32,
        blocker: SceneChunkPayloadReassemblyBlocker,
    },
    PayloadStreamMismatch {
        entry_index: u32,
        stream_class: ChunkLaunchStreamClass,
    },
}

impl FrameAssembler {
    pub(in crate::renderer) fn evaluate_support(
        manifest: &fret_core::SceneChunkManifest,
    ) -> ChunkLaunchSupport {
        let stream_class = manifest_stream_class(manifest);

        match stream_class {
            None => ChunkLaunchSupport::Unsupported {
                stream_class: None,
                reason: ChunkLaunchUnsupportedReason::EmptyManifest,
            },
            Some(stream_class) => {
                if let Some(reason) = manifest.assembly_unsupported_reasons().first().copied() {
                    return ChunkLaunchSupport::Unsupported {
                        stream_class: Some(stream_class),
                        reason: ChunkLaunchUnsupportedReason::ManifestUnsupported(reason),
                    };
                }

                match stream_class {
                    ChunkLaunchStreamClass::ResourceFreeQuad => ChunkLaunchSupport::Supported {
                        stream_class: ChunkLaunchStreamClass::ResourceFreeQuad,
                    },
                    ChunkLaunchStreamClass::ResourceFreeVertexColor => {
                        ChunkLaunchSupport::Supported {
                            stream_class: ChunkLaunchStreamClass::ResourceFreeVertexColor,
                        }
                    }
                    ChunkLaunchStreamClass::Mixed => ChunkLaunchSupport::Unsupported {
                        stream_class: Some(ChunkLaunchStreamClass::Mixed),
                        reason: ChunkLaunchUnsupportedReason::MixedStreams,
                    },
                }
            }
        }
    }

    pub(in crate::renderer) fn try_assemble_supported_frame_encoding(
        &self,
        manifest: &fret_core::SceneChunkManifest,
        context: SceneChunkEncodingContext,
        stream_class: ChunkLaunchStreamClass,
    ) -> Result<SceneEncoding, FrameAssemblyUnsupportedReason> {
        match Self::evaluate_support(manifest) {
            ChunkLaunchSupport::Supported {
                stream_class: supported,
            } if supported == stream_class => {}
            ChunkLaunchSupport::Supported {
                stream_class: supported,
            } => {
                return Err(
                    FrameAssemblyUnsupportedReason::RequestedStreamClassMismatch {
                        requested: stream_class,
                        supported,
                    },
                );
            }
            ChunkLaunchSupport::Unsupported {
                stream_class,
                reason,
            } => {
                return Err(FrameAssemblyUnsupportedReason::UnsupportedLaunch {
                    stream_class,
                    reason,
                });
            }
        }

        let mut encoding = SceneEncoding::default();
        for entry in manifest.entries() {
            let actual_stream_class = entry_stream_class(entry);
            if actual_stream_class != stream_class {
                return Err(FrameAssemblyUnsupportedReason::EntryStreamClassMismatch {
                    entry_index: entry.order_index(),
                    requested: stream_class,
                    actual: actual_stream_class,
                });
            }

            let Some(payload) = self
                .scene_chunk_encoding_state
                .payload_for_entry(context, entry, 0)
            else {
                return Err(FrameAssemblyUnsupportedReason::MissingPayload {
                    entry_index: entry.order_index(),
                });
            };

            if let Some(blocker) = payload.append_only_reassembly_blocker() {
                return Err(FrameAssemblyUnsupportedReason::PayloadReassemblyBlocked {
                    entry_index: entry.order_index(),
                    blocker,
                });
            }

            append_resource_free_payload_encoding(&mut encoding, payload.encoding(), stream_class)
                .ok_or(FrameAssemblyUnsupportedReason::PayloadStreamMismatch {
                    entry_index: entry.order_index(),
                    stream_class,
                })?;
        }

        Ok(encoding)
    }

    pub(in crate::renderer) fn assemble_supported_frame_encoding(
        &self,
        manifest: &fret_core::SceneChunkManifest,
        context: SceneChunkEncodingContext,
        stream_class: ChunkLaunchStreamClass,
    ) -> Option<SceneEncoding> {
        self.try_assemble_supported_frame_encoding(manifest, context, stream_class)
            .ok()
    }

    pub(in crate::renderer) fn begin_frame_with_payloads(
        &mut self,
        manifest: Option<&fret_core::SceneChunkManifest>,
        context: SceneChunkEncodingContext,
        entry_text_resource_keys: &[u64],
        build_payload: impl FnMut(&fret_core::SceneChunkManifestEntry) -> CachedSceneChunkEncoding,
    ) -> SceneChunkEncodingFrameStats {
        self.scene_chunk_encoding_state.begin_frame_with_payloads(
            manifest,
            context,
            entry_text_resource_keys,
            build_payload,
        )
    }

    pub(in crate::renderer) fn record_payload_plan_alignment(
        &self,
        plan: &RenderPlan,
        flat_encoding: &SceneEncoding,
    ) -> SceneChunkPayloadPlanAlignment {
        self.scene_chunk_encoding_state
            .record_payload_plan_alignment(plan, flat_encoding)
    }
}

fn manifest_stream_class(
    manifest: &fret_core::SceneChunkManifest,
) -> Option<ChunkLaunchStreamClass> {
    let mut stream_class = None;
    for entry in manifest.entries() {
        let entry_class = entry_stream_class(entry);
        stream_class = match (stream_class, entry_class) {
            (None, class) => Some(class),
            (Some(existing), class) if existing == class => Some(existing),
            _ => Some(ChunkLaunchStreamClass::Mixed),
        };
    }

    stream_class
}

fn entry_stream_class(entry: &fret_core::SceneChunkManifestEntry) -> ChunkLaunchStreamClass {
    let streams = entry.chunk().closure().draw_streams();
    if streams.is_quad_only() {
        ChunkLaunchStreamClass::ResourceFreeQuad
    } else if streams.is_vertex_color_only() {
        ChunkLaunchStreamClass::ResourceFreeVertexColor
    } else {
        ChunkLaunchStreamClass::Mixed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        Color, DrawOrder, SceneChunk, SceneChunkManifest, SceneChunkManifestEntry, SceneOp,
    };
    use std::sync::Arc;

    fn context() -> SceneChunkEncodingContext {
        SceneChunkEncodingContext {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            viewport_size: (320, 200),
            scale_factor_bits: 1.0f32.to_bits(),
            render_targets_generation: 0,
            images_generation: 0,
            text_quality_key: 0,
            materials_generation: 0,
            material_paint_budget_per_frame: 0,
            material_distinct_budget_per_frame: 0,
            custom_effects_generation: 0,
        }
    }

    fn manifest(entries: &[SceneChunkManifestEntry]) -> SceneChunkManifest {
        let mut manifest = SceneChunkManifest::default();
        for entry in entries {
            manifest.push(entry.clone());
        }
        manifest
    }

    fn entry_for_ops(ops: Arc<[SceneOp]>) -> SceneChunkManifestEntry {
        SceneChunkManifestEntry::new(
            SceneChunk::from_ops(ops),
            Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
            Point::default(),
        )
    }

    fn quad_scene_op() -> SceneOp {
        SceneOp::Quad {
            order: DrawOrder(0),
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
            background: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }
            .into(),
            border: fret_core::Edges::all(Px(0.0)),
            border_paint: Color::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::all(Px(0.0)),
        }
    }

    fn vertex_color_quad_scene_op() -> SceneOp {
        let white = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        SceneOp::VertexColorQuad {
            order: DrawOrder(0),
            points: [
                Point::new(Px(0.0), Px(0.0)),
                Point::new(Px(10.0), Px(0.0)),
                Point::new(Px(10.0), Px(10.0)),
                Point::new(Px(0.0), Px(10.0)),
            ],
            colors: [white; 4],
        }
    }

    fn quad_payload_encoding() -> SceneEncoding {
        let mut encoding = SceneEncoding::default();
        encoding.instances.push(bytemuck::Zeroable::zeroed());
        encoding.uniforms.push(bytemuck::Zeroable::zeroed());
        encoding.uniform_mask_images.push(None);
        encoding.ordered_draws.push(OrderedDraw::Quad(QuadDraw {
            scissor: ScissorRect::full(320, 200),
            uniform_index: 0,
            first_instance: 0,
            instance_count: 1,
            pipeline: QuadPipelineKey {
                fill_kind: 0,
                border_kind: 0,
                border_present: false,
                dash_enabled: false,
                fill_material_sampled: false,
                border_material_sampled: false,
                shadow_mode: false,
            },
        }));
        encoding
    }

    #[test]
    fn frame_assembler_evaluates_mixed_manifest_with_structured_reason() {
        let frame = manifest(&[
            entry_for_ops(Arc::from([quad_scene_op()])),
            entry_for_ops(Arc::from([vertex_color_quad_scene_op()])),
        ]);

        assert_eq!(
            FrameAssembler::evaluate_support(&frame),
            ChunkLaunchSupport::Unsupported {
                stream_class: Some(ChunkLaunchStreamClass::Mixed),
                reason: ChunkLaunchUnsupportedReason::MixedStreams,
            }
        );
    }

    #[test]
    fn frame_assembler_rejects_side_table_manifest_before_payload_lookup() {
        let frame = manifest(&[entry_for_ops(Arc::from([
            SceneOp::PushClipRRect {
                rect: Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0))),
                corner_radii: fret_core::Corners::all(Px(2.0)),
            },
            quad_scene_op(),
            SceneOp::PopClip,
        ]))]);
        let assembler = FrameAssembler::default();

        let result = assembler.try_assemble_supported_frame_encoding(
            &frame,
            context(),
            ChunkLaunchStreamClass::ResourceFreeQuad,
        );
        let Err(reason) = result else {
            panic!("side-table manifest should be rejected before payload lookup");
        };
        assert_eq!(
            reason,
            FrameAssemblyUnsupportedReason::UnsupportedLaunch {
                stream_class: Some(ChunkLaunchStreamClass::ResourceFreeQuad),
                reason: ChunkLaunchUnsupportedReason::ManifestUnsupported(
                    fret_core::SceneChunkManifestUnsupportedReason::EntrySideTableRequired {
                        entry_index: 0,
                        requirements: fret_core::SceneChunkSideTableRequirements {
                            clip_scopes: 1,
                            ..Default::default()
                        },
                    },
                ),
            }
        );
    }

    #[test]
    fn frame_assembler_assembles_supported_quad_payloads() {
        let frame = manifest(&[entry_for_ops(Arc::from([quad_scene_op()]))]);
        let mut assembler = FrameAssembler::default();
        assembler.begin_frame_with_payloads(Some(&frame), context(), &[0], |_| {
            CachedSceneChunkEncoding::new(quad_payload_encoding())
        });

        let encoding = assembler
            .try_assemble_supported_frame_encoding(
                &frame,
                context(),
                ChunkLaunchStreamClass::ResourceFreeQuad,
            )
            .expect("resource-free quad payloads should assemble");

        assert_eq!(encoding.instances.len(), 1);
        assert_eq!(encoding.uniforms.len(), 1);
        assert!(matches!(
            encoding.ordered_draws.as_slice(),
            [OrderedDraw::Quad(_)]
        ));
    }
}
