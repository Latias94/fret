use super::*;

#[test]
fn wheel_scroll_streaming_passes_when_hit_moves_outside_target() {
    let mut dir = std::env::temp_dir();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    dir.push(format!(
        "fret-diag-wheel-scroll-streaming-test-{}-{}",
        std::process::id(),
        ts
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let bundle_path = crate::resolve_bundle_artifact_path(&dir);
    std::fs::write(
        &bundle_path,
        r#"{
  "schema_version": 1,
  "windows": [{
    "window": 1,
    "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
    "snapshots": [
      {
        "frame_id": 0,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]}
        }
      },
      {
        "frame_id": 1,
        "debug": {
          "hit_test": { "hit": 3 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 },
            { "id": 3, "parent": 99 }
          ]}
        }
      }
    ]
  }]
}"#,
    )
    .expect("write bundle");

    check_bundle_for_wheel_scroll_streaming(&bundle_path, "root", 0)
        .expect("expected wheel scroll check to pass");
}

#[test]
fn wheel_scroll_hit_changes_streaming_passes_when_offset_changes() {
    let mut dir = std::env::temp_dir();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    dir.push(format!(
        "fret-diag-wheel-scroll-hit-changes-streaming-test-{}-{}",
        std::process::id(),
        ts
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let bundle_path = crate::resolve_bundle_artifact_path(&dir);
    std::fs::write(
        &bundle_path,
        r#"{
  "schema_version": 1,
  "windows": [{
    "window": 1,
    "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
    "snapshots": [
      {
        "frame_id": 0,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]},
          "virtual_list_windows": [{ "offset": 0.0 }]
        }
      },
      {
        "frame_id": 1,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]},
          "virtual_list_windows": [{ "offset": 12.0 }]
        }
      }
    ]
  }]
}"#,
    )
    .expect("write bundle");

    check_bundle_for_wheel_scroll_hit_changes_streaming(&bundle_path, "root", 0)
        .expect("expected wheel scroll hit-change check to pass");
}
