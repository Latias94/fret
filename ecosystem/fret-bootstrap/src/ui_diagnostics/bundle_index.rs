use serde::{Deserialize, Serialize};

use super::{UiDiagnosticsSnapshotV1, UiDiagnosticsWindowBundleV1, bundle, unix_ms_now};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleIndexV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub bundle_schema_version: u32,
    pub semantics_mode: String,
    pub windows: Vec<UiDiagnosticsBundleIndexWindowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleIndexWindowV1 {
    pub window: u64,
    pub snapshots: Vec<UiDiagnosticsBundleIndexSnapshotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleIndexSnapshotV1 {
    pub tick_id: u64,
    pub frame_id: u64,
    pub window_snapshot_seq: u64,
    pub timestamp_unix_ms: u64,
    pub scene_ops: u64,
    pub scene_fingerprint: u64,
    pub semantics_fingerprint: Option<u64>,
    pub has_semantics: bool,
    pub changed_models: u64,
    pub changed_globals: u64,
}

impl UiDiagnosticsBundleIndexV1 {
    pub(super) fn from_windows(
        exported_unix_ms: u64,
        bundle_schema_version: u32,
        semantics_mode: bundle::BundleSemanticsModeV1,
        windows: &[UiDiagnosticsWindowBundleV1],
    ) -> Self {
        Self {
            schema_version: 1,
            exported_unix_ms,
            bundle_schema_version,
            semantics_mode: format!("{:?}", semantics_mode).to_ascii_lowercase(),
            windows: windows
                .iter()
                .map(|w| UiDiagnosticsBundleIndexWindowV1 {
                    window: w.window,
                    snapshots: w
                        .snapshots
                        .iter()
                        .map(UiDiagnosticsBundleIndexSnapshotV1::from_snapshot)
                        .collect(),
                })
                .collect(),
        }
    }
}

impl UiDiagnosticsBundleIndexSnapshotV1 {
    fn from_snapshot(snapshot: &UiDiagnosticsSnapshotV1) -> Self {
        Self {
            tick_id: snapshot.tick_id,
            frame_id: snapshot.frame_id,
            window_snapshot_seq: snapshot.window_snapshot_seq,
            timestamp_unix_ms: snapshot.timestamp_unix_ms,
            scene_ops: snapshot.scene_ops,
            scene_fingerprint: snapshot.scene_fingerprint,
            semantics_fingerprint: snapshot.semantics_fingerprint,
            has_semantics: snapshot.debug.semantics.is_some(),
            changed_models: snapshot.changed_models.len() as u64,
            changed_globals: snapshot.changed_globals.len() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsBundleMetaV1 {
    pub schema_version: u32,
    pub exported_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub bundle_schema_version: u32,
    pub semantics_mode: String,
    pub window_count: u64,
    pub snapshot_count: u64,
}

impl UiDiagnosticsBundleMetaV1 {
    pub(super) fn from_windows(
        exported_unix_ms: u64,
        bundle_schema_version: u32,
        semantics_mode: bundle::BundleSemanticsModeV1,
        windows: &[UiDiagnosticsWindowBundleV1],
    ) -> Self {
        Self {
            schema_version: 1,
            exported_unix_ms,
            updated_unix_ms: unix_ms_now(),
            bundle_schema_version,
            semantics_mode: format!("{:?}", semantics_mode).to_ascii_lowercase(),
            window_count: windows.len() as u64,
            snapshot_count: windows.iter().map(|w| w.snapshots.len() as u64).sum(),
        }
    }
}
