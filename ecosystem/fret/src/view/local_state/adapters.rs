#[cfg(feature = "shadcn")]
use std::sync::Arc;

use fret_runtime::Model;

use super::{LocalState, LocalStateRawModelExt};

impl<T> fret_ui_kit::declarative::form::IntoFormValueModel<T> for LocalState<T> {
    fn into_form_value_model(self) -> Model<T> {
        self.clone_model()
    }
}

impl<T> fret_ui_kit::declarative::form::IntoFormValueModel<T> for &LocalState<T> {
    fn into_form_value_model(self) -> Model<T> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::form::IntoFormStateModel
    for LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::form::IntoFormStateModel
    for &LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableStateModel
    for LocalState<fret_ui_kit::headless::table::TableState>
{
    fn into_table_state_model(self) -> Model<fret_ui_kit::headless::table::TableState> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableStateModel
    for &LocalState<fret_ui_kit::headless::table::TableState>
{
    fn into_table_state_model(self) -> Model<fret_ui_kit::headless::table::TableState> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableViewOutputModel
    for LocalState<fret_ui_kit::declarative::table::TableViewOutput>
{
    fn into_table_view_output_model(
        self,
    ) -> Model<fret_ui_kit::declarative::table::TableViewOutput> {
        self.clone_model()
    }
}

impl fret_ui_kit::declarative::table::IntoTableViewOutputModel
    for &LocalState<fret_ui_kit::declarative::table::TableViewOutput>
{
    fn into_table_view_output_model(
        self,
    ) -> Model<fret_ui_kit::declarative::table::TableViewOutput> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoBoolModel for LocalState<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoBoolModel for &LocalState<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalBoolModel for LocalState<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalBoolModel for &LocalState<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFormStateModel
    for LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFormStateModel
    for &LocalState<fret_ui_kit::headless::form_state::FormState>
{
    fn into_form_state_model(self) -> Model<fret_ui_kit::headless::form_state::FormState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCheckedStateModel
    for LocalState<fret_ui_headless::checked_state::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_headless::checked_state::CheckedState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCheckedStateModel
    for &LocalState<fret_ui_headless::checked_state::CheckedState>
{
    fn into_checked_state_model(self) -> Model<fret_ui_headless::checked_state::CheckedState> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextValueModel for LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextValueModel for &LocalState<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalTextValueModel for LocalState<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalTextValueModel for &LocalState<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextVecModel for LocalState<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoTextVecModel for &LocalState<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatValueModel for LocalState<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatValueModel for &LocalState<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalFloatValueModel for LocalState<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalFloatValueModel for &LocalState<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatVecModel for LocalState<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoFloatVecModel for &LocalState<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCalendarMonthModel
    for LocalState<fret_ui_kit::headless::calendar::CalendarMonth>
{
    fn into_calendar_month_model(self) -> Model<fret_ui_kit::headless::calendar::CalendarMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoCalendarMonthModel
    for &LocalState<fret_ui_kit::headless::calendar::CalendarMonth>
{
    fn into_calendar_month_model(self) -> Model<fret_ui_kit::headless::calendar::CalendarMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalDateModel for LocalState<Option<time::Date>> {
    fn into_optional_date_model(self) -> Model<Option<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoOptionalDateModel for &LocalState<Option<time::Date>> {
    fn into_optional_date_model(self) -> Model<Option<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoSolarHijriMonthModel
    for LocalState<fret_ui_shadcn::facade::SolarHijriMonth>
{
    fn into_solar_hijri_month_model(self) -> Model<fret_ui_shadcn::facade::SolarHijriMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoSolarHijriMonthModel
    for &LocalState<fret_ui_shadcn::facade::SolarHijriMonth>
{
    fn into_solar_hijri_month_model(self) -> Model<fret_ui_shadcn::facade::SolarHijriMonth> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoU8ValueModel for LocalState<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoU8ValueModel for &LocalState<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateRangeSelectionModel
    for LocalState<fret_ui_kit::headless::calendar::DateRangeSelection>
{
    fn into_date_range_selection_model(
        self,
    ) -> Model<fret_ui_kit::headless::calendar::DateRangeSelection> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateRangeSelectionModel
    for &LocalState<fret_ui_kit::headless::calendar::DateRangeSelection>
{
    fn into_date_range_selection_model(
        self,
    ) -> Model<fret_ui_kit::headless::calendar::DateRangeSelection> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateVecModel for LocalState<Vec<time::Date>> {
    fn into_date_vec_model(self) -> Model<Vec<time::Date>> {
        self.clone_model()
    }
}

#[cfg(feature = "shadcn")]
impl fret_ui_shadcn::facade::IntoDateVecModel for &LocalState<Vec<time::Date>> {
    fn into_date_vec_model(self) -> Model<Vec<time::Date>> {
        self.clone_model()
    }
}
