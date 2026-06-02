//! Small, reusable numeric formatting helpers for editor controls.
//!
//! These helpers live in the policy layer (`fret-ui-editor`) so we can iterate on "editor feel"
//! without expanding the `fret-ui` mechanism surface.

use std::sync::Arc;

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn};
use crate::primitives::drag_value_core::DragValueScalar;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NumericTextAffixes {
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
}

impl NumericTextAffixes {
    pub fn new(prefix: Option<Arc<str>>, suffix: Option<Arc<str>>) -> Self {
        Self { prefix, suffix }
    }

    pub fn prefix(prefix: impl Into<Arc<str>>) -> Self {
        Self {
            prefix: Some(prefix.into()),
            suffix: None,
        }
    }

    pub fn suffix(suffix: impl Into<Arc<str>>) -> Self {
        Self {
            prefix: None,
            suffix: Some(suffix.into()),
        }
    }
}

#[derive(Clone)]
pub struct NumericPresentation<T> {
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    chrome_affixes: NumericTextAffixes,
}

impl<T> NumericPresentation<T> {
    pub fn new(format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            format,
            parse,
            chrome_affixes: NumericTextAffixes::default(),
        }
    }

    pub fn with_chrome_affixes(mut self, chrome_affixes: NumericTextAffixes) -> Self {
        self.chrome_affixes = chrome_affixes;
        self
    }

    pub fn with_chrome_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.chrome_affixes.prefix = Some(prefix.into());
        self
    }

    pub fn with_chrome_suffix(mut self, suffix: impl Into<Arc<str>>) -> Self {
        self.chrome_affixes.suffix = Some(suffix.into());
        self
    }

    pub fn format(&self) -> NumericFormatFn<T> {
        self.format.clone()
    }

    pub fn parse(&self) -> NumericParseFn<T> {
        self.parse.clone()
    }

    pub fn parts(&self) -> (NumericFormatFn<T>, NumericParseFn<T>, NumericTextAffixes) {
        (self.format(), self.parse(), self.cloned_chrome_affixes())
    }

    pub fn chrome_affixes(&self) -> &NumericTextAffixes {
        &self.chrome_affixes
    }

    pub fn cloned_chrome_affixes(&self) -> NumericTextAffixes {
        self.chrome_affixes.clone()
    }

    pub fn chrome_prefix(&self) -> Option<&Arc<str>> {
        self.chrome_affixes.prefix.as_ref()
    }

    pub fn chrome_suffix(&self) -> Option<&Arc<str>> {
        self.chrome_affixes.suffix.as_ref()
    }
}

impl<T> NumericPresentation<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    pub fn fixed_decimals(decimals: usize) -> Self {
        Self::new(fixed_decimals_format(decimals), plain_number_parse())
    }

    pub fn percent_0_1(decimals: usize) -> Self {
        Self::new(percent_0_1_format(decimals), percent_0_1_parse())
    }

    pub fn degrees(decimals: usize) -> Self {
        Self::new(degrees_format(decimals), degrees_parse())
    }
}

pub fn fixed_decimals_format<T>(decimals: usize) -> NumericFormatFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    let decimals = decimals.min(6);
    Arc::new(move |v| Arc::from(format!("{:.*}", decimals, v.to_f64())))
}

pub fn plain_number_parse<T>() -> NumericParseFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    Arc::new(|s| s.trim().parse::<f64>().ok().map(T::from_f64))
}

pub fn affixed_number_format<T>(
    base: NumericFormatFn<T>,
    affixes: NumericTextAffixes,
) -> NumericFormatFn<T>
where
    T: Send + Sync + 'static,
{
    Arc::new(move |value| {
        let base_text = base(value);
        if affixes.prefix.is_none() && affixes.suffix.is_none() {
            return base_text;
        }

        let mut out = String::new();
        if let Some(prefix) = affixes.prefix.as_ref() {
            out.push_str(prefix.as_ref());
        }
        out.push_str(base_text.as_ref());
        if let Some(suffix) = affixes.suffix.as_ref() {
            out.push_str(suffix.as_ref());
        }
        Arc::from(out)
    })
}

pub fn affixed_number_parse<T>(
    base: NumericParseFn<T>,
    affixes: NumericTextAffixes,
) -> NumericParseFn<T>
where
    T: Send + Sync + 'static,
{
    Arc::new(move |text| {
        let mut text = text.trim();

        if let Some(prefix) = affixes.prefix.as_ref()
            && let Some(stripped) = text.strip_prefix(prefix.as_ref())
        {
            text = stripped.trim_start();
        }

        if let Some(suffix) = affixes.suffix.as_ref()
            && let Some(stripped) = text.strip_suffix(suffix.as_ref())
        {
            text = stripped.trim_end();
        }

        base(text)
    })
}

pub(crate) fn suppress_duplicate_chrome_affixes(
    text: &str,
    prefix: Option<Arc<str>>,
    suffix: Option<Arc<str>>,
) -> (Option<Arc<str>>, Option<Arc<str>>) {
    let prefix = prefix.filter(|prefix| !text.trim_start().starts_with(prefix.as_ref()));
    let suffix = suffix.filter(|suffix| !text.trim_end().ends_with(suffix.as_ref()));
    (prefix, suffix)
}

pub fn percent_0_1_format<T>(decimals: usize) -> NumericFormatFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    let decimals = decimals.min(6);
    Arc::new(move |v| {
        let pct = v.to_f64() * 100.0;
        if decimals == 0 {
            Arc::from(format!("{:.0}%", pct))
        } else {
            Arc::from(format!("{:.*}%", decimals, pct))
        }
    })
}

pub fn percent_0_1_parse<T>() -> NumericParseFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    Arc::new(|s| {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if let Some(raw) = s.strip_suffix('%') {
            let v = raw.trim().parse::<f64>().ok()?;
            return Some(T::from_f64(v / 100.0));
        }

        let v = s.parse::<f64>().ok()?;

        // Heuristic: when editing a normalized 0..1 value that is *displayed* as percent, users
        // tend to type `50` (meaning 50%), not `0.5`. Treat values outside [0, 1] as percent.
        if v.abs() > 1.0 {
            return Some(T::from_f64(v / 100.0));
        }

        Some(T::from_f64(v))
    })
}

pub fn degrees_format<T>(decimals: usize) -> NumericFormatFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    affixed_number_format(
        fixed_decimals_format(decimals),
        NumericTextAffixes::suffix("°"),
    )
}

pub fn degrees_parse<T>() -> NumericParseFn<T>
where
    T: DragValueScalar + Send + Sync + 'static,
{
    affixed_number_parse(plain_number_parse(), NumericTextAffixes::suffix("°"))
}
