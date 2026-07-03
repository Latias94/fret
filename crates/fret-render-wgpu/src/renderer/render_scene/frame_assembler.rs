use super::super::*;

#[derive(Default)]
pub(in crate::renderer) struct FrameAssembler {
    scene_chunk_encoding_state: SceneChunkEncodingState,
}

impl FrameAssembler {
    pub(in crate::renderer) fn assemble_supported_frame_encoding(
        &self,
        manifest: &fret_core::SceneChunkManifest,
        context: SceneChunkEncodingContext,
        stream_class: ChunkLaunchStreamClass,
    ) -> Option<SceneEncoding> {
        self.scene_chunk_encoding_state
            .assemble_supported_frame_encoding(manifest, context, stream_class)
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
