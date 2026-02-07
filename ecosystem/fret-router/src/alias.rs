use std::collections::HashSet;

use crate::{PathPattern, PathPatternError, QueryPair, RouteLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryKeyAlias {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteAliasRule {
    pub from: PathPattern,
    pub to: PathPattern,
    pub query_aliases: Vec<QueryKeyAlias>,
    pub query_defaults: Vec<QueryPair>,
    pub preserve_fragment: bool,
}

impl RouteAliasRule {
    pub fn new(from: PathPattern, to: PathPattern) -> Self {
        Self {
            from,
            to,
            query_aliases: Vec::new(),
            query_defaults: Vec::new(),
            preserve_fragment: true,
        }
    }

    pub fn from_paths(from: &str, to: &str) -> Result<Self, PathPatternError> {
        Ok(Self::new(
            PathPattern::parse(from)?,
            PathPattern::parse(to)?,
        ))
    }

    pub fn with_query_alias(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.query_aliases.push(QueryKeyAlias {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    pub fn with_query_default(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        self.query_defaults.push(QueryPair {
            key: key.into(),
            value,
        });
        self
    }

    pub fn preserve_fragment(mut self, preserve: bool) -> Self {
        self.preserve_fragment = preserve;
        self
    }

    pub fn apply(&self, location: &RouteLocation) -> Option<RouteLocation> {
        let location = location.canonicalized();
        let matched = self.from.match_path(location.path.as_str())?;

        let mut next = RouteLocation::from_path(self.to.format_path(matched.params.as_slice())?);
        next.query = location.query.clone();

        for alias in &self.query_aliases {
            for pair in &mut next.query {
                if pair.key == alias.from {
                    pair.key = alias.to.clone();
                }
            }
        }

        for default in &self.query_defaults {
            if !next.query.iter().any(|pair| pair.key == default.key) {
                next.query.push(default.clone());
            }
        }

        if self.preserve_fragment {
            next.fragment = location.fragment;
        }

        next.canonicalize();
        Some(next)
    }
}

#[derive(Debug, Clone)]
pub struct RouteAliasTable {
    rules: Vec<RouteAliasRule>,
    max_hops: usize,
}

impl RouteAliasTable {
    pub fn new(rules: Vec<RouteAliasRule>) -> Self {
        Self { rules, max_hops: 8 }
    }

    pub fn with_max_hops(mut self, max_hops: usize) -> Self {
        self.max_hops = max_hops;
        self
    }

    pub fn rules(&self) -> &[RouteAliasRule] {
        self.rules.as_slice()
    }

    pub fn resolve_once(&self, location: &RouteLocation) -> Option<RouteLocation> {
        self.rules.iter().find_map(|rule| rule.apply(location))
    }

    pub fn resolve(&self, location: &RouteLocation) -> Result<RouteLocation, AliasResolveError> {
        let mut current = location.canonicalized();
        let mut seen = HashSet::<String>::new();
        seen.insert(current.to_url());

        for _ in 0..self.max_hops {
            let Some(next) = self.resolve_once(&current) else {
                return Ok(current);
            };

            if next == current {
                return Ok(current);
            }

            let signature = next.to_url();
            if !seen.insert(signature.clone()) {
                return Err(AliasResolveError::CycleDetected { at: signature });
            }

            current = next;
        }

        Err(AliasResolveError::TooManyRedirects {
            hops: self.max_hops,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasResolveError {
    CycleDetected { at: String },
    TooManyRedirects { hops: usize },
}

impl std::fmt::Display for AliasResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected { at } => write!(f, "route alias cycle detected at '{at}'"),
            Self::TooManyRedirects { hops } => write!(f, "route alias exceeded hop limit {hops}"),
        }
    }
}

impl std::error::Error for AliasResolveError {}

#[cfg(test)]
mod tests {
    use super::{AliasResolveError, RouteAliasRule, RouteAliasTable};
    use crate::RouteLocation;

    #[test]
    fn route_alias_rewrites_legacy_path_with_params() {
        let rule = RouteAliasRule::from_paths("/u/:id", "/users/:id").expect("rule should parse");
        let location = RouteLocation::parse("/u/42?tab=profile#overview");

        let mapped = rule.apply(&location).expect("alias should apply");
        assert_eq!(mapped.to_url(), "/users/42?tab=profile#overview");
    }

    #[test]
    fn route_alias_can_rename_query_keys_and_add_defaults() {
        let rule = RouteAliasRule::from_paths("/gallery", "/gallery")
            .expect("rule should parse")
            .with_query_alias("start_page", "page")
            .with_query_default("source", Some("legacy".to_string()));

        let location = RouteLocation::parse("/gallery?start_page=button");
        let mapped = rule.apply(&location).expect("alias should apply");

        assert_eq!(mapped.to_url(), "/gallery?page=button&source=legacy");
    }

    #[test]
    fn alias_table_resolves_chained_aliases() {
        let table = RouteAliasTable::new(vec![
            RouteAliasRule::from_paths("/old/:id", "/legacy/:id").expect("rule should parse"),
            RouteAliasRule::from_paths("/legacy/:id", "/users/:id").expect("rule should parse"),
        ]);

        let resolved = table
            .resolve(&RouteLocation::parse("/old/7?tab=profile"))
            .expect("alias should resolve");

        assert_eq!(resolved.to_url(), "/users/7?tab=profile");
    }

    #[test]
    fn alias_table_detects_cycles() {
        let table = RouteAliasTable::new(vec![
            RouteAliasRule::from_paths("/a", "/b").expect("rule should parse"),
            RouteAliasRule::from_paths("/b", "/a").expect("rule should parse"),
        ]);

        let error = table
            .resolve(&RouteLocation::from_path("/a"))
            .expect_err("cycle should fail");

        assert!(matches!(error, AliasResolveError::CycleDetected { .. }));
    }

    #[test]
    fn alias_table_respects_hop_limit() {
        let table = RouteAliasTable::new(vec![
            RouteAliasRule::from_paths("/a", "/b").expect("rule should parse"),
            RouteAliasRule::from_paths("/b", "/c").expect("rule should parse"),
            RouteAliasRule::from_paths("/c", "/d").expect("rule should parse"),
        ])
        .with_max_hops(2);

        let error = table
            .resolve(&RouteLocation::from_path("/a"))
            .expect_err("hop limit should fail");

        assert_eq!(error, AliasResolveError::TooManyRedirects { hops: 2 });
    }
}
