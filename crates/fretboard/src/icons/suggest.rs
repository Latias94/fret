use std::path::{Path, PathBuf};

use fret_icons_generator::{PresentationDefaultsConfigFileV1, PresentationRenderMode};
use serde::Deserialize;

use super::contracts::SuggestPresentationDefaultsArgs;

const ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuggestPresentationDefaultsReport {
    pub out_path: PathBuf,
    pub collection: String,
    pub default_render_mode: PresentationRenderMode,
    pub palette: bool,
}

pub(crate) fn run_presentation_defaults_suggestion_contract(
    args: SuggestPresentationDefaultsArgs,
) -> Result<SuggestPresentationDefaultsReport, String> {
    let provenance = load_iconify_acquisition_provenance(&args.provenance)?;
    if provenance.schema_version != ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1 {
        return Err(format!(
            "unsupported acquisition provenance schema version: expected {}, got {}",
            ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1, provenance.schema_version
        ));
    }
    if provenance.acquisition_kind != "iconify-collection" {
        return Err(format!(
            "unsupported acquisition provenance kind `{}`: only `iconify-collection` can derive presentation defaults",
            provenance.acquisition_kind
        ));
    }

    let palette = provenance
        .upstream
        .collection_info
        .and_then(|info| info.palette)
        .ok_or_else(|| {
            format!(
                "acquisition provenance `{}` does not contain `upstream.collection_info.palette`; cannot derive presentation defaults safely",
                args.provenance.display()
            )
        })?;

    let default_render_mode = if palette {
        PresentationRenderMode::OriginalColors
    } else {
        PresentationRenderMode::Mask
    };

    let suggestion = PresentationDefaultsConfigFileV1 {
        schema_version: 1,
        default_render_mode: Some(default_render_mode),
        icon_overrides: Vec::new(),
    };

    let suggestion_json = serde_json::to_string_pretty(&suggestion)
        .map_err(|err| format!("failed to serialize presentation-defaults suggestion: {err}"))?;
    write_text_file(&args.out, &suggestion_json)?;

    Ok(SuggestPresentationDefaultsReport {
        out_path: args.out,
        collection: provenance.collection,
        default_render_mode,
        palette,
    })
}

fn load_iconify_acquisition_provenance(
    path: &Path,
) -> Result<IconifyAcquisitionProvenanceV1, String> {
    if !path.exists() {
        return Err(format!(
            "missing acquisition provenance file: {}",
            path.display()
        ));
    }
    if !path.is_file() {
        return Err(format!(
            "acquisition provenance path is not a file: {}",
            path.display()
        ));
    }

    let content = std::fs::read_to_string(path).map_err(|err| {
        format!(
            "failed to read acquisition provenance `{}`: {err}",
            path.display()
        )
    })?;
    serde_json::from_str(&content).map_err(|err| {
        format!(
            "failed to parse acquisition provenance `{}`: {err}",
            path.display()
        )
    })
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create parent directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    Ok(())
}

fn write_text_file(path: &Path, contents: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    std::fs::write(path, contents)
        .map_err(|err| format!("failed to write `{}`: {err}", path.display()))
}

#[derive(Debug, Clone, Deserialize)]
struct IconifyAcquisitionProvenanceV1 {
    schema_version: u32,
    acquisition_kind: String,
    collection: String,
    upstream: UpstreamMetadataRecord,
}

#[derive(Debug, Clone, Deserialize)]
struct UpstreamMetadataRecord {
    #[serde(default)]
    collection_info: Option<IconifyCollectionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct IconifyCollectionInfo {
    #[serde(default)]
    palette: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::run_presentation_defaults_suggestion_contract;
    use crate::icons::contracts::SuggestPresentationDefaultsArgs;
    use fret_icons_generator::PresentationRenderMode;
    use serde_json::Value;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_provenance(path: &std::path::Path, palette: Option<bool>) {
        let palette_field = match palette {
            Some(value) => format!("\"palette\": {value}"),
            None => String::new(),
        };
        let collection_info = if palette_field.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {palette_field} }}")
        };
        std::fs::write(
            path,
            format!(
                r#"{{
  "schema_version": 1,
  "acquisition_kind": "iconify-collection",
  "collection": "mdi",
  "request": {{ "mode": "subset", "requested_icons": ["home"] }},
  "source": {{
    "api_base_url": "https://api.iconify.design",
    "collection_info_url": "https://api.iconify.design/collection?prefix=mdi&info=true",
    "icons_url": "https://api.iconify.design/mdi.json?icons=home"
  }},
  "upstream": {{
    "title": "Material Design Icons",
    "total": 1,
    "collection_info": {collection_info}
  }},
  "snapshot": {{
    "digest_algorithm": "blake3",
    "digest_hex": "deadbeef",
    "icon_count": 1,
    "alias_count": 0
  }}
}}"#
            ),
        )
        .expect("write provenance");
    }

    #[test]
    fn suggestion_derives_mask_default_when_palette_is_false() {
        let root = make_temp_dir("fretboard-suggest-presentation-mask");
        let provenance = root.join("mdi.provenance.json");
        let out = root.join("presentation-defaults.json");
        write_provenance(&provenance, Some(false));

        let report =
            run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
                provenance,
                out: out.clone(),
            })
            .expect("suggestion should succeed");

        assert_eq!(report.default_render_mode, PresentationRenderMode::Mask);
        let json: Value =
            serde_json::from_str(&std::fs::read_to_string(out).expect("read suggested config"))
                .expect("parse suggested config");
        assert_eq!(json["default_render_mode"], "mask");
        assert_eq!(json["icon_overrides"].as_array().map(|v| v.len()), Some(0));
    }

    #[test]
    fn suggestion_derives_original_colors_default_when_palette_is_true() {
        let root = make_temp_dir("fretboard-suggest-presentation-colors");
        let provenance = root.join("mdi.provenance.json");
        let out = root.join("presentation-defaults.json");
        write_provenance(&provenance, Some(true));

        let report =
            run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
                provenance,
                out: out.clone(),
            })
            .expect("suggestion should succeed");

        assert_eq!(
            report.default_render_mode,
            PresentationRenderMode::OriginalColors
        );
        let json: Value =
            serde_json::from_str(&std::fs::read_to_string(out).expect("read suggested config"))
                .expect("parse suggested config");
        assert_eq!(json["default_render_mode"], "original-colors");
    }

    #[test]
    fn suggestion_rejects_missing_palette_hint() {
        let root = make_temp_dir("fretboard-suggest-presentation-missing-palette");
        let provenance = root.join("mdi.provenance.json");
        let out = root.join("presentation-defaults.json");
        write_provenance(&provenance, None);

        let err = run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
            provenance,
            out,
        })
        .expect_err("missing palette should fail");

        assert!(err.contains("cannot derive presentation defaults safely"));
    }
}
