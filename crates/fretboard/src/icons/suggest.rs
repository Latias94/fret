use std::path::{Path, PathBuf};

use fret_icons_generator::{PresentationDefaultsConfigFileV1, PresentationRenderMode};
use serde::{Deserialize, Serialize};

use super::contracts::SuggestPresentationDefaultsArgs;

const ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1: u32 = 1;
const PRESENTATION_DEFAULTS_SUGGESTION_REPORT_SCHEMA_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuggestPresentationDefaultsReport {
    pub out_path: PathBuf,
    pub report_path: Option<PathBuf>,
    pub collection: String,
    pub default_render_mode: PresentationRenderMode,
    pub palette: bool,
}

pub(crate) fn run_presentation_defaults_suggestion_contract(
    args: SuggestPresentationDefaultsArgs,
) -> Result<SuggestPresentationDefaultsReport, String> {
    let SuggestPresentationDefaultsArgs {
        provenance,
        out,
        report_out,
    } = args;

    let provenance_record = load_iconify_acquisition_provenance(&provenance)?;
    if provenance_record.schema_version != ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1 {
        return Err(format!(
            "unsupported acquisition provenance schema version: expected {}, got {}",
            ICONIFY_ACQUISITION_PROVENANCE_SCHEMA_V1, provenance_record.schema_version
        ));
    }
    if provenance_record.acquisition_kind != "iconify-collection" {
        return Err(format!(
            "unsupported acquisition provenance kind `{}`: only `iconify-collection` can derive presentation defaults",
            provenance_record.acquisition_kind
        ));
    }

    let palette = provenance_record
        .upstream
        .collection_info
        .as_ref()
        .and_then(|info| info.palette)
        .ok_or_else(|| {
            format!(
                "acquisition provenance `{}` does not contain `upstream.collection_info.palette`; cannot derive presentation defaults safely",
                provenance.display()
            )
        })?;

    let default_render_mode = if palette {
        PresentationRenderMode::OriginalColors
    } else {
        PresentationRenderMode::Mask
    };

    validate_output_paths(&provenance, &out, report_out.as_deref())?;

    let suggestion = PresentationDefaultsConfigFileV1 {
        schema_version: 1,
        default_render_mode: Some(default_render_mode),
        icon_overrides: Vec::new(),
    };

    let suggestion_json = serde_json::to_string_pretty(&suggestion)
        .map_err(|err| format!("failed to serialize presentation-defaults suggestion: {err}"))?;
    write_text_file(&out, &suggestion_json)?;

    let report_path = match report_out {
        Some(report_out) => {
            let report = PresentationDefaultsSuggestionReportFileV1::from_provenance(
                &provenance,
                &out,
                &provenance_record,
                palette,
                default_render_mode,
            );
            let report_json = serde_json::to_string_pretty(&report)
                .map_err(|err| format!("failed to serialize suggestion report: {err}"))?;
            write_text_file(&report_out, &report_json)?;
            Some(report_out)
        }
        None => None,
    };

    Ok(SuggestPresentationDefaultsReport {
        out_path: out,
        report_path,
        collection: provenance_record.collection,
        default_render_mode,
        palette,
    })
}

fn validate_output_paths(
    provenance: &Path,
    config_out: &Path,
    report_out: Option<&Path>,
) -> Result<(), String> {
    if config_out == provenance {
        return Err(format!(
            "suggestion output path must differ from provenance path: {}",
            config_out.display()
        ));
    }

    let Some(report_out) = report_out else {
        return Ok(());
    };

    if report_out == config_out {
        return Err(format!(
            "report output path must differ from suggestion output path: {}",
            report_out.display()
        ));
    }
    if report_out == provenance {
        return Err(format!(
            "report output path must differ from provenance path: {}",
            report_out.display()
        ));
    }
    Ok(())
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
    #[serde(default)]
    snapshot: SnapshotRecord,
}

#[derive(Debug, Clone, Deserialize)]
struct UpstreamMetadataRecord {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    total: Option<u32>,
    #[serde(default)]
    collection_info: Option<IconifyCollectionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct IconifyCollectionInfo {
    #[serde(default)]
    palette: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SnapshotRecord {
    #[serde(default)]
    icon_count: Option<usize>,
    #[serde(default)]
    alias_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct PresentationDefaultsSuggestionReportFileV1 {
    schema_version: u32,
    report_kind: String,
    provenance_path: String,
    presentation_defaults_path: String,
    source: SuggestionSourceRecord,
    suggestion: SuggestionDecisionRecord,
    summary: Vec<String>,
    limitations: Vec<String>,
}

impl PresentationDefaultsSuggestionReportFileV1 {
    fn from_provenance(
        provenance_path: &Path,
        config_path: &Path,
        provenance: &IconifyAcquisitionProvenanceV1,
        palette: bool,
        default_render_mode: PresentationRenderMode,
    ) -> Self {
        let render_mode_label = match default_render_mode {
            PresentationRenderMode::Mask => "mask",
            PresentationRenderMode::OriginalColors => "original-colors",
        };
        let collection_title = provenance.upstream.title.clone();
        let source = SuggestionSourceRecord {
            acquisition_kind: provenance.acquisition_kind.clone(),
            collection: provenance.collection.clone(),
            collection_title: collection_title.clone(),
            palette,
            upstream_total: provenance.upstream.total,
            snapshot_icon_count: provenance.snapshot.icon_count,
            snapshot_alias_count: provenance.snapshot.alias_count,
        };
        let suggestion = SuggestionDecisionRecord {
            default_render_mode,
            icon_override_count: 0,
            evidence: vec![format!(
                "upstream.collection_info.palette={palette} -> default_render_mode={render_mode_label}"
            )],
        };

        let mut summary = vec![format!(
            "Collection `{}` declared `palette={palette}` in acquisition provenance.",
            provenance.collection
        )];
        if let Some(title) = collection_title {
            summary.push(format!("Upstream title: `{title}`."));
        }
        summary.push(format!(
            "Suggested pack-level `default_render_mode` is `{render_mode_label}`."
        ));
        summary.push(
            "No per-icon overrides were inferred; the emitted config stays pack-default only."
                .to_string(),
        );

        let limitations = vec![
            "Advisory only: `icons import ...` still requires the emitted `presentation-defaults.json` explicitly.".to_string(),
            "Derivation uses collection-level Iconify provenance only; it does not inspect SVG bodies or infer per-icon overrides.".to_string(),
            "If `upstream.collection_info.palette` is missing, the helper must fail instead of guessing.".to_string(),
        ];

        Self {
            schema_version: PRESENTATION_DEFAULTS_SUGGESTION_REPORT_SCHEMA_V1,
            report_kind: "iconify-presentation-defaults-suggestion".to_string(),
            provenance_path: provenance_path.display().to_string(),
            presentation_defaults_path: config_path.display().to_string(),
            source,
            suggestion,
            summary,
            limitations,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct SuggestionSourceRecord {
    acquisition_kind: String,
    collection: String,
    collection_title: Option<String>,
    palette: bool,
    upstream_total: Option<u32>,
    snapshot_icon_count: Option<usize>,
    snapshot_alias_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct SuggestionDecisionRecord {
    default_render_mode: PresentationRenderMode,
    icon_override_count: usize,
    evidence: Vec<String>,
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
                report_out: None,
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
                report_out: None,
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
            report_out: None,
        })
        .expect_err("missing palette should fail");

        assert!(err.contains("cannot derive presentation defaults safely"));
    }

    #[test]
    fn suggestion_writes_versioned_review_report_when_requested() {
        let root = make_temp_dir("fretboard-suggest-presentation-report");
        let provenance = root.join("mdi.provenance.json");
        let out = root.join("presentation-defaults.json");
        let report_out = root.join("presentation-defaults.report.json");
        write_provenance(&provenance, Some(true));

        let report =
            run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
                provenance: provenance.clone(),
                out: out.clone(),
                report_out: Some(report_out.clone()),
            })
            .expect("suggestion should succeed");

        assert_eq!(report.report_path.as_deref(), Some(report_out.as_path()));

        let json: Value = serde_json::from_str(
            &std::fs::read_to_string(&report_out).expect("read suggestion report"),
        )
        .expect("parse suggestion report");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(
            json["report_kind"],
            Value::String("iconify-presentation-defaults-suggestion".to_string())
        );
        assert_eq!(
            json["presentation_defaults_path"],
            Value::String(out.display().to_string())
        );
        assert_eq!(json["source"]["collection"], "mdi");
        assert_eq!(json["source"]["palette"], true);
        assert_eq!(json["source"]["snapshot_icon_count"], 1);
        assert_eq!(json["suggestion"]["default_render_mode"], "original-colors");
        assert_eq!(json["suggestion"]["icon_override_count"], 0);
        assert!(
            json["summary"]
                .as_array()
                .expect("summary should be an array")
                .iter()
                .any(|line| line
                    .as_str()
                    .is_some_and(|line| line.contains("default_render_mode")))
        );
        assert!(
            json["limitations"]
                .as_array()
                .expect("limitations should be an array")
                .iter()
                .any(|line| line
                    .as_str()
                    .is_some_and(|line| line.contains("Advisory only")))
        );
    }

    #[test]
    fn suggestion_rejects_report_output_that_matches_config_output() {
        let root = make_temp_dir("fretboard-suggest-presentation-report-conflict");
        let provenance = root.join("mdi.provenance.json");
        let out = root.join("presentation-defaults.json");
        write_provenance(&provenance, Some(true));

        let err = run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
            provenance,
            out: out.clone(),
            report_out: Some(out),
        })
        .expect_err("report output path conflict should fail");

        assert!(err.contains("report output path must differ"));
    }

    #[test]
    fn suggestion_rejects_output_that_matches_provenance_path() {
        let root = make_temp_dir("fretboard-suggest-presentation-output-conflict");
        let provenance = root.join("mdi.provenance.json");
        write_provenance(&provenance, Some(true));

        let err = run_presentation_defaults_suggestion_contract(SuggestPresentationDefaultsArgs {
            provenance: provenance.clone(),
            out: provenance,
            report_out: None,
        })
        .expect_err("config output path conflict should fail");

        assert!(err.contains("suggestion output path must differ"));
    }
}
