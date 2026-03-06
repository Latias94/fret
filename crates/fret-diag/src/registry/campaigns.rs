use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::regression_summary::RegressionLaneV1;

pub(crate) const DIAG_CAMPAIGN_MANIFEST_KIND_V1: &str = "diag_campaign_manifest";
pub(crate) const DIAG_CAMPAIGNS_DIR: &str = "diag-campaigns";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampaignDefinition {
    pub id: String,
    pub description: String,
    pub lane: RegressionLaneV1,
    pub profile: Option<String>,
    pub suites: Vec<String>,
    pub scripts: Vec<String>,
    pub owner: Option<String>,
    pub platforms: Vec<String>,
    pub tier: Option<String>,
    pub expected_duration_ms: Option<u64>,
    pub tags: Vec<String>,
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
            suites: UI_GALLERY_SMOKE_SUITES
                .iter()
                .map(|suite| (*suite).to_string())
                .collect(),
            scripts: Vec::new(),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: Some(120_000),
            tags: vec![
                "ui-gallery".to_string(),
                "smoke".to_string(),
                "developer-loop".to_string(),
            ],
            source: CampaignDefinitionSource::Builtin,
        },
        CampaignDefinition {
            id: "ui-gallery-correctness".to_string(),
            description: "Broader UI gallery correctness pass for common interaction surfaces."
                .to_string(),
            lane: RegressionLaneV1::Correctness,
            profile: Some("bounded".to_string()),
            suites: UI_GALLERY_CORRECTNESS_SUITES
                .iter()
                .map(|suite| (*suite).to_string())
                .collect(),
            scripts: Vec::new(),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("correctness".to_string()),
            expected_duration_ms: Some(300_000),
            tags: vec!["ui-gallery".to_string(), "correctness".to_string()],
            source: CampaignDefinitionSource::Builtin,
        },
        CampaignDefinition {
            id: "docking-smoke".to_string(),
            description: "Docking-focused smoke run covering arbitration and hardening basics."
                .to_string(),
            lane: RegressionLaneV1::Smoke,
            profile: Some("bounded".to_string()),
            suites: DOCKING_SMOKE_SUITES
                .iter()
                .map(|suite| (*suite).to_string())
                .collect(),
            scripts: Vec::new(),
            owner: Some("diag".to_string()),
            platforms: vec!["native".to_string()],
            tier: Some("smoke".to_string()),
            expected_duration_ms: Some(120_000),
            tags: vec!["docking".to_string(), "smoke".to_string()],
            source: CampaignDefinitionSource::Builtin,
        },
    ]
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

    pub(crate) fn list_campaigns(&self) -> &[CampaignDefinition] {
        &self.campaigns
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
    serde_json::json!({
        "id": campaign.id,
        "description": campaign.description,
        "lane": campaign.lane,
        "profile": campaign.profile,
        "suites": campaign.suites,
        "scripts": campaign.scripts,
        "owner": campaign.owner,
        "platforms": campaign.platforms,
        "tier": campaign.tier,
        "expected_duration_ms": campaign.expected_duration_ms,
        "tags": campaign.tags,
        "source_kind": source_kind_str(&campaign.source),
        "source_path": match &campaign.source {
            CampaignDefinitionSource::Builtin => None,
            CampaignDefinitionSource::Manifest(path) => Some(path.display().to_string()),
        },
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

    let mut suites = manifest
        .suites
        .into_iter()
        .map(|suite| suite.trim().to_string())
        .filter(|suite| !suite.is_empty())
        .collect::<Vec<_>>();
    suites.sort();
    suites.dedup();

    let mut scripts = manifest
        .scripts
        .into_iter()
        .map(|script| script.trim().to_string())
        .filter(|script| !script.is_empty())
        .collect::<Vec<_>>();
    scripts.sort();
    scripts.dedup();

    if suites.is_empty() && scripts.is_empty() {
        return Err(format!(
            "campaign manifest must contain at least one suite or script: {}",
            path.display()
        ));
    }

    let owner = manifest
        .owner
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let mut platforms = manifest
        .platforms
        .into_iter()
        .map(|platform| platform.trim().to_string())
        .filter(|platform| !platform.is_empty())
        .collect::<Vec<_>>();
    platforms.sort();
    platforms.dedup();
    let tier = manifest
        .tier
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let profile = manifest
        .profile
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let mut tags = manifest
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    tags.sort();
    tags.dedup();

    Ok(CampaignDefinition {
        id: id.to_string(),
        description: description.to_string(),
        lane: manifest.lane,
        profile,
        suites,
        scripts,
        owner,
        platforms,
        tier,
        expected_duration_ms: manifest.expected_duration_ms,
        tags,
        source: CampaignDefinitionSource::Manifest(path.to_path_buf()),
    })
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
        assert_eq!(campaign.suites, UI_GALLERY_SMOKE_SUITES);
        assert!(matches!(campaign.source, CampaignDefinitionSource::Builtin));
        assert_eq!(campaign.tier.as_deref(), Some("smoke"));
    }

    #[test]
    fn builtin_registry_rejects_unknown_id() {
        let registry = CampaignRegistry::builtin();
        let error = registry.resolve("missing-campaign").unwrap_err();
        assert!(error.contains("unknown diag campaign"));
        assert!(error.contains("ui-gallery-smoke"));
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
  "scripts": ["tools/diag-scripts/ui-gallery-layout.json"],
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
        assert_eq!(
            campaign.scripts,
            vec!["tools/diag-scripts/ui-gallery-layout.json".to_string()]
        );
        assert_eq!(campaign.expected_duration_ms, Some(12345));
        assert!(matches!(
            campaign.source,
            CampaignDefinitionSource::Manifest(_)
        ));

        let _ = fs::remove_dir_all(&root);
    }
}
