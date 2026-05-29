impl super::InputTextFilters {
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
