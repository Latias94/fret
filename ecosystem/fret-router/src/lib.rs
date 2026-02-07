//! Router-oriented URL helpers for Fret ecosystem crates.
//!
//! This crate is intentionally lightweight in v1: it provides portable parsing helpers and
//! leaves policy-heavy routing behavior in app/ecosystem layers.

mod alias;
mod base_path;
mod hash;
mod history;
mod location;
mod navigation;
mod path;
mod query;
#[cfg(feature = "query-integration")]
mod query_integration;

pub use alias::{AliasResolveError, QueryKeyAlias, RouteAliasRule, RouteAliasTable};
pub use base_path::{apply_base_path, normalize_base_path, strip_base_path};
pub use hash::{hash_contains_token, hash_token};
pub use history::MemoryHistory;
pub use location::{RouteLocation, canonicalize_query_pairs};
pub use navigation::NavigationAction;
pub use path::{
    PathMatch, PathParam, PathPattern, PathPatternError, RouteEntry, RouteResolution, RouteTable,
    WILDCARD_PARAM, normalize_path,
};
pub use query::{
    QueryPair, first_query_value, first_query_value_from_search_or_hash, format_query_pairs,
    parse_query_pairs, query_values,
};
#[cfg(feature = "query-integration")]
pub use query_integration::{
    NamespaceInvalidationRule, RouteChangePolicy, collect_invalidated_namespaces,
    route_change_matches, route_query_key, route_query_key_with,
};

#[cfg(all(
    target_arch = "wasm32",
    any(feature = "web-history", feature = "hash-routing")
))]
pub mod web;
