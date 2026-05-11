use serde::{Deserialize, Serialize};

use crate::MechanismPredicate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanismDomain {
    Layout,
    Invalidation,
    HitTest,
    Semantics,
    Overlay,
    Focus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismSuite<S> {
    pub schema_version: u32,
    pub suite_id: String,
    pub owner_layer: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<MechanismDomain>,
    pub cases: Vec<MechanismCase<S>>,
}

impl<S> MechanismSuite<S> {
    pub fn from_json_str(raw: &str) -> Result<Self, MechanismSuiteLoadError>
    where
        S: for<'de> Deserialize<'de>,
    {
        let suite: Self = serde_json::from_str(raw)?;
        if suite.schema_version == 0 {
            return Err(MechanismSuiteLoadError::InvalidSchemaVersion {
                suite_id: suite.suite_id,
                schema_version: suite.schema_version,
            });
        }
        Ok(suite)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismCase<S> {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_layer: Option<String>,
    pub scenario: S,
    pub oracle: MechanismOracle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<CaseEvidence>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MechanismOracle {
    #[serde(default)]
    pub predicates: Vec<MechanismPredicate>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CaseEvidence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum MechanismSuiteLoadError {
    #[error("failed to parse mechanism suite JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("mechanism suite {suite_id:?} has invalid schema_version={schema_version}")]
    InvalidSchemaVersion {
        suite_id: String,
        schema_version: u32,
    },
}
