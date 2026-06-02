//! TransformEdit public model owner.

use std::panic::Location;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::NumericPresentation;

use super::{OnTransformEditAxisOutcome, TransformEditOptions};

#[derive(Clone)]
pub struct TransformEditPresentations {
    pub position: NumericPresentation<f64>,
    pub rotation: NumericPresentation<f64>,
    pub scale: NumericPresentation<f64>,
}

impl TransformEditPresentations {
    pub fn new(
        position: NumericPresentation<f64>,
        rotation: NumericPresentation<f64>,
        scale: NumericPresentation<f64>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn shared(presentation: NumericPresentation<f64>) -> Self {
        Self {
            position: presentation.clone(),
            rotation: presentation.clone(),
            scale: presentation,
        }
    }
}

#[derive(Clone)]
pub struct TransformEdit {
    pub pos_x: Model<f64>,
    pub pos_y: Model<f64>,
    pub pos_z: Model<f64>,
    pub rot_x: Model<f64>,
    pub rot_y: Model<f64>,
    pub rot_z: Model<f64>,
    pub scale_x: Model<f64>,
    pub scale_y: Model<f64>,
    pub scale_z: Model<f64>,
    pub format: NumericFormatFn<f64>,
    pub parse: NumericParseFn<f64>,
    pub(super) position_format: NumericFormatFn<f64>,
    pub(super) position_parse: NumericParseFn<f64>,
    pub(super) rotation_format: NumericFormatFn<f64>,
    pub(super) rotation_parse: NumericParseFn<f64>,
    pub(super) scale_format: NumericFormatFn<f64>,
    pub(super) scale_parse: NumericParseFn<f64>,
    pub validate: Option<NumericValidateFn<f64>>,
    pub on_axis_outcome: Option<OnTransformEditAxisOutcome>,
    pub options: TransformEditOptions,
}

impl TransformEdit {
    pub fn new(
        position: (Model<f64>, Model<f64>, Model<f64>),
        rotation: (Model<f64>, Model<f64>, Model<f64>),
        scale: (Model<f64>, Model<f64>, Model<f64>),
        format: NumericFormatFn<f64>,
        parse: NumericParseFn<f64>,
    ) -> Self {
        Self {
            pos_x: position.0,
            pos_y: position.1,
            pos_z: position.2,
            rot_x: rotation.0,
            rot_y: rotation.1,
            rot_z: rotation.2,
            scale_x: scale.0,
            scale_y: scale.1,
            scale_z: scale.2,
            format: format.clone(),
            parse: parse.clone(),
            position_format: format.clone(),
            position_parse: parse.clone(),
            rotation_format: format.clone(),
            rotation_parse: parse.clone(),
            scale_format: format,
            scale_parse: parse,
            validate: None,
            on_axis_outcome: None,
            options: TransformEditOptions::default(),
        }
    }

    pub fn from_presentations(
        position: (Model<f64>, Model<f64>, Model<f64>),
        rotation: (Model<f64>, Model<f64>, Model<f64>),
        scale: (Model<f64>, Model<f64>, Model<f64>),
        presentations: TransformEditPresentations,
    ) -> Self {
        let mut edit = Self::new(
            position,
            rotation,
            scale,
            presentations.position.format(),
            presentations.position.parse(),
        );
        edit.position_format = presentations.position.format();
        edit.position_parse = presentations.position.parse();
        edit.rotation_format = presentations.rotation.format();
        edit.rotation_parse = presentations.rotation.parse();
        edit.scale_format = presentations.scale.format();
        edit.scale_parse = presentations.scale.parse();
        edit.options.position_prefix = presentations.position.chrome_prefix().cloned();
        edit.options.position_suffix = presentations.position.chrome_suffix().cloned();
        edit.options.rotation_prefix = presentations.rotation.chrome_prefix().cloned();
        edit.options.rotation_suffix = presentations.rotation.chrome_suffix().cloned();
        edit.options.scale_prefix = presentations.scale.chrome_prefix().cloned();
        edit.options.scale_suffix = presentations.scale.chrome_suffix().cloned();
        edit
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<f64>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: TransformEditOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnTransformEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        let model_ids = (
            self.pos_x.id(),
            self.pos_y.id(),
            self.pos_z.id(),
            self.rot_x.id(),
            self.rot_y.id(),
            self.rot_z.id(),
            self.scale_x.id(),
            self.scale_y.id(),
            self.scale_z.id(),
        );

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.transform_edit", id_source, model_ids),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.transform_edit", callsite, model_ids),
                |cx| self.into_element_keyed(cx),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TransformEdit, TransformEditPresentations};
    use crate::primitives::NumericPresentation;
    use fret_app::App;
    use std::sync::Arc;

    #[test]
    fn transform_edit_from_presentations_adopts_section_format_parse_and_affixes() {
        let mut app = App::new();
        let pos_x = app.models_mut().insert(0.0f64);
        let pos_y = app.models_mut().insert(0.0f64);
        let pos_z = app.models_mut().insert(0.0f64);
        let rot_x = app.models_mut().insert(0.0f64);
        let rot_y = app.models_mut().insert(0.0f64);
        let rot_z = app.models_mut().insert(0.0f64);
        let scale_x = app.models_mut().insert(1.0f64);
        let scale_y = app.models_mut().insert(1.0f64);
        let scale_z = app.models_mut().insert(1.0f64);

        let position = NumericPresentation::<f64>::fixed_decimals(2).with_chrome_suffix("m");
        let rotation = NumericPresentation::<f64>::degrees(0);
        let scale = NumericPresentation::<f64>::percent_0_1(0);

        let edit = TransformEdit::from_presentations(
            (pos_x, pos_y, pos_z),
            (rot_x, rot_y, rot_z),
            (scale_x, scale_y, scale_z),
            TransformEditPresentations::new(position, rotation, scale),
        );

        assert_eq!((edit.position_format)(1.25).as_ref(), "1.25");
        assert_eq!((edit.position_parse)("1.25"), Some(1.25));
        assert_eq!((edit.rotation_format)(90.0).as_ref(), "90°");
        assert_eq!((edit.rotation_parse)("90°"), Some(90.0));
        assert_eq!((edit.scale_format)(0.25).as_ref(), "25%");
        assert_eq!((edit.scale_parse)("25%"), Some(0.25));
        assert_eq!(edit.options.position_suffix, Some(Arc::from("m")));
        assert!(edit.options.rotation_suffix.is_none());
        assert!(edit.options.scale_suffix.is_none());
    }
}
