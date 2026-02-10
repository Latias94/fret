use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::compare::normalize_repo_relative_path;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PerfSeedMetric {
    TopTotalTimeUs,
    TopLayoutTimeUs,
    TopLayoutEngineSolveTimeUs,
}

impl PerfSeedMetric {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PerfSeedMetric::TopTotalTimeUs => "top_total_time_us",
            PerfSeedMetric::TopLayoutTimeUs => "top_layout_time_us",
            PerfSeedMetric::TopLayoutEngineSolveTimeUs => "top_layout_engine_solve_time_us",
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
            _ => Err(format!(
                "invalid metric (expected top_total_time_us|top_layout_time_us|top_layout_engine_solve_time_us): {s:?}"
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
    // Final per-(script, metric) override map (only for scripts in the current invocation).
    overrides: HashMap<(String, PerfSeedMetric), (PerfBaselineSeed, RuleSourceKind)>,
    // Audit-friendly expanded rules (only for scripts in the current invocation).
    pub(crate) audit_rules: Vec<Value>,
}

impl ResolvedPerfBaselineSeedPolicy {
    pub(crate) fn seed_for(&self, script: &str, metric: PerfSeedMetric) -> PerfBaselineSeed {
        self.overrides
            .get(&(script.to_string(), metric))
            .map(|(seed, _src)| *seed)
            .unwrap_or(self.default_seed)
    }

    pub(crate) fn threshold_seed_policy_json(&self) -> Value {
        Value::Object(
            [
                ("schema_version".to_string(), Value::from(1u64)),
                (
                    "default_seed".to_string(),
                    Value::String(self.default_seed.as_str().to_string()),
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
    source: RuleSourceKind,
}

#[derive(Debug, Clone)]
struct SeedPresetFile {
    default_seed: Option<PerfBaselineSeed>,
    rules: Vec<(String, PerfSeedMetric, PerfBaselineSeed)>,
}

pub(crate) fn scripts_for_perf_suite_name(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "ui-gallery" => Some(&[
            "tools/diag-scripts/ui-gallery-overlay-torture.json",
            "tools/diag-scripts/ui-gallery-dropdown-open-select.json",
            "tools/diag-scripts/ui-gallery-context-menu-right-click.json",
            "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
            "tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json",
            "tools/diag-scripts/ui-gallery-virtual-list-torture.json",
            "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json",
            "tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json",
            "tools/diag-scripts/ui-gallery-window-resize-stress.json",
        ]),
        "ui-gallery-steady" => Some(&[
            "tools/diag-scripts/ui-gallery-overlay-torture-steady.json",
            "tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json",
            "tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json",
            "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json",
            "tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json",
            "tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json",
            "tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json",
            "tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json",
            "tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json",
            "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json",
        ]),
        "extras-marquee-steady" => Some(&["tools/diag-scripts/extras-marquee-steady.json"]),
        _ => None,
    }
}

pub(crate) fn resolve_perf_baseline_seed_policy(
    workspace_root: &Path,
    suite_name: Option<&str>,
    scripts: &[PathBuf],
    preset_paths: &[PathBuf],
    cli_seed_specs: &[String],
) -> Result<ResolvedPerfBaselineSeedPolicy, String> {
    let mut default_seed = PerfBaselineSeed::Max;

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
                source: RuleSourceKind::Default,
            });
            specs.push(SeedRuleSpec {
                scope: script_key.clone(),
                metric: PerfSeedMetric::TopLayoutTimeUs,
                seed: PerfBaselineSeed::P95,
                source: RuleSourceKind::Default,
            });
            specs.push(SeedRuleSpec {
                scope: script_key.clone(),
                metric: PerfSeedMetric::TopLayoutEngineSolveTimeUs,
                seed: PerfBaselineSeed::P95,
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
        for (scope, metric, seed) in preset.rules {
            let source = if scope_is_suite_like(&scope, suite_name) {
                RuleSourceKind::PresetSuite
            } else {
                RuleSourceKind::Preset
            };
            specs.push(SeedRuleSpec {
                scope,
                metric,
                seed,
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
            source: if suite_like {
                RuleSourceKind::CliSuite
            } else {
                RuleSourceKind::Cli
            },
        });
    }

    // Apply layered overrides (last match wins).
    let mut overrides: HashMap<(String, PerfSeedMetric), (PerfBaselineSeed, RuleSourceKind)> =
        HashMap::new();
    let mut audit: HashMap<(String, PerfSeedMetric), (PerfBaselineSeed, RuleSourceKind)> =
        HashMap::new();

    for spec in specs {
        let script_keys =
            expand_scope_to_script_keys(workspace_root, suite_name, &scripts_by_key, &spec.scope)?;
        for key in script_keys {
            overrides.insert((key.clone(), spec.metric), (spec.seed, spec.source));
            audit.insert((key, spec.metric), (spec.seed, spec.source));
        }
    }

    let mut audit_rules: Vec<Value> = Vec::new();
    for ((script, metric), (seed, source)) in audit.into_iter() {
        if seed == default_seed {
            continue;
        }
        audit_rules.push(serde_json::json!({
            "script": script,
            "metric": metric.as_str(),
            "seed": seed.as_str(),
            "source": source.as_str(),
        }));
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
        overrides,
        audit_rules,
    })
}

fn scope_is_suite_like(scope: &str, suite_name: Option<&str>) -> bool {
    scope == "*"
        || scope == "this-suite"
        || scope.starts_with("suite:")
        || suite_name.is_some_and(|s| s == scope)
        || scripts_for_perf_suite_name(scope).is_some()
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

    let rules = root
        .get("rules")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            format!(
                "invalid perf baseline seed preset: missing rules array: {}",
                resolved.display()
            )
        })?;

    let mut out: Vec<(String, PerfSeedMetric, PerfBaselineSeed)> = Vec::new();
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
        out.push((
            scope.to_string(),
            metric.parse::<PerfSeedMetric>()?,
            seed.parse::<PerfBaselineSeed>()?,
        ));
    }

    Ok(SeedPresetFile {
        default_seed,
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
        || scripts_for_perf_suite_name(scope).is_some();
    Ok((scope.to_string(), metric, seed, suite_like))
}

fn expand_scope_to_script_keys(
    workspace_root: &Path,
    suite_name: Option<&str>,
    scripts_by_key: &BTreeMap<String, PathBuf>,
    scope: &str,
) -> Result<Vec<String>, String> {
    let all_keys: Vec<String> = scripts_by_key.keys().cloned().collect();

    if scope == "*" || scope == "this-suite" || suite_name.is_some_and(|s| s == scope) {
        return Ok(all_keys);
    }

    if let Some(name) = scope.strip_prefix("suite:") {
        let Some(paths) = scripts_for_perf_suite_name(name) else {
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

    if let Some(paths) = scripts_for_perf_suite_name(scope) {
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
        let script_path = workspace_root.join("tools/diag-scripts/extras-marquee-steady.json");
        let scripts = vec![script_path];

        let preset_path = workspace_root.join("preset.json");
        std::fs::create_dir_all(workspace_root.join("tools/diag-scripts")).unwrap();
        std::fs::write(
            &preset_path,
            r#"{
  "schema_version": 1,
  "kind": "perf_baseline_seed_policy",
  "default_seed": "max",
  "rules": [
    { "scope": "extras-marquee-steady", "metric": "top_total_time_us", "seed": "p90" },
    { "scope": "extras-marquee-steady", "metric": "top_layout_time_us", "seed": "p90" },
    { "scope": "tools/diag-scripts/extras-marquee-steady.json", "metric": "top_layout_engine_solve_time_us", "seed": "p95" }
  ]
}"#,
        )
        .unwrap();

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("extras-marquee-steady"),
            &scripts,
            &[preset_path.clone()],
            &[],
        )
        .unwrap();

        let script_key = "tools/diag-scripts/extras-marquee-steady.json";
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

        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("extras-marquee-steady"),
            &scripts,
            &[preset_path],
            &[String::from("this-suite@top_total_time_us=p95")],
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
        let scripts = vec![
            workspace_root.join("tools/diag-scripts/ui-gallery-window-resize-stress-steady.json"),
        ];
        let policy = resolve_perf_baseline_seed_policy(
            &workspace_root,
            Some("ui-gallery-steady"),
            &scripts,
            &[],
            &[],
        )
        .unwrap();
        let key = "tools/diag-scripts/ui-gallery-window-resize-stress-steady.json";
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
}
