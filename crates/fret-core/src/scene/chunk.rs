use std::sync::Arc;

use super::{SceneOp, SceneRecording, TextBlobId, mix_scene_op};
use crate::geometry::{Point, Transform2D};

#[derive(Debug, Default, Clone)]
pub struct SceneChunk {
    ops: Arc<[SceneOp]>,
    text_blob_ids: Arc<[TextBlobId]>,
    fingerprint: u64,
}

impl SceneChunk {
    pub fn from_scene(scene: &SceneRecording) -> Self {
        Self::from_ops_and_text_blob_ids(
            Arc::from(scene.ops().to_vec()),
            Arc::from(scene.text_blob_ids().to_vec()),
        )
    }

    pub fn from_ops(ops: Arc<[SceneOp]>) -> Self {
        let text_blob_ids = Arc::from(
            ops.iter()
                .filter_map(|op| match op {
                    SceneOp::Text { text, .. } => Some(*text),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        );
        Self::from_ops_and_text_blob_ids(ops, text_blob_ids)
    }

    pub fn from_ops_and_text_blob_ids(
        ops: Arc<[SceneOp]>,
        text_blob_ids: Arc<[TextBlobId]>,
    ) -> Self {
        #[cfg(debug_assertions)]
        debug_assert!(
            ops.iter()
                .filter_map(|op| match op {
                    SceneOp::Text { text, .. } => Some(*text),
                    _ => None,
                })
                .eq(text_blob_ids.iter().copied()),
            "SceneChunk::from_ops_and_text_blob_ids() received a text blob index that does not match the retained ops"
        );

        let fingerprint = ops
            .iter()
            .fold(0, |fingerprint, op| mix_scene_op(fingerprint, *op));
        Self {
            ops,
            text_blob_ids,
            fingerprint,
        }
    }

    pub fn ops(&self) -> &[SceneOp] {
        &self.ops
    }

    pub fn ops_len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn text_blob_ids(&self) -> &[TextBlobId] {
        &self.text_blob_ids
    }

    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub fn replay_into(&self, scene: &mut SceneRecording) {
        scene.replay_ops_with_text_blob_ids(self.ops(), self.text_blob_ids());
    }

    pub fn replay_translated_into(&self, scene: &mut SceneRecording, delta: Point) {
        scene.replay_ops_translated_with_text_blob_ids(self.ops(), delta, self.text_blob_ids());
    }

    pub fn replay_transformed_into(&self, scene: &mut SceneRecording, transform: Transform2D) {
        scene.replay_ops_transformed_with_text_blob_ids(
            self.ops(),
            transform,
            self.text_blob_ids(),
        );
    }
}
