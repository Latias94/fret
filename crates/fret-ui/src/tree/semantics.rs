use super::*;

pub(super) fn validate_semantics_if_enabled(snapshot: &SemanticsSnapshot) {
    let config = crate::runtime_config::ui_runtime_config();
    if !config.validate_semantics {
        return;
    }

    if let Err(err) = snapshot.validate() {
        tracing::error!(
            node = ?err.node,
            kind = ?err.kind,
            "semantics validation failed (set FRET_VALIDATE_SEMANTICS_PANIC=1 to panic)"
        );

        if config.validate_semantics_panic {
            panic!("semantics validation failed: {err:?}");
        }
    }
}
