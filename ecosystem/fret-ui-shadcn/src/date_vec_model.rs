use fret_runtime::Model;
use time::Date;

/// Narrow interop bridge for multi-select calendar widgets that store their value in a
/// `Model<Vec<Date>>`.
pub trait IntoDateVecModel {
    fn into_date_vec_model(self) -> Model<Vec<Date>>;
}

impl IntoDateVecModel for Model<Vec<Date>> {
    fn into_date_vec_model(self) -> Model<Vec<Date>> {
        self
    }
}

impl IntoDateVecModel for &Model<Vec<Date>> {
    fn into_date_vec_model(self) -> Model<Vec<Date>> {
        self.clone()
    }
}
