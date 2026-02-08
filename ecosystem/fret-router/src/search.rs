use crate::{QueryPair, RouteLocation, canonicalize_query_pairs};

pub trait SearchValue: Sized {
    fn parse(value: &str) -> Option<Self>;
    fn format(&self) -> String;
}

impl SearchValue for String {
    fn parse(value: &str) -> Option<Self> {
        Some(value.to_string())
    }

    fn format(&self) -> String {
        self.clone()
    }
}

impl SearchValue for std::sync::Arc<str> {
    fn parse(value: &str) -> Option<Self> {
        Some(Self::from(value))
    }

    fn format(&self) -> String {
        self.to_string()
    }
}

impl SearchValue for bool {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "1" => Some(true),
            "0" => Some(false),
            other => match other.to_ascii_lowercase().as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
        }
    }

    fn format(&self) -> String {
        if *self { "true" } else { "false" }.to_string()
    }
}

macro_rules! impl_search_value_parse_via_from_str {
    ($ty:ty) => {
        impl SearchValue for $ty {
            fn parse(value: &str) -> Option<Self> {
                value.parse::<Self>().ok()
            }

            fn format(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_search_value_parse_via_from_str!(u8);
impl_search_value_parse_via_from_str!(u16);
impl_search_value_parse_via_from_str!(u32);
impl_search_value_parse_via_from_str!(u64);
impl_search_value_parse_via_from_str!(usize);
impl_search_value_parse_via_from_str!(i8);
impl_search_value_parse_via_from_str!(i16);
impl_search_value_parse_via_from_str!(i32);
impl_search_value_parse_via_from_str!(i64);
impl_search_value_parse_via_from_str!(isize);
impl_search_value_parse_via_from_str!(f32);
impl_search_value_parse_via_from_str!(f64);

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

    pub fn contains_key(&self, key: &str) -> bool {
        self.pairs.iter().any(|pair| pair.key == key)
    }

    pub fn first(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|pair| pair.key == key)
            .and_then(|pair| pair.value.as_deref())
    }

    pub fn first_typed<T>(&self, key: &str) -> Option<T>
    where
        T: SearchValue,
    {
        T::parse(self.first(key)?)
    }

    pub fn values(&self, key: &str) -> Vec<Option<&str>> {
        self.pairs
            .iter()
            .filter(|pair| pair.key == key)
            .map(|pair| pair.value.as_deref())
            .collect()
    }

    pub fn values_typed<T>(&self, key: &str) -> Vec<Option<T>>
    where
        T: SearchValue,
    {
        self.pairs
            .iter()
            .filter(|pair| pair.key == key)
            .map(|pair| pair.value.as_deref().and_then(T::parse))
            .collect()
    }

    pub fn push(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        self.pairs.push(QueryPair {
            key: key.into(),
            value,
        });
        canonicalize_query_pairs(&mut self.pairs);
        self
    }

    pub fn push_typed<T>(self, key: impl Into<String>, value: Option<T>) -> Self
    where
        T: SearchValue,
    {
        self.push(key, value.map(|v| v.format()))
    }

    pub fn with_typed<T>(self, key: impl Into<String>, value: Option<T>) -> Self
    where
        T: SearchValue,
    {
        self.with(key, value.map(|v| v.format()))
    }

    pub fn with_flag(self, key: impl Into<String>, present: bool) -> Self {
        let key = key.into();
        let mut out = self;
        out.pairs.retain(|pair| pair.key != key);
        if present {
            out.pairs.push(QueryPair { key, value: None });
        }
        canonicalize_query_pairs(&mut out.pairs);
        out
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
    use super::{SearchMap, SearchValidationError, SearchValue};
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
    fn search_map_first_typed_parses_values() {
        let location = RouteLocation::parse("/?count=12&debug=true&pi=3.14");
        let map = SearchMap::from_location(&location);
        assert_eq!(map.first_typed::<u32>("count"), Some(12));
        assert_eq!(map.first_typed::<bool>("debug"), Some(true));
        assert_eq!(map.first_typed::<f64>("pi"), Some(3.14));
    }

    #[test]
    fn search_map_values_typed_preserves_duplicates_and_maps_missing_values_to_none() {
        let location = RouteLocation::parse("/?tag=rust&tag&tag=wasm");
        let map = SearchMap::from_location(&location);
        assert_eq!(
            map.values_typed::<String>("tag"),
            vec![None, Some("rust".to_string()), Some("wasm".to_string())]
        );
    }

    #[test]
    fn search_value_bool_is_lenient_to_common_forms() {
        assert_eq!(<bool as SearchValue>::parse("true"), Some(true));
        assert_eq!(<bool as SearchValue>::parse("FALSE"), Some(false));
        assert_eq!(<bool as SearchValue>::parse("1"), Some(true));
        assert_eq!(<bool as SearchValue>::parse("0"), Some(false));
        assert_eq!(<bool as SearchValue>::parse("yes"), None);
    }

    #[test]
    fn search_validation_error_is_displayable() {
        let err = SearchValidationError::new("bad search");
        assert_eq!(err.to_string(), "bad search");
    }
}
