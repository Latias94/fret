use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fret_icons_generator::{
    PresentationOverride, PresentationRenderMode, normalize_svg_directory_icon_name,
};
use serde::Serialize;
use usvg::{ImageKind, Node, Paint};

use super::contracts::SuggestSvgDirPresentationOverridesArgs;

const SVG_DIR_PRESENTATION_OVERRIDES_REPORT_SCHEMA_V1: u32 = 1;
const PRESENTATION_DEFAULTS_CONFIG_SCHEMA_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuggestSvgDirPresentationOverridesReport {
    pub out_path: PathBuf,
    pub report_path: Option<PathBuf>,
    pub analyzed_icon_count: usize,
    pub suggested_override_count: usize,
    pub parse_failure_count: usize,
}

pub(crate) fn run_svg_dir_presentation_overrides_contract(
    args: SuggestSvgDirPresentationOverridesArgs,
) -> Result<SuggestSvgDirPresentationOverridesReport, String> {
    let SuggestSvgDirPresentationOverridesArgs {
        source,
        out,
        report_out,
    } = args;
    validate_source_dir(&source)?;
    validate_output_paths(&source, &out, report_out.as_deref())?;

    let svg_files = collect_svg_files(&source)?;
    if svg_files.is_empty() {
        return Err(format!("no SVG icons found in {}", source.display()));
    }

    let mut report_icons = Vec::with_capacity(svg_files.len());
    let mut icon_overrides = Vec::new();
    let mut parse_failure_count = 0;

    for relative_path in svg_files {
        let icon_name = normalize_svg_directory_icon_name(&relative_path)
            .map_err(|err| format!("failed to derive icon name: {err}"))?;
        let source_relative_path = path_label(&relative_path);
        let absolute_path = source.join(&relative_path);
        let svg_bytes = std::fs::read(&absolute_path)
            .map_err(|err| format!("failed to read SVG `{}`: {err}", absolute_path.display()))?;

        match analyze_svg_bytes(&svg_bytes) {
            Ok(analysis) => {
                let recommendation = recommendation_for_analysis(&analysis);
                if recommendation == SvgPresentationRecommendation::SuggestOriginalColorsOverride {
                    icon_overrides.push(PresentationOverride {
                        icon_name: icon_name.clone(),
                        render_mode: PresentationRenderMode::OriginalColors,
                    });
                }
                report_icons.push(SvgPresentationIconReport::from_analysis(
                    icon_name,
                    source_relative_path,
                    recommendation,
                    analysis,
                ));
            }
            Err(err) => {
                parse_failure_count += 1;
                report_icons.push(SvgPresentationIconReport::parse_error(
                    icon_name,
                    source_relative_path,
                    err,
                ));
            }
        }
    }

    let config = SvgPresentationOverridesConfigFileV1 {
        schema_version: PRESENTATION_DEFAULTS_CONFIG_SCHEMA_V1,
        default_render_mode: None,
        icon_overrides,
    };
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|err| format!("failed to serialize presentation overrides suggestion: {err}"))?;
    write_text_file(&out, &config_json)?;

    let analyzed_icon_count = report_icons.len();
    let suggested_override_count = config.icon_overrides.len();
    let report_path = match report_out {
        Some(report_out) => {
            let report = SvgDirPresentationOverridesReportFileV1::new(
                &source,
                &out,
                analyzed_icon_count,
                suggested_override_count,
                parse_failure_count,
                report_icons,
            );
            let report_json = serde_json::to_string_pretty(&report)
                .map_err(|err| format!("failed to serialize svg analysis report: {err}"))?;
            write_text_file(&report_out, &report_json)?;
            Some(report_out)
        }
        None => None,
    };

    Ok(SuggestSvgDirPresentationOverridesReport {
        out_path: out,
        report_path,
        analyzed_icon_count,
        suggested_override_count,
        parse_failure_count,
    })
}

fn validate_source_dir(source: &Path) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("missing source directory: {}", source.display()));
    }
    if !source.is_dir() {
        return Err(format!(
            "source path is not a directory: {}",
            source.display()
        ));
    }
    Ok(())
}

fn validate_output_paths(
    source: &Path,
    config_out: &Path,
    report_out: Option<&Path>,
) -> Result<(), String> {
    if config_out == source {
        return Err(format!(
            "suggestion output path must differ from source directory: {}",
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
    if report_out == source {
        return Err(format!(
            "report output path must differ from source directory: {}",
            report_out.display()
        ));
    }
    Ok(())
}

fn collect_svg_files(source: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_svg_paths_recursive(source, source, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_svg_paths_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries = std::fs::read_dir(current)
        .map_err(|err| format!("failed to read directory `{}`: {err}", current.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to read directory `{}`: {err}", current.display()))?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_svg_paths_recursive(root, &path, out)?;
            continue;
        }
        let is_svg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));
        if !is_svg {
            continue;
        }
        let relative = path.strip_prefix(root).map_err(|err| {
            format!(
                "failed to derive relative SVG path for `{}`: {err}",
                path.display()
            )
        })?;
        out.push(relative.to_path_buf());
    }

    Ok(())
}

fn analyze_svg_bytes(bytes: &[u8]) -> Result<SvgPresentationAnalysis, String> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &options)
        .map_err(|err| format!("failed to parse SVG: {err}"))?;

    let mut analysis = SvgPresentationAnalysis {
        has_text_nodes: tree.has_text_nodes(),
        ..Default::default()
    };
    collect_group_analysis(tree.root(), &mut analysis);
    Ok(analysis)
}

fn collect_group_analysis(group: &usvg::Group, analysis: &mut SvgPresentationAnalysis) {
    for node in group.children() {
        collect_node_analysis(node, analysis);
        node.subroots(|subroot| collect_group_analysis(subroot, analysis));
    }
}

fn collect_node_analysis(node: &usvg::Node, analysis: &mut SvgPresentationAnalysis) {
    match node {
        Node::Group(group) => collect_group_analysis(group, analysis),
        Node::Path(path) => {
            if let Some(fill) = path.fill() {
                collect_paint_analysis(fill.paint(), analysis);
            }
            if let Some(stroke) = path.stroke() {
                collect_paint_analysis(stroke.paint(), analysis);
            }
        }
        Node::Image(image) => match image.kind() {
            ImageKind::JPEG(_) | ImageKind::PNG(_) | ImageKind::GIF(_) | ImageKind::WEBP(_) => {
                analysis.has_embedded_raster_image = true;
            }
            ImageKind::SVG(_) => {
                analysis.has_embedded_svg_image = true;
            }
        },
        Node::Text(_) => {
            analysis.has_text_nodes = true;
        }
    }
}

fn collect_paint_analysis(paint: &Paint, analysis: &mut SvgPresentationAnalysis) {
    match paint {
        Paint::Color(color) => {
            analysis.solid_colors.insert(format_color(color));
        }
        Paint::LinearGradient(_) | Paint::RadialGradient(_) => {
            analysis.has_gradient_paint = true;
        }
        Paint::Pattern(_) => {
            analysis.has_pattern_paint = true;
        }
    }
}

fn recommendation_for_analysis(
    analysis: &SvgPresentationAnalysis,
) -> SvgPresentationRecommendation {
    if analysis.solid_colors.len() >= 2
        || analysis.has_gradient_paint
        || analysis.has_pattern_paint
        || analysis.has_embedded_raster_image
        || analysis.has_embedded_svg_image
    {
        SvgPresentationRecommendation::SuggestOriginalColorsOverride
    } else {
        SvgPresentationRecommendation::NoOverride
    }
}

fn format_color(color: &usvg::Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

fn path_label(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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

#[derive(Debug, Clone, Default)]
struct SvgPresentationAnalysis {
    solid_colors: BTreeSet<String>,
    has_gradient_paint: bool,
    has_pattern_paint: bool,
    has_embedded_raster_image: bool,
    has_embedded_svg_image: bool,
    has_text_nodes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SvgPresentationRecommendation {
    SuggestOriginalColorsOverride,
    NoOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SvgPresentationParseStatus {
    Parsed,
    ParseError,
}

#[derive(Debug, Clone, Serialize)]
struct SvgPresentationOverridesConfigFileV1 {
    schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_render_mode: Option<PresentationRenderMode>,
    #[serde(default)]
    icon_overrides: Vec<PresentationOverride>,
}

#[derive(Debug, Clone, Serialize)]
struct SvgDirPresentationOverridesReportFileV1 {
    schema_version: u32,
    report_kind: String,
    source_dir: String,
    presentation_defaults_path: String,
    summary: SvgDirPresentationSummary,
    icons: Vec<SvgPresentationIconReport>,
    limitations: Vec<String>,
}

impl SvgDirPresentationOverridesReportFileV1 {
    fn new(
        source_dir: &Path,
        presentation_defaults_path: &Path,
        analyzed_icon_count: usize,
        suggested_override_count: usize,
        parse_failure_count: usize,
        icons: Vec<SvgPresentationIconReport>,
    ) -> Self {
        Self {
            schema_version: SVG_DIR_PRESENTATION_OVERRIDES_REPORT_SCHEMA_V1,
            report_kind: "svg-dir-presentation-overrides".to_string(),
            source_dir: source_dir.display().to_string(),
            presentation_defaults_path: presentation_defaults_path.display().to_string(),
            summary: SvgDirPresentationSummary {
                analyzed_icon_count,
                suggested_override_count,
                parse_failure_count,
                notes: vec![
                    "Only strong evidence produces `original-colors` overrides.".to_string(),
                    "This helper does not infer `default_render_mode`.".to_string(),
                    "Unlisted icons remain on the existing generator/import default path."
                        .to_string(),
                ],
            },
            icons,
            limitations: vec![
                "Conservative by design: single-color SVG assets remain unclassified even when they may be authored-color icons.".to_string(),
                "The helper suggests per-icon `original-colors` overrides only; it does not guess a pack-level default.".to_string(),
                "SVG parsing is best-effort for analysis. Parse failures stay in the report and do not produce overrides.".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct SvgDirPresentationSummary {
    analyzed_icon_count: usize,
    suggested_override_count: usize,
    parse_failure_count: usize,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SvgPresentationIconReport {
    icon_name: String,
    source_relative_path: String,
    parse_status: SvgPresentationParseStatus,
    recommendation: SvgPresentationRecommendation,
    observed_solid_colors: Vec<String>,
    has_gradient_paint: bool,
    has_pattern_paint: bool,
    has_embedded_raster_image: bool,
    has_embedded_svg_image: bool,
    has_text_nodes: bool,
    evidence: Vec<String>,
    warnings: Vec<String>,
}

impl SvgPresentationIconReport {
    fn from_analysis(
        icon_name: String,
        source_relative_path: String,
        recommendation: SvgPresentationRecommendation,
        analysis: SvgPresentationAnalysis,
    ) -> Self {
        let mut evidence = Vec::new();
        if analysis.solid_colors.len() >= 2 {
            evidence.push(format!(
                "multiple distinct solid colors detected: {}",
                analysis
                    .solid_colors
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if analysis.has_gradient_paint {
            evidence.push("gradient paint detected".to_string());
        }
        if analysis.has_pattern_paint {
            evidence.push("pattern paint detected".to_string());
        }
        if analysis.has_embedded_raster_image {
            evidence.push("embedded raster image detected".to_string());
        }
        if analysis.has_embedded_svg_image {
            evidence.push("embedded SVG image detected".to_string());
        }

        let mut warnings = Vec::new();
        if analysis.has_text_nodes {
            warnings.push(
                "text nodes detected; first-party SVG rendering currently expects text-free assets"
                    .to_string(),
            );
        }
        if recommendation == SvgPresentationRecommendation::NoOverride {
            warnings.push(
                "no strong authored-color signal detected; no override was suggested".to_string(),
            );
        }

        Self {
            icon_name,
            source_relative_path,
            parse_status: SvgPresentationParseStatus::Parsed,
            recommendation,
            observed_solid_colors: analysis.solid_colors.into_iter().collect(),
            has_gradient_paint: analysis.has_gradient_paint,
            has_pattern_paint: analysis.has_pattern_paint,
            has_embedded_raster_image: analysis.has_embedded_raster_image,
            has_embedded_svg_image: analysis.has_embedded_svg_image,
            has_text_nodes: analysis.has_text_nodes,
            evidence,
            warnings,
        }
    }

    fn parse_error(icon_name: String, source_relative_path: String, error: String) -> Self {
        Self {
            icon_name,
            source_relative_path,
            parse_status: SvgPresentationParseStatus::ParseError,
            recommendation: SvgPresentationRecommendation::NoOverride,
            observed_solid_colors: Vec::new(),
            has_gradient_paint: false,
            has_pattern_paint: false,
            has_embedded_raster_image: false,
            has_embedded_svg_image: false,
            has_text_nodes: false,
            evidence: Vec::new(),
            warnings: vec![error],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_svg_dir_presentation_overrides_contract;
    use crate::icons::contracts::SuggestSvgDirPresentationOverridesArgs;
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

    #[test]
    fn svg_dir_analysis_suggests_original_colors_for_multicolor_assets_only() {
        let root = make_temp_dir("fretboard-svg-analysis-colors");
        let source = root.join("icons");
        let out = root.join("presentation-defaults.json");
        let report_out = root.join("presentation-defaults.report.json");
        std::fs::create_dir_all(&source).expect("create source dir");
        std::fs::write(
            source.join("brand-logo.svg"),
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#ff0000" d="M0 0h12v24H0z"/><path fill="#0000ff" d="M12 0h12v24H12z"/></svg>"##,
        )
        .expect("write multicolor svg");
        std::fs::write(
            source.join("close.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write monochrome svg");

        let result =
            run_svg_dir_presentation_overrides_contract(SuggestSvgDirPresentationOverridesArgs {
                source,
                out: out.clone(),
                report_out: Some(report_out.clone()),
            })
            .expect("svg-dir analysis should succeed");

        assert_eq!(result.analyzed_icon_count, 2);
        assert_eq!(result.suggested_override_count, 1);
        assert_eq!(result.parse_failure_count, 0);

        let config: Value =
            serde_json::from_str(&std::fs::read_to_string(out).expect("read config"))
                .expect("parse config");
        assert!(config.get("default_render_mode").is_none());
        assert_eq!(
            config["icon_overrides"].as_array().map(|items| items.len()),
            Some(1)
        );
        assert_eq!(config["icon_overrides"][0]["icon_name"], "brand-logo");
        assert_eq!(
            config["icon_overrides"][0]["render_mode"],
            "original-colors"
        );

        let report: Value =
            serde_json::from_str(&std::fs::read_to_string(report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(report["summary"]["suggested_override_count"], 1);
        assert_eq!(
            report["icons"][0]["recommendation"],
            "suggest-original-colors-override"
        );
        assert_eq!(report["icons"][1]["recommendation"], "no-override");
    }

    #[test]
    fn svg_dir_analysis_keeps_single_color_non_black_assets_unclassified() {
        let root = make_temp_dir("fretboard-svg-analysis-single-color");
        let source = root.join("icons");
        let out = root.join("presentation-defaults.json");
        std::fs::create_dir_all(&source).expect("create source dir");
        std::fs::write(
            source.join("brand-mark.svg"),
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#ff0000" d="M0 0h24v24H0z"/></svg>"##,
        )
        .expect("write single-color svg");

        let result =
            run_svg_dir_presentation_overrides_contract(SuggestSvgDirPresentationOverridesArgs {
                source,
                out: out.clone(),
                report_out: None,
            })
            .expect("svg-dir analysis should succeed");

        assert_eq!(result.suggested_override_count, 0);

        let config: Value =
            serde_json::from_str(&std::fs::read_to_string(out).expect("read config"))
                .expect("parse config");
        assert_eq!(
            config["icon_overrides"].as_array().map(|items| items.len()),
            Some(0)
        );
    }

    #[test]
    fn svg_dir_analysis_reports_parse_failures_without_failing_entire_run() {
        let root = make_temp_dir("fretboard-svg-analysis-parse-failure");
        let source = root.join("icons");
        let out = root.join("presentation-defaults.json");
        let report_out = root.join("presentation-defaults.report.json");
        std::fs::create_dir_all(&source).expect("create source dir");
        std::fs::write(
            source.join("broken.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg"><path"#,
        )
        .expect("write broken svg");

        let result =
            run_svg_dir_presentation_overrides_contract(SuggestSvgDirPresentationOverridesArgs {
                source,
                out,
                report_out: Some(report_out.clone()),
            })
            .expect("analysis should keep parse failure in report");

        assert_eq!(result.parse_failure_count, 1);

        let report: Value =
            serde_json::from_str(&std::fs::read_to_string(report_out).expect("read report"))
                .expect("parse report");
        assert_eq!(report["summary"]["parse_failure_count"], 1);
        assert_eq!(report["icons"][0]["parse_status"], "parse-error");
        assert!(
            report["icons"][0]["warnings"][0]
                .as_str()
                .is_some_and(|warning| warning.contains("failed to parse SVG"))
        );
    }

    #[test]
    fn svg_dir_analysis_rejects_report_output_that_matches_config_output() {
        let root = make_temp_dir("fretboard-svg-analysis-report-conflict");
        let source = root.join("icons");
        let out = root.join("presentation-defaults.json");
        std::fs::create_dir_all(&source).expect("create source dir");
        std::fs::write(
            source.join("close.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write svg");

        let err =
            run_svg_dir_presentation_overrides_contract(SuggestSvgDirPresentationOverridesArgs {
                source,
                out: out.clone(),
                report_out: Some(out),
            })
            .expect_err("path conflict should fail");

        assert!(err.contains("report output path must differ"));
    }
}
