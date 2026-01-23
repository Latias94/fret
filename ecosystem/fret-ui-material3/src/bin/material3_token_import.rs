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
    Ms(u32),
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
    LocalVar(String),
    ModuleVar { module: String, var: String },
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
                if k.starts_with("md.sys.state.") {
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
