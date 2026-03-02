use std::sync::Arc;

/// Data model for a combobox option.
///
/// Note: this is intentionally separate from the shadcn/ui v4 part surface, which also exports a
/// `ComboboxItem` element part name. Keeping the data model distinct allows us to converge to the
/// upstream part naming without losing a structured option type.
#[derive(Debug, Clone)]
pub struct ComboboxOption {
    pub value: Arc<str>,
    pub label: Arc<str>,
    /// Optional structured detail for display, typically rendered as a suffix (e.g. `(React)`).
    ///
    /// This is primarily meant to support "object item" adapters without forcing callers to
    /// pre-format richer labels into a single string.
    pub detail: Option<Arc<str>>,
    pub disabled: bool,
    /// Additional strings that participate in cmdk-style filtering/ranking.
    ///
    /// This aligns with `CommandItem::keywords(...)` and cmdk's `keywords` prop.
    pub keywords: Vec<Arc<str>>,
}

impl ComboboxOption {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
            disabled: false,
            keywords: Vec::new(),
        }
    }

    /// Sets a structured detail suffix and appends it to the visible label as `(<detail>)`.
    ///
    /// This preserves the existing string-based label contract while letting callers keep the
    /// underlying data model structured.
    pub fn detail(mut self, detail: impl Into<Arc<str>>) -> Self {
        let detail = detail.into();
        self.detail = Some(detail.clone());
        self.keywords.push(detail.clone());
        self.label = Arc::<str>::from(format!("{} ({})", self.label, detail));
        self
    }

    /// Additional strings used for cmdk-style filtering/ranking.
    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ComboboxOptionGroup {
    pub heading: Arc<str>,
    pub items: Vec<ComboboxOption>,
}

impl ComboboxOptionGroup {
    pub fn new(
        heading: impl Into<Arc<str>>,
        items: impl IntoIterator<Item = ComboboxOption>,
    ) -> Self {
        Self {
            heading: heading.into(),
            items: items.into_iter().collect(),
        }
    }
}
