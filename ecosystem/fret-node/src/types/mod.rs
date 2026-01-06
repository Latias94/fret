//! Type system primitives for node graph connections.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Stable identifier for a type variable used during unification/inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypeVarId(pub u32);

/// A runtime, serializable type descriptor.
///
/// This type is intentionally policy-free:
/// - It does not embed implicit cast rules.
/// - It does not define domain-specific compatibility (shader/tool/workflow rules).
/// Those decisions live in the rules/profile layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeDesc {
    /// Top type: accepts anything.
    Any,
    /// Unknown type: inference placeholder.
    Unknown,
    /// Bottom type: no value can exist (optional for MVP, but reserved).
    Never,
    /// Explicit null value.
    Null,

    /// Boolean.
    Bool,
    /// Integer number.
    Int,
    /// Floating-point number.
    Float,
    /// Unicode string.
    String,
    /// Raw bytes.
    Bytes,

    /// List container.
    List {
        /// Element type.
        of: Box<TypeDesc>,
    },

    /// Map container.
    Map {
        /// Key type.
        key: Box<TypeDesc>,
        /// Value type.
        value: Box<TypeDesc>,
    },

    /// Structural object/record type.
    Object {
        /// Field map. Stable ordering is required for deterministic serialization/diffing.
        fields: BTreeMap<String, TypeDesc>,
        /// If true, additional unknown fields are permitted.
        open: bool,
    },

    /// Union (sum) type.
    ///
    /// The rules layer is responsible for normalization (dedup/sort/flatten) and compatibility.
    Union {
        /// Member types.
        types: Vec<TypeDesc>,
    },

    /// Optional type (syntax sugar for `Union([T, Null])`).
    Option {
        /// Inner type.
        of: Box<TypeDesc>,
    },

    /// Type variable (generic).
    Var {
        /// Variable id.
        id: TypeVarId,
    },

    /// Domain-specific extension point.
    ///
    /// Examples:
    /// - `shader.vec` with parameters (dimensions, precision, space),
    /// - `tool.signature` with structured parameter/result types,
    /// - `schema.json` with a schema payload.
    Opaque {
        /// Domain key (namespaced string).
        key: String,
        /// Optional parameters.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        params: Vec<TypeDesc>,
    },
}
