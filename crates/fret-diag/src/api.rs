use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct CompareOptionsV1 {
    pub warmup_frames: u64,
    pub eps_px: f32,
    pub ignore_bounds: bool,
    pub ignore_scene_fingerprint: bool,
}

impl Default for CompareOptionsV1 {
    fn default() -> Self {
        Self {
            warmup_frames: 0,
            eps_px: 0.5,
            ignore_bounds: false,
            ignore_scene_fingerprint: false,
        }
    }
}

pub fn compare_bundles_to_json(
    a_bundle_json_path: &Path,
    b_bundle_json_path: &Path,
    opts: CompareOptionsV1,
) -> Result<serde_json::Value, String> {
    let report = crate::compare::compare_bundles(
        a_bundle_json_path,
        b_bundle_json_path,
        crate::compare::CompareOptions {
            warmup_frames: opts.warmup_frames,
            eps_px: opts.eps_px,
            ignore_bounds: opts.ignore_bounds,
            ignore_scene_fingerprint: opts.ignore_scene_fingerprint,
        },
    )?;
    Ok(report.to_json())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_bundles_to_json_ok_for_identical_minimal() {
        let dir = std::env::temp_dir().join(format!(
            "fret-diag-api-compare-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");

        let a_path = dir.join("a.bundle.json");
        let b_path = dir.join("b.bundle.json");

        let bundle = serde_json::json!({
          "schema_version": 1,
          "windows": [{
            "window": 1,
            "snapshots": [{
              "tick_id": 1,
              "frame_id": 1,
              "debug": {
                "scene_fingerprint": 1,
                "semantics": null
              }
            }]
          }]
        });
        std::fs::write(&a_path, serde_json::to_vec(&bundle).unwrap()).expect("write a");
        std::fs::write(&b_path, serde_json::to_vec(&bundle).unwrap()).expect("write b");

        let report = compare_bundles_to_json(&a_path, &b_path, CompareOptionsV1::default())
            .expect("compare ok");
        assert_eq!(report.get("ok").and_then(|v| v.as_bool()), Some(true));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
