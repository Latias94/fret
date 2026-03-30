use std::path::{Path, PathBuf};

use fret_diag_protocol::UiScriptResultV1;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtifactLintLevel {
    Error,
    Warning,
    Info,
}

impl ArtifactLintLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArtifactLintReport {
    pub(crate) error_issues: u64,
    pub(crate) payload: Value,
}

fn push_finding(
    findings: &mut Vec<Value>,
    level: ArtifactLintLevel,
    code: &str,
    message: impl Into<String>,
    evidence: Value,
) {
    findings.push(serde_json::json!({
        "level": level.as_str(),
        "code": code,
        "message": message.into(),
        "evidence": evidence,
    }));
}

fn resolve_manifest_path(dir: &Path) -> Option<PathBuf> {
    let direct = dir.join("manifest.json");
    if direct.is_file() {
        return Some(direct);
    }
    let root = dir.join("_root").join("manifest.json");
    if root.is_file() {
        return Some(root);
    }
    None
}

fn resolve_script_result_path(dir: &Path, manifest: &Value) -> PathBuf {
    let from_manifest = manifest
        .get("paths")
        .and_then(|p| {
            p.get("script_result")
                .or_else(|| p.get("script_result_json"))
        })
        .and_then(|v| v.as_str())
        .map(|s| dir.join(s));
    from_manifest.unwrap_or_else(|| dir.join("script.result.json"))
}

fn try_read_json_value(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn try_read_script_result_v1(path: &Path) -> Option<UiScriptResultV1> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn lint_sidecar_basic(
    findings: &mut Vec<Value>,
    dir: &Path,
    expected_kind: &str,
    filename: &str,
    warmup_frames: u64,
) {
    let path = dir.join(filename);
    if !path.is_file() {
        push_finding(
            findings,
            ArtifactLintLevel::Warning,
            "sidecar.missing",
            format!("missing sidecar: {filename}"),
            serde_json::json!({ "path": path.display().to_string() }),
        );
        return;
    }

    let v = match try_read_json_value(&path) {
        Ok(v) => v,
        Err(err) => {
            push_finding(
                findings,
                ArtifactLintLevel::Error,
                "sidecar.invalid_json",
                format!("sidecar is not valid JSON: {filename}"),
                serde_json::json!({ "path": path.display().to_string(), "error": err }),
            );
            return;
        }
    };

    let got_kind = v.get("kind").and_then(|v| v.as_str());
    if got_kind != Some(expected_kind) {
        push_finding(
            findings,
            ArtifactLintLevel::Error,
            "sidecar.kind_mismatch",
            format!("unexpected sidecar kind for {filename} (expected={expected_kind})"),
            serde_json::json!({
                "path": path.display().to_string(),
                "expected_kind": expected_kind,
                "got_kind": got_kind,
            }),
        );
        return;
    }

    let got_schema = v.get("schema_version").and_then(|v| v.as_u64());
    if got_schema != Some(1) {
        push_finding(
            findings,
            ArtifactLintLevel::Error,
            "sidecar.schema_version_mismatch",
            format!("unexpected sidecar schema_version for {filename} (expected=1)"),
            serde_json::json!({
                "path": path.display().to_string(),
                "expected_schema_version": 1,
                "got_schema_version": got_schema,
            }),
        );
        return;
    }

    if warmup_frames > 0 {
        let got_warmup = v.get("warmup_frames").and_then(|v| v.as_u64());
        if got_warmup != Some(warmup_frames) {
            push_finding(
                findings,
                ArtifactLintLevel::Warning,
                "sidecar.warmup_frames_mismatch",
                format!("warmup_frames mismatch for {filename}"),
                serde_json::json!({
                    "path": path.display().to_string(),
                    "expected_warmup_frames": warmup_frames,
                    "got_warmup_frames": got_warmup,
                }),
            );
        }
    }
}

fn counts_from_findings(findings: &[Value]) -> (u64, u64) {
    let errors = findings
        .iter()
        .filter(|f| f.get("level").and_then(|v| v.as_str()) == Some("error"))
        .count() as u64;
    let warnings = findings
        .iter()
        .filter(|f| f.get("level").and_then(|v| v.as_str()) == Some("warning"))
        .count() as u64;
    (errors, warnings)
}

pub(crate) fn lint_run_artifact_dir(
    dir: &Path,
    warmup_frames: u64,
) -> Result<ArtifactLintReport, String> {
    let mut findings: Vec<Value> = Vec::new();

    let Some(manifest_path) = resolve_manifest_path(dir) else {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "manifest.missing",
            "manifest.json not found (expected <dir>/manifest.json or <dir>/_root/manifest.json)",
            serde_json::json!({ "dir": dir.display().to_string() }),
        );
        let (errors, warnings) = counts_from_findings(&findings);
        let payload = serde_json::json!({
            "kind": "diag_artifact_lint",
            "schema_version": 1,
            "ok": false,
            "artifact_dir": dir.display().to_string(),
            "manifest_path": null,
            "error_issues": errors,
            "warning_issues": warnings,
            "findings": findings,
        });
        return Ok(ArtifactLintReport {
            error_issues: errors,
            payload,
        });
    };

    let manifest_dir = manifest_path.parent().unwrap_or(dir).to_path_buf();

    let manifest = match try_read_json_value(&manifest_path) {
        Ok(v) => v,
        Err(err) => {
            push_finding(
                &mut findings,
                ArtifactLintLevel::Error,
                "manifest.invalid_json",
                "manifest.json exists but could not be parsed as JSON",
                serde_json::json!({ "path": manifest_path.display().to_string(), "error": err }),
            );
            let (errors, warnings) = counts_from_findings(&findings);
            let payload = serde_json::json!({
                "kind": "diag_artifact_lint",
                "schema_version": 1,
                "ok": false,
                "artifact_dir": dir.display().to_string(),
                "manifest_path": manifest_path.display().to_string(),
                "error_issues": errors,
                "warning_issues": warnings,
                "findings": findings,
            });
            return Ok(ArtifactLintReport {
                error_issues: errors,
                payload,
            });
        }
    };

    let manifest_schema_version = manifest.get("schema_version").and_then(|v| v.as_u64());
    if manifest_schema_version != Some(2) {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "manifest.schema_version_mismatch",
            "manifest schema_version is not supported (expected 2)",
            serde_json::json!({
                "path": manifest_path.display().to_string(),
                "expected_schema_version": 2,
                "got_schema_version": manifest_schema_version,
            }),
        );
    }

    let generated_unix_ms = manifest
        .get("generated_unix_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if generated_unix_ms == 0 {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Warning,
            "manifest.generated_unix_ms_missing_or_zero",
            "manifest.generated_unix_ms is missing or zero",
            serde_json::json!({ "path": manifest_path.display().to_string() }),
        );
    } else {
        let now = crate::util::now_unix_ms();
        const FUTURE_SKEW_MS: u64 = 24 * 60 * 60 * 1000;
        if generated_unix_ms > now.saturating_add(FUTURE_SKEW_MS) {
            push_finding(
                &mut findings,
                ArtifactLintLevel::Warning,
                "manifest.generated_unix_ms_in_future",
                "manifest.generated_unix_ms is far in the future (clock skew?)",
                serde_json::json!({
                    "generated_unix_ms": generated_unix_ms,
                    "now_unix_ms": now,
                }),
            );
        }
    }

    let manifest_run_id = manifest.get("run_id").and_then(|v| v.as_u64());
    if manifest_run_id.is_none() {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "manifest.run_id_missing",
            "manifest is missing run_id",
            serde_json::json!({ "path": manifest_path.display().to_string() }),
        );
    }

    let dir_run_id = dir
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<u64>().ok());

    let script_result_path = resolve_script_result_path(&manifest_dir, &manifest);
    let script_result = try_read_script_result_v1(&script_result_path);
    if script_result.is_none() {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "script_result.missing_or_invalid",
            "script.result.json is missing or invalid (expected UiScriptResultV1 JSON)",
            serde_json::json!({ "path": script_result_path.display().to_string() }),
        );
    }
    let script_run_id = script_result.as_ref().map(|r| r.run_id);
    if let Some(sr) = script_result.as_ref() {
        if sr.updated_unix_ms == 0 {
            push_finding(
                &mut findings,
                ArtifactLintLevel::Warning,
                "script_result.updated_unix_ms_missing_or_zero",
                "script.result.json updated_unix_ms is missing or zero",
                serde_json::json!({ "path": script_result_path.display().to_string() }),
            );
        } else {
            let now = crate::util::now_unix_ms();
            const FUTURE_SKEW_MS: u64 = 24 * 60 * 60 * 1000;
            if sr.updated_unix_ms > now.saturating_add(FUTURE_SKEW_MS) {
                push_finding(
                    &mut findings,
                    ArtifactLintLevel::Warning,
                    "script_result.updated_unix_ms_in_future",
                    "script.result.json updated_unix_ms is far in the future (clock skew?)",
                    serde_json::json!({
                        "updated_unix_ms": sr.updated_unix_ms,
                        "now_unix_ms": now,
                    }),
                );
            }
        }

        if generated_unix_ms != 0
            && sr.updated_unix_ms != 0
            && generated_unix_ms.saturating_add(10_000) < sr.updated_unix_ms
        {
            push_finding(
                &mut findings,
                ArtifactLintLevel::Warning,
                "timestamps.unexpected_order",
                "manifest.generated_unix_ms is far behind script.result.json updated_unix_ms",
                serde_json::json!({
                    "generated_unix_ms": generated_unix_ms,
                    "script_updated_unix_ms": sr.updated_unix_ms,
                }),
            );
        }
    }

    let manifest_script_updated = manifest
        .get("script_result")
        .and_then(|v| v.get("updated_unix_ms"))
        .and_then(|v| v.as_u64());
    if let (Some(a), Some(b)) = (
        manifest_script_updated,
        script_result.as_ref().map(|r| r.updated_unix_ms),
    ) && a != 0
        && b != 0
        && a != b
    {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Warning,
            "timestamps.script_result_summary_mismatch",
            "manifest.script_result.updated_unix_ms does not match script.result.json updated_unix_ms",
            serde_json::json!({
                "manifest_script_updated_unix_ms": a,
                "script_updated_unix_ms": b,
                "manifest_path": manifest_path.display().to_string(),
                "script_result_path": script_result_path.display().to_string(),
            }),
        );
    }

    if let (Some(a), Some(b)) = (manifest_run_id, script_run_id)
        && a != b
    {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "run_id.mismatch_manifest_vs_script_result",
            "manifest.run_id does not match script.result.json run_id",
            serde_json::json!({
                "manifest_run_id": a,
                "script_run_id": b,
                "manifest_path": manifest_path.display().to_string(),
                "script_result_path": script_result_path.display().to_string(),
            }),
        );
    }
    if let (Some(a), Some(b)) = (dir_run_id, manifest_run_id)
        && a != b
    {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Error,
            "run_id.mismatch_dir_vs_manifest",
            "artifact dir name does not match manifest.run_id",
            serde_json::json!({
                "dir_run_id": a,
                "manifest_run_id": b,
                "artifact_dir": dir.display().to_string(),
                "manifest_path": manifest_path.display().to_string(),
            }),
        );
    }

    if let Some(files) = manifest.get("files").and_then(|v| v.as_array()) {
        for f in files {
            let Some(rel) = f.get("path").and_then(|v| v.as_str()) else {
                continue;
            };
            let expected_bytes = f.get("bytes").and_then(|v| v.as_u64());
            let expected_blake3 = f
                .get("blake3")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let path = manifest_dir.join(rel);
            if !path.is_file() {
                push_finding(
                    &mut findings,
                    ArtifactLintLevel::Error,
                    "file.missing",
                    format!("file referenced by manifest does not exist: {rel}"),
                    serde_json::json!({ "path": path.display().to_string() }),
                );
                continue;
            }
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            if let Some(expected) = expected_bytes
                && expected != bytes.len() as u64
            {
                push_finding(
                    &mut findings,
                    ArtifactLintLevel::Error,
                    "file.bytes_mismatch",
                    format!("file size mismatch for {rel}"),
                    serde_json::json!({
                        "path": path.display().to_string(),
                        "expected_bytes": expected,
                        "got_bytes": bytes.len(),
                    }),
                );
            }
            if let Some(expected) = expected_blake3 {
                let actual = blake3_hex(&bytes);
                if actual != expected {
                    push_finding(
                        &mut findings,
                        ArtifactLintLevel::Error,
                        "file.hash_mismatch",
                        format!("file hash mismatch for {rel}"),
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "expected_blake3": expected,
                            "got_blake3": actual,
                        }),
                    );
                }
            }
        }
    } else {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Warning,
            "manifest.files_missing",
            "manifest has no files[] index; file integrity checks are partial",
            serde_json::json!({ "path": manifest_path.display().to_string() }),
        );
    }

    let bundle_json = manifest.get("bundle_json");
    if bundle_json.is_none() {
        push_finding(
            &mut findings,
            ArtifactLintLevel::Info,
            "bundle_json.not_indexed",
            "manifest has no bundle_json chunk index (this is expected for runs that never captured a bundle)",
            serde_json::json!({}),
        );
    }

    if let Some(bundle_json) = bundle_json {
        let chunks = bundle_json.get("chunks").and_then(|v| v.as_array());
        let expected_total_bytes = bundle_json.get("total_bytes").and_then(|v| v.as_u64());
        let expected_total_blake3 = bundle_json
            .get("blake3")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(chunks) = chunks {
            if chunks.is_empty() {
                push_finding(
                    &mut findings,
                    ArtifactLintLevel::Warning,
                    "bundle_json.chunks_empty",
                    "manifest bundle_json index has an empty chunks[] list",
                    serde_json::json!({ "manifest_path": manifest_path.display().to_string() }),
                );
            } else {
                let mut total_hasher = blake3::Hasher::new();
                let mut total_bytes: u64 = 0;
                for c in chunks {
                    let Some(rel) = c.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let expected_chunk_bytes = c.get("bytes").and_then(|v| v.as_u64());
                    let expected_chunk_blake3 = c
                        .get("blake3")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let path = manifest_dir.join(rel);
                    if !path.is_file() {
                        push_finding(
                            &mut findings,
                            ArtifactLintLevel::Error,
                            "bundle_json.chunk_missing",
                            "bundle json chunk is missing",
                            serde_json::json!({ "path": path.display().to_string() }),
                        );
                        continue;
                    }
                    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
                    total_bytes = total_bytes.saturating_add(bytes.len() as u64);
                    if let Some(expected) = expected_chunk_bytes
                        && expected != bytes.len() as u64
                    {
                        push_finding(
                            &mut findings,
                            ArtifactLintLevel::Error,
                            "bundle_json.chunk_bytes_mismatch",
                            "bundle json chunk size mismatch",
                            serde_json::json!({
                                "path": path.display().to_string(),
                                "expected_bytes": expected,
                                "got_bytes": bytes.len(),
                            }),
                        );
                    }
                    if let Some(expected) = expected_chunk_blake3 {
                        let actual = blake3_hex(&bytes);
                        if actual != expected {
                            push_finding(
                                &mut findings,
                                ArtifactLintLevel::Error,
                                "bundle_json.chunk_hash_mismatch",
                                "bundle json chunk hash mismatch",
                                serde_json::json!({
                                    "path": path.display().to_string(),
                                    "expected_blake3": expected,
                                    "got_blake3": actual,
                                }),
                            );
                        }
                    }
                    total_hasher.update(&bytes);
                }

                if let Some(expected) = expected_total_bytes
                    && expected != total_bytes
                {
                    push_finding(
                        &mut findings,
                        ArtifactLintLevel::Warning,
                        "bundle_json.total_bytes_mismatch",
                        "bundle json total_bytes does not match the sum of chunk sizes",
                        serde_json::json!({
                            "expected_total_bytes": expected,
                            "got_total_bytes": total_bytes,
                        }),
                    );
                }

                if let Some(expected) = expected_total_blake3 {
                    let actual = total_hasher.finalize().to_hex().to_string();
                    if actual != expected {
                        push_finding(
                            &mut findings,
                            ArtifactLintLevel::Error,
                            "bundle_json.total_hash_mismatch",
                            "bundle json total blake3 does not match recomputed value",
                            serde_json::json!({
                                "expected_blake3": expected,
                                "got_blake3": actual,
                            }),
                        );
                    }
                }
            }
        } else {
            push_finding(
                &mut findings,
                ArtifactLintLevel::Warning,
                "bundle_json.chunks_missing",
                "manifest bundle_json index has no chunks[] list",
                serde_json::json!({ "manifest_path": manifest_path.display().to_string() }),
            );
        }
    }

    // Sidecars are best-effort in the per-run directory (they are written when `bundle.json` is aliased).
    // Treat them as warnings here to avoid failing dump-only runs (poke/record-run).
    lint_sidecar_basic(
        &mut findings,
        &manifest_dir,
        "bundle_meta",
        "bundle.meta.json",
        warmup_frames,
    );
    lint_sidecar_basic(
        &mut findings,
        &manifest_dir,
        "bundle_index",
        "bundle.index.json",
        warmup_frames,
    );
    lint_sidecar_basic(
        &mut findings,
        &manifest_dir,
        "frames_index",
        "frames.index.json",
        warmup_frames,
    );
    lint_sidecar_basic(
        &mut findings,
        &manifest_dir,
        "test_ids_index",
        "test_ids.index.json",
        warmup_frames,
    );

    // Re-count from findings for consistency.
    let (errors, warnings) = counts_from_findings(&findings);

    let ok = errors == 0;
    let payload = serde_json::json!({
        "kind": "diag_artifact_lint",
        "schema_version": 1,
        "ok": ok,
        "artifact_dir": dir.display().to_string(),
        "manifest_path": manifest_path.display().to_string(),
        "manifest_dir": manifest_dir.display().to_string(),
        "dir_run_id": dir_run_id,
        "manifest_run_id": manifest_run_id,
        "script_run_id": script_run_id,
        "warmup_frames": warmup_frames,
        "error_issues": errors,
        "warning_issues": warnings,
        "findings": findings,
    });
    Ok(ArtifactLintReport {
        error_issues: errors,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag_protocol::{UiScriptEvidenceV1, UiScriptStageV1};

    fn write_json(path: &Path, v: &Value) {
        let bytes = serde_json::to_vec_pretty(v).expect("json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn artifact_lint_passes_on_basic_manifest_and_script_result() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-lint-basic-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let run_id: u64 = 42;
        let dir = root.join(run_id.to_string());
        std::fs::create_dir_all(&dir).unwrap();

        let script_result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let bytes = serde_json::to_vec_pretty(&script_result).unwrap();
        std::fs::write(dir.join("script.result.json"), &bytes).unwrap();

        let manifest = serde_json::json!({
            "schema_version": 2,
            "generated_unix_ms": 2,
            "run_id": run_id,
            "paths": { "script_result": "script.result.json", "bundle_artifact": "bundle.json" },
            "script_result": { "stage": "passed", "reason_code": null, "updated_unix_ms": script_result.updated_unix_ms },
            "files": [
                { "id": "script_result", "path": "script.result.json", "bytes": bytes.len(), "blake3": blake3_hex(&bytes) }
            ]
        });
        write_json(&dir.join("manifest.json"), &manifest);

        let report = lint_run_artifact_dir(&dir, 0).unwrap();
        assert_eq!(report.error_issues, 0);
        assert!(
            report
                .payload
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_lint_reports_run_id_mismatch() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-lint-mismatch-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let run_id: u64 = 7;
        let dir = root.join(run_id.to_string());
        std::fs::create_dir_all(&dir).unwrap();

        let script_result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        std::fs::write(
            dir.join("script.result.json"),
            serde_json::to_vec_pretty(&script_result).unwrap(),
        )
        .unwrap();

        let manifest = serde_json::json!({
            "schema_version": 2,
            "generated_unix_ms": 2,
            "run_id": run_id + 1,
            "paths": { "script_result": "script.result.json", "bundle_artifact": "bundle.json" },
            "script_result": { "stage": "passed", "reason_code": null, "updated_unix_ms": 1 },
            "files": [],
        });
        write_json(&dir.join("manifest.json"), &manifest);

        let report = lint_run_artifact_dir(&dir, 0).unwrap();
        assert!(report.error_issues > 0);
        assert!(
            !report
                .payload
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_lint_reports_missing_bundle_chunks() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-lint-chunks-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let run_id: u64 = 99;
        let dir = root.join(run_id.to_string());
        std::fs::create_dir_all(&dir).unwrap();

        let script_result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        std::fs::write(
            dir.join("script.result.json"),
            serde_json::to_vec_pretty(&script_result).unwrap(),
        )
        .unwrap();

        let manifest = serde_json::json!({
            "schema_version": 2,
            "generated_unix_ms": 2,
            "run_id": run_id,
            "paths": { "script_result": "script.result.json", "bundle_artifact": "bundle.json" },
            "bundle_json": {
                "mode": "chunks.v1",
                "total_bytes": 1,
                "chunk_bytes": 1,
                "blake3": "deadbeef",
                "chunks": [
                    { "index": 0, "path": "bundle.json.chunks/00000.bin", "bytes": 1, "blake3": "deadbeef" }
                ]
            },
            "files": []
        });
        write_json(&dir.join("manifest.json"), &manifest);

        let report = lint_run_artifact_dir(&dir, 0).unwrap();
        assert!(report.error_issues > 0);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_lint_accepts_legacy_manifest_path_aliases() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-artifact-lint-legacy-paths-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let run_id: u64 = 314;
        let dir = root.join(run_id.to_string());
        std::fs::create_dir_all(&dir).unwrap();

        let script_result = UiScriptResultV1 {
            schema_version: 1,
            run_id,
            updated_unix_ms: 1,
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let bytes = serde_json::to_vec_pretty(&script_result).unwrap();
        std::fs::write(dir.join("script.result.json"), &bytes).unwrap();

        let manifest = serde_json::json!({
            "schema_version": 2,
            "generated_unix_ms": 2,
            "run_id": run_id,
            "paths": { "script_result_json": "script.result.json", "bundle_json": "bundle.json" },
            "script_result": { "stage": "passed", "reason_code": null, "updated_unix_ms": script_result.updated_unix_ms },
            "files": [
                { "id": "script_result_json", "path": "script.result.json", "bytes": bytes.len(), "blake3": blake3_hex(&bytes) }
            ]
        });
        write_json(&dir.join("manifest.json"), &manifest);

        let report = lint_run_artifact_dir(&dir, 0).unwrap();
        assert_eq!(report.error_issues, 0);

        let _ = std::fs::remove_dir_all(&root);
    }
}
