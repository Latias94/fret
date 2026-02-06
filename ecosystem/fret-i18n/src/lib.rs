use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LocaleId(LanguageIdentifier);

impl LocaleId {
    pub fn parse(input: &str) -> Result<Self, LocaleParseError> {
        let id: LanguageIdentifier = input.parse().map_err(LocaleParseError)?;
        Ok(Self(id))
    }

    pub fn as_langid(&self) -> &LanguageIdentifier {
        &self.0
    }

    pub fn en_us() -> Self {
        Self::parse("en-US").expect("hardcoded locale en-US must be valid")
    }
}

impl fmt::Display for LocaleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<LanguageIdentifier> for LocaleId {
    fn from(value: LanguageIdentifier) -> Self {
        Self(value)
    }
}

impl From<LocaleId> for LanguageIdentifier {
    fn from(value: LocaleId) -> Self {
        value.0
    }
}

impl std::str::FromStr for LocaleId {
    type Err = LocaleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Default for LocaleId {
    fn default() -> Self {
        Self::en_us()
    }
}

#[derive(Debug)]
pub struct LocaleParseError(pub unic_langid::LanguageIdentifierError);

impl fmt::Display for LocaleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid locale: {}", self.0)
    }
}

impl std::error::Error for LocaleParseError {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MessageKey(Cow<'static, str>);

impl MessageKey {
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&'static str> for MessageKey {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for MessageKey {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl fmt::Display for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageArgValue {
    String(String),
    Number(f64),
    Integer(i64),
    Unsigned(u64),
    Bool(bool),
}

impl From<String> for MessageArgValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MessageArgValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for MessageArgValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for MessageArgValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<u64> for MessageArgValue {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}

impl From<f64> for MessageArgValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageArgs {
    items: BTreeMap<String, MessageArgValue>,
}

impl MessageArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<MessageArgValue>) -> Self {
        self.insert(key, value);
        self
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<MessageArgValue>) {
        self.items.insert(key.into(), value.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &MessageArgValue)> {
        self.items.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct LocalizedMessage {
    pub text: String,
    pub locale: LocaleId,
    pub fallback_depth: usize,
}

#[derive(Debug, Clone)]
pub enum I18nLookupError {
    MissingBackend,
    MissingKey { key: MessageKey },
    MissingLocale { requested: Vec<LocaleId> },
    Backend { message: String },
}

impl fmt::Display for I18nLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBackend => write!(f, "i18n backend is not configured"),
            Self::MissingKey { key } => write!(f, "missing message key: {key}"),
            Self::MissingLocale { requested } => {
                let chain = requested
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "missing locale for chain: [{chain}]")
            }
            Self::Backend { message } => write!(f, "i18n backend error: {message}"),
        }
    }
}

impl std::error::Error for I18nLookupError {}

pub trait I18nLookup {
    fn format(
        &self,
        preferred_locales: &[LocaleId],
        key: &MessageKey,
        args: Option<&MessageArgs>,
    ) -> Result<LocalizedMessage, I18nLookupError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MissingMessageBehavior {
    #[default]
    ReturnKey,
    ReturnBracketedKey,
}

#[derive(Clone)]
pub struct I18nService {
    preferred_locales: Vec<LocaleId>,
    lookup: Option<Arc<dyn I18nLookup + 'static>>,
    missing_message_behavior: MissingMessageBehavior,
    pseudo_enabled: bool,
}

impl fmt::Debug for I18nService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("I18nService")
            .field("preferred_locales", &self.preferred_locales)
            .field("has_lookup", &self.lookup.is_some())
            .field("missing_message_behavior", &self.missing_message_behavior)
            .field("pseudo_enabled", &self.pseudo_enabled)
            .finish()
    }
}

impl Default for I18nService {
    fn default() -> Self {
        Self {
            preferred_locales: vec![LocaleId::default()],
            lookup: None,
            missing_message_behavior: MissingMessageBehavior::default(),
            pseudo_enabled: false,
        }
    }
}

impl I18nService {
    pub fn new(preferred_locales: Vec<LocaleId>) -> Self {
        let mut out = Self::default();
        out.set_preferred_locales(preferred_locales);
        out
    }

    pub fn with_lookup(mut self, lookup: Arc<dyn I18nLookup + 'static>) -> Self {
        self.lookup = Some(lookup);
        self
    }

    pub fn set_lookup(&mut self, lookup: Option<Arc<dyn I18nLookup + 'static>>) {
        self.lookup = lookup;
    }

    pub fn lookup(&self) -> Option<&Arc<dyn I18nLookup + 'static>> {
        self.lookup.as_ref()
    }

    pub fn set_preferred_locales(&mut self, preferred_locales: Vec<LocaleId>) {
        let mut dedup = Vec::new();
        for locale in preferred_locales {
            if !dedup.contains(&locale) {
                dedup.push(locale);
            }
        }
        if dedup.is_empty() {
            dedup.push(LocaleId::default());
        }
        self.preferred_locales = dedup;
    }

    pub fn preferred_locales(&self) -> &[LocaleId] {
        &self.preferred_locales
    }

    pub fn set_missing_message_behavior(&mut self, behavior: MissingMessageBehavior) {
        self.missing_message_behavior = behavior;
    }

    pub fn missing_message_behavior(&self) -> MissingMessageBehavior {
        self.missing_message_behavior
    }

    pub fn set_pseudo_enabled(&mut self, enabled: bool) {
        self.pseudo_enabled = enabled;
    }

    pub fn pseudo_enabled(&self) -> bool {
        self.pseudo_enabled
    }

    pub fn format(
        &self,
        key: &MessageKey,
        args: Option<&MessageArgs>,
    ) -> Result<LocalizedMessage, I18nLookupError> {
        let lookup = self
            .lookup
            .as_ref()
            .ok_or(I18nLookupError::MissingBackend)?;
        let mut localized = lookup.format(&self.preferred_locales, key, args)?;
        if self.pseudo_enabled {
            localized.text = pseudo_localize(&localized.text);
        }
        Ok(localized)
    }

    pub fn t(&self, key: impl Into<MessageKey>) -> String {
        let key = key.into();
        match self.format(&key, None) {
            Ok(message) => message.text,
            Err(_) => self.missing_message(key.as_str()),
        }
    }

    pub fn t_args(&self, key: impl Into<MessageKey>, args: &MessageArgs) -> String {
        let key = key.into();
        match self.format(&key, Some(args)) {
            Ok(message) => message.text,
            Err(_) => self.missing_message(key.as_str()),
        }
    }

    fn missing_message(&self, key: &str) -> String {
        match self.missing_message_behavior {
            MissingMessageBehavior::ReturnKey => key.to_string(),
            MissingMessageBehavior::ReturnBracketedKey => format!("[missing:{key}]"),
        }
    }
}

fn pseudo_localize(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('⟪');
    for ch in input.chars() {
        let mapped = match ch {
            'a' => 'á',
            'e' => 'é',
            'i' => 'í',
            'o' => 'ó',
            'u' => 'ú',
            'A' => 'Á',
            'E' => 'É',
            'I' => 'Í',
            'O' => 'Ó',
            'U' => 'Ú',
            _ => ch,
        };
        out.push(mapped);
    }
    out.push('⟫');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    struct StaticLookup {
        locale: LocaleId,
        value: &'static str,
    }

    impl I18nLookup for StaticLookup {
        fn format(
            &self,
            preferred_locales: &[LocaleId],
            _key: &MessageKey,
            _args: Option<&MessageArgs>,
        ) -> Result<LocalizedMessage, I18nLookupError> {
            if preferred_locales.contains(&self.locale) {
                Ok(LocalizedMessage {
                    text: self.value.to_string(),
                    locale: self.locale.clone(),
                    fallback_depth: 0,
                })
            } else {
                Err(I18nLookupError::MissingLocale {
                    requested: preferred_locales.to_vec(),
                })
            }
        }
    }

    #[test]
    fn message_args_roundtrip_values() {
        let args = MessageArgs::new()
            .with("name", "fret")
            .with("count", 3_u64)
            .with("ratio", 1.5_f64)
            .with("ok", true);

        assert!(!args.is_empty());

        let collected = args
            .iter()
            .map(|(k, v)| (k.to_string(), format!("{v:?}")))
            .collect::<Vec<_>>();

        assert_eq!(collected.len(), 4);
        assert!(collected.iter().any(|(k, _)| k == "name"));
        assert!(collected.iter().any(|(k, _)| k == "count"));
        assert!(collected.iter().any(|(k, _)| k == "ratio"));
        assert!(collected.iter().any(|(k, _)| k == "ok"));
    }

    #[test]
    fn locale_id_parses_bcp47() {
        let locale = LocaleId::parse("zh-CN").expect("must parse");
        assert_eq!(locale.to_string(), "zh-CN");
    }

    #[test]
    fn i18n_service_defaults_to_en_us_locale() {
        let service = I18nService::default();
        assert_eq!(service.preferred_locales(), &[LocaleId::en_us()]);
    }

    #[test]
    fn i18n_service_without_backend_falls_back_to_key() {
        let service = I18nService::default();
        assert_eq!(service.t("app.title"), "app.title");
    }

    #[test]
    fn i18n_service_supports_pseudo_mode() {
        let locale = LocaleId::parse("en-US").expect("locale must parse");
        let lookup = StaticLookup {
            locale: locale.clone(),
            value: "Open",
        };

        let mut service = I18nService::new(vec![locale]);
        service.set_lookup(Some(Arc::new(lookup)));

        assert_eq!(service.t("menu.file.open"), "Open");

        service.set_pseudo_enabled(true);
        assert_eq!(service.t("menu.file.open"), "⟪Ópén⟫");
    }

    #[test]
    fn i18n_service_dedupes_preferred_locales() {
        let locale = LocaleId::parse("en-US").expect("locale must parse");
        let mut service = I18nService::default();
        service.set_preferred_locales(vec![locale.clone(), locale.clone()]);
        assert_eq!(service.preferred_locales(), &[locale]);
    }
}
