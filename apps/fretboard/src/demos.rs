use std::path::Path;

use crate::cli::workspace_root;

pub(crate) fn list_native_demos() -> Result<(), String> {
    let root = workspace_root()?;
    let bin_dir = root.join("apps").join("fret-demo").join("src").join("bin");
    let mut demos = read_rs_stems(&bin_dir)?;
    demos.sort();
    for demo in demos {
        println!("{demo}");
    }
    Ok(())
}

pub(crate) fn list_web_demos() -> Result<(), String> {
    for demo in web_demos() {
        println!("{demo}");
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

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn web_demos() -> &'static [&'static str] {
    &[
        // Full UI Gallery app (pages: `?page=...`).
        "ui_gallery",
        // Lightweight examples gallery (separate app from `fret-ui-gallery`).
        "components_gallery",
        "chart_demo",
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

pub(crate) fn web_demos_as_vec() -> Vec<String> {
    web_demos().iter().copied().map(String::from).collect()
}

pub(crate) fn validate_web_demo(name: &str) -> Result<(), String> {
    if web_demos().iter().any(|d| *d == name) {
        return Ok(());
    }
    Err(format!(
        "unknown web demo `{name}`\n  try: fretboard list web-demos"
    ))
}

pub(crate) fn list_native_demos_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let bin_dir = workspace_root
        .join("apps")
        .join("fret-demo")
        .join("src")
        .join("bin");
    read_rs_stems(&bin_dir)
}

pub(crate) fn validate_native_demo(demos: &[String], name: &str) -> Result<(), String> {
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

pub(crate) fn prompt_choose_demo(
    label: &str,
    demos: &[String],
    default: Option<&str>,
    validate: impl Fn(&str) -> Result<(), String>,
) -> Result<String, String> {
    if demos.is_empty() {
        return Err(format!("no {label} found"));
    }

    eprintln!("{label}:");
    for (i, demo) in demos.iter().enumerate() {
        eprintln!("  {:>2}) {demo}", i + 1);
    }

    if let Some(default) = default {
        eprint!("Enter number or name (blank = {default}): ");
    } else {
        eprint!("Enter number or name: ");
    }

    use std::io::Write as _;
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input = input.trim();
    if input.is_empty() {
        return default
            .map(|d| d.to_string())
            .ok_or_else(|| "selection cannot be empty".to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n == 0 || n > demos.len() {
            return Err(format!("invalid selection: {n}"));
        }
        return Ok(demos[n - 1].clone());
    }

    validate(input)?;
    Ok(input.to_string())
}
