use fret_runtime::Model;
use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;

/// Narrow interop bridge for Hijri calendar widgets that store their visible month in a
/// `Model<SolarHijriMonth>`.
pub trait IntoSolarHijriMonthModel {
    fn into_solar_hijri_month_model(self) -> Model<SolarHijriMonth>;
}

impl IntoSolarHijriMonthModel for Model<SolarHijriMonth> {
    fn into_solar_hijri_month_model(self) -> Model<SolarHijriMonth> {
        self
    }
}

impl IntoSolarHijriMonthModel for &Model<SolarHijriMonth> {
    fn into_solar_hijri_month_model(self) -> Model<SolarHijriMonth> {
        self.clone()
    }
}
