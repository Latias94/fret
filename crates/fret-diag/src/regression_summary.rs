use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const DIAG_REGRESSION_SUMMARY_KIND_V1: &str = "diag_regression_summary";
pub const DIAG_REGRESSION_SUMMARY_FILENAME_V1: &str = "regression.summary.json";
pub const DIAG_REGRESSION_INDEX_FILENAME_V1: &str = "regression.index.json";
pub const DIAG_MATRIX_SUMMARY_FILENAME_V1: &str = "matrix.summary.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionSummaryV1 {
    pub schema_version: u32,
    pub kind: String,
    pub campaign: RegressionCampaignSummaryV1,
    pub run: RegressionRunSummaryV1,
    pub totals: RegressionTotalsV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<RegressionItemSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlights: Option<RegressionHighlightsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<RegressionArtifactsV1>,
}

impl RegressionSummaryV1 {
    pub fn new(
        campaign: RegressionCampaignSummaryV1,
        run: RegressionRunSummaryV1,
        totals: RegressionTotalsV1,
    ) -> Self {
        Self {
            schema_version: 1,
            kind: DIAG_REGRESSION_SUMMARY_KIND_V1.to_string(),
            campaign,
            run,
            totals,
            items: Vec::new(),
            highlights: None,
            artifacts: None,
        }
    }
}

impl RegressionTotalsV1 {
    pub fn record_status(&mut self, status: RegressionStatusV1) {
        self.items_total = self.items_total.saturating_add(1);
        match status {
            RegressionStatusV1::Passed => self.passed = self.passed.saturating_add(1),
            RegressionStatusV1::FailedDeterministic => {
                self.failed_deterministic = self.failed_deterministic.saturating_add(1)
            }
            RegressionStatusV1::FailedFlaky => {
                self.failed_flaky = self.failed_flaky.saturating_add(1)
            }
            RegressionStatusV1::FailedTooling => {
                self.failed_tooling = self.failed_tooling.saturating_add(1)
            }
            RegressionStatusV1::FailedTimeout => {
                self.failed_timeout = self.failed_timeout.saturating_add(1)
            }
            RegressionStatusV1::SkippedPolicy => {
                self.skipped_policy = self.skipped_policy.saturating_add(1)
            }
            RegressionStatusV1::Quarantined => {
                self.quarantined = self.quarantined.saturating_add(1)
            }
        }
    }
}

impl RegressionHighlightsV1 {
    pub fn from_items(items: &[RegressionItemSummaryV1]) -> Option<Self> {
        let first_failure = items
            .iter()
            .find(|item| item.status != RegressionStatusV1::Passed)
            .map(|item| RegressionHighlightRefV1 {
                item_id: item.item_id.clone(),
                reason_code: item.reason_code.clone(),
            });

        let mut reason_code_counts = std::collections::BTreeMap::<String, u32>::new();
        for item in items {
            if let Some(reason_code) = item.reason_code.as_deref()
                && !reason_code.trim().is_empty()
            {
                *reason_code_counts
                    .entry(reason_code.to_string())
                    .or_default() += 1;
            }
        }

        let mut top_reason_codes: Vec<RegressionReasonCodeCountV1> = reason_code_counts
            .into_iter()
            .map(|(reason_code, count)| RegressionReasonCodeCountV1 { reason_code, count })
            .collect();
        top_reason_codes.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.reason_code.cmp(&right.reason_code))
        });

        if first_failure.is_none() && top_reason_codes.is_empty() {
            return None;
        }

        Some(Self {
            first_failure,
            worst_perf_failure: None,
            flake_examples: Vec::new(),
            quarantine_examples: Vec::new(),
            top_reason_codes,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionCampaignSummaryV1 {
    pub name: String,
    pub lane: RegressionLaneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionRunSummaryV1 {
    pub run_id: String,
    pub created_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<String>,
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RegressionTotalsV1 {
    #[serde(default)]
    pub items_total: u32,
    #[serde(default)]
    pub passed: u32,
    #[serde(default)]
    pub failed_deterministic: u32,
    #[serde(default)]
    pub failed_flaky: u32,
    #[serde(default)]
    pub failed_tooling: u32,
    #[serde(default)]
    pub failed_timeout: u32,
    #[serde(default)]
    pub skipped_policy: u32,
    #[serde(default)]
    pub quarantined: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionHighlightsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worst_perf_failure: Option<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flake_examples: Vec<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quarantine_examples: Vec<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_reason_codes: Vec<RegressionReasonCodeCountV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionHighlightRefV1 {
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionReasonCodeCountV1 {
    pub reason_code: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionArtifactsV1 {
    // Derived summary root for convenience navigation; not a source-of-truth payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_dir: Option<String>,
    // Optional packaged handoff rooted at the summary layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packed_report: Option<String>,
    // Canonical path to the derived summary/index artifact for first-open routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_json: Option<String>,
    // Presentation-facing static report projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_report: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionItemSummaryV1 {
    pub item_id: String,
    pub kind: RegressionItemKindV1,
    pub name: String,
    pub status: RegressionStatusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reason_code: Option<String>,
    pub lane: RegressionLaneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feature_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<RegressionTimingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempts: Option<RegressionAttemptsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<RegressionEvidenceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<RegressionSourceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<RegressionNotesV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegressionItemKindV1 {
    Script,
    Suite,
    MatrixCase,
    PerfCase,
    CampaignStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegressionStatusV1 {
    Passed,
    FailedDeterministic,
    FailedFlaky,
    FailedTooling,
    FailedTimeout,
    SkippedPolicy,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressionLaneV1 {
    Smoke,
    Correctness,
    Matrix,
    Perf,
    Nightly,
    Full,
}

impl RegressionLaneV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Correctness => "correctness",
            Self::Matrix => "matrix",
            Self::Perf => "perf",
            Self::Nightly | Self::Full => "nightly",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "smoke" => Some(Self::Smoke),
            "correctness" => Some(Self::Correctness),
            "matrix" => Some(Self::Matrix),
            "perf" => Some(Self::Perf),
            "nightly" => Some(Self::Nightly),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}

impl Serialize for RegressionLaneV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegressionLaneV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).ok_or_else(|| {
            serde::de::Error::unknown_variant(
                &value,
                &["smoke", "correctness", "matrix", "perf", "nightly", "full"],
            )
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionTimingV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_delay_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionAttemptsV1 {
    pub attempts_total: u32,
    pub attempts_passed: u32,
    pub attempts_failed: u32,
    pub retried: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shrink_summary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionEvidenceV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_artifact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_dir: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "triage_artifact",
        alias = "triage_json"
    )]
    pub triage_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "script_result",
        alias = "script_result_json"
    )]
    pub script_result_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "share_artifact",
        alias = "ai_packet_dir"
    )]
    pub ai_packet_dir: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "packed_report",
        alias = "pack_path"
    )]
    pub pack_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshots_manifest: Option<String>,
    // Projection-only perf summary path. Useful for perf triage, but not canonical cross-surface
    // vocabulary for generic regression evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf_summary_json: Option<String>,
    // Projection-only compare/check artifact path. Useful for matrix/perf drill-down, but not part
    // of the canonical artifact-path vocabulary shared across all regression rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionSourceV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign_case: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionNotesV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_summary_new_sets_kind_and_schema_version() {
        let summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: "ui-gallery-pr".to_string(),
                lane: RegressionLaneV1::Smoke,
                profile: Some("default".to_string()),
                schema_version: None,
                requested_by: None,
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: "20260306-001".to_string(),
                created_unix_ms: 1,
                started_unix_ms: None,
                finished_unix_ms: None,
                duration_ms: None,
                workspace_root: None,
                out_dir: None,
                tool: "fretboard-dev diag campaign".to_string(),
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            RegressionTotalsV1::default(),
        );

        assert_eq!(summary.schema_version, 1);
        assert_eq!(summary.kind, DIAG_REGRESSION_SUMMARY_KIND_V1);
        assert!(summary.items.is_empty());
        assert!(summary.highlights.is_none());
        assert!(summary.artifacts.is_none());
    }

    #[test]
    fn regression_enums_serialize_as_expected() {
        let item = RegressionItemSummaryV1 {
            item_id: "item-1".to_string(),
            kind: RegressionItemKindV1::MatrixCase,
            name: "matrix check".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("assert.mismatch".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Perf,
            owner: None,
            feature_tags: vec!["overlay".to_string()],
            timing: None,
            attempts: None,
            evidence: None,
            source: None,
            notes: None,
        };

        let value = serde_json::to_value(&item).expect("serialize item");
        assert_eq!(
            value.get("kind").and_then(|v| v.as_str()),
            Some("matrix_case")
        );
        assert_eq!(
            value.get("status").and_then(|v| v.as_str()),
            Some("failed_deterministic")
        );
        assert_eq!(value.get("lane").and_then(|v| v.as_str()), Some("perf"));
        assert!(value.get("source_reason_code").is_none());
    }

    #[test]
    fn regression_lane_full_serializes_as_nightly_and_accepts_full_alias() {
        assert_eq!(
            serde_json::to_value(RegressionLaneV1::Full)
                .expect("serialize lane")
                .as_str(),
            Some("nightly")
        );
        assert_eq!(
            serde_json::from_value::<RegressionLaneV1>(serde_json::json!("full"))
                .expect("deserialize lane alias"),
            RegressionLaneV1::Full
        );
        assert_eq!(
            serde_json::from_value::<RegressionLaneV1>(serde_json::json!("nightly"))
                .expect("deserialize canonical lane"),
            RegressionLaneV1::Nightly
        );
    }

    #[test]
    fn regression_summary_serializes_bounded_minimal_shape() {
        let mut summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: "ui-gallery-pr".to_string(),
                lane: RegressionLaneV1::Smoke,
                profile: None,
                schema_version: None,
                requested_by: None,
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: "run-1".to_string(),
                created_unix_ms: 42,
                started_unix_ms: None,
                finished_unix_ms: None,
                duration_ms: None,
                workspace_root: None,
                out_dir: None,
                tool: "fretboard-dev diag campaign".to_string(),
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            RegressionTotalsV1 {
                items_total: 1,
                passed: 0,
                failed_deterministic: 1,
                failed_flaky: 0,
                failed_tooling: 0,
                failed_timeout: 0,
                skipped_policy: 0,
                quarantined: 0,
            },
        );
        summary.items.push(RegressionItemSummaryV1 {
            item_id: "script-1".to_string(),
            kind: RegressionItemKindV1::Script,
            name: "dialog escape focus restore".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("assert.focus_restore.mismatch".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Smoke,
            owner: None,
            feature_tags: Vec::new(),
            timing: Some(RegressionTimingV1 {
                duration_ms: Some(1420),
                queue_delay_ms: None,
                started_unix_ms: None,
                finished_unix_ms: None,
            }),
            attempts: Some(RegressionAttemptsV1 {
                attempts_total: 1,
                attempts_passed: 0,
                attempts_failed: 1,
                retried: false,
                repeat_summary_path: None,
                shrink_summary_path: None,
            }),
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: Some("target/fret-diag/bundle.schema2.json".to_string()),
                bundle_dir: Some("target/fret-diag".to_string()),
                triage_json: Some("target/fret-diag/triage.json".to_string()),
                script_result_json: Some("target/fret-diag/script.result.json".to_string()),
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: None,
                compare_json: None,
                extra: None,
            }),
            source: Some(RegressionSourceV1 {
                script: Some(
                    "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json".to_string(),
                ),
                suite: Some("ui-gallery".to_string()),
                campaign_case: None,
                metadata: None,
            }),
            notes: Some(RegressionNotesV1 {
                summary: Some("focus did not return to trigger".to_string()),
                details: Vec::new(),
            }),
        });

        let value = serde_json::to_value(&summary).expect("serialize summary");
        assert_eq!(
            value.get("schema_version").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            value.get("kind").and_then(|v| v.as_str()),
            Some(DIAG_REGRESSION_SUMMARY_KIND_V1)
        );
        assert_eq!(
            value.pointer("/campaign/lane").and_then(|v| v.as_str()),
            Some("smoke")
        );
        assert_eq!(
            value.pointer("/items/0/status").and_then(|v| v.as_str()),
            Some("failed_deterministic")
        );
        assert_eq!(
            value
                .pointer("/items/0/evidence/triage_artifact")
                .and_then(|v| v.as_str()),
            Some("target/fret-diag/triage.json")
        );
        assert_eq!(
            value
                .pointer("/items/0/evidence/script_result")
                .and_then(|v| v.as_str()),
            Some("target/fret-diag/script.result.json")
        );
        assert!(value.get("highlights").is_none());
        assert!(value.get("artifacts").is_none());
    }

    #[test]
    fn regression_evidence_accepts_legacy_field_aliases() {
        let parsed: RegressionEvidenceV1 = serde_json::from_value(serde_json::json!({
            "bundle_artifact": "target/fret-diag/bundle.schema2.json",
            "triage_json": "target/fret-diag/triage.json",
            "script_result_json": "target/fret-diag/script.result.json",
            "ai_packet_dir": "target/fret-diag/ai.packet",
            "pack_path": "target/fret-diag/share.zip"
        }))
        .expect("deserialize evidence aliases");

        assert_eq!(
            parsed.triage_json.as_deref(),
            Some("target/fret-diag/triage.json")
        );
        assert_eq!(
            parsed.script_result_json.as_deref(),
            Some("target/fret-diag/script.result.json")
        );
        assert_eq!(
            parsed.ai_packet_dir.as_deref(),
            Some("target/fret-diag/ai.packet")
        );
        assert_eq!(
            parsed.pack_path.as_deref(),
            Some("target/fret-diag/share.zip")
        );
    }

    #[test]
    fn regression_projection_fields_stay_additive_and_optional() {
        let value = serde_json::to_value(RegressionItemSummaryV1 {
            item_id: "perf:ui-gallery".to_string(),
            kind: RegressionItemKindV1::PerfCase,
            name: "ui-gallery".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("diag.perf.threshold_failed".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Perf,
            owner: None,
            feature_tags: Vec::new(),
            timing: None,
            attempts: None,
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: Some("target/fret-diag/bundle.schema2.json".to_string()),
                bundle_dir: None,
                triage_json: None,
                script_result_json: None,
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: Some("target/fret-diag/layout.perf.summary.json".to_string()),
                compare_json: Some("target/fret-diag/check.perf.thresholds.json".to_string()),
                extra: None,
            }),
            source: None,
            notes: None,
        })
        .expect("serialize perf item");

        assert_eq!(
            value
                .pointer("/evidence/perf_summary_json")
                .and_then(|value| value.as_str()),
            Some("target/fret-diag/layout.perf.summary.json")
        );
        assert_eq!(
            value
                .pointer("/evidence/compare_json")
                .and_then(|value| value.as_str()),
            Some("target/fret-diag/check.perf.thresholds.json")
        );
    }

    #[test]
    fn regression_totals_record_status_updates_expected_bucket() {
        let mut totals = RegressionTotalsV1::default();
        totals.record_status(RegressionStatusV1::Passed);
        totals.record_status(RegressionStatusV1::FailedDeterministic);
        totals.record_status(RegressionStatusV1::FailedTooling);

        assert_eq!(totals.items_total, 3);
        assert_eq!(totals.passed, 1);
        assert_eq!(totals.failed_deterministic, 1);
        assert_eq!(totals.failed_tooling, 1);
    }

    #[test]
    fn regression_highlights_from_items_collects_first_failure_and_reason_counts() {
        let highlights = RegressionHighlightsV1::from_items(&[
            RegressionItemSummaryV1 {
                item_id: "ok".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "ok".to_string(),
                status: RegressionStatusV1::Passed,
                reason_code: None,
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
            RegressionItemSummaryV1 {
                item_id: "bad".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "bad".to_string(),
                status: RegressionStatusV1::FailedDeterministic,
                reason_code: Some("assert.mismatch".to_string()),
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
            RegressionItemSummaryV1 {
                item_id: "bad-2".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "bad-2".to_string(),
                status: RegressionStatusV1::FailedDeterministic,
                reason_code: Some("assert.mismatch".to_string()),
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
        ])
        .expect("expected highlights");

        assert_eq!(
            highlights
                .first_failure
                .as_ref()
                .map(|v| v.item_id.as_str()),
            Some("bad")
        );
        assert_eq!(
            highlights
                .top_reason_codes
                .first()
                .map(|v| v.reason_code.as_str()),
            Some("assert.mismatch")
        );
        assert_eq!(
            highlights.top_reason_codes.first().map(|v| v.count),
            Some(2)
        );
    }
}
