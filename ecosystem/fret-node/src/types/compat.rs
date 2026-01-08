//! Type compatibility primitives used by connection rules and profiles.

use super::TypeDesc;

/// Result of a type compatibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeCompatibilityResult {
    Compatible,
    Incompatible { reason: String },
}

impl TypeCompatibilityResult {
    pub fn is_compatible(&self) -> bool {
        matches!(self, Self::Compatible)
    }
}

/// Compatibility policy between two types.
///
/// The direction is `from -> to` (assignability), matching edge direction for data flow.
pub trait TypeCompatibility {
    fn compatible(&mut self, from: &TypeDesc, to: &TypeDesc) -> TypeCompatibilityResult;
}

/// A small, conservative default compatibility table.
///
/// This is intentionally minimal: rich domains (shader precision/space, JSON schema, tool
/// signatures) should implement their own policy.
#[derive(Debug, Default, Clone)]
pub struct DefaultTypeCompatibility;

impl TypeCompatibility for DefaultTypeCompatibility {
    fn compatible(&mut self, from: &TypeDesc, to: &TypeDesc) -> TypeCompatibilityResult {
        use TypeCompatibilityResult::{Compatible, Incompatible};
        use TypeDesc::*;

        match (from, to) {
            (_, Any) => Compatible,
            (Any, _) => Compatible,
            (_, Unknown) => Compatible,
            (Unknown, _) => Compatible,

            (Never, _) => Compatible,
            (_, Never) => Incompatible {
                reason: "cannot assign into never".to_string(),
            },

            (Null, Null) => Compatible,
            (Null, Option { .. }) => Compatible,
            (Null, Union { types }) => {
                if types.iter().any(|t| matches!(t, Null)) {
                    Compatible
                } else {
                    Incompatible {
                        reason: "null is not a member of the union".to_string(),
                    }
                }
            }

            (Bool, Bool) | (Int, Int) | (Float, Float) | (String, String) | (Bytes, Bytes) => {
                Compatible
            }

            (Int, Float) => Compatible,

            (Option { of }, other) => self.compatible(of, other),
            (other, Option { of }) => {
                if matches!(other, Null) {
                    Compatible
                } else {
                    self.compatible(other, of)
                }
            }

            (Union { types }, other) => {
                for t in types {
                    if !self.compatible(t, other).is_compatible() {
                        return Incompatible {
                            reason: "union member is incompatible".to_string(),
                        };
                    }
                }
                Compatible
            }
            (other, Union { types }) => {
                for t in types {
                    if self.compatible(other, t).is_compatible() {
                        return Compatible;
                    }
                }
                Incompatible {
                    reason: "no union member is compatible".to_string(),
                }
            }

            (List { of: a }, List { of: b }) => self.compatible(a, b),

            (Map { key: ak, value: av }, Map { key: bk, value: bv }) => {
                if !self.compatible(ak, bk).is_compatible() {
                    return Incompatible {
                        reason: "map key types are incompatible".to_string(),
                    };
                }
                self.compatible(av, bv)
            }

            (
                Object {
                    fields: a_fields,
                    open: a_open,
                },
                Object {
                    fields: b_fields,
                    open: b_open,
                },
            ) => {
                // Conservative rule:
                // - closed target requires exact field set,
                // - open target requires at least the target fields.
                if !*b_open && a_fields.len() != b_fields.len() {
                    return Incompatible {
                        reason: "closed object requires exact field set".to_string(),
                    };
                }
                for (name, b_ty) in b_fields {
                    let Some(a_ty) = a_fields.get(name) else {
                        return Incompatible {
                            reason: format!("missing required field: {name}"),
                        };
                    };
                    if !self.compatible(a_ty, b_ty).is_compatible() {
                        return Incompatible {
                            reason: format!("field type mismatch: {name}"),
                        };
                    }
                }
                if !*b_open && *a_open {
                    // An open source may contain additional unknown fields; keep conservative.
                    return Incompatible {
                        reason: "open object is not assignable to closed object".to_string(),
                    };
                }
                Compatible
            }

            (Var { id: _ }, _) | (_, Var { id: _ }) => Compatible,

            (
                Opaque {
                    key: ak,
                    params: ap,
                },
                Opaque {
                    key: bk,
                    params: bp,
                },
            ) => {
                if ak != bk {
                    return Incompatible {
                        reason: "opaque type keys differ".to_string(),
                    };
                }
                if ap.len() != bp.len() {
                    return Incompatible {
                        reason: "opaque type arity differs".to_string(),
                    };
                }
                for (a, b) in ap.iter().zip(bp.iter()) {
                    if !self.compatible(a, b).is_compatible() {
                        return Incompatible {
                            reason: "opaque type parameter mismatch".to_string(),
                        };
                    }
                }
                Compatible
            }

            _ => Incompatible {
                reason: "types are incompatible".to_string(),
            },
        }
    }
}
