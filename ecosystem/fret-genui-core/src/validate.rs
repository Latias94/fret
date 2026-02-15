//! Spec structural validation.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::catalog::CatalogV1;
use crate::spec::{ElementKey, SpecV1};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecIssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecIssueCode {
    MissingRoot,
    RootNotFound,
    EmptySpec,
    MissingChild,
    VisibleInProps,
    OnInProps,
    RepeatInProps,
    OrphanedElement,
    SchemaVersionUnsupported,
    UnknownComponent,
    InvalidPropKey,
    UnknownEvent,
    UnknownAction,
    InvalidActionParamKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecIssue {
    pub severity: SpecIssueSeverity,
    pub code: SpecIssueCode,
    pub message: String,
    pub element_key: Option<ElementKey>,
}

#[derive(Debug, Clone)]
pub struct ValidateSpecOptions {
    pub check_orphans: bool,
    pub supported_schema_versions: BTreeSet<u32>,
    pub catalog: Option<Arc<CatalogV1>>,
    pub catalog_validation: ValidationMode,
}

impl Default for ValidateSpecOptions {
    fn default() -> Self {
        let mut supported_schema_versions = BTreeSet::new();
        supported_schema_versions.insert(1);
        Self {
            check_orphans: false,
            supported_schema_versions,
            catalog: None,
            catalog_validation: ValidationMode::Ignore,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Treat catalog mismatches as errors.
    Strict,
    /// Downgrade catalog mismatches to warnings.
    Warn,
    /// Do not emit catalog-related issues.
    #[default]
    Ignore,
}

#[derive(Debug, Clone)]
pub struct SpecValidationIssues {
    pub valid: bool,
    pub issues: Vec<SpecIssue>,
}

pub fn validate_spec(spec: &SpecV1, options: ValidateSpecOptions) -> SpecValidationIssues {
    let mut issues: Vec<SpecIssue> = Vec::new();
    let catalog = options.catalog;
    let catalog_mode = options.catalog_validation;

    if !options
        .supported_schema_versions
        .contains(&spec.schema_version)
    {
        issues.push(SpecIssue {
            severity: SpecIssueSeverity::Error,
            code: SpecIssueCode::SchemaVersionUnsupported,
            message: format!("Unsupported schema_version: {}", spec.schema_version),
            element_key: None,
        });
    }

    if spec.elements.is_empty() {
        issues.push(SpecIssue {
            severity: SpecIssueSeverity::Error,
            code: SpecIssueCode::EmptySpec,
            message: "Spec has no elements.".to_string(),
            element_key: None,
        });
        return SpecValidationIssues {
            valid: false,
            issues,
        };
    }

    if spec.root.0.is_empty() {
        issues.push(SpecIssue {
            severity: SpecIssueSeverity::Error,
            code: SpecIssueCode::MissingRoot,
            message: "Spec has no root element defined.".to_string(),
            element_key: None,
        });
        return SpecValidationIssues {
            valid: false,
            issues,
        };
    }

    if !spec.elements.contains_key(&spec.root) {
        issues.push(SpecIssue {
            severity: SpecIssueSeverity::Error,
            code: SpecIssueCode::RootNotFound,
            message: format!("Root element {:?} not found in elements map.", spec.root),
            element_key: Some(spec.root.clone()),
        });
    }

    for (key, element) in spec.elements.iter() {
        if let Some(catalog) = catalog.as_deref() {
            if let Some(severity) = catalog_issue_severity(catalog_mode) {
                let component = catalog.components.get(element.ty.as_str());
                if component.is_none() {
                    issues.push(SpecIssue {
                        severity,
                        code: SpecIssueCode::UnknownComponent,
                        message: format!(
                            "Element {:?} uses unknown component type: {}",
                            key, element.ty
                        ),
                        element_key: Some(key.clone()),
                    });
                }

                if let Some(component) = component {
                    for prop_key in element.props.keys() {
                        if !component.props.contains_key(prop_key) {
                            issues.push(SpecIssue {
                                severity,
                                code: SpecIssueCode::InvalidPropKey,
                                message: format!(
                                    "Element {:?} has unsupported prop key {:?} for component {}",
                                    key, prop_key, element.ty
                                ),
                                element_key: Some(key.clone()),
                            });
                        }
                    }

                    if let Some(on) = element.on.as_ref() {
                        if !component.events.is_empty() {
                            for event in on.keys() {
                                if !component.events.contains(event.as_str()) {
                                    issues.push(SpecIssue {
                                        severity,
                                        code: SpecIssueCode::UnknownEvent,
                                        message: format!(
                                            "Element {:?} binds unknown event {:?} for component {}",
                                            key, event, element.ty
                                        ),
                                        element_key: Some(key.clone()),
                                    });
                                }
                            }
                        }

                        for (event, binding) in on.iter() {
                            for b in binding.iter() {
                                let Some(action_def) = catalog.actions.get(b.action.as_str())
                                else {
                                    issues.push(SpecIssue {
                                        severity,
                                        code: SpecIssueCode::UnknownAction,
                                        message: format!(
                                            "Element {:?} binds unknown action {:?} for event {:?}",
                                            key, b.action, event
                                        ),
                                        element_key: Some(key.clone()),
                                    });
                                    continue;
                                };

                                // If the action declares param keys, enforce them. If it declares no keys,
                                // allow any params (action semantics are app-owned).
                                if !action_def.params.is_empty() {
                                    if let Some(params) = b.params.as_ref() {
                                        for param_key in params.keys() {
                                            if !action_def.params.contains_key(param_key) {
                                                issues.push(SpecIssue {
                                                    severity,
                                                    code: SpecIssueCode::InvalidActionParamKey,
                                                    message: format!(
                                                        "Element {:?} uses unsupported param key {:?} for action {:?}",
                                                        key, param_key, b.action
                                                    ),
                                                    element_key: Some(key.clone()),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if element.props.contains_key("visible") {
            issues.push(SpecIssue {
                severity: SpecIssueSeverity::Error,
                code: SpecIssueCode::VisibleInProps,
                message: format!(
                    "Element {:?} has \"visible\" inside props; it must be a top-level field.",
                    key
                ),
                element_key: Some(key.clone()),
            });
        }
        if element.props.contains_key("on") {
            issues.push(SpecIssue {
                severity: SpecIssueSeverity::Error,
                code: SpecIssueCode::OnInProps,
                message: format!(
                    "Element {:?} has \"on\" inside props; it must be a top-level field.",
                    key
                ),
                element_key: Some(key.clone()),
            });
        }
        if element.props.contains_key("repeat") {
            issues.push(SpecIssue {
                severity: SpecIssueSeverity::Error,
                code: SpecIssueCode::RepeatInProps,
                message: format!(
                    "Element {:?} has \"repeat\" inside props; it must be a top-level field.",
                    key
                ),
                element_key: Some(key.clone()),
            });
        }
        for child in element.children.iter() {
            if !spec.elements.contains_key(child) {
                issues.push(SpecIssue {
                    severity: SpecIssueSeverity::Error,
                    code: SpecIssueCode::MissingChild,
                    message: format!("Element {:?} references missing child {:?}.", key, child),
                    element_key: Some(key.clone()),
                });
            }
        }
    }

    if options.check_orphans {
        let mut reachable: BTreeSet<ElementKey> = BTreeSet::new();

        fn walk(
            cur: &ElementKey,
            elements: &BTreeMap<ElementKey, crate::spec::ElementV1>,
            reachable: &mut BTreeSet<ElementKey>,
        ) {
            if reachable.contains(cur) {
                return;
            }
            reachable.insert(cur.clone());
            let Some(el) = elements.get(cur) else { return };
            for child in el.children.iter() {
                if elements.contains_key(child) {
                    walk(child, elements, reachable);
                }
            }
        }

        if spec.elements.contains_key(&spec.root) {
            walk(&spec.root, &spec.elements, &mut reachable);
        }

        for key in spec.elements.keys() {
            if !reachable.contains(key) {
                issues.push(SpecIssue {
                    severity: SpecIssueSeverity::Warning,
                    code: SpecIssueCode::OrphanedElement,
                    message: format!("Element {:?} is not reachable from root.", key),
                    element_key: Some(key.clone()),
                });
            }
        }
    }

    let valid = !issues
        .iter()
        .any(|i| i.severity == SpecIssueSeverity::Error);
    SpecValidationIssues { valid, issues }
}

fn catalog_issue_severity(mode: ValidationMode) -> Option<SpecIssueSeverity> {
    match mode {
        ValidationMode::Strict => Some(SpecIssueSeverity::Error),
        ValidationMode::Warn => Some(SpecIssueSeverity::Warning),
        ValidationMode::Ignore => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogActionV1, CatalogComponentV1, CatalogPropV1, CatalogV1};
    use crate::spec::{ElementKey, ElementV1, SpecV1};
    use serde_json::Value;

    fn key(s: &str) -> ElementKey {
        ElementKey(s.to_string())
    }

    #[test]
    fn rejects_missing_root() {
        let spec = SpecV1 {
            schema_version: 1,
            root: key(""),
            elements: BTreeMap::new(),
            state: None,
        };
        let out = validate_spec(&spec, ValidateSpecOptions::default());
        assert_eq!(out.valid, false);
        assert!(
            out.issues
                .iter()
                .any(|i| i.code == SpecIssueCode::EmptySpec)
        );
    }

    #[test]
    fn rejects_missing_child() {
        let mut elements = BTreeMap::new();
        elements.insert(
            key("root"),
            ElementV1 {
                ty: "Card".to_string(),
                props: Default::default(),
                children: vec![key("missing")],
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
        let out = validate_spec(&spec, ValidateSpecOptions::default());
        assert_eq!(out.valid, false);
        assert!(
            out.issues
                .iter()
                .any(|i| i.code == SpecIssueCode::MissingChild)
        );
    }

    #[test]
    fn rejects_on_inside_props() {
        let mut elements = BTreeMap::new();
        let mut props = serde_json::Map::new();
        props.insert("on".to_string(), Value::Object(serde_json::Map::new()));
        elements.insert(
            key("root"),
            ElementV1 {
                ty: "Card".to_string(),
                props,
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
        let out = validate_spec(&spec, ValidateSpecOptions::default());
        assert_eq!(out.valid, false);
        assert!(
            out.issues
                .iter()
                .any(|i| i.code == SpecIssueCode::OnInProps)
        );
    }

    #[test]
    fn catalog_validation_rejects_unknown_component_and_prop_keys() {
        let mut catalog = CatalogV1::new();
        catalog.components.insert(
            "Text".to_string(),
            CatalogComponentV1 {
                description: None,
                props: {
                    let mut p = BTreeMap::new();
                    p.insert("text".to_string(), CatalogPropV1::new());
                    p
                },
                events: Default::default(),
            },
        );
        catalog
            .actions
            .insert("setState".to_string(), CatalogActionV1::new());

        let mut elements = BTreeMap::new();
        let mut props = serde_json::Map::new();
        props.insert("nope".to_string(), Value::Bool(true));
        elements.insert(
            key("root"),
            ElementV1 {
                ty: "Text".to_string(),
                props,
                children: vec![],
                visible: None,
                on: None,
                repeat: None,
            },
        );
        elements.insert(
            key("bad"),
            ElementV1 {
                ty: "NotAComponent".to_string(),
                props: Default::default(),
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

        let out = validate_spec(
            &spec,
            ValidateSpecOptions {
                check_orphans: false,
                supported_schema_versions: {
                    let mut set = BTreeSet::new();
                    set.insert(1);
                    set
                },
                catalog: Some(Arc::new(catalog)),
                catalog_validation: ValidationMode::Strict,
            },
        );

        assert!(!out.valid);
        assert!(
            out.issues
                .iter()
                .any(|i| i.code == SpecIssueCode::InvalidPropKey)
        );
        assert!(
            out.issues
                .iter()
                .any(|i| i.code == SpecIssueCode::UnknownComponent)
        );
    }
}
