use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Default)]
struct HotspotStat {
    count: u64,
    bytes_sum: u64,
    bytes_max: u64,
    arrays_len_sum: u64,
    objects_len_sum: u64,
}

impl HotspotStat {
    fn add(&mut self, bytes: u64, array_len: Option<usize>, object_len: Option<usize>) {
        self.count = self.count.saturating_add(1);
        self.bytes_sum = self.bytes_sum.saturating_add(bytes);
        self.bytes_max = self.bytes_max.max(bytes);
        if let Some(n) = array_len {
            self.arrays_len_sum = self.arrays_len_sum.saturating_add(n as u64);
        }
        if let Some(n) = object_len {
            self.objects_len_sum = self.objects_len_sum.saturating_add(n as u64);
        }
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn resolve_bundle_json_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_json_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

fn estimate_json_string_bytes(s: &str) -> usize {
    let mut n = 2usize; // quotes
    for c in s.chars() {
        match c {
            '"' | '\\' => n += 2,
            '\n' | '\r' | '\t' | '\u{0008}' | '\u{000C}' => n += 2,
            c if (c as u32) <= 0x1f => n += 6, // \u00XX
            c => n += c.len_utf8(),
        }
    }
    n
}

fn estimate_json_bytes(v: &serde_json::Value) -> usize {
    match v {
        serde_json::Value::Null => 4,
        serde_json::Value::Bool(b) => {
            if *b {
                4
            } else {
                5
            }
        }
        serde_json::Value::Number(n) => n.to_string().len(),
        serde_json::Value::String(s) => estimate_json_string_bytes(s.as_str()),
        serde_json::Value::Array(a) => {
            let mut n = 2usize; // []
            for (idx, el) in a.iter().enumerate() {
                if idx != 0 {
                    n += 1; // ,
                }
                n += estimate_json_bytes(el);
            }
            n
        }
        serde_json::Value::Object(o) => {
            let mut n = 2usize; // {}
            for (idx, (k, v)) in o.iter().enumerate() {
                if idx != 0 {
                    n += 1; // ,
                }
                n += estimate_json_string_bytes(k.as_str());
                n += 1; // :
                n += estimate_json_bytes(v);
            }
            n
        }
    }
}

fn walk_and_record(
    v: &serde_json::Value,
    path: &str,
    depth: usize,
    max_depth: usize,
    out: &mut HashMap<String, HotspotStat>,
) -> u64 {
    let bytes = estimate_json_bytes(v) as u64;
    let (array_len, object_len) = match v {
        serde_json::Value::Array(a) => (Some(a.len()), None),
        serde_json::Value::Object(o) => (None, Some(o.len())),
        _ => (None, None),
    };
    out.entry(path.to_string())
        .or_default()
        .add(bytes, array_len, object_len);

    if depth >= max_depth {
        return bytes;
    }

    match v {
        serde_json::Value::Array(a) => {
            let child_path = format!("{path}[]");
            for el in a {
                walk_and_record(el, child_path.as_str(), depth + 1, max_depth, out);
            }
        }
        serde_json::Value::Object(o) => {
            for (k, child) in o {
                let child_path = format!("{path}.{k}");
                walk_and_record(child, child_path.as_str(), depth + 1, max_depth, out);
            }
        }
        _ => {}
    }

    bytes
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_hotspots(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    hotspots_out: Option<PathBuf>,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut max_depth: usize = 7;
    let mut min_bytes: u64 = 0;
    let mut top: usize = 20;
    let mut force: bool = false;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--max-depth" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --max-depth".to_string());
                };
                max_depth = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --max-depth (expected usize)".to_string())?;
                i += 1;
            }
            "--min-bytes" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --min-bytes".to_string());
                };
                min_bytes = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --min-bytes (expected u64)".to_string())?;
                i += 1;
            }
            "--hotspots-top" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --hotspots-top".to_string());
                };
                top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --hotspots-top (expected usize)".to_string())?;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for hotspots: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let bundle_path =
        resolve_bundle_json_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;

    let file_bytes = std::fs::metadata(&bundle_path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())?;

    const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
    if !force && file_bytes > DEFAULT_MAX_FILE_BYTES {
        return Err(format!(
            "bundle.json is too large to analyze safely by default (size={} > {}); re-run with --force",
            human_bytes(file_bytes),
            human_bytes(DEFAULT_MAX_FILE_BYTES)
        ));
    }

    let t_parse = Instant::now();
    let file = std::fs::File::open(&bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let bundle: serde_json::Value = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
    let parse_ms = t_parse.elapsed().as_millis() as u64;

    let t_analyze = Instant::now();
    let mut stats: HashMap<String, HotspotStat> = HashMap::new();
    let estimated_root_bytes = walk_and_record(&bundle, "$", 0, max_depth, &mut stats);
    let analyze_ms = t_analyze.elapsed().as_millis() as u64;

    #[derive(Debug, Clone)]
    struct Row {
        path: String,
        stat: HotspotStat,
    }

    let mut rows: Vec<Row> = stats
        .into_iter()
        .map(|(path, stat)| Row { path, stat })
        .filter(|r| r.stat.bytes_sum >= min_bytes)
        .collect();

    rows.sort_by(|a, b| {
        b.stat
            .bytes_sum
            .cmp(&a.stat.bytes_sum)
            .then_with(|| b.stat.bytes_max.cmp(&a.stat.bytes_max))
            .then_with(|| a.path.cmp(&b.path))
    });

    if top > 0 && rows.len() > top {
        rows.truncate(top);
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "diag.hotspots",
        "bundle": bundle_path.display().to_string(),
        "file_bytes": file_bytes,
        "estimated_minified_bytes": estimated_root_bytes,
        "max_depth": max_depth,
        "min_bytes": min_bytes,
        "top": top,
        "timing_ms": {
            "parse": parse_ms,
            "analyze": analyze_ms,
        },
        "results": rows.iter().map(|r| serde_json::json!({
            "path": r.path,
            "count": r.stat.count,
            "bytes_sum": r.stat.bytes_sum,
            "bytes_max": r.stat.bytes_max,
            "arrays_len_sum": r.stat.arrays_len_sum,
            "objects_len_sum": r.stat.objects_len_sum,
        })).collect::<Vec<_>>(),
    });

    if let Some(out) = hotspots_out.map(|p| crate::resolve_path(workspace_root, p)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
        if !stats_json {
            println!("{}", out.display());
            return Ok(());
        }
    }

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    println!(
        "bundle={} file_bytes={} estimated_minified_bytes={} parse_ms={} analyze_ms={} max_depth={} min_bytes={} top={}",
        bundle_path.display(),
        human_bytes(file_bytes),
        human_bytes(estimated_root_bytes),
        parse_ms,
        analyze_ms,
        max_depth,
        human_bytes(min_bytes),
        top
    );

    for r in rows {
        let avg_array_len = if r.stat.count > 0 && r.stat.arrays_len_sum > 0 {
            Some(r.stat.arrays_len_sum as f64 / r.stat.count as f64)
        } else {
            None
        };
        let avg_object_len = if r.stat.count > 0 && r.stat.objects_len_sum > 0 {
            Some(r.stat.objects_len_sum as f64 / r.stat.count as f64)
        } else {
            None
        };
        if let Some(avg) = avg_array_len {
            println!(
                "{} count={} bytes_sum={} bytes_max={} arrays_avg_len={:.1}",
                r.path,
                r.stat.count,
                human_bytes(r.stat.bytes_sum),
                human_bytes(r.stat.bytes_max),
                avg
            );
        } else if let Some(avg) = avg_object_len {
            println!(
                "{} count={} bytes_sum={} bytes_max={} objects_avg_len={:.1}",
                r.path,
                r.stat.count,
                human_bytes(r.stat.bytes_sum),
                human_bytes(r.stat.bytes_max),
                avg
            );
        } else {
            println!(
                "{} count={} bytes_sum={} bytes_max={}",
                r.path,
                r.stat.count,
                human_bytes(r.stat.bytes_sum),
                human_bytes(r.stat.bytes_max)
            );
        }
    }

    Ok(())
}
