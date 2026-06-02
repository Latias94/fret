use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::controls::{
    AxisDragValue, AxisDragValueOptions, AxisDragValueOutcome, AxisDragValueResetAction,
    NumericFormatFn, NumericParseFn, NumericValidateFn, OnAxisDragValueOutcome,
};

pub type OnAxisReset = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VecEditAxis {
    X,
    Y,
    Z,
    W,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VecEditAxisOutcome {
    axis: VecEditAxis,
    outcome: AxisDragValueOutcome,
}

impl VecEditAxisOutcome {
    pub(crate) fn new(axis: VecEditAxis, outcome: AxisDragValueOutcome) -> Self {
        Self { axis, outcome }
    }

    pub fn axis(self) -> VecEditAxis {
        self.axis
    }

    pub fn outcome(self) -> AxisDragValueOutcome {
        self.outcome
    }
}

pub type OnVecEditAxisOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, VecEditAxisOutcome) + 'static>;

#[derive(Debug, Clone)]
pub struct AxisResetOptions {
    pub enabled: bool,
    pub icon: IconId,
    pub a11y_label: Arc<str>,
    pub test_id: Option<Arc<str>>,
}

impl Default for AxisResetOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            icon: fret_icons::ids::ui::RESET,
            a11y_label: Arc::from("Reset axis"),
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct AxisReset {
    pub options: AxisResetOptions,
    pub on_reset: OnAxisReset,
}

impl AxisReset {
    pub fn new(on_reset: OnAxisReset) -> Self {
        Self {
            options: AxisResetOptions::default(),
            on_reset,
        }
    }

    pub fn options(mut self, options: AxisResetOptions) -> Self {
        self.options = options;
        self
    }
}

pub(super) fn axis_group<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    axis: VecEditAxis,
    axis_gap: Px,
    reset: Option<AxisReset>,
    grow: bool,
    id_source: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    label: Arc<str>,
    color: Color,
    model: Model<T>,
    prefix: Option<Arc<str>>,
    suffix: Option<Arc<str>>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_axis_outcome: Option<OnVecEditAxisOutcome>,
) -> AnyElement
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    let reset = reset.and_then(|reset| {
        if !reset.options.enabled {
            return None;
        }

        let on_reset = reset.on_reset.clone();
        let on_activate: OnActivate = Arc::new(move |host, action_cx, _reason: ActivateReason| {
            on_reset(host, action_cx);
        });

        Some(AxisDragValueResetAction {
            icon: reset.options.icon,
            a11y_label: reset.options.a11y_label.clone(),
            test_id: reset.options.test_id.clone(),
            on_activate,
        })
    });
    let axis_outcome: Option<OnAxisDragValueOutcome> = on_axis_outcome.map(|on_axis_outcome| {
        let handler: OnAxisDragValueOutcome = Arc::new(
            move |host: &mut dyn UiActionHost,
                  action_cx: ActionCx,
                  outcome: AxisDragValueOutcome| {
                on_axis_outcome(host, action_cx, VecEditAxisOutcome::new(axis, outcome));
            },
        );
        handler
    });

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: if grow { 1.0 } else { 0.0 },
                    basis: if grow {
                        Length::Px(Px(0.0))
                    } else {
                        Length::Auto
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(axis_gap),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            vec![
                AxisDragValue::new(label, color, model, format, parse)
                    .validate(validate)
                    .on_outcome(axis_outcome)
                    .options(AxisDragValueOptions {
                        prefix: prefix.clone(),
                        suffix: suffix.clone(),
                        id_source: id_source.clone(),
                        test_id: test_id.clone(),
                        size: fret_ui_kit::Size::Small,
                        reset: reset.clone(),
                        ..Default::default()
                    })
                    .into_element(cx),
            ]
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{VecEditAxis, VecEditAxisOutcome};
    use crate::primitives::EditSessionOutcome;

    #[test]
    fn vec_edit_axis_outcome_exposes_read_only_signals() {
        let outcome = VecEditAxisOutcome::new(VecEditAxis::Z, EditSessionOutcome::Committed);

        assert_eq!(outcome.axis(), VecEditAxis::Z);
        assert_eq!(outcome.outcome(), EditSessionOutcome::Committed);
    }
}
