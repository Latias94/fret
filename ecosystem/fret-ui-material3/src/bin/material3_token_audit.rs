//! Developer tool: audit Material token coverage against both:
//! - keys referenced by `fret-ui-material3` sources, and
//! - keys available in `repo-ref/material-web` v30 sassvars (when present).
//!
//! This binary is intentionally not part of the runtime library API.
//! It is a "keep us honest" tool to reduce long-tail drift while aiming for outcome alignment.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use fret_ui_material3::tokens::v30;

fn allowlisted_non_material_web_tokens() -> BTreeSet<&'static str> {
    BTreeSet::from([
        // Fret-specific: enforced minimum touch target policy.
        "md.sys.layout.minimum-touch-target.size",
        // Fret-specific: opt into using expressive component token variants when configured.
        "md.sys.fret.material.is-expressive",
        // Fret-specific escape hatch: allow overriding shadow color without forking the elevation logic.
        // Defaults to `md.sys.color.shadow`.
        "md.comp.dialog.container.shadow-color",
    ])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1).collect::<Vec<_>>())?;

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("failed to locate workspace root from CARGO_MANIFEST_DIR")?
        .to_path_buf();

    let source_dir = crate_dir.join("src");
    let v30_path = crate_dir.join("src").join("tokens").join("v30.rs");

    if args.debug {
        eprintln!("audit: scanning source keys...");
        let _ = std::io::stderr().flush();
    }
    let used = extract_used_keys_from_rs_tree(&source_dir)?;
    let (expanded_from_templates, unexpanded_templates) = expand_key_templates(&used.templates);
    let used_expanded = {
        let mut out = used.exact.clone();
        out.extend(expanded_from_templates.iter().cloned());
        out
    };
    if args.debug {
        eprintln!(
            "audit: source scan done (exact={}, templates={}, expanded={})",
            used.exact.len(),
            used.templates.len(),
            expanded_from_templates.len()
        );
        let _ = std::io::stderr().flush();
        eprintln!("audit: building injected key set from v30 ThemeConfig...");
        let _ = std::io::stderr().flush();
    }
    let injected = injected_md_keys_from_v30_theme_config();
    if args.debug {
        eprintln!("audit: injected key set ready (keys={})", injected.len());
        let _ = std::io::stderr().flush();
    }

    println!("Material3 token audit");
    println!("- crate: {}", crate_dir.display());
    println!("- workspace: {}", workspace_root.display());
    println!("- source: {}", source_dir.display());
    println!("- injected: {}", v30_path.display());
    println!();

    let missing_injection = used_expanded
        .difference(&injected)
        .cloned()
        .collect::<BTreeSet<_>>();
    let unused_injection = injected
        .difference(&used_expanded)
        .cloned()
        .collect::<BTreeSet<_>>();

    println!("Counts");
    println!("- used keys (exact): {}", used.exact.len());
    println!("- used keys (templates): {}", used.templates.len());
    println!(
        "- used keys (expanded from templates): {}",
        expanded_from_templates.len()
    );
    println!(
        "- used key templates (unexpanded): {}",
        unexpanded_templates.len()
    );
    println!("- used keys (total): {}", used_expanded.len());
    println!("- injected keys (exact): {}", injected.len());
    println!("- missing injected keys: {}", missing_injection.len());
    println!("- unused injected keys: {}", unused_injection.len());
    println!();

    if !unexpanded_templates.is_empty() {
        println!("Unexpanded key templates (showing up to {}):", args.limit);
        for k in unexpanded_templates.iter().take(args.limit) {
            println!("- {k}");
        }
        if unexpanded_templates.len() > args.limit {
            println!("- ... ({} more)", unexpanded_templates.len() - args.limit);
        }
        println!();
    }

    if !missing_injection.is_empty() {
        println!("Missing injected keys (used by code but not present in v30 token injection):");
        print_grouped(&missing_injection, args.limit);
        println!();
    }

    if args.show_unused && !unused_injection.is_empty() {
        println!("Unused injected keys (present in v30 injection but not referenced by code):");
        print_grouped(&unused_injection, args.limit);
        println!();
    }

    if let Some(material_web_dir) = resolve_material_web_dir(&workspace_root, args.material_web_dir)
    {
        let sassvars_dir = material_web_dir
            .join("tokens")
            .join("versions")
            .join("v30_0")
            .join("sass");
        if sassvars_dir.is_dir() {
            let material_web = extract_md_keys_from_material_web_sassvars(&sassvars_dir)?;

            println!("Material Web v30 sassvars");
            println!("- dir: {}", sassvars_dir.display());
            println!("- keys: {}", material_web.len());
            println!();

            let allowlisted = allowlisted_non_material_web_tokens();
            let unknown_vs_material_web = used_expanded
                .difference(&material_web)
                .filter(|k| !allowlisted.contains(k.as_str()))
                .cloned()
                .collect::<BTreeSet<_>>();
            if !unknown_vs_material_web.is_empty() {
                println!("Unknown keys (used by code but not found in material-web v30 sassvars):");
                print_grouped(&unknown_vs_material_web, args.limit);
                println!();
            }

            if args.show_material_missing {
                let expected_prefixes = default_expected_prefixes();
                let mut missing_by_prefix: BTreeMap<&'static str, BTreeSet<String>> =
                    BTreeMap::new();
                for prefix in expected_prefixes {
                    let mw = material_web
                        .iter()
                        .filter(|k| k.starts_with(prefix))
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    if mw.is_empty() {
                        continue;
                    }
                    let missing = mw.difference(&injected).cloned().collect::<BTreeSet<_>>();
                    if !missing.is_empty() {
                        missing_by_prefix.insert(prefix, missing);
                    }
                }

                if !missing_by_prefix.is_empty() {
                    println!("Material-web keys missing in our injection (by prefix):");
                    for (prefix, keys) in missing_by_prefix {
                        println!("- {prefix}*: {} missing", keys.len());
                        for k in keys.iter().take(args.limit) {
                            println!("  - {k}");
                        }
                        if keys.len() > args.limit {
                            println!("  - ... ({} more)", keys.len() - args.limit);
                        }
                    }
                    println!();
                }
            }
        } else {
            eprintln!(
                "warn: material-web sassvars dir not found: {}",
                sassvars_dir.display()
            );
        }
    } else {
        eprintln!(
            "note: material-web checkout not found. Set --material-web-dir <path> or MATERIAL_WEB_DIR.\n\
                  Expected default: <workspace>/repo-ref/material-web"
        );
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Args {
    material_web_dir: Option<PathBuf>,
    limit: usize,
    show_unused: bool,
    show_material_missing: bool,
    debug: bool,
}

impl Args {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut out = Self {
            material_web_dir: None,
            limit: 50,
            show_unused: false,
            show_material_missing: true,
            debug: false,
        };

        let mut it = args.into_iter();
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--material-web-dir" => {
                    let Some(v) = it.next() else {
                        return Err("--material-web-dir requires a path".to_string());
                    };
                    out.material_web_dir = Some(PathBuf::from(v));
                }
                "--limit" => {
                    let Some(v) = it.next() else {
                        return Err("--limit requires a number".to_string());
                    };
                    out.limit = v
                        .parse::<usize>()
                        .map_err(|_| "--limit must be a number".to_string())?;
                    out.limit = out.limit.max(1);
                }
                "--show-unused" => out.show_unused = true,
                "--no-material-missing" => out.show_material_missing = false,
                "--debug" => out.debug = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown arg: {other} (try --help)")),
            }
        }
        Ok(out)
    }
}

fn print_help() {
    println!(
        "material3_token_audit\n\
         \n\
         Usage:\n\
           cargo run -p fret-ui-material3 --bin material3_token_audit -- [options]\n\
         \n\
         Options:\n\
           --material-web-dir <path>   Path to material-web checkout (optional)\n\
                                      (or set MATERIAL_WEB_DIR)\n\
           --limit <n>                 Max items per section (default: 50)\n\
           --show-unused               Print injected-but-unused keys\n\
           --no-material-missing       Skip material-web missing-by-prefix report\n\
           --debug                     Print progress to stderr\n\
           --help                      Show this help\n"
    );
}

#[derive(Debug, Default)]
struct KeyScan {
    exact: BTreeSet<String>,
    templates: BTreeSet<String>,
}

fn expand_key_templates(templates: &BTreeSet<String>) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut expanded = BTreeSet::new();
    let mut unexpanded = BTreeSet::new();

    for t in templates {
        if let Some(keys) = expand_key_template(t) {
            expanded.extend(keys);
        } else {
            unexpanded.insert(t.clone());
        }
    }

    (expanded, unexpanded)
}

fn expand_key_template(template: &str) -> Option<BTreeSet<String>> {
    if template.starts_with("md.comp.button.") {
        return expand_button_template(template);
    }
    if template.starts_with("md.comp.icon-button.") {
        return expand_icon_button_template(template);
    }
    if template.starts_with("md.comp.radio-button.") {
        return expand_radio_button_template(template);
    }
    if template.starts_with("md.comp.switch.") {
        return expand_switch_template(template);
    }
    if template.starts_with("md.sys.typescale.") {
        return expand_typescale_template(template);
    }

    None
}

fn ensure_no_template_braces(keys: &BTreeSet<String>) -> Option<BTreeSet<String>> {
    keys.iter()
        .all(|k| !k.contains('{') && !k.contains('}'))
        .then_some(keys.clone())
}

fn expand_placeholder(
    keys: &BTreeSet<String>,
    placeholder: &str,
    values: &[&'static str],
) -> BTreeSet<String> {
    if !keys.iter().any(|k| k.contains(placeholder)) {
        return keys.clone();
    }

    let mut out = BTreeSet::new();
    for k in keys {
        if k.contains(placeholder) {
            for v in values {
                out.insert(k.replace(placeholder, v));
            }
        } else {
            out.insert(k.clone());
        }
    }
    out
}

fn expand_button_template(template: &str) -> Option<BTreeSet<String>> {
    const VARIANTS: &[&str] = &["filled", "tonal", "elevated", "outlined", "text"];
    const SUFFIXES: &[&str] = &[
        "hovered.state-layer.color",
        "focused.state-layer.color",
        "pressed.state-layer.color",
    ];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{variant_key}", VARIANTS);
    keys = expand_placeholder(&keys, "{}", VARIANTS);
    keys = expand_placeholder(&keys, "{suffix}", SUFFIXES);

    ensure_no_template_braces(&keys)
}

fn expand_icon_button_template(template: &str) -> Option<BTreeSet<String>> {
    const VARIANTS: &[&str] = &["standard", "filled", "tonal", "outlined"];
    const STATE_DOTTED: &[&str] = &["hovered.", "focused.", "pressed."];
    const STATE_TRIMMED: &[&str] = &["hovered", "focused", "pressed"];

    let mut keys = BTreeSet::from([template.to_string()]);

    // Named variant placeholder.
    keys = expand_placeholder(&keys, "{variant_key}", VARIANTS);

    // Special-case: format-style `{}` used as the variant slot only for
    // `md.comp.icon-button.{}.pressed.state-layer.opacity`.
    keys = expand_icon_button_variant_slot(&keys, VARIANTS);

    // Expand `select_prefix` based on the variant segment present in the key.
    keys = expand_icon_button_select_prefix(&keys);

    // Outlined uses a `prefix` naming, but it's the same select-prefix concept.
    keys = expand_placeholder(&keys, "{prefix}", &["", "selected."]);

    // Interaction state placeholders.
    keys = expand_placeholder(&keys, "{state}", STATE_DOTTED);
    keys = expand_placeholder(&keys, "{}", STATE_TRIMMED);

    ensure_no_template_braces(&keys)
}

fn expand_icon_button_variant_slot(
    keys: &BTreeSet<String>,
    variants: &[&'static str],
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for k in keys {
        if k.starts_with("md.comp.icon-button.{}.") {
            for v in variants {
                out.insert(k.replacen(
                    "md.comp.icon-button.{}.",
                    &format!("md.comp.icon-button.{v}."),
                    1,
                ));
            }
        } else {
            out.insert(k.clone());
        }
    }
    out
}

fn expand_icon_button_select_prefix(keys: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for k in keys {
        if !k.contains("{select_prefix}") {
            out.insert(k.clone());
            continue;
        }

        let Some(variant) = icon_button_variant_from_key(k) else {
            out.insert(k.clone());
            continue;
        };

        for prefix in icon_button_select_prefixes(variant) {
            out.insert(k.replace("{select_prefix}", prefix));
        }
    }
    out
}

fn icon_button_variant_from_key(key: &str) -> Option<&str> {
    let rest = key.strip_prefix("md.comp.icon-button.")?;
    Some(rest.split('.').next().unwrap_or_default())
}

fn icon_button_select_prefixes(variant: &str) -> &'static [&'static str] {
    match variant {
        // Standard: base tokens are unselected; selected uses a distinct prefix.
        "standard" => &["", "selected."],
        // Filled: base tokens are the selected look; unselected uses a distinct prefix.
        "filled" => &["", "unselected."],
        // Tonal: base tokens are unselected; selected uses a distinct prefix.
        "tonal" => &["", "selected."],
        // Outlined: base tokens are unselected; selected uses a distinct prefix.
        "outlined" => &["", "selected."],
        _ => &[""],
    }
}

fn expand_radio_button_template(template: &str) -> Option<BTreeSet<String>> {
    const GROUPS: &[&str] = &["selected", "unselected"];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{group}", GROUPS);

    ensure_no_template_braces(&keys)
}

fn expand_switch_template(template: &str) -> Option<BTreeSet<String>> {
    const GROUPS: &[&str] = &["selected", "unselected"];
    const STATES: &[&str] = &["hover", "focus", "pressed"];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{group}", GROUPS);
    keys = expand_placeholder(&keys, "{state}", STATES);

    ensure_no_template_braces(&keys)
}

fn expand_typescale_template(template: &str) -> Option<BTreeSet<String>> {
    const NAMES: &[&str] = &[
        "display-large",
        "display-medium",
        "display-small",
        "headline-large",
        "headline-medium",
        "headline-small",
        "title-large",
        "title-medium",
        "title-small",
        "body-large",
        "body-medium",
        "body-small",
        "label-large",
        "label-medium",
        "label-small",
    ];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{name}", NAMES);

    ensure_no_template_braces(&keys)
}

fn extract_used_keys_from_rs_tree(dir: &Path) -> Result<KeyScan, Box<dyn std::error::Error>> {
    let mut scan = KeyScan::default();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path)?;
        for entry in entries {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension() != Some(OsStr::new("rs")) {
                continue;
            }

            let content = fs::read_to_string(&p)?;
            scan_md_string_literals(&content, &mut scan);
        }
    }
    Ok(scan)
}

fn injected_md_keys_from_v30_theme_config() -> BTreeSet<String> {
    let cfg = v30::theme_config_with_colors(
        v30::TypographyOptions::default(),
        v30::ColorSchemeOptions::default(),
    );

    let mut out = BTreeSet::new();
    for k in cfg.colors.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.metrics.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.numbers.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.durations_ms.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.easings.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.text_styles.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }
    for k in cfg.corners.keys() {
        if k.starts_with("md.") {
            out.insert(k.clone());
        }
    }

    out
}

fn extract_md_keys_from_material_web_sassvars(
    sassvars_dir: &Path,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let mut out = BTreeSet::new();
    for entry in fs::read_dir(sassvars_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("scss")) {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        for line in content.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("/// md.") {
                let token = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(')');
                if !token.is_empty() {
                    out.insert(format!("md.{token}"));
                }
            }
        }
    }
    Ok(out)
}

fn resolve_material_web_dir(
    workspace_root: &Path,
    override_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(p) = override_dir {
        return Some(p);
    }
    if let Ok(p) = env::var("MATERIAL_WEB_DIR") {
        if !p.trim().is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    let default = workspace_root.join("repo-ref").join("material-web");
    default.is_dir().then_some(default)
}

fn default_expected_prefixes() -> &'static [&'static str] {
    &[
        "md.comp.button.",
        "md.comp.icon-button.",
        "md.comp.checkbox.",
        "md.comp.switch.",
        "md.comp.radio-button.",
        "md.comp.outlined-text-field.",
        "md.comp.filled-text-field.",
        "md.comp.primary-navigation-tab.",
        "md.comp.menu.",
        "md.sys.state.",
        "md.sys.motion.",
        "md.sys.shape.",
        "md.sys.typescale.",
        "md.sys.color.",
    ]
}

fn print_grouped(keys: &BTreeSet<String>, limit: usize) {
    let mut by_prefix: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for k in keys {
        let prefix = group_prefix(k);
        by_prefix.entry(prefix).or_default().push(k);
    }

    for (prefix, group) in by_prefix {
        println!("- {prefix}: {} keys", group.len());
        for k in group.iter().take(limit) {
            println!("  - {k}");
        }
        if group.len() > limit {
            println!("  - ... ({} more)", group.len() - limit);
        }
    }
}

fn group_prefix(key: &str) -> &str {
    if let Some(comp) = key.strip_prefix("md.comp.") {
        let name = comp.split('.').next().unwrap_or("comp");
        // Group per component surface (e.g. md.comp.radio-button.*).
        // Keep the `md.comp.` prefix so it's obvious in logs.
        return match name {
            "button" => "md.comp.button",
            "icon-button" => "md.comp.icon-button",
            "checkbox" => "md.comp.checkbox",
            "switch" => "md.comp.switch",
            "radio-button" => "md.comp.radio-button",
            "outlined-text-field" => "md.comp.outlined-text-field",
            "filled-text-field" => "md.comp.filled-text-field",
            "primary-navigation-tab" => "md.comp.primary-navigation-tab",
            "menu" => "md.comp.menu",
            _ => "md.comp.<other>",
        };
    }
    if let Some(sys) = key.strip_prefix("md.sys.") {
        let name = sys.split('.').next().unwrap_or("sys");
        return match name {
            "color" => "md.sys.color",
            "state" => "md.sys.state",
            "motion" => "md.sys.motion",
            "shape" => "md.sys.shape",
            "typescale" => "md.sys.typescale",
            _ => "md.sys.<other>",
        };
    }
    "other"
}

fn is_prefix_only_key(key: &str) -> bool {
    if key == "md.sys." || key == "md.comp." {
        return true;
    }
    if key.ends_with('.') {
        return true;
    }
    match key.strip_prefix("md.sys.") {
        Some(rest) => !rest.contains('.'),
        None => match key.strip_prefix("md.comp.") {
            Some(rest) => rest.matches('.').count() < 2,
            None => false,
        },
    }
}

fn scan_md_string_literals(content: &str, scan: &mut KeyScan) {
    let mut i = 0usize;
    while let Some(pos) = content[i..].find("\"md.") {
        let start = i + pos + 1;
        let tail = &content[start..];
        let end = tail.find('"').unwrap_or(tail.len());
        let key = &tail[..end];
        if key.starts_with("md.") && !is_prefix_only_key(key) {
            if key.contains('{') || key.contains('}') {
                scan.templates.insert(key.to_string());
            } else {
                scan.exact.insert(key.to_string());
            }
        }
        i = start + end + 1;
    }
}
