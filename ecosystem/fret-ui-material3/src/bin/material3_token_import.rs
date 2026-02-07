//! Developer tool: import a subset of Material Web v30 sassvars into a Rust `ThemeConfig` injector.
//!
//! This is intentionally a developer workflow binary, not runtime API.
//!
//! Usage:
//! `cargo run -p fret-ui-material3 --bin material3_token_import -- --material-web-dir <path>`
//! or:
//! `cargo run -p fret-ui-material3 --bin material3_token_import -- --sass-dir <path>`
//!
//! Output is written into this crate and is intended to be checked into git.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
struct Args {
    sass_dir: PathBuf,
    out: PathBuf,
    prefixes: Vec<String>,
    debug: bool,
    check: bool,
    rustfmt: bool,
}

impl Args {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let default_out = crate_dir
            .join("src")
            .join("tokens")
            .join("material_web_v30.rs");

        let mut material_web_dir: Option<PathBuf> = None;
        let mut sass_dir: Option<PathBuf> = None;
        let mut out: PathBuf = default_out;
        let mut prefixes: Vec<String> = vec![
            "md.sys.motion.".to_string(),
            "md.sys.state.".to_string(),
            "md.sys.state.focus-indicator.".to_string(),
            "md.sys.typescale.".to_string(),
            "md.sys.shape.".to_string(),
            // MVP component prefixes we actively align today.
            "md.comp.button.".to_string(),
            "md.comp.badge.".to_string(),
            "md.comp.fab".to_string(),
            "md.comp.extended-fab".to_string(),
            "md.comp.outlined-segmented-button.".to_string(),
            "md.comp.radio-button.".to_string(),
            "md.comp.checkbox.".to_string(),
            "md.comp.switch.".to_string(),
            "md.comp.icon-button.".to_string(),
            "md.comp.primary-navigation-tab.".to_string(),
            "md.comp.navigation-bar.".to_string(),
            "md.comp.navigation-drawer.".to_string(),
            "md.comp.navigation-rail.".to_string(),
            "md.comp.menu.".to_string(),
            "md.comp.list.".to_string(),
            "md.comp.plain-tooltip.".to_string(),
            "md.comp.rich-tooltip.".to_string(),
            "md.comp.snackbar.".to_string(),
            "md.comp.search-bar.".to_string(),
            "md.comp.search-view.".to_string(),
            "md.comp.carousel-item.".to_string(),
            "md.comp.top-app-bar.small.".to_string(),
            "md.comp.top-app-bar.small.centered.".to_string(),
            "md.comp.top-app-bar.medium.".to_string(),
            "md.comp.top-app-bar.large.".to_string(),
            "md.comp.sheet.bottom.".to_string(),
            "md.comp.date-picker.docked.".to_string(),
            "md.comp.date-picker.modal.".to_string(),
            "md.comp.time-picker.".to_string(),
            "md.comp.time-input.".to_string(),
            "md.comp.outlined-text-field.".to_string(),
            "md.comp.filled-text-field.".to_string(),
            "md.comp.outlined-select.".to_string(),
            "md.comp.filled-select.".to_string(),
            "md.comp.outlined-autocomplete.".to_string(),
            "md.comp.filled-autocomplete.".to_string(),
            "md.comp.dialog.".to_string(),
            "md.comp.full-screen-dialog.".to_string(),
            "md.comp.divider.".to_string(),
            "md.comp.progress-indicator.".to_string(),
            "md.comp.slider.".to_string(),
            "md.comp.assist-chip.".to_string(),
            "md.comp.filter-chip.".to_string(),
            "md.comp.input-chip.".to_string(),
            "md.comp.suggestion-chip.".to_string(),
            "md.comp.filled-card.".to_string(),
            "md.comp.elevated-card.".to_string(),
            "md.comp.outlined-card.".to_string(),
        ];
        let mut debug = false;
        let mut check = false;
        let mut rustfmt = true;

        while let Some(a) = args.next() {
            match a.as_str() {
                "--material-web-dir" => {
                    let v = args.next().ok_or("--material-web-dir expects a path")?;
                    material_web_dir = Some(PathBuf::from(v));
                }
                "--sass-dir" => {
                    let v = args.next().ok_or("--sass-dir expects a path")?;
                    sass_dir = Some(PathBuf::from(v));
                }
                "--out" => {
                    let v = args.next().ok_or("--out expects a path")?;
                    out = PathBuf::from(v);
                }
                "--prefix" => {
                    let v = args.next().ok_or("--prefix expects a string")?;
                    prefixes.push(v);
                }
                "--debug" => debug = true,
                "--check" => check = true,
                "--no-rustfmt" => rustfmt = false,
                "--help" => {
                    return Err(help());
                }
                _ => return Err(format!("unknown arg: {a}\n\n{}", help())),
            }
        }

        let sass_dir = if let Some(s) = sass_dir {
            s
        } else {
            let material_web_dir = material_web_dir
                .or_else(|| env::var("MATERIAL_WEB_DIR").ok().map(PathBuf::from))
                .or_else(|| default_material_web_dir(&crate_dir));

            let Some(material_web_dir) = material_web_dir else {
                return Err(format!(
                    "missing input: pass --sass-dir <path> or --material-web-dir <path> (or set MATERIAL_WEB_DIR)\n\n{}",
                    help()
                ));
            };

            material_web_dir
                .join("tokens")
                .join("versions")
                .join("v30_0")
                .join("sass")
        };

        Ok(Self {
            sass_dir,
            out,
            prefixes,
            debug,
            check,
            rustfmt,
        })
    }
}

fn help() -> String {
    [
        "material3_token_import",
        "",
        "Usage:",
        "cargo run -p fret-ui-material3 --bin material3_token_import -- [options]",
        "",
        "Options:",
        "--material-web-dir <path>   Path to material-web checkout (optional)",
        "(or set MATERIAL_WEB_DIR; if omitted we try to auto-discover repo-root/repo-ref via git)",
        "--sass-dir <path>           Path to v30 sassvars directory (overrides material-web-dir)",
        "--out <path>                Output Rust file path (default: crate src/tokens/material_web_v30.rs)",
        "--prefix <string>           Include only md.* keys with this prefix (repeatable)",
        "--check                     Verify output matches --out and exit non-zero if not",
        "--no-rustfmt                Skip rustfmt formatting of the generated Rust file",
        "--debug                     Print details to stderr",
        "--help                      Show this help",
        "",
    ]
    .join("\n")
}

fn default_material_web_dir(crate_dir: &Path) -> Option<PathBuf> {
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf());

    if let Some(workspace_root) = workspace_root.as_ref() {
        let local = workspace_root.join("repo-ref").join("material-web");
        if local.is_dir() {
            return Some(local);
        }
    }

    repo_root_from_git_common_dir(workspace_root.as_deref().unwrap_or(crate_dir))
        .map(|repo_root| repo_root.join("repo-ref").join("material-web"))
        .filter(|p| p.is_dir())
}

fn repo_root_from_git_common_dir(start_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(start_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let common_dir = PathBuf::from(trimmed);
    let common_dir = if common_dir.is_absolute() {
        common_dir
    } else {
        start_dir.join(common_dir)
    };
    let common_dir = common_dir.canonicalize().unwrap_or(common_dir);
    common_dir.parent().map(|p| p.to_path_buf())
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Null,
    Number(f32),
    Px(f32),
    CornerSetPx {
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    },
    Rem(f32),
    Ms(u32),
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    LocalVar(String),
    ModuleVar {
        module: String,
        var: String,
    },
}

#[derive(Debug, Clone)]
struct TokenDef {
    token_key: String,
    module: String,
    expr: Expr,
}

#[derive(Debug, Default, Clone)]
struct ParseOut {
    by_token: BTreeMap<String, TokenDef>,
    by_module_var: BTreeMap<(String, String), String>, // (module,var) -> token_key
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = match Args::parse(env::args().skip(1)) {
        Ok(a) => a,
        Err(h) if h.contains("material3_token_import") => {
            eprintln!("{h}");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    if args.debug {
        eprintln!("import: sass_dir={}", args.sass_dir.display());
        eprintln!("import: out={}", args.out.display());
        eprintln!("import: prefixes={:?}", args.prefixes);
        eprintln!("import: check={}", args.check);
        eprintln!("import: rustfmt={}", args.rustfmt);
    }

    let out_rs = generate_output(&args)?;

    if args.check {
        let expected = fs::read_to_string(&args.out).map_err(|e| {
            format!(
                "check failed: unable to read output file {}\n{e}",
                args.out.display()
            )
        })?;

        if expected == out_rs {
            if args.debug {
                eprintln!("check: OK {}", args.out.display());
            } else {
                println!("OK {}", args.out.display());
            }
            return Ok(());
        }

        eprintln!(
            "check failed: generated output differs from {}",
            args.out.display()
        );
        eprintln!("hint: run without --check to update the file");
        std::process::exit(1);
    }

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, out_rs)?;
    if args.rustfmt {
        rustfmt_file(&args.out)?;
    }

    if args.debug {
        eprintln!("import: wrote {}", args.out.display());
    } else {
        println!("Wrote {}", args.out.display());
    }

    Ok(())
}

fn generate_output(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = parse_sass_dir(&args.sass_dir)?;
    let selected = select_tokens(&parsed, &args.prefixes);
    if args.debug {
        let selected_autocomplete = selected
            .iter()
            .filter(|def| def.token_key.contains("autocomplete"))
            .count();
        eprintln!(
            "import: parsed_tokens={} selected_tokens={} selected_autocomplete={}",
            parsed.by_token.len(),
            selected.len(),
            selected_autocomplete
        );
    }

    let resolved = resolve_all(selected, &parsed)?;
    let out_rs = emit_rust(&resolved, &args.sass_dir);

    if !args.rustfmt {
        return Ok(out_rs);
    }

    let tmp = args
        .out
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("material_web_v30.__tmp_token_import.rs");

    fs::write(&tmp, &out_rs)?;
    let fmt_result = rustfmt_file(&tmp);
    let formatted = fs::read_to_string(&tmp).unwrap_or(out_rs);
    let _ = fs::remove_file(&tmp);
    fmt_result?;

    Ok(formatted)
}

fn rustfmt_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(path)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("rustfmt failed for {}:\n{stderr}", path.display()).into())
}

fn parse_sass_dir(dir: &Path) -> Result<ParseOut, Box<dyn std::error::Error>> {
    let mut out = ParseOut::default();
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("scss"))
        .collect();
    files.sort();

    for path in files {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("invalid scss file name")?
            .to_string();
        let module = module_name_from_scss_file(&file_name);
        let src = fs::read_to_string(&path)?;

        let mut last_token_key: Option<String> = None;
        for line in src.lines() {
            let line = line.trim();
            if let Some(k) = parse_doc_token_key(line) {
                last_token_key = Some(k);
                continue;
            }

            let Some((var, rhs)) = parse_assignment_line(line) else {
                continue;
            };
            let Some(token_key) = last_token_key.take() else {
                continue;
            };

            let expr = parse_expr(rhs);
            let def = TokenDef {
                token_key: token_key.clone(),
                module: module.clone(),
                expr,
            };

            out.by_module_var
                .insert((module.clone(), var.clone()), token_key.clone());
            out.by_token.insert(token_key, def);
        }
    }

    Ok(out)
}

fn select_tokens(parsed: &ParseOut, prefixes: &[String]) -> Vec<TokenDef> {
    let mut out: Vec<TokenDef> = Vec::new();
    'outer: for def in parsed.by_token.values() {
        for p in prefixes {
            if def.token_key.starts_with(p) {
                out.push(def.clone());
                continue 'outer;
            }
        }
    }
    out.sort_by(|a, b| a.token_key.cmp(&b.token_key));
    out
}

fn resolve_all(
    selected: Vec<TokenDef>,
    parsed: &ParseOut,
) -> Result<Vec<TokenDef>, Box<dyn std::error::Error>> {
    let mut out: Vec<TokenDef> = Vec::with_capacity(selected.len());
    for mut def in selected {
        def.expr = resolve_expr(&def.module, &def.expr, parsed, &mut BTreeSet::new())?;
        out.push(def);
    }
    Ok(out)
}

fn resolve_expr(
    current_module: &str,
    expr: &Expr,
    parsed: &ParseOut,
    stack: &mut BTreeSet<(String, String)>,
) -> Result<Expr, Box<dyn std::error::Error>> {
    match expr {
        Expr::LocalVar(var) => {
            let key = (current_module.to_string(), var.clone());
            if !stack.insert(key.clone()) {
                return Err(
                    format!("cycle while resolving ${var} in module {current_module}").into(),
                );
            }
            let token_key = parsed
                .by_module_var
                .get(&key)
                .ok_or_else(|| format!("unresolved local var ${var} in module {current_module}"))?
                .clone();
            let def = parsed
                .by_token
                .get(&token_key)
                .ok_or_else(|| format!("missing token def for {token_key}"))?;
            let resolved = resolve_expr(&def.module, &def.expr, parsed, stack)?;
            stack.remove(&key);
            Ok(resolved)
        }
        Expr::ModuleVar { module, var } => {
            if module == "md-ref-typeface" && (var == "plain" || var == "brand") {
                return Ok(Expr::ModuleVar {
                    module: module.clone(),
                    var: var.clone(),
                });
            }
            if module == "md-sys-color" || module == "md-ref-palette" {
                return Ok(Expr::ModuleVar {
                    module: module.clone(),
                    var: var.clone(),
                });
            }
            let key = (module.clone(), var.clone());
            if !stack.insert(key.clone()) {
                return Err(format!("cycle while resolving {module}.${var}").into());
            }
            let token_key = parsed
                .by_module_var
                .get(&key)
                .ok_or_else(|| format!("unresolved module var {module}.${var}"))?
                .clone();
            let def = parsed
                .by_token
                .get(&token_key)
                .ok_or_else(|| format!("missing token def for {token_key}"))?;
            let resolved = resolve_expr(&def.module, &def.expr, parsed, stack)?;
            stack.remove(&key);
            Ok(resolved)
        }
        other => Ok(other.clone()),
    }
}

fn emit_rust(defs: &[TokenDef], sass_dir: &Path) -> String {
    let mut out = String::new();
    let canonical_suffix = PathBuf::from("repo-ref")
        .join("material-web")
        .join("tokens")
        .join("versions")
        .join("v30_0")
        .join("sass");
    let sass_label = if path_ends_with(sass_dir, &canonical_suffix) {
        canonical_suffix.display().to_string()
    } else {
        sass_dir.display().to_string()
    };

    writeln!(
        out,
        "// AUTOGENERATED by `material3_token_import`. DO NOT EDIT BY HAND."
    )
    .ok();
    writeln!(out, "//").ok();
    writeln!(
        out,
        "// Source: Material Web v30 sassvars in `{sass_label}`"
    )
    .ok();
    writeln!(out).ok();
    writeln!(out, "use fret_ui::{{theme::CubicBezier, ThemeConfig}};").ok();
    writeln!(
        out,
        "use fret_core::{{Corners, FontId, FontWeight, Px, TextSlant, TextStyle}};"
    )
    .ok();
    writeln!(out).ok();

    emit_inject_fn(
        &mut out,
        "inject_sys_state",
        defs.iter()
            .filter(|d| {
                d.token_key.starts_with("md.sys.state.")
                    && !d.token_key.starts_with("md.sys.state.focus-indicator.")
            })
            .collect::<Vec<_>>(),
    );
    emit_inject_fn(
        &mut out,
        "inject_sys_state_focus_indicator",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.sys.state.focus-indicator."))
            .collect::<Vec<_>>(),
    );
    emit_inject_fn(
        &mut out,
        "inject_sys_motion",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.sys.motion."))
            .collect::<Vec<_>>(),
    );
    emit_inject_sys_shape(
        &mut out,
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.sys.shape."))
            .collect::<Vec<_>>(),
    );

    emit_inject_sys_typescale(
        &mut out,
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.sys.typescale."))
            .collect::<Vec<_>>(),
    );

    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_button_scalars",
        "md.comp.button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_badge_scalars",
        "md.comp.badge.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.badge."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_fab_scalars",
        "md.comp.fab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.fab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_extended_fab_scalars",
        "md.comp.extended-fab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.extended-fab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_outlined_segmented_button_scalars",
        "md.comp.outlined-segmented-button.",
        defs.iter()
            .filter(|d| {
                d.token_key
                    .starts_with("md.comp.outlined-segmented-button.")
            })
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_radio_button_scalars",
        "md.comp.radio-button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.radio-button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_checkbox_scalars",
        "md.comp.checkbox.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.checkbox."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_switch_scalars",
        "md.comp.switch.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.switch."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_icon_button_scalars",
        "md.comp.icon-button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.icon-button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_primary_navigation_tab_scalars",
        "md.comp.primary-navigation-tab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.primary-navigation-tab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_navigation_bar_scalars",
        "md.comp.navigation-bar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-bar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_navigation_drawer_scalars",
        "md.comp.navigation-drawer.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-drawer."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_navigation_rail_scalars",
        "md.comp.navigation-rail.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-rail."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_menu_scalars",
        "md.comp.menu.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.menu."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_list_scalars",
        "md.comp.list.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.list."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_plain_tooltip_scalars",
        "md.comp.plain-tooltip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.plain-tooltip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_rich_tooltip_scalars",
        "md.comp.rich-tooltip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.rich-tooltip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_snackbar_scalars",
        "md.comp.snackbar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.snackbar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_search_bar_scalars",
        "md.comp.search-bar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.search-bar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_search_view_scalars",
        "md.comp.search-view.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.search-view."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_carousel_item_scalars",
        "md.comp.carousel-item.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.carousel-item."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_dialog_scalars",
        "md.comp.dialog.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.dialog."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_full_screen_dialog_scalars",
        "md.comp.full-screen-dialog.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.full-screen-dialog."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_divider_scalars",
        "md.comp.divider.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.divider."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_progress_indicator_scalars",
        "md.comp.progress-indicator.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.progress-indicator."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_slider_scalars",
        "md.comp.slider.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.slider."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_assist_chip_scalars",
        "md.comp.assist-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.assist-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_filter_chip_scalars",
        "md.comp.filter-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filter-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_input_chip_scalars",
        "md.comp.input-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.input-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_suggestion_chip_scalars",
        "md.comp.suggestion-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.suggestion-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_filled_card_scalars",
        "md.comp.filled-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-card."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_elevated_card_scalars",
        "md.comp.elevated-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.elevated-card."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_outlined_card_scalars",
        "md.comp.outlined-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-card."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_outlined_text_field_scalars",
        "md.comp.outlined-text-field.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-text-field."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_filled_text_field_scalars",
        "md.comp.filled-text-field.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-text-field."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_outlined_select_scalars",
        "md.comp.outlined-select.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-select."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_filled_select_scalars",
        "md.comp.filled-select.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-select."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_outlined_autocomplete_scalars",
        "md.comp.outlined-autocomplete.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-autocomplete."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_filled_autocomplete_scalars",
        "md.comp.filled-autocomplete.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-autocomplete."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_date_picker_docked_scalars",
        "md.comp.date-picker.docked.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.date-picker.docked."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_date_picker_modal_scalars",
        "md.comp.date-picker.modal.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.date-picker.modal."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_time_picker_scalars",
        "md.comp.time-picker.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.time-picker."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_scalars(
        &mut out,
        "inject_comp_time_input_scalars",
        "md.comp.time-input.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.time-input."))
            .collect::<Vec<_>>(),
    );

    emit_copy_color_helper(&mut out);
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_button_colors_from_sys",
        "md.comp.button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_badge_colors_from_sys",
        "md.comp.badge.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.badge."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_fab_colors_from_sys",
        "md.comp.fab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.fab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_extended_fab_colors_from_sys",
        "md.comp.extended-fab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.extended-fab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_outlined_segmented_button_colors_from_sys",
        "md.comp.outlined-segmented-button.",
        defs.iter()
            .filter(|d| {
                d.token_key
                    .starts_with("md.comp.outlined-segmented-button.")
            })
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_radio_button_colors_from_sys",
        "md.comp.radio-button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.radio-button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_checkbox_colors_from_sys",
        "md.comp.checkbox.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.checkbox."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_switch_colors_from_sys",
        "md.comp.switch.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.switch."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_icon_button_colors_from_sys",
        "md.comp.icon-button.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.icon-button."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_primary_navigation_tab_colors_from_sys",
        "md.comp.primary-navigation-tab.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.primary-navigation-tab."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_navigation_bar_colors_from_sys",
        "md.comp.navigation-bar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-bar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_navigation_drawer_colors_from_sys",
        "md.comp.navigation-drawer.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-drawer."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_navigation_rail_colors_from_sys",
        "md.comp.navigation-rail.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.navigation-rail."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_menu_colors_from_sys",
        "md.comp.menu.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.menu."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_list_colors_from_sys",
        "md.comp.list.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.list."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_plain_tooltip_colors_from_sys",
        "md.comp.plain-tooltip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.plain-tooltip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_rich_tooltip_colors_from_sys",
        "md.comp.rich-tooltip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.rich-tooltip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_snackbar_colors_from_sys",
        "md.comp.snackbar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.snackbar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_search_bar_colors_from_sys",
        "md.comp.search-bar.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.search-bar."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_search_view_colors_from_sys",
        "md.comp.search-view.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.search-view."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_carousel_item_colors_from_sys",
        "md.comp.carousel-item.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.carousel-item."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_outlined_text_field_colors_from_sys",
        "md.comp.outlined-text-field.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-text-field."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_filled_text_field_colors_from_sys",
        "md.comp.filled-text-field.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-text-field."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_outlined_select_colors_from_sys",
        "md.comp.outlined-select.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-select."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_filled_select_colors_from_sys",
        "md.comp.filled-select.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-select."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_outlined_autocomplete_colors_from_sys",
        "md.comp.outlined-autocomplete.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-autocomplete."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_filled_autocomplete_colors_from_sys",
        "md.comp.filled-autocomplete.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-autocomplete."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_date_picker_docked_colors_from_sys",
        "md.comp.date-picker.docked.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.date-picker.docked."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_date_picker_modal_colors_from_sys",
        "md.comp.date-picker.modal.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.date-picker.modal."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_time_picker_colors_from_sys",
        "md.comp.time-picker.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.time-picker."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_time_input_colors_from_sys",
        "md.comp.time-input.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.time-input."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_dialog_colors_from_sys",
        "md.comp.dialog.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.dialog."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_full_screen_dialog_colors_from_sys",
        "md.comp.full-screen-dialog.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.full-screen-dialog."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_divider_colors_from_sys",
        "md.comp.divider.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.divider."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_progress_indicator_colors_from_sys",
        "md.comp.progress-indicator.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.progress-indicator."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_slider_colors_from_sys",
        "md.comp.slider.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.slider."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_assist_chip_colors_from_sys",
        "md.comp.assist-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.assist-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_filter_chip_colors_from_sys",
        "md.comp.filter-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filter-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_input_chip_colors_from_sys",
        "md.comp.input-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.input-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_suggestion_chip_colors_from_sys",
        "md.comp.suggestion-chip.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.suggestion-chip."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_filled_card_colors_from_sys",
        "md.comp.filled-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.filled-card."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_elevated_card_colors_from_sys",
        "md.comp.elevated-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.elevated-card."))
            .collect::<Vec<_>>(),
    );
    emit_inject_comp_color_aliases(
        &mut out,
        "inject_comp_outlined_card_colors_from_sys",
        "md.comp.outlined-card.",
        defs.iter()
            .filter(|d| d.token_key.starts_with("md.comp.outlined-card."))
            .collect::<Vec<_>>(),
    );

    out
}

fn path_ends_with(path: &Path, suffix: &Path) -> bool {
    let path: Vec<_> = path.components().collect();
    let suffix: Vec<_> = suffix.components().collect();
    if suffix.len() > path.len() {
        return false;
    }
    let offset = path.len() - suffix.len();
    path[offset..] == suffix[..]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypefaceFlavor {
    Plain,
    Brand,
}

#[derive(Debug, Clone)]
struct SysTypescaleRoleDef {
    key: String,
    size_rem: f32,
    line_height_rem: f32,
    tracking_rem: f32,
    weight: u16,
    face: TypefaceFlavor,
}

fn emit_inject_sys_typescale(out: &mut String, defs: Vec<&TokenDef>) {
    let role_names = [
        "display-large",
        "display-medium",
        "display-small",
        "headline-large",
        "headline-medium",
        "headline-small",
        "title-large",
        "title-medium",
        "title-small",
        "label-large",
        "label-medium",
        "label-small",
        "body-large",
        "body-medium",
        "body-small",
    ];

    let mut roles: Vec<SysTypescaleRoleDef> = Vec::new();
    for name in role_names {
        let base = format!("md.sys.typescale.{name}");
        if let Some(r) = read_typescale_role(&defs, &base) {
            roles.push(r);
        }
        let emphasized = format!("md.sys.typescale.emphasized.{name}");
        if let Some(r) = read_typescale_role(&defs, &emphasized) {
            roles.push(r);
        }
    }
    roles.sort_by(|a, b| a.key.cmp(&b.key));

    if roles.is_empty() {
        return;
    }

    writeln!(out, "#[derive(Debug, Clone, Copy)]").ok();
    writeln!(out, "struct SysTypescaleRole {{").ok();
    writeln!(out, "    key: &'static str,").ok();
    writeln!(out, "    size_rem: f32,").ok();
    writeln!(out, "    line_height_rem: f32,").ok();
    writeln!(out, "    tracking_rem: f32,").ok();
    writeln!(out, "    weight: u16,").ok();
    writeln!(out, "    face: TypefaceFlavor,").ok();
    writeln!(out, "}}").ok();
    writeln!(out).ok();

    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]").ok();
    writeln!(out, "enum TypefaceFlavor {{ Plain, Brand }}").ok();
    writeln!(out).ok();

    writeln!(
        out,
        "pub(crate) fn inject_sys_typescale(cfg: &mut ThemeConfig, typography: &super::v30::TypographyOptions) {{"
    )
    .ok();
    writeln!(out, "    let rem_in_px = typography.rem_in_px;").ok();
    writeln!(out, "    for role in [").ok();
    for r in &roles {
        let face = match r.face {
            TypefaceFlavor::Plain => "TypefaceFlavor::Plain",
            TypefaceFlavor::Brand => "TypefaceFlavor::Brand",
        };
        writeln!(
            out,
            "        SysTypescaleRole {{ key: {:?}, size_rem: {:?}, line_height_rem: {:?}, tracking_rem: {:?}, weight: {:?}, face: {} }},",
            r.key, r.size_rem, r.line_height_rem, r.tracking_rem, r.weight, face
        )
        .ok();
    }
    writeln!(out, "    ] {{").ok();
    writeln!(out, "        let size_px = Px(role.size_rem * rem_in_px);").ok();
    writeln!(
        out,
        "        let line_height_px = Px(role.line_height_rem * rem_in_px);"
    )
    .ok();
    writeln!(
        out,
        "        let tracking_em = if role.size_rem.abs() <= f32::EPSILON {{ 0.0 }} else {{ role.tracking_rem / role.size_rem }};"
    )
    .ok();
    writeln!(
        out,
        "        let font: FontId = match role.face {{ TypefaceFlavor::Plain => typography.plain_font.clone(), TypefaceFlavor::Brand => typography.brand_font.clone() }};"
    )
    .ok();
    writeln!(
        out,
        "        cfg.text_styles.insert(role.key.to_string(), TextStyle {{"
    )
    .ok();
    writeln!(out, "            font,").ok();
    writeln!(out, "            size: size_px,").ok();
    writeln!(out, "            weight: FontWeight(role.weight),").ok();
    writeln!(out, "            slant: TextSlant::Normal,").ok();
    writeln!(out, "            line_height: Some(line_height_px),").ok();
    writeln!(out, "            letter_spacing_em: Some(tracking_em),").ok();
    writeln!(out, "        }});").ok();
    writeln!(out, "    }}").ok();
    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn read_typescale_role(defs: &[&TokenDef], base_key: &str) -> Option<SysTypescaleRoleDef> {
    let get = |suffix: &str| -> Option<&Expr> {
        let key = format!("{base_key}.{suffix}");
        defs.iter().find(|d| d.token_key == key).map(|d| &d.expr)
    };

    let size_rem = match get("size")? {
        Expr::Rem(v) => *v,
        _ => return None,
    };
    let line_height_rem = match get("line-height")? {
        Expr::Rem(v) => *v,
        _ => return None,
    };
    let tracking_rem = match get("tracking")? {
        Expr::Rem(v) => *v,
        _ => return None,
    };

    let weight = match get("weight")? {
        Expr::Number(n) => n.round().clamp(1.0, 10_000.0) as u16,
        _ => return None,
    };

    let face = match get("font")? {
        Expr::ModuleVar { module, var } if module == "md-ref-typeface" && var == "plain" => {
            TypefaceFlavor::Plain
        }
        Expr::ModuleVar { module, var } if module == "md-ref-typeface" && var == "brand" => {
            TypefaceFlavor::Brand
        }
        _ => TypefaceFlavor::Plain,
    };

    Some(SysTypescaleRoleDef {
        key: base_key.to_string(),
        size_rem,
        line_height_rem,
        tracking_rem,
        weight,
        face,
    })
}

fn emit_inject_comp_scalars(out: &mut String, fn_name: &str, prefix: &str, defs: Vec<&TokenDef>) {
    let mut keys: Vec<(&str, &Expr)> = defs
        .iter()
        .map(|d| (d.token_key.as_str(), &d.expr))
        .collect();
    keys.sort_by(|a, b| a.0.cmp(b.0));

    writeln!(out, "pub(crate) fn {fn_name}(cfg: &mut ThemeConfig) {{").ok();
    writeln!(out, "    // Source: Material Web v30 sassvars").ok();
    writeln!(out, "    // Prefix: `{prefix}`").ok();
    writeln!(out).ok();

    for (k, expr) in keys {
        // Skip color tokens: our runtime expects parsed color strings, but Material3 uses dynamic
        // `md.sys.color.*` generation. Colors remain derived via `theme_config_with_colors`.
        if k.ends_with(".color") || k.contains(".color.") {
            continue;
        }

        match expr {
            Expr::Px(px) => {
                writeln!(out, "    cfg.metrics.insert({k:?}.to_string(), {px:?});").ok();
            }
            Expr::CornerSetPx {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => {
                writeln!(
                    out,
                    "    cfg.corners.insert({k:?}.to_string(), Corners {{ top_left: Px({top_left:?}), top_right: Px({top_right:?}), bottom_right: Px({bottom_right:?}), bottom_left: Px({bottom_left:?}) }});"
                )
                .ok();
            }
            Expr::Number(n) => {
                writeln!(out, "    cfg.numbers.insert({k:?}.to_string(), {n:?});").ok();
            }
            Expr::Ms(ms) => {
                writeln!(out, "    cfg.durations_ms.insert({k:?}.to_string(), {ms});").ok();
            }
            Expr::CubicBezier { x1, y1, x2, y2 } => {
                writeln!(
                    out,
                    "    cfg.easings.insert({k:?}.to_string(), CubicBezier {{ x1: {x1:?}, y1: {y1:?}, x2: {x2:?}, y2: {y2:?} }});"
                )
                .ok();
            }
            _ => {}
        }
    }

    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn emit_copy_color_helper(out: &mut String) {
    writeln!(
        out,
        "fn copy_color(cfg: &mut ThemeConfig, to_key: &str, from_key: &str) {{"
    )
    .ok();
    writeln!(
        out,
        "    let Some(c) = cfg.colors.get(from_key).cloned() else {{"
    )
    .ok();
    writeln!(out, "        return;").ok();
    writeln!(out, "    }};").ok();
    writeln!(out, "    cfg.colors.insert(to_key.to_string(), c);").ok();
    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn emit_inject_comp_color_aliases(
    out: &mut String,
    fn_name: &str,
    prefix: &str,
    defs: Vec<&TokenDef>,
) {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    for d in defs {
        let key = d.token_key.as_str();
        let from = match &d.expr {
            Expr::ModuleVar { module, var } if module == "md-sys-color" => {
                let mut key = String::with_capacity("md.sys.color.".len() + var.len());
                key.push_str("md.sys.color.");
                key.push_str(var);
                Some(key)
            }
            Expr::ModuleVar { module, var } if module == "md-ref-palette" => {
                let mut key = String::with_capacity("md.ref.palette.".len() + var.len());
                key.push_str("md.ref.palette.");
                key.push_str(var);
                Some(key)
            }
            _ => None,
        };
        let Some(from_key) = from else { continue };
        pairs.push((key, from_key));
    }

    if pairs.is_empty() {
        return;
    }

    pairs.sort_by(|a, b| a.0.cmp(b.0));
    writeln!(out, "pub(crate) fn {fn_name}(cfg: &mut ThemeConfig) {{").ok();
    writeln!(out, "    // Source: Material Web v30 sassvars").ok();
    writeln!(out, "    // Prefix: `{prefix}`").ok();
    writeln!(out).ok();
    for (to_key, from_key) in pairs {
        writeln!(out, "    copy_color(cfg, {to_key:?}, {from_key:?});").ok();
    }
    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn emit_inject_sys_shape(out: &mut String, defs: Vec<&TokenDef>) {
    if defs.is_empty() {
        return;
    }

    let mut keys: Vec<(&str, &Expr)> = defs
        .iter()
        .map(|d| (d.token_key.as_str(), &d.expr))
        .collect();
    keys.sort_by(|a, b| a.0.cmp(b.0));

    writeln!(
        out,
        "pub(crate) fn inject_sys_shape(cfg: &mut ThemeConfig) {{"
    )
    .ok();
    writeln!(out, "    // Source: Material Web v30 sassvars").ok();
    writeln!(out, "    // Prefix: `md.sys.shape.`").ok();
    writeln!(out).ok();

    for (k, expr) in keys {
        match expr {
            Expr::Px(px) => {
                writeln!(out, "    cfg.metrics.insert({k:?}.to_string(), {px:?});").ok();
            }
            Expr::CornerSetPx {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => {
                writeln!(
                    out,
                    "    cfg.corners.insert({k:?}.to_string(), Corners {{ top_left: Px({top_left:?}), top_right: Px({top_right:?}), bottom_right: Px({bottom_right:?}), bottom_left: Px({bottom_left:?}) }});"
                )
                .ok();
            }
            _ => {}
        }
    }

    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn emit_inject_fn(out: &mut String, fn_name: &str, defs: Vec<&TokenDef>) {
    writeln!(out, "pub(crate) fn {fn_name}(cfg: &mut ThemeConfig) {{").ok();
    for d in defs {
        match (&d.token_key[..], &d.expr) {
            (k, Expr::Ms(ms)) => {
                if k.starts_with("md.sys.motion.duration.") {
                    writeln!(out, "    cfg.durations_ms.insert({k:?}.to_string(), {ms});").ok();
                }
            }
            (k, Expr::CubicBezier { x1, y1, x2, y2 }) => {
                if k.starts_with("md.sys.motion.easing.") {
                    writeln!(
                        out,
                        "    cfg.easings.insert({k:?}.to_string(), CubicBezier {{ x1: {x1:?}, y1: {y1:?}, x2: {x2:?}, y2: {y2:?} }});"
                    )
                    .ok();
                }
            }
            (k, Expr::Px(px)) => {
                if k.starts_with("md.sys.state.focus-indicator.") {
                    writeln!(out, "    cfg.metrics.insert({k:?}.to_string(), {px:?});").ok();
                }
            }
            (k, Expr::Number(n)) => {
                if k.starts_with("md.sys.state.")
                    || (k.starts_with("md.sys.motion.spring.")
                        && (k.ends_with(".damping") || k.ends_with(".stiffness")))
                {
                    writeln!(out, "    cfg.numbers.insert({k:?}.to_string(), {n:?});").ok();
                }
            }
            _ => {}
        }
    }
    writeln!(out, "}}").ok();
    writeln!(out).ok();
}

fn parse_doc_token_key(line: &str) -> Option<String> {
    let line = line.trim_start_matches("///").trim();
    if !line.starts_with("md.") {
        return None;
    }
    let end = line
        .find(' ')
        .or_else(|| line.find('('))
        .unwrap_or(line.len());
    Some(line[..end].trim().to_string())
}

fn parse_assignment_line(line: &str) -> Option<(String, String)> {
    if !line.starts_with('$') {
        return None;
    }
    let (lhs, rhs) = line.split_once(':')?;
    let lhs = lhs.trim();
    let rhs = rhs.trim();
    let rhs = rhs.strip_suffix(';').unwrap_or(rhs).trim();
    let var = lhs.strip_prefix('$')?.trim().to_string();
    if var.is_empty() || rhs.is_empty() {
        return None;
    }
    Some((var, rhs.to_string()))
}

fn module_name_from_scss_file(file_name: &str) -> String {
    // Material Web sassvars use filenames like `_md-sys-motion.scss`.
    file_name
        .trim_start_matches('_')
        .trim_end_matches(".scss")
        .to_string()
}

fn parse_expr(rhs: String) -> Expr {
    let rhs = rhs.trim();
    if rhs.contains("null") {
        return Expr::Null;
    }

    let corner_parts: Vec<&str> = rhs.split_whitespace().collect();
    if corner_parts.len() == 4 && corner_parts.iter().all(|p| p.ends_with("px")) {
        let parse_px = |p: &str| -> Option<f32> { p.trim_end_matches("px").trim().parse().ok() };
        if let (Some(top_left), Some(top_right), Some(bottom_right), Some(bottom_left)) = (
            parse_px(corner_parts[0]),
            parse_px(corner_parts[1]),
            parse_px(corner_parts[2]),
            parse_px(corner_parts[3]),
        ) {
            return Expr::CornerSetPx {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            };
        }
    }

    if let Some(rest) = rhs.strip_suffix("ms") {
        if let Ok(ms) = rest.trim().parse::<u32>() {
            return Expr::Ms(ms);
        }
    }

    if let Some(rest) = rhs.strip_suffix("px") {
        if let Ok(px) = rest.trim().parse::<f32>() {
            return Expr::Px(px);
        }
    }

    if let Some(rest) = rhs.strip_suffix("rem") {
        if let Ok(rem) = rest.trim().parse::<f32>() {
            return Expr::Rem(rem);
        }
    }

    if let Some(cb) = rhs
        .strip_prefix("cubic-bezier(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = cb.split(',').map(|p| p.trim()).collect();
        if parts.len() == 4 {
            if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
                parts[3].parse::<f32>(),
            ) {
                return Expr::CubicBezier { x1, y1, x2, y2 };
            }
        }
    }

    if let Ok(n) = rhs.parse::<f32>() {
        return Expr::Number(n);
    }

    if let Some(var) = rhs.strip_prefix('$') {
        return Expr::LocalVar(var.trim().to_string());
    }

    if let Some((module, var)) = rhs.split_once(".$") {
        return Expr::ModuleVar {
            module: module.trim().to_string(),
            var: var.trim().to_string(),
        };
    }

    Expr::Null
}
