use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::regression_summary::RegressionLaneV1;

pub(crate) const DIAG_CAMPAIGN_MANIFEST_KIND_V1: &str = "diag_campaign_manifest";
pub(crate) const DIAG_CAMPAIGNS_DIR: &str = "diag-campaigns";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CampaignItemKind {
    Suite,
    Script,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampaignItemDefinition {
    pub kind: CampaignItemKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampaignDefinition {
    pub id: String,
    pub description: String,
    pub lane: RegressionLaneV1,
    pub profile: Option<String>,
    pub items: Vec<CampaignItemDefinition>,
    pub owner: Option<String>,
    pub platforms: Vec<String>,
    pub tier: Option<String>,
    pub expected_duration_ms: Option<u64>,
    pub tags: Vec<String>,
    pub requires_capabilities: Vec<String>,
    pub flake_policy: Option<String>,
    pub source: CampaignDefinitionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CampaignDefinitionSource {
    Builtin,
    Manifest(PathBuf),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CampaignRegistry {
    campaigns: Vec<CampaignDefinition>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CampaignFilterOptions {
    pub lane: Option<RegressionLaneV1>,
    pub tier: Option<String>,
    pub tags: Vec<String>,
    pub platforms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CampaignManifestV1 {
    schema_version: u64,
    kind: String,
    id: String,
    description: String,
    lane: RegressionLaneV1,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    items: Vec<CampaignManifestItemV1>,
    #[serde(default)]
    suites: Vec<String>,
    #[serde(default)]
    scripts: Vec<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    expected_duration_ms: Option<u64>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    requires_capabilities: Vec<String>,
    #[serde(default)]
    flake_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CampaignManifestItemV1 {
    kind: CampaignItemKind,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    suite: Option<String>,
    #[serde(default)]
    script: Option<String>,
}

const UI_GALLERY_SMOKE_SUITES: &[&str] = &["ui-gallery-lite-smoke", "ui-gallery-layout"];
const UI_GALLERY_CORRECTNESS_SUITES: &[&str] = &["ui-gallery", "ui-gallery-code-editor"];
const DOCKING_SMOKE_SUITES: &[&str] = &["diag-hardening-smoke-docking", "docking-arbitration"];

fn builtin_campaign_definitions() -> Vec<CampaignDefinition> {
    vec![
        CampaignDefinition {
            id: "ui-gallery-smoke".to_string(),
            description: "Fast UI gallery smoke coverage with layout sanity.".to_string(),
            lane: RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            items: suite_items(UI_GALLERY_SMOKE_SUITES),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: Some(120_000),
            tags: vec![
                "ui-gallery".to_string(),
                "smoke".to_string(),
                "developer-loop".to_string(),
            ],
            requires_capabilities: Vec::new(),
            flake_policy: Some("fail_fast".to_string()),
            source: CampaignDefinitionSource::Builtin,
        },
        CampaignDefinition {
            id: "ui-gallery-correctness".to_string(),
            description: "Broader UI gallery correctness pass for common interaction surfaces."
                .to_string(),
            lane: RegressionLaneV1::Correctness,
            profile: Some("bounded".to_string()),
            items: suite_items(UI_GALLERY_CORRECTNESS_SUITES),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("correctness".to_string()),
            expected_duration_ms: Some(300_000),
            tags: vec!["ui-gallery".to_string(), "correctness".to_string()],
            requires_capabilities: Vec::new(),
            flake_policy: Some("retry_once".to_string()),
            source: CampaignDefinitionSource::Builtin,
        },
        CampaignDefinition {
            id: "docking-smoke".to_string(),
            description: "Docking-focused smoke run covering arbitration and hardening basics."
                .to_string(),
            lane: RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            items: suite_items(DOCKING_SMOKE_SUITES),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: Some(120_000),
            tags: vec!["docking".to_string(), "smoke".to_string()],
            requires_capabilities: Vec::new(),
            flake_policy: Some("fail_fast".to_string()),
            source: CampaignDefinitionSource::Builtin,
        },
    ]
}

impl CampaignDefinition {
    pub(crate) fn suites(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| item.kind == CampaignItemKind::Suite)
            .map(|item| item.value.as_str())
            .collect()
    }

    pub(crate) fn scripts(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|item| item.kind == CampaignItemKind::Script)
            .map(|item| item.value.as_str())
            .collect()
    }

    pub(crate) fn suite_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.kind == CampaignItemKind::Suite)
            .count()
    }

    pub(crate) fn script_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.kind == CampaignItemKind::Script)
            .count()
    }

    pub(crate) fn matches_filter(&self, filter: &CampaignFilterOptions) -> bool {
        if let Some(lane) = filter.lane
            && self.lane != lane
        {
            return false;
        }
        if let Some(tier) = filter.tier.as_deref()
            && !self
                .tier
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case(tier))
        {
            return false;
        }
        if !filter.tags.is_empty() {
            let tags = self
                .tags
                .iter()
                .map(|tag| tag.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if !filter
                .tags
                .iter()
                .all(|tag| tags.iter().any(|value| value == &tag.to_ascii_lowercase()))
            {
                return false;
            }
        }
        if !filter.platforms.is_empty() {
            let platforms = self
                .platforms
                .iter()
                .map(|platform| platform.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if !filter.platforms.iter().all(|platform| {
                platforms
                    .iter()
                    .any(|value| value == &platform.to_ascii_lowercase())
            }) {
                return false;
            }
        }
        true
    }
}

impl CampaignRegistry {
    pub(crate) fn builtin() -> Self {
        Self {
            campaigns: builtin_campaign_definitions(),
        }
    }

    pub(crate) fn load_from_workspace_root(workspace_root: &Path) -> Result<Self, String> {
        let mut merged: BTreeMap<String, CampaignDefinition> = Self::builtin()
            .campaigns
            .into_iter()
            .map(|campaign| (campaign.id.clone(), campaign))
            .collect();

        let manifests_dir = campaigns_dir_from_workspace_root(workspace_root);
        if manifests_dir.is_dir() {
            let manifests = load_manifest_campaigns_from_dir(&manifests_dir)?;
            for campaign in manifests {
                merged.insert(campaign.id.clone(), campaign);
            }
        }

        Ok(Self {
            campaigns: merged.into_values().collect(),
        })
    }

    pub(crate) fn filtered_campaigns<'a>(
        &'a self,
        filter: &CampaignFilterOptions,
    ) -> Vec<&'a CampaignDefinition> {
        self.campaigns
            .iter()
            .filter(|campaign| campaign.matches_filter(filter))
            .collect()
    }

    pub(crate) fn resolve(&self, campaign_id: &str) -> Result<&CampaignDefinition, String> {
        self.campaigns
            .iter()
            .find(|campaign| campaign.id == campaign_id)
            .ok_or_else(|| {
                format!(
                    "unknown diag campaign: {}\nknown campaigns: {}",
                    campaign_id,
                    self.campaigns
                        .iter()
                        .map(|campaign| campaign.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

pub(crate) fn campaigns_dir_from_workspace_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join("tools").join(DIAG_CAMPAIGNS_DIR)
}

pub(crate) fn campaign_to_json(campaign: &CampaignDefinition) -> serde_json::Value {
    let suites = campaign.suites();
    let scripts = campaign.scripts();
    serde_json::json!({
        "id": campaign.id,
        "description": campaign.description,
        "lane": campaign.lane,
        "profile": campaign.profile,
        "items": campaign.items.iter().map(item_to_json).collect::<Vec<_>>(),
        "suites": suites,
        "scripts": scripts,
        "owner": campaign.owner,
        "platforms": campaign.platforms,
        "tier": campaign.tier,
        "expected_duration_ms": campaign.expected_duration_ms,
        "tags": campaign.tags,
        "requires_capabilities": campaign.requires_capabilities,
        "flake_policy": campaign.flake_policy,
        "source_kind": source_kind_str(&campaign.source),
        "source_path": match &campaign.source {
            CampaignDefinitionSource::Builtin => None,
            CampaignDefinitionSource::Manifest(path) => Some(path.display().to_string()),
        },
    })
}

pub(crate) fn item_to_json(item: &CampaignItemDefinition) -> serde_json::Value {
    serde_json::json!({
        "kind": item_kind_str(item.kind),
        "value": item.value,
    })
}

pub(crate) fn parse_lane(raw: &str) -> Result<RegressionLaneV1, String> {
    match raw {
        "smoke" => Ok(RegressionLaneV1::Smoke),
        "correctness" => Ok(RegressionLaneV1::Correctness),
        "matrix" => Ok(RegressionLaneV1::Matrix),
        "perf" => Ok(RegressionLaneV1::Perf),
        "nightly" => Ok(RegressionLaneV1::Nightly),
        "full" => Ok(RegressionLaneV1::Full),
        other => Err(format!("unknown regression lane: {other}")),
    }
}

pub(crate) fn lane_to_str(lane: RegressionLaneV1) -> &'static str {
    match lane {
        RegressionLaneV1::Smoke => "smoke",
        RegressionLaneV1::Correctness => "correctness",
        RegressionLaneV1::Matrix => "matrix",
        RegressionLaneV1::Perf => "perf",
        RegressionLaneV1::Nightly => "nightly",
        RegressionLaneV1::Full => "full",
    }
}

pub(crate) fn source_kind_str(source: &CampaignDefinitionSource) -> &'static str {
    match source {
        CampaignDefinitionSource::Builtin => "builtin",
        CampaignDefinitionSource::Manifest(_) => "manifest",
    }
}

pub(crate) fn item_kind_str(kind: CampaignItemKind) -> &'static str {
    match kind {
        CampaignItemKind::Suite => "suite",
        CampaignItemKind::Script => "script",
    }
}

fn suite_items(values: &[&str]) -> Vec<CampaignItemDefinition> {
    values
        .iter()
        .map(|value| CampaignItemDefinition {
            kind: CampaignItemKind::Suite,
            value: (*value).to_string(),
        })
        .collect()
}

fn normalize_optional_string(raw: Option<String>) -> Option<String> {
    raw.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_lowercase_string_list(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn load_manifest_campaigns_from_dir(dir: &Path) -> Result<Vec<CampaignDefinition>, String> {
    let mut manifest_paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| {
            format!(
                "failed to read campaign manifests dir {}: {}",
                dir.display(),
                e
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect();
    manifest_paths.sort();

    let mut seen_paths_by_id = BTreeMap::<String, PathBuf>::new();
    let mut campaigns = Vec::new();
    for manifest_path in manifest_paths {
        let campaign = load_manifest_campaign(&manifest_path)?;
        if let Some(previous) = seen_paths_by_id.insert(campaign.id.clone(), manifest_path.clone())
        {
            return Err(format!(
                "duplicate campaign id `{}` in manifests: {} and {}",
                campaign.id,
                previous.display(),
                manifest_path.display()
            ));
        }
        campaigns.push(campaign);
    }

    Ok(campaigns)
}

fn load_manifest_campaign(path: &Path) -> Result<CampaignDefinition, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("failed to read campaign manifest {}: {}", path.display(), e))?;
    let manifest: CampaignManifestV1 = serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid campaign manifest {}: {}", path.display(), e))?;

    if manifest.schema_version != 1 {
        return Err(format!(
            "invalid campaign manifest schema_version (expected 1): {} ({})",
            manifest.schema_version,
            path.display()
        ));
    }
    if manifest.kind != DIAG_CAMPAIGN_MANIFEST_KIND_V1 {
        return Err(format!(
            "invalid campaign manifest kind (expected {}): {} ({})",
            DIAG_CAMPAIGN_MANIFEST_KIND_V1,
            manifest.kind,
            path.display()
        ));
    }

    let id = manifest.id.trim();
    if id.is_empty() {
        return Err(format!(
            "campaign manifest id must not be empty: {}",
            path.display()
        ));
    }
    let description = manifest.description.trim();
    if description.is_empty() {
        return Err(format!(
            "campaign manifest description must not be empty: {}",
            path.display()
        ));
    }

    let items = parse_manifest_items(path, manifest.items, manifest.suites, manifest.scripts)?;
    if items.is_empty() {
        return Err(format!(
            "campaign manifest must contain at least one item: {}",
            path.display()
        ));
    }

    let owner = normalize_optional_string(manifest.owner);
    let mut platforms = manifest
        .platforms
        .into_iter()
        .map(|platform| platform.trim().to_string())
        .filter(|platform| !platform.is_empty())
        .collect::<Vec<_>>();
    platforms.sort();
    platforms.dedup();
    let tier = normalize_optional_string(manifest.tier);
    let profile = normalize_optional_string(manifest.profile);
    let mut tags = manifest
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    tags.sort();
    tags.dedup();
    let requires_capabilities = normalize_lowercase_string_list(manifest.requires_capabilities);
    let flake_policy =
        normalize_optional_string(manifest.flake_policy).map(|value| value.to_ascii_lowercase());

    Ok(CampaignDefinition {
        id: id.to_string(),
        description: description.to_string(),
        lane: manifest.lane,
        profile,
        items,
        owner,
        platforms,
        tier,
        expected_duration_ms: manifest.expected_duration_ms,
        tags,
        requires_capabilities,
        flake_policy,
        source: CampaignDefinitionSource::Manifest(path.to_path_buf()),
    })
}

fn parse_manifest_items(
    path: &Path,
    manifest_items: Vec<CampaignManifestItemV1>,
    suites: Vec<String>,
    scripts: Vec<String>,
) -> Result<Vec<CampaignItemDefinition>, String> {
    if !manifest_items.is_empty() {
        let mut items = Vec::with_capacity(manifest_items.len());
        for (index, item) in manifest_items.into_iter().enumerate() {
            let value = match item.kind {
                CampaignItemKind::Suite => item.value.or(item.suite),
                CampaignItemKind::Script => item.value.or(item.script),
            }
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "campaign manifest item {} is missing a value: {}",
                    index,
                    path.display()
                )
            })?;
            items.push(CampaignItemDefinition {
                kind: item.kind,
                value,
            });
        }
        return Ok(items);
    }

    let mut items = Vec::new();
    for suite in suites {
        let suite = suite.trim();
        if !suite.is_empty() {
            items.push(CampaignItemDefinition {
                kind: CampaignItemKind::Suite,
                value: suite.to_string(),
            });
        }
    }
    for script in scripts {
        let script = script.trim();
        if !script.is_empty() {
            items.push(CampaignItemDefinition {
                kind: CampaignItemKind::Script,
                value: script.to_string(),
            });
        }
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("fret-diag-campaign-registry-{now}"))
    }

    #[test]
    fn parse_lane_accepts_known_values() {
        assert_eq!(parse_lane("smoke").unwrap(), RegressionLaneV1::Smoke);
        assert_eq!(
            parse_lane("correctness").unwrap(),
            RegressionLaneV1::Correctness
        );
        assert_eq!(parse_lane("perf").unwrap(), RegressionLaneV1::Perf);
    }

    #[test]
    fn builtin_registry_finds_known_id() {
        let registry = CampaignRegistry::builtin();
        let campaign = registry.resolve("ui-gallery-smoke").unwrap();
        assert_eq!(campaign.id, "ui-gallery-smoke");
        assert_eq!(campaign.suite_count(), 2);
        assert!(matches!(campaign.source, CampaignDefinitionSource::Builtin));
        assert_eq!(campaign.tier.as_deref(), Some("smoke"));
        assert_eq!(campaign.flake_policy.as_deref(), Some("fail_fast"));
        assert!(campaign.requires_capabilities.is_empty());
    }

    #[test]
    fn builtin_registry_rejects_unknown_id() {
        let registry = CampaignRegistry::builtin();
        let error = registry.resolve("missing-campaign").unwrap_err();
        assert!(error.contains("unknown diag campaign"));
        assert!(error.contains("ui-gallery-smoke"));
    }

    #[test]
    fn campaign_filter_matches_lane_tag_and_platform() {
        let registry = CampaignRegistry::builtin();
        let filter = CampaignFilterOptions {
            lane: Some(RegressionLaneV1::Smoke),
            tier: Some("smoke".to_string()),
            tags: vec!["ui-gallery".to_string()],
            platforms: vec!["native".to_string()],
        };
        let campaigns = registry.filtered_campaigns(&filter);
        assert_eq!(campaigns.len(), 1);
        assert_eq!(campaigns[0].id, "ui-gallery-smoke");
    }

    #[test]
    fn workspace_registry_loads_manifest_and_overrides_builtin() {
        let root = unique_temp_dir();
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        let manifest_path = manifests_dir.join("ui-gallery-smoke.json");
        fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "ui-gallery-smoke",
  "description": "Manifest-backed smoke override.",
  "lane": "smoke",
  "profile": "bounded",
  "items": [
    { "kind": "script", "value": "tools/diag-scripts/ui-gallery-layout.json" },
    { "kind": "suite", "value": "ui-gallery-layout" }
  ],
  "owner": "diag",
  "platforms": ["native"],
  "tier": "smoke",
  "expected_duration_ms": 12345,
  "tags": ["manifest", "smoke"]
}"#,
        )
        .expect("write manifest");

        let registry = CampaignRegistry::load_from_workspace_root(&root).unwrap();
        let campaign = registry.resolve("ui-gallery-smoke").unwrap();
        assert_eq!(campaign.description, "Manifest-backed smoke override.");
        assert_eq!(campaign.items.len(), 2);
        assert_eq!(campaign.items[0].kind, CampaignItemKind::Script);
        assert_eq!(campaign.items[1].kind, CampaignItemKind::Suite);
        assert_eq!(campaign.expected_duration_ms, Some(12345));
        assert!(campaign.requires_capabilities.is_empty());
        assert!(campaign.flake_policy.is_none());
        assert!(matches!(
            campaign.source,
            CampaignDefinitionSource::Manifest(_)
        ));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn manifest_campaign_normalizes_capabilities_and_flake_policy() {
        let root = unique_temp_dir();
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        let manifest_path = manifests_dir.join("ui-gallery-correctness.json");
        fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "ui-gallery-correctness",
  "description": "Manifest-backed correctness campaign.",
  "lane": "correctness",
  "items": [
    { "kind": "suite", "value": "ui-gallery" }
  ],
  "requires_capabilities": [" diag.script_v2 ", "GPU_PICK", "gpu_pick"],
  "flake_policy": " Retry_Once "
}"#,
        )
        .expect("write manifest");

        let registry = CampaignRegistry::load_from_workspace_root(&root).unwrap();
        let campaign = registry.resolve("ui-gallery-correctness").unwrap();
        assert_eq!(
            campaign.requires_capabilities,
            vec!["diag.script_v2".to_string(), "gpu_pick".to_string()]
        );
        assert_eq!(campaign.flake_policy.as_deref(), Some("retry_once"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn workspace_registry_loads_legacy_top_level_suites_and_scripts_manifest() {
        let root = unique_temp_dir();
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        let manifest_path = manifests_dir.join("legacy-shape.json");
        fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "legacy-shape",
  "description": "Legacy top-level suite/script manifest shape.",
  "lane": "smoke",
  "suites": ["ui-gallery-lite-smoke", " ui-gallery-layout "],
  "scripts": ["tools/diag-scripts/ui-gallery-layout.json", "  "]
}"#,
        )
        .expect("write manifest");

        let registry = CampaignRegistry::load_from_workspace_root(&root).unwrap();
        let campaign = registry.resolve("legacy-shape").unwrap();
        assert_eq!(campaign.items.len(), 3);
        assert_eq!(campaign.items[0].kind, CampaignItemKind::Suite);
        assert_eq!(campaign.items[0].value, "ui-gallery-lite-smoke");
        assert_eq!(campaign.items[1].kind, CampaignItemKind::Suite);
        assert_eq!(campaign.items[1].value, "ui-gallery-layout");
        assert_eq!(campaign.items[2].kind, CampaignItemKind::Script);
        assert_eq!(
            campaign.items[2].value,
            "tools/diag-scripts/ui-gallery-layout.json"
        );
        assert_eq!(campaign.suite_count(), 2);
        assert_eq!(campaign.script_count(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn campaign_to_json_includes_campaign_metadata_contract_fields() {
        let campaign = CampaignDefinition {
            id: "campaign-json".to_string(),
            description: "sample".to_string(),
            lane: RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            items: vec![CampaignItemDefinition {
                kind: CampaignItemKind::Suite,
                value: "ui-gallery-lite-smoke".to_string(),
            }],
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: Some(10),
            tags: vec!["ui-gallery".to_string()],
            requires_capabilities: vec!["diag.script_v2".to_string()],
            flake_policy: Some("fail_fast".to_string()),
            source: CampaignDefinitionSource::Builtin,
        };

        let json = campaign_to_json(&campaign);
        assert_eq!(
            json.get("requires_capabilities")
                .and_then(|value| value.as_array())
                .cloned(),
            Some(vec![serde_json::Value::String(
                "diag.script_v2".to_string()
            )])
        );
        assert_eq!(
            json.get("flake_policy").and_then(|value| value.as_str()),
            Some("fail_fast")
        );
    }
}
