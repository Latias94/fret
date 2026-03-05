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
    renderer_intermediate_peak_in_use_bytes: Option<u64>,
    renderer_gpu_images_bytes_estimate: Option<u64>,
    renderer_gpu_render_targets_bytes_estimate: Option<u64>,
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
    let mut fits_linear: Vec<(String, String)> = Vec::new();
    let mut include_regions_sorted_top = false;
    let mut top_sessions: Option<usize> = None;
    let mut include_regions_sorted_agg = false;
    let mut regions_sorted_agg_top: usize = 10;
    let mut include_regions_sorted_detail_agg = false;
    let mut regions_sorted_detail_agg_top: usize = 12;
    let mut include_footprint_categories_agg = false;
    let mut footprint_categories_agg_top: usize = 12;
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
            "--fit-linear" | "--fit_linear" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --fit-linear".to_string());
                };
                let (y_key, x_key) =
                    v.split_once(':')
                        .or_else(|| v.split_once(','))
                        .ok_or_else(|| {
                            "invalid value for --fit-linear: expected \"<y_key>:<x_key>\""
                                .to_string()
                        })?;
                let y_key = y_key.trim();
                let x_key = x_key.trim();
                if y_key.is_empty() || x_key.is_empty() {
                    return Err(
                        "invalid value for --fit-linear: expected \"<y_key>:<x_key>\"".to_string(),
                    );
                }
                fits_linear.push((y_key.to_string(), x_key.to_string()));
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
            "--vmmap-regions-sorted-detail-agg" => {
                include_regions_sorted_detail_agg = true;
                i += 1;
            }
            "--footprint-categories-agg" => {
                include_footprint_categories_agg = true;
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
            "--footprint-categories-agg-top" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --footprint-categories-agg-top".to_string());
                };
                footprint_categories_agg_top = v.parse::<usize>().map_err(|e| {
                    format!("invalid value for --footprint-categories-agg-top: {e}")
                })?;
                if footprint_categories_agg_top == 0 {
                    return Err("--footprint-categories-agg-top must be >= 1".to_string());
                }
                i += 2;
            }
            "--vmmap-regions-sorted-detail-agg-top" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err(
                        "missing value for --vmmap-regions-sorted-detail-agg-top".to_string()
                    );
                };
                regions_sorted_detail_agg_top = v.parse::<usize>().map_err(|e| {
                    format!("invalid value for --vmmap-regions-sorted-detail-agg-top: {e}")
                })?;
                if regions_sorted_detail_agg_top == 0 {
                    return Err("--vmmap-regions-sorted-detail-agg-top must be >= 1".to_string());
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
                    "usage: fretboard diag memory-summary [<base_or_session_out_dir>] [--within-session <id|latest|all>] [--top-sessions <n>] [--sort-key <key>] [--fit-linear <y_key>:<x_key>] [--top <n>] [--vmmap-regions-sorted-top] [--vmmap-regions-sorted-agg] [--vmmap-regions-sorted-agg-top <n>] [--vmmap-regions-sorted-detail-agg] [--vmmap-regions-sorted-detail-agg-top <n>] [--footprint-categories-agg] [--footprint-categories-agg-top <n>] [--no-recursive] [--max-depth <n>] [--max-samples <n>] [--json] [--out <path>]".to_string(),
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
    for (y_key, x_key) in &fits_linear {
        validate_u64_key(y_key).map_err(|e| format!("invalid --fit-linear y_key: {e}"))?;
        validate_u64_key(x_key).map_err(|e| format!("invalid --fit-linear x_key: {e}"))?;
    }

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
        &fits_linear,
        top_rows.max(1),
        &rows,
        include_regions_sorted_agg,
        regions_sorted_agg_top.max(1),
        include_regions_sorted_detail_agg,
        regions_sorted_detail_agg_top.max(1),
        include_footprint_categories_agg,
        footprint_categories_agg_top.max(1),
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
        renderer_intermediate_peak_in_use_bytes: get_u64(
            bundle,
            "renderer_intermediate_peak_in_use_bytes",
        ),
        renderer_gpu_images_bytes_estimate: get_u64(bundle, "renderer_gpu_images_bytes_estimate"),
        renderer_gpu_render_targets_bytes_estimate: get_u64(
            bundle,
            "renderer_gpu_render_targets_bytes_estimate",
        ),
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
    fits_linear: &[(String, String)],
    top: usize,
    rows: &[MemorySampleRow],
    include_regions_sorted_agg: bool,
    regions_sorted_agg_top: usize,
    include_regions_sorted_detail_agg: bool,
    regions_sorted_detail_agg_top: usize,
    include_footprint_categories_agg: bool,
    footprint_categories_agg_top: usize,
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
        "renderer_intermediate_peak_in_use_bytes": stats_u64(rows.iter().filter_map(|r| r.renderer_intermediate_peak_in_use_bytes).collect()),
        "renderer_gpu_images_bytes_estimate": stats_u64(rows.iter().filter_map(|r| r.renderer_gpu_images_bytes_estimate).collect()),
        "renderer_gpu_render_targets_bytes_estimate": stats_u64(rows.iter().filter_map(|r| r.renderer_gpu_render_targets_bytes_estimate).collect()),
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

    if !fits_linear.is_empty() {
        if let Some(obj) = out.as_object_mut() {
            let fits = fits_linear
                .iter()
                .map(|(y_key, x_key)| linear_fit_u64(rows, y_key, x_key))
                .collect::<Vec<_>>();
            obj.insert("fits_linear".to_string(), serde_json::Value::Array(fits));
        }
    }

    if include_regions_sorted_agg {
        if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "vmmap_regions_sorted_agg".to_string(),
                vmmap_regions_sorted_agg(rows, regions_sorted_agg_top.max(1)),
            );
        }
    }

    if include_regions_sorted_detail_agg {
        if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "vmmap_regions_sorted_detail_agg".to_string(),
                vmmap_regions_sorted_detail_agg(rows, regions_sorted_detail_agg_top.max(1)),
            );
        }
    }

    if include_footprint_categories_agg {
        if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "footprint_categories_agg".to_string(),
                footprint_categories_agg(rows, footprint_categories_agg_top.max(1)),
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

fn footprint_categories_agg(rows: &[MemorySampleRow], top: usize) -> serde_json::Value {
    use std::collections::BTreeMap;

    let mut by_category_dirty: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut samples_present: usize = 0;

    for row in rows {
        let fp_path = row.out_dir.join("resource.footprint.json");
        let Some(v) = crate::util::read_json_value(&fp_path) else {
            continue;
        };
        let Some(categories) = v
            .get("macos_footprint_tool_steady")
            .and_then(|v| v.get("categories"))
            .and_then(|v| v.as_object())
        else {
            continue;
        };

        samples_present = samples_present.saturating_add(1);
        for (category, entry) in categories {
            let Some(dirty) = entry.get("dirty_bytes").and_then(|v| v.as_u64()) else {
                continue;
            };
            if dirty == 0 {
                continue;
            }
            by_category_dirty
                .entry(category.clone())
                .or_default()
                .push(dirty);
        }
    }

    let mut rows_out: Vec<(String, U64Stats)> = Vec::new();
    for (k, mut values) in by_category_dirty {
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
        "kind": "footprint_categories_agg",
        "samples_present": samples_present,
        "top": top,
        "by_category": rows_out.into_iter().map(|(k, s)| serde_json::json!({
            "category": k,
            "present": s.count_present,
            "min": s.min,
            "p50": s.p50,
            "p90": s.p90,
            "max": s.max,
        })).collect::<Vec<_>>(),
        "note": "Aggregates `dirty_bytes` per category across samples (macOS-only).",
    })
}

fn vmmap_regions_sorted_detail_agg(rows: &[MemorySampleRow], top: usize) -> serde_json::Value {
    use std::collections::BTreeMap;

    let mut by_key: BTreeMap<(String, String), Vec<u64>> = BTreeMap::new();
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

        let mut sample_sums: BTreeMap<(String, String), u64> = BTreeMap::new();
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
            let detail = entry.get("detail").and_then(|v| v.as_str()).unwrap_or("");
            let detail_key = normalize_vmmap_regions_sorted_detail_key(region_type, detail);
            let key = (region_type.to_string(), detail_key);
            let cur = sample_sums.get(&key).copied().unwrap_or(0);
            sample_sums.insert(key, cur.saturating_add(dirty));
        }

        for (k, dirty_sum) in sample_sums {
            by_key.entry(k).or_default().push(dirty_sum);
        }
    }

    let mut rows_out: Vec<((String, String), U64Stats)> = Vec::new();
    for (k, mut values) in by_key {
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
        "kind": "vmmap_regions_sorted_detail_agg",
        "samples_present": samples_present,
        "top": top,
        "by_region_type_detail": rows_out.into_iter().map(|((region_type, detail_key), s)| serde_json::json!({
            "region_type": region_type,
            "detail_key": detail_key,
            "present": s.count_present,
            "min": s.min,
            "p50": s.p50,
            "p90": s.p90,
            "max": s.max,
        })).collect::<Vec<_>>(),
    })
}

fn normalize_vmmap_regions_sorted_detail_key(region_type: &str, detail: &str) -> String {
    let tokens: Vec<&str> = detail.split_whitespace().collect();
    let mut idx: usize = 0;

    if tokens
        .first()
        .is_some_and(|t| t.contains('/') && t.len() <= 16)
    {
        idx += 1;
    }
    while idx < tokens.len()
        && (tokens[idx].starts_with("SM=") || tokens[idx].starts_with("PURGE="))
    {
        idx += 1;
    }

    let rem = tokens.get(idx..).unwrap_or(&[]).join(" ");
    let rem = rem.trim();
    if rem.is_empty() {
        return "(none)".to_string();
    }

    if region_type.eq_ignore_ascii_case("IOSurface") {
        if let Some(q) = extract_single_quoted(rem) {
            return format!("'{}'", q);
        }
        if rem.starts_with("SurfaceID:") {
            return "SurfaceID".to_string();
        }
    }

    if rem.starts_with("DefaultMallocZone_0x") || rem.starts_with("DefaultMallocZone_0X") {
        return "DefaultMallocZone".to_string();
    }

    let mut out = rem.to_string();
    out = redact_hex_addresses(&out);
    if out.len() > 96 {
        out.truncate(96);
        out.push_str("...");
    }
    out
}

fn extract_single_quoted(s: &str) -> Option<String> {
    let start = s.find('\'')?;
    let rest = &s[start + 1..];
    let end_rel = rest.find('\'')?;
    let inner = &rest[..end_rel];
    (!inner.trim().is_empty()).then_some(inner.trim().to_string())
}

fn redact_hex_addresses(s: &str) -> String {
    // Replace any `0x<hex>` substring with `0x…` to avoid address churn.
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i: usize = 0;
    while i < bytes.len() {
        if bytes[i] == b'0' && i + 1 < bytes.len() && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
        {
            let mut j = i + 2;
            let mut any = false;
            while j < bytes.len() {
                let c = bytes[j] as char;
                if c.is_ascii_hexdigit() {
                    any = true;
                    j += 1;
                    continue;
                }
                break;
            }
            if any {
                out.push_str("0x…");
                i = j;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn validate_u64_key(key: &str) -> Result<(), String> {
    if valid_u64_keys().contains(&key) {
        return Ok(());
    }
    Err(format!(
        "unknown key: {key}\n\
valid keys:\n  {}",
        valid_u64_keys().join("\n  ")
    ))
}

fn validate_sort_key(key: &str) -> Result<(), String> {
    validate_u64_key(key).map_err(|e| e.replace("unknown key:", "unknown --sort-key:"))
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
        "renderer_intermediate_peak_in_use_bytes": r.renderer_intermediate_peak_in_use_bytes,
        "renderer_gpu_images_bytes_estimate": r.renderer_gpu_images_bytes_estimate,
        "renderer_gpu_render_targets_bytes_estimate": r.renderer_gpu_render_targets_bytes_estimate,
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
    opt_u64_for_key(row, key).unwrap_or(0)
}

fn opt_u64_for_key(row: &MemorySampleRow, key: &str) -> Option<u64> {
    match key {
        "macos_physical_footprint_peak_bytes" => row.macos_physical_footprint_peak_bytes,
        "macos_owned_unmapped_memory_dirty_bytes" => row.macos_owned_unmapped_memory_dirty_bytes,
        "macos_io_surface_dirty_bytes" => row.macos_io_surface_dirty_bytes,
        "macos_io_accelerator_dirty_bytes" => row.macos_io_accelerator_dirty_bytes,
        "macos_malloc_small_dirty_bytes" => row.macos_malloc_small_dirty_bytes,
        "macos_malloc_dirty_bytes_total" => row.macos_malloc_dirty_bytes_total,
        "macos_malloc_zones_total_allocated_bytes" => row.macos_malloc_zones_total_allocated_bytes,
        "macos_malloc_zones_total_frag_bytes" => row.macos_malloc_zones_total_frag_bytes,
        "wgpu_metal_current_allocated_size_bytes_min" => {
            row.wgpu_metal_current_allocated_size_bytes_min
        }
        "wgpu_metal_current_allocated_size_bytes_max" => {
            row.wgpu_metal_current_allocated_size_bytes_max
        }
        "wgpu_allocator_total_allocated_bytes" => row.wgpu_allocator_total_allocated_bytes,
        "wgpu_allocator_total_reserved_bytes" => row.wgpu_allocator_total_reserved_bytes,
        "renderer_intermediate_peak_in_use_bytes" => row.renderer_intermediate_peak_in_use_bytes,
        "renderer_gpu_images_bytes_estimate" => row.renderer_gpu_images_bytes_estimate,
        "renderer_gpu_render_targets_bytes_estimate" => {
            row.renderer_gpu_render_targets_bytes_estimate
        }
        "render_text_atlas_bytes_live_estimate_total" => {
            row.render_text_atlas_bytes_live_estimate_total
        }
        "render_text_registered_font_blobs_total_bytes" => {
            row.render_text_registered_font_blobs_total_bytes
        }
        "render_text_registered_font_blobs_count" => row.render_text_registered_font_blobs_count,
        _ => None,
    }
}

fn valid_u64_keys() -> &'static [&'static str] {
    &[
        "macos_physical_footprint_peak_bytes",
        "macos_owned_unmapped_memory_dirty_bytes",
        "macos_io_surface_dirty_bytes",
        "macos_io_accelerator_dirty_bytes",
        "macos_malloc_small_dirty_bytes",
        "macos_malloc_dirty_bytes_total",
        "macos_malloc_zones_total_allocated_bytes",
        "macos_malloc_zones_total_frag_bytes",
        "wgpu_metal_current_allocated_size_bytes_min",
        "wgpu_metal_current_allocated_size_bytes_max",
        "wgpu_allocator_total_allocated_bytes",
        "wgpu_allocator_total_reserved_bytes",
        "renderer_intermediate_peak_in_use_bytes",
        "renderer_gpu_images_bytes_estimate",
        "renderer_gpu_render_targets_bytes_estimate",
        "render_text_atlas_bytes_live_estimate_total",
        "render_text_registered_font_blobs_total_bytes",
        "render_text_registered_font_blobs_count",
    ]
}

fn linear_fit_u64(rows: &[MemorySampleRow], y_key: &str, x_key: &str) -> serde_json::Value {
    let mut points: Vec<(f64, f64)> = Vec::new();
    for row in rows {
        let Some(x) = opt_u64_for_key(row, x_key) else {
            continue;
        };
        let Some(y) = opt_u64_for_key(row, y_key) else {
            continue;
        };
        points.push((x as f64, y as f64));
    }

    let n = points.len();
    let mut out = serde_json::json!({
        "y_key": y_key,
        "x_key": x_key,
        "points": n,
    });

    if n < 2 {
        return out;
    }

    let mean_x: f64 = points.iter().map(|(x, _)| x).sum::<f64>() / n as f64;
    let mean_y: f64 = points.iter().map(|(_, y)| y).sum::<f64>() / n as f64;

    let mut var_x: f64 = 0.0;
    let mut cov_xy: f64 = 0.0;
    for (x, y) in &points {
        let dx = x - mean_x;
        let dy = y - mean_y;
        var_x += dx * dx;
        cov_xy += dx * dy;
    }
    if var_x <= 0.0 {
        return out;
    }

    let slope = cov_xy / var_x;
    let intercept = mean_y - slope * mean_x;

    let mut sst: f64 = 0.0;
    let mut sse: f64 = 0.0;
    for (x, y) in &points {
        let y_hat = intercept + slope * x;
        let dy = y - mean_y;
        let err = y - y_hat;
        sst += dy * dy;
        sse += err * err;
    }
    let r2 = if sst > 0.0 {
        Some(1.0 - sse / sst)
    } else {
        None
    };

    let slope_ppm: Option<u64> =
        (slope.is_finite() && slope >= 0.0).then(|| (slope * 1_000_000.0).round() as u64);
    let intercept_bytes: Option<u64> =
        (intercept.is_finite() && intercept >= 0.0).then(|| intercept.round() as u64);

    if let Some(obj) = out.as_object_mut() {
        obj.insert("slope".to_string(), serde_json::json!(slope));
        obj.insert("intercept".to_string(), serde_json::json!(intercept));
        if let Some(r2) = r2 {
            obj.insert("r2".to_string(), serde_json::json!(r2));
        }
        obj.insert(
            "suggested".to_string(),
            serde_json::json!({
                "intercept_bytes": intercept_bytes,
                "slope_ppm": slope_ppm,
            }),
        );
    }

    out
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
            let images = r
                .get("renderer_gpu_images_bytes_estimate")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let targets = r
                .get("renderer_gpu_render_targets_bytes_estimate")
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            out.push_str(&format!(
                "    - {id}: footprint_peak={peak} owned_unmapped_dirty={owned} io_surface_dirty={io_surface} io_accel_dirty={io_accel} metal_alloc_max={metal_max} gpu_images={images} gpu_targets={targets} dir={dir}\n"
            ));
        }
    }

    if let Some(fits) = report.get("fits_linear").and_then(|v| v.as_array()) {
        out.push_str("  fits_linear:\n");
        for f in fits {
            let y_key = f.get("y_key").and_then(|v| v.as_str()).unwrap_or("y");
            let x_key = f.get("x_key").and_then(|v| v.as_str()).unwrap_or("x");
            let points = f.get("points").and_then(|v| v.as_u64()).unwrap_or(0);
            let slope = f.get("slope").and_then(|v| v.as_f64());
            let intercept = f.get("intercept").and_then(|v| v.as_f64());
            let r2 = f.get("r2").and_then(|v| v.as_f64());
            let suggested_intercept_bytes = f
                .get("suggested")
                .and_then(|v| v.get("intercept_bytes"))
                .and_then(|v| v.as_u64())
                .map(human_bytes)
                .unwrap_or_else(|| "n/a".to_string());
            let suggested_slope_ppm = f
                .get("suggested")
                .and_then(|v| v.get("slope_ppm"))
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".to_string());

            if let (Some(slope), Some(intercept)) = (slope, intercept) {
                let intercept_h = human_bytes(intercept.max(0.0).round() as u64);
                let r2_h = r2
                    .map(|v| format!("{v:.3}"))
                    .unwrap_or_else(|| "n/a".to_string());
                out.push_str(&format!(
                    "    - {y_key} ~ {intercept_h} + {slope:.4} * {x_key} (points={points}, r2={r2_h}, suggested_intercept={suggested_intercept_bytes}, suggested_slope_ppm={suggested_slope_ppm})\n"
                ));
            } else {
                out.push_str(&format!(
                    "    - {y_key} vs {x_key}: insufficient data (points={points})\n"
                ));
            }
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

    if let Some(agg) = report
        .get("vmmap_regions_sorted_detail_agg")
        .and_then(|v| v.as_object())
    {
        let present = agg
            .get("samples_present")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "  vmmap_regions_sorted_detail_agg: samples_present={present}\n"
        ));
        if let Some(rows) = agg.get("by_region_type_detail").and_then(|v| v.as_array()) {
            for r in rows {
                let Some(region_type) = r.get("region_type").and_then(|v| v.as_str()) else {
                    continue;
                };
                let detail_key = r
                    .get("detail_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(none)");
                let p = r.get("present").and_then(|v| v.as_u64()).unwrap_or(0);
                let min = r.get("min").and_then(|v| v.as_u64()).unwrap_or(0);
                let p50 = r.get("p50").and_then(|v| v.as_u64()).unwrap_or(0);
                let p90 = r.get("p90").and_then(|v| v.as_u64()).unwrap_or(0);
                let max = r.get("max").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!(
                    "    - {region_type} | {detail_key}: present={p} min={} p50={} p90={} max={}\n",
                    human_bytes(min),
                    human_bytes(p50),
                    human_bytes(p90),
                    human_bytes(max),
                ));
            }
        }
    }

    if let Some(agg) = report
        .get("footprint_categories_agg")
        .and_then(|v| v.as_object())
    {
        let present = agg
            .get("samples_present")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "  footprint_categories_agg: samples_present={present}\n"
        ));
        if let Some(rows) = agg.get("by_category").and_then(|v| v.as_array()) {
            for r in rows {
                let Some(k) = r.get("category").and_then(|v| v.as_str()) else {
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
