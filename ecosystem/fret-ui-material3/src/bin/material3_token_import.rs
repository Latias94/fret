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

#[derive(Debug, Clone)]
struct Args {
    sass_dir: PathBuf,
    out: PathBuf,
    prefixes: Vec<String>,
    debug: bool,
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
            "md.comp.checkbox.".to_string(),
            "md.comp.switch.".to_string(),
            "md.comp.icon-button.".to_string(),
            "md.comp.primary-navigation-tab.".to_string(),
            "md.comp.navigation-bar.".to_string(),
            "md.comp.navigation-drawer.".to_string(),
            "md.comp.navigation-rail.".to_string(),
            "md.comp.menu.".to_string(),
            "md.comp.outlined-text-field.".to_string(),
            "md.comp.filled-text-field.".to_string(),
            "md.comp.dialog.".to_string(),
            "md.comp.full-screen-dialog.".to_string(),
        ];
        let mut debug = false;

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
                "--help" => {
                    return Err(help());
                }
                _ => return Err(format!("unknown arg: {a}\n\n{}", help())),
            }
        }

        let sass_dir = if let Some(s) = sass_dir {
            s
        } else if let Some(mw) =
            material_web_dir.or_else(|| env::var("MATERIAL_WEB_DIR").ok().map(PathBuf::from))
        {
            mw.join("tokens")
                .join("versions")
                .join("v30_0")
                .join("sass")
        } else {
            return Err(format!(
                "missing input: pass --sass-dir <path> or --material-web-dir <path> (or set MATERIAL_WEB_DIR)\n\n{}",
                help()
            ));
        };

        Ok(Self {
            sass_dir,
            out,
            prefixes,
            debug,
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
        "(or set MATERIAL_WEB_DIR)",
        "--sass-dir <path>           Path to v30 sassvars directory (overrides material-web-dir)",
        "--out <path>                Output Rust file path (default: crate src/tokens/material_web_v30.rs)",
        "--prefix <string>           Include only md.* keys with this prefix (repeatable)",
        "--debug                     Print details to stderr",
        "--help                      Show this help",
        "",
    ]
    .join("\n")
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
    }

    let parsed = parse_sass_dir(&args.sass_dir)?;
    let selected = select_tokens(&parsed, &args.prefixes);

    let resolved = resolve_all(selected, &parsed)?;
    let out_rs = emit_rust(&resolved, &args.sass_dir);

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, out_rs)?;

    if args.debug {
        eprintln!("import: wrote {}", args.out.display());
    } else {
        println!("Wrote {}", args.out.display());
    }

    Ok(())
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
    writeln!(out, "use fret_ui::theme::CubicBezier;").ok();
    writeln!(out, "use fret_ui::ThemeConfig;").ok();
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
