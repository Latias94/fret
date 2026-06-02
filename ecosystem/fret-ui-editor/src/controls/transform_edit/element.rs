use fret_core::{Axis, Edges, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SpacingLength};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::EditorDensity;

use super::sections::{section_col, section_col_with_link, section_row};
use super::sync::{linked_scale_model, uniform_scale_sync, uniform_scale_sync_slot};
use super::{TransformEdit, TransformEditLayoutVariant};

mod section_control;

use section_control::transform_edit_section_controls;

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

        let section_controls = transform_edit_section_controls(&self);

        let mut el = match self.options.variant {
            TransformEditLayoutVariant::Column => {
                let position_control = section_controls.position.clone();
                let rotation_control = section_controls.rotation.clone();
                let scale_control = section_controls.scale.clone();
                let link_test_id = section_controls.link_test_id.clone();
                cx.flex(
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
                                position_control.into_element(cx)
                            }),
                            section_row(cx, density, "R", "Rotation", false, None, move |cx| {
                                rotation_control.into_element(cx)
                            }),
                            section_row(
                                cx,
                                density,
                                "S",
                                "Scale",
                                self.options.show_link_scale_toggle,
                                Some((linked_scale.clone(), link_test_id.clone())),
                                move |cx| scale_control.into_element(cx),
                            ),
                        ]
                    },
                )
            }
            TransformEditLayoutVariant::Row => {
                let position_control = section_controls.position.clone();
                let rotation_control = section_controls.rotation.clone();
                let scale_control = section_controls.scale.clone();
                let link_test_id = section_controls.link_test_id.clone();
                cx.flex(
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
                                position_control.into_element(cx)
                            }),
                            section_col(cx, "Rotation", move |cx| {
                                rotation_control.into_element(cx)
                            }),
                            section_col_with_link(
                                cx,
                                "Scale",
                                self.options.show_link_scale_toggle,
                                linked_scale.clone(),
                                link_test_id.clone(),
                                move |cx| scale_control.into_element(cx),
                            ),
                        ]
                    },
                )
            }
        };

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
