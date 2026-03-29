use fret_ui::theme::ThemeConfig;
use fret_vscode_theme::{
    MappingEntry, VscodeImportMapping, VscodeSyntaxImportOptions,
    syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping,
};

pub(crate) mod contracts;

use self::contracts::{ThemeCommandArgs, ThemeImportVscodeCommandArgs, ThemeTargetContract};

pub(crate) fn run_theme_contract(args: ThemeCommandArgs) -> Result<(), String> {
    match args.target {
        ThemeTargetContract::ImportVscode(args) => import_vscode_cmd(args),
    }
}

fn import_vscode_cmd(args: ThemeImportVscodeCommandArgs) -> Result<(), String> {
    let input = args.input;
    let out = args.out;
    let base = args.base;
    let report = args.report;
    let map = args.map;
    let all_tags = args.all_tags;
    let force = args.force;
    let name_override = args.name;

    let bytes = std::fs::read(&input).map_err(|e| e.to_string())?;

    let mut mapping: Option<VscodeImportMapping> = if let Some(map_path) = map {
        let map_bytes = std::fs::read(&map_path).map_err(|e| e.to_string())?;
        let mapping: VscodeImportMapping =
            serde_json::from_slice(&map_bytes).map_err(|e| e.to_string())?;
        Some(mapping)
    } else {
        None
    };

    if !args.sets.is_empty() {
        let m = mapping.get_or_insert_with(VscodeImportMapping::default);
        for (k, v) in args
            .sets
            .iter()
            .map(|raw| parse_mapping_set(raw))
            .collect::<Result<Vec<_>, _>>()?
        {
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

fn parse_mapping_set(raw: &str) -> Result<(String, String), String> {
    let (k, v) = raw
        .split_once('=')
        .ok_or_else(|| "--set requires a value like key=value".to_string())?;
    let k = k.trim();
    let v = v.trim();
    if k.is_empty() || v.is_empty() {
        return Err("--set requires a non-empty key and value".to_string());
    }
    Ok((k.to_string(), v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_mapping_set;

    #[test]
    fn parse_mapping_set_requires_non_empty_key_and_value() {
        assert_eq!(
            parse_mapping_set("color.syntax.keyword=#ff00aa"),
            Ok(("color.syntax.keyword".to_string(), "#ff00aa".to_string()))
        );
        assert!(parse_mapping_set("missing-separator").is_err());
        assert!(parse_mapping_set("=value").is_err());
        assert!(parse_mapping_set("key=").is_err());
    }
}
