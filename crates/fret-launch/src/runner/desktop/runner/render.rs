use fret_core::Scene;
use tracing::error;

pub(super) fn validate_scene_if_enabled(scene: &Scene) {
    if std::env::var_os("FRET_VALIDATE_SCENE").is_none() {
        return;
    }

    if let Err(err) = scene.validate() {
        error!(
            index = err.index,
            op = ?err.op,
            kind = ?err.kind,
            error = %err,
            "scene validation failed (set FRET_VALIDATE_SCENE_PANIC=1 to panic)"
        );

        if std::env::var_os("FRET_VALIDATE_SCENE_PANIC").is_some() {
            panic!("scene validation failed: {err}");
        }
    }
}
