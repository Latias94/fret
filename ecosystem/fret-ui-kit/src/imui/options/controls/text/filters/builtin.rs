mod filtering;

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
}
