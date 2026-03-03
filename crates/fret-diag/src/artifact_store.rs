use std::path::{Path, PathBuf};

use fret_diag_protocol::UiScriptResultV1;

/// A small, focused API boundary for per-run tooling artifacts.
///
/// This is intentionally a thin wrapper over `run_artifacts` in v1. The goal is to keep
/// "artifact materialization + integrity" operations behind one module boundary so we can
/// refactor storage/layout/chunking without touching unrelated orchestration code.
pub(crate) struct RunArtifactStore<'a> {
    out_dir: &'a Path,
    run_id: u64,
}

impl<'a> RunArtifactStore<'a> {
    pub(crate) fn new(out_dir: &'a Path, run_id: u64) -> Self {
        Self { out_dir, run_id }
    }

    pub(crate) fn run_dir(&self) -> PathBuf {
        run_id_artifact_dir(self.out_dir, self.run_id)
    }

    pub(crate) fn write_script_result(&self, result: &UiScriptResultV1) {
        crate::run_artifacts::write_run_id_script_result(self.out_dir, self.run_id, result);
    }

    pub(crate) fn write_bundle_artifact(&self, bundle_artifact_path: &Path) {
        crate::run_artifacts::write_run_id_bundle_json(
            self.out_dir,
            self.run_id,
            bundle_artifact_path,
        );
    }

    pub(crate) fn refresh_manifest_file_index(&self) {
        crate::run_artifacts::refresh_run_id_manifest_file_index(self.out_dir, self.run_id);
    }
}

pub(crate) fn run_id_artifact_dir(out_dir: &Path, run_id: u64) -> PathBuf {
    crate::run_artifacts::run_id_artifact_dir(out_dir, run_id)
}

pub(crate) fn materialize_bundle_json_from_manifest_chunks_if_missing(
    dir: &Path,
) -> Result<Option<PathBuf>, String> {
    crate::run_artifacts::materialize_bundle_json_from_manifest_chunks_if_missing(dir)
}

pub(crate) fn materialize_run_id_bundle_json_from_chunks_if_missing(
    out_dir: &Path,
    run_id: u64,
) -> Result<Option<PathBuf>, String> {
    crate::run_artifacts::materialize_run_id_bundle_json_from_chunks_if_missing(out_dir, run_id)
}
