use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::compare::{PerfBaselineUiThresholdMode, normalize_repo_relative_path};
use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfBaselineSeed {
    Max,
    P90,
    P95,
}

impl PerfBaselineSeed {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PerfBaselineSeed::Max => "max",
            PerfBaselineSeed::P90 => "p90",
            PerfBaselineSeed::P95 => "p95",
        }
    }
}

impl std::str::FromStr for PerfBaselineSeed {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "max" => Ok(PerfBaselineSeed::Max),
            "p90" => Ok(PerfBaselineSeed::P90),
            "p95" => Ok(PerfBaselineSeed::P95),
            _ => Err(format!("invalid seed (expected max|p90|p95): {s:?}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PerfThresholdTuning {
    pub(crate) min_slack_us: u64,
    pub(crate) quantum_us: u64,
}

impl Default for PerfThresholdTuning {
    fn default() -> Self {
        Self {
            min_slack_us: 0,
            quantum_us: 1,
        }
    }
}

impl PerfThresholdTuning {
    pub(crate) fn as_json(self) -> Option<Value> {
        let default = Self::default();
        if self == default {
            return None;
        }
        Some(serde_json::json!({
            "min_slack_us": self.min_slack_us,
            "quantum_us": self.quantum_us,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum PerfSeedMetric {
    TopTotalTimeUs,
    TopLayoutTimeUs,
    TopLayoutEngineSolveTimeUs,
    FrameP95TotalTimeUs,
    FrameP95LayoutTimeUs,
    FrameP95LayoutEngineSolveTimeUs,
    PointerMoveDispatchTimeUs,
    PointerMoveHitTestTimeUs,
    RendererEncodeSceneUs,
    RendererUploadUs,
    RendererRecordPassesUs,
    RendererEncoderFinishUs,
    RendererPrepareTextUs,
    RendererPrepareSvgUs,
}

impl PerfSeedMetric {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PerfSeedMetric::TopTotalTimeUs => "top_total_time_us",
            PerfSeedMetric::TopLayoutTimeUs => "top_layout_time_us",
            PerfSeedMetric::TopLayoutEngineSolveTimeUs => "top_layout_engine_solve_time_us",
            PerfSeedMetric::FrameP95TotalTimeUs => "frame_p95_total_time_us",
            PerfSeedMetric::FrameP95LayoutTimeUs => "frame_p95_layout_time_us",
            PerfSeedMetric::FrameP95LayoutEngineSolveTimeUs => {
                "frame_p95_layout_engine_solve_time_us"
            }
            PerfSeedMetric::PointerMoveDispatchTimeUs => "pointer_move_max_dispatch_time_us",
            PerfSeedMetric::PointerMoveHitTestTimeUs => "pointer_move_max_hit_test_time_us",
            PerfSeedMetric::RendererEncodeSceneUs => "renderer_encode_scene_us",
            PerfSeedMetric::RendererUploadUs => "renderer_upload_us",
            PerfSeedMetric::RendererRecordPassesUs => "renderer_record_passes_us",
            PerfSeedMetric::RendererEncoderFinishUs => "renderer_encoder_finish_us",
            PerfSeedMetric::RendererPrepareTextUs => "renderer_prepare_text_us",
            PerfSeedMetric::RendererPrepareSvgUs => "renderer_prepare_svg_us",
        }
    }
}

impl std::str::FromStr for PerfSeedMetric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "top_total_time_us" => Ok(PerfSeedMetric::TopTotalTimeUs),
            "top_layout_time_us" => Ok(PerfSeedMetric::TopLayoutTimeUs),
            "top_layout_engine_solve_time_us" => Ok(PerfSeedMetric::TopLayoutEngineSolveTimeUs),
            "frame_p95_total_time_us" => Ok(PerfSeedMetric::FrameP95TotalTimeUs),
            "frame_p95_layout_time_us" => Ok(PerfSeedMetric::FrameP95LayoutTimeUs),
            "frame_p95_layout_engine_solve_time_us" => {
                Ok(PerfSeedMetric::FrameP95LayoutEngineSolveTimeUs)
            }
            "pointer_move_max_dispatch_time_us" => Ok(PerfSeedMetric::PointerMoveDispatchTimeUs),
            "pointer_move_max_hit_test_time_us" => Ok(PerfSeedMetric::PointerMoveHitTestTimeUs),
            "renderer_encode_scene_us" => Ok(PerfSeedMetric::RendererEncodeSceneUs),
            "renderer_upload_us" => Ok(PerfSeedMetric::RendererUploadUs),
            "renderer_record_passes_us" => Ok(PerfSeedMetric::RendererRecordPassesUs),
            "renderer_encoder_finish_us" => Ok(PerfSeedMetric::RendererEncoderFinishUs),
            "renderer_prepare_text_us" => Ok(PerfSeedMetric::RendererPrepareTextUs),
            "renderer_prepare_svg_us" => Ok(PerfSeedMetric::RendererPrepareSvgUs),
            _ => Err(format!(
                "invalid metric (expected top_total_time_us|top_layout_time_us|top_layout_engine_solve_time_us|frame_p95_total_time_us|frame_p95_layout_time_us|frame_p95_layout_engine_solve_time_us|pointer_move_max_dispatch_time_us|pointer_move_max_hit_test_time_us|renderer_encode_scene_us|renderer_upload_us|renderer_record_passes_us|renderer_encoder_finish_us|renderer_prepare_text_us|renderer_prepare_svg_us): {s:?}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleSourceKind {
    Default,
    Preset,
    PresetSuite,
    Cli,
    CliSuite,
}

impl RuleSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            RuleSourceKind::Default => "default",
            RuleSourceKind::Preset => "preset",
            RuleSourceKind::PresetSuite => "preset-suite",
            RuleSourceKind::Cli => "cli",
            RuleSourceKind::CliSuite => "cli-suite",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPerfBaselineSeedPolicy {
    pub(crate) default_seed: PerfBaselineSeed,
    pub(crate) ui_threshold_mode: PerfBaselineUiThresholdMode,
    // Final per-(script, metric) override map (only for scripts in the current invocation).
    overrides: HashMap<(String, PerfSeedMetric), ResolvedPerfRule>,
    // Audit-friendly expanded rules (only for scripts in the current invocation).
    pub(crate) audit_rules: Vec<Value>,
}

impl ResolvedPerfBaselineSeedPolicy {
    pub(crate) fn seed_for(&self, script: &str, metric: PerfSeedMetric) -> PerfBaselineSeed {
        self.overrides
            .get(&(script.to_string(), metric))
            .map(|rule| rule.seed)
            .unwrap_or(self.default_seed)
    }

    pub(crate) fn tuning_for(&self, script: &str, metric: PerfSeedMetric) -> PerfThresholdTuning {
        self.overrides
            .get(&(script.to_string(), metric))
            .map(|rule| rule.tuning)
            .unwrap_or_default()
    }

    pub(crate) fn ui_threshold_mode(&self) -> PerfBaselineUiThresholdMode {
        self.ui_threshold_mode
    }

    pub(crate) fn threshold_seed_policy_json(&self) -> Value {
        Value::Object(
            [
                ("schema_version".to_string(), Value::from(1u64)),
                (
                    "default_seed".to_string(),
                    Value::String(self.default_seed.as_str().to_string()),
                ),
                (
                    "ui_threshold_mode".to_string(),
                    Value::String(self.ui_threshold_mode.as_str().to_string()),
                ),
                ("rules".to_string(), Value::Array(self.audit_rules.clone())),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Debug, Clone)]
struct SeedRuleSpec {
    scope: String,
    metric: PerfSeedMetric,
    seed: PerfBaselineSeed,
    min_slack_us: u64,
    quantum_us: u64,
    source: RuleSourceKind,
}

#[derive(Debug, Clone)]
struct SeedPresetFile {
    default_seed: Option<PerfBaselineSeed>,
    ui_threshold_mode: Option<PerfBaselineUiThresholdMode>,
    rules: Vec<(String, PerfSeedMetric, PerfBaselineSeed, u64, u64)>,
}

#[derive(Debug, Clone, Copy)]
struct ResolvedPerfRule {
    seed: PerfBaselineSeed,
    tuning: PerfThresholdTuning,
    source: RuleSourceKind,
}

fn perf_suite_membership_name(name: &str) -> Option<&str> {
    match name {
        "ui-gallery" | "perf-ui-gallery" => Some("perf-ui-gallery"),
        "ui-gallery-steady" | "perf-ui-gallery-steady" => Some("perf-ui-gallery-steady"),
        "ui-gallery-overlay-steady" | "perf-ui-gallery-overlay-steady" => {
            Some("perf-ui-gallery-overlay-steady")
        }
        "ui-gallery-overlay-interaction-steady" | "perf-ui-gallery-overlay-interaction-steady" => {
            Some("perf-ui-gallery-overlay-interaction-steady")
        }
        "ui-gallery-context-menu-right-click-steady"
        | "perf-ui-gallery-context-menu-right-click-steady" => {
            Some("perf-ui-gallery-context-menu-right-click-steady")
        }
        "ui-gallery-dialog-escape-focus-restore-steady"
        | "perf-ui-gallery-dialog-escape-focus-restore-steady" => {
            Some("perf-ui-gallery-dialog-escape-focus-restore-steady")
        }
        "ui-gallery-dropdown-open-select-steady"
        | "perf-ui-gallery-dropdown-open-select-steady" => {
            Some("perf-ui-gallery-dropdown-open-select-steady")
        }
        "ui-gallery-overlay-pointer-move-steady"
        | "perf-ui-gallery-overlay-pointer-move-steady" => {
            Some("perf-ui-gallery-overlay-pointer-move-steady")
        }
        "ui-gallery-overlay-torture-steady" | "perf-ui-gallery-overlay-torture-steady" => {
            Some("perf-ui-gallery-overlay-torture-steady")
        }
        "ui-gallery-hit-test-torture-steady" | "perf-ui-gallery-hit-test-torture-steady" => {
            Some("perf-ui-gallery-hit-test-torture-steady")
        }
        "ui-gallery-layout-steady" | "perf-ui-gallery-layout-steady" => {
            Some("perf-ui-gallery-layout-steady")
        }
        "ui-gallery-scroll-area" | "perf-ui-gallery-scroll-area" => {
            Some("perf-ui-gallery-scroll-area")
        }
        "ui-gallery-virtual-list" | "perf-ui-gallery-virtual-list" => {
            Some("perf-ui-gallery-virtual-list")
        }
        "ui-resize-probes" | "perf-ui-resize-probes" => Some("perf-ui-resize-probes"),
        "ui-code-editor-resize-probes" | "perf-ui-code-editor-resize-probes" => {
            Some("perf-ui-code-editor-resize-probes")
        }
        "ui-gallery-complex-steady" | "perf-ui-gallery-complex-steady" => {
            Some("perf-ui-gallery-complex-steady")
        }
        "ui-gallery-complex-typical" | "perf-ui-gallery-complex-typical" => {
            Some("perf-ui-gallery-complex-typical")
        }
        "ui-gallery-code-editor-torture-autoscroll-typical"
        | "perf-ui-gallery-code-editor-torture-autoscroll-typical" => {
            Some("perf-ui-gallery-code-editor-torture-autoscroll-typical")
        }
        "docking-arbitration-steady" | "perf-docking-arbitration-steady" => {
            Some("perf-docking-arbitration-steady")
        }
        "extras-marquee-steady" | "perf-extras-marquee-steady" => {
            Some("perf-extras-marquee-steady")
        }
        "liquid-glass-backdrop-warp-steady" | "perf-liquid-glass-backdrop-warp-steady" => {
            Some("perf-liquid-glass-backdrop-warp-steady")
        }
        _ if name.starts_with("perf-") => Some(name),
        _ => None,
    }
}

fn is_known_perf_suite_name(name: &str) -> bool {
    perf_suite_membership_name(name).is_some()
}

pub(crate) fn scripts_for_perf_suite_name(
    workspace_root: &Path,
    name: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(membership) = perf_suite_membership_name(name) else {
        return Ok(None);
    };

    let registry_path = promoted_registry_default_path(workspace_root);
    let registry = PromotedScriptRegistry::load_from_path(&registry_path)?;

    let mut entries: Vec<(&str, &str)> = registry
        .entries()
        .iter()
        .filter(|e| e.suite_memberships.iter().any(|s| s == membership))
        .map(|e| (e.id.as_str(), e.path.as_str()))
        .collect();
    entries
        .sort_by(|(a_id, a_path), (b_id, b_path)| a_id.cmp(b_id).then_with(|| a_path.cmp(b_path)));

    let out: Vec<String> = entries
        .into_iter()
        .map(|(_id, path)| path.to_string())
        .collect();
    if out.is_empty() {
        return Err(format!(
            "perf suite resolved to no scripts in promoted registry: suite={name:?} membership={membership:?}\n\
hint: ensure tools/diag-scripts/suites/{membership}/*.json exists and regenerate tools/diag-scripts/index.json via `cargo run -p fretboard-dev -- diag registry write`"
        ));
    }
    Ok(Some(out))
}

pub(crate) fn resolve_perf_baseline_seed_policy(
    workspace_root: &Path,
    suite_name: Option<&str>,
    scripts: &[PathBuf],
    preset_paths: &[PathBuf],
    cli_seed_specs: &[String],
    cli_ui_threshold_mode: Option<PerfBaselineUiThresholdMode>,
) -> Result<ResolvedPerfBaselineSeedPolicy, String> {
    let mut default_seed = PerfBaselineSeed::Max;
    let mut ui_threshold_mode = PerfBaselineUiThresholdMode::default();

    let scripts_by_key: BTreeMap<String, PathBuf> = scripts
        .iter()
        .map(|p| (normalize_repo_relative_path(workspace_root, p), p.clone()))
        .collect();

    // Layer 1: built-in defaults (minimal, but keep these stable).
    let mut specs: Vec<SeedRuleSpec> = Vec::new();
    for script_key in scripts_by_key.keys() {
        if script_key.ends_with("ui-gallery-window-resize-stress.json")
            || script_key.ends_with("ui-gallery-window-resize-stress-steady.json")
        {
            specs.push(SeedRuleSpec {
                scope: script_key.clone(),
                metric: PerfSeedMetric::TopTotalTimeUs,
                seed: PerfBaselineSeed::P95,
                min_slack_us: 0,
                quantum_us: 1,
                source: RuleSourceKind::Default,
            });
            specs.push(SeedRuleSpec {
                scope: script_key.clone(),
                metric: PerfSeedMetric::TopLayoutTimeUs,
                seed: PerfBaselineSeed::P95,
                min_slack_us: 0,
                quantum_us: 1,
                source: RuleSourceKind::Default,
            });
            specs.push(SeedRuleSpec {
                scope: script_key.clone(),
                metric: PerfSeedMetric::TopLayoutEngineSolveTimeUs,
                seed: PerfBaselineSeed::P95,
                min_slack_us: 0,
                quantum_us: 1,
                source: RuleSourceKind::Default,
            });
        }
    }

    // Layer 2: JSON presets (repeatable; applied in CLI order).
    for path in preset_paths {
        let preset = read_seed_preset(workspace_root, path)?;
        if let Some(seed) = preset.default_seed {
            default_seed = seed;
        }
        if let Some(mode) = preset.ui_threshold_mode {
            ui_threshold_mode = mode;
        }
        for (scope, metric, seed, min_slack_us, quantum_us) in preset.rules {
            let source = if scope_is_suite_like(&scope, suite_name) {
                RuleSourceKind::PresetSuite
            } else {
                RuleSourceKind::Preset
            };
            specs.push(SeedRuleSpec {
                scope,
                metric,
                seed,
                min_slack_us,
                quantum_us,
                source,
            });
        }
    }

    // Layer 3: explicit CLI overrides (repeatable; highest precedence).
    for spec in cli_seed_specs {
        let (scope, metric, seed, suite_like) = parse_cli_seed_spec(spec)?;
        specs.push(SeedRuleSpec {
            scope,
            metric,
            seed,
            min_slack_us: 0,
            quantum_us: 1,
            source: if suite_like {
                RuleSourceKind::CliSuite
            } else {
                RuleSourceKind::Cli
            },
        });
    }

    if let Some(mode) = cli_ui_threshold_mode {
        ui_threshold_mode = mode;
    }

    // Apply layered overrides (last match wins).
    let mut overrides: HashMap<(String, PerfSeedMetric), ResolvedPerfRule> = HashMap::new();
    let mut audit: HashMap<(String, PerfSeedMetric), ResolvedPerfRule> = HashMap::new();

    for spec in specs {
        let script_keys =
            expand_scope_to_script_keys(workspace_root, suite_name, &scripts_by_key, &spec.scope)?;
        for key in script_keys {
            let rule = ResolvedPerfRule {
                seed: spec.seed,
                tuning: PerfThresholdTuning {
                    min_slack_us: spec.min_slack_us,
                    quantum_us: spec.quantum_us,
                },
                source: spec.source,
            };
            overrides.insert((key.clone(), spec.metric), rule);
            audit.insert((key, spec.metric), rule);
        }
    }

    let mut audit_rules: Vec<Value> = Vec::new();
    for ((script, metric), rule) in audit.into_iter() {
        if rule.seed == default_seed && rule.tuning == PerfThresholdTuning::default() {
            continue;
        }
        let mut obj = serde_json::json!({
            "script": script,
            "metric": metric.as_str(),
            "seed": rule.seed.as_str(),
            "source": rule.source.as_str(),
        });
        if let Some(tuning) = rule.tuning.as_json() {
            if let Some(map) = obj.as_object_mut() {
                map.insert("tuning".to_string(), tuning);
            }
        }
        audit_rules.push(obj);
    }
    audit_rules.sort_by(|a, b| {
        let as_script = a
            .get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("script").and_then(|v| v.as_str()).unwrap_or(""));
        if as_script != std::cmp::Ordering::Equal {
            return as_script;
        }
        a.get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("metric").and_then(|v| v.as_str()).unwrap_or(""))
    });

    Ok(ResolvedPerfBaselineSeedPolicy {
        default_seed,
        ui_threshold_mode,
        overrides,
        audit_rules,
    })
}

fn scope_is_suite_like(scope: &str, suite_name: Option<&str>) -> bool {
    scope == "*"
        || scope == "this-suite"
        || scope.starts_with("suite:")
        || suite_name.is_some_and(|s| s == scope)
        || is_known_perf_suite_name(scope)
}

fn read_seed_preset(workspace_root: &Path, path: &Path) -> Result<SeedPresetFile, String> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };
    let bytes = std::fs::read(&resolved).map_err(|e| {
        format!(
            "failed to read perf baseline seed preset {}: {e}",
            resolved.display()
        )
    })?;
    let root: Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "failed to parse perf baseline seed preset JSON {}: {e}",
            resolved.display()
        )
    })?;

    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if schema_version != 1 {
        return Err(format!(
            "unsupported perf baseline seed preset schema_version={schema_version} (expected 1): {}",
            resolved.display()
        ));
    }
    let kind = root.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    if kind != "perf_baseline_seed_policy" {
        return Err(format!(
            "invalid perf baseline seed preset kind={kind:?} (expected \"perf_baseline_seed_policy\"): {}",
            resolved.display()
        ));
    }

    let default_seed = root
        .get("default_seed")
        .and_then(|v| v.as_str())
        .map(|s| s.parse::<PerfBaselineSeed>())
        .transpose()?;
    let ui_threshold_mode = root
        .get("ui_threshold_mode")
        .and_then(|v| v.as_str())
        .map(|s| s.parse::<PerfBaselineUiThresholdMode>())
        .transpose()?;

    let rules = root
        .get("rules")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            format!(
                "invalid perf baseline seed preset: missing rules array: {}",
                resolved.display()
            )
        })?;

    let mut out: Vec<(String, PerfSeedMetric, PerfBaselineSeed, u64, u64)> = Vec::new();
    for rule in rules {
        let Some(scope) = rule.get("scope").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(metric) = rule.get("metric").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(seed) = rule.get("seed").and_then(|v| v.as_str()) else {
            continue;
        };
        let min_slack_us = rule
            .get("min_slack_us")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let quantum_us = match rule.get("quantum_us").and_then(|v| v.as_u64()) {
            Some(0) => {
                return Err(format!(
                    "invalid perf baseline seed preset: quantum_us must be > 0: {}",
                    resolved.display()
                ));
            }
            Some(v) => v,
            None => 1,
        };
        out.push((
            scope.to_string(),
            metric.parse::<PerfSeedMetric>()?,
            seed.parse::<PerfBaselineSeed>()?,
            min_slack_us,
            quantum_us,
        ));
    }

    Ok(SeedPresetFile {
        default_seed,
        ui_threshold_mode,
        rules: out,
    })
}

fn parse_cli_seed_spec(
    spec: &str,
) -> Result<(String, PerfSeedMetric, PerfBaselineSeed, bool), String> {
    // `<scope>@<metric>=<seed>`
    let (scope, rest) = spec.split_once('@').ok_or_else(|| {
        format!("invalid --perf-baseline-seed spec (expected scope@metric=max|p90|p95): {spec:?}")
    })?;
    let (metric, seed) = rest.split_once('=').ok_or_else(|| {
        format!("invalid --perf-baseline-seed spec (expected scope@metric=max|p90|p95): {spec:?}")
    })?;
    let metric = metric.parse::<PerfSeedMetric>()?;
    let seed = seed.parse::<PerfBaselineSeed>()?;
    let suite_like = scope == "*"
        || scope == "this-suite"
        || scope.starts_with("suite:")
        || is_known_perf_suite_name(scope);
    Ok((scope.to_string(), metric, seed, suite_like))
}

fn expand_scope_to_script_keys(
    workspace_root: &Path,
    suite_name: Option<&str>,
    scripts_by_key: &BTreeMap<String, PathBuf>,
    scope: &str,
) -> Result<Vec<String>, String> {
    let all_keys: Vec<String> = scripts_by_key.keys().cloned().collect();
    let scope_norm = scope.replace('\\', "/");
    let scope = scope_norm.as_str();

    if scope == "*" || scope == "this-suite" || suite_name.is_some_and(|s| s == scope) {
        return Ok(all_keys);
    }

    if let Some(name) = scope.strip_prefix("suite:") {
        let Some(paths) = scripts_for_perf_suite_name(workspace_root, name)? else {
            return Err(format!("unknown perf suite in seed scope: {name:?}"));
        };
        let mut out: Vec<String> = Vec::new();
        for p in paths {
            let key = normalize_repo_relative_path(workspace_root, &workspace_root.join(p));
            if scripts_by_key.contains_key(&key) {
                out.push(key);
            }
        }
        return Ok(out);
    }

    if let Some(paths) = scripts_for_perf_suite_name(workspace_root, scope)? {
        let mut out: Vec<String> = Vec::new();
        for p in paths {
            let key = normalize_repo_relative_path(workspace_root, &workspace_root.join(p));
            if scripts_by_key.contains_key(&key) {
                out.push(key);
            }
        }
        return Ok(out);
    }

    // Treat as a single script path.
    let key = if Path::new(scope).is_absolute() {
        normalize_repo_relative_path(workspace_root, Path::new(scope))
    } else {
        normalize_repo_relative_path(workspace_root, &workspace_root.join(scope))
    };
    if scripts_by_key.contains_key(&key) {
        Ok(vec![key])
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_policy_preset_and_cli_overrides_apply_in_order() {
        let workspace_root = std::env::temp_dir().join("fret-diag-seed-policy-test");
        let script_path =
            workspace_root.join("tools/diag-scripts/extras/extras-marquee-steady.json");
        let scripts = vec![script_path];

        let preset_path = workspace_root.join("preset.json");
        std::fs::create_dir_all(&workspace_root).unwrap();
        std::fs::write(
            &preset_path,
            r#"{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "rules": [
    { "scope": "extras-marquee-steady", "metric": "top_total_time_us", "seed": "p90" },
    { "scope": "extras-marquee-steady", "metric": "top_layout_time_us", "seed": "p90", "min_slack_us": 12, "quantum_us": 4 },
    { "scope": "tools/diag-scripts/extras/extras-marquee-steady.json", "metric": "top_layout_engine_solve_time_us", "seed": "p95", "min_slack_us": 24, "quantum_us": 8 }
  ]
}"#,
        )
        .unwrap();

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("extras-marquee-steady"),
            &scripts,
            std::slice::from_ref(&preset_path),
            &[],
            None,
        )
        .unwrap();

        let script_key = "tools/diag-scripts/extras/extras-marquee-steady.json";
        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::TopTotalTimeUs),
            PerfBaselineSeed::P90
        );
        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::TopLayoutTimeUs),
            PerfBaselineSeed::P90
        );
        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs),
            PerfBaselineSeed::P95
        );
        assert_eq!(
            policy.tuning_for(script_key, PerfSeedMetric::TopTotalTimeUs),
            PerfThresholdTuning::default()
        );
        assert_eq!(
            policy.tuning_for(script_key, PerfSeedMetric::TopLayoutTimeUs),
            PerfThresholdTuning {
                min_slack_us: 12,
                quantum_us: 4,
            }
        );
        assert_eq!(
            policy.tuning_for(script_key, PerfSeedMetric::TopLayoutEngineSolveTimeUs),
            PerfThresholdTuning {
                min_slack_us: 24,
                quantum_us: 8,
            }
        );

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("extras-marquee-steady"),
            &scripts,
            &[preset_path],
            &[String::from("this-suite@top_total_time_us=p95")],
            None,
        )
        .unwrap();

        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::TopTotalTimeUs),
            PerfBaselineSeed::P95
        );
    }

    #[test]
    fn built_in_defaults_cover_ui_gallery_resize_stress() {
        let workspace_root = std::env::temp_dir().join("fret-diag-seed-policy-test-defaults");
        let scripts = vec![workspace_root.join(
            "tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json",
        )];
        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("ui-gallery-steady"),
            &scripts,
            &[],
            &[],
            None,
        )
        .unwrap();
        let key = "tools/diag-scripts/ui-gallery/perf/ui-gallery-window-resize-stress-steady.json";
        assert_eq!(
            policy.seed_for(key, PerfSeedMetric::TopTotalTimeUs),
            PerfBaselineSeed::P95
        );
        assert_eq!(
            policy.seed_for(key, PerfSeedMetric::TopLayoutTimeUs),
            PerfBaselineSeed::P95
        );
        assert_eq!(
            policy.seed_for(key, PerfSeedMetric::TopLayoutEngineSolveTimeUs),
            PerfBaselineSeed::P95
        );
    }

    #[test]
    fn seed_policy_preset_can_tune_pointer_move_hit_test_thresholds() {
        let workspace_root = std::env::temp_dir().join("fret-diag-seed-policy-test-pointer-move");
        let script_path = workspace_root
            .join("tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json");
        let scripts = vec![script_path];

        let preset_path = workspace_root.join("preset.json");
        std::fs::create_dir_all(&workspace_root).unwrap();
        std::fs::write(
            &preset_path,
            r#"{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "rules": [
    {
      "scope": "tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json",
      "metric": "pointer_move_max_hit_test_time_us",
      "seed": "max",
      "quantum_us": 4
    }
  ]
}"#,
        )
        .unwrap();

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("ui-gallery-overlay-steady"),
            &scripts,
            std::slice::from_ref(&preset_path),
            &[],
            None,
        )
        .unwrap();

        let script_key =
            "tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json";
        assert_eq!(
            policy.tuning_for(script_key, PerfSeedMetric::PointerMoveHitTestTimeUs),
            PerfThresholdTuning {
                min_slack_us: 0,
                quantum_us: 4,
            }
        );
        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::PointerMoveHitTestTimeUs),
            PerfBaselineSeed::Max
        );
    }

    #[test]
    fn seed_policy_preset_and_cli_can_set_ui_threshold_mode() {
        let workspace_root = std::env::temp_dir().join("fret-diag-seed-policy-test-ui-mode");
        let script_path = workspace_root.join("tools/diag-scripts/ui-gallery/typical-probe.json");
        let scripts = vec![script_path];

        let preset_path = workspace_root.join("preset.json");
        std::fs::create_dir_all(&workspace_root).unwrap();
        std::fs::write(
            &preset_path,
            r#"{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "ui_threshold_mode": "frame_p95",
  "rules": []
}"#,
        )
        .unwrap();

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            None,
            &scripts,
            std::slice::from_ref(&preset_path),
            &[],
            None,
        )
        .unwrap();
        assert_eq!(
            policy.ui_threshold_mode(),
            PerfBaselineUiThresholdMode::FrameP95
        );
        assert_eq!(
            policy.threshold_seed_policy_json()["ui_threshold_mode"],
            "frame_p95"
        );

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            None,
            &scripts,
            &[preset_path],
            &[],
            Some(PerfBaselineUiThresholdMode::TopAndFrameP95),
        )
        .unwrap();
        assert_eq!(
            policy.ui_threshold_mode(),
            PerfBaselineUiThresholdMode::TopAndFrameP95
        );
    }

    #[test]
    fn seed_policy_preset_can_tune_renderer_thresholds() {
        let workspace_root = std::env::temp_dir().join("fret-diag-seed-policy-test-renderer");
        let script_path = workspace_root.join(
            "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json",
        );
        let scripts = vec![script_path];

        let preset_path = workspace_root.join("preset.json");
        std::fs::create_dir_all(&workspace_root).unwrap();
        std::fs::write(
            &preset_path,
            r#"{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "rules": [
    {
      "scope": "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json",
      "metric": "renderer_prepare_svg_us",
      "seed": "max",
      "min_slack_us": 32,
      "quantum_us": 8
    }
  ]
}"#,
        )
        .unwrap();

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            None,
            &scripts,
            std::slice::from_ref(&preset_path),
            &[],
            None,
        )
        .unwrap();

        let script_key = "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-steady.json";
        assert_eq!(
            policy.seed_for(script_key, PerfSeedMetric::RendererPrepareSvgUs),
            PerfBaselineSeed::Max
        );
        assert_eq!(
            policy.tuning_for(script_key, PerfSeedMetric::RendererPrepareSvgUs),
            PerfThresholdTuning {
                min_slack_us: 32,
                quantum_us: 8,
            }
        );
    }

    #[test]
    fn perf_suite_membership_name_covers_overlay_single_script_follow_ons() {
        let cases = [
            (
                "ui-gallery-context-menu-right-click-steady",
                "perf-ui-gallery-context-menu-right-click-steady",
            ),
            (
                "perf-ui-gallery-context-menu-right-click-steady",
                "perf-ui-gallery-context-menu-right-click-steady",
            ),
            (
                "ui-gallery-dialog-escape-focus-restore-steady",
                "perf-ui-gallery-dialog-escape-focus-restore-steady",
            ),
            (
                "perf-ui-gallery-dialog-escape-focus-restore-steady",
                "perf-ui-gallery-dialog-escape-focus-restore-steady",
            ),
            (
                "ui-gallery-dropdown-open-select-steady",
                "perf-ui-gallery-dropdown-open-select-steady",
            ),
            (
                "perf-ui-gallery-dropdown-open-select-steady",
                "perf-ui-gallery-dropdown-open-select-steady",
            ),
            (
                "ui-gallery-overlay-pointer-move-steady",
                "perf-ui-gallery-overlay-pointer-move-steady",
            ),
            (
                "perf-ui-gallery-overlay-pointer-move-steady",
                "perf-ui-gallery-overlay-pointer-move-steady",
            ),
            (
                "ui-gallery-overlay-torture-steady",
                "perf-ui-gallery-overlay-torture-steady",
            ),
            (
                "perf-ui-gallery-overlay-torture-steady",
                "perf-ui-gallery-overlay-torture-steady",
            ),
            (
                "ui-gallery-hit-test-torture-steady",
                "perf-ui-gallery-hit-test-torture-steady",
            ),
            (
                "perf-ui-gallery-hit-test-torture-steady",
                "perf-ui-gallery-hit-test-torture-steady",
            ),
            (
                "ui-gallery-code-editor-torture-autoscroll-typical",
                "perf-ui-gallery-code-editor-torture-autoscroll-typical",
            ),
            (
                "perf-ui-gallery-code-editor-torture-autoscroll-typical",
                "perf-ui-gallery-code-editor-torture-autoscroll-typical",
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(perf_suite_membership_name(input), Some(expected));
        }
    }

    #[test]
    fn perf_suite_membership_name_accepts_registry_backed_perf_suites() {
        assert_eq!(
            perf_suite_membership_name("perf-ui-gallery-new-focused-suite"),
            Some("perf-ui-gallery-new-focused-suite")
        );
        assert_eq!(
            perf_suite_membership_name("ui-gallery-new-focused-suite"),
            None
        );
    }
}
