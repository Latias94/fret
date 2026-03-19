use fret_runtime::Model;
use time::Date;

/// Narrow interop bridge for single-date widgets that store their value in a
/// `Model<Option<Date>>`.
pub trait IntoOptionalDateModel {
    fn into_optional_date_model(self) -> Model<Option<Date>>;
}

impl IntoOptionalDateModel for Model<Option<Date>> {
    fn into_optional_date_model(self) -> Model<Option<Date>> {
        self
    }
}

impl IntoOptionalDateModel for &Model<Option<Date>> {
    fn into_optional_date_model(self) -> Model<Option<Date>> {
        self.clone()
    }
}
