//! Spec auto-fix helpers (json-render-inspired).
//!
//! This module provides small, conservative fixups for common LLM mistakes.
//! It intentionally does not introduce policy into the renderer; apps opt-in.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::spec::{ElementV1, OnBindingV1, RepeatV1, SpecV1};
use crate::validate::{SpecIssue, SpecIssueSeverity};
use crate::visibility::VisibilityConditionV1;

#[derive(Debug, Clone, Default)]
pub struct SpecFixups {
    pub fixes: Vec<String>,
}

/// Auto-fix common spec issues and return a corrected copy.
///
/// Currently fixes (when parsable):
/// - `visible` inside `props` → moved to element level
/// - `on` inside `props` → moved to element level
/// - `repeat` inside `props` → moved to element level
pub fn auto_fix_spec(spec: &SpecV1) -> (SpecV1, SpecFixups) {
    let mut out = spec.clone();
    let mut fixups = SpecFixups::default();

    for (key, element) in out.elements.iter_mut() {
        let moved = auto_fix_element(element, &mut fixups);
        if moved > 0 {
            fixups
                .fixes
                .push(format!("Fixed {moved} field(s) on element {:?}", key));
        }
    }

    (out, fixups)
}

fn auto_fix_element(element: &mut ElementV1, fixups: &mut SpecFixups) -> usize {
    let mut moved: usize = 0;

    if let Some(v) = element.props.get("visible").cloned()
        && let Ok(cond) = serde_json::from_value::<VisibilityConditionV1>(v)
    {
        element.visible = Some(cond);
        let _ = element.props.remove("visible");
        moved = moved.saturating_add(1);
        fixups
            .fixes
            .push("Moved \"visible\" from props to element level.".to_string());
    }

    if let Some(v) = element.props.get("on").cloned()
        && let Ok(on) = parse_on(v)
    {
        element.on = Some(on);
        let _ = element.props.remove("on");
        moved = moved.saturating_add(1);
        fixups
            .fixes
            .push("Moved \"on\" from props to element level.".to_string());
    }

    if let Some(v) = element.props.get("repeat").cloned()
        && let Ok(repeat) = serde_json::from_value::<RepeatV1>(v)
    {
        element.repeat = Some(repeat);
        let _ = element.props.remove("repeat");
        moved = moved.saturating_add(1);
        fixups
            .fixes
            .push("Moved \"repeat\" from props to element level.".to_string());
    }

    moved
}

fn parse_on(v: Value) -> Result<BTreeMap<String, OnBindingV1>, serde_json::Error> {
    serde_json::from_value::<BTreeMap<String, OnBindingV1>>(v)
}

/// Format validation issues into a human-readable string suitable for inclusion in an AI repair prompt.
pub fn format_spec_issues_for_repair_prompt(issues: &[SpecIssue]) -> String {
    let errors = issues
        .iter()
        .filter(|i| i.severity == SpecIssueSeverity::Error)
        .collect::<Vec<_>>();
    if errors.is_empty() {
        return String::new();
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push("The generated UI spec has the following errors:".to_string());
    for issue in errors {
        lines.push(format!("- {} ({:?})", issue.message, issue.code));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ElementKey, SpecV1};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn key(s: &str) -> ElementKey {
        ElementKey(s.to_string())
    }

    #[test]
    fn auto_fix_moves_visible_on_repeat_out_of_props_when_parsable() {
        let mut elements = BTreeMap::new();
        elements.insert(
            key("root"),
            ElementV1 {
                ty: "Button".to_string(),
                props: serde_json::from_value(json!({
                    "visible": {"$state": "/flag"},
                    "on": {"press": {"action": "setState", "params": {"statePath": "/x", "value": 1}}},
                    "repeat": {"statePath": "/todos", "key": "id"},
                }))
                .unwrap(),
                children: vec![],
                visible: None,
                on: None,
                repeat: None,
            },
        );
        let spec = SpecV1 {
            schema_version: 1,
            root: key("root"),
            elements,
            state: None,
        };

        let (fixed, _fixups) = auto_fix_spec(&spec);
        let el = fixed.elements.get(&key("root")).unwrap();
        assert!(el.visible.is_some());
        assert!(el.on.is_some());
        assert!(el.repeat.is_some());
        assert!(!el.props.contains_key("visible"));
        assert!(!el.props.contains_key("on"));
        assert!(!el.props.contains_key("repeat"));
    }

    #[test]
    fn auto_fix_is_conservative_when_values_are_not_parsable() {
        let mut elements = BTreeMap::new();
        elements.insert(
            key("root"),
            ElementV1 {
                ty: "Button".to_string(),
                props: serde_json::from_value(json!({
                    "visible": {"$state": 123}, // invalid type
                    "on": 123, // invalid
                    "repeat": {"key": "id"}, // missing statePath
                }))
                .unwrap(),
                children: vec![],
                visible: None,
                on: None,
                repeat: None,
            },
        );
        let spec = SpecV1 {
            schema_version: 1,
            root: key("root"),
            elements,
            state: None,
        };

        let (fixed, _fixups) = auto_fix_spec(&spec);
        let el = fixed.elements.get(&key("root")).unwrap();
        assert!(el.visible.is_none());
        assert!(el.on.is_none());
        assert!(el.repeat.is_none());
        assert!(el.props.contains_key("visible"));
        assert!(el.props.contains_key("on"));
        assert!(el.props.contains_key("repeat"));
    }
}
