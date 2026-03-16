//! Transform editor (position / rotation / scale) built from vec and numeric primitives.
//!
//! This is intentionally an ecosystem-level policy control:
//! - it composes `Vec3Edit` for the numeric editing surface,
//! - it optionally provides a "link scale" toggle,
//! - it can (best-effort) keep scale axes in sync while linked.

use std::panic::Location;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, FontWeight, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::controls::{
    AxisDragValueOutcome, Checkbox, CheckboxOptions, NumericFormatFn, NumericParseFn,
    NumericValidateFn, OnVecEditAxisOutcome, Vec3Edit, VecEditAxis, VecEditAxisOutcome,
    VecEditOptions,
};
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::{EditorDensity, NumericPresentation};

fn derived_id_source(base: Option<&Arc<str>>, suffix: &str) -> Option<Arc<str>> {
    base.map(|id| Arc::<str>::from(format!("{}.{}", id.as_ref(), suffix)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEditLayoutVariant {
    Column,
    Row,
}

impl Default for TransformEditLayoutVariant {
    fn default() -> Self {
        Self::Column
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEditSection {
    Position,
    Rotation,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformEditAxisOutcome {
    pub section: TransformEditSection,
    pub axis: VecEditAxis,
    pub outcome: AxisDragValueOutcome,
}

pub type OnTransformEditAxisOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, TransformEditAxisOutcome) + 'static>;

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

#[derive(Debug, Clone)]
pub struct TransformEditOptions {
    pub layout: LayoutStyle,
    pub variant: TransformEditLayoutVariant,
    pub section_gap: Px,
    pub show_link_scale_toggle: bool,
    pub position_prefix: Option<Arc<str>>,
    pub position_suffix: Option<Arc<str>>,
    pub rotation_prefix: Option<Arc<str>>,
    pub rotation_suffix: Option<Arc<str>>,
    pub scale_prefix: Option<Arc<str>>,
    pub scale_suffix: Option<Arc<str>>,
    /// If `None`, an internal per-element model is used.
    pub linked_scale: Option<Model<bool>>,
    pub default_linked_scale: bool,
    /// Explicit identity source for internal state (linked-scale model, uniform-scale memory).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple transform edits from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub link_test_id: Option<Arc<str>>,
}

impl Default for TransformEditOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            variant: TransformEditLayoutVariant::default(),
            section_gap: Px(6.0),
            show_link_scale_toggle: true,
            position_prefix: None,
            position_suffix: None,
            rotation_prefix: None,
            rotation_suffix: None,
            scale_prefix: None,
            scale_suffix: None,
            linked_scale: None,
            default_linked_scale: false,
            id_source: None,
            test_id: None,
            link_test_id: None,
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
    position_format: NumericFormatFn<f64>,
    position_parse: NumericParseFn<f64>,
    rotation_format: NumericFormatFn<f64>,
    rotation_parse: NumericParseFn<f64>,
    scale_format: NumericFormatFn<f64>,
    scale_parse: NumericParseFn<f64>,
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

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
                            TransformEditAxisOutcome {
                                section: TransformEditSection::Position,
                                axis: outcome.axis,
                                outcome: outcome.outcome,
                            },
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
                            TransformEditAxisOutcome {
                                section: TransformEditSection::Rotation,
                                axis: outcome.axis,
                                outcome: outcome.outcome,
                            },
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
                        TransformEditAxisOutcome {
                            section: TransformEditSection::Scale,
                            axis: outcome.axis,
                            outcome: outcome.outcome,
                        },
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

fn section_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    short: &'static str,
    a11y: &'static str,
    show_link: bool,
    link: Option<(Model<bool>, Option<Arc<str>>)>,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let label_fg = editor_muted_foreground(theme);
    let badge_bg = theme
        .color_by_key("muted")
        .or_else(|| theme.color_by_key("component.card.bg"))
        .unwrap_or_else(|| theme.color_token("background"));
    let badge_border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("component.card.border"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let badge_w = Px(density.row_height.0.max(density.hit_thickness.0));

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            out.push(cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(badge_w),
                            height: Length::Px(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: Some(badge_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(badge_border),
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Arc::from(short),
                        style: Some(typography::as_control_text(TextStyle {
                            size: Px(11.0),
                            weight: FontWeight::SEMIBOLD,
                            line_height: Some(density.row_height),
                            ..Default::default()
                        })),
                        color: Some(label_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: TextAlign::Center,
                        ink_overflow: Default::default(),
                    })]
                },
            ));

            if show_link {
                if let Some((linked, test_id)) = link {
                    let mut el = Checkbox::new(linked)
                        .options(CheckboxOptions {
                            a11y_label: Some(Arc::from(a11y)),
                            focusable: true,
                            enabled: true,
                            ..Default::default()
                        })
                        .into_element(cx);
                    if let Some(test_id) = test_id.as_ref() {
                        el = el.test_id(test_id.clone());
                    }
                    out.push(cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Auto,
                                    height: Length::Px(density.row_height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Horizontal,
                            gap: SpacingLength::Px(Px(4.0)),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            vec![
                                el,
                                cx.text_props(TextProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Auto,
                                            height: Length::Auto,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    text: Arc::from("Link"),
                                    style: Some(typography::as_control_text(TextStyle {
                                        size: Px(11.0),
                                        line_height: Some(density.row_height),
                                        ..Default::default()
                                    })),
                                    color: Some(label_fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                    align: TextAlign::Start,
                                    ink_overflow: Default::default(),
                                }),
                            ]
                        },
                    ));
                }
            }

            out.push(content(cx));
            out
        },
    )
}

fn section_col<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let label_fg = editor_muted_foreground(theme);

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                cx.text_props(TextProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Arc::from(label),
                    style: Some(typography::as_control_text(TextStyle {
                        size: Px(11.0),
                        weight: FontWeight::SEMIBOLD,
                        line_height: Some(Px(14.0)),
                        ..Default::default()
                    })),
                    color: Some(label_fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
                content(cx),
            ]
        },
    )
}

fn section_col_with_link<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    show_link: bool,
    linked_scale: Model<bool>,
    link_test_id: Option<Arc<str>>,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let mut col = section_col(cx, label, content);
    if show_link {
        let theme = Theme::global(&*cx.app);
        let label_fg = editor_muted_foreground(theme);

        col = cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(4.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| {
                let mut out = vec![col];

                let mut el = Checkbox::new(linked_scale)
                    .options(CheckboxOptions {
                        a11y_label: Some(Arc::from("Link scale")),
                        focusable: true,
                        enabled: true,
                        ..Default::default()
                    })
                    .into_element(cx);
                if let Some(test_id) = link_test_id.as_ref() {
                    el = el.test_id(test_id.clone());
                }

                out.push(cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(4.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            el,
                            cx.text_props(TextProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Auto,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                text: Arc::from("Uniform"),
                                style: Some(typography::as_control_text(TextStyle {
                                    size: Px(10.0),
                                    line_height: Some(Px(12.0)),
                                    ..Default::default()
                                })),
                                color: Some(label_fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                                align: TextAlign::Start,
                                ink_overflow: Default::default(),
                            }),
                        ]
                    },
                ));

                out
            },
        );
    }
    col
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

#[track_caller]
fn linked_scale_model<H: UiHost>(cx: &mut ElementContext<'_, H>, default: bool) -> Model<bool> {
    cx.local_model(move || default)
}

#[track_caller]
fn uniform_scale_sync_slot<H: UiHost>(cx: &mut ElementContext<'_, H>) -> fret_ui::GlobalElementId {
    cx.slot_id()
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

fn uniform_scale_sync<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    sync_slot: fret_ui::GlobalElementId,
    _linked: &Model<bool>,
    scale: (&Model<f64>, &Model<f64>, &Model<f64>),
) {
    let (sx, sy, sz) = (
        cx.get_model_copied(scale.0, Invalidation::Layout)
            .unwrap_or(1.0),
        cx.get_model_copied(scale.1, Invalidation::Layout)
            .unwrap_or(1.0),
        cx.get_model_copied(scale.2, Invalidation::Layout)
            .unwrap_or(1.0),
    );

    let next = cx.state_for(
        sync_slot,
        || None::<(f64, f64, f64)>,
        |last| {
            let last_v = *last;
            *last = Some((sx, sy, sz));

            match last_v {
                None => Some((sx, sx, sx)),
                Some((lx, ly, lz)) => {
                    let dx = !approx_eq(sx, lx);
                    let dy = !approx_eq(sy, ly);
                    let dz = !approx_eq(sz, lz);
                    let diffs = dx as u8 + dy as u8 + dz as u8;
                    if diffs == 1 {
                        if dx {
                            Some((sx, sx, sx))
                        } else if dy {
                            Some((sy, sy, sy))
                        } else {
                            Some((sz, sz, sz))
                        }
                    } else {
                        None
                    }
                }
            }
        },
    );

    let Some((ux, uy, uz)) = next else { return };

    let mut did = false;
    if !approx_eq(sx, ux) {
        let _ = cx.app.models_mut().update(scale.0, |v| *v = ux);
        did = true;
    }
    if !approx_eq(sy, uy) {
        let _ = cx.app.models_mut().update(scale.1, |v| *v = uy);
        did = true;
    }
    if !approx_eq(sz, uz) {
        let _ = cx.app.models_mut().update(scale.2, |v| *v = uz);
        did = true;
    }

    if did {
        cx.state_for(
            sync_slot,
            || None::<(f64, f64, f64)>,
            |last| *last = Some((ux, uy, uz)),
        );
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
