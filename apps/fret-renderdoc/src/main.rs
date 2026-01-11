use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow};
use renderdog_automation::{QRenderDocPythonRequest, RenderDocInstallation};

#[derive(Debug, serde::Serialize)]
struct DumpRequest {
    capture_path: String,
    marker_contains: String,
    selection: String,
    max_results: usize,
    output_dir: String,
    basename: String,
    dump_uniform_bytes: bool,
    dump_clip_stack_entries: usize,
    save_outputs_png: bool,
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

fn parse_args() -> Result<(Option<PathBuf>, DumpRequest), anyhow::Error> {
    let mut renderdoc_dir: Option<PathBuf> = None;
    let mut capture_path: Option<String> = None;
    let mut marker_contains: Option<String> = None;
    let mut selection: String = "last".to_string();
    let mut max_results: usize = 200;
    let mut output_dir: Option<String> = None;
    let mut basename: Option<String> = None;
    let mut dump_uniform_bytes = true;
    let mut dump_clip_stack_entries: usize = 64;
    let mut save_outputs_png = true;

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
            "-h" | "--help" => {
                return Err(anyhow!(
                    "Usage:\n  fret-renderdoc dump --capture <path.rdc> --marker <substring> [options]\n\nOptions:\n  --renderdoc-dir <dir>   RenderDoc install root (contains qrenderdoc + renderdoccmd)\n  --selection <first|last|all> (default: last)\n  --max-results <n>       (default: 200)\n  --out <dir>             Output dir (default: .fret/renderdoc-inspect/<ts>)\n  --basename <name>       Output basename (default: fret_dump)\n  --no-uniform-bytes      Do not dump constant buffer bytes\n  --clip-entries <n>      Dump N ClipRRectUniform entries (default: 64)\n  --no-outputs-png        Do not save pipeline output PNGs\n\nNotes:\n  - If auto-detection fails, set RENDERDOG_RENDERDOC_DIR=<RenderDoc install root> or pass --renderdoc-dir.\n  - Run from the repo root so tools/renderdoc/*.py can be found.\n"
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
            selection,
            max_results,
            output_dir: out_dir_path.display().to_string(),
            basename,
            dump_uniform_bytes,
            dump_clip_stack_entries,
            save_outputs_png,
        },
    ))
}

fn main() -> Result<(), anyhow::Error> {
    let (renderdoc_dir, req) = parse_args()?;

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

    println!("{}", resp_path.display());
    Ok(())
}
