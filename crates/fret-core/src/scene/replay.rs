use super::*;

impl SceneRecording {
    pub fn replay_ops(&mut self, ops: &[SceneOp]) {
        self.ops.reserve(ops.len());
        for &op in ops {
            if let SceneOp::Text { text, .. } = op {
                self.text_blob_ids.push(text);
            }
            self.fingerprint = mix_scene_op(self.fingerprint, op);
            self.ops.push(op);
        }
    }

    /// Replay retained scene ops with a precomputed text-blob index.
    ///
    /// `text_blob_ids` must contain exactly the `SceneOp::Text` blob ids from `ops`, in op order.
    /// This keeps replay semantically equivalent to `replay_ops` while allowing hot cache-hit paths
    /// to avoid rescanning their retained op buffer.
    pub fn replay_ops_with_text_blob_ids(&mut self, ops: &[SceneOp], text_blob_ids: &[TextBlobId]) {
        #[cfg(debug_assertions)]
        debug_assert!(
            ops.iter()
                .filter_map(|op| match op {
                    SceneOp::Text { text, .. } => Some(*text),
                    _ => None,
                })
                .eq(text_blob_ids.iter().copied()),
            "Scene::replay_ops_with_text_blob_ids() received a text blob index that does not match the replayed ops"
        );

        self.text_blob_ids.extend_from_slice(text_blob_ids);
        self.ops.reserve(ops.len());
        for &op in ops {
            self.fingerprint = mix_scene_op(self.fingerprint, op);
            self.ops.push(op);
        }
    }

    pub fn replay_ops_translated(&mut self, ops: &[SceneOp], delta: Point) {
        if delta.x.0 == 0.0 && delta.y.0 == 0.0 {
            self.replay_ops(ops);
            return;
        }

        self.replay_ops_transformed(ops, Transform2D::translation(delta));
    }

    pub fn replay_ops_translated_with_text_blob_ids(
        &mut self,
        ops: &[SceneOp],
        delta: Point,
        text_blob_ids: &[TextBlobId],
    ) {
        if delta.x.0 == 0.0 && delta.y.0 == 0.0 {
            self.replay_ops_with_text_blob_ids(ops, text_blob_ids);
            return;
        }

        self.replay_ops_transformed_with_text_blob_ids(
            ops,
            Transform2D::translation(delta),
            text_blob_ids,
        );
    }

    pub fn replay_ops_transformed(&mut self, ops: &[SceneOp], transform: Transform2D) {
        self.with_transform(transform, |scene| scene.replay_ops(ops));
    }

    pub fn replay_ops_transformed_with_text_blob_ids(
        &mut self,
        ops: &[SceneOp],
        transform: Transform2D,
        text_blob_ids: &[TextBlobId],
    ) {
        self.with_transform(transform, |scene| {
            scene.replay_ops_with_text_blob_ids(ops, text_blob_ids)
        });
    }
}
