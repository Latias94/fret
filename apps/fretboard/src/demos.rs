use std::path::Path;

use crate::cli::workspace_root;

pub(crate) fn list_native_demos(args: Vec<String>) -> Result<(), String> {
    let list_all = parse_list_all_flag(args)?;
    let root = workspace_root()?;
    let bin_dir = root.join("apps").join("fret-demo").join("src").join("bin");
    let mut demos = read_rs_stems(&bin_dir)?;
    demos.sort();

    let (official, maintainer) = split_official_native_demos(&demos);
    for demo in official.iter() {
        println!("{demo}");
    }

    if list_all {
        if !maintainer.is_empty() {
            println!();
        }
        for demo in maintainer.iter() {
            println!("{demo}");
        }
    } else if !maintainer.is_empty() {
        eprintln!(
            "note: {} maintainer/stress demos hidden (use: fretboard-dev list native-demos --all)",
            maintainer.len()
        );
    }
    Ok(())
}

pub(crate) fn list_web_demos(args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        return Err("list web-demos does not accept extra args".to_string());
    }
    for demo in web_demos() {
        println!("{demo}");
    }
    Ok(())
}

pub(crate) fn list_cookbook_examples(args: Vec<String>) -> Result<(), String> {
    let list_all = parse_list_all_flag(args)?;
    let root = workspace_root()?;
    let examples = list_cookbook_examples_from(&root)?;
    let (official, lab) = split_official_cookbook_examples(&examples);
    for ex in official.iter() {
        println!("{ex}");
    }
    if list_all {
        if !lab.is_empty() {
            println!();
        }
        for ex in lab.iter() {
            if let Some(feature_hint) = cookbook_example_feature_hint(ex) {
                println!("{ex}    # requires: {feature_hint}");
            } else {
                println!("{ex}");
            }
        }
    } else if !lab.is_empty() {
        eprintln!(
            "note: {} lab examples hidden (use: fretboard-dev list cookbook-examples --all)",
            lab.len()
        );
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

fn parse_list_all_flag(args: Vec<String>) -> Result<bool, String> {
    let mut all = false;
    for a in args {
        match a.as_str() {
            "--all" => all = true,
            other => return Err(format!("unknown list argument: {other}")),
        }
    }
    Ok(all)
}

fn is_official_native_demo(id: &str) -> bool {
    // Keep this list small and intentional: it defines the user-facing story.
    // Maintainer/stress harnesses remain runnable via `--bin` and discoverable via `--all`.
    matches!(id, "simple_todo_demo" | "todo_demo")
}

fn split_official_native_demos(all: &[String]) -> (Vec<String>, Vec<String>) {
    let mut official = Vec::new();
    let mut maintainer = Vec::new();
    for id in all {
        if is_official_native_demo(id) {
            official.push(id.clone());
        } else {
            maintainer.push(id.clone());
        }
    }
    (official, maintainer)
}

pub(crate) fn official_native_demos(all: &[String]) -> Vec<String> {
    let (official, _) = split_official_native_demos(all);
    official
}

fn is_official_cookbook_example(id: &str) -> bool {
    // Keep this list small and intentional: it defines the onboarding story.
    matches!(
        id,
        "hello"
            | "simple_todo"
            | "overlay_basics"
            | "text_input_basics"
            | "commands_keymap_basics"
            | "theme_switching_basics"
            | "virtual_list_basics"
            | "effects_layer_basics"
            | "hello_counter"
    )
}

pub(crate) fn cookbook_example_feature_hint(id: &str) -> Option<&'static str> {
    let hint = match id {
        "icons_and_assets_basics" => "--features cookbook-assets",
        "assets_reload_epoch_basics" => "--features cookbook-assets",
        "data_table_basics" => "--features cookbook-table",
        "image_asset_cache_basics" => "--features cookbook-image-assets,cookbook-renderer",
        "compositing_alpha_basics" => "--features cookbook-renderer",
        "drop_shadow_basics" => "--features cookbook-renderer",
        "query_basics" => "--features cookbook-query",
        "mutation_toast_feedback_basics" => "--features cookbook-mutation",
        "router_basics" => "--features cookbook-router",
        "undo_basics" => "--features cookbook-undo",
        "async_inbox_basics" => "--features cookbook-async",
        "imui_action_basics" => "--features cookbook-imui",
        "docking_basics" => "--features cookbook-docking",
        "embedded_viewport_basics" => "--features cookbook-interop",
        "external_texture_import_basics" => "--features cookbook-interop",
        "customv1_basics" => "--features cookbook-customv1",
        "utility_window_materials_windows" => "--features cookbook-bootstrap",
        "markdown_and_code_basics" => "--features cookbook-markdown",
        "canvas_pan_zoom_basics" => "--features cookbook-canvas",
        "chart_interactions_basics" => "--features cookbook-chart",
        "gizmo_basics" => "--features cookbook-gizmo",
        _ => return None,
    };
    Some(hint)
}

fn split_official_cookbook_examples(all: &[String]) -> (Vec<String>, Vec<String>) {
    let mut official = Vec::new();
    let mut lab = Vec::new();
    for id in all {
        if is_official_cookbook_example(id) {
            official.push(id.clone());
        } else {
            lab.push(id.clone());
        }
    }
    (official, lab)
}

fn web_demos() -> &'static [&'static str] {
    &[
        // Full UI Gallery app (pages: `?page=...`).
        "ui_gallery",
        // Simple onboarding baseline (matches `fretboard-dev new simple-todo`).
        "simple-todo",
        // Lightweight examples gallery (separate app from `fret-ui-gallery`).
        "components_gallery",
        // Custom effect authoring templates (WebGPU/WGSL; see workstreams renderer-effects-semantics-and-extensibility-v1).
        "custom_effect_v2_web_demo",
        "custom_effect_v2_lut_web_demo",
        "custom_effect_v2_identity_web_demo",
        "custom_effect_v2_glass_chrome_web_demo",
        "custom_effect_v3_web_demo",
        // External texture imports (web copy path; ADR 0234).
        "external_texture_imports_web_demo",
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
    if web_demos().contains(&name) {
        return Ok(());
    }
    Err(format!(
        "unknown web demo `{name}`\n  try: fretboard-dev list web-demos"
    ))
}

pub(crate) fn list_cookbook_examples_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let examples_dir = workspace_root
        .join("apps")
        .join("fret-cookbook")
        .join("examples");
    let mut examples = read_rs_stems(&examples_dir)?;
    examples.sort();
    Ok(examples)
}

pub(crate) fn validate_cookbook_example(examples: &[String], name: &str) -> Result<(), String> {
    if examples.iter().any(|e| e == name) {
        return Ok(());
    }

    let mut hint = String::new();
    for e in examples {
        if e.contains(name) || name.contains(e) {
            hint = format!("\n  hint: did you mean `{e}`?");
            break;
        }
    }

    Err(format!(
        "unknown cookbook example `{name}`{hint}\n  try: fretboard-dev list cookbook-examples"
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
        "unknown native demo `{name}`{hint}\n  try: fretboard-dev list native-demos"
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
