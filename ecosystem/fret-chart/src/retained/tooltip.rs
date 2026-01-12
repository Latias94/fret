use std::collections::BTreeMap;

use delinea::engine::window::DataWindow;
use delinea::engine::{AxisPointerOutput, model::ChartModel};
use delinea::{AxisId, ChartEngine, SeriesId};

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipTextLine {
    pub source_series: Option<SeriesId>,
    pub text: String,
}

pub trait TooltipFormatter: Send + Sync {
    fn format_axis_pointer(
        &self,
        engine: &ChartEngine,
        axis_windows: &BTreeMap<AxisId, DataWindow>,
        axis_pointer: &AxisPointerOutput,
    ) -> Vec<TooltipTextLine>;
}

#[derive(Debug, Default)]
pub struct DefaultTooltipFormatter;

impl DefaultTooltipFormatter {
    fn apply_line_template(template: &str, label: &str, value: &str) -> String {
        template.replace("{label}", label).replace("{value}", value)
    }

    fn apply_range_template(template: &str, min: &str, max: &str) -> String {
        template.replace("{min}", min).replace("{max}", max)
    }

    fn format_value_value_axis_decimals(
        value: f64,
        decimals: u8,
        trim_trailing_zeros: bool,
    ) -> String {
        if !value.is_finite() {
            return value.to_string();
        }

        let mut out = format!("{:.*}", decimals as usize, value);
        if !trim_trailing_zeros {
            return out;
        }

        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
        if out.is_empty() { "0".to_string() } else { out }
    }

    fn format_value_for_tooltip(
        model: &ChartModel,
        axis: AxisId,
        window: DataWindow,
        value: f64,
        spec: &delinea::TooltipSpecV1,
    ) -> String {
        let Some(axis_model) = model.axes.get(&axis) else {
            return delinea::engine::axis::format_value_for(model, axis, window, value);
        };

        match &axis_model.scale {
            delinea::AxisScale::Value(_) if spec.value_decimals.is_some() => {
                Self::format_value_value_axis_decimals(
                    value,
                    spec.value_decimals.unwrap_or(0),
                    spec.trim_trailing_zeros,
                )
            }
            _ => delinea::engine::axis::format_value_for(model, axis, window, value),
        }
    }

    fn series_override<'a>(
        spec: &'a delinea::TooltipSpecV1,
        series: SeriesId,
    ) -> Option<&'a delinea::TooltipSeriesOverrideV1> {
        spec.series_overrides.iter().find(|o| o.series == series)
    }

    fn effective_series_line_template<'a>(
        spec: &'a delinea::TooltipSpecV1,
        override_spec: Option<&'a delinea::TooltipSeriesOverrideV1>,
    ) -> &'a str {
        override_spec
            .and_then(|o| o.series_line_template.as_deref())
            .unwrap_or(spec.series_line_template.as_str())
    }

    fn effective_missing_value<'a>(
        spec: &'a delinea::TooltipSpecV1,
        override_spec: Option<&'a delinea::TooltipSeriesOverrideV1>,
    ) -> &'a str {
        override_spec
            .and_then(|o| o.missing_value.as_deref())
            .unwrap_or(spec.missing_value.as_str())
    }

    fn effective_range_template<'a>(
        spec: &'a delinea::TooltipSpecV1,
        override_spec: Option<&'a delinea::TooltipSeriesOverrideV1>,
    ) -> &'a str {
        override_spec
            .and_then(|o| o.range_template.as_deref())
            .unwrap_or(spec.range_template.as_str())
    }

    fn effective_value_decimals(
        spec: &delinea::TooltipSpecV1,
        override_spec: Option<&delinea::TooltipSeriesOverrideV1>,
    ) -> Option<u8> {
        override_spec
            .and_then(|o| o.value_decimals)
            .or(spec.value_decimals)
    }

    fn effective_trim_trailing_zeros(
        spec: &delinea::TooltipSpecV1,
        override_spec: Option<&delinea::TooltipSeriesOverrideV1>,
    ) -> bool {
        override_spec
            .and_then(|o| o.trim_trailing_zeros)
            .unwrap_or(spec.trim_trailing_zeros)
    }

    fn format_value_for_tooltip_with_override(
        model: &ChartModel,
        axis: AxisId,
        window: DataWindow,
        value: f64,
        spec: &delinea::TooltipSpecV1,
        override_spec: Option<&delinea::TooltipSeriesOverrideV1>,
    ) -> String {
        let Some(axis_model) = model.axes.get(&axis) else {
            return delinea::engine::axis::format_value_for(model, axis, window, value);
        };

        let value_decimals = Self::effective_value_decimals(spec, override_spec);
        let trim_trailing_zeros = Self::effective_trim_trailing_zeros(spec, override_spec);

        match &axis_model.scale {
            delinea::AxisScale::Value(_) if value_decimals.is_some() => {
                Self::format_value_value_axis_decimals(
                    value,
                    value_decimals.unwrap_or(0),
                    trim_trailing_zeros,
                )
            }
            _ => delinea::engine::axis::format_value_for(model, axis, window, value),
        }
    }

    fn axis_label(model: &ChartModel, axis: AxisId) -> String {
        let kind = model
            .axes
            .get(&axis)
            .map(|a| a.kind)
            .unwrap_or(delinea::AxisKind::X);

        let name = model.axes.get(&axis).and_then(|a| a.name.as_deref());

        match (kind, name) {
            (delinea::AxisKind::X, Some(name)) => format!("x ({name})"),
            (delinea::AxisKind::Y, Some(name)) => format!("y ({name})"),
            (delinea::AxisKind::X, None) => "x".to_string(),
            (delinea::AxisKind::Y, None) => "y".to_string(),
        }
    }

    fn series_label(model: &ChartModel, series: SeriesId) -> String {
        model
            .series
            .get(&series)
            .and_then(|s| s.name.as_deref())
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("Series {}", series.0))
    }
}

impl TooltipFormatter for DefaultTooltipFormatter {
    fn format_axis_pointer(
        &self,
        engine: &ChartEngine,
        axis_windows: &BTreeMap<AxisId, DataWindow>,
        axis_pointer: &AxisPointerOutput,
    ) -> Vec<TooltipTextLine> {
        let model = engine.model();
        let default_spec = delinea::TooltipSpecV1::default();
        let spec = model.tooltip.as_ref().unwrap_or(&default_spec);

        match &axis_pointer.tooltip {
            delinea::TooltipOutput::Item(item) => {
                let mut lines = Vec::with_capacity(3);
                let series_value = Self::series_label(model, item.series);
                let series_override = Self::series_override(spec, item.series);
                lines.push(TooltipTextLine {
                    source_series: Some(item.series),
                    text: Self::apply_line_template(
                        Self::effective_series_line_template(spec, series_override),
                        "series",
                        &series_value,
                    ),
                });

                let x_window = axis_windows.get(&item.x_axis).copied().unwrap_or_default();
                let x_label = Self::axis_label(model, item.x_axis);
                let x_value = Self::format_value_for_tooltip(
                    model,
                    item.x_axis,
                    x_window,
                    item.x_value,
                    spec,
                );
                lines.push(TooltipTextLine {
                    source_series: None,
                    text: Self::apply_line_template(&spec.axis_line_template, &x_label, &x_value),
                });

                let y_window = axis_windows.get(&item.y_axis).copied().unwrap_or_default();
                let y_label = Self::axis_label(model, item.y_axis);
                let y_value = Self::format_value_for_tooltip(
                    model,
                    item.y_axis,
                    y_window,
                    item.y_value,
                    spec,
                );
                lines.push(TooltipTextLine {
                    source_series: None,
                    text: Self::apply_line_template(&spec.axis_line_template, &y_label, &y_value),
                });

                lines
            }
            delinea::TooltipOutput::Axis(axis) => {
                let mut lines = Vec::with_capacity(1 + axis.series.len());
                let axis_window = axis_windows.get(&axis.axis).copied().unwrap_or_default();
                let axis_label = Self::axis_label(model, axis.axis);
                let axis_value = Self::format_value_for_tooltip(
                    model,
                    axis.axis,
                    axis_window,
                    axis.axis_value,
                    spec,
                );
                lines.push(TooltipTextLine {
                    source_series: None,
                    text: Self::apply_line_template(
                        &spec.axis_line_template,
                        &axis_label,
                        &axis_value,
                    ),
                });

                for entry in &axis.series {
                    let label = Self::series_label(model, entry.series);
                    let series_override = Self::series_override(spec, entry.series);
                    let window = axis_windows
                        .get(&entry.value_axis)
                        .copied()
                        .unwrap_or_default();

                    let value = match &entry.value {
                        delinea::TooltipSeriesValue::Missing => {
                            Self::effective_missing_value(spec, series_override).to_string()
                        }
                        delinea::TooltipSeriesValue::Scalar(v) => {
                            Self::format_value_for_tooltip_with_override(
                                model,
                                entry.value_axis,
                                window,
                                *v,
                                spec,
                                series_override,
                            )
                        }
                        delinea::TooltipSeriesValue::Range { min, max } => {
                            let a = Self::format_value_for_tooltip_with_override(
                                model,
                                entry.value_axis,
                                window,
                                *min,
                                spec,
                                series_override,
                            );
                            let b = Self::format_value_for_tooltip_with_override(
                                model,
                                entry.value_axis,
                                window,
                                *max,
                                spec,
                                series_override,
                            );
                            Self::apply_range_template(
                                Self::effective_range_template(spec, series_override),
                                &a,
                                &b,
                            )
                        }
                    };

                    lines.push(TooltipTextLine {
                        source_series: Some(entry.series),
                        text: Self::apply_line_template(
                            Self::effective_series_line_template(spec, series_override),
                            &label,
                            &value,
                        ),
                    });
                }

                lines
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use delinea::text::{TextMeasurer, TextMetrics};
    use delinea::{
        AxisKind, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
        SeriesSpec, WorkBudget,
    };
    use fret_core::{Point, Px, Rect, Size};

    #[derive(Debug, Default)]
    struct NullTextMeasurer;

    impl TextMeasurer for NullTextMeasurer {
        fn measure(
            &mut self,
            _text: delinea::ids::StringId,
            _style: delinea::text::TextStyleId,
        ) -> TextMetrics {
            TextMetrics::default()
        }
    }

    #[test]
    fn default_formatter_formats_axis_trigger_tooltip_lines() {
        let dataset_id = delinea::DatasetId::new(1);
        let grid_id = delinea::GridId::new(1);
        let x_axis = delinea::AxisId::new(1);
        let y_axis = delinea::AxisId::new(2);
        let series_a = delinea::SeriesId::new(1);
        let series_b = delinea::SeriesId::new(2);
        let x_field = delinea::FieldId::new(1);
        let y_a_field = delinea::FieldId::new(2);
        let y_b_field = delinea::FieldId::new(3);

        let spec = ChartSpec {
            id: delinea::ChartId::new(1),
            viewport: Some(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            )),
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_a_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_b_field,
                        column: 2,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Time".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: delinea::AxisPointerTrigger::Axis,
                snap: false,
                trigger_distance_px: 0.0,
                throttle_px: 0.0,
            }),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: series_a,
                    name: Some("A".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_a_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: series_b,
                    name: Some("B".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_b_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let mut engine = ChartEngine::new(spec).unwrap();
        let mut table = delinea::data::DataTable::default();
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 2.0]));
        engine.datasets_mut().insert(dataset_id, table);

        let mut measurer = NullTextMeasurer::default();
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        assert!(!step.unfinished);

        engine.apply_action(delinea::Action::HoverAt {
            point: Point::new(Px(50.0), Px(50.0)),
        });
        let step = engine
            .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
            .unwrap();
        assert!(!step.unfinished);

        let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
        let formatter = DefaultTooltipFormatter::default();
        let lines =
            formatter.format_axis_pointer(&engine, &engine.output().axis_windows, axis_pointer);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].source_series, None);
        assert_eq!(lines[0].text, "x (Time): 0.5");
        assert_eq!(lines[1].source_series, Some(series_a));
        assert_eq!(lines[1].text, "A: 0.5");
        assert_eq!(lines[2].source_series, Some(series_b));
        assert_eq!(lines[2].text, "B: 1");
    }

    #[test]
    fn tooltip_spec_v1_customizes_templates_and_decimals() {
        let dataset_id = delinea::DatasetId::new(1);
        let grid_id = delinea::GridId::new(1);
        let x_axis = delinea::AxisId::new(1);
        let y_axis = delinea::AxisId::new(2);
        let series_a = delinea::SeriesId::new(1);
        let series_b = delinea::SeriesId::new(2);
        let x_field = delinea::FieldId::new(1);
        let y_a_field = delinea::FieldId::new(2);
        let y_b_field = delinea::FieldId::new(3);

        let tooltip = delinea::TooltipSpecV1 {
            axis_line_template: "{value} @ {label}".to_string(),
            series_line_template: "[{label}]={value}".to_string(),
            missing_value: "(missing)".to_string(),
            range_template: "{min}..{max}".to_string(),
            value_decimals: Some(2),
            trim_trailing_zeros: false,
            series_overrides: Vec::default(),
        };

        let spec = ChartSpec {
            id: delinea::ChartId::new(1),
            viewport: Some(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            )),
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_a_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_b_field,
                        column: 2,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Time".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: Some(tooltip),
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: delinea::AxisPointerTrigger::Axis,
                snap: false,
                trigger_distance_px: 0.0,
                throttle_px: 0.0,
            }),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: series_a,
                    name: Some("A".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_a_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: series_b,
                    name: Some("B".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_b_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let mut engine = ChartEngine::new(spec).unwrap();
        let mut table = delinea::data::DataTable::default();
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 2.0]));
        engine.datasets_mut().insert(dataset_id, table);

        let mut measurer = NullTextMeasurer::default();
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        assert!(!step.unfinished);

        engine.apply_action(delinea::Action::HoverAt {
            point: Point::new(Px(50.0), Px(50.0)),
        });
        let step = engine
            .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
            .unwrap();
        assert!(!step.unfinished);

        let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
        let formatter = DefaultTooltipFormatter::default();
        let lines =
            formatter.format_axis_pointer(&engine, &engine.output().axis_windows, axis_pointer);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].source_series, None);
        assert_eq!(lines[0].text, "0.50 @ x (Time)");
        assert_eq!(lines[1].source_series, Some(series_a));
        assert_eq!(lines[1].text, "[A]=0.50");
        assert_eq!(lines[2].source_series, Some(series_b));
        assert_eq!(lines[2].text, "[B]=1.00");
    }

    #[test]
    fn tooltip_spec_v1_per_series_overrides_apply_to_series_rows() {
        let dataset_id = delinea::DatasetId::new(1);
        let grid_id = delinea::GridId::new(1);
        let x_axis = delinea::AxisId::new(1);
        let y_axis = delinea::AxisId::new(2);
        let series_a = delinea::SeriesId::new(1);
        let series_b = delinea::SeriesId::new(2);
        let x_field = delinea::FieldId::new(1);
        let y_a_field = delinea::FieldId::new(2);
        let y_b_field = delinea::FieldId::new(3);

        let tooltip = delinea::TooltipSpecV1 {
            axis_line_template: "{label}: {value}".to_string(),
            series_line_template: "{label}={value}".to_string(),
            missing_value: "-".to_string(),
            range_template: "{min}..{max}".to_string(),
            value_decimals: Some(2),
            trim_trailing_zeros: false,
            series_overrides: vec![delinea::TooltipSeriesOverrideV1 {
                series: series_b,
                series_line_template: Some("B only: {value}".to_string()),
                missing_value: Some("(none)".to_string()),
                range_template: None,
                value_decimals: Some(0),
                trim_trailing_zeros: Some(true),
            }],
        };

        let spec = ChartSpec {
            id: delinea::ChartId::new(1),
            viewport: Some(Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            )),
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_a_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_b_field,
                        column: 2,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Time".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: Some(tooltip),
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: delinea::AxisPointerTrigger::Axis,
                snap: false,
                trigger_distance_px: 0.0,
                throttle_px: 0.0,
            }),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: series_a,
                    name: Some("A".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_a_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: series_b,
                    name: Some("B".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_b_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let mut engine = ChartEngine::new(spec).unwrap();
        let mut table = delinea::data::DataTable::default();
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 1.0]));
        table.push_column(delinea::data::Column::F64(vec![0.0, 2.0]));
        engine.datasets_mut().insert(dataset_id, table);

        let mut measurer = NullTextMeasurer::default();
        let step = engine
            .step(&mut measurer, WorkBudget::new(262_144, 0, 32))
            .unwrap();
        assert!(!step.unfinished);

        engine.apply_action(delinea::Action::HoverAt {
            point: Point::new(Px(50.0), Px(50.0)),
        });
        let step = engine
            .step(&mut measurer, WorkBudget::new(32_768, 0, 8))
            .unwrap();
        assert!(!step.unfinished);

        let axis_pointer = engine.output().axis_pointer.as_ref().unwrap();
        let formatter = DefaultTooltipFormatter::default();
        let lines =
            formatter.format_axis_pointer(&engine, &engine.output().axis_windows, axis_pointer);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].source_series, None);
        assert_eq!(lines[0].text, "x (Time): 0.50");
        assert_eq!(lines[1].source_series, Some(series_a));
        assert_eq!(lines[1].text, "A=0.50");
        assert_eq!(lines[2].source_series, Some(series_b));
        assert_eq!(lines[2].text, "B only: 1");
    }
}
