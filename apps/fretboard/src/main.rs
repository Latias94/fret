use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "hotpatch" => hotpatch_cmd(args.collect()),
        "list" => match args.next().as_deref() {
            Some("native-demos") => list_native_demos(),
            Some("web-demos") => list_web_demos(),
            Some(other) => Err(format!("unknown list target: {other}")),
            None => Err("missing list target (try: list native-demos)".to_string()),
        },
        "dev" => match args.next().as_deref() {
            Some("native") => dev_native(args.collect()),
            Some("web") => dev_web(args.collect()),
            Some(other) => Err(format!("unknown dev target: {other}")),
            None => Err("missing dev target (try: dev native)".to_string()),
        },
        other => Err(format!("unknown command: {other}")),
    }
}

fn help() -> Result<(), String> {
    println!(
        r#"fretboard — dev tooling for the Fret workspace

Usage:
  fretboard help
  fretboard hotpatch poke
  fretboard hotpatch path
  fretboard list native-demos
  fretboard list web-demos
  fretboard dev native [--bin <name> | --choose] [--hotpatch] [-- <args...>]
  fretboard dev web [--port <port>] [--demo <demo> | --choose]

Examples:
  fretboard dev native --bin components_gallery
  fretboard dev native --choose
  fretboard dev native --bin image_upload_demo -- --help
  fretboard dev native --hotpatch --choose
  fretboard hotpatch poke
  fretboard dev web --demo plot_demo
"#
    );
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}

fn list_native_demos() -> Result<(), String> {
    let root = workspace_root()?;
    let bin_dir = root.join("apps").join("fret-demo").join("src").join("bin");
    let mut demos = read_rs_stems(&bin_dir)?;
    demos.sort();
    for demo in demos {
        println!("{demo}");
    }
    Ok(())
}

fn list_web_demos() -> Result<(), String> {
    for demo in web_demos() {
        println!("{demo}");
    }
    Ok(())
}

fn dev_native(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;
    let demos = list_native_demos_from(&root)?;

    let mut bin: Option<String> = None;
    let mut choose = false;
    let mut hotpatch = false;
    let mut passthrough: Vec<String> = Vec::new();

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--bin" => {
                bin = Some(
                    it.next()
                        .ok_or_else(|| "--bin requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--hotpatch" => hotpatch = true,
            "--" => {
                passthrough.extend(it);
                break;
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev native: {other}")),
        }
    }

    let bin = match (bin.as_deref(), choose) {
        (Some(name), _) => {
            validate_native_demo(&demos, name)?;
            name.to_string()
        }
        (None, true) => prompt_choose_demo(&demos)?,
        (None, false) => "components_gallery".to_string(),
    };

    let mut cmd = Command::new("cargo");
    cmd.current_dir(root).args(["run", "-p", "fret-demo"]);
    if hotpatch {
        cmd.args(["--features", "hotpatch"]);
        cmd.env("FRET_HOTPATCH", "1");
        cmd.env("FRET_HOTPATCH_TRIGGER_PATH", ".fret/hotpatch.touch");
    }
    cmd.args(["--bin", &bin]);
    if !passthrough.is_empty() {
        cmd.arg("--").args(passthrough);
    }

    let status = cmd.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("cargo exited with status: {status}"));
    }
    Ok(())
}

fn dev_web(args: Vec<String>) -> Result<(), String> {
    let mut port: Option<u16> = None;
    let mut demo: Option<String> = None;
    let mut choose = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--port" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--port requires a value".to_string())?;
                port = Some(raw.parse::<u16>().map_err(|e| e.to_string())?);
            }
            "--demo" => {
                demo = Some(
                    it.next()
                        .ok_or_else(|| "--demo requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev web: {other}")),
        }
    }

    let root = workspace_root()?;
    let web_dir = root.join("apps").join("fret-demo-web");

    let effective_port = port.unwrap_or(8080);
    let mut url = format!("http://127.0.0.1:{effective_port}");
    let demo = match (demo.as_deref(), choose) {
        (Some(name), _) => {
            validate_web_demo(name)?;
            Some(name.to_string())
        }
        (None, true) => Some(prompt_choose_demo(&web_demos_as_vec())?),
        (None, false) => None,
    };
    if let Some(demo) = demo.as_deref() {
        url.push_str(&format!("/?demo={demo}"));
    }

    eprintln!("Starting Trunk dev server in `{}`", display_path(&web_dir));
    eprintln!("Open: {url}");

    let mut cmd = Command::new("trunk");
    cmd.current_dir(web_dir).args(["serve"]);
    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }

    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            e.to_string()
        }
    })?;
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
}

fn read_rs_stems(dir: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir failed for `{}`: {e}", display_path(dir)))?;
    for ent in rd {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        out.push(stem.to_string());
    }
    Ok(out)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn hotpatch_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("poke") => hotpatch_poke(),
        Some("path") => {
            let root = workspace_root()?;
            println!("{}", hotpatch_trigger_path(&root).display());
            Ok(())
        }
        Some("help") | Some("-h") | Some("--help") | None => {
            println!(
                r#"Usage:
  fretboard hotpatch poke   # update the trigger file (causes runner reload when enabled)
  fretboard hotpatch path   # print the trigger file path

Notes:
  - Requires running the app with `--hotpatch` (sets `FRET_HOTPATCH=1`).
  - The runner watches `FRET_HOTPATCH_TRIGGER_PATH` (default: `.fret/hotpatch.touch`)."#
            );
            Ok(())
        }
        Some(other) => Err(format!("unknown hotpatch subcommand: {other}")),
    }
}

fn hotpatch_trigger_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".fret").join("hotpatch.touch")
}

fn hotpatch_poke() -> Result<(), String> {
    let root = workspace_root()?;
    let path = hotpatch_trigger_path(&root);
    let dir = path
        .parent()
        .ok_or_else(|| "invalid hotpatch path".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let marker = format!("{}", now.as_nanos());

    std::fs::write(&path, marker).map_err(|e| e.to_string())?;
    println!("{}", path.display());
    Ok(())
}

fn web_demos() -> &'static [&'static str] {
    &[
        "components_gallery",
        "plot_demo",
        "bars_demo",
        "grouped_bars_demo",
        "stacked_bars_demo",
        "area_demo",
        "candlestick_demo",
        "error_bars_demo",
        "heatmap_demo",
        "histogram_demo",
        "shaded_demo",
        "stairs_demo",
        "stems_demo",
        "linked_cursor_demo",
        "inf_lines_demo",
        "tags_demo",
        "drag_demo",
    ]
}

fn web_demos_as_vec() -> Vec<String> {
    web_demos().iter().copied().map(String::from).collect()
}

fn validate_web_demo(name: &str) -> Result<(), String> {
    if web_demos().iter().any(|d| *d == name) {
        return Ok(());
    }
    Err(format!(
        "unknown web demo `{name}`\n  try: fretboard list web-demos"
    ))
}

fn list_native_demos_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let bin_dir = workspace_root
        .join("apps")
        .join("fret-demo")
        .join("src")
        .join("bin");
    read_rs_stems(&bin_dir)
}

fn validate_native_demo(demos: &[String], name: &str) -> Result<(), String> {
    if demos.iter().any(|d| d == name) {
        return Ok(());
    }

    let mut hint = String::new();
    for d in demos {
        if d.contains(name) || name.contains(d) {
            hint = format!("\n  hint: did you mean `{d}`?");
            break;
        }
    }

    Err(format!(
        "unknown native demo `{name}`{hint}\n  try: fretboard list native-demos"
    ))
}

fn prompt_choose_demo(demos: &[String]) -> Result<String, String> {
    if demos.is_empty() {
        return Err("no native demos found".to_string());
    }

    eprintln!("Select a demo:");
    for (i, demo) in demos.iter().enumerate() {
        eprintln!("  {:>2}) {demo}", i + 1);
    }
    eprint!("Enter number or name: ");

    use std::io::Write as _;
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input = input.trim();
    if input.is_empty() {
        return Ok("components_gallery".to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n == 0 || n > demos.len() {
            return Err(format!("invalid selection: {n}"));
        }
        return Ok(demos[n - 1].clone());
    }

    validate_native_demo(demos, input)?;
    Ok(input.to_string())
}
