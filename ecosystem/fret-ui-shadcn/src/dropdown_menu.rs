use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Edges, FontId, FontWeight, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextStyle,
    TextWrap, Transform2D,
};
use fret_icons::ids;
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    OpacityProps, PressableProps, RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps,
    SemanticsProps, SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::menu;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{ColorRef, MetricRef, OverlayController, OverlayPresence, Space};

use crate::overlay_motion;
use crate::popper_arrow::{self, DiamondArrowStyle};

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

#[derive(Debug, Clone)]
pub enum DropdownMenuEntry {
    Item(DropdownMenuItem),
    CheckboxItem(DropdownMenuCheckboxItem),
    RadioGroup(DropdownMenuRadioGroup),
    RadioItem(DropdownMenuRadioItem),
    Label(DropdownMenuLabel),
    Group(DropdownMenuGroup),
    Separator,
}

#[derive(Debug, Clone)]
pub struct DropdownMenuItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub inset: bool,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
    pub variant: DropdownMenuItemVariant,
    pub submenu: Option<Vec<DropdownMenuEntry>>,
}

impl DropdownMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            inset: false,
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            trailing: None,
            variant: DropdownMenuItemVariant::Default,
            submenu: None,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn variant(mut self, variant: DropdownMenuItemVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn submenu(mut self, entries: Vec<DropdownMenuEntry>) -> Self {
        self.submenu = Some(entries);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

/// shadcn/ui `DropdownMenuCheckboxItem` (v4).
#[derive(Debug, Clone)]
pub struct DropdownMenuCheckboxItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub checked: Model<bool>,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl DropdownMenuCheckboxItem {
    pub fn new(checked: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            checked,
            leading: None,
            disabled: false,
            close_on_select: false,
            command: None,
            a11y_label: None,
            trailing: None,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

/// shadcn/ui `DropdownMenuRadioGroup` (v4).
#[derive(Debug, Clone)]
pub struct DropdownMenuRadioGroup {
    pub value: Model<Option<Arc<str>>>,
    pub items: Vec<DropdownMenuRadioItemSpec>,
}

impl DropdownMenuRadioGroup {
    pub fn new(value: Model<Option<Arc<str>>>) -> Self {
        Self {
            value,
            items: Vec::new(),
        }
    }

    pub fn item(mut self, item: DropdownMenuRadioItemSpec) -> Self {
        self.items.push(item);
        self
    }
}

#[derive(Debug, Clone)]
pub struct DropdownMenuRadioItemSpec {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl DropdownMenuRadioItemSpec {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            trailing: None,
        }
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }

    fn into_item(self, group_value: Model<Option<Arc<str>>>) -> DropdownMenuRadioItem {
        DropdownMenuRadioItem {
            label: self.label,
            value: self.value,
            group_value,
            leading: self.leading,
            disabled: self.disabled,
            close_on_select: self.close_on_select,
            command: self.command,
            a11y_label: self.a11y_label,
            trailing: self.trailing,
        }
    }
}

/// shadcn/ui `DropdownMenuRadioItem` (v4).
#[derive(Debug, Clone)]
pub struct DropdownMenuRadioItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub group_value: Model<Option<Arc<str>>>,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl DropdownMenuRadioItem {
    pub fn new(
        group_value: Model<Option<Arc<str>>>,
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
    ) -> Self {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
            group_value,
            leading: None,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            trailing: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn leading(mut self, element: AnyElement) -> Self {
        self.leading = Some(element);
        self
    }

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

/// shadcn/ui `DropdownMenuLabel` (v4).
#[derive(Debug, Clone)]
pub struct DropdownMenuLabel {
    pub text: Arc<str>,
    pub inset: bool,
}

impl DropdownMenuLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            inset: false,
        }
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
}

/// shadcn/ui `DropdownMenuGroup` (v4).
///
/// In the upstream DOM implementation, this is a structural wrapper. In Fret, we currently treat
/// it as a transparent grouping node and simply flatten its entries for rendering/navigation.
#[derive(Debug, Clone)]
pub struct DropdownMenuGroup {
    pub entries: Vec<DropdownMenuEntry>,
}

impl DropdownMenuGroup {
    pub fn new(entries: Vec<DropdownMenuEntry>) -> Self {
        Self { entries }
    }
}

/// shadcn/ui `DropdownMenuShortcut` (v4).
///
/// This is typically rendered as trailing, muted text inside a menu item.
#[derive(Debug, Clone)]
pub struct DropdownMenuShortcut {
    pub text: Arc<str>,
}

impl DropdownMenuShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let font_size = theme
            .metric_by_key("component.dropdown_menu.shortcut.font_size")
            .unwrap_or_else(|| Px((theme.metrics.font_size.0 - 1.0).max(10.0)));
        let font_line_height = theme
            .metric_by_key("component.dropdown_menu.shortcut.line_height")
            .unwrap_or_else(|| Px((theme.metrics.font_line_height.0 - 2.0).max(font_size.0)));

        cx.text_props(TextProps {
            layout: {
                let mut layout = LayoutStyle::default();
                // new-york-v4: `ml-auto` to push shortcut to the trailing edge.
                layout.margin.left = fret_ui::element::MarginEdge::Auto;
                layout
            },
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                // new-york-v4: `text-xs`.
                size: font_size,
                weight: FontWeight::NORMAL,
                line_height: Some(font_line_height),
                // new-york-v4: `tracking-widest`.
                letter_spacing_em: Some(0.10),
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

fn flatten_entries(into: &mut Vec<DropdownMenuEntry>, entries: Vec<DropdownMenuEntry>) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Group(group) => flatten_entries(into, group.entries),
            DropdownMenuEntry::RadioGroup(group) => {
                for item in group.items {
                    into.push(DropdownMenuEntry::RadioItem(
                        item.into_item(group.value.clone()),
                    ));
                }
            }
            other => into.push(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckableIndicatorKind {
    Check,
    RadioDot,
}

fn checkable_menu_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    leading: Option<AnyElement>,
    reserve_leading_slot: bool,
    trailing: Option<AnyElement>,
    indicator_kind: CheckableIndicatorKind,
    indicator_on: bool,
    disabled: bool,
    row_bg: fret_core::Color,
    row_fg: fret_core::Color,
    text_style: TextStyle,
    _font_size: Px,
    _font_line_height: Px,
    pad_x: Px,
    pad_x_inset: Px,
    pad_y: Px,
    radius_sm: Px,
    text_disabled: fret_core::Color,
) -> Vec<AnyElement> {
    let indicator_fg = if disabled { text_disabled } else { row_fg };

    vec![cx.container(
        ContainerProps {
            layout: LayoutStyle::default(),
            padding: Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                // new-york-v4: checkbox/radio items use `pl-8`.
                left: pad_x_inset,
            },
            background: Some(row_bg),
            corner_radii: fret_core::Corners::all(radius_sm),
            ..Default::default()
        },
        move |cx| {
            let indicator = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        position: fret_ui::element::PositionStyle::Absolute,
                        inset: fret_ui::element::InsetStyle {
                            top: Some(Px(0.0)),
                            right: None,
                            bottom: Some(Px(0.0)),
                            // new-york-v4: indicator slot uses `left-2`.
                            left: Some(pad_x),
                        },
                        size: SizeStyle {
                            width: Length::Px(Px(16.0)),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            if !indicator_on {
                                return Vec::new();
                            }

                            match indicator_kind {
                                CheckableIndicatorKind::Check => vec![decl_icon::icon_with(
                                    cx,
                                    ids::ui::CHECK,
                                    Some(Px(16.0)),
                                    Some(ColorRef::Color(indicator_fg)),
                                )],
                                CheckableIndicatorKind::RadioDot => vec![cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(8.0));
                                            layout.size.height = Length::Px(Px(8.0));
                                            layout
                                        },
                                        padding: Edges::all(Px(0.0)),
                                        background: Some(indicator_fg),
                                        shadow: None,
                                        border: Edges::all(Px(0.0)),
                                        border_color: None,
                                        corner_radii: fret_core::Corners::all(Px(999.0)),
                                    },
                                    |_cx| Vec::new(),
                                )],
                            }
                        },
                    )]
                },
            );

            let mut row: Vec<AnyElement> = Vec::with_capacity(
                2 + usize::from(leading.is_some() || reserve_leading_slot)
                    + usize::from(trailing.is_some()),
            );

            if let Some(l) = leading.clone() {
                row.push(menu_icon_slot(cx, l));
            } else if reserve_leading_slot {
                row.push(menu_icon_slot_empty(cx));
            }

            row.push(cx.text_props(TextProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.min_width = Some(Px(0.0));
                    layout.flex.grow = 1.0;
                    layout.flex.shrink = 1.0;
                    layout.flex.basis = Length::Px(Px(0.0));
                    layout
                },
                text: label.clone(),
                style: Some(text_style.clone()),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                color: Some(if disabled { text_disabled } else { row_fg }),
            }));

            if let Some(t) = trailing.clone() {
                row.push(t);
            }

            let content = cx.flex(
                FlexProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout
                    },
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(8.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| row.clone(),
            );

            vec![content, indicator]
        },
    )]
}

fn submenu_chevron_right_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    fg: fret_core::Color,
    _font_size: Px,
    _font_line_height: Px,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout.margin.left = fret_ui::element::MarginEdge::Auto;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            vec![decl_icon::icon_with(
                cx,
                ids::ui::CHEVRON_RIGHT,
                Some(Px(16.0)),
                Some(ColorRef::Color(fg)),
            )]
        },
    )
}

fn menu_icon_slot<H: UiHost>(cx: &mut ElementContext<'_, H>, element: AnyElement) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![element.clone()],
    )
}

fn menu_icon_slot_empty<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(16.0));
                layout.size.height = Length::Px(Px(16.0));
                layout.flex.shrink = 0.0;
                layout
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        |_cx| Vec::new(),
    )
}

/// shadcn/ui `Dropdown Menu` (v4).
///
/// This is a dismissible popover overlay (non-modal) backed by the component-layer overlay
/// manager (`fret-ui-kit/overlay_controller.rs`).
#[derive(Clone)]
pub struct DropdownMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    min_width: Px,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    align_leading_icons: bool,
}

impl std::fmt::Debug for DropdownMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropdownMenu")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl DropdownMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: DropdownMenuAlign::default(),
            side: DropdownMenuSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(8.0),
            typeahead_timeout_ticks: 30,
            min_width: Px(128.0),
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            align_leading_icons: true,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn align_leading_icons(mut self, align: bool) -> Self {
        self.align_leading_icons = align;
        self
    }

    /// Enables a DropdownMenu arrow (Radix `DropdownMenuArrow`-style).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);
            let motion = OverlayController::transition_with_durations_and_easing(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::shadcn_ease,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };
            let opacity = motion.progress;
            let opening = is_open;
            let arrow = self.arrow;
            let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.dropdown_menu.arrow_size")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                    .unwrap_or(Px(12.0))
            });
            let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.dropdown_menu.arrow_padding")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                    .unwrap_or(theme.metrics.radius_md)
            });

            let trigger = trigger(cx);
            let trigger_id = trigger.id;
            menu::trigger::wire_open_on_arrow_keys(cx, trigger_id, self.open.clone());
            let overlay_root_name = OverlayController::popover_root_name(trigger_id);
            let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
            let submenu =
                cx.with_root_name(&overlay_root_name, |cx| {
                    menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
                });

            if overlay_presence.present {
                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let open = self.open;
                let open_for_overlay = open.clone();
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let min_width = self.min_width;
                let align_leading_icons = self.align_leading_icons;
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();

                let (overlay_children, dismissible_on_pointer_move) =
                    cx.with_root_name(&overlay_root_name, move |cx| {
                    let theme = &theme;
                    let anchor = overlay::anchor_bounds_for_element(cx, trigger_id);
                    let Some(anchor) = anchor else {
                        return (Vec::new(), None);
                    };

                    let mut flat: Vec<DropdownMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, entries(cx));
                    let entries = flat;
                    let reserve_leading_slot = align_leading_icons
                        && entries.iter().any(|e| match e {
                            DropdownMenuEntry::Item(item) => item.leading.is_some(),
                            DropdownMenuEntry::CheckboxItem(item) => item.leading.is_some(),
                            DropdownMenuEntry::RadioItem(item) => item.leading.is_some(),
                            DropdownMenuEntry::Label(_)
                            | DropdownMenuEntry::Group(_)
                            | DropdownMenuEntry::RadioGroup(_)
                            | DropdownMenuEntry::Separator => false,
                        });

                    let item_count = entries
                        .iter()
                        .filter(|e| {
                            matches!(
                                e,
                                DropdownMenuEntry::Item(_)
                                    | DropdownMenuEntry::CheckboxItem(_)
                                    | DropdownMenuEntry::RadioItem(_)
                            )
                        })
                        .count();
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                        .iter()
                        .map(|e| match e {
                            DropdownMenuEntry::Item(item) => (item.label.clone(), item.disabled),
                            DropdownMenuEntry::CheckboxItem(item) => {
                                (item.label.clone(), item.disabled)
                            }
                            DropdownMenuEntry::RadioItem(item) => (item.label.clone(), item.disabled),
                            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {
                                (Arc::from(""), true)
                            }
                            DropdownMenuEntry::Group(_) | DropdownMenuEntry::RadioGroup(_) => {
                                unreachable!("entries are flattened")
                            }
                        })
                        .unzip();

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                     let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                     // shadcn: content width tracks trigger width (with a minimum), and height clamps
                     // to available space (scrolls internally).
                     let desired = Size::new(Px(anchor.size.width.0.max(min_width.0)), Px(1.0e9));

                    let align = match align {
                        DropdownMenuAlign::Start => Align::Start,
                        DropdownMenuAlign::Center => Align::Center,
                        DropdownMenuAlign::End => Align::End,
                    };
                    let side = match side {
                        DropdownMenuSide::Top => Side::Top,
                        DropdownMenuSide::Right => Side::Right,
                        DropdownMenuSide::Bottom => Side::Bottom,
                        DropdownMenuSide::Left => Side::Left,
                    };

                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let layout = popper::popper_content_layout_sized(
                        outer,
                        anchor,
                        desired,
                        popper::PopperContentPlacement::new(
                            LayoutDirection::Ltr,
                            side,
                            align,
                            side_offset,
                        )
                        .with_arrow(arrow_options, arrow_protrusion),
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let extra_left = wrapper_insets.left;
                    let extra_top = wrapper_insets.top;
                    let origin = popper::popper_content_transform_origin(
                        &layout,
                        anchor,
                        arrow.then_some(arrow_size),
                    );
                    let zoom = overlay_motion::shadcn_zoom_transform(origin, opacity);
                    let slide = if opening {
                        overlay_motion::shadcn_enter_slide_transform(layout.side, opacity, opening)
                    } else {
                        Transform2D::IDENTITY
                    };
                    let transform = slide * zoom;

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    // new-york-v4:
                    // - `DropdownMenuContent`: `shadow-md`
                    // - `DropdownMenuSubContent`: `shadow-lg`
                    let shadow = decl_style::shadow_md(&theme, theme.metrics.radius_sm);
                    let shadow_submenu = decl_style::shadow_lg(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    // new-york-v4: item rows use `px-2`.
                    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                    // new-york-v4: item rows use `py-1.5`.
                    let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                    let bg = theme
                        .color_by_key("popover")
                        .or_else(|| theme.color_by_key("popover.background"))
                        .unwrap_or(theme.colors.panel_background);
                    let fg = theme
                        .color_by_key("popover.foreground")
                        .or_else(|| theme.color_by_key("popover-foreground"))
                        .unwrap_or(theme.colors.text_primary);
                    let accent = theme
                        .color_by_key("accent")
                        .unwrap_or(theme.colors.hover_background);
                    let accent_fg = theme
                        .color_by_key("accent.foreground")
                        .or_else(|| theme.color_by_key("accent-foreground"))
                        .unwrap_or(theme.colors.text_primary);

                    let entries_for_submenu = entries.clone();
                    let open_for_menu = open_for_overlay.clone();
                    let open_for_submenu = open_for_overlay.clone();

                    let submenu_for_content = submenu.clone();
                    let submenu_for_panel = submenu.clone();
                    let arrow_el = popper_arrow::diamond_arrow_element(
                        cx,
                        &layout,
                        wrapper_insets,
                        arrow_size,
                        DiamondArrowStyle {
                            bg,
                            border: Some(border),
                            border_width: Px(1.0),
                        },
                    );

                    let content = cx.semantics(
                        SemanticsProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            role: SemanticsRole::Menu,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![popper_content::popper_wrapper_at(
                                cx,
                                placed,
                                wrapper_insets,
                                move |cx| {
                                    let panel = menu::content_panel::menu_panel_container_at(
                                        cx,
                                        Rect::new(Point::new(extra_left, extra_top), placed.size),
                                        move |layout| ContainerProps {
                                            layout,
                                            padding: Edges::all(Px(4.0)),
                                            background: Some(bg),
                                            shadow: Some(shadow),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border),
                                            corner_radii: fret_core::Corners::all(
                                                theme.metrics.radius_sm,
                                            ),
                                        },
                                        move |cx| {
                                    let scroll_layout = LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    };

                                    vec![cx.scroll(
                                        ScrollProps {
                                            layout: scroll_layout,
                                            axis: ScrollAxis::Y,
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let content_focus_id_for_scroll =
                                                content_focus_id_for_children.clone();
                                            let roving = menu::content::menu_roving_group_apg_prefix_typeahead(
                                                cx,
                                                RovingFlexProps {
                                                    flex: FlexProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.width = Length::Fill;
                                                            layout
                                                        },
                                                        direction: fret_core::Axis::Vertical,
                                                        gap: Px(0.0),
                                                        padding: Edges::all(Px(0.0)),
                                                        justify: MainAlign::Start,
                                                        align: CrossAlign::Stretch,
                                                        wrap: false,
                                                    },
                                                    roving: RovingFocusProps {
                                                        enabled: true,
                                                        wrap: true,
                                                        disabled: disabled_arc.clone(),
                                                        ..Default::default()
                                                    },
                                                },
                                                labels_arc.clone(),
                                                typeahead_timeout_ticks,
                                                move |cx| {
                                                    let font_size = theme.metrics.font_size;
                                                    let font_line_height = theme.metrics.font_line_height;
                                                    let radius_sm = theme.metrics.radius_sm;
                                                    let text_disabled = theme.colors.text_disabled;
                                                    let icon_muted_fg = theme
                                                        .color_by_key("muted.foreground")
                                                        .or_else(|| theme.color_by_key("muted-foreground"))
                                                        .unwrap_or(theme.colors.text_muted);
                                                    let destructive_fg = theme
                                                        .color_by_key("destructive")
                                                        .or_else(|| {
                                                            theme.color_by_key("destructive.background")
                                                        })
                                                        .unwrap_or(theme.colors.text_primary);
                                                    let destructive_bg = alpha_mul(destructive_fg, 0.10);

                                                    let text_style = TextStyle {
                                                        font: fret_core::FontId::default(),
                                                        size: font_size,
                                                        weight: fret_core::FontWeight::NORMAL,
                                                        line_height: Some(font_line_height),
                                                        letter_spacing_em: None,
                                                    };

                                                    let mut out: Vec<AnyElement> =
                                                        Vec::with_capacity(entries.len());

                                                    let mut item_ix: usize = 0;
                                                    for entry in entries.clone() {
                                                        match entry {
                                                    DropdownMenuEntry::Label(label) => {
                                                        let fg = theme
                                                            .color_by_key("muted.foreground")
                                                            .or_else(|| theme.color_by_key("muted-foreground"))
                                                            .unwrap_or(theme.colors.text_muted);
                                                        let text = label.text.clone();
                                                        let pad_left =
                                                            if label.inset { pad_x_inset } else { pad_x };
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: LayoutStyle::default(),
                                                                padding: Edges {
                                                                    top: pad_y,
                                                                    right: pad_x,
                                                                    bottom: pad_y,
                                                                    left: pad_left,
                                                                },
                                                                ..Default::default()
                                                            },
                                                            move |cx| {
                                                                vec![cx.text_props(TextProps {
                                                                    layout: LayoutStyle::default(),
                                                                    text,
                                                                    style: Some(TextStyle {
                                                                        font: FontId::default(),
                                                                        size: font_size,
                                                                        weight: FontWeight::MEDIUM,
                                                                        line_height: Some(
                                                                            font_line_height,
                                                                        ),
                                                                        letter_spacing_em: None,
                                                                    }),
                                                                    wrap: TextWrap::None,
                                                                    overflow: TextOverflow::Clip,
                                                                    color: Some(fg),
                                                                })]
                                                            },
                                                        ));
                                                    }
                                                    DropdownMenuEntry::Group(_) => {
                                                        unreachable!("groups are flattened")
                                                    }
                                                    DropdownMenuEntry::RadioGroup(_) => {
                                                        unreachable!("radio groups are flattened")
                                                    }
                                                    DropdownMenuEntry::Separator => {
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width =
                                                                        Length::Fill;
                                                                    layout.size.height =
                                                                        Length::Px(Px(1.0));
                                                                    // new-york-v4: `-mx-1 my-1`.
                                                                    layout.margin.left =
                                                                        fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                    layout.margin.right =
                                                                        fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                    layout.margin.top =
                                                                        fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                    layout.margin.bottom =
                                                                        fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                    layout
                                                                },
                                                                padding: Edges::all(Px(0.0)),
                                                                background: Some(border),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ));
                                                            }
                                                    DropdownMenuEntry::CheckboxItem(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let checked = item.checked.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let disabled = item.disabled;
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let leading = item.leading.clone();
                                                        let trailing = item.trailing.clone();
                                                        let open = open_for_menu.clone();
                                                        let text_style = text_style.clone();
                                                        let submenu_for_item =
                                                            submenu_for_content.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(
                                                                |cx, st, item_id| {
                                                                    let checked_now = cx
                                                                        .watch_model(&checked)
                                                                        .copied()
                                                                        .unwrap_or(false);

                                                                    let _ = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        disabled,
                                                                        false,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        None,
                                                                    );

                                                                    if !disabled {
                                                                        menu::checkbox_item::wire_toggle_on_activate(
                                                                            cx,
                                                                            checked.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_opt(command);
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(
                                                                                &open,
                                                                                false,
                                                                            );
                                                                        }
                                                                    }

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Px(28.0));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: menu::item::menu_item_checkbox_a11y(
                                                                            a11y_label,
                                                                            checked_now,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered
                                                                        || st.pressed
                                                                        || st.focused
                                                                    {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = checkable_menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing.clone(),
                                                                        CheckableIndicatorKind::Check,
                                                                        checked_now,
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x_inset,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    );

                                                                    (props, children)
                                                                },
                                                            )
                                                        }));
                                                    }
                                                    DropdownMenuEntry::RadioItem(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let group_value = item.group_value.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let disabled = item.disabled;
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let leading = item.leading.clone();
                                                        let trailing = item.trailing.clone();
                                                        let open = open_for_menu.clone();
                                                        let text_style = text_style.clone();
                                                        let submenu_for_item =
                                                            submenu_for_content.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(
                                                                |cx, st, item_id| {
                                                                    let selected = cx
                                                                        .watch_model(&group_value)
                                                                        .cloned()
                                                                        .flatten();
                                                                    let is_selected =
                                                                        menu::radio_group::is_selected(
                                                                            selected.as_ref(),
                                                                            &value,
                                                                        );

                                                                    let _ = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        disabled,
                                                                        false,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        None,
                                                                    );

                                                                    if !disabled {
                                                                        menu::radio_group::wire_select_on_activate(
                                                                            cx,
                                                                            group_value.clone(),
                                                                            value.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_opt(command);
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(
                                                                                &open,
                                                                                false,
                                                                            );
                                                                        }
                                                                    }

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Px(28.0));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: menu::item::menu_item_radio_a11y(
                                                                            a11y_label,
                                                                            is_selected,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered
                                                                        || st.pressed
                                                                        || st.focused
                                                                    {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = checkable_menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing.clone(),
                                                                        CheckableIndicatorKind::RadioDot,
                                                                        is_selected,
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x_inset,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    );

                                                                    (props, children)
                                                                },
                                                            )
                                                        }));
                                                    }
                                                    DropdownMenuEntry::Item(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let disabled = item.disabled;
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let leading = item.leading.clone();
                                                        let trailing = item.trailing.clone();
                                                        let variant = item.variant;
                                                        let has_submenu = item.submenu.is_some();
                                                        let pad_left =
                                                            if item.inset { pad_x_inset } else { pad_x };
                                                        let open = open_for_menu.clone();
                                                        let text_style = text_style.clone();
                                                        let submenu_for_item =
                                                            submenu_for_content.clone();

                                                                out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(|cx, st, item_id| {
                                                                let geometry_hint = has_submenu.then(|| {
                                                                    let outer =
                                                                        overlay::outer_bounds_with_window_margin(
                                                                            cx.bounds,
                                                                            window_margin,
                                                                        );
                                                                    let desired = Size::new(
                                                                        Px(192.0),
                                                                        Px(1.0e9),
                                                                    );
                                                                    menu::sub_trigger::MenuSubTriggerGeometryHint {
                                                                        outer,
                                                                        desired,
                                                                    }
                                                                });
                                                                let is_open_submenu =
                                                                    menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        disabled,
                                                                        has_submenu,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        geometry_hint,
                                                                    )
                                                                    .unwrap_or(false);

                                                                if !has_submenu && !disabled {
                                                                    cx.pressable_dispatch_command_opt(command);
                                                                    if close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }
                                                                }

                                                                let props = PressableProps {
                                                                    layout: {
                                                                        let mut layout = LayoutStyle::default();
                                                                        layout.size.width = Length::Fill;
                                                                        layout.size.min_height = Some(Px(28.0));
                                                                        layout
                                                                    },
                                                                    enabled: !disabled,
                                                                    focusable: !disabled,
                                                                    focus_ring: Some(ring),
                                                                    a11y: menu::item::menu_item_a11y(
                                                                        a11y_label,
                                                                        has_submenu
                                                                            .then_some(is_open_submenu),
                                                                    )
                                                                    .with_collection_position(
                                                                        collection_index,
                                                                        item_count,
                                                                    ),
                                                                    ..Default::default()
                                                                };

                                                                let mut row_bg = fret_core::Color::TRANSPARENT;
                                                                let mut row_fg = if variant == DropdownMenuItemVariant::Destructive {
                                                                    destructive_fg
                                                                } else {
                                                                    fg
                                                                };
                                                                if st.hovered
                                                                    || st.pressed
                                                                    || st.focused
                                                                    || is_open_submenu
                                                                {
                                                                    if variant
                                                                        == DropdownMenuItemVariant::Destructive
                                                                    {
                                                                        row_bg = destructive_bg;
                                                                        row_fg = destructive_fg;
                                                                    } else {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }
                                                                }

                                                                let children = vec![cx.container(
                                                                        ContainerProps {
                                                                            layout: LayoutStyle::default(),
                                                                            padding: Edges {
                                                                                top: pad_y,
                                                                                right: pad_x,
                                                                                bottom: pad_y,
                                                                                left: pad_left,
                                                                            },
                                                                            background: Some(row_bg),
                                                                            corner_radii: fret_core::Corners::all(radius_sm),
                                                                            ..Default::default()
                                                                        },
                                                                    move |cx| {
                                                                        let mut row: Vec<AnyElement> = Vec::with_capacity(
                                                                            2 + usize::from(
                                                                                leading.is_some()
                                                                                    || reserve_leading_slot,
                                                                            ) + usize::from(trailing.is_some())
                                                                                + usize::from(has_submenu),
                                                                        );
                                                                        if let Some(l) = leading.clone() {
                                                                            row.push(menu_icon_slot(cx, l));
                                                                        } else if reserve_leading_slot {
                                                                            row.push(menu_icon_slot_empty(cx));
                                                                        }
                                                                        row.push(cx.text_props(TextProps {
                                                                            layout: {
                                                                                let mut layout = LayoutStyle::default();
                                                                                layout.size.width = Length::Fill;
                                                                                layout.size.min_width = Some(Px(0.0));
                                                                                layout.flex.grow = 1.0;
                                                                                layout.flex.shrink = 1.0;
                                                                                layout.flex.basis = Length::Px(Px(0.0));
                                                                                layout
                                                                            },
                                                                            text: label.clone(),
                                                                            style: Some(text_style.clone()),
                                                                            wrap: TextWrap::None,
                                                                            overflow: TextOverflow::Clip,
                                                                            color: Some(if disabled { text_disabled } else { row_fg }),
                                                                        }));

                                                                        if let Some(t) = trailing.clone() {
                                                                            row.push(t);
                                                                        }
                                                                        if has_submenu {
                                                                            row.push(submenu_chevron_right_text(
                                                                                cx,
                                                                                if disabled {
                                                                                    text_disabled
                                                                                } else {
                                                                                    icon_muted_fg
                                                                                },
                                                                                font_size,
                                                                                font_line_height,
                                                                            ));
                                                                        }

                                                                        vec![cx.flex(
                                                                            FlexProps {
                                                                                layout: {
                                                                                    let mut layout = LayoutStyle::default();
                                                                                    layout.size.width = Length::Fill;
                                                                                    layout
                                                                                },
                                                                                direction: fret_core::Axis::Horizontal,
                                                                                gap: Px(8.0),
                                                                                padding: Edges::all(Px(0.0)),
                                                                                justify: MainAlign::Start,
                                                                                align: CrossAlign::Center,
                                                                                wrap: false,
                                                                            },
                                                                            move |_cx| row.clone(),
                                                                        )]
                                                                    },
                                                                )];

                                                                (props, children)
                                                            })
                                                        }));
                                                            }
                                                        }
                                                    }

                                                    out
                                                },
                                            );
                                            if content_focus_id_for_scroll.get().is_none() {
                                                content_focus_id_for_scroll.set(Some(roving.id));
                                            }
                                            vec![roving]
                                        },
                                    )]
                                        },
                                    );

                                    if let Some(arrow_el) = arrow_el {
                                        vec![arrow_el, panel]
                                    } else {
                                        vec![panel]
                                    }
                                },
                            )]
                        },
                    );

                    let opacity_layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let content = cx.opacity_props(
                        OpacityProps {
                            layout: opacity_layout.clone(),
                            opacity,
                        },
                        move |cx| {
                            vec![cx.visual_transform_props(
                                VisualTransformProps {
                                    layout: opacity_layout,
                                    transform,
                                },
                                move |_cx| vec![content],
                            )]
                        },
                    );

                    let dismissible_on_pointer_move =
                        menu::root::submenu_pointer_move_handler(submenu.clone(), submenu_cfg);

                    let mut children = vec![content];
                    let desired = Size::new(Px(192.0), Px(1.0e9));
                    let open_submenu = menu::sub::with_open_submenu(
                        cx,
                        &submenu_for_panel,
                        outer,
                        desired,
                        |_cx, open_value, geometry| (open_value, geometry.floating),
                    );

                    if let Some((open_value, placed)) = open_submenu {
                        let submenu_entries = entries_for_submenu.iter().find_map(|e| {
                            let DropdownMenuEntry::Item(item) = e else {
                                return None;
                            };
                            let Some(sub) = item.submenu.clone() else {
                                return None;
                            };
                            (item.value.as_ref() == open_value.as_ref()).then_some(sub)
                        });

                        if let Some(submenu_entries) = submenu_entries {
                                        let mut flat: Vec<DropdownMenuEntry> = Vec::new();
                                        flatten_entries(&mut flat, submenu_entries);
                                        let submenu_entries = flat;
                                        let reserve_leading_slot = align_leading_icons
                                            && submenu_entries.iter().any(|e| match e {
                                                DropdownMenuEntry::Item(item) => item.leading.is_some(),
                                                DropdownMenuEntry::CheckboxItem(item) => {
                                                    item.leading.is_some()
                                                }
                                                DropdownMenuEntry::RadioItem(item) => item.leading.is_some(),
                                                DropdownMenuEntry::Label(_)
                                                | DropdownMenuEntry::Group(_)
                                                | DropdownMenuEntry::RadioGroup(_)
                                                | DropdownMenuEntry::Separator => false,
                                            });
                                        let item_count = submenu_entries
                                            .iter()
                                            .filter(|e| {
                                                matches!(
                                                    e,
                                                    DropdownMenuEntry::Item(_)
                                                        | DropdownMenuEntry::CheckboxItem(_)
                                                        | DropdownMenuEntry::RadioItem(_)
                                                )
                                            })
                                            .count();

                                            let font_size = theme.metrics.font_size;
                                            let font_line_height = theme.metrics.font_line_height;
                                            let radius_sm = theme.metrics.radius_sm;
                                            let text_disabled = theme.colors.text_disabled;
                                            let destructive_fg = theme
                                                .color_by_key("destructive")
                                                .or_else(|| theme.color_by_key("destructive.background"))
                                                .unwrap_or(theme.colors.text_primary);
                                            let destructive_bg = alpha_mul(destructive_fg, 0.10);
                                            let label_fg = theme
                                                .color_by_key("muted.foreground")
                                                .or_else(|| theme.color_by_key("muted-foreground"))
                                                .unwrap_or(theme.colors.text_muted);

                                            let text_style = TextStyle {
                                                font: FontId::default(),
                                                size: font_size,
                                                weight: FontWeight::NORMAL,
                                                line_height: Some(font_line_height),
                                                letter_spacing_em: None,
                                            };

                                            let (submenu_labels, submenu_disabled_flags): (
                                                Vec<Arc<str>>,
                                                Vec<bool>,
                                            ) = submenu_entries
                                                .iter()
                                                .map(|e| match e {
                                                    DropdownMenuEntry::Item(item) => {
                                                        (item.label.clone(), item.disabled)
                                                    }
                                                    DropdownMenuEntry::CheckboxItem(item) => {
                                                        (item.label.clone(), item.disabled)
                                                    }
                                                    DropdownMenuEntry::RadioItem(item) => {
                                                        (item.label.clone(), item.disabled)
                                                    }
                                                    DropdownMenuEntry::Label(_)
                                                    | DropdownMenuEntry::Separator => {
                                                        (Arc::from(""), true)
                                                    }
                                                    DropdownMenuEntry::Group(_)
                                                    | DropdownMenuEntry::RadioGroup(_) => {
                                                        unreachable!("entries are flattened")
                                                    }
                                                })
                                                .unzip();
                                            let submenu_labels_arc: Arc<[Arc<str>]> =
                                                Arc::from(submenu_labels.into_boxed_slice());
                                            let submenu_disabled_arc: Arc<[bool]> = Arc::from(
                                                submenu_disabled_flags.into_boxed_slice(),
                                            );
                                            let roving = RovingFocusProps {
                                                enabled: true,
                                                wrap: true,
                                                disabled: submenu_disabled_arc,
                                                ..Default::default()
                                            };

                                            let submenu_models_for_panel = submenu_for_panel.clone();
                                            let submenu_panel = menu::sub_content::submenu_panel_at(
                                                cx,
                                                placed,
                                                move |layout| ContainerProps {
                                                    layout,
                                                    padding: Edges::all(Px(4.0)),
                                                    background: Some(bg),
                                                    shadow: Some(shadow_submenu),
                                                    border: Edges::all(Px(1.0)),
                                                    border_color: Some(border),
                                                    corner_radii: fret_core::Corners::all(
                                                        theme.metrics.radius_sm,
                                                    ),
                                                },
                                                move |cx| {
                                                    let mut item_ix: usize = 0;
                                                    let mut rows: Vec<AnyElement> =
                                                        Vec::with_capacity(submenu_entries.len());

                                                    for entry in submenu_entries.clone() {
                                                        match entry {
                                                            DropdownMenuEntry::Label(label) => {
                                                                let text = label.text.clone();
                                                                let pad_left = if label.inset {
                                                                    pad_x_inset
                                                                } else {
                                                                    pad_x
                                                                };
                                                                rows.push(cx.container(
                                                                    ContainerProps {
                                                                        layout: LayoutStyle::default(),
                                                                        padding: Edges {
                                                                            top: pad_y,
                                                                            right: pad_x,
                                                                            bottom: pad_y,
                                                                            left: pad_left,
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| {
                                                                        vec![cx.text_props(TextProps {
                                                                            layout: LayoutStyle::default(),
                                                                            text,
                                                                            style: Some(TextStyle {
                                                                                font: FontId::default(),
                                                                                size: font_size,
                                                                                weight: FontWeight::MEDIUM,
                                                                                line_height: Some(font_line_height),
                                                                                letter_spacing_em: None,
                                                                            }),
                                                                            wrap: TextWrap::None,
                                                                            overflow: TextOverflow::Clip,
                                                                            color: Some(label_fg),
                                                                        })]
                                                                    },
                                                                ));
                                                            }
                                                            DropdownMenuEntry::Group(_) => {
                                                                unreachable!("groups are flattened")
                                                            }
                                                            DropdownMenuEntry::RadioGroup(_) => {
                                                                unreachable!("radio groups are flattened")
                                                            }
                                                            DropdownMenuEntry::Separator => {
                                                                rows.push(cx.container(
                                                                    ContainerProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.height =
                                                                                Length::Px(Px(1.0));
                                                                            // new-york-v4: `-mx-1 my-1`.
                                                                            layout.margin.left =
                                                                                fret_ui::element::MarginEdge::Px(
                                                                                    Px(-4.0),
                                                                                );
                                                                            layout.margin.right =
                                                                                fret_ui::element::MarginEdge::Px(
                                                                                    Px(-4.0),
                                                                                );
                                                                            layout.margin.top =
                                                                                fret_ui::element::MarginEdge::Px(
                                                                                    Px(4.0),
                                                                                );
                                                                            layout.margin.bottom =
                                                                                fret_ui::element::MarginEdge::Px(
                                                                                    Px(4.0),
                                                                                );
                                                                            layout
                                                                        },
                                                                        padding: Edges::all(Px(0.0)),
                                                                        background: Some(border),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ));
                                                            }
                                                            DropdownMenuEntry::CheckboxItem(item) => {
                                                                let collection_index = item_ix;
                                                                item_ix = item_ix.saturating_add(1);

                                                                let label = item.label.clone();
                                                                let value = item.value.clone();
                                                                let checked = item.checked.clone();
                                                                let a11y_label = item
                                                                    .a11y_label
                                                                    .clone()
                                                                    .or_else(|| Some(label.clone()));
                                                                let disabled = item.disabled;
                                                                let close_on_select = item.close_on_select;
                                                                let command = item.command;
                                                                let leading = item.leading.clone();
                                                                let trailing = item.trailing.clone();
                                                                let open = open_for_submenu.clone();
                                                                let submenu_for_key =
                                                                    submenu_models_for_panel.clone();
                                                                let text_style = text_style.clone();

                                                                rows.push(cx.keyed(value.clone(), |cx| {
                                                                    cx.pressable_with_id_props(
                                                                        |cx, st, item_id| {
                                                                            menu::sub_content::wire_item(
                                                                                cx,
                                                                                item_id,
                                                                                disabled,
                                                                                &submenu_for_key,
                                                                            );

                                                                            let checked_now = cx
                                                                                .watch_model(&checked)
                                                                                .copied()
                                                                                .unwrap_or(false);
                                                                            if !disabled {
                                                                                menu::checkbox_item::wire_toggle_on_activate(
                                                                                    cx,
                                                                                    checked.clone(),
                                                                                );
                                                                            }
                                                                            cx.pressable_dispatch_command_opt(command);
                                                                            if !disabled && close_on_select {
                                                                                cx.pressable_set_bool(&open, false);
                                                                            }

                                                                            let props = PressableProps {
                                                                                layout: {
                                                                                    let mut layout = LayoutStyle::default();
                                                                                    layout.size.width = Length::Fill;
                                                                                    layout.size.min_height = Some(Px(28.0));
                                                                                    layout
                                                                                },
                                                                                enabled: !disabled,
                                                                                focusable: !disabled,
                                                                                focus_ring: Some(ring),
                                                                                a11y: menu::item::menu_item_checkbox_a11y(
                                                                                    a11y_label,
                                                                                    checked_now,
                                                                                )
                                                                                .with_collection_position(
                                                                                    collection_index,
                                                                                    item_count,
                                                                                ),
                                                                                ..Default::default()
                                                                            };

                                                                            let mut row_bg =
                                                                                fret_core::Color::TRANSPARENT;
                                                                            let mut row_fg = fg;
                                                                            if st.hovered || st.pressed || st.focused {
                                                                                row_bg = accent;
                                                                                row_fg = accent_fg;
                                                                            }

                                                                            let children = checkable_menu_row_children(
                                                                                cx,
                                                                                label.clone(),
                                                                                leading.clone(),
                                                                                reserve_leading_slot,
                                                                                trailing.clone(),
                                                                                CheckableIndicatorKind::Check,
                                                                                checked_now,
                                                                                disabled,
                                                                                row_bg,
                                                                                row_fg,
                                                                                text_style.clone(),
                                                                                font_size,
                                                                                font_line_height,
                                                                                pad_x,
                                                                                pad_x_inset,
                                                                                pad_y,
                                                                                radius_sm,
                                                                                text_disabled,
                                                                            );

                                                                            (props, children)
                                                                        },
                                                                    )
                                                                }));
                                                            }
                                                            DropdownMenuEntry::RadioItem(item) => {
                                                                let collection_index = item_ix;
                                                                item_ix = item_ix.saturating_add(1);

                                                                let label = item.label.clone();
                                                                let value = item.value.clone();
                                                                let group_value = item.group_value.clone();
                                                                let a11y_label = item
                                                                    .a11y_label
                                                                    .clone()
                                                                    .or_else(|| Some(label.clone()));
                                                                let disabled = item.disabled;
                                                                let close_on_select = item.close_on_select;
                                                                let command = item.command;
                                                                let leading = item.leading.clone();
                                                                let trailing = item.trailing.clone();
                                                                let open = open_for_submenu.clone();
                                                                let submenu_for_key =
                                                                    submenu_models_for_panel.clone();
                                                                let text_style = text_style.clone();

                                                                rows.push(cx.keyed(value.clone(), |cx| {
                                                                    cx.pressable_with_id_props(
                                                                        |cx, st, item_id| {
                                                                            menu::sub_content::wire_item(
                                                                                cx,
                                                                                item_id,
                                                                                disabled,
                                                                                &submenu_for_key,
                                                                            );

                                                                            let selected = cx
                                                                                .watch_model(&group_value)
                                                                                .cloned()
                                                                                .flatten();
                                                                            let is_selected = menu::radio_group::is_selected(
                                                                                selected.as_ref(),
                                                                                &value,
                                                                            );
                                                                            if !disabled {
                                                                                menu::radio_group::wire_select_on_activate(
                                                                                    cx,
                                                                                    group_value.clone(),
                                                                                    value.clone(),
                                                                                );
                                                                            }
                                                                            cx.pressable_dispatch_command_opt(command);
                                                                            if !disabled && close_on_select {
                                                                                cx.pressable_set_bool(&open, false);
                                                                            }

                                                                            let props = PressableProps {
                                                                                layout: {
                                                                                    let mut layout = LayoutStyle::default();
                                                                                    layout.size.width = Length::Fill;
                                                                                    layout.size.min_height = Some(Px(28.0));
                                                                                    layout
                                                                                },
                                                                                enabled: !disabled,
                                                                                focusable: !disabled,
                                                                                focus_ring: Some(ring),
                                                                                a11y: menu::item::menu_item_radio_a11y(
                                                                                    a11y_label,
                                                                                    is_selected,
                                                                                )
                                                                                .with_collection_position(
                                                                                    collection_index,
                                                                                    item_count,
                                                                                ),
                                                                                ..Default::default()
                                                                            };

                                                                            let mut row_bg =
                                                                                fret_core::Color::TRANSPARENT;
                                                                            let mut row_fg = fg;
                                                                            if st.hovered || st.pressed || st.focused {
                                                                                row_bg = accent;
                                                                                row_fg = accent_fg;
                                                                            }

                                                                            let children = checkable_menu_row_children(
                                                                                cx,
                                                                                label.clone(),
                                                                                leading.clone(),
                                                                                reserve_leading_slot,
                                                                                trailing.clone(),
                                                                                CheckableIndicatorKind::RadioDot,
                                                                                is_selected,
                                                                                disabled,
                                                                                row_bg,
                                                                                row_fg,
                                                                                text_style.clone(),
                                                                                font_size,
                                                                                font_line_height,
                                                                                pad_x,
                                                                                pad_x_inset,
                                                                                pad_y,
                                                                                radius_sm,
                                                                                text_disabled,
                                                                            );

                                                                            (props, children)
                                                                        },
                                                                    )
                                                                }));
                                                            }
                                                            DropdownMenuEntry::Item(item) => {
                                                                let collection_index = item_ix;
                                                                item_ix = item_ix.saturating_add(1);

                                                                let label = item.label.clone();
                                                                let value = item.value.clone();
                                                                let a11y_label = item
                                                                    .a11y_label
                                                                    .clone()
                                                                    .or_else(|| Some(label.clone()));
                                                                let disabled = item.disabled;
                                                                let close_on_select = item.close_on_select;
                                                                let command = item.command;
                                                                let leading = item.leading.clone();
                                                                let trailing = item.trailing.clone();
                                                                let variant = item.variant;
                                                                let pad_left =
                                                                    if item.inset { pad_x_inset } else { pad_x };
                                                                let open = open_for_submenu.clone();
                                                                let submenu_for_key =
                                                                    submenu_models_for_panel.clone();
                                                                let text_style = text_style.clone();

                                                                        rows.push(cx.keyed(value.clone(), |cx| {
                                                                    cx.pressable_with_id_props(
                                                                        |cx, st, item_id| {
                                                                            menu::sub_content::wire_item(
                                                                                cx,
                                                                                item_id,
                                                                                disabled,
                                                                                &submenu_for_key,
                                                                            );
                                                                            cx.pressable_dispatch_command_opt(command);
                                                                            if !disabled && close_on_select {
                                                                                cx.pressable_set_bool(&open, false);
                                                                            }

                                                                            let props = PressableProps {
                                                                                layout: {
                                                                                    let mut layout = LayoutStyle::default();
                                                                                    layout.size.width = Length::Fill;
                                                                                    layout.size.min_height = Some(Px(28.0));
                                                                                    layout
                                                                                },
                                                                                enabled: !disabled,
                                                                                focusable: !disabled,
                                                                                focus_ring: Some(ring),
                                                                                a11y: menu::item::menu_item_a11y(
                                                                                    a11y_label,
                                                                                    None,
                                                                                )
                                                                                .with_collection_position(
                                                                                    collection_index,
                                                                                    item_count,
                                                                                ),
                                                                                ..Default::default()
                                                                            };

                                                                            let mut row_bg =
                                                                                fret_core::Color::TRANSPARENT;
                                                                            let mut row_fg = if variant
                                                                                == DropdownMenuItemVariant::Destructive
                                                                            {
                                                                                destructive_fg
                                                                            } else {
                                                                                fg
                                                                            };
                                                                            if st.hovered || st.pressed || st.focused {
                                                                                if variant
                                                                                    == DropdownMenuItemVariant::Destructive
                                                                                {
                                                                                    row_bg = destructive_bg;
                                                                                    row_fg = destructive_fg;
                                                                                } else {
                                                                                    row_bg = accent;
                                                                                    row_fg = accent_fg;
                                                                                }
                                                                            }

                                                                            let children = vec![cx.container(
                                                                                ContainerProps {
                                                                                    layout: LayoutStyle::default(),
                                                                                    padding: Edges {
                                                                                        top: pad_y,
                                                                                        right: pad_x,
                                                                                        bottom: pad_y,
                                                                                        left: pad_left,
                                                                                    },
                                                                                    background: Some(row_bg),
                                                                                    corner_radii: fret_core::Corners::all(radius_sm),
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx| {
                                                                                    let mut row: Vec<AnyElement> = Vec::with_capacity(
                                                                                        1 + usize::from(
                                                                                            leading.is_some()
                                                                                                || reserve_leading_slot,
                                                                                        ) + usize::from(trailing.is_some()),
                                                                                    );
                                                                                    if let Some(l) = leading.clone() {
                                                                                        row.push(menu_icon_slot(cx, l));
                                                                                    } else if reserve_leading_slot {
                                                                                        row.push(menu_icon_slot_empty(cx));
                                                                                    }
                                                                                    row.push(cx.text_props(TextProps {
                                                                                        layout: {
                                                                                            let mut layout = LayoutStyle::default();
                                                                                            layout.size.width = Length::Fill;
                                                                                            layout.size.min_width = Some(Px(0.0));
                                                                                            layout.flex.grow = 1.0;
                                                                                            layout.flex.shrink = 1.0;
                                                                                            layout.flex.basis = Length::Px(Px(0.0));
                                                                                            layout
                                                                                        },
                                                                                        text: label.clone(),
                                                                                        style: Some(text_style.clone()),
                                                                                        wrap: TextWrap::None,
                                                                                        overflow: TextOverflow::Clip,
                                                                                        color: Some(if disabled { text_disabled } else { row_fg }),
                                                                                    }));

                                                                                    if let Some(t) = trailing.clone() {
                                                                                        row.push(t);
                                                                                    }

                                                                                    vec![cx.flex(
                                                                                        FlexProps {
                                                                                            layout: {
                                                                                                let mut layout = LayoutStyle::default();
                                                                                                layout.size.width = Length::Fill;
                                                                                                layout
                                                                                            },
                                                                                            direction: fret_core::Axis::Horizontal,
                                                                                            gap: Px(8.0),
                                                                                            padding: Edges::all(Px(0.0)),
                                                                                            justify: MainAlign::Start,
                                                                                            align: CrossAlign::Center,
                                                                                            wrap: false,
                                                                                        },
                                                                                        move |_cx| row.clone(),
                                                                                    )]
                                                                                },
                                                                            )];

                                                                            (props, children)
                                                                        },
                                                                    )
                                                                }));
                                                            }
                                                        }
                                                    }

                                                    vec![menu::sub_content::submenu_roving_group_apg_prefix_typeahead(
                                                        cx,
                                                        RovingFlexProps {
                                                            flex: FlexProps {
                                                                layout: LayoutStyle::default(),
                                                                direction: fret_core::Axis::Vertical,
                                                                gap: Px(0.0),
                                                                padding: Edges::all(Px(0.0)),
                                                                justify: MainAlign::Start,
                                                                align: CrossAlign::Stretch,
                                                                wrap: false,
                                                            },
                                                            roving,
                                                        },
                                                        submenu_labels_arc.clone(),
                                                        typeahead_timeout_ticks,
                                                        move |_cx| rows.clone(),
                                                    )]
                                                },
                                            );

                                        children.push(submenu_panel);
                                    }
                                }

                    (children, Some(dismissible_on_pointer_move))
                });

                let request = menu::root::dismissible_menu_request(
                    cx,
                    trigger_id,
                    trigger_id,
                    open,
                    overlay_presence,
                    overlay_children,
                    overlay_root_name,
                    content_focus_id.get(),
                    dismissible_on_pointer_move,
                );
                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, MouseButtons, PathCommand, Point, PointerEvent,
        Rect, SvgId, SvgService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, SemanticsRole, Size as CoreSize};
    use fret_core::{
        TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::{Effect, FrameId};
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(0.0), Px(0.0)),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        entries: Vec<DropdownMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu",
            move |cx| {
                vec![DropdownMenu::new(open).into_element(
                    cx,
                    |cx| {
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )
                    },
                    move |_cx| entries,
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_capture_trigger_id(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        trigger_id_out: Model<Option<fret_ui::elements::GlobalElementId>>,
        entries: Vec<DropdownMenuEntry>,
    ) -> (fret_core::NodeId, fret_ui::elements::GlobalElementId) {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let trigger_id_out_for_render = trigger_id_out.clone();
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu",
            move |cx| {
                let trigger_id_out = trigger_id_out_for_render.clone();
                vec![DropdownMenu::new(open).into_element(
                    cx,
                    move |cx| {
                        let trigger = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        );
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&trigger_id_out, |v| *v = Some(trigger.id));
                        trigger
                    },
                    move |_cx| entries,
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);

        let trigger_id = app
            .models_mut()
            .read(&trigger_id_out, |v| *v)
            .ok()
            .flatten()
            .expect("captured trigger element id");
        (root, trigger_id)
    }

    fn render_frame_focusable_trigger(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        entries: Vec<DropdownMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu-trigger",
            move |cx| {
                vec![DropdownMenu::new(open).into_element(
                    cx,
                    |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st| {
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        )
                    },
                    move |_cx| entries,
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_clipped_surface(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        entries: Vec<DropdownMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu-clipped-surface",
            move |cx| {
                let clipped_surface = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(200.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.overflow = Overflow::Clip;
                            layout
                        },
                        ..Default::default()
                    },
                    |cx| {
                        vec![DropdownMenu::new(open).into_element(
                            cx,
                            |cx| {
                                cx.container(
                                    ContainerProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(120.0));
                                            layout.size.height = Length::Px(Px(20.0));
                                            layout.position =
                                                fret_ui::element::PositionStyle::Absolute;
                                            layout.inset.top = Some(Px(30.0));
                                            layout.inset.left = Some(Px(10.0));
                                            layout
                                        },
                                        ..Default::default()
                                    },
                                    |_cx| Vec::new(),
                                )
                            },
                            move |_cx| entries,
                        )]
                    },
                );
                vec![clipped_surface]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        entries: Vec<DropdownMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu-underlay",
            move |cx| {
                let trigger_id_out = trigger_id_out.clone();
                let underlay_id_out = underlay_id_out.clone();

                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Relative;
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let underlay = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(380.0));
                                    layout.inset.top = Some(Px(200.0));
                                    layout.size.width = Length::Px(Px(220.0));
                                    layout.size.height = Length::Px(Px(120.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            {
                                let underlay_id_out = underlay_id_out.clone();
                                move |cx, _st, id| {
                                    underlay_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                }
                            },
                        );

                        let dropdown_menu = DropdownMenu::new(open).into_element(
                            cx,
                            {
                                let trigger_id_out = trigger_id_out.clone();
                                move |cx| {
                                    cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.position =
                                                    fret_ui::element::PositionStyle::Absolute;
                                                layout.inset.left = Some(Px(0.0));
                                                layout.inset.top = Some(Px(0.0));
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            trigger_id_out.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    )
                                }
                            },
                            move |_cx| entries,
                        );

                        vec![underlay, dropdown_menu]
                    },
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn dropdown_menu_items_have_collection_position_metadata_excluding_separators() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha")),
                DropdownMenuEntry::Separator,
                DropdownMenuEntry::Item(DropdownMenuItem::new("Beta")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Gamma")),
            ],
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha")),
                DropdownMenuEntry::Separator,
                DropdownMenuEntry::Item(DropdownMenuItem::new("Beta")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Gamma")),
            ],
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Beta"))
            .expect("Beta menu item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }

    #[test]
    fn dropdown_menu_portal_escapes_overflow_clip_ancestor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: closed, establish trigger bounds for placement.
        let _ = render_frame_clipped_surface(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open and ensure the menu item is hit-testable outside the clipped surface.
        let _ = render_frame_clipped_surface(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snapshot
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");

        let clip_bottom = 40.0f32;
        let point = Point::new(
            Px(alpha.bounds.origin.x.0 + 2.0),
            Px(alpha.bounds.origin.y.0 + 2.0),
        );
        assert!(
            point.y.0 > clip_bottom,
            "expected menu item to be outside clipped surface; y={} clip_bottom={}",
            point.y.0,
            clip_bottom
        );

        let hit = ui
            .debug_hit_test(point)
            .hit
            .expect("expected hit in dropdown menu outside clipped surface");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&alpha.id),
            "expected hit to be within Alpha menu item subtree; point={point:?} hit={hit:?} alpha={:?} path={path:?}",
            alpha.id
        );
    }

    #[test]
    fn dropdown_menu_opens_on_arrow_down_from_focused_trigger() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))];

        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha")),
            "menu items should render after ArrowDown opens the menu"
        );
    }

    #[test]
    fn dropdown_menu_escape_closes_and_restores_trigger_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: closed, capture trigger id.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        // Open via keyboard.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open, focus should land on the first item (keyboard modality).
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");
        assert_eq!(ui.focus(), Some(alpha.id));

        // Escape should close and restore focus to trigger.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Frame 3: closed, focus restored.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            trigger_id_out,
            underlay_id_out,
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dropdown_menu_item_select_closes_and_restores_trigger_focus() {
        use fret_core::MouseButton;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: closed, capture trigger id.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        // Open via keyboard and select first item with pointer.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");
        let position = Point::new(
            Px(alpha.bounds.origin.x.0 + 2.0),
            Px(alpha.bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Next frame: closed, focus restored.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            trigger_id_out,
            underlay_id_out,
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dropdown_menu_outside_press_closes_without_overriding_underlay_focus() {
        use fret_core::MouseButton;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: closed, establish stable trigger bounds.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open, pointer modality should focus inside the menu content (not the trigger).
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move inside the menu when opened in pointer modality"
        );

        // Click the underlay while the menu is open: should close via observer pass, but must not
        // activate or focus the underlay (non-click-through dismissal).
        let position = Point::new(Px(410.0), Px(310.0));
        let underlay_id = underlay_id_out.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");

        let underlay_bounds = ui
            .debug_node_bounds(underlay_node)
            .expect("underlay bounds");
        assert!(
            underlay_bounds.contains(position),
            "expected click position to fall inside underlay bounds; pos={position:?} bounds={underlay_bounds:?}"
        );

        let hit = ui.debug_hit_test(position).hit;
        let hit_path = hit.map(|hit| ui.debug_node_path(hit));
        assert!(
            hit_path
                .as_ref()
                .is_some_and(|path| path.contains(&underlay_node)),
            "expected hit-testing to target the underlay subtree at this position; hit={hit:?} hit_path={hit_path:?} underlay={underlay_node:?}"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_ne!(
            ui.focus(),
            Some(underlay_node),
            "expected underlay to not be focused by a non-click-through dismissal; focus_after={:?}",
            ui.focus()
        );

        // Frame 3: closed, focus should remain on the trigger.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            trigger_id_out,
            underlay_id_out.clone(),
            vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))],
        );
        assert_eq!(ui.focus(), Some(trigger_node));
    }

    #[test]
    fn dropdown_menu_pointer_open_focuses_content_not_first_item() {
        use fret_core::MouseButton;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))];

        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = rect_center(trigger_bounds);

        // shadcn `DropdownMenu` uses a caller-owned open model; treat this pointer interaction as
        // the "open reason" and flip the open model like a trigger would.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");

        let focus = ui.focus().expect("expected focus after pointer-open");
        assert_ne!(
            focus, alpha.id,
            "pointer-open should not move focus to the first menu item (Radix onEntryFocus preventDefault)"
        );
        assert_ne!(
            focus, trigger,
            "pointer-open should focus menu content/roving container rather than keeping trigger focus"
        );
    }

    fn rect_center(rect: Rect) -> Point {
        Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 / 2.0),
            Px(rect.origin.y.0 + rect.size.height.0 / 2.0),
        )
    }

    #[test]
    fn dropdown_menu_submenu_opens_on_hover_and_closes_on_leave() {
        use std::time::Duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![
            DropdownMenuEntry::Item(DropdownMenuItem::new("More").submenu(vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Alpha")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Beta")),
            ])),
            DropdownMenuEntry::Item(DropdownMenuItem::new("Other")),
        ];

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        let more_bounds = more.bounds;
        let more_node = more.id;
        assert!(
            more_bounds.size.width.0 > 0.0,
            "expected submenu trigger bounds to be non-zero width for hit-testing; more_bounds={more_bounds:?}"
        );
        assert!(
            ui.debug_node_bounds(more_node).is_some(),
            "expected submenu trigger node to exist in the UI tree; more_node={more_node:?} layers={:?}",
            ui.debug_layers_in_paint_order()
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: rect_center(more_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );
        let hover_hit = ui.debug_hit_test(rect_center(more_bounds));
        assert!(
            hover_hit.hit.is_some(),
            "expected hover move position to hit-test inside the menu; layers={:?}",
            ui.debug_layers_in_paint_order()
        );
        let hit_node = hover_hit.hit.expect("hit node");
        let hit_path = ui.debug_node_path(hit_node);
        assert!(
            hit_path.contains(&more_node),
            "expected hover move position to be inside the submenu trigger pressable; hit={hit_node:?} more={more_node:?} more_bounds={more_bounds:?} more_node_bounds={:?} path={hit_path:?} hover_hit={hover_hit:?} layers={:?}",
            ui.debug_node_bounds(more_node),
            ui.debug_layers_in_paint_order()
        );

        let effects = app.flush_effects();
        let open_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == Duration::from_millis(100) => {
                Some(*token)
            }
            _ => None,
        });
        let Some(open_timer) = open_timer else {
            panic!("expected submenu open-delay timer effect");
        };

        // Third frame: hovering does not open the submenu immediately (open-delay timer).
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item after hover");
        assert!(
            !more.flags.expanded,
            "submenu trigger should not report expanded=true before open-delay timer fires"
        );
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu items should not render before the open-delay timer fires"
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: open_timer });

        // Fourth frame: after open timer fires, the submenu opens.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item after open timer");
        assert!(
            more.flags.expanded,
            "submenu trigger should report expanded=true after open-delay timer fires"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu items should render after the open-delay timer fires"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(390.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let effects = app.flush_effects();
        let timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, .. } => Some(*token),
            _ => None,
        });
        let Some(timer) = timer else {
            panic!("expected submenu safe-hover close timer effect");
        };

        // Fifth frame: leaving the safe corridor arms a short close delay (submenu remains visible).
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu should remain visible during the close delay"
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: timer });

        // Sixth frame: after the close timer fires, the submenu closes.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("More").submenu(vec![
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Alpha")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Beta")),
                ])),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Other")),
            ],
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu should close after the close delay timer fires"
        );
    }

    #[test]
    fn dropdown_menu_submenu_does_not_switch_while_pointer_moves_through_safe_corridor() {
        use std::time::Duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id_out = app.models_mut().insert(None);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(500.0), Px(260.0)),
        );
        let mut services = FakeServices::default();

        let many_sub_items = (0..16)
            .map(|i| DropdownMenuEntry::Item(DropdownMenuItem::new(format!("Sub {i}"))))
            .collect::<Vec<_>>();

        let entries = vec![
            DropdownMenuEntry::Item(DropdownMenuItem::new("More").submenu(many_sub_items)),
            DropdownMenuEntry::Item(DropdownMenuItem::new("Other").submenu(vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Other A")),
            ])),
        ];

        // First frame: establish stable trigger bounds.
        let (_, trigger_id) = render_frame_capture_trigger_id(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu.
        let (_, trigger_id2) = render_frame_capture_trigger_id(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            entries.clone(),
        );
        assert_eq!(trigger_id2, trigger_id, "expected trigger id to be stable");

        // Hover "More" to arm the open-delay timer.
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        let more_bounds = more.bounds;

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: rect_center(more_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let effects = app.flush_effects();
        let open_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == Duration::from_millis(100) => {
                Some(*token)
            }
            _ => None,
        });
        let Some(open_timer) = open_timer else {
            panic!("expected submenu open-delay timer effect");
        };

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: open_timer });

        // Third frame: after the open timer fires, "More" submenu is open.
        let (_, trigger_id3) = render_frame_capture_trigger_id(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            entries.clone(),
        );
        assert_eq!(trigger_id3, trigger_id, "expected trigger id to be stable");

        let overlay_root_name = OverlayController::popover_root_name(trigger_id);
        let submenu_models = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            &overlay_root_name,
            |cx| menu::sub::ensure_models(cx),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item after open");
        assert!(more.flags.expanded, "expected More to be expanded");

        let other = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Other"))
            .expect("Other menu item");

        // Choose a point near the "Other" item's right edge, so the pointer direction is towards
        // the right-side submenu panel and Radix-style pointer grace intent should apply.
        let safe_point = Point::new(
            Px(other.bounds.origin.x.0 + other.bounds.size.width.0 - 2.0),
            Px(other.bounds.origin.y.0 + other.bounds.size.height.0 * 0.75),
        );

        // Sanity: chosen point must actually hover the "Other" item.
        let hit = ui.debug_hit_test(safe_point);
        let hit_node = hit.hit.expect("expected hit at safe_point");
        let hit_path = ui.debug_node_path(hit_node);
        assert!(
            hit_path.contains(&other.id),
            "expected safe point to hit-test within Other pressable; safe_point={safe_point:?} hit={hit:?} path={hit_path:?} other_id={:?}",
            other.id
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                position: safe_point,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let effects = app.flush_effects();
        let last_pointer = app
            .models_mut()
            .read(&submenu_models.last_pointer, |v| *v)
            .ok()
            .flatten();
        let pointer_dir = app
            .models_mut()
            .read(&submenu_models.pointer_dir, |v| *v)
            .ok()
            .flatten();
        let grace_intent = app
            .models_mut()
            .read(&submenu_models.pointer_grace_intent, |v| *v)
            .ok()
            .flatten();
        let geometry = app
            .models_mut()
            .read(&submenu_models.geometry, |v| *v)
            .ok()
            .flatten();
        let open_trigger = app
            .models_mut()
            .read(&submenu_models.trigger, |v| *v)
            .ok()
            .flatten();
        let open_trigger_bounds =
            open_trigger.and_then(|id| fret_ui::elements::bounds_for_element(&mut app, window, id));
        let open_trigger_visual_bounds = open_trigger
            .and_then(|id| fret_ui::elements::visual_bounds_for_element(&mut app, window, id));
        let open_trigger_root_bounds = open_trigger
            .and_then(|id| fret_ui::elements::root_bounds_for_element(&mut app, window, id));
        assert_eq!(
            last_pointer,
            Some(safe_point),
            "expected submenu model to observe pointer move; last_pointer={last_pointer:?} safe_point={safe_point:?}"
        );
        assert_eq!(
            pointer_dir,
            Some(menu::pointer_grace_intent::GraceSide::Right),
            "expected pointer direction to be towards right-side submenu; pointer_dir={pointer_dir:?} geometry={geometry:?}"
        );
        let Some(grace_intent) = grace_intent else {
            panic!(
                "expected pointer grace intent to be set; geometry={geometry:?} open_trigger={open_trigger:?} open_trigger_bounds={open_trigger_bounds:?} open_trigger_visual_bounds={open_trigger_visual_bounds:?} open_trigger_root_bounds={open_trigger_root_bounds:?} more_bounds={more_bounds:?} safe_point={safe_point:?} last_pointer={last_pointer:?} pointer_dir={pointer_dir:?}",
            );
        };
        assert!(
            menu::pointer_grace_intent::is_pointer_in_grace_area(safe_point, grace_intent),
            "expected safe point to lie inside grace area; safe_point={safe_point:?} intent={grace_intent:?} geometry={geometry:?}"
        );
        let maybe_switch_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == Duration::from_millis(100) => {
                Some(*token)
            }
            _ => None,
        });
        if let Some(token) = maybe_switch_timer {
            // Even if a switch timer was armed due to event ordering, it must be canceled/ignored
            // while the pointer is inside the safe-hover corridor.
            ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
        }

        // Fourth frame: submenu should remain open (no switch to "Other").
        let (_, trigger_id4) = render_frame_capture_trigger_id(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out,
            entries,
        );
        assert_eq!(trigger_id4, trigger_id, "expected trigger id to be stable");

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item after hover other");
        let other = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Other"))
            .expect("Other menu item after hover other");

        let has_sub0 = snap
            .nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub 0"));
        let has_other_a = snap
            .nodes
            .iter()
            .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Other A"));

        assert!(
            more.flags.expanded,
            "expected submenu to remain open while pointer is in safe corridor (other_expanded={} has_sub0={} has_other_a={})",
            other.flags.expanded, has_sub0, has_other_a
        );
        assert!(
            !other.flags.expanded,
            "expected Other submenu to remain closed while pointer is in safe corridor"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub 0")),
            "expected More submenu items to remain visible"
        );
        assert!(
            !snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Other A")
            }),
            "expected Other submenu items to not appear"
        );
    }

    #[test]
    fn dropdown_menu_submenu_opens_on_arrow_right_without_pointer_move() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![DropdownMenuEntry::Item(
            DropdownMenuItem::new("More").submenu(vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Alpha")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Beta")),
            ]),
        )];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        ui.set_focus(Some(more.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
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
            open,
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu items should render after ArrowRight opens the submenu"
        );
    }
}
