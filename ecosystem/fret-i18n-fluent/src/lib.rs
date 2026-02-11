use std::collections::HashMap;
use std::rc::Rc;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use fret_i18n::{
    I18nLookup, I18nLookupError, LocaleId, LocalizedMessage, MessageArgValue, MessageArgs,
    MessageKey,
};
use unic_langid::LanguageIdentifier;

#[derive(Default)]
pub struct FluentCatalog {
    bundles: HashMap<LocaleId, Rc<FluentBundle<FluentResource>>>,
}

impl FluentCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_locale_ftl(
        &mut self,
        locale: LocaleId,
        ftl: &str,
    ) -> Result<(), FluentCatalogError> {
        let resource = FluentResource::try_new(ftl.to_string())
            .map_err(|(_, errors)| FluentCatalogError::Parse { errors })?;

        let langid: LanguageIdentifier = locale.clone().into();
        let mut bundle = FluentBundle::new(vec![langid]);
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .map_err(|errors| FluentCatalogError::AddResource { errors })?;

        self.bundles.insert(locale, Rc::new(bundle));
        Ok(())
    }

    pub fn has_locale(&self, locale: &LocaleId) -> bool {
        self.bundles.contains_key(locale)
    }
}

#[derive(Debug)]
pub enum FluentCatalogError {
    Parse {
        errors: Vec<fluent_syntax::parser::ParserError>,
    },
    AddResource {
        errors: Vec<fluent_bundle::FluentError>,
    },
}

impl std::fmt::Display for FluentCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { errors } => {
                write!(f, "failed to parse fluent resource ({})", errors.len())
            }
            Self::AddResource { errors } => {
                write!(f, "failed to add fluent resource ({})", errors.len())
            }
        }
    }
}

impl std::error::Error for FluentCatalogError {}

pub struct FluentLookup {
    catalog: Rc<FluentCatalog>,
}

impl FluentLookup {
    pub fn new(catalog: Rc<FluentCatalog>) -> Self {
        Self { catalog }
    }
}

impl I18nLookup for FluentLookup {
    fn format(
        &self,
        preferred_locales: &[LocaleId],
        key: &MessageKey,
        args: Option<&MessageArgs>,
    ) -> Result<LocalizedMessage, I18nLookupError> {
        if preferred_locales.is_empty() {
            return Err(I18nLookupError::MissingLocale {
                requested: Vec::new(),
            });
        }

        for (depth, locale) in preferred_locales.iter().enumerate() {
            let Some(bundle) = self.catalog.bundles.get(locale) else {
                continue;
            };

            let Some(message) = bundle.get_message(key.as_str()) else {
                continue;
            };
            let Some(pattern) = message.value() else {
                continue;
            };

            let mut errors = Vec::new();
            let fluent_args = args.map(build_fluent_args);
            let rendered = bundle.format_pattern(pattern, fluent_args.as_ref(), &mut errors);
            if !errors.is_empty() {
                return Err(I18nLookupError::Backend {
                    message: format!(
                        "format pattern failed for key '{}' in locale '{}': {} errors",
                        key,
                        locale,
                        errors.len()
                    ),
                });
            }

            return Ok(LocalizedMessage {
                text: rendered.into_owned(),
                locale: locale.clone(),
                fallback_depth: depth,
            });
        }

        let has_any_locale = preferred_locales
            .iter()
            .any(|locale| self.catalog.has_locale(locale));

        if has_any_locale {
            Err(I18nLookupError::MissingKey { key: key.clone() })
        } else {
            Err(I18nLookupError::MissingLocale {
                requested: preferred_locales.to_vec(),
            })
        }
    }
}

fn build_fluent_args(args: &MessageArgs) -> FluentArgs<'_> {
    let mut fluent_args = FluentArgs::new();
    for (key, value) in args.iter() {
        let value = match value {
            MessageArgValue::String(v) => FluentValue::from(v.clone()),
            MessageArgValue::Number(v) => FluentValue::from(*v),
            MessageArgValue::Integer(v) => FluentValue::from(*v),
            MessageArgValue::Unsigned(v) => FluentValue::from(*v as i64),
            MessageArgValue::Bool(v) => FluentValue::from(v.to_string()),
        };
        fluent_args.set(key.to_string(), value);
    }
    fluent_args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(input: &str) -> LocaleId {
        LocaleId::parse(input).expect("locale should parse")
    }

    #[test]
    fn uses_first_matching_locale_and_formats_args() {
        let mut catalog = FluentCatalog::new();
        catalog
            .add_locale_ftl(
                locale("en-US"),
                "hello = Hello, { $name }!\nitems = You have { $count } items.",
            )
            .expect("catalog should load");
        catalog
            .add_locale_ftl(locale("zh-CN"), "hello = 你好，{ $name }！")
            .expect("catalog should load");

        let lookup = FluentLookup::new(Rc::new(catalog));

        let args = MessageArgs::new().with("name", "Fret");
        let message = lookup
            .format(
                &[locale("zh-CN"), locale("en-US")],
                &MessageKey::from("hello"),
                Some(&args),
            )
            .expect("message should format");

        assert_eq!(message.text, "你好，Fret！");
        assert_eq!(message.locale.to_string(), "zh-CN");
        assert_eq!(message.fallback_depth, 0);

        let args = MessageArgs::new().with("count", 3_u64);
        let message = lookup
            .format(
                &[locale("fr-FR"), locale("en-US")],
                &MessageKey::from("items"),
                Some(&args),
            )
            .expect("fallback message should format");

        assert_eq!(message.text, "You have 3 items.");
        assert_eq!(message.locale.to_string(), "en-US");
        assert_eq!(message.fallback_depth, 1);
    }

    #[test]
    fn missing_locale_and_key_are_distinguished() {
        let mut catalog = FluentCatalog::new();
        catalog
            .add_locale_ftl(locale("en-US"), "hello = Hello")
            .expect("catalog should load");
        let lookup = FluentLookup::new(Rc::new(catalog));

        let no_locale = lookup.format(&[locale("fr-FR")], &MessageKey::from("hello"), None);
        assert!(matches!(
            no_locale,
            Err(I18nLookupError::MissingLocale { .. })
        ));

        let missing_key = lookup.format(&[locale("en-US")], &MessageKey::from("not_found"), None);
        assert!(matches!(
            missing_key,
            Err(I18nLookupError::MissingKey { .. })
        ));
    }
}
