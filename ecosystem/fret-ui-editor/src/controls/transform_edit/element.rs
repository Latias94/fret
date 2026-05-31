use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SpacingLength};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::controls::{
    NumericFormatFn, NumericParseFn, OnVecEditAxisOutcome, Vec3Edit, VecEditAxisOutcome,
    VecEditOptions,
};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::{EditorDensity, NumericPresentation};

use super::sections::{section_col, section_col_with_link, section_row};
use super::sync::{linked_scale_model, uniform_scale_sync, uniform_scale_sync_slot};
use super::{
    TransformEdit, TransformEditAxisOutcome, TransformEditLayoutVariant, TransformEditSection,
};

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

impl TransformEdit {
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let linked_scale = self
            .options
            .linked_scale
            .clone()
            .unwrap_or_else(|| linked_scale_model(cx, self.options.default_linked_scale));

        // Best-effort "uniform scale" behavior:
        // When linked, mirror the most recently changed axis across X/Y/Z.
        if cx
            .get_model_copied(&linked_scale, Invalidation::Layout)
            .unwrap_or(false)
        {
            let sync_slot = uniform_scale_sync_slot(cx);
            uniform_scale_sync(
                cx,
                sync_slot,
                &linked_scale,
                (&self.scale_x, &self.scale_y, &self.scale_z),
            );
        } else {
            let sync_slot = uniform_scale_sync_slot(cx);
            cx.state_for(sync_slot, || None::<(f64, f64, f64)>, |st| *st = None);
        }

        let pos = (self.pos_x.clone(), self.pos_y.clone(), self.pos_z.clone());
        let rot = (self.rot_x.clone(), self.rot_y.clone(), self.rot_z.clone());
        let scl = (
            self.scale_x.clone(),
            self.scale_y.clone(),
            self.scale_z.clone(),
        );

        let validate = self.validate.clone();
        let on_axis_outcome = self.on_axis_outcome.clone();
        let position_id_source = derived_id_source(self.options.id_source.as_ref(), "position");
        let rotation_id_source = derived_id_source(self.options.id_source.as_ref(), "rotation");
        let scale_id_source = derived_id_source(self.options.id_source.as_ref(), "scale");
        let position_test_id = derived_test_id(self.options.test_id.as_ref(), "position");
        let rotation_test_id = derived_test_id(self.options.test_id.as_ref(), "rotation");
        let scale_test_id = derived_test_id(self.options.test_id.as_ref(), "scale");
        let link_test_id = self
            .options
            .link_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "link-scale"));

        let validate_pos = validate.clone();
        let pos_presentation = transform_section_presentation(
            self.position_format.clone(),
            self.position_parse.clone(),
            self.options.position_prefix.clone(),
            self.options.position_suffix.clone(),
        );
        let pos_options = VecEditOptions {
            id_source: position_id_source,
            test_id: position_test_id,
            ..Default::default()
        };
        let pos_outcome: Option<OnVecEditAxisOutcome> =
            on_axis_outcome.clone().map(|on_axis_outcome| {
                let handler: OnVecEditAxisOutcome = Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          outcome: VecEditAxisOutcome| {
                        on_axis_outcome(
                            host,
                            action_cx,
                            TransformEditAxisOutcome::new(
                                TransformEditSection::Position,
                                outcome.axis(),
                                outcome.outcome(),
                            ),
                        );
                    },
                );
                handler
            });
        let validate_rot = validate.clone();
        let rot_presentation = transform_section_presentation(
            self.rotation_format.clone(),
            self.rotation_parse.clone(),
            self.options.rotation_prefix.clone(),
            self.options.rotation_suffix.clone(),
        );
        let rot_options = VecEditOptions {
            id_source: rotation_id_source,
            test_id: rotation_test_id,
            ..Default::default()
        };
        let rot_outcome: Option<OnVecEditAxisOutcome> =
            on_axis_outcome.clone().map(|on_axis_outcome| {
                let handler: OnVecEditAxisOutcome = Arc::new(
                    move |host: &mut dyn UiActionHost,
                          action_cx: ActionCx,
                          outcome: VecEditAxisOutcome| {
                        on_axis_outcome(
                            host,
                            action_cx,
                            TransformEditAxisOutcome::new(
                                TransformEditSection::Rotation,
                                outcome.axis(),
                                outcome.outcome(),
                            ),
                        );
                    },
                );
                handler
            });
        let validate_scl = validate.clone();
        let scl_presentation = transform_section_presentation(
            self.scale_format.clone(),
            self.scale_parse.clone(),
            self.options.scale_prefix.clone(),
            self.options.scale_suffix.clone(),
        );
        let scl_options = VecEditOptions {
            id_source: scale_id_source,
            test_id: scale_test_id,
            ..Default::default()
        };
        let scl_outcome: Option<OnVecEditAxisOutcome> = on_axis_outcome.map(|on_axis_outcome| {
            let handler: OnVecEditAxisOutcome = Arc::new(
                move |host: &mut dyn UiActionHost,
                      action_cx: ActionCx,
                      outcome: VecEditAxisOutcome| {
                    on_axis_outcome(
                        host,
                        action_cx,
                        TransformEditAxisOutcome::new(
                            TransformEditSection::Scale,
                            outcome.axis(),
                            outcome.outcome(),
                        ),
                    );
                },
            );
            handler
        });

        let mut el = match self.options.variant {
            TransformEditLayoutVariant::Column => cx.flex(
                FlexProps {
                    layout: self.options.layout,
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(self.options.section_gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |cx| {
                    vec![
                        section_row(cx, density, "P", "Position", false, None, move |cx| {
                            Vec3Edit::from_presentation(
                                pos.0.clone(),
                                pos.1.clone(),
                                pos.2.clone(),
                                pos_presentation.clone(),
                            )
                            .validate(validate_pos.clone())
                            .on_axis_outcome(pos_outcome.clone())
                            .options(pos_options.clone())
                            .into_element(cx)
                        }),
                        section_row(cx, density, "R", "Rotation", false, None, move |cx| {
                            Vec3Edit::from_presentation(
                                rot.0.clone(),
                                rot.1.clone(),
                                rot.2.clone(),
                                rot_presentation.clone(),
                            )
                            .validate(validate_rot.clone())
                            .on_axis_outcome(rot_outcome.clone())
                            .options(rot_options.clone())
                            .into_element(cx)
                        }),
                        section_row(
                            cx,
                            density,
                            "S",
                            "Scale",
                            self.options.show_link_scale_toggle,
                            Some((linked_scale.clone(), link_test_id.clone())),
                            move |cx| {
                                Vec3Edit::from_presentation(
                                    scl.0.clone(),
                                    scl.1.clone(),
                                    scl.2.clone(),
                                    scl_presentation.clone(),
                                )
                                .validate(validate_scl.clone())
                                .on_axis_outcome(scl_outcome.clone())
                                .options(scl_options.clone())
                                .into_element(cx)
                            },
                        ),
                    ]
                },
            ),
            TransformEditLayoutVariant::Row => cx.flex(
                FlexProps {
                    layout: self.options.layout,
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(self.options.section_gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Start,
                    wrap: false,
                },
                move |cx| {
                    vec![
                        section_col(cx, "Position", move |cx| {
                            Vec3Edit::from_presentation(
                                pos.0.clone(),
                                pos.1.clone(),
                                pos.2.clone(),
                                pos_presentation.clone(),
                            )
                            .validate(validate_pos.clone())
                            .on_axis_outcome(pos_outcome.clone())
                            .options(pos_options.clone())
                            .into_element(cx)
                        }),
                        section_col(cx, "Rotation", move |cx| {
                            Vec3Edit::from_presentation(
                                rot.0.clone(),
                                rot.1.clone(),
                                rot.2.clone(),
                                rot_presentation.clone(),
                            )
                            .validate(validate_rot.clone())
                            .on_axis_outcome(rot_outcome.clone())
                            .options(rot_options.clone())
                            .into_element(cx)
                        }),
                        section_col_with_link(
                            cx,
                            "Scale",
                            self.options.show_link_scale_toggle,
                            linked_scale.clone(),
                            link_test_id.clone(),
                            move |cx| {
                                Vec3Edit::from_presentation(
                                    scl.0.clone(),
                                    scl.1.clone(),
                                    scl.2.clone(),
                                    scl_presentation.clone(),
                                )
                                .validate(validate_scl.clone())
                                .on_axis_outcome(scl_outcome.clone())
                                .options(scl_options.clone())
                                .into_element(cx)
                            },
                        ),
                    ]
                },
            ),
        };

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
