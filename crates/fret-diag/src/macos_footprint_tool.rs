use std::path::Path;
use std::process::{Command, Stdio};

use crate::util::{now_unix_ms, read_json_value};

fn u64_from_json(v: &serde_json::Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok()))
}

fn bytes_from_value_with_unit(v: &serde_json::Value, bytes_per_unit: u64) -> Option<u64> {
    u64_from_json(v).map(|n| n.saturating_mul(bytes_per_unit.max(1)))
}

fn category_entry_from_json(
    entry: &serde_json::Value,
    bytes_per_unit: u64,
) -> Option<serde_json::Value> {
    let obj = entry.as_object()?;
    Some(serde_json::json!({
        "dirty_bytes": obj.get("dirty").and_then(|v| bytes_from_value_with_unit(v, bytes_per_unit)),
        "swapped_bytes": obj.get("swapped").and_then(|v| bytes_from_value_with_unit(v, bytes_per_unit)),
        "clean_bytes": obj.get("clean").and_then(|v| bytes_from_value_with_unit(v, bytes_per_unit)),
        "reclaimable_bytes": obj.get("reclaimable").and_then(|v| bytes_from_value_with_unit(v, bytes_per_unit)),
        "wired_bytes": obj.get("wired").and_then(|v| bytes_from_value_with_unit(v, bytes_per_unit)),
        "regions": obj.get("regions").and_then(u64_from_json),
    }))
}

pub(crate) fn collect_macos_footprint_tool_best_effort(
    pid: u32,
    out_dir: &Path,
    footprint_file: &str,
) -> Option<serde_json::Value> {
    if footprint_file.trim().is_empty() {
        return None;
    }

    let out_path = out_dir.join(footprint_file);
    let out = Command::new("/usr/bin/footprint")
        .args([
            "--pid",
            &pid.to_string(),
            "--format",
            "bytes",
            "--json",
            out_path.to_string_lossy().as_ref(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }

    let v = read_json_value(&out_path)?;
    let bytes_per_unit = v.get("bytes per unit").and_then(u64_from_json).unwrap_or(1);
    let proc0 = v
        .get("processes")
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let total_footprint_bytes = proc0
        .get("footprint")
        .and_then(|n| bytes_from_value_with_unit(n, bytes_per_unit));

    let phys_footprint_bytes = proc0
        .get("auxiliary")
        .and_then(|a| a.get("phys_footprint"))
        .and_then(|n| bytes_from_value_with_unit(n, bytes_per_unit));
    let phys_footprint_peak_bytes = proc0
        .get("auxiliary")
        .and_then(|a| a.get("phys_footprint_peak"))
        .and_then(|n| bytes_from_value_with_unit(n, bytes_per_unit));

    let mut categories_out = serde_json::Map::new();
    if let Some(categories) = proc0.get("categories").and_then(|c| c.as_object()) {
        for (k, entry) in categories {
            if let Some(v) = category_entry_from_json(entry, bytes_per_unit) {
                categories_out.insert(k.clone(), v);
            }
        }
    }

    Some(serde_json::json!({
        "collector": "footprint --pid <pid> --format bytes",
        "captured_unix_ms": now_unix_ms(),
        "footprint_file": footprint_file,
        "pid": pid,
        "total_footprint_bytes": total_footprint_bytes,
        "phys_footprint_bytes": phys_footprint_bytes,
        "phys_footprint_peak_bytes": phys_footprint_peak_bytes,
        "categories": categories_out,
        "note": "best-effort; output is tool-collected at a point in time.",
    }))
}
