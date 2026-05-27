use std::sync::Arc;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InputTextFilters {
    pub decimal: bool,
    pub hexadecimal: bool,
    pub scientific: bool,
    pub uppercase: bool,
    pub no_blank: bool,
}

impl InputTextFilters {
    pub const fn none() -> Self {
        Self {
            decimal: false,
            hexadecimal: false,
            scientific: false,
            uppercase: false,
            no_blank: false,
        }
    }

    pub const fn decimal() -> Self {
        Self {
            decimal: true,
            ..Self::none()
        }
    }

    pub const fn hexadecimal() -> Self {
        Self {
            hexadecimal: true,
            ..Self::none()
        }
    }

    pub const fn scientific() -> Self {
        Self {
            scientific: true,
            ..Self::none()
        }
    }

    pub const fn uppercase() -> Self {
        Self {
            uppercase: true,
            ..Self::none()
        }
    }

    pub const fn no_blank() -> Self {
        Self {
            no_blank: true,
            ..Self::none()
        }
    }

    pub const fn with_decimal(mut self) -> Self {
        self.decimal = true;
        self
    }

    pub const fn with_hexadecimal(mut self) -> Self {
        self.hexadecimal = true;
        self
    }

    pub const fn with_scientific(mut self) -> Self {
        self.scientific = true;
        self
    }

    pub const fn with_uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub const fn with_no_blank(mut self) -> Self {
        self.no_blank = true;
        self
    }

    pub const fn is_empty(self) -> bool {
        !self.decimal && !self.hexadecimal && !self.scientific && !self.uppercase && !self.no_blank
    }

    pub fn filter_text(self, text: &str) -> String {
        if self.is_empty() {
            return text.to_string();
        }

        text.chars().filter_map(|c| self.filter_char(c)).collect()
    }

    fn filter_char(self, mut c: char) -> Option<char> {
        if self.decimal && !is_decimal_input_char(c) {
            return None;
        }
        if self.scientific && !is_scientific_input_char(c) {
            return None;
        }
        if self.hexadecimal && !c.is_ascii_hexdigit() {
            return None;
        }
        if self.uppercase && c.is_ascii_lowercase() {
            c = c.to_ascii_uppercase();
        }
        if self.no_blank && matches!(c, ' ' | '\t') {
            return None;
        }
        Some(c)
    }
}

fn is_decimal_input_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | '*' | '/')
}

fn is_scientific_input_char(c: char) -> bool {
    is_decimal_input_char(c) || matches!(c, 'e' | 'E')
}

#[derive(Clone)]
pub struct InputTextCustomFilter {
    filter: Arc<dyn Fn(&str) -> String + 'static>,
}

impl InputTextCustomFilter {
    pub fn new(filter: impl Fn(&str) -> String + 'static) -> Self {
        Self {
            filter: Arc::new(filter),
        }
    }

    pub fn filter_text(&self, text: &str) -> String {
        (self.filter)(text)
    }
}

impl std::fmt::Debug for InputTextCustomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputTextCustomFilter")
            .finish_non_exhaustive()
    }
}
