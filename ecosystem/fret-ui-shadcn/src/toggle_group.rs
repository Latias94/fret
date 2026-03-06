use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps, RovingFlexProps,
    RovingFocusProps, SpinnerProps, SvgIconProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius, Space,
    WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot,
    resolve_override_slot_opt,
};

use crate::toggle::{ToggleSize, ToggleVariant};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn toggle_bg_hover(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted"))
}

fn toggle_bg_on(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("accent")
        .unwrap_or_else(|| theme.color_token("accent"))
}

fn toggle_border(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"))
}

fn toggle_fg(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn toggle_fg_muted(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"))
}

fn toggle_fg_accent(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("accent-foreground")
        .unwrap_or_else(|| theme.color_token("accent-foreground"))
}

fn toggle_ring_color(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"))
}

fn toggle_group_item_h(theme: &ThemeSnapshot, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle_group.item_h", Px(36.0)),
        ToggleSize::Sm => ("component.toggle_group.item_h_sm", Px(32.0)),
        ToggleSize::Lg => ("component.toggle_group.item_h_lg", Px(40.0)),
    };
    theme
        .metric_by_key(key)
        .map(|v| Px(v.0.max(fallback.0)))
        .unwrap_or(fallback)
}

fn toggle_group_item_pad_x(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.toggle_group.item_pad_x")
        .unwrap_or(Px(12.0))
}

pub use fret_ui_kit::primitives::toggle_group::{ToggleGroupKind, ToggleGroupOrientation};

fn apply_item_inherited_style(
    mut element: AnyElement,
    fg: Color,
    default_icon_color: Color,
) -> AnyElement {
    match &mut element.kind {
        fret_ui::element::ElementKind::Text(props) => {
            props.color.get_or_insert(fg);
        }
        fret_ui::element::ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
            // Heuristic:
            // - Older callsites may build an `SvgIcon` with the default white color.
            // - `declarative::icon::icon(...)` built outside a `currentColor` provider resolves
            //   `muted-foreground` eagerly.
            //
            // In a toggle group item, both shapes should track the item's foreground by default.
            let is_default_white = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            let is_default_muted_fg = *color == default_icon_color;
            if is_default_white || is_default_muted_fg {
                *color = fg;
            }
        }
        fret_ui::element::ElementKind::Spinner(SpinnerProps { color, .. }) => {
            color.get_or_insert(fg);
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_item_inherited_style(child, fg, default_icon_color))
        .collect();
    element
}

#[derive(Clone)]
enum ToggleGroupModel {
    Single {
        model: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
    },
    Multiple {
        model: Option<Model<Vec<Arc<str>>>>,
        default_value: Vec<Arc<str>>,
    },
}

pub struct ToggleGroupItem {
    value: Arc<str>,
    children: Vec<AnyElement>,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ToggleGroupItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleGroupItem")
            .field("value", &self.value.as_ref())
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .finish()
    }
}

impl ToggleGroupItem {
    pub fn new(value: impl Into<Arc<str>>, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            value: value.into(),
            children: children.into_iter().collect(),
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    /// Icon-only convenience constructor (common in shadcn ToggleGroup docs).
    ///
    /// The icon is stored as an `IconId` and built by the ToggleGroup host so it can inherit the
    /// resolved item foreground via `currentColor`.
    pub fn icon(value: impl Into<Arc<str>>, icon: IconId) -> Self {
        Self::new(value, std::iter::empty::<AnyElement>()).leading_icon(icon)
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Optional diagnostics selector for the toggle item pressable root.
    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToggleGroupStyle {
    pub item_background: OverrideSlot<ColorRef>,
    pub item_foreground: OverrideSlot<ColorRef>,
    pub item_border_color: OverrideSlot<ColorRef>,
}

impl ToggleGroupStyle {
    pub fn item_background(
        mut self,
        item_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_background = Some(item_background);
        self
    }

    pub fn item_foreground(
        mut self,
        item_foreground: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_foreground = Some(item_foreground);
        self
    }

    pub fn item_border_color(
        mut self,
        item_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_border_color = Some(item_border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.item_background.is_some() {
            self.item_background = other.item_background;
        }
        if other.item_foreground.is_some() {
            self.item_foreground = other.item_foreground;
        }
        if other.item_border_color.is_some() {
            self.item_border_color = other.item_border_color;
        }
        self
    }
}

pub struct ToggleGroup {
    model: ToggleGroupModel,
    items: Vec<ToggleGroupItem>,
    disabled: bool,
    control_id: Option<ControlId>,
    roving_focus: bool,
    orientation: ToggleGroupOrientation,
    loop_navigation: bool,
    items_flex_1: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: Space,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ToggleGroupStyle,
}

impl std::fmt::Debug for ToggleGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.model {
            ToggleGroupModel::Single { .. } => ToggleGroupKind::Single,
            ToggleGroupModel::Multiple { .. } => ToggleGroupKind::Multiple,
        };
        f.debug_struct("ToggleGroup")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("roving_focus", &self.roving_focus)
            .field("orientation", &self.orientation)
            .field("loop_navigation", &self.loop_navigation)
            .field("items_flex_1", &self.items_flex_1)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("spacing", &self.spacing)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl ToggleGroup {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Single {
                model: Some(model),
                default_value: None,
            },
            items: Vec::new(),
            disabled: false,
            control_id: None,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            items_flex_1: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled single-select toggle group with an optional initial value (Radix
    /// `defaultValue`).
    pub fn single_uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: ToggleGroupModel::Single {
                model: None,
                default_value: default_value.map(Into::into),
            },
            items: Vec::new(),
            disabled: false,
            control_id: None,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            items_flex_1: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Multiple {
                model: Some(model),
                default_value: Vec::new(),
            },
            items: Vec::new(),
            disabled: false,
            control_id: None,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            items_flex_1: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled multi-select toggle group with an initial set of values (Radix
    /// `defaultValue`).
    pub fn multiple_uncontrolled<I, T>(default_value: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<str>>,
    {
        let default_value = default_value.into_iter().map(Into::into).collect();
        Self {
            model: ToggleGroupModel::Multiple {
                model: None,
                default_value,
            },
            items: Vec::new(),
            disabled: false,
            control_id: None,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            items_flex_1: false,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Binds this ToggleGroup to a logical form control id (similar to HTML `id`).
    ///
    /// When set, `Label::for_control(ControlId)` forwards focus to the group's current "tab stop"
    /// item (or the first enabled item).
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    /// When `true` (default), installs roving focus behavior (Radix `rovingFocus`).
    pub fn roving_focus(mut self, roving_focus: bool) -> Self {
        self.roving_focus = roving_focus;
        self
    }

    pub fn orientation(mut self, orientation: ToggleGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    /// When `true`, each toggle item participates in flex growth (Tailwind-like `flex-1`).
    ///
    /// Notes:
    /// - This should only be used when the parent layout provides a definite main-axis size.
    /// - In auto-sized compositions, forcing `flex: 1` can trigger very deep layout recursion.
    pub fn items_flex_1(mut self, flex_1: bool) -> Self {
        self.items_flex_1 = flex_1;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn spacing(mut self, spacing: Space) -> Self {
        self.spacing = spacing;
        self
    }

    /// When enabled, items in a horizontal group use `flex-1` so their pressable bounds stretch to
    /// fill the available width.
    pub fn items_full_width(mut self, full_width: bool) -> Self {
        self.items_flex_1 = full_width;
        self
    }

    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
        self.items.extend(items);
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

    pub fn style(mut self, style: ToggleGroupStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model;
        let items = self.items;
        let group_disabled = self.disabled;
        let control_id = self.control_id;
        let roving_focus = self.roving_focus;
        let orientation = self.orientation;
        let loop_navigation = self.loop_navigation;
        let items_flex_1 = self.items_flex_1;
        let variant = self.variant;
        let size_token = self.size;
        let spacing = self.spacing;
        let chrome = self.chrome;
        let layout = self.layout;
        let style_override = self.style;

        let control_id = control_id.clone();
        let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));

        let theme = Theme::global(&*cx.app).snapshot();

        let (model_single, model_multi, selected_single, selected_multi) = match &model {
            ToggleGroupModel::Single {
                model: controlled,
                default_value,
            } => {
                let model = fret_ui_kit::primitives::toggle_group::toggle_group_use_single_model(
                    cx,
                    controlled.clone(),
                    || default_value.clone(),
                )
                .model();
                let selected = cx.watch_model(&model).layout().cloned().flatten();
                (Some(model), None, selected, None)
            }
            ToggleGroupModel::Multiple {
                model: controlled,
                default_value,
            } => {
                let model = fret_ui_kit::primitives::toggle_group::toggle_group_use_multiple_model(
                    cx,
                    controlled.clone(),
                    || default_value.clone(),
                )
                .model();
                let selected = cx.watch_model(&model).layout().cloned();
                (None, Some(model), None, selected)
            }
        };

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> =
            items.iter().map(|i| group_disabled || i.disabled).collect();

        let tab_stop = if roving_focus {
            match (selected_single.as_deref(), selected_multi.as_ref()) {
                (Some(selected), _) => {
                    fret_ui_kit::primitives::toggle_group::tab_stop_index_single(
                        &values,
                        Some(selected),
                        &disabled_flags,
                    )
                }
                (_, Some(selected)) => {
                    fret_ui_kit::primitives::toggle_group::tab_stop_index_multiple(
                        &values,
                        selected,
                        &disabled_flags,
                    )
                }
                _ => fret_ui_kit::primitives::roving_focus_group::first_enabled(&disabled_flags),
            }
        } else {
            None
        };
        let control_target_index = tab_stop.or_else(|| {
            fret_ui_kit::primitives::roving_focus_group::first_enabled(&disabled_flags)
        });

        let gap = MetricRef::space(spacing).resolve(&theme);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = alpha_mul(toggle_ring_color(&theme), 0.5);
        let ring_border = toggle_ring_color(&theme);

        let item_h = toggle_group_item_h(&theme, size_token);
        let pad_x = toggle_group_item_pad_x(&theme);
        let pad_y = Px(0.0);

        let bg_hover_muted = toggle_bg_hover(&theme);
        let bg_on = toggle_bg_on(&theme);
        let border = toggle_border(&theme);

        let ToggleGroupStyle {
            item_background,
            item_foreground,
            item_border_color,
        } = style_override;

        let hover_bg = match variant {
            ToggleVariant::Default => bg_hover_muted,
            ToggleVariant::Outline => bg_on,
        };

        let hover_fg = match variant {
            ToggleVariant::Default => toggle_fg_muted(&theme),
            ToggleVariant::Outline => toggle_fg_accent(&theme),
        };

        let default_item_background = WidgetStateProperty::new(None)
            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::ACTIVE, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::SELECTED, Some(ColorRef::Color(bg_on)))
            .when(WidgetStates::DISABLED, None);

        let default_item_foreground = {
            let fg = toggle_fg(&theme);
            let fg_disabled = alpha_mul(fg, 0.5);
            WidgetStateProperty::new(ColorRef::Color(fg))
                .when(WidgetStates::HOVERED, ColorRef::Color(hover_fg))
                .when(WidgetStates::ACTIVE, ColorRef::Color(hover_fg))
                .when(
                    WidgetStates::SELECTED,
                    ColorRef::Color(toggle_fg_accent(&theme)),
                )
                .when(WidgetStates::DISABLED, ColorRef::Color(fg_disabled))
        };

        let default_item_border_color = WidgetStateProperty::new(None)
            .when(
                WidgetStates::FOCUS_VISIBLE,
                Some(ColorRef::Color(ring_border)),
            )
            .when(WidgetStates::DISABLED, None);

        let item_background_override = item_background;
        let item_foreground_override = item_foreground;
        let item_border_color_override = item_border_color;

        let mut group_props = decl_style::container_props(&theme, chrome, layout);
        group_props.corner_radii = Corners::all(radius);
        if matches!(variant, ToggleVariant::Outline) && gap.0 > 0.0 {
            group_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
        }

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement::default()
                .radius(radius)
                .border_width(Px(1.0))
                .border_color(ColorRef::Color(Color::TRANSPARENT)),
            ToggleVariant::Outline => ChromeRefinement::default()
                .radius(radius)
                .border_width(Px(1.0))
                .border_color(ColorRef::Color(border)),
        };

        let roving = RovingFocusProps {
            enabled: roving_focus && !group_disabled,
            wrap: loop_navigation,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
        };

        cx.container(group_props, move |cx| {
            let item_background_override = item_background_override.clone();
            let item_foreground_override = item_foreground_override.clone();
            let item_border_color_override = item_border_color_override.clone();
            let default_item_background = default_item_background.clone();
            let default_item_foreground = default_item_foreground.clone();
            let default_item_border_color = default_item_border_color.clone();
            let control_id = control_id.clone();
            let control_registry = control_registry.clone();

            let flex = FlexProps {
                direction: match orientation {
                    ToggleGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
                    ToggleGroupOrientation::Vertical => fret_core::Axis::Vertical,
                },
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: match orientation {
                    ToggleGroupOrientation::Horizontal => CrossAlign::Center,
                    ToggleGroupOrientation::Vertical => CrossAlign::Stretch,
                },
                wrap: false,
                ..Default::default()
            };

            let inner_gap = MetricRef::space(Space::N1).resolve(&theme);

            let render_items = move |cx: &mut ElementContext<'_, H>| {
                if roving_focus {
                    cx.roving_nav_apg();
                }
                let n = items.len();
                let mut out = Vec::with_capacity(n);

                for (idx, item) in items.into_iter().enumerate() {
                    let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                    let enabled = !item_disabled;
                    let is_control_target =
                        control_target_index.is_some_and(|control_idx| control_idx == idx);
                    let focusable = if roving_focus {
                        tab_stop.is_some_and(|i| i == idx)
                    } else {
                        enabled
                    };
                    let on = selected_single
                        .as_deref()
                        .is_some_and(|v| v == item.value.as_ref())
                        || selected_multi.as_ref().is_some_and(|selected| {
                            selected.iter().any(|v| v.as_ref() == item.value.as_ref())
                        });

                    let corners = if gap.0 <= 0.0 {
                        let first = idx == 0;
                        let last = idx + 1 == n;
                        match orientation {
                            ToggleGroupOrientation::Horizontal => Corners {
                                top_left: if first { radius } else { Px(0.0) },
                                bottom_left: if first { radius } else { Px(0.0) },
                                top_right: if last { radius } else { Px(0.0) },
                                bottom_right: if last { radius } else { Px(0.0) },
                            },
                            ToggleGroupOrientation::Vertical => Corners {
                                top_left: if first { radius } else { Px(0.0) },
                                top_right: if first { radius } else { Px(0.0) },
                                bottom_left: if last { radius } else { Px(0.0) },
                                bottom_right: if last { radius } else { Px(0.0) },
                            },
                        }
                    } else {
                        Corners::all(radius)
                    };

                    let mut base_props = decl_style::container_props(
                        &theme,
                        base_chrome.clone(),
                        LayoutRefinement::default(),
                    );
                    base_props.padding = Edges {
                        top: pad_y,
                        right: pad_x,
                        bottom: pad_y,
                        left: pad_x,
                    }
                    .into();
                    base_props.corner_radii = corners;

                    if gap.0 <= 0.0
                        && variant == ToggleVariant::Outline
                        && idx > 0
                        && (base_props.border.left.0 > 0.0 || base_props.border.top.0 > 0.0)
                    {
                        match orientation {
                            ToggleGroupOrientation::Horizontal => {
                                base_props.border.left = Px(0.0);
                            }
                            ToggleGroupOrientation::Vertical => {
                                base_props.border.top = Px(0.0);
                            }
                        }
                    }

                    let value = item.value.clone();
                    let a11y_label = item.a11y_label.clone().unwrap_or_else(|| value.clone());
                    let mut a11y = if model_single.is_some() {
                        fret_ui_kit::primitives::toggle_group::toggle_group_item_a11y_single(
                            a11y_label.clone(),
                            on,
                        )
                    } else {
                        fret_ui_kit::primitives::toggle_group::toggle_group_item_a11y_multiple(
                            a11y_label.clone(),
                            on,
                        )
                    };
                    if let Some(test_id) = item.test_id.clone() {
                        a11y.test_id = Some(test_id);
                    }
                    let children = item.children;
                    let leading_icon = item.leading_icon;
                    let trailing_icon = item.trailing_icon;
                    let model_single = model_single.clone();
                    let model_multi = model_multi.clone();
                    let pressable_layout = {
                        let mut refinement = LayoutRefinement::default()
                            .h_px(item_h)
                            .min_h(item_h)
                            .min_w_0();
                        if items_flex_1 && matches!(orientation, ToggleGroupOrientation::Horizontal)
                        {
                            refinement = refinement.flex_1();
                        } else {
                            refinement = refinement.flex_none();
                        }
                        decl_style::layout_style(&theme, refinement)
                    };

                    let item_theme = theme.clone();
                    let item_background_override = item_background_override.clone();
                    let item_foreground_override = item_foreground_override.clone();
                    let item_border_color_override = item_border_color_override.clone();
                    let default_item_background = default_item_background.clone();
                    let default_item_foreground = default_item_foreground.clone();
                    let default_item_border_color = default_item_border_color.clone();
                    let inner_gap = inner_gap;
                    let control_id_for_register = control_id.clone();
                    let control_registry_for_register = control_registry.clone();
                    let is_control_target_for_register = is_control_target;

                    out.push(cx.keyed(value.clone(), move |cx| {
                        control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                            if is_control_target_for_register
                                && let (Some(control_id), Some(control_registry)) = (
                                    control_id_for_register.clone(),
                                    control_registry_for_register.clone(),
                                )
                            {
                                let entry = ControlEntry {
                                    element: _id,
                                    enabled,
                                    action: ControlAction::Noop,
                                };
                                let _ = cx.app.models_mut().update(&control_registry, |reg| {
                                    reg.register_control(cx.window, cx.frame_id, control_id, entry);
                                });
                            }

                            if let Some(m) = model_single.as_ref() {
                                let model = m.clone();
                                let value = value.clone();
                                cx.pressable_add_on_activate(Arc::new(
                                    move |host, _action_cx, _reason| {
                                        let current =
                                            host.models_mut().get_cloned(&model).flatten();
                                        let next = if current
                                            .as_ref()
                                            .is_some_and(|cur| cur.as_ref() == value.as_ref())
                                        {
                                            None
                                        } else {
                                            Some(value.clone())
                                        };
                                        let _ = host.models_mut().update(&model, |v| *v = next);
                                    },
                                ));
                            }
                            if let Some(m) = model_multi.as_ref() {
                                cx.pressable_toggle_vec_arc_str(m, value.clone());
                            }

                            let mut states = WidgetStates::from_pressable(cx, st, enabled);
                            states.set(WidgetState::Selected, on);

                            let fg_ref = resolve_override_slot(
                                item_foreground_override.as_ref(),
                                &default_item_foreground,
                                states,
                            );
                            let fg = fg_ref.resolve(&item_theme);
                            let default_icon_color = toggle_fg_muted(&item_theme);

                            let mut chrome_props = base_props;
                            if let Some(bg) = resolve_override_slot_opt(
                                item_background_override.as_ref(),
                                &default_item_background,
                                states,
                            ) {
                                chrome_props.background = Some(bg.resolve(&item_theme));
                            }

                            if let Some(border_color) = resolve_override_slot_opt(
                                item_border_color_override.as_ref(),
                                &default_item_border_color,
                                states,
                            ) {
                                chrome_props.border_color = Some(border_color.resolve(&item_theme));
                            }

                            let mut styled_children: Vec<AnyElement> = Vec::with_capacity(
                                leading_icon.is_some() as usize
                                    + children.len()
                                    + trailing_icon.is_some() as usize,
                            );
                            styled_children.extend(children.into_iter().map(|child| {
                                apply_item_inherited_style(child, fg, default_icon_color)
                            }));

                            let content = move |cx: &mut ElementContext<'_, H>| {
                                current_color::scope_children(cx, fg_ref.clone(), |cx| {
                                    let mut content_children: Vec<AnyElement> =
                                        Vec::with_capacity(styled_children.len() + 2);
                                    if let Some(icon) = leading_icon.clone() {
                                        content_children.push(decl_icon::icon(cx, icon));
                                    }
                                    content_children.extend(styled_children);
                                    if let Some(icon) = trailing_icon.clone() {
                                        content_children.push(decl_icon::icon(cx, icon));
                                    }

                                    vec![cx.flex(
                                        FlexProps {
                                            layout: {
                                                let mut layout =
                                                    fret_ui::element::LayoutStyle::default();
                                                layout.size.height = fret_ui::element::Length::Fill;
                                                // Only force a full-width inner layout when the
                                                // item itself is intended to stretch (Tailwind
                                                // `flex-1` / "full width" toggle groups). For
                                                // auto-sized groups, forcing `Fill` here makes the
                                                // pressable bounds expand and results in large
                                                // invisible hit boxes.
                                                if items_flex_1
                                                    && matches!(
                                                        orientation,
                                                        ToggleGroupOrientation::Horizontal
                                                    )
                                                {
                                                    layout.size.width =
                                                        fret_ui::element::Length::Fill;
                                                }
                                                layout
                                            },
                                            direction: fret_core::Axis::Horizontal,
                                            gap: inner_gap.into(),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Center,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        move |_cx| content_children,
                                    )]
                                })
                            };

                            (
                                PressableProps {
                                    layout: pressable_layout,
                                    enabled,
                                    focusable,
                                    focus_ring: Some(ring),
                                    a11y,
                                    ..Default::default()
                                },
                                chrome_props,
                                content,
                            )
                        })
                    }));
                }

                out
            };

            if roving_focus {
                vec![cx.roving_flex(RovingFlexProps { flex, roving }, render_items)]
            } else {
                vec![cx.flex(flex, render_items)]
            }
        })
    }
}

pub fn toggle_group_single<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = ToggleGroupItem>,
{
    ToggleGroup::single(model).items(f(cx)).into_element(cx)
}

pub fn toggle_group_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = ToggleGroupItem>,
{
    ToggleGroup::single_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

pub fn toggle_group_multiple<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = ToggleGroupItem>,
{
    ToggleGroup::multiple(model).items(f(cx)).into_element(cx)
}

pub fn toggle_group_multiple_uncontrolled<H: UiHost, V, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: V,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    V: IntoIterator,
    V::Item: Into<Arc<str>>,
    I: IntoIterator<Item = ToggleGroupItem>,
{
    ToggleGroup::multiple_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_ui::element::{CrossAlign, ElementKind, Length, SpacingLength};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
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
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render_single(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-single",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![ToggleGroup::single(model).items(items).into_element(cx)]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_single_without_roving_focus(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-single-no-roving",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::single(model)
                        .roving_focus(false)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_single_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-single-uncontrolled",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::single_uncontrolled(default_value.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_multiple_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Vec<Arc<str>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-multiple-uncontrolled",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::multiple_uncontrolled(default_value.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn toggle_group_single_deactivates_when_activating_selected_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);
    }

    #[test]
    fn toggle_group_single_arrow_moves_focus_without_changing_value() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::RadioButton);
        assert_eq!(focused_node.label.as_deref(), Some("beta"));
    }

    #[test]
    fn toggle_group_single_without_roving_focus_does_not_move_focus_on_arrow() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_single_without_roving_focus(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_single_without_roving_focus(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.label.as_deref(), Some("alpha"));
    }

    #[test]
    fn toggle_group_single_uncontrolled_applies_default_value_once_and_allows_deactivate() {
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

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(true));

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(false));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(false));
    }

    #[test]
    fn toggle_group_multiple_uncontrolled_applies_default_value_once_and_allows_toggle() {
        fn pressed(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(label))
                .is_some_and(|n| {
                    n.flags.pressed_state == Some(fret_core::SemanticsPressedState::True)
                })
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let default_value = vec![Arc::from("alpha"), Arc::from("gamma")];
        let root = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value.clone(),
        );
        assert!(pressed(&ui, "alpha"));
        assert!(pressed(&ui, "gamma"));

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value.clone(),
        );
        assert!(!pressed(&ui, "alpha"));
        assert!(pressed(&ui, "gamma"));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value,
        );
        assert!(!pressed(&ui, "alpha"));
        assert!(pressed(&ui, "gamma"));
    }

    fn apply_theme(app: &mut App) {
        crate::shadcn_themes::apply_shadcn_new_york(
            app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
    }

    fn bounds_320x240() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    fn render_group_props(app: &mut App, window: AppWindowId, group: ToggleGroup) -> AnyElement {
        fret_ui::elements::with_element_cx(app, window, bounds_320x240(), "test", |cx| {
            group.into_element(cx)
        })
    }

    fn group_items() -> Vec<ToggleGroupItem> {
        vec![
            ToggleGroupItem::new("one", Vec::<AnyElement>::new()),
            ToggleGroupItem::new("two", Vec::<AnyElement>::new()),
        ]
    }

    #[test]
    fn toggle_group_root_defaults_to_w_fit_and_zero_gap() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let el = render_group_props(
            &mut app,
            window,
            ToggleGroup::single(model).items(group_items()),
        );

        let ElementKind::Container(props) = &el.kind else {
            panic!("expected ToggleGroup root to be a container");
        };
        assert_eq!(props.layout.size.width, Length::Auto);

        let child = el.children.first().expect("container child");
        let flex = match &child.kind {
            ElementKind::RovingFlex(props) => &props.flex,
            ElementKind::Flex(props) => props,
            _ => panic!("expected ToggleGroup to contain a flex/roving flex child"),
        };
        assert_eq!(flex.gap, SpacingLength::Px(Px(0.0)));
    }

    #[test]
    fn toggle_group_vertical_orientation_stretches_items_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let el = render_group_props(
            &mut app,
            window,
            ToggleGroup::single(model)
                .orientation(ToggleGroupOrientation::Vertical)
                .items(group_items()),
        );

        let child = el.children.first().expect("container child");
        let flex = match &child.kind {
            ElementKind::RovingFlex(props) => &props.flex,
            ElementKind::Flex(props) => props,
            _ => panic!("expected ToggleGroup to contain a flex/roving flex child"),
        };
        assert_eq!(flex.direction, fret_core::Axis::Vertical);
        assert_eq!(flex.align, CrossAlign::Stretch);
    }
}
