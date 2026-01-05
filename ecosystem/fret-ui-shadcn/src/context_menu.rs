use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Edges, MouseButton, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::action::PointerDownCx;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PointerRegionProps, PointerRegionState, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::menu;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};

use crate::dropdown_menu::{DropdownMenuAlign, DropdownMenuSide};
use crate::popper_arrow::{self, DiamondArrowStyle};

#[derive(Debug, Clone)]
pub enum ContextMenuEntry {
    Item(ContextMenuItem),
    CheckboxItem(ContextMenuCheckboxItem),
    RadioGroup(ContextMenuRadioGroup),
    RadioItem(ContextMenuRadioItem),
    Label(ContextMenuLabel),
    Group(ContextMenuGroup),
    Separator,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub inset: bool,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            inset: false,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            trailing: None,
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

/// shadcn/ui `ContextMenuLabel` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuLabel {
    pub text: Arc<str>,
    pub inset: bool,
}

impl ContextMenuLabel {
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

/// shadcn/ui `ContextMenuGroup` (v4).
///
/// In the upstream DOM implementation, this is a structural wrapper. In Fret, we currently treat
/// it as a transparent grouping node and simply flatten its entries for rendering/navigation.
#[derive(Debug, Clone)]
pub struct ContextMenuGroup {
    pub entries: Vec<ContextMenuEntry>,
}

impl ContextMenuGroup {
    pub fn new(entries: Vec<ContextMenuEntry>) -> Self {
        Self { entries }
    }
}

/// shadcn/ui `ContextMenuShortcut` (v4).
///
/// This is typically rendered as trailing, muted text inside a menu item.
#[derive(Debug, Clone)]
pub struct ContextMenuShortcut {
    pub text: Arc<str>,
}

impl ContextMenuShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: fret_core::FontId::default(),
                size: theme.metrics.font_size,
                weight: fret_core::FontWeight::NORMAL,
                line_height: Some(theme.metrics.font_line_height),
                letter_spacing_em: Some(0.12),
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

/// shadcn/ui `ContextMenuCheckboxItem` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuCheckboxItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub checked: Model<bool>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl ContextMenuCheckboxItem {
    pub fn new(checked: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            checked,
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

/// shadcn/ui `ContextMenuRadioGroup` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuRadioGroup {
    pub value: Model<Option<Arc<str>>>,
    pub items: Vec<ContextMenuRadioItemSpec>,
}

impl ContextMenuRadioGroup {
    pub fn new(value: Model<Option<Arc<str>>>) -> Self {
        Self {
            value,
            items: Vec::new(),
        }
    }

    pub fn item(mut self, item: ContextMenuRadioItemSpec) -> Self {
        self.items.push(item);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ContextMenuRadioItemSpec {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl ContextMenuRadioItemSpec {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        let value = value.into();
        let label = label.into();
        Self {
            label,
            value,
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }

    fn into_item(self, group_value: Model<Option<Arc<str>>>) -> ContextMenuRadioItem {
        ContextMenuRadioItem {
            label: self.label,
            value: self.value,
            group_value,
            disabled: self.disabled,
            close_on_select: self.close_on_select,
            command: self.command,
            a11y_label: self.a11y_label,
            trailing: self.trailing,
        }
    }
}

/// shadcn/ui `ContextMenuRadioItem` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuRadioItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub group_value: Model<Option<Arc<str>>>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl ContextMenuRadioItem {
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

fn flatten_entries(into: &mut Vec<ContextMenuEntry>, entries: Vec<ContextMenuEntry>) {
    for entry in entries {
        match entry {
            ContextMenuEntry::Group(group) => flatten_entries(into, group.entries),
            ContextMenuEntry::RadioGroup(group) => {
                for item in group.items {
                    into.push(ContextMenuEntry::RadioItem(
                        item.into_item(group.value.clone()),
                    ));
                }
            }
            other => into.push(other),
        }
    }
}

fn menu_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    trailing: Option<AnyElement>,
    indicator_on: Option<bool>,
    disabled: bool,
    row_bg: fret_core::Color,
    row_fg: fret_core::Color,
    text_style: TextStyle,
    font_size: Px,
    font_line_height: Px,
    pad_left: Px,
    pad_x: Px,
    pad_y: Px,
    radius_sm: Px,
    text_disabled: fret_core::Color,
) -> Vec<AnyElement> {
    vec![cx.container(
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
            let has_indicator = indicator_on.is_some();
            let mut row: Vec<AnyElement> = Vec::with_capacity(
                usize::from(has_indicator) + 1 + usize::from(trailing.is_some()),
            );

            if let Some(is_on) = indicator_on {
                let indicator_fg = if disabled { text_disabled } else { row_fg };
                row.push(cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(16.0));
                            layout.size.height = Length::Px(Px(16.0));
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
                        if !is_on {
                            return Vec::new();
                        }

                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: Arc::from("\u{2713}"),
                            style: Some(TextStyle {
                                font: fret_core::FontId::default(),
                                size: font_size,
                                weight: fret_core::FontWeight::MEDIUM,
                                line_height: Some(font_line_height),
                                letter_spacing_em: None,
                            }),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            color: Some(indicator_fg),
                        })]
                    },
                ));
            }

            row.push(cx.text_props(TextProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                },
                text: label.clone(),
                style: Some(text_style.clone()),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
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
    )]
}

/// shadcn/ui `ContextMenu` root (v4).
///
/// This is a dismissible popover (non-modal) opened by a component-owned pointer policy:
/// - right click
/// - (macOS) ctrl + left click
///
/// Notes:
/// - Position is anchored at the last pointer-down location observed within the trigger region.
/// - Keyboard invocation via Shift+F10 is supported (there is no dedicated `ContextMenu` key in
///   `fret_core::KeyCode` yet).
#[derive(Clone)]
pub struct ContextMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
}

impl std::fmt::Debug for ContextMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenu")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl ContextMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: DropdownMenuAlign::Start,
            side: DropdownMenuSide::Bottom,
            side_offset: Px(4.0),
            window_margin: Px(8.0),
            typeahead_timeout_ticks: 30,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
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

    /// Enables a ContextMenu arrow (Radix `ContextMenuArrow`-style).
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
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);
            let arrow = self.arrow;
            let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_size")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                    .unwrap_or(Px(12.0))
            });
            let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_padding")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                    .unwrap_or(theme.metrics.radius_md)
            });

            let id = cx.root_id();
            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            menu::trigger::wire_open_on_shift_f10(cx, trigger_id, self.open.clone());

            let open = self.open;
            let open_for_pointer = open.clone();
            let pointer_policy = Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                           _cx: fret_ui::action::ActionCx,
                                           down: PointerDownCx| {
                let is_right_click = down.button == MouseButton::Right;
                let is_macos_ctrl_click =
                    cfg!(target_os = "macos") && down.button == MouseButton::Left && down.modifiers.ctrl;

                if !is_right_click && !is_macos_ctrl_click {
                    return false;
                }

                let _ = host.models_mut().update(&open_for_pointer, |v| *v = true);
                true
            });

            let trigger = cx.pointer_region(PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(pointer_policy);
                vec![trigger]
            });

            let pointer_down = cx.with_state(PointerRegionState::default, |st| st.last_down);
            let anchor_point = pointer_down.map(|it| it.position);

            if is_open {
                let overlay_root_name = OverlayController::popover_root_name(id);

                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let open_for_overlay = open.clone();
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();

                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let trigger_bounds =
                        overlay::anchor_bounds_for_element(cx, trigger_id);
                    let anchor = anchor_point.or_else(|| trigger_bounds.map(|r| r.origin));
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };

                    let mut flat: Vec<ContextMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, entries(cx));
                    let entries = flat;

                    let item_count = entries
                        .iter()
                        .filter(|e| {
                            matches!(
                                e,
                                ContextMenuEntry::Item(_)
                                    | ContextMenuEntry::CheckboxItem(_)
                                    | ContextMenuEntry::RadioItem(_)
                            )
                        })
                        .count();
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                        .iter()
                        .map(|e| match e {
                            ContextMenuEntry::Item(item) => (item.label.clone(), item.disabled),
                            ContextMenuEntry::CheckboxItem(item) => (item.label.clone(), item.disabled),
                            ContextMenuEntry::RadioItem(item) => (item.label.clone(), item.disabled),
                            ContextMenuEntry::Label(_) | ContextMenuEntry::Separator => {
                                (Arc::from(""), true)
                            }
                            ContextMenuEntry::Group(_) | ContextMenuEntry::RadioGroup(_) => {
                                unreachable!("entries are flattened")
                            }
                        })
                        .unzip();

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let estimated = Size::new(Px(220.0), Px(200.0));

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

                    let anchor_rect = overlay::anchor_rect_from_point(anchor);
                    let layout = popper::popper_content_layout_sized(
                        outer,
                        anchor_rect,
                        estimated,
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

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N2).resolve(&theme);

                    let arrow_el = popper_arrow::diamond_arrow_element(
                        cx,
                        &layout,
                        wrapper_insets,
                        arrow_size,
                        DiamondArrowStyle {
                            bg: theme.colors.panel_background,
                            border: Some(border),
                            border_width: Px(1.0),
                        },
                    );

                    let content = cx.semantics(
                        SemanticsProps {
                            layout: LayoutStyle::default(),
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
                                            background: Some(theme.colors.panel_background),
                                            shadow: Some(shadow),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border),
                                            corner_radii: fret_core::Corners::all(
                                                theme.metrics.radius_sm,
                                            ),
                                        },
                                        move |cx| {
                                            let content_focus_id_for_panel =
                                                content_focus_id_for_children.clone();
                                            let roving = menu::content::menu_roving_group_apg_prefix_typeahead(
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

                                            let text_style = TextStyle {
                                                font: fret_core::FontId::default(),
                                                size: theme.metrics.font_size,
                                                weight: fret_core::FontWeight::NORMAL,
                                                line_height: Some(theme.metrics.font_line_height),
                                                letter_spacing_em: None,
                                            };
                                            let radius_sm = theme.metrics.radius_sm;
                                            let text_disabled = theme.colors.text_disabled;
                                            let label_fg = theme
                                                .color_by_key("muted.foreground")
                                                .or_else(|| theme.color_by_key("muted-foreground"))
                                                .unwrap_or(theme.colors.text_muted);
                                            let accent = theme
                                                .color_by_key("accent")
                                                .unwrap_or(theme.colors.hover_background);
                                            let accent_fg = theme
                                                .color_by_key("accent.foreground")
                                                .or_else(|| theme.color_by_key("accent-foreground"))
                                                .unwrap_or(theme.colors.text_primary);
                                            let fg = theme.colors.text_primary;

                                            let mut out: Vec<AnyElement> =
                                                Vec::with_capacity(entries.len());

                                            let mut item_ix: usize = 0;
                                            for entry in entries.clone() {
                                                match entry {
                                                    ContextMenuEntry::Label(label) => {
                                                        let pad_left =
                                                            if label.inset { pad_x_inset } else { pad_x };
                                                        let text = label.text.clone();
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
                                                                        font: fret_core::FontId::default(),
                                                                        size: theme.metrics.font_size,
                                                                        weight: fret_core::FontWeight::MEDIUM,
                                                                        line_height: Some(
                                                                            theme.metrics.font_line_height,
                                                                        ),
                                                                        letter_spacing_em: None,
                                                                    }),
                                                                    wrap: TextWrap::None,
                                                                    overflow: TextOverflow::Ellipsis,
                                                                    color: Some(label_fg),
                                                                })]
                                                            },
                                                        ));
                                                    }
                                                    ContextMenuEntry::Group(_) => {
                                                        unreachable!("groups are flattened")
                                                    }
                                                    ContextMenuEntry::RadioGroup(_) => {
                                                        unreachable!("radio groups are flattened")
                                                    }
                                                    ContextMenuEntry::Separator => {
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width = Length::Fill;
                                                                    layout.size.height =
                                                                        Length::Px(Px(1.0));
                                                                    layout
                                                                },
                                                                padding: Edges::all(Px(0.0)),
                                                                background: Some(border),
                                                                ..Default::default()
                                                            },
                                                            |_cx| Vec::new(),
                                                        ));
                                                    }
                                                    ContextMenuEntry::Item(item) => {
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
                                                        let trailing = item.trailing.clone();
                                                        let pad_left =
                                                            if item.inset { pad_x_inset } else { pad_x };
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable(
                                                                PressableProps {
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
                                                                    a11y: menu::item::menu_item_a11y(
                                                                        a11y_label,
                                                                        None,
                                                                    )
                                                                    .with_collection_position(
                                                                        collection_index,
                                                                        item_count,
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                                move |cx, st| {
                                                                    cx.pressable_dispatch_command_opt(command);
                                                                    if !disabled && close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }

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

                                                                    menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        trailing.clone(),
                                                                        None,
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        theme.metrics.font_size,
                                                                        theme.metrics.font_line_height,
                                                                        pad_left,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    )
                                                                },
                                                            )
                                                         }));
                                                     }
                                                    ContextMenuEntry::CheckboxItem(item) => {
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
                                                        let trailing = item.trailing.clone();
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            let checked_now = cx
                                                                .watch_model(&checked)
                                                                .copied()
                                                                .unwrap_or(false);
                                                            cx.pressable(
                                                                PressableProps {
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
                                                                        a11y_label.clone(),
                                                                        checked_now,
                                                                    )
                                                                    .with_collection_position(
                                                                        collection_index,
                                                                        item_count,
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                                move |cx, st| {
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

                                                                    menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        trailing.clone(),
                                                                        Some(checked_now),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        theme.metrics.font_size,
                                                                        theme.metrics.font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    )
                                                                },
                                                            )
                                                        }));
                                                    }
                                                    ContextMenuEntry::RadioItem(item) => {
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
                                                        let trailing = item.trailing.clone();
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            let selected = cx
                                                                .watch_model(&group_value)
                                                                .cloned()
                                                                .flatten();
                                                            let is_selected = menu::radio_group::is_selected(
                                                                selected.as_ref(),
                                                                &value,
                                                            );
                                                            cx.pressable(
                                                                PressableProps {
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
                                                                        a11y_label.clone(),
                                                                        is_selected,
                                                                    )
                                                                    .with_collection_position(
                                                                        collection_index,
                                                                        item_count,
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                                move |cx, st| {
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

                                                                    menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        trailing.clone(),
                                                                        Some(is_selected),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        theme.metrics.font_size,
                                                                        theme.metrics.font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    )
                                                                },
                                                            )
                                                        }));
                                                    }
                                                  }
                                              }

                                                    out
                                                },
                                            );
                                            if content_focus_id_for_panel.get().is_none() {
                                                content_focus_id_for_panel.set(Some(roving.id));
                                            }
                                            vec![roving]
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

                    vec![content]
                });

                let mut request = OverlayRequest::dismissible_popover(
                    id,
                    trigger_id,
                    open,
                    OverlayPresence::instant(true),
                    overlay_children,
                );
                request.consume_outside_pointer_events = true;
                request.root_name = Some(overlay_root_name);
                if !fret_ui::input_modality::is_keyboard(cx.app, Some(cx.window)) {
                    request.initial_focus = content_focus_id.get();
                }
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
        AppWindowId, Event, KeyCode, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    };
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_core::{TextStyle, UiServices};
    use fret_runtime::FrameId;
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
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
            "context-menu",
            |cx| {
                vec![ContextMenu::new(open).into_element(
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
                    |_cx| {
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Alpha")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Beta")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Gamma")),
                        ]
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

    fn render_frame_focusable_trigger(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
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
            "context-menu-shift-f10",
            |cx| {
                vec![ContextMenu::new(open).into_element(
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
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
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
    fn context_menu_opens_on_shift_f10() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        // Second frame: ContextMenu emits its OverlayRequest while rendering.
        // Re-rendering the root is required for the menu items to appear.
        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha")
            }),
            "menu items should render after Shift+F10 opens the context menu"
        );
    }

    #[test]
    fn context_menu_pointer_open_focuses_content_not_first_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
            }),
        );

        // Second frame: ContextMenu emits its OverlayRequest while rendering.
        let _ =
            render_frame_focusable_trigger(&mut ui, &mut app, &mut services, window, bounds, open);

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

    #[test]
    fn context_menu_items_have_collection_position_metadata_excluding_separators() {
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
}
