use fret_diag_protocol::{UiActionScriptV1, UiActionScriptV2, UiActionStepV2};
use serde_json::Value;

pub(crate) fn script_schema_version_from_value(value: &Value) -> u32 {
    value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32
}

pub(crate) fn upgrade_script_json_value_to_v2_if_needed(
    value: Value,
) -> Result<(Value, bool), String> {
    let schema_version = script_schema_version_from_value(&value);
    match schema_version {
        1 => {
            let script: UiActionScriptV1 =
                serde_json::from_value(value).map_err(|e| e.to_string())?;
            let upgraded = UiActionScriptV2 {
                schema_version: 2,
                meta: script.meta,
                steps: script.steps.into_iter().map(UiActionStepV2::from).collect(),
            };
            let value = serde_json::to_value(upgraded).map_err(|e| e.to_string())?;
            Ok((value, true))
        }
        2 => Ok((value, false)),
        _ => Err(format!(
            "unknown script schema_version (expected 1 or 2): {}",
            schema_version
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag_protocol::UiActionStepV1;

    #[test]
    fn upgrade_v1_to_v2_sets_schema_version_and_changes() {
        let v1 = UiActionScriptV1 {
            schema_version: 1,
            meta: None,
            steps: vec![UiActionStepV1::WaitFrames { n: 1 }],
        };
        let value = serde_json::to_value(v1).expect("v1 to json should succeed");
        let (upgraded, changed) =
            upgrade_script_json_value_to_v2_if_needed(value).expect("upgrade should succeed");
        assert!(changed);
        assert_eq!(script_schema_version_from_value(&upgraded), 2);
    }

    #[test]
    fn upgrade_rejects_unknown_schema_versions() {
        let value = serde_json::json!({
            "schema_version": 99,
            "steps": [],
        });
        let err = upgrade_script_json_value_to_v2_if_needed(value).unwrap_err();
        assert!(err.contains("unknown script schema_version"));
    }
}
