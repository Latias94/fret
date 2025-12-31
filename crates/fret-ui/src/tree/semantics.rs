use super::*;

pub(super) fn validate_semantics_if_enabled(snapshot: &SemanticsSnapshot) {
    if std::env::var_os("FRET_VALIDATE_SEMANTICS").is_none() {
        return;
    }

    if let Err(err) = snapshot.validate() {
        tracing::error!(
            node = ?err.node,
            kind = ?err.kind,
            "semantics validation failed (set FRET_VALIDATE_SEMANTICS_PANIC=1 to panic)"
        );

        if std::env::var_os("FRET_VALIDATE_SEMANTICS_PANIC").is_some() {
            panic!("semantics validation failed: {err:?}");
        }
    }
}
