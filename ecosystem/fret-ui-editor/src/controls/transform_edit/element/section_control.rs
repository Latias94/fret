//! TransformEdit section VecEdit control owner.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{
    NumericFormatFn, NumericParseFn, NumericValidateFn, OnTransformEditAxisOutcome,
    OnVecEditAxisOutcome, Vec3Edit, VecEditAxisOutcome, VecEditOptions,
};
use crate::primitives::NumericPresentation;
use crate::primitives::input_group::derived_test_id;

use super::super::{TransformEdit, TransformEditAxisOutcome, TransformEditSection};

#[derive(Clone)]
pub(super) struct TransformEditSectionControl {
    models: (Model<f64>, Model<f64>, Model<f64>),
    presentation: NumericPresentation<f64>,
    validate: Option<NumericValidateFn<f64>>,
    on_axis_outcome: Option<OnVecEditAxisOutcome>,
    options: VecEditOptions,
}

pub(super) struct TransformEditSectionControls {
    pub(super) position: TransformEditSectionControl,
    pub(super) rotation: TransformEditSectionControl,
    pub(super) scale: TransformEditSectionControl,
    pub(super) link_test_id: Option<Arc<str>>,
}

impl TransformEditSectionControl {
    pub(super) fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Vec3Edit::from_presentation(
            self.models.0,
            self.models.1,
            self.models.2,
            self.presentation,
        )
        .validate(self.validate)
        .on_axis_outcome(self.on_axis_outcome)
        .options(self.options)
        .into_element(cx)
    }
}

pub(super) fn transform_edit_section_controls(
    edit: &TransformEdit,
) -> TransformEditSectionControls {
    TransformEditSectionControls {
        position: TransformEditSectionControl {
            models: (edit.pos_x.clone(), edit.pos_y.clone(), edit.pos_z.clone()),
            presentation: transform_section_presentation(
                edit.position_format.clone(),
                edit.position_parse.clone(),
                edit.options.position_prefix.clone(),
                edit.options.position_suffix.clone(),
            ),
            validate: edit.validate.clone(),
            on_axis_outcome: map_section_outcome(
                TransformEditSection::Position,
                edit.on_axis_outcome.clone(),
            ),
            options: VecEditOptions {
                id_source: derived_id_source(edit.options.id_source.as_ref(), "position"),
                test_id: derived_test_id(edit.options.test_id.as_ref(), "position"),
                ..Default::default()
            },
        },
        rotation: TransformEditSectionControl {
            models: (edit.rot_x.clone(), edit.rot_y.clone(), edit.rot_z.clone()),
            presentation: transform_section_presentation(
                edit.rotation_format.clone(),
                edit.rotation_parse.clone(),
                edit.options.rotation_prefix.clone(),
                edit.options.rotation_suffix.clone(),
            ),
            validate: edit.validate.clone(),
            on_axis_outcome: map_section_outcome(
                TransformEditSection::Rotation,
                edit.on_axis_outcome.clone(),
            ),
            options: VecEditOptions {
                id_source: derived_id_source(edit.options.id_source.as_ref(), "rotation"),
                test_id: derived_test_id(edit.options.test_id.as_ref(), "rotation"),
                ..Default::default()
            },
        },
        scale: TransformEditSectionControl {
            models: (
                edit.scale_x.clone(),
                edit.scale_y.clone(),
                edit.scale_z.clone(),
            ),
            presentation: transform_section_presentation(
                edit.scale_format.clone(),
                edit.scale_parse.clone(),
                edit.options.scale_prefix.clone(),
                edit.options.scale_suffix.clone(),
            ),
            validate: edit.validate.clone(),
            on_axis_outcome: map_section_outcome(
                TransformEditSection::Scale,
                edit.on_axis_outcome.clone(),
            ),
            options: VecEditOptions {
                id_source: derived_id_source(edit.options.id_source.as_ref(), "scale"),
                test_id: derived_test_id(edit.options.test_id.as_ref(), "scale"),
                ..Default::default()
            },
        },
        link_test_id: edit
            .options
            .link_test_id
            .clone()
            .or_else(|| derived_test_id(edit.options.test_id.as_ref(), "link-scale")),
    }
}

fn derived_id_source(base: Option<&Arc<str>>, suffix: &str) -> Option<Arc<str>> {
    base.map(|id| Arc::<str>::from(format!("{}.{}", id.as_ref(), suffix)))
}

fn transform_section_presentation(
    format: NumericFormatFn<f64>,
    parse: NumericParseFn<f64>,
    prefix: Option<Arc<str>>,
    suffix: Option<Arc<str>>,
) -> NumericPresentation<f64> {
    let mut presentation = NumericPresentation::new(format, parse);
    if let Some(prefix) = prefix {
        presentation = presentation.with_chrome_prefix(prefix);
    }
    if let Some(suffix) = suffix {
        presentation = presentation.with_chrome_suffix(suffix);
    }
    presentation
}

fn map_section_outcome(
    section: TransformEditSection,
    on_axis_outcome: Option<OnTransformEditAxisOutcome>,
) -> Option<OnVecEditAxisOutcome> {
    on_axis_outcome.map(|on_axis_outcome| {
        let handler: OnVecEditAxisOutcome = Arc::new(
            move |host: &mut dyn UiActionHost, action_cx: ActionCx, outcome: VecEditAxisOutcome| {
                on_axis_outcome(
                    host,
                    action_cx,
                    TransformEditAxisOutcome::new(section, outcome.axis(), outcome.outcome()),
                );
            },
        );
        handler
    })
}
