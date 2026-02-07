use super::*;

impl SceneRecording {
    pub fn replay_ops(&mut self, ops: &[SceneOp]) {
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

    pub fn replay_ops_transformed(&mut self, ops: &[SceneOp], transform: Transform2D) {
        self.with_transform(transform, |scene| scene.replay_ops(ops));
    }
}
