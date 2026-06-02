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
use std::process::Command;

use fret_ui_material3::tokens::{usage, v30, v30_overlay_metadata};

const TOKEN_USAGE_MANIFEST_PATH: &str = "tests/fixtures/material3_token_usage_manifest_v1.json";

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
    let used = usage::scan_audited_sources(&crate_dir).map_err(std::io::Error::other)?;

    if args.check_usage_manifest {
        check_usage_manifest(&crate_dir, &used)?;
        return Ok(());
    }
    if args.update_usage_manifest {
        update_usage_manifest(&crate_dir, &used)?;
        return Ok(());
    }

    let used_exact = used.exact_keys();
    let used_templates = used.template_keys();
    let template_expansion = usage::expand_key_templates(&crate_dir, &used_templates);
    let used_expanded = {
        let mut out = used_exact.clone();
        out.extend(template_expansion.expanded.iter().cloned());
        out
    };
    if args.debug {
        eprintln!(
            "audit: source scan done (sources={}, exact={}, templates={}, expanded={})",
            used.sources.len(),
            used_exact.len(),
            used_templates.len(),
            template_expansion.expanded.len()
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
    let missing_overlay_metadata_injection = v30_overlay_metadata::EXACT_TOKEN_METADATA
        .iter()
        .map(|meta| meta.key.to_string())
        .filter(|key| !injected.contains(key))
        .collect::<BTreeSet<_>>();

    let mut check_failures: Vec<String> = Vec::new();
    if args.check {
        if !template_expansion.unexpanded.is_empty() {
            check_failures.push(format!(
                "unexpanded key templates: {}",
                template_expansion.unexpanded.len()
            ));
        }
        if !missing_injection.is_empty() {
            check_failures.push(format!(
                "missing injected keys: {}",
                missing_injection.len()
            ));
        }
        if !missing_overlay_metadata_injection.is_empty() {
            check_failures.push(format!(
                "overlay metadata keys missing injection: {}",
                missing_overlay_metadata_injection.len()
            ));
        }
    }

    println!("Counts");
    println!("- audited source files: {}", used.sources.len());
    println!("- used keys (exact): {}", used_exact.len());
    println!("- used keys (templates): {}", used_templates.len());
    println!(
        "- used keys (expanded from templates): {}",
        template_expansion.expanded.len()
    );
    println!(
        "- used key templates (unexpanded): {}",
        template_expansion.unexpanded.len()
    );
    println!("- used keys (total): {}", used_expanded.len());
    println!("- injected keys (exact): {}", injected.len());
    println!("- missing injected keys: {}", missing_injection.len());
    println!(
        "- overlay metadata keys missing injection: {}",
        missing_overlay_metadata_injection.len()
    );
    println!("- unused injected keys: {}", unused_injection.len());
    println!();

    if !template_expansion.unexpanded.is_empty() {
        println!("Unexpanded key templates (showing up to {}):", args.limit);
        for k in template_expansion.unexpanded.iter().take(args.limit) {
            println!("- {k}");
        }
        if template_expansion.unexpanded.len() > args.limit {
            println!(
                "- ... ({} more)",
                template_expansion.unexpanded.len() - args.limit
            );
        }
        println!();
    }

    if !missing_injection.is_empty() {
        println!("Missing injected keys (used by code but not present in v30 token injection):");
        print_grouped(&missing_injection, args.limit);
        println!();
    }

    if !missing_overlay_metadata_injection.is_empty() {
        println!("Overlay metadata keys missing from v30 token injection:");
        print_grouped(&missing_overlay_metadata_injection, args.limit);
        println!();
    }

    if args.show_unused && !unused_injection.is_empty() {
        println!("Unused injected keys (present in v30 injection but not referenced by code):");
        print_grouped(&unused_injection, args.limit);
        println!();
    }

    let mut unclassified_vs_material_web: BTreeSet<String> = BTreeSet::new();
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

            let mut classified_overlay: BTreeMap<
                v30_overlay_metadata::MaterialOverlayTokenOrigin,
                BTreeSet<String>,
            > = BTreeMap::new();
            for key in used_expanded.difference(&material_web) {
                if let Some(meta) = v30_overlay_metadata::metadata_for_key(key) {
                    classified_overlay
                        .entry(meta.origin())
                        .or_default()
                        .insert(key.clone());
                } else {
                    unclassified_vs_material_web.insert(key.clone());
                }
            }

            let classified_overlay_count = classified_overlay
                .values()
                .map(BTreeSet::len)
                .sum::<usize>();
            println!(
                "Known overlay/backfill keys not in Material Web v30 sassvars: {classified_overlay_count}"
            );
            if !classified_overlay.is_empty() {
                for (origin, keys) in classified_overlay {
                    println!("- {}: {}", origin.as_str(), keys.len());
                }
            }
            println!(
                "- unclassified non-Material-Web keys: {}",
                unclassified_vs_material_web.len()
            );
            println!();
            if !unclassified_vs_material_web.is_empty() {
                println!(
                    "Unclassified keys (used by code but not found in material-web v30 sassvars):"
                );
                print_grouped(&unclassified_vs_material_web, args.limit);
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
                    let missing = mw
                        .difference(&injected)
                        .filter(|k| !should_ignore_material_web_missing_key(k))
                        .cloned()
                        .collect::<BTreeSet<_>>();
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
            if args.check {
                check_failures.push("material-web sassvars dir not found".to_string());
            }
        }
    } else {
        eprintln!(
            "note: material-web checkout not found. Set --material-web-dir <path> or MATERIAL_WEB_DIR.\n\
                  Expected default: <repo-root>/repo-ref/material-web (or <workspace>/repo-ref/material-web when present)"
        );
        if args.check {
            check_failures.push("material-web checkout not found".to_string());
        }
    }

    if args.check && !unclassified_vs_material_web.is_empty() {
        check_failures.push(format!(
            "unclassified keys vs material-web: {}",
            unclassified_vs_material_web.len()
        ));
    }

    if args.check && !check_failures.is_empty() {
        eprintln!("check failed:");
        for f in &check_failures {
            eprintln!("- {f}");
        }
        std::process::exit(1);
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
    check: bool,
    check_usage_manifest: bool,
    update_usage_manifest: bool,
}

impl Args {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut out = Self {
            material_web_dir: None,
            limit: 50,
            show_unused: false,
            show_material_missing: true,
            debug: false,
            check: false,
            check_usage_manifest: false,
            update_usage_manifest: false,
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
                "--check" => out.check = true,
                "--check-usage-manifest" => out.check_usage_manifest = true,
                "--update-usage-manifest" => out.update_usage_manifest = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown arg: {other} (try --help)")),
            }
        }
        if out.check_usage_manifest && out.update_usage_manifest {
            return Err(
                "--check-usage-manifest and --update-usage-manifest are mutually exclusive"
                    .to_string(),
            );
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
           --check                     Exit non-zero when coverage is not clean\n\
           --check-usage-manifest      Exit non-zero when the checked token usage manifest is stale\n\
           --update-usage-manifest     Rewrite the checked token usage manifest from source scan\n\
           --no-material-missing       Skip material-web missing-by-prefix report\n\
           --debug                     Print progress to stderr\n\
           --help                      Show this help\n"
    );
}

fn check_usage_manifest(
    crate_dir: &Path,
    used: &usage::MaterialTokenUsageScan,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = token_usage_manifest_path(crate_dir);
    let expected = fs::read_to_string(&path)?;
    let generated = usage::default_usage_manifest_json(used);
    if expected.replace("\r\n", "\n") == generated {
        println!("OK {}", path.display());
        return Ok(());
    }

    eprintln!("check failed: generated token usage manifest differs from checked fixture");
    eprintln!("path: {}", path.display());
    eprintln!("hint: run with --update-usage-manifest to refresh it");
    std::process::exit(1);
}

fn update_usage_manifest(
    crate_dir: &Path,
    used: &usage::MaterialTokenUsageScan,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = token_usage_manifest_path(crate_dir);
    fs::write(&path, usage::default_usage_manifest_json(used))?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn token_usage_manifest_path(crate_dir: &Path) -> PathBuf {
    crate_dir.join(TOKEN_USAGE_MANIFEST_PATH)
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
                    let key = format!("md.{token}");
                    out.insert(key.clone());

                    // Material Web emits typography tokens as leaf scalars (font/size/line-height/...),
                    // but Fret represents them as a grouped `TextStyle` token (e.g.
                    // `md.comp.date-picker.modal.header.headline`). To keep `--check` strict while still
                    // allowing these derived keys, also treat the group key as "known" when we see a
                    // typography leaf.
                    //
                    // Important: do NOT treat `.size` as a typography leaf here. Many non-typography
                    // keys use `.size` (e.g. icon sizes), and adding their parent keys would produce
                    // noisy "missing injection" reports that are not meaningful for Fret's token model.
                    for leaf in ["font", "line-height", "tracking", "weight", "type"] {
                        if let Some(base) = key.strip_suffix(&format!(".{leaf}")) {
                            out.insert(base.to_string());
                        }
                    }
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
    if let Ok(p) = env::var("MATERIAL_WEB_DIR")
        && !p.trim().is_empty()
    {
        return Some(PathBuf::from(p));
    }
    let default = workspace_root.join("repo-ref").join("material-web");
    if default.is_dir() {
        return Some(default);
    }

    repo_root_from_git_common_dir(workspace_root)
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

fn should_ignore_material_web_missing_key(key: &str) -> bool {
    // Material Web exposes many typography tokens as individual scalars (size, line-height, ...).
    // Fret represents typography via `ThemeConfig.text_styles` (TextStyle), so these are expected
    // to be "missing" when comparing raw sassvar keys.
    let last = key.rsplit('.').next().unwrap_or(key);
    if matches!(
        last,
        "font" | "line-height" | "size" | "tracking" | "type" | "weight"
    ) {
        return true;
    }

    // Material Web also includes group keys for nested token objects (e.g. the spring specs).
    // We only inject the leaf scalars (`...damping`, `...stiffness`).
    if let Some(rest) = key.strip_prefix("md.sys.motion.spring.")
        && !rest.ends_with(".damping")
        && !rest.ends_with(".stiffness")
    {
        return true;
    }

    // Material Web path tokens are structured objects; Fret doesn't import them yet.
    if key == "md.sys.motion.path" {
        return true;
    }

    false
}
