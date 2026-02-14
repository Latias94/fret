use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiArtifactStatsV1, UiScriptResultV1, UiScriptStageV1};
use serde::{Deserialize, Serialize};

use crate::util::{now_unix_ms, write_json_value};

const BUNDLE_JSON_CHUNK_BYTES: usize = 256 * 1024;

pub(crate) fn run_id_artifact_dir(out_dir: &Path, run_id: u64) -> PathBuf {
    out_dir.join(run_id.to_string())
}

pub(crate) fn write_run_id_script_result(out_dir: &Path, run_id: u64, result: &UiScriptResultV1) {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let path = dir.join("script.result.json");
    let _ = write_json_value(
        &path,
        &serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({})),
    );
    write_run_id_manifest_json(out_dir, run_id, result);
}

pub(crate) fn write_run_id_bundle_json(out_dir: &Path, run_id: u64, bundle_json_path: &Path) {
    if !bundle_json_path.is_file() {
        return;
    }
    let dir = run_id_artifact_dir(out_dir, run_id);
    let dst = dir.join("bundle.json");
    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Best-effort alias: keep a stable per-run path even when the underlying bundle export directory
    // is timestamp/label-based (filesystem) or message-derived (WS).
    if std::fs::copy(bundle_json_path, &dst).is_ok() {
        let chunks = write_run_id_bundle_json_chunks(out_dir, run_id, &dst);
        if let Ok(chunks) = chunks {
            update_run_id_manifest_with_bundle_json_chunks(out_dir, run_id, &chunks);
        }
    }
}

fn stage_as_str(stage: &UiScriptStageV1) -> &'static str {
    match stage {
        UiScriptStageV1::Queued => "queued",
        UiScriptStageV1::Running => "running",
        UiScriptStageV1::Passed => "passed",
        UiScriptStageV1::Failed => "failed",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestPathsV1 {
    script_result_json: String,
    bundle_json: String,
}

fn default_manifest_paths() -> RunManifestPathsV1 {
    RunManifestPathsV1 {
        script_result_json: "script.result.json".to_string(),
        bundle_json: "bundle.json".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestScriptResultSummaryV1 {
    stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason_code: Option<String>,
    updated_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestChunkV1 {
    index: u32,
    path: String,
    bytes: u64,
    blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestChunkedFileV1 {
    mode: String,
    total_bytes: u64,
    chunk_bytes: u64,
    blake3: String,
    chunks: Vec<RunManifestChunkV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestFileV1 {
    id: String,
    path: String,
    bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunManifestV2 {
    schema_version: u32,
    generated_unix_ms: u64,
    run_id: u64,
    #[serde(default = "default_manifest_paths")]
    paths: RunManifestPathsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    script_result: Option<RunManifestScriptResultSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_bundle_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_bundle_artifact: Option<UiArtifactStatsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bundle_json: Option<RunManifestChunkedFileV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    files: Vec<RunManifestFileV1>,
}

impl RunManifestV2 {
    fn new(run_id: u64) -> Self {
        Self {
            schema_version: 2,
            generated_unix_ms: now_unix_ms(),
            run_id,
            paths: RunManifestPathsV1 {
                script_result_json: "script.result.json".to_string(),
                bundle_json: "bundle.json".to_string(),
            },
            script_result: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
            bundle_json: None,
            files: Vec::new(),
        }
    }
}

fn read_run_id_script_result(out_dir: &Path, run_id: u64) -> Option<UiScriptResultV1> {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let bytes = std::fs::read(dir.join("script.result.json")).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn push_file_entry_if_present(manifest: &mut RunManifestV2, dir: &Path, id: &str, rel: &str) {
    let path = dir.join(rel);
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    manifest.files.push(RunManifestFileV1 {
        id: id.to_string(),
        path: rel.to_string(),
        bytes: bytes.len() as u64,
        blake3: Some(blake3_hex(&bytes)),
    });
}

pub(crate) fn write_run_id_manifest_json(out_dir: &Path, run_id: u64, result: &UiScriptResultV1) {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let path = dir.join("manifest.json");

    let mut manifest = RunManifestV2::new(run_id);
    manifest.generated_unix_ms = now_unix_ms();
    manifest.script_result = Some(RunManifestScriptResultSummaryV1 {
        stage: stage_as_str(&result.stage).to_string(),
        reason_code: result.reason_code.clone(),
        updated_unix_ms: result.updated_unix_ms,
    });
    manifest.last_bundle_dir = result.last_bundle_dir.clone();
    manifest.last_bundle_artifact = result.last_bundle_artifact.clone();

    push_file_entry_if_present(&mut manifest, &dir, "script_result_json", "script.result.json");
    let _ = write_json_value(
        &path,
        &serde_json::to_value(&manifest).unwrap_or_else(|_| serde_json::json!({})),
    );
}

#[derive(Debug, Clone)]
pub(crate) struct BundleJsonChunksV1 {
    pub total_bytes: u64,
    pub chunk_bytes: u64,
    pub blake3: String,
    pub chunks: Vec<BundleJsonChunkV1>,
}

#[derive(Debug, Clone)]
pub(crate) struct BundleJsonChunkV1 {
    pub index: u32,
    pub rel_path: String,
    pub bytes: u64,
    pub blake3: String,
}

fn run_id_bundle_json_chunk_dir(out_dir: &Path, run_id: u64) -> PathBuf {
    run_id_artifact_dir(out_dir, run_id)
        .join("chunks")
        .join("bundle_json")
}

pub(crate) fn write_run_id_bundle_json_chunks(
    out_dir: &Path,
    run_id: u64,
    bundle_json_path: &Path,
) -> Result<BundleJsonChunksV1, String> {
    let chunks_dir = run_id_bundle_json_chunk_dir(out_dir, run_id);
    std::fs::create_dir_all(&chunks_dir).map_err(|e| e.to_string())?;

    if let Ok(entries) = std::fs::read_dir(&chunks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    let mut file = std::fs::File::open(bundle_json_path).map_err(|e| e.to_string())?;
    let mut total_hasher = blake3::Hasher::new();

    let mut index: u32 = 0;
    let mut total_bytes: u64 = 0;
    let mut chunks: Vec<BundleJsonChunkV1> = Vec::new();

    loop {
        let mut buf = vec![0u8; BUNDLE_JSON_CHUNK_BYTES];
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        buf.truncate(n);
        total_bytes = total_bytes.saturating_add(n as u64);

        total_hasher.update(&buf);
        let chunk_hash = blake3::hash(&buf).to_hex().to_string();

        let name = format!("chunk-{index:06}");
        let chunk_path = chunks_dir.join(&name);
        let mut out = std::fs::File::create(&chunk_path).map_err(|e| e.to_string())?;
        out.write_all(&buf).map_err(|e| e.to_string())?;
        out.flush().ok();

        let rel_path = PathBuf::from("chunks")
            .join("bundle_json")
            .join(&name)
            .to_string_lossy()
            .replace('\\', "/");
        chunks.push(BundleJsonChunkV1 {
            index,
            rel_path,
            bytes: n as u64,
            blake3: chunk_hash,
        });

        index = index.saturating_add(1);
    }

    Ok(BundleJsonChunksV1 {
        total_bytes,
        chunk_bytes: BUNDLE_JSON_CHUNK_BYTES as u64,
        blake3: total_hasher.finalize().to_hex().to_string(),
        chunks,
    })
}

pub(crate) fn materialize_run_id_bundle_json_from_chunks_if_missing(
    out_dir: &Path,
    run_id: u64,
) -> Result<Option<PathBuf>, String> {
    let run_dir = run_id_artifact_dir(out_dir, run_id);
    materialize_bundle_json_from_manifest_chunks_if_missing(&run_dir)
}

fn update_run_id_manifest_with_bundle_json_chunks(
    out_dir: &Path,
    run_id: u64,
    chunks: &BundleJsonChunksV1,
) {
    let dir = run_id_artifact_dir(out_dir, run_id);
    let path = dir.join("manifest.json");

    let mut manifest = if path.is_file() {
        std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<RunManifestV2>(&b).ok())
            .unwrap_or_else(|| RunManifestV2::new(run_id))
    } else {
        RunManifestV2::new(run_id)
    };

    if manifest.script_result.is_none() {
        if let Some(result) = read_run_id_script_result(out_dir, run_id) {
            manifest.script_result = Some(RunManifestScriptResultSummaryV1 {
                stage: stage_as_str(&result.stage).to_string(),
                reason_code: result.reason_code.clone(),
                updated_unix_ms: result.updated_unix_ms,
            });
            manifest.last_bundle_dir = result.last_bundle_dir.clone();
            manifest.last_bundle_artifact = result.last_bundle_artifact.clone();
        }
    }

    manifest.schema_version = 2;
    manifest.generated_unix_ms = now_unix_ms();
    manifest.run_id = run_id;
    manifest.bundle_json = Some(RunManifestChunkedFileV1 {
        mode: "chunks.v1".to_string(),
        total_bytes: chunks.total_bytes,
        chunk_bytes: chunks.chunk_bytes,
        blake3: chunks.blake3.clone(),
        chunks: chunks
            .chunks
            .iter()
            .map(|c| RunManifestChunkV1 {
                index: c.index,
                path: c.rel_path.clone(),
                bytes: c.bytes,
                blake3: c.blake3.clone(),
            })
            .collect(),
    });

    // Refresh file index (bounded to stable, small files; bundle.json is indexed via chunks).
    manifest.files.retain(|f| f.id != "script_result_json");
    push_file_entry_if_present(&mut manifest, &dir, "script_result_json", "script.result.json");

    let _ = write_json_value(
        &path,
        &serde_json::to_value(&manifest).unwrap_or_else(|_| serde_json::json!({})),
    );
}

pub(crate) fn materialize_bundle_json_from_manifest_chunks_if_missing(
    dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let bundle_json_path = dir.join("bundle.json");
    if bundle_json_path.is_file() {
        return Ok(Some(bundle_json_path));
    }

    let manifest_path = dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Ok(None);
    }

    let bytes = std::fs::read(&manifest_path).map_err(|e| e.to_string())?;
    let parsed = serde_json::from_slice::<RunManifestV2>(&bytes).map_err(|e| {
        format!(
            "manifest.json was not valid JSON (v2): {} ({})",
            manifest_path.display(),
            e
        )
    })?;

    let Some(bundle_json) = parsed.bundle_json else {
        return Ok(None);
    };
    if bundle_json.chunks.is_empty() {
        return Ok(None);
    }

    let expected_total_blake3 = Some(bundle_json.blake3);

    let tmp_path = dir.join("bundle.json.tmp");
    let mut out = std::fs::File::create(&tmp_path).map_err(|e| e.to_string())?;
    let mut total_hasher = blake3::Hasher::new();
    for chunk in bundle_json.chunks {
        let rel = chunk.path;
        let expected_chunk_blake3 = Some(chunk.blake3);
        let expected_chunk_bytes = Some(chunk.bytes);

        let chunk_path = dir.join(rel);
        let bytes = std::fs::read(&chunk_path).map_err(|e| {
            format!("failed to read bundle json chunk ({}): {}", chunk_path.display(), e)
        })?;
        if let Some(expected) = expected_chunk_bytes {
            if expected != bytes.len() as u64 {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(format!(
                    "bundle json chunk size mismatch ({}): expected={} actual={}",
                    chunk_path.display(),
                    expected,
                    bytes.len()
                ));
            }
        }
        if let Some(expected) = expected_chunk_blake3 {
            let actual = blake3::hash(&bytes).to_hex().to_string();
            if actual != expected {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(format!(
                    "bundle json chunk hash mismatch ({}): expected={} actual={}",
                    chunk_path.display(),
                    expected,
                    actual
                ));
            }
        }

        total_hasher.update(&bytes);
        out.write_all(&bytes).map_err(|e| e.to_string())?;
    }
    out.flush().ok();

    if let Some(expected) = expected_total_blake3 {
        let actual = total_hasher.finalize().to_hex().to_string();
        if actual != expected {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!(
                "bundle json total hash mismatch ({}): expected={} actual={}",
                manifest_path.display(),
                expected,
                actual
            ));
        }
    }

    std::fs::rename(&tmp_path, &bundle_json_path).map_err(|e| e.to_string())?;
    Ok(Some(bundle_json_path))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use fret_diag_protocol::{UiScriptEvidenceV1, UiScriptStageV1};

    use super::*;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn write_run_id_script_result_also_writes_manifest_json() {
        let root = make_temp_dir("fret-diag-run-artifacts-manifest");
        let run_id = 7u64;
        let result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: now_unix_ms(),
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };

        write_run_id_script_result(&root, run_id, &result);

        let manifest_path = root.join(run_id.to_string()).join("manifest.json");
        let bytes = std::fs::read(&manifest_path).expect("read manifest.json");
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse manifest.json");
        assert_eq!(parsed.get("run_id").and_then(|v| v.as_u64()), Some(run_id));
        assert_eq!(
            parsed
                .get("script_result")
                .and_then(|v| v.get("stage"))
                .and_then(|v| v.as_str()),
            Some("passed")
        );
        assert_eq!(
            parsed
                .get("paths")
                .and_then(|v| v.get("script_result_json"))
                .and_then(|v| v.as_str()),
            Some("script.result.json")
        );
    }

    #[test]
    fn write_run_id_bundle_json_writes_chunks_and_updates_manifest() {
        let root = make_temp_dir("fret-diag-run-artifacts-chunks");
        let run_id = 9u64;

        let result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: now_unix_ms(),
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        write_run_id_script_result(&root, run_id, &result);

        let export_dir = root.join("export");
        std::fs::create_dir_all(&export_dir).expect("create export dir");
        let src = export_dir.join("bundle.json");
        std::fs::write(&src, br#"{ "schema_version": 1, "windows": [] }"#)
            .expect("write src bundle.json");

        write_run_id_bundle_json(&root, run_id, &src);

        let chunks_dir = run_id_bundle_json_chunk_dir(&root, run_id);
        let entries = std::fs::read_dir(&chunks_dir)
            .expect("read chunks dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        assert!(entries > 0, "expected at least one chunk file");

        let manifest_path = root.join(run_id.to_string()).join("manifest.json");
        let bytes = std::fs::read(&manifest_path).expect("read manifest.json");
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse manifest.json");
        assert!(
            parsed.get("bundle_json").is_some(),
            "expected bundle_json section"
        );
        assert!(
            parsed
                .get("bundle_json")
                .and_then(|v| v.get("blake3"))
                .and_then(|v| v.as_str())
                .is_some(),
            "expected bundle_json.blake3"
        );
        assert_eq!(
            parsed
                .get("bundle_json")
                .and_then(|v| v.get("chunks"))
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true),
            false
        );

        let run_dir = root.join(run_id.to_string());
        let bundle = std::fs::read(run_dir.join("bundle.json")).expect("read run bundle.json");
        assert!(!bundle.is_empty());
    }

    #[test]
    fn materialize_run_id_bundle_json_from_chunks_if_missing_reconstructs_bundle_json() {
        let root = make_temp_dir("fret-diag-run-artifacts-reconstruct");
        let run_id = 11u64;

        let result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: now_unix_ms(),
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        write_run_id_script_result(&root, run_id, &result);

        let run_dir = root.join(run_id.to_string());
        let bundle_json_path = run_dir.join("bundle.json");
        std::fs::write(&bundle_json_path, br#"{ "schema_version": 1, "windows": [] }"#)
            .expect("write bundle.json");

        let chunks = write_run_id_bundle_json_chunks(&root, run_id, &bundle_json_path)
            .expect("write chunks");
        update_run_id_manifest_with_bundle_json_chunks(&root, run_id, &chunks);

        std::fs::remove_file(&bundle_json_path).expect("remove bundle.json");

        let rebuilt = materialize_run_id_bundle_json_from_chunks_if_missing(&root, run_id)
            .expect("rebuilt result")
            .expect("rebuilt path");
        assert!(rebuilt.is_file());

        let bytes = std::fs::read(rebuilt).expect("read rebuilt bundle.json");
        assert!(!bytes.is_empty());
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse rebuilt json");
        assert_eq!(
            parsed.get("schema_version").and_then(|v| v.as_u64()),
            Some(1)
        );
    }
}
