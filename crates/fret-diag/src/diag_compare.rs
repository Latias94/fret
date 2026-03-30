use super::*;
use crate::commands::resolve;
use crate::util::read_json_value;

#[derive(Debug, Clone)]
pub(crate) struct CompareCmdContext {
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub warmup_frames: u64,
    pub compare_eps_px: f32,
    pub compare_ignore_bounds: bool,
    pub compare_ignore_scene_fingerprint: bool,
    pub compare_footprint: bool,
    pub stats_json: bool,
}

pub(crate) fn cmd_compare(ctx: CompareCmdContext) -> Result<(), String> {
    let CompareCmdContext {
        rest,
        workspace_root,
        warmup_frames,
        compare_eps_px,
        compare_ignore_bounds,
        compare_ignore_scene_fingerprint,
        compare_footprint,
        stats_json,
    } = ctx;

    let Some(a_src) = rest.first().cloned() else {
        return Err(
            "missing bundle A path (try: fretboard diag compare <base_or_session_out_dir|bundle_a_dir|bundle_a.json|bundle_a.schema2.json> <base_or_session_out_dir|bundle_b_dir|bundle_b.json|bundle_b.schema2.json>)"
                .to_string(),
        );
    };
    let Some(b_src) = rest.get(1).cloned() else {
        return Err(
            "missing bundle B path (try: fretboard diag compare <base_or_session_out_dir|bundle_a_dir|bundle_a.json|bundle_a.schema2.json> <base_or_session_out_dir|bundle_b_dir|bundle_b.json|bundle_b.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 2 {
        return Err(format!("unexpected arguments: {}", rest[2..].join(" ")));
    }

    let a_src = resolve_path(&workspace_root, PathBuf::from(a_src));
    let b_src = resolve_path(&workspace_root, PathBuf::from(b_src));

    if compare_footprint {
        return cmd_compare_resource_footprint(&a_src, &b_src, stats_json);
    }

    let a_resolved = resolve::resolve_bundle_ref(&a_src)?;
    let b_resolved = resolve::resolve_bundle_ref(&b_src)?;
    let a_bundle_path = a_resolved.bundle_artifact;
    let b_bundle_path = b_resolved.bundle_artifact;

    let report = compare_bundles(
        &a_bundle_path,
        &b_bundle_path,
        CompareOptions {
            warmup_frames,
            eps_px: compare_eps_px,
            ignore_bounds: compare_ignore_bounds,
            ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
        },
    )?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report.to_json()).unwrap_or_else(|_| "{}".to_string())
        );
        if !report.ok {
            std::process::exit(1);
        }
        Ok(())
    } else if report.ok {
        report.print_human();
        Ok(())
    } else {
        Err(report.to_human_error())
    }
}

#[derive(Debug, Clone)]
struct FootprintSummary {
    physical_footprint_bytes: Option<u64>,
    physical_footprint_peak_bytes: Option<u64>,
    top_dirty_region_type: Option<String>,
    top_dirty_region_bytes: Option<u64>,
    top_allocated_malloc_zone: Option<String>,
    top_allocated_malloc_bytes: Option<u64>,
    top_allocated_malloc_frag_bytes: Option<u64>,
    top_allocated_malloc_frag_percent: Option<f64>,
}

#[derive(Debug, Clone)]
struct FootprintDelta {
    physical_footprint_peak_delta_bytes: Option<i64>,
    top_dirty_region_delta_bytes: Option<i64>,
    top_allocated_malloc_delta_bytes: Option<i64>,
    top_allocated_malloc_frag_delta_bytes: Option<i64>,
}

#[derive(Debug, Clone)]
struct FootprintCompareReport {
    schema_version: u64,
    a_out_dir: String,
    b_out_dir: String,
    a: FootprintSummary,
    b: FootprintSummary,
    delta: FootprintDelta,
    top_region_dirty_deltas: Vec<serde_json::Value>,
    top_malloc_alloc_deltas: Vec<serde_json::Value>,
}

fn cmd_compare_resource_footprint(a_src: &Path, b_src: &Path, json: bool) -> Result<(), String> {
    let a_session = resolve_session_root_for_resource_footprint(a_src);
    let b_session = resolve_session_root_for_resource_footprint(b_src);

    let a_fp_path = a_session.join("resource.footprint.json");
    let b_fp_path = b_session.join("resource.footprint.json");
    if !a_fp_path.is_file() {
        return Err(format!(
            "missing resource footprint: {}\n\
hint: run via `fretboard diag repro ... --session-auto --launch -- <cmd>` so the session root includes `resource.footprint.json`",
            a_fp_path.display()
        ));
    }
    if !b_fp_path.is_file() {
        return Err(format!(
            "missing resource footprint: {}\n\
hint: run via `fretboard diag repro ... --session-auto --launch -- <cmd>` so the session root includes `resource.footprint.json`",
            b_fp_path.display()
        ));
    }

    let a_v = read_json_value(&a_fp_path);
    let b_v = read_json_value(&b_fp_path);
    let a_v = a_v.ok_or_else(|| format!("failed to parse JSON: {}", a_fp_path.display()))?;
    let b_v = b_v.ok_or_else(|| format!("failed to parse JSON: {}", b_fp_path.display()))?;

    let a_summary = summarize_resource_footprint(&a_v);
    let b_summary = summarize_resource_footprint(&b_v);

    let top_region_dirty_deltas = compare_top_region_dirty_deltas(&a_v, &b_v, 12);
    let top_malloc_alloc_deltas = compare_top_malloc_alloc_deltas(&a_v, &b_v, 12);

    let delta = FootprintDelta {
        physical_footprint_peak_delta_bytes: opt_delta_i64(
            a_summary.physical_footprint_peak_bytes,
            b_summary.physical_footprint_peak_bytes,
        ),
        top_dirty_region_delta_bytes: opt_delta_i64(
            a_summary.top_dirty_region_bytes,
            b_summary.top_dirty_region_bytes,
        ),
        top_allocated_malloc_delta_bytes: opt_delta_i64(
            a_summary.top_allocated_malloc_bytes,
            b_summary.top_allocated_malloc_bytes,
        ),
        top_allocated_malloc_frag_delta_bytes: opt_delta_i64(
            a_summary.top_allocated_malloc_frag_bytes,
            b_summary.top_allocated_malloc_frag_bytes,
        ),
    };

    let report = FootprintCompareReport {
        schema_version: 1,
        a_out_dir: a_session.display().to_string(),
        b_out_dir: b_session.display().to_string(),
        a: a_summary,
        b: b_summary,
        delta,
        top_region_dirty_deltas,
        top_malloc_alloc_deltas,
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": report.schema_version,
                "a_out_dir": report.a_out_dir,
                "b_out_dir": report.b_out_dir,
                "a": footprint_summary_to_json(&report.a),
                "b": footprint_summary_to_json(&report.b),
                "delta": {
                    "physical_footprint_peak_delta_bytes": report.delta.physical_footprint_peak_delta_bytes,
                    "top_dirty_region_delta_bytes": report.delta.top_dirty_region_delta_bytes,
                    "top_allocated_malloc_delta_bytes": report.delta.top_allocated_malloc_delta_bytes,
                    "top_allocated_malloc_frag_delta_bytes": report.delta.top_allocated_malloc_frag_delta_bytes,
                },
                "top_region_dirty_deltas": report.top_region_dirty_deltas,
                "top_malloc_alloc_deltas": report.top_malloc_alloc_deltas,
            }))
            .unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    print_footprint_compare_human(&report);
    Ok(())
}

fn resolve_session_root_for_resource_footprint(src: &Path) -> PathBuf {
    if src.is_dir() && src.join("resource.footprint.json").is_file() {
        return src.to_path_buf();
    }

    if src.is_file()
        && let Some(parent) = src.parent() {
            if parent.join("resource.footprint.json").is_file() {
                return parent.to_path_buf();
            }
            // If the user pointed at a bundle artifact, it lives under a bundle export dir, which
            // lives under the session root.
            if let Some(grand) = parent.parent()
                && grand.join("resource.footprint.json").is_file() {
                    return grand.to_path_buf();
                }
        }

    if src.is_dir() {
        // Common case: user provided a base out dir or session out dir; resolve to the latest bundle
        // dir and then climb back to the session root (which holds `resource.footprint.json`).
        let bundle_dir = resolve::resolve_base_or_session_out_dir_to_latest_bundle_dir_or_self(src);
        if let Some(session_root) = bundle_dir.parent()
            && session_root.join("resource.footprint.json").is_file() {
                return session_root.to_path_buf();
            }
    }

    // Fallback: return the input directory so the error message can reference a useful path.
    src.to_path_buf()
}

fn summarize_resource_footprint(v: &serde_json::Value) -> FootprintSummary {
    let physical_footprint_bytes = v
        .pointer("/macos_vmmap/physical_footprint_bytes")
        .and_then(|v| v.as_u64());
    let physical_footprint_peak_bytes = v
        .pointer("/macos_vmmap/physical_footprint_peak_bytes")
        .and_then(|v| v.as_u64());

    let (top_dirty_region_type, top_dirty_region_bytes) = v
        .pointer("/macos_vmmap/tables/regions/top_dirty/0")
        .and_then(|v| v.as_object())
        .map(|o| {
            let ty = o
                .get("region_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let dirty = o.get("dirty_bytes").and_then(|v| v.as_u64());
            (ty, dirty)
        })
        .unwrap_or((None, None));

    let (
        top_allocated_malloc_zone,
        top_allocated_malloc_bytes,
        top_allocated_malloc_frag_bytes,
        top_allocated_malloc_frag_percent,
    ) = v
        .pointer("/macos_vmmap/tables/malloc_zones/top_allocated/0")
        .and_then(|v| v.as_object())
        .map(|o| {
            let zone = o
                .get("zone")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let allocated = o.get("allocated_bytes").and_then(|v| v.as_u64());
            let frag = o.get("frag_bytes").and_then(|v| v.as_u64());
            let frag_percent = o.get("frag_percent").and_then(|v| v.as_f64());
            (zone, allocated, frag, frag_percent)
        })
        .unwrap_or((None, None, None, None));

    FootprintSummary {
        physical_footprint_bytes,
        physical_footprint_peak_bytes,
        top_dirty_region_type,
        top_dirty_region_bytes,
        top_allocated_malloc_zone,
        top_allocated_malloc_bytes,
        top_allocated_malloc_frag_bytes,
        top_allocated_malloc_frag_percent,
    }
}

fn footprint_summary_to_json(s: &FootprintSummary) -> serde_json::Value {
    serde_json::json!({
        "physical_footprint_bytes": s.physical_footprint_bytes,
        "physical_footprint_peak_bytes": s.physical_footprint_peak_bytes,
        "top_dirty_region_type": s.top_dirty_region_type,
        "top_dirty_region_bytes": s.top_dirty_region_bytes,
        "top_allocated_malloc_zone": s.top_allocated_malloc_zone,
        "top_allocated_malloc_bytes": s.top_allocated_malloc_bytes,
        "top_allocated_malloc_frag_bytes": s.top_allocated_malloc_frag_bytes,
        "top_allocated_malloc_frag_percent": s.top_allocated_malloc_frag_percent,
    })
}

fn compare_top_region_dirty_deltas(
    a_v: &serde_json::Value,
    b_v: &serde_json::Value,
    top_n: usize,
) -> Vec<serde_json::Value> {
    let a = collect_region_stats(a_v);
    let b = collect_region_stats(b_v);

    let mut keys: Vec<String> = a
        .keys()
        .chain(b.keys())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    keys.sort();

    let mut rows: Vec<(i64, serde_json::Value)> = Vec::new();
    for k in keys {
        let a_dirty = a.get(&k).map(|r| r.dirty_bytes).unwrap_or(0);
        let b_dirty = b.get(&k).map(|r| r.dirty_bytes).unwrap_or(0);
        let delta = b_dirty as i64 - a_dirty as i64;
        if delta == 0 {
            continue;
        }

        rows.push((
            delta,
            serde_json::json!({
                "region_type": k,
                "a_dirty_bytes": a_dirty,
                "b_dirty_bytes": b_dirty,
                "delta_dirty_bytes": delta,
            }),
        ));
    }

    rows.sort_by_key(|(delta, row)| {
        let abs = delta.unsigned_abs();
        let a_dirty = row
            .get("a_dirty_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let b_dirty = row
            .get("b_dirty_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        (
            std::cmp::Reverse(abs),
            std::cmp::Reverse(a_dirty.max(b_dirty)),
        )
    });
    rows.into_iter().take(top_n).map(|(_, v)| v).collect()
}

fn compare_top_malloc_alloc_deltas(
    a_v: &serde_json::Value,
    b_v: &serde_json::Value,
    top_n: usize,
) -> Vec<serde_json::Value> {
    let a = collect_malloc_zone_stats(a_v);
    let b = collect_malloc_zone_stats(b_v);

    let mut keys: Vec<String> = a
        .keys()
        .chain(b.keys())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    keys.sort();

    let mut rows: Vec<(i64, serde_json::Value)> = Vec::new();
    for k in keys {
        let a_alloc = a.get(&k).map(|r| r.allocated_bytes).unwrap_or(0);
        let b_alloc = b.get(&k).map(|r| r.allocated_bytes).unwrap_or(0);
        let a_frag = a.get(&k).map(|r| r.frag_bytes).unwrap_or(0);
        let b_frag = b.get(&k).map(|r| r.frag_bytes).unwrap_or(0);
        let delta = b_alloc as i64 - a_alloc as i64;
        let delta_frag = b_frag as i64 - a_frag as i64;
        if delta == 0 && delta_frag == 0 {
            continue;
        }

        rows.push((
            delta,
            serde_json::json!({
                "zone": k,
                "a_allocated_bytes": a_alloc,
                "b_allocated_bytes": b_alloc,
                "delta_allocated_bytes": delta,
                "a_frag_bytes": a_frag,
                "b_frag_bytes": b_frag,
                "delta_frag_bytes": delta_frag,
            }),
        ));
    }

    rows.sort_by_key(|(delta, row)| {
        let abs = delta.unsigned_abs();
        let a_alloc = row
            .get("a_allocated_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let b_alloc = row
            .get("b_allocated_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        (
            std::cmp::Reverse(abs),
            std::cmp::Reverse(a_alloc.max(b_alloc)),
        )
    });
    rows.into_iter().take(top_n).map(|(_, v)| v).collect()
}

#[derive(Debug, Default, Clone, Copy)]
struct RegionRowAgg {
    dirty_bytes: u64,
}

fn collect_region_stats(v: &serde_json::Value) -> std::collections::HashMap<String, RegionRowAgg> {
    let mut out: std::collections::HashMap<String, RegionRowAgg> = std::collections::HashMap::new();
    for ptr in [
        "/macos_vmmap/tables/regions/top_dirty",
        "/macos_vmmap/tables/regions/top_resident",
    ] {
        let Some(rows) = v.pointer(ptr).and_then(|v| v.as_array()) else {
            continue;
        };
        for row in rows {
            let Some(obj) = row.as_object() else {
                continue;
            };
            let Some(region_type) = obj.get("region_type").and_then(|v| v.as_str()) else {
                continue;
            };
            let dirty = obj.get("dirty_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
            out.insert(region_type.to_string(), RegionRowAgg { dirty_bytes: dirty });
        }
    }
    out
}

#[derive(Debug, Default, Clone, Copy)]
struct MallocZoneAgg {
    allocated_bytes: u64,
    frag_bytes: u64,
}

fn collect_malloc_zone_stats(
    v: &serde_json::Value,
) -> std::collections::HashMap<String, MallocZoneAgg> {
    let mut out: std::collections::HashMap<String, MallocZoneAgg> =
        std::collections::HashMap::new();
    for ptr in [
        "/macos_vmmap/tables/malloc_zones/top_allocated",
        "/macos_vmmap/tables/malloc_zones/top_frag",
    ] {
        let Some(rows) = v.pointer(ptr).and_then(|v| v.as_array()) else {
            continue;
        };
        for row in rows {
            let Some(obj) = row.as_object() else {
                continue;
            };
            let Some(zone) = obj.get("zone").and_then(|v| v.as_str()) else {
                continue;
            };
            let allocated = obj
                .get("allocated_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let frag = obj.get("frag_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
            out.insert(
                zone.to_string(),
                MallocZoneAgg {
                    allocated_bytes: allocated,
                    frag_bytes: frag,
                },
            );
        }
    }
    out
}

fn opt_delta_i64(a: Option<u64>, b: Option<u64>) -> Option<i64> {
    Some(b? as i64 - a? as i64)
}

fn human_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    let f = n as f64;
    if f >= GB {
        format!("{:.2}GiB", f / GB)
    } else if f >= MB {
        format!("{:.2}MiB", f / MB)
    } else if f >= KB {
        format!("{:.2}KiB", f / KB)
    } else {
        format!("{n}B")
    }
}

fn print_footprint_compare_human(report: &FootprintCompareReport) {
    println!(
        "resource footprint compare (schema v{}):",
        report.schema_version
    );
    println!("  A: {}", report.a_out_dir);
    println!("  B: {}", report.b_out_dir);
    println!();

    print_summary_line_u64_opt(
        "physical_footprint_peak_bytes",
        report.a.physical_footprint_peak_bytes,
        report.b.physical_footprint_peak_bytes,
    );
    print_summary_line_u64_opt(
        "owned_unmapped_memory_dirty_bytes (top region)",
        report.a.top_dirty_region_bytes,
        report.b.top_dirty_region_bytes,
    );
    if let (Some(a), Some(b)) = (
        report.a.top_dirty_region_type.as_deref(),
        report.b.top_dirty_region_type.as_deref(),
    ) {
        println!("  top_dirty_region_type: A={a} B={b}");
    }

    print_summary_line_u64_opt(
        "DefaultMallocZone allocated_bytes (top zone)",
        report.a.top_allocated_malloc_bytes,
        report.b.top_allocated_malloc_bytes,
    );
    print_summary_line_u64_opt(
        "DefaultMallocZone frag_bytes (top zone)",
        report.a.top_allocated_malloc_frag_bytes,
        report.b.top_allocated_malloc_frag_bytes,
    );
    if let (Some(a), Some(b)) = (
        report.a.top_allocated_malloc_zone.as_deref(),
        report.b.top_allocated_malloc_zone.as_deref(),
    ) {
        println!("  top_allocated_malloc_zone: A={a} B={b}");
    }

    println!();
    if report.top_region_dirty_deltas.is_empty() {
        println!("  region dirty deltas: (none)");
    } else {
        println!("  region dirty deltas (top):");
        for row in report.top_region_dirty_deltas.iter() {
            let ty = row
                .get("region_type")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let delta = row
                .get("delta_dirty_bytes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let a = row
                .get("a_dirty_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let b = row
                .get("b_dirty_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let sign = if delta < 0 { "-" } else { "+" };
            println!(
                "    {ty}: A={} B={} Δ={}{}",
                human_bytes(a),
                human_bytes(b),
                sign,
                human_bytes(delta.unsigned_abs()),
            );
        }
    }

    println!();
    if report.top_malloc_alloc_deltas.is_empty() {
        println!("  malloc zone deltas: (none)");
    } else {
        println!("  malloc zone deltas (top):");
        for row in report.top_malloc_alloc_deltas.iter() {
            let zone = row
                .get("zone")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let delta = row
                .get("delta_allocated_bytes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let a = row
                .get("a_allocated_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let b = row
                .get("b_allocated_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let delta_frag = row
                .get("delta_frag_bytes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let sign = if delta < 0 { "-" } else { "+" };
            let sign_frag = if delta_frag < 0 { "-" } else { "+" };
            println!(
                "    {zone}: alloc A={} B={} Δ={}{} frag Δ={}{}",
                human_bytes(a),
                human_bytes(b),
                sign,
                human_bytes(delta.unsigned_abs()),
                sign_frag,
                human_bytes(delta_frag.unsigned_abs()),
            );
        }
    }
}

fn print_summary_line_u64_opt(label: &str, a: Option<u64>, b: Option<u64>) {
    match (a, b) {
        (Some(a), Some(b)) => {
            let delta = b as i64 - a as i64;
            let delta_abs = delta.unsigned_abs();
            let sign = if delta < 0 { "-" } else { "+" };
            println!(
                "  {label}: A={} B={} Δ={}{}",
                human_bytes(a),
                human_bytes(b),
                sign,
                human_bytes(delta_abs)
            );
        }
        _ => {
            println!("  {label}: A=null B=null");
        }
    }
}
