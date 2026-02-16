use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow};
use renderdog_automation::{QRenderDocPythonRequest, RenderDocInstallation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SummaryFormat {
    Markdown,
    Csv,
}

fn parse_summary_format(s: &str) -> Result<SummaryFormat, anyhow::Error> {
    match s {
        "md" | "markdown" => Ok(SummaryFormat::Markdown),
        "csv" => Ok(SummaryFormat::Csv),
        other => Err(anyhow!("invalid summary format: {other} (expected md|csv)")),
    }
}

#[derive(Debug, serde::Serialize)]
struct DumpRequest {
    capture_path: String,
    marker_contains: String,
    only_drawcalls: bool,
    selection: String,
    max_results: usize,
    output_dir: String,
    basename: String,
    dump_uniform_bytes: bool,
    dump_clip_stack_entries: usize,
    save_outputs_png: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptSummaryTopMarkerPath {
    marker_path: String,
    count: u64,
    first_event_id: i64,
    last_event_id: i64,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptSummaryTopLeafMarker {
    leaf: String,
    count: u64,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptSummary {
    matches_count: u64,
    unique_marker_paths: u64,
    fret_like_matches_count: u64,
    top_marker_paths: Vec<ScriptSummaryTopMarkerPath>,
    top_leaf_markers: Vec<ScriptSummaryTopLeafMarker>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptActionTreeFlags {
    drawcall: u64,
    dispatch: u64,
    push_marker: u64,
    pop_marker: u64,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptActionTreeSummary {
    total_actions: u64,
    total_children: u64,
    max_depth: u64,
    flags: ScriptActionTreeFlags,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptResult {
    #[allow(dead_code)]
    capture_path: String,
    #[allow(dead_code)]
    selection: String,
    summary: Option<ScriptSummary>,
    action_tree: Option<ScriptActionTreeSummary>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptResponse {
    ok: bool,
    result: Option<ScriptResult>,
    error: Option<String>,
}

const SCRIPT_REQ_NAME: &str = "fret_dump_pass_state_json.request.json";
const SCRIPT_RESP_NAME: &str = "fret_dump_pass_state_json.response.json";

fn repo_root() -> Result<PathBuf, anyhow::Error> {
    std::env::current_dir().context("current_dir")
}

fn default_out_dir(root: &Path) -> PathBuf {
    root.join(".fret").join("renderdoc-inspect")
}

fn now_suffix() -> String {
    // Good enough for a run dir name; doesn't need to be stable across machines.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", ts.as_millis())
}

fn script_path(root: &Path) -> PathBuf {
    root.join("tools")
        .join("renderdoc")
        .join("fret_dump_pass_state_json.py")
}

fn print_summary(resp_path: &Path, format: SummaryFormat, top: usize) -> Result<(), anyhow::Error> {
    let bytes =
        std::fs::read(resp_path).with_context(|| format!("read {}", resp_path.display()))?;
    let resp: ScriptResponse = serde_json::from_slice(&bytes).context("parse response json")?;
    if !resp.ok {
        return Err(anyhow!(
            "renderdoc script failed: {}",
            resp.error.unwrap_or_default()
        ));
    }

    let Some(result) = resp.result else {
        return Err(anyhow!("missing result in response json"));
    };
    let Some(summary) = result.summary else {
        return Err(anyhow!("missing summary in response json"));
    };

    match format {
        SummaryFormat::Markdown => {
            eprintln!(
                "summary: matches={} fret_like={} unique_paths={}",
                summary.matches_count, summary.fret_like_matches_count, summary.unique_marker_paths
            );
            if let Some(tree) = result.action_tree {
                eprintln!(
                    "action_tree: actions={} children={} depth={} drawcalls={} dispatch={} push_marker={} pop_marker={}",
                    tree.total_actions,
                    tree.total_children,
                    tree.max_depth,
                    tree.flags.drawcall,
                    tree.flags.dispatch,
                    tree.flags.push_marker,
                    tree.flags.pop_marker
                );
            }
            eprintln!();
            eprintln!("| marker_path | count | event_id_range |");
            eprintln!("|---|---:|---:|");
            for row in summary.top_marker_paths.iter().take(top) {
                let range = if row.first_event_id >= 0 && row.last_event_id >= 0 {
                    format!("{}..{}", row.first_event_id, row.last_event_id)
                } else {
                    "-".to_string()
                };
                eprintln!("| {} | {} | {} |", row.marker_path, row.count, range);
            }

            if !summary.top_leaf_markers.is_empty() {
                eprintln!();
                eprintln!("| leaf | count |");
                eprintln!("|---|---:|");
                for row in summary.top_leaf_markers.iter().take(top) {
                    eprintln!("| {} | {} |", row.leaf, row.count);
                }
            }
        }
        SummaryFormat::Csv => {
            println!("marker_path,count,first_event_id,last_event_id");
            for row in summary.top_marker_paths.iter().take(top) {
                println!(
                    "{},{},{},{}",
                    row.marker_path.replace('"', "\"\""),
                    row.count,
                    row.first_event_id,
                    row.last_event_id
                );
            }
        }
    }

    Ok(())
}

type ParsedArgs = (Option<PathBuf>, DumpRequest, Option<(SummaryFormat, usize)>);

fn parse_args() -> Result<ParsedArgs, anyhow::Error> {
    let mut renderdoc_dir: Option<PathBuf> = None;
    let mut capture_path: Option<String> = None;
    let mut marker_contains: Option<String> = None;
    let mut only_drawcalls = false;
    let mut selection: String = "last".to_string();
    let mut max_results: usize = 200;
    let mut output_dir: Option<String> = None;
    let mut basename: Option<String> = None;
    let mut dump_uniform_bytes = true;
    let mut dump_clip_stack_entries: usize = 64;
    let mut save_outputs_png = true;
    let mut print_summary_cfg: Option<(SummaryFormat, usize)> = None;

    let mut argv: Vec<String> = std::env::args().skip(1).collect();
    // Optional subcommand for ergonomics (e.g. `fret-renderdoc dump --capture ...`).
    if argv.first().is_some_and(|s| s == "dump") {
        argv.remove(0);
    }

    let mut args = argv.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--renderdoc-dir" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow!("--renderdoc-dir requires a value"))?;
                renderdoc_dir = Some(PathBuf::from(v));
            }
            "--capture" => {
                capture_path = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--capture requires a value"))?,
                );
            }
            "--marker" => {
                marker_contains = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--marker requires a value"))?,
                );
            }
            "--selection" => {
                selection = args
                    .next()
                    .ok_or_else(|| anyhow!("--selection requires a value"))?;
            }
            "--max-results" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow!("--max-results requires a value"))?;
                max_results = v.parse().context("parse --max-results")?;
            }
            "--out" => {
                output_dir = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--out requires a value"))?,
                );
            }
            "--basename" => {
                basename = Some(
                    args.next()
                        .ok_or_else(|| anyhow!("--basename requires a value"))?,
                );
            }
            "--no-uniform-bytes" => dump_uniform_bytes = false,
            "--clip-entries" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow!("--clip-entries requires a value"))?;
                dump_clip_stack_entries = v.parse().context("parse --clip-entries")?;
            }
            "--no-outputs-png" => save_outputs_png = false,
            "--only-drawcalls" => only_drawcalls = true,
            "--print-summary" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow!("--print-summary requires a value"))?;
                let fmt = parse_summary_format(&v)?;
                let top = args
                    .next()
                    .map(|v| v.parse::<usize>().context("parse --print-summary <top>"))
                    .transpose()?
                    .unwrap_or(20);
                print_summary_cfg = Some((fmt, top));
            }
            "-h" | "--help" => {
                return Err(anyhow!(
                    "Usage:\n  fret-renderdoc dump --capture <path.rdc> --marker <substring> [options]\n\nOptions:\n  --renderdoc-dir <dir>   RenderDoc install root (contains qrenderdoc + renderdoccmd)\n  --only-drawcalls        Filter matches to draw/dispatch actions only (default: false)\n  --selection <first|last|all> (default: last)\n  --max-results <n>       (default: 200)\n  --out <dir>             Output dir (default: .fret/renderdoc-inspect/<ts>)\n  --basename <name>       Output basename (default: fret_dump)\n  --no-uniform-bytes      Do not dump constant buffer bytes\n  --clip-entries <n>      Dump N ClipRRectUniform entries (default: 64)\n  --no-outputs-png        Do not save pipeline output PNGs\n  --print-summary <md|csv> [top] Print a top-N marker_path breakdown to stdout\n\nNotes:\n  - If auto-detection fails, set RENDERDOG_RENDERDOC_DIR=<RenderDoc install root> or pass --renderdoc-dir.\n  - Run from the repo root so tools/renderdoc/*.py can be found.\n"
                ));
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }

    let capture_path = capture_path.ok_or_else(|| anyhow!("missing --capture"))?;
    let marker_contains = marker_contains.ok_or_else(|| anyhow!("missing --marker"))?;
    if selection != "first" && selection != "last" && selection != "all" {
        return Err(anyhow!(
            "invalid --selection: {selection} (expected first|last|all)"
        ));
    }

    let repo = repo_root()?;

    let capture_path = {
        let p = PathBuf::from(&capture_path);
        if p.is_absolute() { p } else { repo.join(p) }
    };

    let out_dir_path = output_dir
        .map(PathBuf::from)
        .map(|p| if p.is_absolute() { p } else { repo.join(p) })
        .unwrap_or_else(|| default_out_dir(&repo).join(now_suffix()));
    std::fs::create_dir_all(&out_dir_path)
        .with_context(|| format!("create output dir: {}", out_dir_path.display()))?;

    let basename = basename.unwrap_or_else(|| "fret_dump".to_string());

    Ok((
        renderdoc_dir,
        DumpRequest {
            capture_path: capture_path.display().to_string(),
            marker_contains,
            only_drawcalls,
            selection,
            max_results,
            output_dir: out_dir_path.display().to_string(),
            basename,
            dump_uniform_bytes,
            dump_clip_stack_entries,
            save_outputs_png,
        },
        print_summary_cfg,
    ))
}

fn main() -> Result<(), anyhow::Error> {
    let (renderdoc_dir, req, print_summary_cfg) = parse_args()?;

    let root = repo_root()?;
    let script = script_path(&root);
    if !script.is_file() {
        return Err(anyhow!(
            "missing script: {} (run from repo root?)",
            script.display()
        ));
    }

    let install = if let Some(dir) = renderdoc_dir {
        RenderDocInstallation::from_root_dir(dir)?
    } else {
        RenderDocInstallation::detect()?
    };

    let run_dir = PathBuf::from(&req.output_dir);
    let req_path = run_dir.join(SCRIPT_REQ_NAME);
    let resp_path = run_dir.join(SCRIPT_RESP_NAME);
    std::fs::write(&req_path, serde_json::to_vec_pretty(&req)?)?;

    let result = install
        .run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script,
            args: Vec::new(),
            working_dir: Some(run_dir),
        })
        .context("run qrenderdoc --python")?;

    // `qrenderdoc --python` output is primarily useful for diagnosing environment issues.
    if !result.stdout.trim().is_empty() {
        eprintln!("{}", result.stdout);
    }
    if !result.stderr.trim().is_empty() {
        eprintln!("{}", result.stderr);
    }

    if let Some((fmt, top)) = print_summary_cfg {
        print_summary(&resp_path, fmt, top)?;
    }

    println!("{}", resp_path.display());
    Ok(())
}
