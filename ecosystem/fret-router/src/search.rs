use crate::{QueryPair, RouteLocation, canonicalize_query_pairs};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchMap {
    pairs: Vec<QueryPair>,
}

impl SearchMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs(mut pairs: Vec<QueryPair>) -> Self {
        canonicalize_query_pairs(&mut pairs);
        Self { pairs }
    }

    pub fn from_location(location: &RouteLocation) -> Self {
        Self::from_pairs(location.canonicalized().query)
    }

    pub fn pairs(&self) -> &[QueryPair] {
        self.pairs.as_slice()
    }

    pub fn into_pairs(self) -> Vec<QueryPair> {
        self.pairs
    }

    pub fn first(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|pair| pair.key == key)
            .and_then(|pair| pair.value.as_deref())
    }

    pub fn values(&self, key: &str) -> Vec<Option<&str>> {
        self.pairs
            .iter()
            .filter(|pair| pair.key == key)
            .map(|pair| pair.value.as_deref())
            .collect()
    }

    pub fn with(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        let key = key.into();
        self.pairs.retain(|pair| pair.key != key);
        self.pairs.push(QueryPair { key, value });
        canonicalize_query_pairs(&mut self.pairs);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchValidationError {
    message: String,
}

impl SearchValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl std::fmt::Display for SearchValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::error::Error for SearchValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchValidationMode {
    Lenient,
    Strict,
}

pub type ValidateSearchFn =
    fn(location: &RouteLocation, search: &SearchMap) -> Result<SearchMap, SearchValidationError>;

#[derive(Debug, Clone)]
pub struct RouteSearchTable<R> {
    validators: std::collections::HashMap<R, ValidateSearchFn>,
}

impl<R> Default for RouteSearchTable<R> {
    fn default() -> Self {
        Self {
            validators: std::collections::HashMap::new(),
        }
    }
}

impl<R> RouteSearchTable<R>
where
    R: std::hash::Hash + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, route: R, validate: ValidateSearchFn) -> Option<ValidateSearchFn> {
        self.validators.insert(route, validate)
    }

    pub fn validator_for(&self, route: &R) -> Option<ValidateSearchFn> {
        self.validators.get(route).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchMap, SearchValidationError};
    use crate::{QueryPair, RouteLocation};

    #[test]
    fn search_map_canonicalizes_pairs() {
        let map = SearchMap::from_pairs(vec![
            QueryPair {
                key: "b".to_string(),
                value: Some("2".to_string()),
            },
            QueryPair {
                key: "a".to_string(),
                value: Some("1".to_string()),
            },
        ]);

        assert_eq!(map.pairs()[0].key, "a");
        assert_eq!(map.pairs()[1].key, "b");
    }

    #[test]
    fn search_map_values_keep_duplicates_in_canonical_order() {
        let location = RouteLocation::parse("/?tag=rust&tag=wasm&tag");
        let map = SearchMap::from_location(&location);
        assert_eq!(map.values("tag"), vec![None, Some("rust"), Some("wasm")]);
    }

    #[test]
    fn search_validation_error_is_displayable() {
        let err = SearchValidationError::new("bad search");
        assert_eq!(err.to_string(), "bad search");
    }
}
