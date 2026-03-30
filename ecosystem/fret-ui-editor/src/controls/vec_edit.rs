//! Vector editors (Vec2/Vec3/Vec4) built on top of `DragValue<T>`.
//!
//! These controls are policy-heavy and meant for inspector-like surfaces:
//! - compact axis labels (X/Y/Z/W)
//! - axis color tokens (`editor.axis.*`)
//! - shared numeric formatting/parsing policies

use std::panic::Location;
use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::controls::{
    AxisDragValue, AxisDragValueOptions, AxisDragValueOutcome, NumericFormatFn, NumericParseFn,
    NumericValidateFn, OnAxisDragValueOutcome,
};
use crate::primitives::EditorTokenKeys;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::{NumericPresentation, style::EditorStyle};

fn axis_color(theme: &Theme, key: &'static str, fallback: Color) -> Color {
    theme.color_by_key(key).unwrap_or(fallback)
}

fn derived_id_source(base: Option<&Arc<str>>, suffix: &str) -> Option<Arc<str>> {
    base.map(|id| Arc::<str>::from(format!("{}.{}", id.as_ref(), suffix)))
}

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
    pub axis: VecEditAxis,
    pub outcome: AxisDragValueOutcome,
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

fn axis_group<H: UiHost, T>(
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

        Some(crate::controls::AxisDragValueResetAction {
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
                on_axis_outcome(host, action_cx, VecEditAxisOutcome { axis, outcome });
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

#[derive(Debug, Clone)]
pub struct VecEditOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal element keys.
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub variant: VecEditLayoutVariant,
    pub gap: Px,
    pub axis_gap: Px,
    pub auto_stack_below: Option<Px>,
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
            prefix: None,
            suffix: None,
            id_source: None,
            variant: VecEditLayoutVariant::Auto,
            gap: Px(6.0),
            axis_gap: Px(4.0),
            auto_stack_below: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VecEditLayoutVariant {
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    ///
    /// This is a policy-only heuristic intended to avoid “tiny inputs” when a property grid is
    /// narrow (common in editor sidebars).
    #[default]
    Auto,
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
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
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
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
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

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let model_ids = (x_id, y_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec2_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec2_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let bounds = cx.layout_query_bounds(cx.root_id(), Invalidation::Layout);

        let (x_color, y_color, auto_below, axis_min_width) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);

            let x_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_X_COLOR,
                Color::from_srgb_hex_rgb(0xf2_59_59),
            );
            let y_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_Y_COLOR,
                Color::from_srgb_hex_rgb(0x59_f2_59),
            );

            let auto_below = self
                .options
                .auto_stack_below
                .unwrap_or(style.vec_auto_stack_below);

            (x_color, y_color, auto_below, style.vec_axis_min_width)
        };
        let auto_below = {
            let min_total = axis_min_width.0 * 2.0 + (self.options.gap.0 * 1.0);
            Px(auto_below.0.max(min_total))
        };
        let variant = match self.options.variant {
            VecEditLayoutVariant::Row => VecEditLayoutVariant::Row,
            VecEditLayoutVariant::Column => VecEditLayoutVariant::Column,
            VecEditLayoutVariant::Auto => {
                if bounds.is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0) {
                    VecEditLayoutVariant::Column
                } else {
                    VecEditLayoutVariant::Row
                }
            }
        };

        let grow = variant == VecEditLayoutVariant::Row;
        let direction = match variant {
            VecEditLayoutVariant::Row => Axis::Horizontal,
            VecEditLayoutVariant::Column => Axis::Vertical,
            VecEditLayoutVariant::Auto => Axis::Horizontal,
        };
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
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
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
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
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, z, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
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

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let z_id = self.z.id();
        let model_ids = (x_id, y_id, z_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec3_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec3_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let bounds = cx.layout_query_bounds(cx.root_id(), Invalidation::Layout);

        let (x_color, y_color, z_color, auto_below, axis_min_width) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);

            let x_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_X_COLOR,
                Color::from_srgb_hex_rgb(0xf2_59_59),
            );
            let y_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_Y_COLOR,
                Color::from_srgb_hex_rgb(0x59_f2_59),
            );
            let z_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_Z_COLOR,
                Color::from_srgb_hex_rgb(0x59_8c_f2),
            );

            let auto_below = self
                .options
                .auto_stack_below
                .unwrap_or(style.vec_auto_stack_below);

            (
                x_color,
                y_color,
                z_color,
                auto_below,
                style.vec_axis_min_width,
            )
        };
        let auto_below = {
            let min_total = axis_min_width.0 * 3.0 + (self.options.gap.0 * 2.0);
            Px(auto_below.0.max(min_total))
        };
        let variant = match self.options.variant {
            VecEditLayoutVariant::Row => VecEditLayoutVariant::Row,
            VecEditLayoutVariant::Column => VecEditLayoutVariant::Column,
            VecEditLayoutVariant::Auto => {
                if bounds.is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0) {
                    VecEditLayoutVariant::Column
                } else {
                    VecEditLayoutVariant::Row
                }
            }
        };

        let grow = variant == VecEditLayoutVariant::Row;
        let direction = match variant {
            VecEditLayoutVariant::Row => Axis::Horizontal,
            VecEditLayoutVariant::Column => Axis::Vertical,
            VecEditLayoutVariant::Auto => Axis::Horizontal,
        };
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let z_id_source = derived_id_source(self.options.id_source.as_ref(), "z");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");
        let z_test_id = derived_test_id(self.options.test_id.as_ref(), "z");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Z,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        grow,
                        z_id_source.clone(),
                        z_test_id.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
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
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
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
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        w: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, z, w, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
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

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let z_id = self.z.id();
        let w_id = self.w.id();
        let model_ids = (x_id, y_id, z_id, w_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec4_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec4_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let bounds = cx.layout_query_bounds(cx.root_id(), Invalidation::Layout);

        let (x_color, y_color, z_color, w_color, auto_below, axis_min_width) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);

            let x_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_X_COLOR,
                Color::from_srgb_hex_rgb(0xf2_59_59),
            );
            let y_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_Y_COLOR,
                Color::from_srgb_hex_rgb(0x59_f2_59),
            );
            let z_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_Z_COLOR,
                Color::from_srgb_hex_rgb(0x59_8c_f2),
            );
            let w_color = axis_color(
                theme,
                EditorTokenKeys::AXIS_W_COLOR,
                Color::from_srgb_hex_rgb(0xcc_cc_cc),
            );

            let auto_below = self
                .options
                .auto_stack_below
                .unwrap_or(style.vec_auto_stack_below);

            (
                x_color,
                y_color,
                z_color,
                w_color,
                auto_below,
                style.vec_axis_min_width,
            )
        };
        let auto_below = {
            let min_total = axis_min_width.0 * 4.0 + (self.options.gap.0 * 3.0);
            Px(auto_below.0.max(min_total))
        };
        let variant = match self.options.variant {
            VecEditLayoutVariant::Row => VecEditLayoutVariant::Row,
            VecEditLayoutVariant::Column => VecEditLayoutVariant::Column,
            VecEditLayoutVariant::Auto => {
                if bounds.is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0) {
                    VecEditLayoutVariant::Column
                } else {
                    VecEditLayoutVariant::Row
                }
            }
        };

        let grow = variant == VecEditLayoutVariant::Row;
        let direction = match variant {
            VecEditLayoutVariant::Row => Axis::Horizontal,
            VecEditLayoutVariant::Column => Axis::Vertical,
            VecEditLayoutVariant::Auto => Axis::Horizontal,
        };
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let z_id_source = derived_id_source(self.options.id_source.as_ref(), "z");
        let w_id_source = derived_id_source(self.options.id_source.as_ref(), "w");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");
        let z_test_id = derived_test_id(self.options.test_id.as_ref(), "z");
        let w_test_id = derived_test_id(self.options.test_id.as_ref(), "w");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Z,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        grow,
                        z_id_source.clone(),
                        z_test_id.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::W,
                        self.options.axis_gap,
                        self.reset_w.clone(),
                        grow,
                        w_id_source.clone(),
                        w_test_id.clone(),
                        Arc::from("W"),
                        w_color,
                        self.w.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
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

#[cfg(test)]
mod tests {
    use super::Vec3Edit;
    use crate::primitives::NumericPresentation;
    use fret_app::App;
    use std::sync::Arc;

    #[test]
    fn vec3_edit_from_presentation_adopts_format_parse_and_chrome_affixes() {
        let mut app = App::new();
        let x = app.models_mut().insert(1.0f64);
        let y = app.models_mut().insert(2.0f64);
        let z = app.models_mut().insert(3.0f64);
        let presentation = NumericPresentation::<f64>::fixed_decimals(2)
            .with_chrome_prefix("$")
            .with_chrome_suffix("ms");

        let edit = Vec3Edit::from_presentation(x, y, z, presentation);

        assert_eq!((edit.format)(1.25).as_ref(), "1.25");
        assert_eq!((edit.parse)("1.25"), Some(1.25));
        assert_eq!(edit.options.prefix, Some(Arc::from("$")));
        assert_eq!(edit.options.suffix, Some(Arc::from("ms")));
    }
}
