use crate::engine::model::ChartModel;
use crate::ids::{AxisId, FieldId, SeriesId};
use crate::scale::AxisScale;
use crate::spec::SeriesKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarMapping {
    pub orientation: BarOrientation,
    pub category_axis: AxisId,
    pub value_axis: AxisId,
    pub category_field: FieldId,
    pub value_field: FieldId,
}

pub fn bar_mapping_for_series(model: &ChartModel, series_id: SeriesId) -> Option<BarMapping> {
    let series = model.series.get(&series_id)?;
    if series.kind != SeriesKind::Bar {
        return None;
    }

    let x_axis = model.axes.get(&series.x_axis)?;
    let y_axis = model.axes.get(&series.y_axis)?;

    let x_is_category = matches!(x_axis.scale, AxisScale::Category(_));
    let y_is_category = matches!(y_axis.scale, AxisScale::Category(_));

    match (x_is_category, y_is_category) {
        (true, false) => Some(BarMapping {
            orientation: BarOrientation::Vertical,
            category_axis: series.x_axis,
            value_axis: series.y_axis,
            category_field: series.encode.x,
            value_field: series.encode.y,
        }),
        (false, true) => Some(BarMapping {
            orientation: BarOrientation::Horizontal,
            category_axis: series.y_axis,
            value_axis: series.x_axis,
            category_field: series.encode.y,
            value_field: series.encode.x,
        }),
        _ => None,
    }
}
