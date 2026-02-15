//! Vector editors (Vec2/Vec3/Vec4) built on top of `DragValue<T>`.
//!
//! These controls are policy-heavy and meant for inspector-like surfaces:
//! - compact axis labels (X/Y/Z/W)
//! - axis color tokens (`editor.axis.*`)
//! - shared numeric formatting/parsing policies

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Color, Edges, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::controls::{DragValue, NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::{EditorDensity, EditorTokenKeys};

fn axis_color(theme: &Theme, key: &'static str, fallback: Color) -> Color {
    theme.color_by_key(key).unwrap_or(fallback)
}

pub type OnAxisReset = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct AxisResetOptions {
    pub enabled: bool,
    pub glyph: Arc<str>,
    pub a11y_label: Arc<str>,
    pub test_id: Option<Arc<str>>,
}

impl Default for AxisResetOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            glyph: Arc::from("⟲"),
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

fn axis_label_el<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    label: Arc<str>,
    color: Color,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(0.0)),
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
                text: label.clone(),
                style: Some(TextStyle {
                    size: Px(10.0),
                    line_height: Some(density.row_height),
                    ..Default::default()
                }),
                color: Some(color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Center,
            })]
        },
    )
}

fn axis_group<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    axis_gap: Px,
    reset: Option<AxisReset>,
    label: Arc<str>,
    color: Color,
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
) -> AnyElement
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    let reset_el = reset.and_then(|reset| {
        if !reset.options.enabled {
            return None;
        }

        let glyph = reset.options.glyph.clone();
        let a11y_label = reset.options.a11y_label.clone();
        let on_reset = reset.on_reset.clone();

        let theme = Theme::global(&*cx.app);
        let reset_fg = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));

        let mut el = cx.pressable(
            PressableProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(density.hit_thickness),
                        height: Length::Px(density.row_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                focusable: false,
                a11y: PressableA11y {
                    label: Some(a11y_label),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st| {
                let on_activate: OnActivate = Arc::new({
                    let on_reset = on_reset.clone();
                    move |host, action_cx, _reason: ActivateReason| {
                        on_reset(host, action_cx);
                    }
                });
                cx.pressable_add_on_activate(on_activate);

                vec![cx.text_props(TextProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: glyph.clone(),
                    style: Some(TextStyle {
                        size: Px(10.0),
                        line_height: Some(density.row_height),
                        ..Default::default()
                    }),
                    color: Some(reset_fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::Center,
                })]
            },
        );

        if let Some(test_id) = reset.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        Some(el)
    });

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
            gap: axis_gap,
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            if let Some(reset) = reset_el {
                out.push(reset);
            }
            out.push(axis_label_el(cx, density, label, color));
            out.push(
                DragValue::new(model, format, parse)
                    .validate(validate)
                    .into_element(cx),
            );
            out
        },
    )
}

#[derive(Debug, Clone)]
pub struct VecEditOptions {
    pub layout: LayoutStyle,
    pub gap: Px,
    pub axis_gap: Px,
    pub test_id: Option<Arc<str>>,
}

impl Default for VecEditOptions {
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
            gap: Px(6.0),
            axis_gap: Px(4.0),
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct Vec2Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub options: VecEditOptions,
}

impl<T> Vec2Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            reset_x: None,
            reset_y: None,
            format,
            parse,
            validate: None,
            options: VecEditOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let x_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_X_COLOR,
            Color {
                r: 0.95,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        );
        let y_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_Y_COLOR,
            Color {
                r: 0.35,
                g: 0.95,
                b: 0.35,
                a: 1.0,
            },
        );

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction: Axis::Horizontal,
                gap: self.options.gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}

#[derive(Clone)]
pub struct Vec3Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub z: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub reset_z: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub options: VecEditOptions,
}

impl<T> Vec3Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            z,
            reset_x: None,
            reset_y: None,
            reset_z: None,
            format,
            parse,
            validate: None,
            options: VecEditOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn reset_z(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_z = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let x_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_X_COLOR,
            Color {
                r: 0.95,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        );
        let y_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_Y_COLOR,
            Color {
                r: 0.35,
                g: 0.95,
                b: 0.35,
                a: 1.0,
            },
        );
        let z_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_Z_COLOR,
            Color {
                r: 0.35,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
        );

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction: Axis::Horizontal,
                gap: self.options.gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}

#[derive(Clone)]
pub struct Vec4Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub z: Model<T>,
    pub w: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub reset_z: Option<AxisReset>,
    pub reset_w: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub options: VecEditOptions,
}

impl<T> Vec4Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        w: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            z,
            w,
            reset_x: None,
            reset_y: None,
            reset_z: None,
            reset_w: None,
            format,
            parse,
            validate: None,
            options: VecEditOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn reset_z(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_z = reset;
        self
    }

    pub fn reset_w(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_w = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let x_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_X_COLOR,
            Color {
                r: 0.95,
                g: 0.35,
                b: 0.35,
                a: 1.0,
            },
        );
        let y_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_Y_COLOR,
            Color {
                r: 0.35,
                g: 0.95,
                b: 0.35,
                a: 1.0,
            },
        );
        let z_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_Z_COLOR,
            Color {
                r: 0.35,
                g: 0.55,
                b: 0.95,
                a: 1.0,
            },
        );
        let w_color = axis_color(
            theme,
            EditorTokenKeys::AXIS_W_COLOR,
            Color {
                r: 0.8,
                g: 0.8,
                b: 0.8,
                a: 1.0,
            },
        );

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction: Axis::Horizontal,
                gap: self.options.gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                    axis_group(
                        cx,
                        density,
                        self.options.axis_gap,
                        self.reset_w.clone(),
                        Arc::from("W"),
                        w_color,
                        self.w.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
