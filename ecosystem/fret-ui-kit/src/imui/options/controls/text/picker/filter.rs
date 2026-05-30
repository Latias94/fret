#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextPickerFilter {
    #[default]
    ContainsCaseInsensitive,
    PrefixCaseInsensitive,
    None,
}

impl InputTextPickerFilter {
    pub fn matches(self, query: &str, candidate: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        match self {
            Self::None => true,
            Self::PrefixCaseInsensitive => candidate
                .to_lowercase()
                .starts_with(query.to_lowercase().as_str()),
            Self::ContainsCaseInsensitive => candidate
                .to_lowercase()
                .contains(query.to_lowercase().as_str()),
        }
    }
}
