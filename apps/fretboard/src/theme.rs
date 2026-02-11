use std::path::PathBuf;

use fret_ui::theme::ThemeConfig;
use fret_vscode_theme::{
    MappingEntry, VscodeImportMapping, VscodeSyntaxImportOptions,
    syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping,
};

pub(crate) fn theme_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(target) = it.next() else {
        return Err(
            "missing theme target (try: fretboard theme import-vscode <theme.json>)".to_string(),
        );
    };

    match target.as_str() {
        "--help" | "-h" => crate::cli::help(),
        "import-vscode" => import_vscode_cmd(it.collect()),
        other => Err(format!("unknown theme target: {other}")),
    }
}

fn import_vscode_cmd(args: Vec<String>) -> Result<(), String> {
    let mut input: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut base: Option<PathBuf> = None;
    let mut report: Option<PathBuf> = None;
    let mut map: Option<PathBuf> = None;
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut all_tags = false;
    let mut force = false;
    let mut name_override: Option<String> = None;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--out requires a value".to_string())?;
                out = Some(PathBuf::from(raw));
            }
            "--base" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--base requires a value".to_string())?;
                base = Some(PathBuf::from(raw));
            }
            "--name" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--name requires a value".to_string())?;
                name_override = Some(raw);
            }
            "--report" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--report requires a value".to_string())?;
                report = Some(PathBuf::from(raw));
            }
            "--map" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--map requires a value".to_string())?;
                map = Some(PathBuf::from(raw));
            }
            "--set" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--set requires a value like key=value".to_string())?;
                let (k, v) = raw
                    .split_once('=')
                    .ok_or_else(|| "--set requires a value like key=value".to_string())?;
                let k = k.trim();
                let v = v.trim();
                if k.is_empty() || v.is_empty() {
                    return Err("--set requires a non-empty key and value".to_string());
                }
                sets.push((k.to_string(), v.to_string()));
            }
            "--all-tags" => all_tags = true,
            "--force" => force = true,
            "--help" | "-h" => return crate::cli::help(),
            other if other.starts_with('-') => {
                return Err(format!("unknown argument for theme import-vscode: {other}"));
            }
            other => {
                if input.is_some() {
                    return Err(format!("unexpected extra argument: {other}"));
                }
                input = Some(PathBuf::from(other));
            }
        }
    }

    let input = input.ok_or_else(|| "missing input theme json".to_string())?;
    let bytes = std::fs::read(&input).map_err(|e| e.to_string())?;

    let mut mapping: Option<VscodeImportMapping> = if let Some(map_path) = map {
        let map_bytes = std::fs::read(&map_path).map_err(|e| e.to_string())?;
        let mapping: VscodeImportMapping =
            serde_json::from_slice(&map_bytes).map_err(|e| e.to_string())?;
        Some(mapping)
    } else {
        None
    };

    if !sets.is_empty() {
        let m = mapping.get_or_insert_with(VscodeImportMapping::default);
        for (k, v) in sets {
            if k.starts_with("color.syntax.") {
                m.tokens.insert(
                    k,
                    MappingEntry {
                        scopes: Vec::new(),
                        foreground: Some(v),
                    },
                );
            } else {
                m.highlights.insert(
                    k,
                    MappingEntry {
                        scopes: Vec::new(),
                        foreground: Some(v),
                    },
                );
            }
        }
    }

    let (patch, report_data) =
        syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping(
            &bytes,
            VscodeSyntaxImportOptions {
                generate_all_fret_syntax_tokens: all_tags,
            },
            mapping.as_ref(),
        )
        .map_err(|e| e.to_string())?;

    let patch_name = patch.name.clone();
    let patch_token_count = patch.colors.len();
    if patch_token_count == 0 {
        return Err("no syntax colors were extracted from tokenColors".to_string());
    }

    if let Some(report_path) = report {
        if report_path.exists() && !force {
            return Err(format!(
                "refusing to overwrite existing file: {} (use --force)",
                report_path.display()
            ));
        }
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&report_data).map_err(|e| e.to_string())?;
        std::fs::write(&report_path, json).map_err(|e| e.to_string())?;
        println!("wrote {}", report_path.display());
    }

    let (out_cfg, applied, inserted) = if let Some(base_path) = base {
        let base_bytes = std::fs::read(&base_path).map_err(|e| e.to_string())?;
        let mut base_cfg: ThemeConfig =
            serde_json::from_slice(&base_bytes).map_err(|e| e.to_string())?;

        let mut inserted = 0usize;
        for (k, v) in patch.colors {
            if base_cfg.colors.insert(k, v).is_none() {
                inserted += 1;
            }
        }

        base_cfg.name = name_override.unwrap_or_else(|| {
            format!(
                "{} + {}",
                base_cfg.name.trim(),
                patch_name.trim().trim_start_matches("vscode/"),
            )
        });

        (base_cfg, patch_token_count, inserted)
    } else {
        let mut cfg = patch;
        if let Some(name) = name_override {
            cfg.name = name;
        }
        let inserted = cfg.colors.len();
        (cfg, inserted, inserted)
    };

    let json = serde_json::to_string_pretty(&out_cfg).map_err(|e| e.to_string())?;

    match out {
        Some(path) => {
            if path.exists() && !force {
                return Err(format!(
                    "refusing to overwrite existing file: {} (use --force)",
                    path.display()
                ));
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
            println!(
                "wrote {} (applied {} syntax token{}; new {})",
                path.display(),
                applied,
                if applied == 1 { "" } else { "s" },
                inserted
            );
            Ok(())
        }
        None => {
            print!("{json}");
            Ok(())
        }
    }
}
