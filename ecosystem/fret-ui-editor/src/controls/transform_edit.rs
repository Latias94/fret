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
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::controls::{Checkbox, CheckboxOptions, Vec3Edit, VecEditOptions};
use crate::controls::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::EditorDensity;
use crate::primitives::input_group::derived_test_id;

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
    pub validate: Option<NumericValidateFn<f64>>,
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
            format,
            parse,
            validate: None,
            options: TransformEditOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<f64>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: TransformEditOptions) -> Self {
        self.options = options;
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
            uniform_scale_sync(
                cx,
                &linked_scale,
                (&self.scale_x, &self.scale_y, &self.scale_z),
            );
        } else {
            cx.with_state(|| None::<(f64, f64, f64)>, |st| *st = None);
        }

        let pos = (self.pos_x.clone(), self.pos_y.clone(), self.pos_z.clone());
        let rot = (self.rot_x.clone(), self.rot_y.clone(), self.rot_z.clone());
        let scl = (
            self.scale_x.clone(),
            self.scale_y.clone(),
            self.scale_z.clone(),
        );

        let fmt = self.format.clone();
        let parse = self.parse.clone();
        let validate = self.validate.clone();
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

        let fmt_pos = fmt.clone();
        let parse_pos = parse.clone();
        let validate_pos = validate.clone();
        let pos_options = VecEditOptions {
            prefix: self.options.position_prefix.clone(),
            suffix: self.options.position_suffix.clone(),
            id_source: position_id_source,
            test_id: position_test_id,
            ..Default::default()
        };
        let fmt_rot = fmt.clone();
        let parse_rot = parse.clone();
        let validate_rot = validate.clone();
        let rot_options = VecEditOptions {
            prefix: self.options.rotation_prefix.clone(),
            suffix: self.options.rotation_suffix.clone(),
            id_source: rotation_id_source,
            test_id: rotation_test_id,
            ..Default::default()
        };
        let fmt_scl = fmt.clone();
        let parse_scl = parse.clone();
        let validate_scl = validate.clone();
        let scl_options = VecEditOptions {
            prefix: self.options.scale_prefix.clone(),
            suffix: self.options.scale_suffix.clone(),
            id_source: scale_id_source,
            test_id: scale_test_id,
            ..Default::default()
        };

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
                            Vec3Edit::new(
                                pos.0.clone(),
                                pos.1.clone(),
                                pos.2.clone(),
                                fmt_pos.clone(),
                                parse_pos.clone(),
                            )
                            .validate(validate_pos.clone())
                            .options(pos_options.clone())
                            .into_element(cx)
                        }),
                        section_row(cx, density, "R", "Rotation", false, None, move |cx| {
                            Vec3Edit::new(
                                rot.0.clone(),
                                rot.1.clone(),
                                rot.2.clone(),
                                fmt_rot.clone(),
                                parse_rot.clone(),
                            )
                            .validate(validate_rot.clone())
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
                                Vec3Edit::new(
                                    scl.0.clone(),
                                    scl.1.clone(),
                                    scl.2.clone(),
                                    fmt_scl.clone(),
                                    parse_scl.clone(),
                                )
                                .validate(validate_scl.clone())
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
                            Vec3Edit::new(
                                pos.0.clone(),
                                pos.1.clone(),
                                pos.2.clone(),
                                fmt_pos.clone(),
                                parse_pos.clone(),
                            )
                            .validate(validate_pos.clone())
                            .options(pos_options.clone())
                            .into_element(cx)
                        }),
                        section_col(cx, "Rotation", move |cx| {
                            Vec3Edit::new(
                                rot.0.clone(),
                                rot.1.clone(),
                                rot.2.clone(),
                                fmt_rot.clone(),
                                parse_rot.clone(),
                            )
                            .validate(validate_rot.clone())
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
                                Vec3Edit::new(
                                    scl.0.clone(),
                                    scl.1.clone(),
                                    scl.2.clone(),
                                    fmt_scl.clone(),
                                    parse_scl.clone(),
                                )
                                .validate(validate_scl.clone())
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
    let label_fg = theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));
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
    let label_fg = theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));

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
        let label_fg = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));

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

fn linked_scale_model<H: UiHost>(cx: &mut ElementContext<'_, H>, default: bool) -> Model<bool> {
    let m = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(default);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

fn uniform_scale_sync<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
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

    let next = cx.with_state(
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
        cx.with_state(
            || None::<(f64, f64, f64)>,
            |last| *last = Some((ux, uy, uz)),
        );
    }
}
