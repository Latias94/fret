use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::direction::LayoutDirection;
use crate::optional_text_value_model::IntoOptionalTextValueModel;
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, Rect, Size, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, RovingFocusProps, SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, ControlRegistry, control_registry_model,
};
use fret_ui_kit::primitives::radio_group as radio_group_prim;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Space, WidgetState,
    WidgetStateProperty, WidgetStates, resolve_override_slot,
};

use crate::overlay_motion;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn row_gap(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.radio_group.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme))
}

fn label_gap(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.radio_group.label_gap")
        // Upstream shadcn `radio-group-demo` uses `gap-3` between icon and label.
        .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme))
}

fn icon_size(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.radio_group.icon_size_px")
        .unwrap_or(Px(16.0))
}

fn indicator_size(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.radio_group.indicator_size_px")
        .unwrap_or(Px(8.0))
}

fn radio_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.radio_group.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.radio_group.line_height")
        .unwrap_or(px);

    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    // Upstream shadcn `Label` defaults to `font-medium`.
    style.weight = FontWeight::MEDIUM;
    style
}

fn radio_border(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .expect("missing theme token: input/border")
}

fn radio_ring(theme: &ThemeSnapshot) -> Color {
    theme.color_token("ring")
}

fn radio_fg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("foreground")
}

fn radio_indicator(theme: &ThemeSnapshot) -> Color {
    theme.color_token("primary")
}

pub use fret_ui_kit::primitives::radio_group::RadioGroupOrientation;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RadioGroupRenderMode {
    FullItem,
    ControlOnly,
}

#[derive(Clone)]
struct RadioGroupRenderContext {
    theme: ThemeSnapshot,
    fit_width: bool,
    is_rtl: bool,
    orientation: RadioGroupOrientation,
    root: radio_group_prim::RadioGroupRoot,
    model: Model<Option<Arc<str>>>,
    control_id_for_register: Option<ControlId>,
    control_registry_for_register: Option<Model<ControlRegistry>>,
    set_size: Option<u32>,
    gap_x: Px,
    icon: Px,
    indicator: Px,
    text_style: TextStyle,
    style_override: RadioGroupStyle,
    default_icon_border_color: WidgetStateProperty<ColorRef>,
    default_label_color: WidgetStateProperty<ColorRef>,
    default_indicator_color: WidgetStateProperty<ColorRef>,
    pressable_layout_full: LayoutStyle,
}

#[derive(Clone, Debug)]
struct RadioGroupPartsItem {
    value: Arc<str>,
    label: Arc<str>,
    control_id: Option<ControlId>,
    enabled: bool,
    aria_invalid: bool,
    idx: usize,
    tab_stop: bool,
    item_test_id: Option<Arc<str>>,
}

#[derive(Clone)]
pub struct RadioGroupParts {
    render: RadioGroupRenderContext,
    items: Arc<[RadioGroupPartsItem]>,
    by_value: Arc<HashMap<Arc<str>, usize>>,
    next_expected_index: Rc<Cell<usize>>,
}

impl RadioGroupParts {
    /// Renders the radio control for a previously declared item.
    ///
    /// Items must be rendered in the same order they were added to [`RadioGroup`], which keeps the
    /// roving-focus order aligned with the visual row order for docs-shaped compositions.
    #[track_caller]
    pub fn control<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        value: impl Into<Arc<str>>,
    ) -> AnyElement {
        let value = value.into();
        let Some(idx) = self.by_value.get(&value).copied() else {
            panic!("unknown radio-group parts item value: {}", value.as_ref());
        };
        let expected = self.next_expected_index.get();
        debug_assert_eq!(
            idx, expected,
            "radio-group parts items must render in declaration order"
        );
        self.next_expected_index.set(expected.saturating_add(1));

        build_radio_item_element(
            cx,
            &self.render,
            &self.items[idx],
            None,
            RadioGroupItemVariant::Default,
            RadioGroupRenderMode::ControlOnly,
        )
    }
}

#[track_caller]
fn build_radio_item_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    render: &RadioGroupRenderContext,
    item: &RadioGroupPartsItem,
    item_children: Option<Vec<AnyElement>>,
    item_variant: RadioGroupItemVariant,
    render_mode: RadioGroupRenderMode,
) -> AnyElement {
    let theme = render.theme.clone();
    let icon = render.icon;
    let indicator = render.indicator;
    let gap_x = render.gap_x;
    let fit_width = render.fit_width;
    let is_rtl = render.is_rtl;
    let orientation = render.orientation;
    let root_for_item = render.root.clone();
    let model_for_control = render.model.clone();
    let control_id_for_register = render.control_id_for_register.clone();
    let control_registry_for_register = render.control_registry_for_register.clone();
    let style_override = render.style_override.clone();
    let default_icon_border_color = render.default_icon_border_color.clone();
    let default_label_color = render.default_label_color.clone();
    let default_indicator_color = render.default_indicator_color.clone();
    let pressable_layout_full = render.pressable_layout_full;
    let text_style = render.text_style.clone();
    let set_size = render.set_size;

    let item_test_id = item.item_test_id.clone();
    let item_enabled = item.enabled;
    let tab_stop = item.tab_stop;
    let aria_invalid = item.aria_invalid;
    let item_control_id = item.control_id.clone();
    let visible_label = item.label.clone();
    let value_for_semantics = item.value.clone();
    let radio_value = item.value.clone();
    let value_for_control = item.value.clone();

    let labelled_by_element = if let (Some(control_id), Some(control_registry)) = (
        item_control_id.as_ref(),
        control_registry_for_register.as_ref(),
    ) {
        cx.app
            .models()
            .read(control_registry, |reg| {
                reg.label_for(cx.window, control_id).map(|l| l.element)
            })
            .ok()
            .flatten()
    } else {
        None
    };
    let described_by_element = if let (Some(control_id), Some(control_registry)) = (
        item_control_id.as_ref(),
        control_registry_for_register.as_ref(),
    ) {
        cx.app
            .models()
            .read(control_registry, |reg| {
                reg.described_by_for(cx.window, control_id)
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let radius = Px((icon.0 * 0.5).max(0.0));
    let mut ring_style = decl_style::focus_ring(&theme, radius);
    // Upstream shadcn radio-group uses `focus-visible:ring-[3px]` (no offset).
    ring_style.width = theme
        .metric_by_key("component.radio_group.focus_ring_width_px")
        .unwrap_or(Px(3.0));
    ring_style.offset = theme
        .metric_by_key("component.radio_group.focus_ring_offset_px")
        .unwrap_or(Px(0.0));
    if ring_style.offset.0 <= 0.0 {
        ring_style.offset_color = None;
    }
    if aria_invalid {
        ring_style.color = crate::theme_variants::invalid_control_ring_color(
            &theme,
            theme.color_token("destructive"),
        );
    }

    let mut element = radio_group_prim::RadioGroupItem::new(radio_value)
        .label(item.label.clone())
        .disabled(!item_enabled)
        .index(item.idx)
        .tab_stop(tab_stop)
        .set_size(set_size)
        .into_element_with_props_hook(
            cx,
            &root_for_item,
            PressableProps {
                layout: match render_mode {
                    RadioGroupRenderMode::ControlOnly => LayoutStyle::default(),
                    RadioGroupRenderMode::FullItem => {
                        let has_custom_children = item_children.is_some();
                        if !fit_width
                            || (has_custom_children
                                && orientation == RadioGroupOrientation::Vertical
                                && matches!(item_variant, RadioGroupItemVariant::Default))
                        {
                            pressable_layout_full
                        } else {
                            LayoutStyle::default()
                        }
                    }
                },
                enabled: item_enabled,
                focusable: tab_stop,
                focus_ring: Some(ring_style),
                focus_ring_bounds: match render_mode {
                    RadioGroupRenderMode::ControlOnly => Some(Rect::new(
                        Point::new(Px(0.0), Px(0.0)),
                        Size::new(icon, icon),
                    )),
                    RadioGroupRenderMode::FullItem => match item_variant {
                        RadioGroupItemVariant::Default => Some(Rect::new(
                            Point::new(Px(0.0), Px(0.0)),
                            Size::new(icon, icon),
                        )),
                        RadioGroupItemVariant::ChoiceCard => None,
                    },
                },
                ..Default::default()
            },
            move |cx, st, id, checked, props| {
                if let Some(test_id) = item_test_id.clone() {
                    props.a11y.test_id = Some(test_id);
                }
                if let (Some(control_id), Some(control_registry)) = (
                    item_control_id.clone(),
                    control_registry_for_register.clone(),
                ) {
                    let entry = ControlEntry {
                        element: id,
                        enabled: item_enabled,
                        action: ControlAction::SetOptionalArcStr(
                            model_for_control.clone(),
                            value_for_control.clone(),
                        ),
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }
                if tab_stop
                    && let (Some(control_id), Some(control_registry)) = (
                        control_id_for_register.clone(),
                        control_registry_for_register.clone(),
                    )
                {
                    let entry = ControlEntry {
                        element: id,
                        enabled: item_enabled,
                        action: ControlAction::Noop,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }

                let theme = Theme::global(&*cx.app).snapshot();
                let theme_for_icon = theme.clone();

                let mut states = WidgetStates::from_pressable(cx, st, item_enabled);
                states.set(WidgetState::Selected, checked);

                let border_color = resolve_override_slot(
                    style_override.icon_border_color.as_ref(),
                    &default_icon_border_color,
                    states,
                )
                .resolve(&theme);
                let border_color = if aria_invalid {
                    let destructive = theme.color_token("destructive");
                    if item_enabled {
                        destructive
                    } else {
                        alpha_mul(destructive, 0.5)
                    }
                } else {
                    border_color
                };
                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let focus_visible = states.contains(WidgetStates::FOCUS_VISIBLE);
                let ring_alpha = drive_tween_f32_for_element(
                    cx,
                    id,
                    "radio_group.item.ring.alpha",
                    if focus_visible { 1.0 } else { 0.0 },
                    duration,
                    overlay_motion::shadcn_ease,
                );
                props.focus_ring_always_paint = ring_alpha.animating;

                let ring_style = if ring_alpha.animating {
                    let mut ring_style = ring_style;
                    ring_style.color.a = (ring_style.color.a * ring_alpha.value).clamp(0.0, 1.0);
                    if let Some(offset_color) = ring_style.offset_color {
                        ring_style.offset_color = Some(Color {
                            a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                            ..offset_color
                        });
                    }
                    ring_style
                } else {
                    ring_style
                };
                props.focus_ring = Some(ring_style);

                let border_color = drive_tween_color_for_element(
                    cx,
                    id,
                    "radio_group.item.icon.border",
                    border_color,
                    duration,
                    overlay_motion::shadcn_ease,
                )
                .value;
                let fg = resolve_override_slot(
                    style_override.label_color.as_ref(),
                    &default_label_color,
                    states,
                )
                .resolve(&theme);
                let dot = resolve_override_slot(
                    style_override.indicator_color.as_ref(),
                    &default_indicator_color,
                    states,
                )
                .resolve(&theme);

                let has_custom_children = item_children.is_some();
                let icon_layout = decl_style::layout_style(
                    &theme,
                    if matches!(render_mode, RadioGroupRenderMode::FullItem) && has_custom_children
                    {
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(icon)
                            .h_px(icon)
                            .mt_px(Px(1.0))
                    } else {
                        fret_ui_kit::LayoutRefinement::default()
                            .w_px(icon)
                            .h_px(icon)
                    },
                );
                // Upstream shadcn radio-group uses `dark:bg-input/30` for the icon chrome.
                let icon_bg = theme
                    .color_by_key("component.input.bg")
                    .unwrap_or(Color::TRANSPARENT);
                let icon_props = ContainerProps {
                    layout: icon_layout,
                    padding: Edges::all(Px(0.0)).into(),
                    background: Some(icon_bg),
                    shadow: Some(decl_style::shadow_xs(&theme, radius)),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                };

                let indicator_layout = decl_style::layout_style(
                    &theme,
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(indicator)
                        .h_px(indicator),
                );
                let indicator_props = ContainerProps {
                    layout: indicator_layout,
                    padding: Edges::all(Px(0.0)).into(),
                    background: Some(dot),
                    shadow: None,
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(Px((indicator.0 * 0.5).max(0.0))),
                    ..Default::default()
                };

                let icon_element = cx.container(icon_props, move |cx| {
                    if !checked {
                        return Vec::new();
                    }

                    vec![cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(
                                &theme_for_icon,
                                fret_ui_kit::LayoutRefinement::default().size_full(),
                            ),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| vec![cx.container(indicator_props, |_cx| Vec::new())],
                    )]
                });

                if matches!(render_mode, RadioGroupRenderMode::ControlOnly) {
                    return vec![icon_element];
                }

                let force_full_row = is_rtl
                    && orientation == RadioGroupOrientation::Vertical
                    && item_variant == RadioGroupItemVariant::Default;
                let row_layout = LayoutStyle {
                    size: SizeStyle {
                        width: if fit_width && !force_full_row {
                            Length::Auto
                        } else {
                            Length::Fill
                        },
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let label_props = TextProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                    ),
                    text: visible_label.clone(),
                    style: Some(text_style.clone()),
                    color: Some(fg),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                };

                let justify = if is_rtl && item_variant == RadioGroupItemVariant::Default {
                    MainAlign::End
                } else {
                    MainAlign::Start
                };

                let align = if has_custom_children {
                    CrossAlign::Start
                } else {
                    CrossAlign::Center
                };

                let item_content = cx.flex(
                    FlexProps {
                        layout: row_layout,
                        direction: fret_core::Axis::Horizontal,
                        gap: gap_x.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify,
                        align,
                        wrap: false,
                    },
                    move |cx| {
                        let mut out = Vec::new();
                        let mut label_children = if let Some(children) = item_children {
                            children
                        } else {
                            vec![cx.text_props(label_props)]
                        };

                        match item_variant {
                            RadioGroupItemVariant::Default => {
                                if is_rtl {
                                    out.append(&mut label_children);
                                    out.push(icon_element);
                                } else {
                                    out.push(icon_element);
                                    out.append(&mut label_children);
                                }
                            }
                            RadioGroupItemVariant::ChoiceCard => {
                                out.append(&mut label_children);
                                out.push(icon_element);
                            }
                        }

                        out
                    },
                );

                match item_variant {
                    RadioGroupItemVariant::Default => vec![item_content],
                    RadioGroupItemVariant::ChoiceCard => {
                        let primary = radio_indicator(&theme);
                        let checked_bg = crate::theme_variants::radio_group_choice_card_checked_bg(
                            &theme, primary,
                        );
                        let border = radio_border(&theme);

                        let mut chrome = ChromeRefinement::default()
                            .p_4()
                            .rounded_md()
                            .border_1()
                            .border_color(ColorRef::Color(border));
                        if checked {
                            chrome = chrome
                                .bg(ColorRef::Color(checked_bg))
                                .border_color(ColorRef::Color(primary));
                        }

                        let container = decl_style::container_props(
                            &theme,
                            chrome,
                            fret_ui_kit::LayoutRefinement::default().w_full(),
                        );
                        vec![cx.container(container, move |_cx| vec![item_content])]
                    }
                }
            },
        );

    if labelled_by_element.is_some() || described_by_element.is_some() {
        let mut decoration = SemanticsDecoration::default();
        if let Some(labelled_by) = labelled_by_element {
            decoration.labelled_by_element = Some(labelled_by.0);
        }
        if let Some(desc) = described_by_element {
            decoration.described_by_element = Some(desc.0);
        }
        element = element.attach_semantics(decoration);
    }

    debug_assert!(
        !value_for_semantics.is_empty(),
        "radio-group item values must not be empty"
    );

    element
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioGroupItemVariant {
    #[default]
    Default,
    ChoiceCard,
}

#[derive(Debug)]
pub struct RadioGroupItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub children: Option<Vec<AnyElement>>,
    pub control_id: Option<ControlId>,
    pub disabled: bool,
    pub aria_invalid: bool,
    pub variant: RadioGroupItemVariant,
}

impl RadioGroupItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: None,
            control_id: None,
            disabled: false,
            aria_invalid: false,
            variant: RadioGroupItemVariant::default(),
        }
    }

    /// Overrides the default item contents (icon + label text) to enable composable labels.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn child(mut self, child: AnyElement) -> Self {
        self.children = Some(vec![child]);
        self
    }

    /// Binds this radio item to a logical form control id so `FieldLabel::for_control(...)`
    /// can target a specific item instead of only the group's current tab stop.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Apply the upstream `aria-invalid` error state chrome (border + focus ring color).
    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn variant(mut self, variant: RadioGroupItemVariant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct RadioGroupStyle {
    pub icon_border_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub indicator_color: OverrideSlot<ColorRef>,
}

impl RadioGroupStyle {
    pub fn icon_border_color(
        mut self,
        icon_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.icon_border_color = Some(icon_border_color);
        self
    }

    pub fn label_color(mut self, label_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(label_color);
        self
    }

    pub fn indicator_color(
        mut self,
        indicator_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.indicator_color = Some(indicator_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.icon_border_color.is_some() {
            self.icon_border_color = other.icon_border_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.indicator_color.is_some() {
            self.indicator_color = other.indicator_color;
        }
        self
    }
}

pub struct RadioGroup {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<RadioGroupItem>,
    disabled: bool,
    required: bool,
    control_id: Option<ControlId>,
    a11y_label: Option<Arc<str>>,
    test_id_prefix: Option<Arc<str>>,
    orientation: RadioGroupOrientation,
    loop_navigation: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: RadioGroupStyle,
}

impl RadioGroup {
    pub fn new(model: impl IntoOptionalTextValueModel) -> Self {
        Self {
            model: Some(model.into_optional_text_value_model()),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            required: false,
            control_id: None,
            a11y_label: None,
            test_id_prefix: None,
            orientation: RadioGroupOrientation::default(),
            loop_navigation: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: RadioGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled radio group with an optional initial value (Radix `defaultValue`).
    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            required: false,
            control_id: None,
            a11y_label: None,
            test_id_prefix: None,
            orientation: RadioGroupOrientation::default(),
            loop_navigation: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: RadioGroupStyle::default(),
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Binds this RadioGroup to a logical form control id (similar to HTML `id`).
    ///
    /// When set, `Label::for_control(ControlId)` forwards focus to the active radio item, and the
    /// group uses `aria-labelledby` / `aria-describedby`-like semantics via the control registry.
    ///
    /// Use [`RadioGroupItem::control_id`] when labels should target specific radio items.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Sets a stable `test_id` prefix for radio items (useful for diag scripts).
    ///
    /// Items use the shape `{prefix}-item-{idx}`.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    /// Sets the uncontrolled initial selection value (Radix `defaultValue`).
    ///
    /// Note: If a controlled `model` is provided, this value is ignored.
    pub fn default_value<T: Into<Arc<str>>>(mut self, default_value: Option<T>) -> Self {
        self.default_value = default_value.map(Into::into);
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: RadioGroupStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            model,
            default_value,
            items,
            disabled,
            required,
            control_id,
            a11y_label,
            test_id_prefix,
            orientation,
            loop_navigation,
            chrome,
            layout,
            style,
        } = self;
        let has_item_control_ids = items.iter().any(|item| item.control_id.is_some());

        cx.scope(|cx| {
            let control_id = control_id.clone();
            let control_registry = (control_id.is_some() || has_item_control_ids)
                .then(|| control_registry_model(cx));
            let labelled_by_element = if a11y_label.is_some() {
                None
            } else if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.label_for(cx.window, control_id).map(|l| l.element)
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };
            let described_by_element = if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| reg.described_by_for(cx.window, control_id))
                    .ok()
                    .flatten()
            } else {
                None
            };

            let theme = Theme::global(&*cx.app).snapshot();
            let gap_y = row_gap(&theme);
            let gap_x = label_gap(&theme);
            let icon = icon_size(&theme);
            let indicator = indicator_size(&theme);

            let text_style = radio_text_style(&theme);
            let fg = radio_fg(&theme);
            let border = radio_border(&theme);
            let ring = radio_ring(&theme);
            let dot = radio_indicator(&theme);

            let default_icon_border_color = WidgetStateProperty::new(ColorRef::Color(border))
                .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring))
                .when(
                    WidgetStates::DISABLED,
                    ColorRef::Color(alpha_mul(border, 0.5)),
                );

            let default_label_color = WidgetStateProperty::new(ColorRef::Color(fg)).when(
                WidgetStates::DISABLED,
                // shadcn v4 `Label`: `peer-disabled:opacity-50`
                ColorRef::Color(alpha_mul(fg, 0.5)),
            );

            let default_indicator_color = WidgetStateProperty::new(ColorRef::Color(dot))
                .when(WidgetStates::DISABLED, ColorRef::Color(alpha_mul(dot, 0.5)));

            let group_disabled = disabled;
            let group_required = required;
            let group_label = a11y_label.clone();
            let test_id_prefix = test_id_prefix.clone();
            let style_override = style;
            let model = radio_group_prim::radio_group_use_model(cx, model.clone(), || {
                default_value.clone()
            })
            .model();
            let is_rtl = crate::direction::use_direction(cx, None)
                == LayoutDirection::Rtl;

            let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().flatten();
            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            let disabled_by_idx: Vec<bool> =
                items.iter().map(|i| group_disabled || i.disabled).collect();
            let active = roving_focus_group::active_index_from_str_keys(
                &values,
                selected.as_deref(),
                &disabled_by_idx,
            );

            let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
            let disabled_arc: Arc<[bool]> = Arc::from(disabled_by_idx.clone().into_boxed_slice());
            let set_size = u32::try_from(items.len())
                .ok()
                .and_then(|n| (n > 0).then_some(n));

            let mut radix_root = radio_group_prim::RadioGroupRoot::new(model.clone())
                .disabled(group_disabled)
                .required(group_required)
                .orientation(orientation)
                .loop_navigation(loop_navigation);
            if let Some(label) = group_label.clone() {
                radix_root = radix_root.a11y_label(label);
            }

            let root_for_items = radix_root.clone();
            let list = radix_root.list(values_arc.clone(), disabled_arc.clone());
            let control_id_for_register = control_id.clone();
            let control_registry_for_register = control_registry.clone();

            let container_props = decl_style::container_props(&theme, chrome, layout);
            let fit_width = matches!(container_props.layout.size.width, Length::Auto);
            let list_layout = if fit_width {
                LayoutStyle::default()
            } else {
                decl_style::layout_style(&theme, fret_ui_kit::LayoutRefinement::default().w_full())
            };
            let pressable_layout_full =
                decl_style::layout_style(&theme, fret_ui_kit::LayoutRefinement::default().w_full());

            let list_element = list.into_element(
                cx,
                RovingFlexProps {
                    flex: FlexProps {
                        layout: list_layout,
                        direction: match orientation {
                            RadioGroupOrientation::Vertical => fret_core::Axis::Vertical,
                            RadioGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
                        },
                        gap: match orientation {
                            RadioGroupOrientation::Vertical => gap_y,
                            RadioGroupOrientation::Horizontal => gap_x,
                        }
                        .into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: match orientation {
                            RadioGroupOrientation::Vertical => {
                                if is_rtl {
                                    CrossAlign::End
                                } else {
                                    CrossAlign::Stretch
                                }
                            }
                            RadioGroupOrientation::Horizontal => CrossAlign::Center,
                        },
                        wrap: false,
                        ..Default::default()
                    },
                    roving: RovingFocusProps::default(),
                },
                move |cx| {
                    let mut out = Vec::with_capacity(items.len());
                    for (idx, item) in items.into_iter().enumerate() {
                        let item_disabled = disabled_by_idx.get(idx).copied().unwrap_or(true);
                        let item_enabled = !item_disabled;
                        let tab_stop = active.is_some_and(|a| a == idx);
                        let aria_invalid = item.aria_invalid;
                        let item_variant = item.variant;
                        let item_control_id = item.control_id;
                        let control_id_for_register = control_id_for_register.clone();
                        let control_registry_for_register = control_registry_for_register.clone();
                        let tab_stop_for_register = tab_stop;
                        let item_test_id = test_id_prefix.as_ref().map(|p| {
                            Arc::<str>::from(format!("{p}-item-{idx}"))
                        });

                        let radius = Px((icon.0 * 0.5).max(0.0));
                        let mut ring_style = decl_style::focus_ring(&theme, radius);
                        // Upstream shadcn radio-group uses `focus-visible:ring-[3px]` (no offset).
                        ring_style.width = theme
                            .metric_by_key("component.radio_group.focus_ring_width_px")
                            .unwrap_or(Px(3.0));
                        ring_style.offset = theme
                            .metric_by_key("component.radio_group.focus_ring_offset_px")
                            .unwrap_or(Px(0.0));
                        if ring_style.offset.0 <= 0.0 {
                            ring_style.offset_color = None;
                        }
                        if aria_invalid {
                            ring_style.color = crate::theme_variants::invalid_control_ring_color(
                                &theme,
                                theme.color_token("destructive"),
                            );
                        }
                        let a11y_label = item.label;
                        let value = item.value;
                        let item_children = item.children;
                        let has_custom_children = item_children.is_some();
                        let text_style = text_style.clone();
                        let root_for_item = root_for_items.clone();
                        let style_override = style_override.clone();
                        let default_icon_border_color = default_icon_border_color.clone();
                        let default_label_color = default_label_color.clone();
                        let default_indicator_color = default_indicator_color.clone();
                        let key = value.clone();
                        let radio_value = value.clone();
                        let value_for_control = value.clone();
                        let model_for_control = model.clone();
                        out.push(cx.keyed(key, move |cx| {
                            let base_ring_style = ring_style;
                            let focus_ring_for_props = base_ring_style;
                            radio_group_prim::RadioGroupItem::new(radio_value)
                                .label(a11y_label.clone())
                                .disabled(!item_enabled)
                                .index(idx)
                                .tab_stop(tab_stop)
                                .set_size(set_size)
                                .into_element_with_props_hook(
                                    cx,
                                    &root_for_item,
                                    PressableProps {
                                        layout: if !fit_width
                                            || (has_custom_children
                                                && orientation == RadioGroupOrientation::Vertical
                                                && matches!(
                                                    item_variant,
                                                    RadioGroupItemVariant::Default
                                                )) {
                                            pressable_layout_full
                                        } else {
                                            LayoutStyle::default()
                                        },
                                        enabled: item_enabled,
                                        focusable: tab_stop,
                                        focus_ring: Some(focus_ring_for_props),
                                        focus_ring_bounds: match item_variant {
                                            RadioGroupItemVariant::Default => Some(Rect::new(
                                                Point::new(Px(0.0), Px(0.0)),
                                                Size::new(icon, icon),
                                            )),
                                            RadioGroupItemVariant::ChoiceCard => None,
                                        },
                                        ..Default::default()
                                    },
                                    move |cx, st, id, checked, props| {
                                        if let Some(test_id) = item_test_id.clone() {
                                            props.a11y.test_id = Some(test_id);
                                        }
                                        if let (Some(control_id), Some(control_registry)) = (
                                            item_control_id.clone(),
                                            control_registry_for_register.clone(),
                                        ) {
                                            let entry = ControlEntry {
                                                element: id,
                                                enabled: item_enabled,
                                                action: ControlAction::SetOptionalArcStr(
                                                    model_for_control.clone(),
                                                    value_for_control.clone(),
                                                ),
                                            };
                                            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                                                reg.register_control(cx.window, cx.frame_id, control_id, entry);
                                            });
                                        }
                                        if tab_stop_for_register
                                            && let (Some(control_id), Some(control_registry)) = (
                                                control_id_for_register.clone(),
                                                control_registry_for_register.clone(),
                                            )
                                        {
                                            let entry = ControlEntry {
                                                element: id,
                                                enabled: item_enabled,
                                                action: ControlAction::Noop,
                                            };
                                            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                                                reg.register_control(cx.window, cx.frame_id, control_id, entry);
                                            });
                                        }

                                        let theme = Theme::global(&*cx.app).snapshot();
                                        let theme_for_icon = theme.clone();

                                        let mut states =
                                            WidgetStates::from_pressable(cx, st, item_enabled);
                                        states.set(WidgetState::Selected, checked);

                                        let border_color = resolve_override_slot(
                                            style_override.icon_border_color.as_ref(),
                                            &default_icon_border_color,
                                            states,
                                        )
                                        .resolve(&theme);
                                        let border_color = if aria_invalid {
                                            let destructive = theme.color_token("destructive");
                                            if item_enabled {
                                                destructive
                                            } else {
                                                alpha_mul(destructive, 0.5)
                                            }
                                        } else {
                                            border_color
                                        };
                                        let duration = overlay_motion::shadcn_motion_duration_150(cx);
                                        let focus_visible =
                                            states.contains(WidgetStates::FOCUS_VISIBLE);
                                        let ring_alpha = drive_tween_f32_for_element(
                                            cx,
                                            id,
                                            "radio_group.item.ring.alpha",
                                            if focus_visible { 1.0 } else { 0.0 },
                                            duration,
                                            overlay_motion::shadcn_ease,
                                        );
                                        props.focus_ring_always_paint = ring_alpha.animating;

                                        // Keep the steady-state focus ring paint-time (focus +
                                        // focus-visible) so focus changes do not depend on a
                                        // re-render between layout and paint.
                                        //
                                        // When we are animating (in/out), apply alpha scaling and
                                        // enable `focus_ring_always_paint` so the ring can fade out
                                        // after blur.
                                        let ring_style = if ring_alpha.animating {
                                            let mut ring_style = base_ring_style;
                                            ring_style.color.a = (ring_style.color.a
                                                * ring_alpha.value)
                                                .clamp(0.0, 1.0);
                                            if let Some(offset_color) = ring_style.offset_color {
                                                ring_style.offset_color = Some(Color {
                                                    a: (offset_color.a * ring_alpha.value)
                                                        .clamp(0.0, 1.0),
                                                    ..offset_color
                                                });
                                            }
                                            ring_style
                                        } else {
                                            base_ring_style
                                        };
                                        props.focus_ring = Some(ring_style);

                                        let border_color = drive_tween_color_for_element(
                                            cx,
                                            id,
                                            "radio_group.item.icon.border",
                                            border_color,
                                            duration,
                                            overlay_motion::shadcn_ease,
                                        )
                                        .value;
                                        let fg = resolve_override_slot(
                                            style_override.label_color.as_ref(),
                                            &default_label_color,
                                            states,
                                        )
                                        .resolve(&theme);
                                        let dot = resolve_override_slot(
                                            style_override.indicator_color.as_ref(),
                                            &default_indicator_color,
                                            states,
                                        )
                                        .resolve(&theme);

                                        let has_custom_children = item_children.is_some();
                                        let icon_layout = decl_style::layout_style(
                                            &theme,
                                            if has_custom_children {
                                                fret_ui_kit::LayoutRefinement::default()
                                                    .w_px(icon)
                                                    .h_px(icon)
                                                    .mt_px(Px(1.0))
                                            } else {
                                                fret_ui_kit::LayoutRefinement::default()
                                                    .w_px(icon)
                                                    .h_px(icon)
                                            },
                                        );
                                        // Upstream shadcn radio-group uses `dark:bg-input/30` for the icon chrome.
                                        let icon_bg = theme
                                            .color_by_key("component.input.bg")
                                            .unwrap_or(Color::TRANSPARENT);
                                        let icon_props = ContainerProps {
                                            layout: icon_layout,
                                            padding: Edges::all(Px(0.0)).into(),
                                            background: Some(icon_bg),
                                            shadow: Some(decl_style::shadow_xs(&theme, radius)),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border_color),
                                            corner_radii: Corners::all(radius),
                                            ..Default::default()
                                        };

                                        let force_full_row = is_rtl
                                            && orientation == RadioGroupOrientation::Vertical
                                            && matches!(
                                                item_variant,
                                                RadioGroupItemVariant::Default
                                            );
                                        let row_layout = LayoutStyle {
                                            size: SizeStyle {
                                                width: if fit_width && !force_full_row {
                                                    Length::Auto
                                                } else {
                                                    Length::Fill
                                                },
                                                height: Length::Auto,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };

                                        let indicator_layout = decl_style::layout_style(
                                            &theme,
                                            fret_ui_kit::LayoutRefinement::default()
                                                .w_px(indicator)
                                                .h_px(indicator),
                                        );
                                        let indicator_props = ContainerProps {
                                            layout: indicator_layout,
                                            padding: Edges::all(Px(0.0)).into(),
                                            background: Some(dot),
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(Px(
                                                (indicator.0 * 0.5).max(0.0)
                                            )),
                                            ..Default::default()
                                        };

                                        let label = a11y_label.clone();
                                        let label_props = TextProps {
                                            layout: decl_style::layout_style(
                                                &theme,
                                                LayoutRefinement::default()
                                                    .flex_grow(1.0)
                                                    .min_w_0(),
                                            ),
                                            text: label,
                                            style: Some(text_style.clone()),
                                            color: Some(fg),
                                            wrap: TextWrap::Word,
                                            overflow: TextOverflow::Clip,
                                            align: fret_core::TextAlign::Start,
                                            ink_overflow: Default::default(),
                                        };

                                        let icon_element =
                                            cx.container(icon_props, move |cx| {
                                                if !checked {
                                                    return Vec::new();
                                                }

                                                vec![cx.flex(
                                                FlexProps {
                                                    layout: decl_style::layout_style(
                                                        &theme_for_icon,
                                                        fret_ui_kit::LayoutRefinement::default()
                                                            .size_full(),
                                                    ),
                                                    direction: fret_core::Axis::Horizontal,
                                                    gap: Px(0.0).into(),
                                                    padding: Edges::all(Px(0.0)).into(),
                                                    justify: MainAlign::Center,
                                                    align: CrossAlign::Center,
                                                    wrap: false,
                                                },
                                                move |cx| {
                                                    vec![cx.container(
                                                        indicator_props,
                                                        |_cx| Vec::new(),
                                                    )]
                                                },
                                            )]
                                            });

                                        let justify = if is_rtl
                                            && matches!(
                                                item_variant,
                                                RadioGroupItemVariant::Default
                                            ) {
                                            MainAlign::End
                                        } else {
                                            MainAlign::Start
                                        };

                                        let align = if has_custom_children {
                                            CrossAlign::Start
                                        } else {
                                            CrossAlign::Center
                                        };
                                        let item_content = cx.flex(
                                            FlexProps {
                                                layout: row_layout,
                                                direction: fret_core::Axis::Horizontal,
                                                gap: gap_x.into(),
                                                padding: Edges::all(Px(0.0)).into(),
                                                justify,
                                                align,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let mut out = Vec::new();
                                                let mut label_children =
                                                    if let Some(children) = item_children {
                                                        children
                                                    } else {
                                                        vec![cx.text_props(label_props)]
                                                    };

                                                match item_variant {
                                                    RadioGroupItemVariant::Default => {
                                                        if is_rtl {
                                                            out.append(&mut label_children);
                                                            out.push(icon_element);
                                                        } else {
                                                            out.push(icon_element);
                                                            out.append(&mut label_children);
                                                        }
                                                    }
                                                    RadioGroupItemVariant::ChoiceCard => {
                                                        out.append(&mut label_children);
                                                        out.push(icon_element);
                                                    }
                                                };

                                                out
                                            },
                                        );

                                        match item_variant {
                                            RadioGroupItemVariant::Default => vec![item_content],
                                            RadioGroupItemVariant::ChoiceCard => {
                                                let primary = radio_indicator(&theme);
                                                let checked_bg =
                                                    crate::theme_variants::radio_group_choice_card_checked_bg(
                                                        &theme,
                                                        primary,
                                                    );
                                                let border = radio_border(&theme);

                                                let mut chrome = ChromeRefinement::default()
                                                    .p_4()
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(ColorRef::Color(border));
                                                if checked {
                                                    chrome = chrome
                                                        .bg(ColorRef::Color(checked_bg))
                                                        .border_color(ColorRef::Color(primary));
                                                }

                                                let container = decl_style::container_props(
                                                    &theme,
                                                    chrome,
                                                    fret_ui_kit::LayoutRefinement::default()
                                                        .w_full(),
                                                );
                                                vec![cx.container(container, move |_cx| {
                                                    vec![item_content]
                                                })]
                                            }
                                        }
                                    },
                                )
                        }));
                    }
                    out
                },
            );
            let list_element = if labelled_by_element.is_some() || described_by_element.is_some() {
                let mut decoration = SemanticsDecoration::default();
                if a11y_label.is_none()
                    && let Some(labelled_by) = labelled_by_element {
                        decoration.labelled_by_element = Some(labelled_by.0);
                    }
                if let Some(desc) = described_by_element {
                    decoration.described_by_element = Some(desc.0);
                }
                list_element.attach_semantics(decoration)
            } else {
                list_element
            };

            cx.container(container_props, move |_cx| vec![list_element])
        })
    }

    /// Renders the group using the already-declared item metadata while letting callers compose
    /// each row around `parts.control(...)`.
    ///
    /// This is the typed docs-parity lane for examples that need external `Field`,
    /// `FieldLabel::for_control(...)`, or `FieldDescription` layout around the radio control
    /// without widening `RadioGroup` to a generic root children API.
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>, &RadioGroupParts) -> Vec<AnyElement>,
    ) -> AnyElement {
        let Self {
            model,
            default_value,
            items,
            disabled,
            required,
            control_id,
            a11y_label,
            test_id_prefix,
            orientation,
            loop_navigation,
            chrome,
            layout,
            style,
        } = self;
        let has_item_control_ids = items.iter().any(|item| item.control_id.is_some());

        cx.scope(|cx| {
            let control_id = control_id.clone();
            let control_registry = (control_id.is_some() || has_item_control_ids)
                .then(|| control_registry_model(cx));
            let labelled_by_element = if a11y_label.is_some() {
                None
            } else if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.label_for(cx.window, control_id).map(|l| l.element)
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };
            let described_by_element = if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| reg.described_by_for(cx.window, control_id))
                    .ok()
                    .flatten()
            } else {
                None
            };

            let theme = Theme::global(&*cx.app).snapshot();
            let gap_y = row_gap(&theme);
            let gap_x = label_gap(&theme);
            let icon = icon_size(&theme);
            let indicator = indicator_size(&theme);
            let text_style = radio_text_style(&theme);
            let fg = radio_fg(&theme);
            let border = radio_border(&theme);
            let ring = radio_ring(&theme);
            let dot = radio_indicator(&theme);

            let default_icon_border_color = WidgetStateProperty::new(ColorRef::Color(border))
                .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring))
                .when(
                    WidgetStates::DISABLED,
                    ColorRef::Color(alpha_mul(border, 0.5)),
                );
            let default_label_color = WidgetStateProperty::new(ColorRef::Color(fg)).when(
                WidgetStates::DISABLED,
                ColorRef::Color(alpha_mul(fg, 0.5)),
            );
            let default_indicator_color = WidgetStateProperty::new(ColorRef::Color(dot))
                .when(WidgetStates::DISABLED, ColorRef::Color(alpha_mul(dot, 0.5)));

            let group_disabled = disabled;
            let group_required = required;
            let group_label = a11y_label.clone();
            let test_id_prefix = test_id_prefix.clone();
            let style_override = style;
            let model = radio_group_prim::radio_group_use_model(cx, model.clone(), || {
                default_value.clone()
            })
            .model();
            let is_rtl = crate::direction::use_direction(cx, None) == LayoutDirection::Rtl;

            let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().flatten();
            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            let disabled_by_idx: Vec<bool> =
                items.iter().map(|i| group_disabled || i.disabled).collect();
            let active = roving_focus_group::active_index_from_str_keys(
                &values,
                selected.as_deref(),
                &disabled_by_idx,
            );

            let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
            let disabled_arc: Arc<[bool]> = Arc::from(disabled_by_idx.clone().into_boxed_slice());
            let set_size = u32::try_from(items.len())
                .ok()
                .and_then(|n| (n > 0).then_some(n));

            let mut radix_root = radio_group_prim::RadioGroupRoot::new(model.clone())
                .disabled(group_disabled)
                .required(group_required)
                .orientation(orientation)
                .loop_navigation(loop_navigation);
            if let Some(label) = group_label.clone() {
                radix_root = radix_root.a11y_label(label);
            }

            let list = radix_root
                .clone()
                .list(values_arc.clone(), disabled_arc.clone());
            let control_id_for_register = control_id.clone();
            let control_registry_for_register = control_registry.clone();

            let container_props = decl_style::container_props(&theme, chrome, layout);
            let fit_width = matches!(container_props.layout.size.width, Length::Auto);
            let list_layout = if fit_width {
                LayoutStyle::default()
            } else {
                decl_style::layout_style(&theme, fret_ui_kit::LayoutRefinement::default().w_full())
            };

            let render = RadioGroupRenderContext {
                theme: theme.clone(),
                fit_width,
                is_rtl,
                orientation,
                root: radix_root.clone(),
                model: model.clone(),
                control_id_for_register,
                control_registry_for_register,
                set_size,
                gap_x,
                icon,
                indicator,
                text_style,
                style_override,
                default_icon_border_color,
                default_label_color,
                default_indicator_color,
                pressable_layout_full: decl_style::layout_style(
                    &theme,
                    fret_ui_kit::LayoutRefinement::default().w_full(),
                ),
            };

            let parts_items: Vec<RadioGroupPartsItem> = items
                .iter()
                .enumerate()
                .map(|(idx, item)| RadioGroupPartsItem {
                    value: item.value.clone(),
                    label: item.label.clone(),
                    control_id: item.control_id.clone(),
                    enabled: !disabled_by_idx.get(idx).copied().unwrap_or(true),
                    aria_invalid: item.aria_invalid,
                    idx,
                    tab_stop: active.is_some_and(|a| a == idx),
                    item_test_id: test_id_prefix
                        .as_ref()
                        .map(|p| Arc::<str>::from(format!("{p}-item-{idx}"))),
                })
                .collect();

            let by_value: HashMap<Arc<str>, usize> = parts_items
                .iter()
                .enumerate()
                .map(|(idx, item)| (item.value.clone(), idx))
                .collect();
            let parts = RadioGroupParts {
                render,
                items: Arc::from(parts_items.into_boxed_slice()),
                by_value: Arc::new(by_value),
                next_expected_index: Rc::new(Cell::new(0)),
            };

            let parts_for_children = parts.clone();
            let list_element = list.into_element(
                cx,
                RovingFlexProps {
                    flex: FlexProps {
                        layout: list_layout,
                        direction: match orientation {
                            RadioGroupOrientation::Vertical => fret_core::Axis::Vertical,
                            RadioGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
                        },
                        gap: match orientation {
                            RadioGroupOrientation::Vertical => gap_y,
                            RadioGroupOrientation::Horizontal => gap_x,
                        }
                        .into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: match orientation {
                            RadioGroupOrientation::Vertical => {
                                if is_rtl {
                                    CrossAlign::End
                                } else {
                                    CrossAlign::Stretch
                                }
                            }
                            RadioGroupOrientation::Horizontal => CrossAlign::Center,
                        },
                        wrap: false,
                        ..Default::default()
                    },
                    roving: RovingFocusProps::default(),
                },
                move |cx| {
                    let children = f(cx, &parts_for_children);
                    debug_assert_eq!(
                        parts_for_children.next_expected_index.get(),
                        parts_for_children.items.len(),
                        "radio-group into_element_parts must render each declared item exactly once and in order"
                    );
                    children
                },
            );
            let list_element = if labelled_by_element.is_some() || described_by_element.is_some() {
                let mut decoration = SemanticsDecoration::default();
                if a11y_label.is_none()
                    && let Some(labelled_by) = labelled_by_element {
                        decoration.labelled_by_element = Some(labelled_by.0);
                    }
                if let Some(desc) = described_by_element {
                    decoration.described_by_element = Some(desc.0);
                }
                list_element.attach_semantics(decoration)
            } else {
                list_element
            };

            cx.container(container_props, move |_cx| vec![list_element])
        })
    }
}

/// Builder-preserving controlled helper for the common radio-group authoring path.
pub fn radio_group(
    model: impl IntoOptionalTextValueModel,
    items: Vec<RadioGroupItem>,
) -> RadioGroup {
    let mut group = RadioGroup::new(model);
    for item in items {
        group = group.item(item);
    }
    group
}

/// Builder-preserving uncontrolled helper for the common `defaultValue` authoring path.
pub fn radio_group_uncontrolled<T: Into<Arc<str>>>(
    default_value: Option<T>,
    items: Vec<RadioGroupItem>,
) -> RadioGroup {
    let mut group = RadioGroup::uncontrolled(default_value);
    for item in items {
        group = group.item(item);
    }
    group
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, PathCommand, SemanticsRole, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{Event, KeyCode};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_runtime::FrameId;
    use fret_ui::element::{ElementKind, PressableProps};
    use fret_ui::{Theme, ThemeConfig, UiTree};
    use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
    use fret_ui_kit::primitives::control_registry::ControlId;

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn radio_group_selected_indicator_is_centered_in_icon() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("b")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-indicator-centered",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Options")
                        .item(RadioGroupItem::new("a", "Alpha"))
                        .item(RadioGroupItem::new("b", "Beta"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let theme = Theme::global(&app).snapshot();
        let icon = icon_size(&theme);
        let indicator = indicator_size(&theme);
        let dot = radio_indicator(&theme);

        let mut dot_rect: Option<Rect> = None;
        let mut icon_rects: Vec<Rect> = Vec::new();
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                rect,
                background,
                border,
                border_paint,
                ..
            } = op
            else {
                continue;
            };

            let is_dot = (rect.size.width.0 - indicator.0).abs() <= 0.1
                && (rect.size.height.0 - indicator.0).abs() <= 0.1
                && background.paint == fret_core::Paint::Solid(dot);
            if is_dot {
                dot_rect = Some(*rect);
            }

            let is_icon = (rect.size.width.0 - icon.0).abs() <= 0.1
                && (rect.size.height.0 - icon.0).abs() <= 0.1
                && background.paint == fret_core::Paint::TRANSPARENT
                && border.left.0 > 0.0
                && border.top.0 > 0.0
                && border.right.0 > 0.0
                && border.bottom.0 > 0.0
                && matches!(border_paint.paint, fret_core::Paint::Solid(c) if c.a > 0.0);
            if is_icon {
                icon_rects.push(*rect);
            }
        }

        let dot_rect = dot_rect.expect("missing radio indicator dot quad");
        let icon_rect = icon_rects
            .into_iter()
            .find(|r| {
                dot_rect.origin.x.0 >= r.origin.x.0
                    && dot_rect.origin.y.0 >= r.origin.y.0
                    && (dot_rect.origin.x.0 + dot_rect.size.width.0)
                        <= (r.origin.x.0 + r.size.width.0)
                    && (dot_rect.origin.y.0 + dot_rect.size.height.0)
                        <= (r.origin.y.0 + r.size.height.0)
            })
            .expect("missing radio icon quad containing dot");

        let icon_cx = icon_rect.origin.x.0 + icon_rect.size.width.0 * 0.5;
        let icon_cy = icon_rect.origin.y.0 + icon_rect.size.height.0 * 0.5;
        let dot_cx = dot_rect.origin.x.0 + dot_rect.size.width.0 * 0.5;
        let dot_cy = dot_rect.origin.y.0 + dot_rect.size.height.0 * 0.5;

        assert!(
            (dot_cx - icon_cx).abs() <= 0.2,
            "expected dot center_x {dot_cx} close to icon center_x {icon_cx}"
        );
        assert!(
            (dot_cy - icon_cy).abs() <= 0.2,
            "expected dot center_y {dot_cy} close to icon center_y {icon_cy}"
        );
    }

    #[test]
    fn radio_group_item_aria_invalid_uses_destructive_border_color() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("invalid")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-invalid-border",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Options")
                        .item(RadioGroupItem::new("valid", "Valid"))
                        .item(RadioGroupItem::new("invalid", "Invalid").aria_invalid(true))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let theme = Theme::global(&app).snapshot();
        let icon = icon_size(&theme);
        let destructive = theme.color_token("destructive");

        let mut icon_border_colors: Vec<Color> = Vec::new();
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                rect,
                background,
                border,
                border_paint,
                ..
            } = op
            else {
                continue;
            };

            let is_icon = (rect.size.width.0 - icon.0).abs() <= 0.1
                && (rect.size.height.0 - icon.0).abs() <= 0.1
                && background.paint == fret_core::Paint::TRANSPARENT
                && border.left.0 > 0.0
                && border.top.0 > 0.0
                && border.right.0 > 0.0
                && border.bottom.0 > 0.0
                && matches!(border_paint.paint, fret_core::Paint::Solid(c) if c.a > 0.0);
            if is_icon
                && let fret_core::Paint::Solid(border_color) = border_paint.paint {
                    icon_border_colors.push(border_color);
                }
        }

        assert!(
            icon_border_colors.len() >= 2,
            "expected at least 2 radio icon quads (got {})",
            icon_border_colors.len()
        );
        assert!(
            icon_border_colors.contains(&destructive),
            "expected an aria-invalid icon border quad with destructive color"
        );
        assert!(
            icon_border_colors.iter().any(|c| *c != destructive),
            "expected a non-invalid icon border quad that is not destructive"
        );
    }

    #[test]
    fn radio_group_choice_card_applies_checked_background_and_border() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("beta")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-choice-card-checked",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Options")
                        .item(
                            RadioGroupItem::new("beta", "Beta")
                                .variant(RadioGroupItemVariant::ChoiceCard),
                        )
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let theme = Theme::global(&app).snapshot();
        let primary = radio_indicator(&theme);
        let expected_bg =
            crate::theme_variants::radio_group_choice_card_checked_bg(&theme, primary);

        let mut total_quads = 0usize;
        let mut bg_matches = 0usize;
        let mut border_paint_matches = 0usize;
        let mut bg_and_border_paint_matches = 0usize;
        let mut found = false;
        let mut sample_quad = None::<(fret_core::Paint, Edges, fret_core::Paint)>;
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                background,
                border,
                border_paint,
                ..
            } = op
            else {
                continue;
            };

            total_quads = total_quads.saturating_add(1);
            if sample_quad.is_none() {
                sample_quad = Some((background.paint, *border, border_paint.paint));
            }
            if background.paint == fret_core::Paint::Solid(expected_bg) {
                bg_matches = bg_matches.saturating_add(1);
            }
            if border_paint.paint == fret_core::Paint::Solid(primary) {
                border_paint_matches = border_paint_matches.saturating_add(1);
            }
            if background.paint == fret_core::Paint::Solid(expected_bg)
                && border_paint.paint == fret_core::Paint::Solid(primary)
            {
                bg_and_border_paint_matches = bg_and_border_paint_matches.saturating_add(1);
            }

            if background.paint == fret_core::Paint::Solid(expected_bg)
                && border_paint.paint == fret_core::Paint::Solid(primary)
                && border.left.0 > 0.0
                && border.top.0 > 0.0
                && border.right.0 > 0.0
                && border.bottom.0 > 0.0
            {
                found = true;
                break;
            }
        }

        assert!(
            found,
            "missing checked choice-card background/border quad (total_quads={total_quads}, bg_matches={bg_matches}, border_paint_matches={border_paint_matches}, bg_and_border_paint_matches={bg_and_border_paint_matches}, sample_quad={sample_quad:?})"
        );
    }

    #[test]
    fn radio_group_choice_card_items_stretch_to_group_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("pro")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(640.0), Px(360.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-choice-card-stretch-width",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Plans")
                        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                        .item(
                            RadioGroupItem::new("plus", "Plus")
                                .variant(RadioGroupItemVariant::ChoiceCard),
                        )
                        .item(
                            RadioGroupItem::new("pro", "Pro")
                                .variant(RadioGroupItemVariant::ChoiceCard),
                        )
                        .item(
                            RadioGroupItem::new("enterprise", "Enterprise")
                                .variant(RadioGroupItemVariant::ChoiceCard),
                        )
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let group_bounds = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioGroup && n.label.as_deref() == Some("Plans"))
            .map(|n| n.bounds)
            .expect("radio group semantics node");

        let mut item_bounds: Vec<(String, Rect)> = Vec::new();
        for label in ["Plus", "Pro", "Enterprise"] {
            let b = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some(label))
                .map(|n| n.bounds)
                .expect("radio button semantics node");
            item_bounds.push((label.to_string(), b));
        }

        let eps = 0.5;
        for (label, b) in &item_bounds {
            assert!(
                (b.size.width.0 - group_bounds.size.width.0).abs() <= eps,
                "expected choice-card item '{label}' width {} close to group width {}",
                b.size.width.0,
                group_bounds.size.width.0
            );
        }

        let w0 = item_bounds[0].1.size.width.0;
        for (label, b) in &item_bounds[1..] {
            assert!(
                (b.size.width.0 - w0).abs() <= eps,
                "expected choice-card item '{label}' width {} close to first item width {}",
                b.size.width.0,
                w0
            );
        }
    }

    #[test]
    fn radio_group_emits_radio_group_and_radio_button_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("b")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Options")
                        .item(RadioGroupItem::new("a", "Alpha"))
                        .item(RadioGroupItem::new("b", "Beta"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::RadioGroup && n.label.as_deref() == Some("Options")
            }),
            "radio group should expose RadioGroup role + label"
        );

        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha radio");
        assert_eq!(alpha.flags.checked, Some(false));
        assert_eq!(alpha.pos_in_set, Some(1));
        assert_eq!(alpha.set_size, Some(2));

        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Beta"))
            .expect("Beta radio");
        assert_eq!(beta.flags.checked, Some(true));
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(2));
    }

    #[test]
    fn radio_group_required_exposes_required_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("b")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-required",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .required(true)
                        .a11y_label("Options")
                        .item(RadioGroupItem::new("a", "Alpha"))
                        .item(RadioGroupItem::new("b", "Beta"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let group = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioGroup && n.label.as_deref() == Some("Options"))
            .expect("radio group semantics node");
        assert!(group.flags.required);
    }

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        orientation: RadioGroupOrientation,
        loop_navigation: bool,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![
                    RadioGroup::new(model)
                        .a11y_label("Options")
                        .orientation(orientation)
                        .loop_navigation(loop_navigation)
                        .item(RadioGroupItem::new("alpha", "Alpha"))
                        .item(RadioGroupItem::new("beta", "Beta"))
                        .item(RadioGroupItem::new("gamma", "Gamma"))
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
        orientation: RadioGroupOrientation,
        loop_navigation: bool,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![
                    RadioGroup::uncontrolled(default_value.clone())
                        .a11y_label("Options")
                        .orientation(orientation)
                        .loop_navigation(loop_navigation)
                        .item(RadioGroupItem::new("alpha", "Alpha"))
                        .item(RadioGroupItem::new("beta", "Beta"))
                        .item(RadioGroupItem::new("gamma", "Gamma"))
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn radio_group_uncontrolled_applies_default_value_once_and_does_not_reset() {
        fn checked(ui: &UiTree<App>, label: &str) -> Option<bool> {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some(label))
                .and_then(|n| n.flags.checked)
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(true));
        assert_eq!(checked(&ui, "Beta"), Some(false));

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(false));
        assert_eq!(checked(&ui, "Beta"), Some(true));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(false));
        assert_eq!(checked(&ui, "Beta"), Some(true));
    }

    #[test]
    fn radio_group_horizontal_arrow_right_moves_and_selects() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::RadioButton);
        assert_eq!(focused_node.label.as_deref(), Some("Beta"));
    }

    #[test]
    fn radio_group_loop_false_does_not_wrap_at_end() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("gamma")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            false,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            false,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("gamma"));
    }

    #[test]
    fn radio_group_does_not_select_on_enter_key() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("alpha")));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn radio_group_selects_on_space_key() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("alpha")));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
    }

    fn find_pressable_by_label<'a>(el: &'a AnyElement, label: &str) -> Option<&'a PressableProps> {
        if let ElementKind::Pressable(props) = &el.kind {
            if props.a11y.label.as_deref() == Some(label) {
                return Some(props);
            }
        }

        for child in &el.children {
            if let Some(found) = find_pressable_by_label(child, label) {
                return Some(found);
            }
        }

        None
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => return Some(props),
            _ => {}
        }

        for child in &el.children {
            if let Some(found) = find_text_by_content(child, text) {
                return Some(found);
            }
        }

        None
    }

    #[test]
    fn radio_group_word_wrapped_label_can_shrink_within_row() {
        let window = AppWindowId::default();
        let mut app = App::new();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "radio-wrap", |cx| {
            RadioGroup::new(model.clone())
                .a11y_label("Options")
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(160.0)))
                .item(RadioGroupItem::new(
                    "alpha",
                    "A very long radio label that should wrap instead of overflowing the row",
                ))
                .into_element(cx)
        });

        let label = find_text_by_content(
            &el,
            "A very long radio label that should wrap instead of overflowing the row",
        )
        .expect("radio group label text");

        assert_eq!(label.wrap, TextWrap::Word);
        assert_eq!(label.layout.flex.grow, 1.0);
        assert_eq!(label.layout.flex.shrink, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn radio_group_item_control_id_allows_field_label_to_select_specific_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let model = app.models_mut().insert(Some(Arc::from("yearly")));
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let monthly_id = ControlId::from("plan-monthly");
        let yearly_id = ControlId::from("plan-yearly");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "radio-group-item-control-id-label-forwarding",
            |cx| {
                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                    vec![
                        crate::field::FieldLabel::new("Monthly")
                            .for_control(monthly_id.clone())
                            .test_id("radio.item.monthly.label")
                            .into_element(cx),
                        crate::field::FieldLabel::new("Yearly")
                            .for_control(yearly_id.clone())
                            .test_id("radio.item.yearly.label")
                            .into_element(cx),
                        RadioGroup::new(model.clone())
                            .a11y_label("Subscription Plan")
                            .item(
                                RadioGroupItem::new("monthly", "Monthly")
                                    .control_id(monthly_id.clone()),
                            )
                            .item(
                                RadioGroupItem::new("yearly", "Yearly")
                                    .control_id(yearly_id.clone()),
                            )
                            .into_element(cx),
                    ]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
        let monthly_label_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("radio.item.monthly.label"))
            .map(|n| n.id)
            .expect("expected monthly label node");
        let monthly_label_bounds = ui
            .debug_node_bounds(monthly_label_node)
            .expect("expected monthly label bounds");
        let label_center = Point::new(
            Px(monthly_label_bounds.origin.x.0 + monthly_label_bounds.size.width.0 * 0.5),
            Px(monthly_label_bounds.origin.y.0 + monthly_label_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(7),
                position: label_center,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(7),
                position: label_center,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        assert_eq!(
            app.models().get_cloned(&model).flatten().as_deref(),
            Some("monthly")
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus after clicking item label");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::RadioButton);
        assert_eq!(focused_node.label.as_deref(), Some("Monthly"));
        assert_eq!(ui.focus(), Some(focused_node.id));
        let _ = root;
    }

    #[test]
    fn radio_group_item_focus_ring_tweens_in_and_out_like_a_transition() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<Option<Arc<str>>>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

            let model_for_render = model.clone();
            let ring_alpha_out_for_render = ring_alpha_out.clone();
            let always_paint_out_for_render = always_paint_out.clone();
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-radio-group-focus-ring-transition",
                move |cx| {
                    let el = RadioGroup::new(model_for_render.clone())
                        .item(RadioGroupItem::new("alpha", "Alpha"))
                        .item(RadioGroupItem::new("beta", "Beta"))
                        .into_element(cx);
                    let pressable = find_pressable_by_label(&el, "Alpha").expect("alpha pressable");
                    let ring = pressable.focus_ring.expect("focus ring");
                    ring_alpha_out_for_render.set(Some(ring.color.a));
                    always_paint_out_for_render.set(Some(pressable.focus_ring_always_paint));
                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        app.set_frame_id(FrameId(0));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(a0 > 0.0, "expected non-zero base ring alpha, got {a0}");
        let base_alpha = a0;

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_node = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Alpha"))
            .expect("alpha semantics");
        ui.set_focus(Some(alpha_node.id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(
            a1 >= 0.0 && a1 < base_alpha,
            "expected ring alpha to animate in, got base_alpha={base_alpha} a1={a1}"
        );

        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for _ in 0..settle {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4 && (a_focused - base_alpha).abs() <= 1e-4,
            "expected ring alpha to settle at base_alpha={base_alpha}, got a1={a1} a_focused={a_focused}"
        );

        ui.set_focus(None);
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur >= 0.0 && a_blur < a_focused,
            "expected ring alpha to animate out after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(always_paint, "expected always_paint while animating out");

        for _ in 0..settle {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            (a_final - base_alpha).abs() <= 1e-4,
            "expected base ring alpha={base_alpha}, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected always_paint to stop after settling"
        );
    }
}
