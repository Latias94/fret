use std::path::{Path, PathBuf};

use crate::commands::resolve;

#[derive(Debug, Clone)]
struct MemorySampleRow {
    sample_id: String,
    out_dir: PathBuf,
    script_path: Option<String>,

    // process_footprint (macOS vmmap-derived, if available)
    killed: Option<bool>,
    macos_physical_footprint_peak_bytes: Option<u64>,
    macos_owned_unmapped_memory_dirty_bytes: Option<u64>,
    macos_io_surface_dirty_bytes: Option<u64>,
    macos_io_accelerator_dirty_bytes: Option<u64>,
    macos_malloc_small_dirty_bytes: Option<u64>,
    macos_malloc_dirty_bytes_total: Option<u64>,
    macos_malloc_zones_total_allocated_bytes: Option<u64>,
    macos_malloc_zones_total_frag_bytes: Option<u64>,

    // bundle_last_frame_stats (if available)
    wgpu_metal_current_allocated_size_bytes_min: Option<u64>,
    wgpu_metal_current_allocated_size_bytes_max: Option<u64>,
    wgpu_allocator_total_allocated_bytes: Option<u64>,
    wgpu_allocator_total_reserved_bytes: Option<u64>,
    render_text_atlas_bytes_live_estimate_total: Option<u64>,
    render_text_registered_font_blobs_total_bytes: Option<u64>,
    render_text_registered_font_blobs_count: Option<u64>,

    // Optional: deeper vmmap attribution hints (macOS-only)
    macos_vmmap_regions_sorted_top_dirty_region_type: Option<String>,
    macos_vmmap_regions_sorted_top_dirty_detail: Option<String>,
    macos_vmmap_regions_sorted_top_dirty_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
struct U64Stats {
    count_present: usize,
    min: u64,
    p50: u64,
    p90: u64,
    max: u64,
}

pub(crate) fn cmd_memory_summary(
    rest: &[String],
    resolved_out_dir: &Path,
    workspace_root: &Path,
    json: bool,
    top_rows: usize,
    out: Option<&Path>,
) -> Result<(), String> {
    let mut target: Option<String> = None;
    let mut within_session: Option<String> = None;
    let mut sort_key: String = "macos_physical_footprint_peak_bytes".to_string();
    let mut include_regions_sorted_top = false;
    let mut top_sessions: Option<usize> = None;
    let mut include_regions_sorted_agg = false;
    let mut regions_sorted_agg_top: usize = 10;
    let mut no_recursive = false;
    let mut recursive_max_depth: usize = 3;
    let mut recursive_max_samples: usize = 200;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--within-session" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --within-session".to_string());
                };
                within_session = Some(v.to_string());
                i += 2;
            }
            "--sort-key" | "--sort_key" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --sort-key".to_string());
                };
                sort_key = v.to_string();
                i += 2;
            }
            "--vmmap-regions-sorted-top" => {
                include_regions_sorted_top = true;
                i += 1;
            }
            "--vmmap-regions-sorted-agg" => {
                include_regions_sorted_agg = true;
                i += 1;
            }
            "--vmmap-regions-sorted-agg-top" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --vmmap-regions-sorted-agg-top".to_string());
                };
                regions_sorted_agg_top = v.parse::<usize>().map_err(|e| {
                    format!("invalid value for --vmmap-regions-sorted-agg-top: {e}")
                })?;
                if regions_sorted_agg_top == 0 {
                    return Err("--vmmap-regions-sorted-agg-top must be >= 1".to_string());
                }
                i += 2;
            }
            "--no-recursive" => {
                no_recursive = true;
                i += 1;
            }
            "--max-depth" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --max-depth".to_string());
                };
                recursive_max_depth = v
                    .parse::<usize>()
                    .map_err(|e| format!("invalid value for --max-depth: {e}"))?;
                i += 2;
            }
            "--max-samples" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --max-samples".to_string());
                };
                recursive_max_samples = v
                    .parse::<usize>()
                    .map_err(|e| format!("invalid value for --max-samples: {e}"))?;
                if recursive_max_samples == 0 {
                    return Err("--max-samples must be >= 1".to_string());
                }
                i += 2;
            }
            "--top-sessions" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --top-sessions".to_string());
                };
                let n = v
                    .parse::<usize>()
                    .map_err(|e| format!("invalid value for --top-sessions: {e}"))?;
                if n == 0 {
                    return Err("--top-sessions must be >= 1".to_string());
                }
                top_sessions = Some(n);
                i += 2;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: fretboard diag memory-summary [<base_or_session_out_dir>] [--within-session <id|latest|all>] [--top-sessions <n>] [--sort-key <key>] [--top <n>] [--vmmap-regions-sorted-top] [--vmmap-regions-sorted-agg] [--vmmap-regions-sorted-agg-top <n>] [--no-recursive] [--max-depth <n>] [--max-samples <n>] [--json] [--out <path>]".to_string(),
                );
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag memory-summary flag: {other}"));
            }
            _ => {
                if target.is_some() {
                    return Err(format!("unexpected arguments: {}", rest[i..].join(" ")));
                }
                target = Some(rest[i].clone());
                i += 1;
            }
        }
    }

    let src = target
        .map(PathBuf::from)
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| resolved_out_dir.to_path_buf());

    validate_sort_key(&sort_key)?;

    let mut sample_dirs = resolve_sample_dirs(&src, within_session.as_deref(), top_sessions)?;
    if sample_dirs.is_empty()
        && src.is_dir()
        && !no_recursive
        && recursive_max_depth > 0
        && recursive_max_samples > 0
    {
        sample_dirs = resolve_sample_dirs_recursive(
            &src,
            within_session.as_deref(),
            top_sessions,
            recursive_max_depth,
            recursive_max_samples,
        )?;
    }
    if sample_dirs.is_empty() {
        return Err(format!(
            "no diagnostics samples found under: {}\n\
hint: point at a session root, a base dir containing `sessions/`, or a parent dir with multiple dated out dirs.\n\
hint: run with `--session-auto` (recommended) so samples appear under `<dir>/sessions/<session_id>/`",
            src.display()
        ));
    }

    let mut rows: Vec<MemorySampleRow> = Vec::new();
    for (sample_id, sample_dir) in sample_dirs {
        if let Some(row) = read_sample_row(&sample_id, &sample_dir, include_regions_sorted_top) {
            rows.push(row);
        }
    }

    if rows.is_empty() {
        return Err(format!(
            "no parseable evidence.index.json under: {}\n\
hint: ensure each session root contains `evidence.index.json`",
            src.display()
        ));
    }

    let report = build_report(
        &src,
        &sort_key,
        top_rows.max(1),
        &rows,
        include_regions_sorted_agg,
        regions_sorted_agg_top.max(1),
    );
    let output_bytes: Vec<u8> = if json {
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?
    } else {
        human_report(&report).into_bytes()
    };

    if let Some(out) = out {
        let out = crate::resolve_path(workspace_root, out.to_path_buf());
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&out, output_bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }

    print!("{}", String::from_utf8_lossy(&output_bytes));
    Ok(())
}

fn resolve_sample_dirs(
    src: &Path,
    within_session: Option<&str>,
    top_sessions: Option<usize>,
) -> Result<Vec<(String, PathBuf)>, String> {
    if src.is_dir() && resolve::looks_like_diag_session_root(src) {
        let id = src
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("session")
            .to_string();
        return Ok(vec![(id, src.to_path_buf())]);
    }

    if src.is_dir() && src.join(crate::session::SESSIONS_DIRNAME).is_dir() {
        let want = within_session.unwrap_or("all").trim();
        let sessions = crate::session::collect_sessions(src)?;
        if sessions.is_empty() {
            return Ok(Vec::new());
        }

        let mut picks: Vec<(String, PathBuf)> = Vec::new();
        if want.is_empty() || want == "all" {
            let n = top_sessions.unwrap_or(usize::MAX);
            for s in sessions.into_iter().take(n) {
                picks.push((s.session_id, s.session_dir));
            }
            return Ok(picks);
        }

        let sid = if want == "latest" {
            sessions
                .first()
                .map(|s| s.session_id.clone())
                .unwrap_or_else(|| "latest".to_string())
        } else {
            crate::session::sanitize_session_id(want)
        };

        let out_dir = crate::session::session_out_dir(src, &sid);
        if !out_dir.is_dir() {
            return Err(format!(
                "session directory does not exist: {}\n\
hint: list sessions via `fretboard diag list sessions --dir {}`",
                out_dir.display(),
                src.display()
            ));
        }
        picks.push((sid, out_dir));
        return Ok(picks);
    }

    if src.is_dir() {
        // A non-session out dir (legacy). Treat it as a single sample if it looks like one.
        if src.join("evidence.index.json").is_file()
            || src.join("resource.footprint.json").is_file()
        {
            return Ok(vec![("out_dir".to_string(), src.to_path_buf())]);
        }
    }

    Ok(Vec::new())
}

fn resolve_sample_dirs_recursive(
    src: &Path,
    within_session: Option<&str>,
    top_sessions: Option<usize>,
    max_depth: usize,
    max_samples: usize,
) -> Result<Vec<(String, PathBuf)>, String> {
    use std::collections::{HashSet, VecDeque};

    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let mut q: VecDeque<(PathBuf, usize)> = VecDeque::new();
    q.push_back((src.to_path_buf(), 0));

    let mut seen_dirs: HashSet<String> = HashSet::new();
    let mut visited_dirs: usize = 0;

    const MAX_VISITED_DIRS: usize = 4000;

    while let Some((dir, depth)) = q.pop_front() {
        if out.len() >= max_samples {
            break;
        }
        if visited_dirs >= MAX_VISITED_DIRS {
            break;
        }
        if !dir.is_dir() {
            continue;
        }
        let key = dir.to_string_lossy().to_string();
        if !seen_dirs.insert(key) {
            continue;
        }
        visited_dirs = visited_dirs.saturating_add(1);

        let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with('.') || name == ".git" || name == "repo-ref" || name == "node_modules" {
            continue;
        }

        if resolve::looks_like_diag_session_root(&dir) {
            let id = rel_sample_id(src, &dir);
            out.push((id, dir));
            continue;
        }

        if dir.join(crate::session::SESSIONS_DIRNAME).is_dir() {
            let base_rel = rel_sample_id(src, &dir);
            let picks = resolve_sample_dirs(&dir, within_session, top_sessions)?;
            for (sid, session_dir) in picks {
                if out.len() >= max_samples {
                    break;
                }
                let id = if base_rel == "." || base_rel.is_empty() {
                    sid
                } else {
                    format!("{base_rel}/sessions/{sid}")
                };
                out.push((id, session_dir));
            }
            continue;
        }

        if dir.join("evidence.index.json").is_file()
            || dir.join("resource.footprint.json").is_file()
        {
            let id = rel_sample_id(src, &dir);
            out.push((id, dir));
            continue;
        }

        if depth >= max_depth {
            continue;
        }

        let iter = match std::fs::read_dir(&dir) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in iter.flatten() {
            if out.len() >= max_samples {
                break;
            }
            let child = entry.path();
            if !child.is_dir() {
                continue;
            }
            q.push_back((child, depth + 1));
        }
    }

    if out.len() > max_samples {
        out.truncate(max_samples);
    }

    Ok(out)
}

fn rel_sample_id(root: &Path, dir: &Path) -> String {
    let rel = dir.strip_prefix(root).unwrap_or(dir);
    let s = rel.to_string_lossy().replace('\\', "/");
    if s.trim().is_empty() {
        ".".to_string()
    } else {
        s
    }
}

fn read_sample_row(
    sample_id: &str,
    sample_dir: &Path,
    include_regions_sorted_top: bool,
) -> Option<MemorySampleRow> {
    let evidence_index_path = sample_dir.join("evidence.index.json");
    let evidence = crate::util::read_json_value(&evidence_index_path)?;
    let resources = evidence.get("resources")?;

    let process = resources
        .get("process_footprint")
        .unwrap_or(&serde_json::Value::Null);
    let bundle = resources
        .get("bundle_last_frame_stats")
        .unwrap_or(&serde_json::Value::Null);

    let get_u64 =
        |v: &serde_json::Value, k: &str| -> Option<u64> { v.get(k).and_then(|v| v.as_u64()) };
    let get_bool =
        |v: &serde_json::Value, k: &str| -> Option<bool> { v.get(k).and_then(|v| v.as_bool()) };
    let get_str = |v: &serde_json::Value, k: &str| -> Option<String> {
        v.get(k).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    let script_path = evidence
        .get("summary")
        .and_then(|v| v.get("scripts"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("script_path"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut out = MemorySampleRow {
        sample_id: sample_id.to_string(),
        out_dir: sample_dir.to_path_buf(),
        script_path,
        killed: get_bool(process, "killed"),
        macos_physical_footprint_peak_bytes: get_u64(
            process,
            "macos_physical_footprint_peak_bytes",
        ),
        macos_owned_unmapped_memory_dirty_bytes: get_u64(
            process,
            "macos_owned_unmapped_memory_dirty_bytes",
        ),
        macos_io_surface_dirty_bytes: get_u64(process, "macos_io_surface_dirty_bytes"),
        macos_io_accelerator_dirty_bytes: get_u64(process, "macos_io_accelerator_dirty_bytes"),
        macos_malloc_small_dirty_bytes: get_u64(process, "macos_malloc_small_dirty_bytes"),
        macos_malloc_dirty_bytes_total: get_u64(process, "macos_malloc_dirty_bytes_total"),
        macos_malloc_zones_total_allocated_bytes: get_u64(
            process,
            "macos_malloc_zones_total_allocated_bytes",
        ),
        macos_malloc_zones_total_frag_bytes: get_u64(
            process,
            "macos_malloc_zones_total_frag_bytes",
        ),
        wgpu_metal_current_allocated_size_bytes_min: get_u64(
            bundle,
            "wgpu_metal_current_allocated_size_bytes_min",
        ),
        wgpu_metal_current_allocated_size_bytes_max: get_u64(
            bundle,
            "wgpu_metal_current_allocated_size_bytes_max",
        ),
        wgpu_allocator_total_allocated_bytes: get_u64(
            bundle,
            "wgpu_allocator_total_allocated_bytes",
        ),
        wgpu_allocator_total_reserved_bytes: get_u64(bundle, "wgpu_allocator_total_reserved_bytes"),
        render_text_atlas_bytes_live_estimate_total: get_u64(
            bundle,
            "render_text_atlas_bytes_live_estimate_total",
        ),
        render_text_registered_font_blobs_total_bytes: get_u64(
            bundle,
            "render_text_registered_font_blobs_total_bytes",
        ),
        render_text_registered_font_blobs_count: get_u64(
            bundle,
            "render_text_registered_font_blobs_count",
        ),
        macos_vmmap_regions_sorted_top_dirty_region_type: None,
        macos_vmmap_regions_sorted_top_dirty_detail: None,
        macos_vmmap_regions_sorted_top_dirty_bytes: None,
    };

    if include_regions_sorted_top {
        let fp_path = sample_dir.join("resource.footprint.json");
        if let Some(v) = crate::util::read_json_value(&fp_path) {
            let top = v
                .get("macos_vmmap_regions_sorted_steady")
                .and_then(|v| v.get("tables"))
                .and_then(|v| v.get("regions"))
                .and_then(|v| v.get("top_dirty"))
                .and_then(|v| v.as_array())
                .and_then(|a| a.first());
            if let Some(top) = top {
                out.macos_vmmap_regions_sorted_top_dirty_region_type = get_str(top, "region_type");
                out.macos_vmmap_regions_sorted_top_dirty_detail = get_str(top, "detail");
                out.macos_vmmap_regions_sorted_top_dirty_bytes = get_u64(top, "dirty_bytes");
            }
        }
    }

    Some(out)
}

fn build_report(
    src: &Path,
    sort_key: &str,
    top: usize,
    rows: &[MemorySampleRow],
    include_regions_sorted_agg: bool,
    regions_sorted_agg_top: usize,
) -> serde_json::Value {
    let mut sorted: Vec<MemorySampleRow> = rows.to_vec();
    sorted.sort_by(|a, b| {
        let av = sort_u64_for_key(a, sort_key);
        let bv = sort_u64_for_key(b, sort_key);
        bv.cmp(&av).then_with(|| a.sample_id.cmp(&b.sample_id))
    });

    let fields = serde_json::json!({
        "macos_physical_footprint_peak_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_physical_footprint_peak_bytes).collect()),
        "macos_owned_unmapped_memory_dirty_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_owned_unmapped_memory_dirty_bytes).collect()),
        "macos_io_surface_dirty_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_io_surface_dirty_bytes).collect()),
        "macos_io_accelerator_dirty_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_io_accelerator_dirty_bytes).collect()),
        "macos_malloc_small_dirty_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_malloc_small_dirty_bytes).collect()),
        "macos_malloc_dirty_bytes_total": stats_u64(rows.iter().filter_map(|r| r.macos_malloc_dirty_bytes_total).collect()),
        "macos_malloc_zones_total_allocated_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_malloc_zones_total_allocated_bytes).collect()),
        "macos_malloc_zones_total_frag_bytes": stats_u64(rows.iter().filter_map(|r| r.macos_malloc_zones_total_frag_bytes).collect()),
        "wgpu_metal_current_allocated_size_bytes_min": stats_u64(rows.iter().filter_map(|r| r.wgpu_metal_current_allocated_size_bytes_min).collect()),
        "wgpu_metal_current_allocated_size_bytes_max": stats_u64(rows.iter().filter_map(|r| r.wgpu_metal_current_allocated_size_bytes_max).collect()),
        "wgpu_allocator_total_allocated_bytes": stats_u64(rows.iter().filter_map(|r| r.wgpu_allocator_total_allocated_bytes).collect()),
        "wgpu_allocator_total_reserved_bytes": stats_u64(rows.iter().filter_map(|r| r.wgpu_allocator_total_reserved_bytes).collect()),
        "render_text_atlas_bytes_live_estimate_total": stats_u64(rows.iter().filter_map(|r| r.render_text_atlas_bytes_live_estimate_total).collect()),
        "render_text_registered_font_blobs_total_bytes": stats_u64(rows.iter().filter_map(|r| r.render_text_registered_font_blobs_total_bytes).collect()),
        "render_text_registered_font_blobs_count": stats_u64(rows.iter().filter_map(|r| r.render_text_registered_font_blobs_count).collect()),
    });

    let top_rows = sorted
        .iter()
        .take(top)
        .map(|r| row_to_json(r))
        .collect::<Vec<_>>();

    let mut out = serde_json::json!({
        "schema_version": 1,
        "kind": "memory_summary",
        "src": src.display().to_string(),
        "samples": rows.len(),
        "sort_key": sort_key,
        "top": top,
        "fields": fields,
        "top_rows": top_rows,
    });

    if include_regions_sorted_agg {
        if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "vmmap_regions_sorted_agg".to_string(),
                vmmap_regions_sorted_agg(rows, regions_sorted_agg_top.max(1)),
            );
        }
    }

    out
}

fn vmmap_regions_sorted_agg(rows: &[MemorySampleRow], top: usize) -> serde_json::Value {
    use std::collections::BTreeMap;

    let mut by_region_type: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut samples_present: usize = 0;

    for row in rows {
        let fp_path = row.out_dir.join("resource.footprint.json");
        let Some(v) = crate::util::read_json_value(&fp_path) else {
            continue;
        };
        let regions_sorted = v
            .get("macos_vmmap_regions_sorted_steady")
            .or_else(|| v.get("macos_vmmap_regions_sorted"));
        let Some(top_dirty) = regions_sorted
            .and_then(|v| v.get("tables"))
            .and_then(|v| v.get("regions"))
            .and_then(|v| v.get("top_dirty"))
            .and_then(|v| v.as_array())
        else {
            continue;
        };

        samples_present = samples_present.saturating_add(1);

        let mut sample_sums: BTreeMap<String, u64> = BTreeMap::new();
        for entry in top_dirty {
            let Some(region_type) = entry.get("region_type").and_then(|v| v.as_str()) else {
                continue;
            };
            let dirty = entry
                .get("dirty_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if dirty == 0 {
                continue;
            }
            *sample_sums.entry(region_type.to_string()).or_default() = sample_sums
                .get(region_type)
                .copied()
                .unwrap_or(0)
                .saturating_add(dirty);
        }

        for (k, dirty_sum) in sample_sums {
            by_region_type.entry(k).or_default().push(dirty_sum);
        }
    }

    let mut rows_out: Vec<(String, U64Stats)> = Vec::new();
    for (k, mut values) in by_region_type {
        if values.is_empty() {
            continue;
        }
        values.sort_unstable();
        let stats = U64Stats {
            count_present: values.len(),
            min: *values.first().unwrap_or(&0),
            p50: quantile_sorted(&values, 0.50),
            p90: quantile_sorted(&values, 0.90),
            max: *values.last().unwrap_or(&0),
        };
        rows_out.push((k, stats));
    }

    rows_out.sort_by(|a, b| b.1.p90.cmp(&a.1.p90).then_with(|| a.0.cmp(&b.0)));
    if rows_out.len() > top {
        rows_out.truncate(top);
    }

    serde_json::json!({
        "schema_version": 1,
        "kind": "vmmap_regions_sorted_agg",
        "samples_present": samples_present,
        "top": top,
        "by_region_type": rows_out.into_iter().map(|(k, s)| serde_json::json!({
            "region_type": k,
            "present": s.count_present,
            "min": s.min,
            "p50": s.p50,
            "p90": s.p90,
            "max": s.max,
        })).collect::<Vec<_>>(),
    })
}

fn validate_sort_key(key: &str) -> Result<(), String> {
    const VALID: &[&str] = &[
        "macos_physical_footprint_peak_bytes",
        "macos_owned_unmapped_memory_dirty_bytes",
        "macos_io_surface_dirty_bytes",
        "macos_io_accelerator_dirty_bytes",
        "macos_malloc_dirty_bytes_total",
        "wgpu_metal_current_allocated_size_bytes_max",
        "render_text_atlas_bytes_live_estimate_total",
        "render_text_registered_font_blobs_total_bytes",
        "render_text_registered_font_blobs_count",
    ];

    if VALID.contains(&key) {
        return Ok(());
    }
    Err(format!(
        "unknown --sort-key: {key}\n\
valid keys:\n  {}",
        VALID.join("\n  ")
    ))
}

fn row_to_json(r: &MemorySampleRow) -> serde_json::Value {
    serde_json::json!({
        "sample_id": r.sample_id,
        "out_dir": r.out_dir.display().to_string(),
        "script_path": r.script_path,
        "killed": r.killed,
        "macos_physical_footprint_peak_bytes": r.macos_physical_footprint_peak_bytes,
        "macos_owned_unmapped_memory_dirty_bytes": r.macos_owned_unmapped_memory_dirty_bytes,
        "macos_io_surface_dirty_bytes": r.macos_io_surface_dirty_bytes,
        "macos_io_accelerator_dirty_bytes": r.macos_io_accelerator_dirty_bytes,
        "macos_malloc_small_dirty_bytes": r.macos_malloc_small_dirty_bytes,
        "macos_malloc_dirty_bytes_total": r.macos_malloc_dirty_bytes_total,
        "macos_malloc_zones_total_allocated_bytes": r.macos_malloc_zones_total_allocated_bytes,
        "macos_malloc_zones_total_frag_bytes": r.macos_malloc_zones_total_frag_bytes,
        "wgpu_metal_current_allocated_size_bytes_min": r.wgpu_metal_current_allocated_size_bytes_min,
        "wgpu_metal_current_allocated_size_bytes_max": r.wgpu_metal_current_allocated_size_bytes_max,
        "wgpu_allocator_total_allocated_bytes": r.wgpu_allocator_total_allocated_bytes,
        "wgpu_allocator_total_reserved_bytes": r.wgpu_allocator_total_reserved_bytes,
        "render_text_atlas_bytes_live_estimate_total": r.render_text_atlas_bytes_live_estimate_total,
        "render_text_registered_font_blobs_total_bytes": r.render_text_registered_font_blobs_total_bytes,
        "render_text_registered_font_blobs_count": r.render_text_registered_font_blobs_count,
        "macos_vmmap_regions_sorted_top_dirty_region_type": r.macos_vmmap_regions_sorted_top_dirty_region_type,
        "macos_vmmap_regions_sorted_top_dirty_detail": r.macos_vmmap_regions_sorted_top_dirty_detail,
        "macos_vmmap_regions_sorted_top_dirty_bytes": r.macos_vmmap_regions_sorted_top_dirty_bytes,
    })
}

fn stats_u64(mut values: Vec<u64>) -> Option<serde_json::Value> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let s = U64Stats {
        count_present: values.len(),
        min: *values.first().unwrap_or(&0),
        p50: quantile_sorted(&values, 0.50),
        p90: quantile_sorted(&values, 0.90),
        max: *values.last().unwrap_or(&0),
    };
    Some(serde_json::json!({
        "present": s.count_present,
        "min": s.min,
        "p50": s.p50,
        "p90": s.p90,
        "max": s.max,
    }))
}

fn quantile_sorted(values: &[u64], q: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    if values.len() == 1 {
        return values[0];
    }
    let q = q.clamp(0.0, 1.0);
    let idx_f = (values.len() as f64 - 1.0) * q;
    let idx = idx_f.floor() as usize;
    values[idx.min(values.len() - 1)]
}

fn sort_u64_for_key(row: &MemorySampleRow, key: &str) -> u64 {
    match key {
        "macos_physical_footprint_peak_bytes" => row.macos_physical_footprint_peak_bytes,
        "macos_owned_unmapped_memory_dirty_bytes" => row.macos_owned_unmapped_memory_dirty_bytes,
        "macos_io_surface_dirty_bytes" => row.macos_io_surface_dirty_bytes,
        "macos_io_accelerator_dirty_bytes" => row.macos_io_accelerator_dirty_bytes,
        "macos_malloc_dirty_bytes_total" => row.macos_malloc_dirty_bytes_total,
        "wgpu_metal_current_allocated_size_bytes_max" => {
            row.wgpu_metal_current_allocated_size_bytes_max
        }
        "render_text_atlas_bytes_live_estimate_total" => {
            row.render_text_atlas_bytes_live_estimate_total
        }
        "render_text_registered_font_blobs_total_bytes" => {
            row.render_text_registered_font_blobs_total_bytes
        }
        "render_text_registered_font_blobs_count" => row.render_text_registered_font_blobs_count,
        _ => row.macos_physical_footprint_peak_bytes,
    }
    .unwrap_or(0)
}

fn human_report(report: &serde_json::Value) -> String {
    let samples = report.get("samples").and_then(|v| v.as_u64()).unwrap_or(0);
    let sort_key = report
        .get("sort_key")
        .and_then(|v| v.as_str())
        .unwrap_or("macos_physical_footprint_peak_bytes");
    let top = report.get("top").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut out = String::new();
    out.push_str("memory_summary:\n");
    out.push_str(&format!("  samples: {samples}\n"));
    out.push_str(&format!("  sort_key: {sort_key}\n"));
    out.push_str(&format!("  top: {top}\n"));

    if let Some(fields) = report.get("fields").and_then(|v| v.as_object()) {
        out.push_str("  fields:\n");
        let mut keys: Vec<&String> = fields.keys().collect();
        keys.sort();
        for k in keys {
            let Some(v) = fields.get(k.as_str()) else {
                continue;
            };
            let Some(obj) = v.as_object() else {
                continue;
            };
            let p = obj.get("present").and_then(|v| v.as_u64()).unwrap_or(0);
            let min = obj.get("min").and_then(|v| v.as_u64()).unwrap_or(0);
            let p50 = obj.get("p50").and_then(|v| v.as_u64()).unwrap_or(0);
            let p90 = obj.get("p90").and_then(|v| v.as_u64()).unwrap_or(0);
            let max = obj.get("max").and_then(|v| v.as_u64()).unwrap_or(0);

            out.push_str(&format!(
                "    {k}: present={p} min={} p50={} p90={} max={}\n",
                human_bytes(min),
                human_bytes(p50),
                human_bytes(p90),
                human_bytes(max),
            ));
        }
    }

    if let Some(rows) = report.get("top_rows").and_then(|v| v.as_array()) {
        out.push_str("  top_rows:\n");
        for r in rows {
            let id = r
                .get("sample_id")
                .and_then(|v| v.as_str())
                .unwrap_or("sample");
            let dir = r.get("out_dir").and_then(|v| v.as_str()).unwrap_or("-");
            let peak = r
                .get("macos_physical_footprint_peak_bytes")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let owned = r
                .get("macos_owned_unmapped_memory_dirty_bytes")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let io_surface = r
                .get("macos_io_surface_dirty_bytes")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let io_accel = r
                .get("macos_io_accelerator_dirty_bytes")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let metal_max = r
                .get("wgpu_metal_current_allocated_size_bytes_max")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            out.push_str(&format!(
                "    - {id}: footprint_peak={peak} owned_unmapped_dirty={owned} io_surface_dirty={io_surface} io_accel_dirty={io_accel} metal_alloc_max={metal_max} dir={dir}\n"
            ));
        }
    }

    if let Some(agg) = report
        .get("vmmap_regions_sorted_agg")
        .and_then(|v| v.as_object())
    {
        let present = agg
            .get("samples_present")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "  vmmap_regions_sorted_agg: samples_present={present}\n"
        ));
        if let Some(rows) = agg.get("by_region_type").and_then(|v| v.as_array()) {
            for r in rows {
                let Some(k) = r.get("region_type").and_then(|v| v.as_str()) else {
                    continue;
                };
                let p = r.get("present").and_then(|v| v.as_u64()).unwrap_or(0);
                let min = r.get("min").and_then(|v| v.as_u64()).unwrap_or(0);
                let p50 = r.get("p50").and_then(|v| v.as_u64()).unwrap_or(0);
                let p90 = r.get("p90").and_then(|v| v.as_u64()).unwrap_or(0);
                let max = r.get("max").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!(
                    "    - {k}: present={p} min={} p50={} p90={} max={}\n",
                    human_bytes(min),
                    human_bytes(p50),
                    human_bytes(p90),
                    human_bytes(max),
                ));
            }
        }
    }

    out
}

fn human_bytes(n: u64) -> String {
    const MB: f64 = (1024 * 1024) as f64;
    const GB: f64 = (1024 * 1024 * 1024) as f64;
    let f = n as f64;
    if f >= GB {
        format!("{:.2}GiB", f / GB)
    } else {
        format!("{:.2}MiB", f / MB)
    }
}
