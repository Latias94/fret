use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Edges, FontId, FontWeight, Point, Px, Rect, Size, TextStyle};
use fret_icons::ids;
use fret_runtime::{CommandId, Model};
use fret_ui::action::OnDismissRequest;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, PressableProps, RingStyle, RovingFlexProps, RovingFocusProps,
    ScrollAxis, ScrollProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::dropdown_menu as menu;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::{
    ui, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence, Radius, Space,
};

use crate::overlay_motion;
use crate::popper_arrow::{self, DiamondArrowStyle};
use crate::shortcut_display::command_shortcut_label;

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn is_dark_background(theme: &Theme) -> bool {
    let bg = theme.color_required("background");
    let luma = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
    luma < 0.5
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
    pub content: Option<AnyElement>,
    pub padding: Option<Edges>,
    pub estimated_height: Option<Px>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
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
            content: None,
            padding: None,
            estimated_height: None,
            disabled: false,
            close_on_select: true,
            command: None,
            a11y_label: None,
            test_id: None,
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

    pub fn content(mut self, element: AnyElement) -> Self {
        self.content = Some(element);
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn estimated_height(mut self, height: Px) -> Self {
        self.estimated_height = Some(height);
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

    pub fn submenu(mut self, entries: impl IntoIterator<Item = DropdownMenuEntry>) -> Self {
        self.submenu = Some(entries.into_iter().collect());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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
/// In the upstream DOM implementation, this is a structural wrapper (`MenuPrimitive.Group`).
/// In Fret, we preserve it as a structural semantics wrapper (`role=Group`) without changing
/// layout, so menu roving/typeahead still matches Radix while keeping group boundaries.
#[derive(Debug, Clone)]
pub struct DropdownMenuGroup {
    pub entries: Vec<DropdownMenuEntry>,
}

impl DropdownMenuGroup {
    pub fn new(entries: impl IntoIterator<Item = DropdownMenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
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
        let fg = theme.color_required("muted-foreground");

        let base_size = theme.metric_required("font.size");
        let base_line_height = theme.metric_required("font.line_height");
        let font_size = theme
            .metric_by_key("component.dropdown_menu.shortcut.font_size")
            .unwrap_or_else(|| Px((base_size.0 - 1.0).max(10.0)));
        let font_line_height = theme
            .metric_by_key("component.dropdown_menu.shortcut.line_height")
            .unwrap_or_else(|| Px((base_line_height.0 - 2.0).max(font_size.0)));

        ui::text(cx, self.text)
            .layout(LayoutRefinement::default().ml_auto())
            .text_size_px(font_size)
            .line_height_px(font_line_height)
            .font_normal()
            .letter_spacing_em(0.10)
            .nowrap()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

fn focusable_item_count(entries: &[DropdownMenuEntry]) -> usize {
    let mut out = 0usize;
    for entry in entries {
        match entry {
            DropdownMenuEntry::Item(_)
            | DropdownMenuEntry::CheckboxItem(_)
            | DropdownMenuEntry::RadioItem(_) => out += 1,
            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {}
            DropdownMenuEntry::Group(group) => out += focusable_item_count(&group.entries),
            DropdownMenuEntry::RadioGroup(group) => out += group.items.len(),
        }
    }
    out
}

fn reserve_leading_slot(entries: &[DropdownMenuEntry]) -> bool {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Item(item) => {
                if item.leading.is_some() {
                    return true;
                }
            }
            DropdownMenuEntry::CheckboxItem(item) => {
                if item.leading.is_some() {
                    return true;
                }
            }
            DropdownMenuEntry::RadioItem(item) => {
                if item.leading.is_some() {
                    return true;
                }
            }
            DropdownMenuEntry::RadioGroup(group) => {
                if group.items.iter().any(|i| i.leading.is_some()) {
                    return true;
                }
            }
            DropdownMenuEntry::Group(group) => {
                if reserve_leading_slot(&group.entries) {
                    return true;
                }
            }
            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {}
        }
    }
    false
}

fn collect_roving_labels_and_disabled(
    entries: &[DropdownMenuEntry],
    labels: &mut Vec<Arc<str>>,
    disabled: &mut Vec<bool>,
) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Item(item) => {
                labels.push(item.label.clone());
                disabled.push(item.disabled);
            }
            DropdownMenuEntry::CheckboxItem(item) => {
                labels.push(item.label.clone());
                disabled.push(item.disabled);
            }
            DropdownMenuEntry::RadioItem(item) => {
                labels.push(item.label.clone());
                disabled.push(item.disabled);
            }
            DropdownMenuEntry::RadioGroup(group) => {
                for item in &group.items {
                    labels.push(item.label.clone());
                    disabled.push(item.disabled);
                }
            }
            DropdownMenuEntry::Group(group) => {
                collect_roving_labels_and_disabled(&group.entries, labels, disabled);
            }
            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {}
        }
    }
}

fn find_submenu_entries_by_value(
    entries: &[DropdownMenuEntry],
    open_value: &str,
) -> Option<Vec<DropdownMenuEntry>> {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Item(item) => {
                if item.value.as_ref() == open_value {
                    return item.submenu.clone();
                }
            }
            DropdownMenuEntry::Group(group) => {
                if let Some(found) = find_submenu_entries_by_value(&group.entries, open_value) {
                    return Some(found);
                }
            }
            DropdownMenuEntry::CheckboxItem(_)
            | DropdownMenuEntry::RadioGroup(_)
            | DropdownMenuEntry::RadioItem(_)
            | DropdownMenuEntry::Label(_)
            | DropdownMenuEntry::Separator => {}
        }
    }
    None
}

fn menu_structural_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    role: fret_core::SemanticsRole,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.semantic_flex(
        fret_ui::element::SemanticFlexProps {
            role,
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
        },
        move |_cx| children.clone(),
    )
}

fn estimated_menu_panel_height_for_entries(
    entries: &[DropdownMenuEntry],
    row_height: Px,
    max_height: Px,
) -> Px {
    // new-york-v4: menu panels use `p-1` and `border`.
    let panel_padding_y = Px(8.0);
    let panel_border_y = Px(2.0);

    fn add_entries(height: &mut f32, entries: &[DropdownMenuEntry], row_height: f32) {
        for entry in entries {
            match entry {
                DropdownMenuEntry::Separator => {
                    // new-york-v4: `Separator` uses `-mx-1 my-1` (1px line + 4px + 4px).
                    *height += 9.0;
                }
                DropdownMenuEntry::Label(_)
                | DropdownMenuEntry::Item(_)
                | DropdownMenuEntry::CheckboxItem(_)
                | DropdownMenuEntry::RadioItem(_) => {
                    let entry_height = match entry {
                        DropdownMenuEntry::Item(item) => {
                            item.estimated_height.map(|h| h.0).unwrap_or(row_height)
                        }
                        _ => row_height,
                    };
                    *height += entry_height.max(0.0);
                }
                DropdownMenuEntry::Group(group) => add_entries(height, &group.entries, row_height),
                DropdownMenuEntry::RadioGroup(group) => {
                    *height += row_height.max(0.0) * group.items.len() as f32;
                }
            }
        }
    }

    let mut height = panel_padding_y.0 + panel_border_y.0;
    add_entries(&mut height, entries, row_height.0);

    let height = height.max(0.0);
    Px(height.min(max_height.0.max(0.0)))
}

fn estimated_menu_text_width(text: &str, font_size: Px) -> Px {
    // We can't measure text here (ElementContext doesn't expose services), so approximate.
    //
    // This is intentionally tuned for shadcn/new-york-v4 defaults (Geist at `text-sm`), and is
    // only used for popper desired sizing (so the menu panel isn't artificially clamped to the
    // `min-w-[8rem]` floor when labels are longer).
    let mut ems: f32 = 0.0;
    for ch in text.chars() {
        ems += if ch == ' ' { 0.33_f32 } else { 0.464_f32 };
    }
    Px(font_size.0 * ems.max(0.0))
}

fn estimated_menu_panel_width_for_entries(
    entries: &[DropdownMenuEntry],
    font_size: Px,
    pad_x: Px,
    pad_x_inset: Px,
    reserve_leading_slot: bool,
) -> Px {
    // new-york-v4: panels use `p-1` and `border`.
    let panel_padding_x = Px(8.0);
    let panel_border_x = Px(2.0);

    let leading_slot_w = if reserve_leading_slot {
        // new-york-v4 items reserve a 16px icon slot with an 8px gap (`size-4` + `gap-2`).
        Px(24.0)
    } else {
        Px(0.0)
    };

    fn add_entries(
        out_max: &mut f32,
        entries: &[DropdownMenuEntry],
        font_size: Px,
        pad_x: Px,
        pad_x_inset: Px,
        leading_slot_w: Px,
    ) {
        for entry in entries {
            let row_w = match entry {
                DropdownMenuEntry::Separator => None,
                DropdownMenuEntry::Label(label) => {
                    Some(estimated_menu_text_width(&label.text, font_size).0 + pad_x.0 * 2.0)
                }
                DropdownMenuEntry::Item(item) => {
                    let left_pad = if item.inset { pad_x_inset } else { pad_x };
                    Some(
                        estimated_menu_text_width(&item.label, font_size).0
                            + left_pad.0
                            + pad_x.0
                            + leading_slot_w.0,
                    )
                }
                DropdownMenuEntry::CheckboxItem(item) => Some(
                    estimated_menu_text_width(&item.label, font_size).0
                        + pad_x.0
                        + pad_x.0
                        + Px(24.0).0,
                ),
                DropdownMenuEntry::RadioItem(item) => Some(
                    estimated_menu_text_width(&item.label, font_size).0
                        + pad_x.0
                        + pad_x.0
                        + Px(24.0).0,
                ),
                DropdownMenuEntry::Group(group) => {
                    add_entries(
                        out_max,
                        &group.entries,
                        font_size,
                        pad_x,
                        pad_x_inset,
                        leading_slot_w,
                    );
                    None
                }
                DropdownMenuEntry::RadioGroup(group) => {
                    for item in &group.items {
                        let w = estimated_menu_text_width(&item.label, font_size).0
                            + pad_x.0
                            + pad_x.0
                            + Px(24.0).0;
                        *out_max = out_max.max(w);
                    }
                    None
                }
            };

            if let Some(w) = row_w {
                *out_max = out_max.max(w.max(0.0));
            }
        }
    }

    let mut max_row_w = 0.0;
    add_entries(
        &mut max_row_w,
        entries,
        font_size,
        pad_x,
        pad_x_inset,
        leading_slot_w,
    );

    Px((max_row_w + panel_padding_x.0 + panel_border_x.0).max(0.0))
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
                                        ..Default::default()
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

            let style = text_style.clone();
            let mut text = ui::text(cx, label.clone())
                .layout(LayoutRefinement::default().min_w_0().flex_1())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .nowrap()
                .text_color(ColorRef::Color(if disabled {
                    text_disabled
                } else {
                    row_fg
                }));

            if let Some(line_height) = style.line_height {
                text = text.line_height_px(line_height);
            }

            if let Some(letter_spacing_em) = style.letter_spacing_em {
                text = text.letter_spacing_em(letter_spacing_em);
            }

            row.push(text.into_element(cx));

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
/// This is a dismissible popover overlay backed by the component-layer overlay manager
/// (`fret-ui-kit/overlay_controller.rs`).
#[derive(Clone)]
pub struct DropdownMenu {
    open: Model<bool>,
    modal: bool,
    align: DropdownMenuAlign,
    align_offset: Px,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    min_width: Px,
    submenu_min_width: Px,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    align_leading_icons: bool,
    on_dismiss_request: Option<OnDismissRequest>,
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
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .finish()
    }
}

impl DropdownMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            modal: true,
            align: DropdownMenuAlign::default(),
            align_offset: Px(0.0),
            side: DropdownMenuSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(0.0),
            typeahead_timeout_ticks: 30,
            min_width: Px(128.0),
            submenu_min_width: Px(128.0),
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            align_leading_icons: true,
            on_dismiss_request: None,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    /// Controls whether outside-press dismissal should be click-through (Radix `modal={false}`).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
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

    pub fn submenu_min_width(mut self, min_width: Px) -> Self {
        self.submenu_min_width = min_width;
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

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn into_element<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = DropdownMenuEntry>,
    {
        cx.scope(|cx| {
            let overlay_id = cx.root_id();
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx
                .watch_model(&self.open)
                .layout()
                .copied()
                .unwrap_or(false);
            let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                0.95,
                1.0,
                overlay_motion::shadcn_ease,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };
            let opacity = motion.opacity;
            let scale = motion.scale;
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
                    .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme))
            });

            // Keep the trigger element id stable across frames even if upstream rendering changes
            // the number of elements/state slots created before/after this callsite.
            //
            // Overlay placement relies on looking up the trigger's anchor bounds from the
            // previous layout pass. If the trigger id churns between frames, the overlay will
            // transiently lose its anchor and blink out (breaking Radix-style menu grace
            // behaviors like submenu close delays).
            let trigger = cx.keyed(("dropdown-menu-trigger", overlay_id), |cx| trigger(cx));
            let trigger_id = trigger.id;
            let first_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            let last_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            menu::trigger::wire_open_or_focus_on_arrow_keys(
                cx,
                trigger_id,
                self.open.clone(),
                first_item_focus_id.clone(),
                last_item_focus_id.clone(),
            );
            let overlay_root_name = menu::dropdown_menu_root_name(overlay_id);
            let overlay_root_name_for_controls: Arc<str> = Arc::from(overlay_root_name.clone());
            let content_id_for_trigger =
                menu::content_panel::menu_content_semantics_id(cx, &overlay_root_name);
            let trigger =
                menu::trigger::apply_menu_trigger_a11y(trigger, is_open, Some(content_id_for_trigger));
            let on_dismiss_request = self.on_dismiss_request.clone();
            let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
            let submenu =
                cx.with_root_name(&overlay_root_name, |cx| {
                    menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
                });

            if overlay_presence.present {
                let align = self.align;
                let align_offset = self.align_offset;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let open = self.open;
                let modal = self.modal;
                let open_for_overlay = open.clone();
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let min_width = self.min_width;
                let submenu_min_width = self.submenu_min_width;
                let align_leading_icons = self.align_leading_icons;
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();
                let first_item_focus_id_for_request = first_item_focus_id.clone();
                let direction = direction_prim::use_direction_in_scope(cx, None);

                let (overlay_children, dismissible_on_pointer_move) =
                     cx.with_root_name(&overlay_root_name, move |cx| {
                     let theme = &theme;

                     #[derive(Default)]
                     struct TriggerAnchorCache {
                         last: Option<Rect>,
                     }

                     let anchor_now = overlay::anchor_bounds_for_element(cx, trigger_id);
                     let anchor = cx.with_state(TriggerAnchorCache::default, |st| {
                         if let Some(anchor) = anchor_now {
                             st.last = Some(anchor);
                         }
                         st.last
                     });

                    #[cfg(debug_assertions)]
                    if std::env::var_os("FRET_DEBUG_DROPDOWN_MENU_ANCHOR").is_some() {
                        eprintln!(
                            "dropdown_menu anchor: frame={} overlay_id={:?} trigger_id={:?} anchor={:?}",
                            cx.app.frame_id().0,
                            overlay_id,
                            trigger_id,
                            anchor
                        );
                    }

                     let Some(anchor) = anchor else {
                         return (Vec::new(), None);
                     };

                    let entries: Vec<DropdownMenuEntry> = entries(cx).into_iter().collect();
                    let entries: Arc<[DropdownMenuEntry]> = Arc::from(entries.into_boxed_slice());
                    let reserve_leading_slot_enabled =
                        align_leading_icons && reserve_leading_slot(&entries);

                    let item_count = focusable_item_count(&entries);
                    let mut labels: Vec<Arc<str>> = Vec::with_capacity(item_count);
                    let mut disabled_flags: Vec<bool> = Vec::with_capacity(item_count);
                    collect_roving_labels_and_disabled(&entries, &mut labels, &mut disabled_flags);

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                     let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

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

                    let popper_placement =
                        popper::PopperContentPlacement::new(direction, side, align, side_offset)
                            .with_align_offset(align_offset)
                            // Match Radix/Floating behavior: align+collision should clamp along
                            // the alignment axis when the aligned placement would overflow.
                            .with_shift_cross_axis(true)
                            .with_arrow(arrow_options, arrow_protrusion);

                    // shadcn: content width tracks trigger width (with a minimum), and height
                    // clamps to available space (scrolls internally). Radix exposes the available
                    // metrics via `--radix-dropdown-menu-content-available-*`; compute the same
                    // values from the popper substrate.
                    let popper_vars =
                        menu::dropdown_menu_popper_vars(outer, anchor, min_width, popper_placement);
                    let font_size = theme.metric_required("font.size");
                    let pad_x = MetricRef::space(Space::N2).resolve(theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(theme);
                    let estimated_w = estimated_menu_panel_width_for_entries(
                        &entries,
                        font_size,
                        pad_x,
                        pad_x_inset,
                        reserve_leading_slot_enabled,
                    );
                    let desired_w = Px(
                        estimated_w
                            .0
                            .max(min_width.0)
                            .min(popper_vars.available_width.0),
                    );
                    let max_h = theme
                        .metric_by_key("component.dropdown_menu.max_height")
                        .map(|h| Px(h.0.min(popper_vars.available_height.0)))
                        .unwrap_or(popper_vars.available_height);
                    let font_line_height = theme.metric_required("font.line_height");
                    let pad_y = MetricRef::space(Space::N1p5).resolve(theme);
                    let row_height = Px(font_line_height.0 + pad_y.0 * 2.0);
                    let desired_h = estimated_menu_panel_height_for_entries(&entries, row_height, max_h);
                    let desired = Size::new(desired_w, desired_h);

                    let layout =
                        popper::popper_content_layout_sized(outer, anchor, desired, popper_placement);

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let extra_left = wrapper_insets.left;
                    let extra_top = wrapper_insets.top;
                    let origin = popper::popper_content_transform_origin(
                        &layout,
                        anchor,
                        arrow.then_some(arrow_size),
                    );
                    let transform = overlay_motion::shadcn_popper_presence_transform(
                        layout.side,
                        origin,
                        opacity,
                        scale,
                        opening,
                    );

                    let border = theme.color_required("border");
                    let radius_sm = MetricRef::radius(Radius::Sm).resolve(&theme);
                    let ring = decl_style::focus_ring(&theme, radius_sm);
                    // new-york-v4: item rows use `px-2`.
                    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                    let bg = theme.color_required("popover.background");
                    let fg = theme.color_required("popover.foreground");
                    let accent = theme.color_required("accent");
                    let accent_fg = theme.color_required("accent-foreground");

                    let panel_chrome = crate::ui_builder_ext::surfaces::menu_style_chrome();
                    let submenu_chrome =
                        crate::ui_builder_ext::surfaces::menu_sub_style_chrome().rounded(Radius::Sm);

                    let entries_for_submenu = entries.clone();
                    let open_for_menu = open_for_overlay.clone();
                    let open_for_submenu = open_for_overlay.clone();

                    let submenu_for_content = submenu.clone();
                    let submenu_for_panel = submenu.clone();

                    let first_item_focus_id_for_items = first_item_focus_id.clone();
                    let last_item_focus_id_for_items = last_item_focus_id.clone();
                    let overlay_root_name_for_controls_for_content =
                        overlay_root_name_for_controls.clone();
                    let overlay_root_name_for_controls_for_submenu =
                        overlay_root_name_for_controls.clone();

                    // Match Radix: `role=menu` is on the content panel element (not a fullscreen
                    // wrapper). We keep the popper wrapper for arrow hit-test expansion, but
                    // position it locally inside the menu semantics node.
                    let content_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            left: Some(placed.origin.x),
                            top: Some(placed.origin.y),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(placed.size.width),
                            height: Length::Px(placed.size.height),
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };

                    let placed_local = Rect::new(Point::new(Px(0.0), Px(0.0)), placed.size);

                    let (content_id, content) = menu::content_panel::menu_content_semantics_with_id(
                        cx,
                        content_layout,
                        move |cx| {
                            vec![popper_content::popper_wrapper_at(
                                cx,
                                placed_local,
                                wrapper_insets,
                                move |cx| {
                                    let arrow_el = arrow
                                        .then(|| {
                                            popper_arrow::diamond_arrow_element(
                                                cx,
                                                &layout,
                                                wrapper_insets,
                                                arrow_size,
                                                DiamondArrowStyle {
                                                    bg,
                                                    border: Some(border),
                                                    border_width: Px(1.0),
                                                },
                                            )
                                        })
                                        .flatten();

                                    let theme_for_panel = theme.clone();
                                    let panel_chrome_for_panel = panel_chrome.clone();
                                    let panel = menu::content_panel::menu_panel_container_at(
                                        cx,
                                        Rect::new(Point::new(extra_left, extra_top), placed.size),
                                        move |layout| {
                                            let mut props = decl_style::container_props(
                                                &theme_for_panel,
                                                panel_chrome_for_panel.clone(),
                                                LayoutRefinement::default(),
                                            );
                                            props.layout = layout;
                                            props
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

                                    vec![cx.keyed("menu-scroll", |cx| {
                                        cx.scroll(
                                        ScrollProps {
                                            layout: scroll_layout,
                                            axis: ScrollAxis::Y,
                                            ..Default::default()
                                        },
                                        move |cx| {
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
                                                        wrap: false,
                                                        disabled: disabled_arc.clone(),
                                                        ..Default::default()
                                                    },
                                                },
                                                labels_arc.clone(),
                                                typeahead_timeout_ticks,
                                                move |cx| {
                                                    let font_size = theme.metric_required("font.size");
                                                    let font_line_height =
                                                        theme.metric_required("font.line_height");
                                                    let text_disabled =
                                                        alpha_mul(theme.color_required("foreground"), 0.5);
                                                    let icon_muted_fg =
                                                        theme.color_required("muted-foreground");
                                                    let destructive_fg = theme.color_required("destructive");
                                                    let destructive_bg_alpha =
                                                        if is_dark_background(&theme) { 0.20 } else { 0.10 };
                                                    let destructive_bg = theme
                                                        .color_by_key(if destructive_bg_alpha >= 0.2 {
                                                            "destructive/20"
                                                        } else {
                                                            "destructive/10"
                                                        })
                                                        .unwrap_or_else(|| {
                                                            alpha_mul(destructive_fg, destructive_bg_alpha)
                                                        });

                                                    let text_style = TextStyle {
                                                        font: fret_core::FontId::default(),
                                                        size: font_size,
                                                        weight: fret_core::FontWeight::NORMAL,
                                                        slant: Default::default(),
                                                        line_height: Some(font_line_height),
                                                        letter_spacing_em: None,
                                                    };

                                                    let mut item_ix: usize = 0;

                                                    #[derive(Clone)]
                                                    struct RenderEnv {
                                                        reserve_leading_slot_enabled: bool,
                                                        item_count: usize,
                                                        ring: RingStyle,
                                                        border: fret_core::Color,
                                                        radius_sm: Px,
                                                        pad_x: Px,
                                                        pad_x_inset: Px,
                                                        pad_y: Px,
                                                        font_size: Px,
                                                        font_line_height: Px,
                                                        text_style: TextStyle,
                                                        text_disabled: fret_core::Color,
                                                        label_fg: fret_core::Color,
                                                        accent: fret_core::Color,
                                                        accent_fg: fret_core::Color,
                                                        fg: fret_core::Color,
                                                        icon_muted_fg: fret_core::Color,
                                                        destructive_fg: fret_core::Color,
                                                        destructive_bg: fret_core::Color,
                                                        row_height: Px,
                                                        window_margin: Px,
                                                        submenu_min_width: Px,
                                                        submenu_max_height_metric: Option<Px>,
                                                        open: Model<bool>,
                                                        submenu_for_content: menu::sub::MenuSubmenuModels,
                                                        submenu_cfg: menu::sub::MenuSubmenuConfig,
                                                        overlay_root_name_for_controls: Arc<str>,
                                                        first_item_focus_id: Rc<Cell<Option<GlobalElementId>>>,
                                                        last_item_focus_id: Rc<Cell<Option<GlobalElementId>>>,
                                                    }

                                                    fn render_entries<H: UiHost>(
                                                        cx: &mut ElementContext<'_, H>,
                                                        entries: &[DropdownMenuEntry],
                                                        item_ix: &mut usize,
                                                        env: &RenderEnv,
                                                    ) -> Vec<AnyElement> {
                                                        let reserve_leading_slot_enabled =
                                                            env.reserve_leading_slot_enabled;
                                                        let item_count = env.item_count;
                                                        let ring = env.ring.clone();
                                                        let border = env.border;
                                                        let radius_sm = env.radius_sm;
                                                        let pad_x = env.pad_x;
                                                        let pad_x_inset = env.pad_x_inset;
                                                        let pad_y = env.pad_y;
                                                        let font_size = env.font_size;
                                                        let font_line_height = env.font_line_height;
                                                        let text_style = env.text_style.clone();
                                                        let text_disabled = env.text_disabled;
                                                        let label_fg = env.label_fg;
                                                        let accent = env.accent;
                                                        let accent_fg = env.accent_fg;
                                                        let fg = env.fg;
                                                        let icon_muted_fg = env.icon_muted_fg;
                                                        let destructive_fg = env.destructive_fg;
                                                        let destructive_bg = env.destructive_bg;
                                                        let row_height = env.row_height;
                                                        let window_margin = env.window_margin;
                                                        let submenu_min_width = env.submenu_min_width;
                                                        let submenu_max_height_metric =
                                                            env.submenu_max_height_metric;
                                                        let open_for_menu = env.open.clone();
                                                        let submenu_for_content =
                                                            env.submenu_for_content.clone();
                                                        let submenu_cfg = env.submenu_cfg.clone();
                                                        let overlay_root_name_for_controls =
                                                            env.overlay_root_name_for_controls.clone();
                                                        let first_item_focus_id_for_items =
                                                            env.first_item_focus_id.clone();
                                                        let last_item_focus_id_for_items =
                                                            env.last_item_focus_id.clone();

                                                        let mut out: Vec<AnyElement> =
                                                            Vec::with_capacity(entries.len());

                                                        for entry in entries.iter().cloned() {
                                                            match entry {
                                                    DropdownMenuEntry::Label(label) => {
                                                        let fg = label_fg;
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
                                                                vec![ui::text(cx, text)
                                                                    .text_size_px(font_size)
                                                                    .line_height_px(font_line_height)
                                                                    .font_medium()
                                                                    .nowrap()
                                                                    .text_color(ColorRef::Color(fg))
                                                                    .into_element(cx)]
                                                            },
                                                        ));
                                                    }
                                                    DropdownMenuEntry::Group(group) => {
                                                        let children =
                                                            render_entries(cx, &group.entries, item_ix, env);
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children,
                                                        ));
                                                    }
                                                    DropdownMenuEntry::RadioGroup(group) => {
                                                        let group_value = group.value.clone();
                                                        let items: Vec<DropdownMenuEntry> = group
                                                            .items
                                                            .into_iter()
                                                            .map(|spec| {
                                                                DropdownMenuEntry::RadioItem(
                                                                    spec.into_item(group_value.clone()),
                                                                )
                                                            })
                                                            .collect();
                                                        let children =
                                                            render_entries(cx, &items, item_ix, env);
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children,
                                                        ));
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
                                                        let collection_index = *item_ix;
                                                        *item_ix = (*item_ix).saturating_add(1);

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
                                                                        if first_item_focus_id_for_items.get().is_none() {
                                                                            first_item_focus_id_for_items.set(Some(item_id));
                                                                        }
                                                                        last_item_focus_id_for_items.set(Some(item_id));
                                                                        menu::checkbox_item::wire_toggle_on_activate(
                                                                            cx,
                                                                            checked.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
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

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                        })
                                                                    });

                                                                    let children = checkable_menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot_enabled,
                                                                        trailing,
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
                                                        let collection_index = *item_ix;
                                                        *item_ix = (*item_ix).saturating_add(1);

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
                                                                        if first_item_focus_id_for_items.get().is_none() {
                                                                            first_item_focus_id_for_items.set(Some(item_id));
                                                                        }
                                                                        last_item_focus_id_for_items.set(Some(item_id));
                                                                        menu::radio_group::wire_select_on_activate(
                                                                            cx,
                                                                            group_value.clone(),
                                                                            value.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
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

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                        })
                                                                    });

                                                                    let children = checkable_menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot_enabled,
                                                                        trailing,
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
                                                        let collection_index = *item_ix;
                                                        *item_ix = (*item_ix).saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let test_id = item.test_id.clone();
                                                        let disabled = item.disabled;
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let leading = item.leading.clone();
                                                        let trailing = item.trailing.clone();
                                                        let content = item.content.clone();
                                                        let padding_override = item.padding;
                                                        let estimated_height = item.estimated_height;
                                                        let variant = item.variant;
                                                        let has_submenu = item.submenu.is_some();
                                                        let submenu_entries_for_hint = item.submenu.clone();
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
                                                                    let submenu_max_h =
                                                                        submenu_max_height_metric
                                                                            .map(|h| {
                                                                                Px(h.0.min(
                                                                                    outer.size.height.0,
                                                                                ))
                                                                            })
                                                                            .unwrap_or(outer.size.height);
                                                                    let entries_for_estimate =
                                                                        submenu_entries_for_hint
                                                                            .clone()
                                                                            .unwrap_or_default();
                                                                    let desired_h =
                                                                        estimated_menu_panel_height_for_entries(
                                                                            &entries_for_estimate,
                                                                            row_height,
                                                                            submenu_max_h,
                                                                        );
                                                                    let desired =
                                                                        Size::new(submenu_min_width, desired_h);
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

                                                                if !disabled {
                                                                    if first_item_focus_id_for_items.get().is_none() {
                                                                        first_item_focus_id_for_items.set(Some(item_id));
                                                                    }
                                                                    last_item_focus_id_for_items.set(Some(item_id));
                                                                }

                                                                if !has_submenu && !disabled {
                                                                    cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                    if close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }
                                                                }

                                                                let controls = has_submenu.then(|| {
                                                                    menu::sub_content::submenu_content_semantics_id(
                                                                        cx,
                                                                        overlay_root_name_for_controls.as_ref(),
                                                                        &value,
                                                                    )
                                                                });
                                                                let mut a11y =
                                                                    menu::item::menu_item_a11y_with_controls(
                                                                        a11y_label,
                                                                        has_submenu
                                                                            .then_some(is_open_submenu),
                                                                        controls,
                                                                    );
                                                                a11y.test_id = test_id.clone();
                                                                let props = PressableProps {
                                                                    layout: {
                                                                        let mut layout = LayoutStyle::default();
                                                                        layout.size.width = Length::Fill;
                                                                        layout.size.min_height = Some(row_height);
                                                                        if let Some(h) = estimated_height {
                                                                            layout.size.height = Length::Px(h);
                                                                            layout.size.min_height = Some(h);
                                                                        }
                                                                        layout
                                                                    },
                                                                    enabled: !disabled,
                                                                    focusable: !disabled,
                                                                    focus_ring: Some(ring),
                                                                    a11y: a11y.with_collection_position(
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

                                                                let trailing = if has_submenu {
                                                                    trailing.clone()
                                                                } else {
                                                                    trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                        })
                                                                    })
                                                                };

                                                                let row_padding = padding_override.unwrap_or(Edges {
                                                                    top: pad_y,
                                                                    right: pad_x,
                                                                    bottom: pad_y,
                                                                    left: pad_left,
                                                                });

                                                                let children = vec![cx.container(
                                                                        ContainerProps {
                                                                            layout: LayoutStyle::default(),
                                                                            padding: row_padding,
                                                                            background: Some(row_bg),
                                                                            corner_radii: fret_core::Corners::all(radius_sm),
                                                                            ..Default::default()
                                                                        },
                                                                    move |cx| {
                                                                        if let Some(custom) = content.clone() {
                                                                            return vec![custom];
                                                                        }

                                                                        let mut row: Vec<AnyElement> = Vec::with_capacity(
                                                                            2 + usize::from(
                                                                                leading.is_some()
                                                                                    || reserve_leading_slot_enabled,
                                                                            ) + usize::from(trailing.is_some())
                                                                                + usize::from(has_submenu),
                                                                        );
                                                                        if let Some(l) = leading.clone() {
                                                                            row.push(menu_icon_slot(cx, l));
                                                                        } else if reserve_leading_slot_enabled {
                                                                            row.push(menu_icon_slot_empty(cx));
                                                                        }
                                                                        let style = text_style.clone();
                                                                        let mut text = ui::text(cx, label.clone())
                                                                            .layout(LayoutRefinement::default().min_w_0().flex_1())
                                                                            .text_size_px(style.size)
                                                                            .font_weight(style.weight)
                                                                            .nowrap()
                                                                            .text_color(ColorRef::Color(if disabled { text_disabled } else { row_fg }));

                                                                        if let Some(line_height) = style.line_height {
                                                                            text = text.line_height_px(line_height);
                                                                        }

                                                                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                                                                            text = text.letter_spacing_em(letter_spacing_em);
                                                                        }

                                                                        row.push(text.into_element(cx));

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
                                                    }

                                                    let env = RenderEnv {
                                                        reserve_leading_slot_enabled,
                                                        item_count,
                                                        ring,
                                                        border,
                                                        radius_sm,
                                                        pad_x,
                                                        pad_x_inset,
                                                        pad_y,
                                                        font_size,
                                                        font_line_height,
                                                        text_style,
                                                        text_disabled,
                                                        label_fg: icon_muted_fg,
                                                        accent,
                                                        accent_fg,
                                                        fg,
                                                        icon_muted_fg,
                                                        destructive_fg,
                                                        destructive_bg,
                                                        row_height,
                                                        window_margin,
                                                        submenu_min_width,
                                                        submenu_max_height_metric: theme
                                                            .metric_by_key(
                                                                "component.dropdown_menu.max_height",
                                                            ),
                                                        open: open_for_menu.clone(),
                                                        submenu_for_content:
                                                            submenu_for_content.clone(),
                                                        submenu_cfg,
                                                        overlay_root_name_for_controls:
                                                            overlay_root_name_for_controls_for_content
                                                                .clone(),
                                                        first_item_focus_id:
                                                            first_item_focus_id_for_items.clone(),
                                                        last_item_focus_id:
                                                            last_item_focus_id_for_items.clone(),
                                                    };

                                                    render_entries(
                                                        cx,
                                                        entries.as_ref(),
                                                        &mut item_ix,
                                                        &env,
                                                    )
                                                },
                                            );
                                            vec![roving]
                                        },
                                        )
                                    })]
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
                    content_focus_id_for_children.set(Some(content_id));
                    cx.key_on_key_down_for(
                        content_id,
                        Arc::new({
                            let first_item_focus_id = first_item_focus_id.clone();
                            let last_item_focus_id = last_item_focus_id.clone();
                            move |host, _cx, it| {
                                if it.repeat {
                                    return false;
                                }
                                match it.key {
                                    fret_core::KeyCode::ArrowDown => {
                                        let Some(target) = first_item_focus_id.get() else {
                                            return false;
                                        };
                                        host.request_focus(target);
                                        true
                                    }
                                    fret_core::KeyCode::ArrowUp => {
                                        let Some(target) = last_item_focus_id.get() else {
                                            return false;
                                        };
                                        host.request_focus(target);
                                        true
                                    }
                                    _ => false,
                                }
                            }
                        }),
                    );

                    let content =
                        overlay_motion::wrap_opacity_and_render_transform(cx, opacity, transform, vec![content]);

                    let dismissible_on_pointer_move =
                        menu::root::submenu_pointer_move_handler(submenu.clone(), submenu_cfg);

                    let mut children = vec![content];
                    let submenu_open_value = cx
                        .watch_model(&submenu_for_panel.open_value)
                        .layout()
                        .cloned()
                        .unwrap_or(None);
                    let desired = submenu_open_value
                        .as_deref()
                        .and_then(|open_value| {
                            find_submenu_entries_by_value(entries_for_submenu.as_ref(), open_value)
                        })
                        .map(|submenu_entries| {
                            let submenu_max_h = theme
                                .metric_by_key("component.dropdown_menu.max_height")
                                .map(|h| Px(h.0.min(outer.size.height.0)))
                                .unwrap_or(outer.size.height);
                            let desired_h = estimated_menu_panel_height_for_entries(
                                &submenu_entries,
                                row_height,
                                submenu_max_h,
                            );
                            Size::new(submenu_min_width, desired_h)
                        })
                        .unwrap_or_else(|| {
                            let submenu_max_h = theme
                                .metric_by_key("component.dropdown_menu.max_height")
                                .map(|h| Px(h.0.min(outer.size.height.0)))
                                .unwrap_or(outer.size.height);
                            Size::new(submenu_min_width, submenu_max_h)
                        });
                    let submenu_is_open = submenu_open_value.is_some();
                    let submenu_present = submenu_is_open;
                    let submenu_opacity = 1.0;
                    let submenu_scale = 1.0;

                    let open_submenu = menu::sub::with_open_submenu_synced(
                        cx,
                        &submenu_for_panel,
                        outer,
                        desired,
                        |_cx, open_value, geometry| (open_value, geometry),
                    );

                    #[derive(Default)]
                    struct SubmenuLastGeometry {
                        geometry: Option<menu::sub::MenuSubmenuGeometry>,
                    }

                    let last_geometry = cx.with_state(SubmenuLastGeometry::default, |st| {
                        if let Some((_, geometry)) = open_submenu.as_ref() {
                            st.geometry = Some(*geometry);
                        }
                        st.geometry
                    });

                    let mut submenu_gate: Option<AnyElement> = None;
                    if submenu_present {
                        let Some(open_value) = submenu_open_value.clone() else {
                            return (children, Some(dismissible_on_pointer_move));
                        };
                        let geometry = open_submenu.map(|(_, geometry)| geometry).or(last_geometry);

                        let Some(geometry) = geometry else {
                            return (children, Some(dismissible_on_pointer_move));
                        };

                        let submenu_entries = find_submenu_entries_by_value(
                            entries_for_submenu.as_ref(),
                            open_value.as_ref(),
                        );

                        if let Some(submenu_entries) = submenu_entries {
                            let reserve_leading_slot_enabled =
                                align_leading_icons && reserve_leading_slot(&submenu_entries);
                            let item_count = focusable_item_count(&submenu_entries);

                                            let font_size = theme.metric_required("font.size");
                                            let font_line_height =
                                                theme.metric_required("font.line_height");
                                            let text_disabled =
                                                alpha_mul(theme.color_required("foreground"), 0.5);
                                            let destructive_fg = theme.color_required("destructive");
                                            let destructive_bg_alpha =
                                                if is_dark_background(&theme) { 0.20 } else { 0.10 };
                                            let destructive_bg = theme
                                                .color_by_key(if destructive_bg_alpha >= 0.2 {
                                                    "destructive/20"
                                                } else {
                                                    "destructive/10"
                                                })
                                                .unwrap_or_else(|| {
                                                    alpha_mul(destructive_fg, destructive_bg_alpha)
                                                });
                                            let label_fg = theme.color_required("muted-foreground");

                                            let text_style = TextStyle {
                                                font: FontId::default(),
                                                size: font_size,
                                                weight: FontWeight::NORMAL,
                                                slant: Default::default(),
                                                line_height: Some(font_line_height),
                                                letter_spacing_em: None,
                                            };

                                            let mut submenu_labels: Vec<Arc<str>> =
                                                Vec::with_capacity(item_count);
                                            let mut submenu_disabled_flags: Vec<bool> =
                                                Vec::with_capacity(item_count);
                                            collect_roving_labels_and_disabled(
                                                &submenu_entries,
                                                &mut submenu_labels,
                                                &mut submenu_disabled_flags,
                                            );
                                            let submenu_labels_arc: Arc<[Arc<str>]> =
                                                Arc::from(submenu_labels.into_boxed_slice());
                                            let submenu_disabled_arc: Arc<[bool]> = Arc::from(
                                                submenu_disabled_flags.into_boxed_slice(),
                                            );
                                            let roving = RovingFocusProps {
                                                enabled: true,
                                                wrap: false,
                                                disabled: submenu_disabled_arc,
                                                ..Default::default()
                                            };

                                            let submenu_models_for_panel = submenu_for_panel.clone();
                                            let labelled_by_element = cx
                                                .app
                                                .models_mut()
                                                .read(&submenu_models_for_panel.trigger, |v| *v)
                                                .ok()
                                                .flatten();
                                            let theme_for_submenu_panel = theme.clone();
                                            let submenu_chrome_for_panel = submenu_chrome.clone();
                                            let submenu_panel = menu::sub_content::submenu_panel_scroll_y_for_value_at(
                                                cx,
                                                open_value.clone(),
                                                geometry.floating,
                                                labelled_by_element,
                                                move |layout| {
                                                    let mut props = decl_style::container_props(
                                                        &theme_for_submenu_panel,
                                                        submenu_chrome_for_panel.clone(),
                                                        LayoutRefinement::default(),
                                                    );
                                                    props.layout = layout;
                                                    props
                                                },
                                                move |cx| {
                                                    let mut item_ix: usize = 0;

                                                    #[derive(Clone)]
                                                    struct RenderEnv {
                                                        reserve_leading_slot_enabled: bool,
                                                        item_count: usize,
                                                        ring: RingStyle,
                                                        border: fret_core::Color,
                                                        radius_sm: Px,
                                                        pad_x: Px,
                                                        pad_x_inset: Px,
                                                        pad_y: Px,
                                                        font_size: Px,
                                                        font_line_height: Px,
                                                        text_style: TextStyle,
                                                        text_disabled: fret_core::Color,
                                                        label_fg: fret_core::Color,
                                                        accent: fret_core::Color,
                                                        accent_fg: fret_core::Color,
                                                        fg: fret_core::Color,
                                                        destructive_fg: fret_core::Color,
                                                        destructive_bg: fret_core::Color,
                                                        row_height: Px,
                                                        window_margin: Px,
                                                        submenu_min_width: Px,
                                                        submenu_max_height_metric: Option<Px>,
                                                        open: Model<bool>,
                                                        submenu_models: menu::sub::MenuSubmenuModels,
                                                        submenu_cfg: menu::sub::MenuSubmenuConfig,
                                                        overlay_root_name_for_controls: Arc<str>,
                                                    }

                                                    fn render_entries<H: UiHost>(
                                                        cx: &mut ElementContext<'_, H>,
                                                        entries: &[DropdownMenuEntry],
                                                        item_ix: &mut usize,
                                                        env: &RenderEnv,
                                                    ) -> Vec<AnyElement> {
                                                        let reserve_leading_slot_enabled =
                                                            env.reserve_leading_slot_enabled;
                                                        let item_count = env.item_count;
                                                        let ring = env.ring.clone();
                                                        let border = env.border;
                                                        let radius_sm = env.radius_sm;
                                                        let pad_x = env.pad_x;
                                                        let pad_x_inset = env.pad_x_inset;
                                                        let pad_y = env.pad_y;
                                                        let font_size = env.font_size;
                                                        let font_line_height = env.font_line_height;
                                                        let text_style = env.text_style.clone();
                                                        let text_disabled = env.text_disabled;
                                                        let label_fg = env.label_fg;
                                                        let accent = env.accent;
                                                        let accent_fg = env.accent_fg;
                                                        let fg = env.fg;
                                                        let destructive_fg = env.destructive_fg;
                                                        let destructive_bg = env.destructive_bg;
                                                        let _row_height = env.row_height;
                                                        let _window_margin = env.window_margin;
                                                        let _submenu_min_width = env.submenu_min_width;
                                                        let _submenu_max_height_metric =
                                                            env.submenu_max_height_metric;
                                                        let open_for_submenu = env.open.clone();
                                                        let submenu_models_for_panel =
                                                            env.submenu_models.clone();
                                                        let _submenu_cfg = env.submenu_cfg.clone();
                                                        let _overlay_root_name_for_controls =
                                                            env.overlay_root_name_for_controls.clone();
                                                        let gating = crate::command_gating::snapshot_for_window(
                                                            &*cx.app,
                                                            cx.window,
                                                        );

                                                        let mut rows: Vec<AnyElement> =
                                                            Vec::with_capacity(entries.len());

                                                        for entry in entries.iter().cloned() {
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
                                                                        vec![ui::text(cx, text)
                                                                            .text_size_px(font_size)
                                                                            .line_height_px(font_line_height)
                                                                            .font_medium()
                                                                            .nowrap()
                                                                            .text_color(ColorRef::Color(label_fg))
                                                                            .into_element(cx)]
                                                                    },
                                                                ));
                                                            }
                                                            DropdownMenuEntry::Group(group) => {
                                                                let children = render_entries(
                                                                    cx,
                                                                    &group.entries,
                                                                    item_ix,
                                                                    env,
                                                                );
                                                                rows.push(menu_structural_group(
                                                                    cx,
                                                                    fret_core::SemanticsRole::Group,
                                                                    children,
                                                                ));
                                                            }
                                                            DropdownMenuEntry::RadioGroup(group) => {
                                                                let group_value = group.value.clone();
                                                                let items: Vec<DropdownMenuEntry> = group
                                                                    .items
                                                                    .into_iter()
                                                                    .map(|spec| {
                                                                        DropdownMenuEntry::RadioItem(
                                                                            spec.into_item(group_value.clone()),
                                                                        )
                                                                    })
                                                                    .collect();
                                                                let children =
                                                                    render_entries(cx, &items, item_ix, env);
                                                                rows.push(menu_structural_group(
                                                                    cx,
                                                                    fret_core::SemanticsRole::Group,
                                                                    children,
                                                                ));
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
                                                                let collection_index = *item_ix;
                                                                *item_ix = (*item_ix).saturating_add(1);

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
                                                                            cx.pressable_dispatch_command_if_enabled_opt(command.clone());
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

                                                                            let trailing = trailing.clone().or_else(|| {
                                                                                command.as_ref().and_then(|cmd| {
                                                                                    command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                        .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                                })
                                                                            });

                                                                            let children = checkable_menu_row_children(
                                                                                cx,
                                                                                label.clone(),
                                                                                leading.clone(),
                                                                                reserve_leading_slot_enabled,
                                                                                trailing,
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
                                                                let collection_index = *item_ix;
                                                                *item_ix = (*item_ix).saturating_add(1);

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
                                                                            cx.pressable_dispatch_command_if_enabled_opt(command.clone());
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

                                                                            let trailing = trailing.clone().or_else(|| {
                                                                                command.as_ref().and_then(|cmd| {
                                                                                    command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                        .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                                })
                                                                            });

                                                                            let children = checkable_menu_row_children(
                                                                                cx,
                                                                                label.clone(),
                                                                                leading.clone(),
                                                                                reserve_leading_slot_enabled,
                                                                                trailing,
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
                                                        let collection_index = *item_ix;
                                                        *item_ix = (*item_ix).saturating_add(1);

                                                        let label = item.label.clone();
                                                        let value = item.value.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let test_id = item.test_id.clone();
                                                        let close_on_select = item.close_on_select;
                                                        let command = item.command;
                                                        let disabled = item.disabled
                                                            || crate::command_gating::command_is_disabled_by_gating(
                                                                &*cx.app,
                                                                &gating,
                                                                command.as_ref(),
                                                            );
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
                                                                            cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                            if !disabled && close_on_select {
                                                                                cx.pressable_set_bool(&open, false);
                                                                            }

                                                                            let mut a11y = menu::item::menu_item_a11y(a11y_label, None);
                                                                            a11y.test_id = test_id.clone();
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
                                                                                a11y: a11y.with_collection_position(
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

                                                                            let trailing = trailing.clone().or_else(|| {
                                                                                command.as_ref().and_then(|cmd| {
                                                                                    command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                        .map(|text| DropdownMenuShortcut::new(text).into_element(cx))
                                                                                })
                                                                            });

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
                                                                                                || reserve_leading_slot_enabled,
                                                                                        ) + usize::from(trailing.is_some()),
                                                                                    );
                                                                                    if let Some(l) = leading.clone() {
                                                                                        row.push(menu_icon_slot(cx, l));
                                                                                    } else if reserve_leading_slot_enabled {
                                                                                        row.push(menu_icon_slot_empty(cx));
                                                                                    }
                                                                                    let style = text_style.clone();
                                                                                    let mut text = ui::text(cx, label.clone())
                                                                                        .layout(LayoutRefinement::default().min_w_0().flex_1())
                                                                                        .text_size_px(style.size)
                                                                                        .font_weight(style.weight)
                                                                                        .nowrap()
                                                                                        .text_color(ColorRef::Color(if disabled { text_disabled } else { row_fg }));

                                                                                    if let Some(line_height) = style.line_height {
                                                                                        text = text.line_height_px(line_height);
                                                                                    }

                                                                                    if let Some(letter_spacing_em) = style.letter_spacing_em {
                                                                                        text = text.letter_spacing_em(letter_spacing_em);
                                                                                    }

                                                                                    row.push(text.into_element(cx));

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

                                                    rows
                                                    }

                                                    let env = RenderEnv {
                                                        reserve_leading_slot_enabled,
                                                        item_count,
                                                        ring,
                                                        border,
                                                        radius_sm,
                                                        pad_x,
                                                        pad_x_inset,
                                                        pad_y,
                                                        font_size,
                                                        font_line_height,
                                                        text_style,
                                                        text_disabled,
                                                        label_fg,
                                                        accent,
                                                        accent_fg,
                                                        fg,
                                                        destructive_fg,
                                                        destructive_bg,
                                                        row_height,
                                                        window_margin,
                                                        submenu_min_width,
                                                        submenu_max_height_metric: theme
                                                            .metric_by_key(
                                                                "component.dropdown_menu.max_height",
                                                            ),
                                                        open: open_for_submenu.clone(),
                                                        submenu_models: submenu_models_for_panel.clone(),
                                                        submenu_cfg,
                                                        overlay_root_name_for_controls:
                                                            overlay_root_name_for_controls_for_submenu
                                                                .clone(),
                                                    };

                                                    let rows = render_entries(
                                                        cx,
                                                        submenu_entries.as_ref(),
                                                        &mut item_ix,
                                                        &env,
                                                    );

                                                    let roving = menu::sub_content::submenu_roving_group_apg_prefix_typeahead(
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
                                                        submenu_models_for_panel.clone(),
                                                        move |_cx| rows.clone(),
                                                    );
                                                    vec![roving]
                                                },
                                            );

                                        let side = overlay_motion::anchored_side(
                                            geometry.reference,
                                            geometry.floating,
                                        );
                                        let origin =
                                            overlay_motion::shadcn_transform_origin_for_anchored_rect(
                                                geometry.reference,
                                                geometry.floating,
                                                side,
                                            );
                                        let transform = overlay_motion::shadcn_popper_presence_transform(
                                            side,
                                            origin,
                                            submenu_opacity,
                                            submenu_scale,
                                            true,
                                        );

                                        let submenu_panel =
                                            overlay_motion::wrap_opacity_and_render_transform(
                                                cx,
                                                submenu_opacity,
                                                transform,
                                                vec![submenu_panel],
                                            );

                                        submenu_gate = Some(submenu_panel);
                                    }
                                }

                    #[cfg(debug_assertions)]
                    if std::env::var_os("FRET_DEBUG_DROPDOWN_SUBMENU_RENDER").is_some() {
                        eprintln!(
                            "dropdown_menu submenu render: frame={} open_value={:?} is_open={} present={} gate_child_present={}",
                            cx.app.frame_id().0,
                            submenu_open_value,
                            submenu_is_open,
                            submenu_present,
                            submenu_gate.is_some()
                        );
                    }

                    let mut submenu_gate_layout = LayoutStyle::default();
                    submenu_gate_layout.size.width = Length::Fill;
                    submenu_gate_layout.size.height = Length::Fill;
                    let submenu_gate = cx.interactivity_gate_props(
                        fret_ui::element::InteractivityGateProps {
                            layout: submenu_gate_layout,
                            present: submenu_present,
                            interactive: submenu_is_open,
                        },
                        move |_cx| submenu_gate.into_iter().collect::<Vec<_>>(),
                    );
                    children.push(submenu_gate);

                    (children, Some(dismissible_on_pointer_move))
                });

                let request = menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
                    cx,
                    overlay_id,
                    trigger_id,
                    open,
                    overlay_presence,
                    overlay_children,
                    overlay_root_name,
                    menu::root::MenuInitialFocusTargets::new()
                        .pointer_content_focus(content_focus_id.get())
                        .keyboard_entry_focus(first_item_focus_id_for_request.get()),
                    None,
                    None,
                    on_dismiss_request.clone(),
                    dismissible_on_pointer_move,
                    modal,
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

    use std::sync::atomic::{AtomicUsize, Ordering};

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
    use fret_ui::element::PressableA11y;
    use fret_ui::UiTree;
    use fret_ui_kit::primitives::direction as direction_prim;
    use fret_ui_kit::primitives::direction::LayoutDirection;

    #[test]
    fn estimated_menu_panel_height_clamps_to_max_height() {
        let entries: Vec<DropdownMenuEntry> = (0..100)
            .map(|i| {
                DropdownMenuEntry::Item(
                    DropdownMenuItem::new(format!("Item {i}")).on_select(CommandId::new("noop")),
                )
            })
            .collect();

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height = estimated_menu_panel_height_for_entries(&entries, row_height, max_height);
        assert_eq!(height, max_height);
    }

    #[test]
    fn estimated_menu_panel_height_shrinks_for_short_menus() {
        let entries = vec![
            DropdownMenuEntry::Item(
                DropdownMenuItem::new("Apple").on_select(CommandId::new("noop")),
            ),
            DropdownMenuEntry::Item(
                DropdownMenuItem::new("Orange").on_select(CommandId::new("noop")),
            ),
        ];

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height = estimated_menu_panel_height_for_entries(&entries, row_height, max_height);
        assert_eq!(height, Px(50.0));
    }

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

        let changed_models = app.take_changed_models();
        let changed_globals = app.take_changed_globals();
        let _ =
            fret_ui::frame_pipeline::propagate_changes(ui, app, &changed_models, &changed_globals);

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

    fn render_frame_capture_submenu_models(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        submenu_models_out: Model<Option<menu::sub::MenuSubmenuModels>>,
        entries: Vec<DropdownMenuEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        let changed_models = app.take_changed_models();
        let changed_globals = app.take_changed_globals();
        let _ =
            fret_ui::frame_pipeline::propagate_changes(ui, app, &changed_models, &changed_globals);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "dropdown-menu",
            move |cx| {
                let submenu_models_out = submenu_models_out.clone();
                vec![DropdownMenu::new(open.clone()).into_element(
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

                        let is_open = cx.app.models_mut().get_copied(&open).unwrap_or(false);
                        let overlay_root_name = menu::dropdown_menu_root_name(cx.root_id());
                        let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
                        let submenu_models = cx.with_root_name(&overlay_root_name, |cx| {
                            menu::root::sync_root_open_and_ensure_submenu(
                                cx,
                                is_open,
                                cx.root_id(),
                                submenu_cfg,
                            )
                        });
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&submenu_models_out, |v| *v = Some(submenu_models));

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

    fn render_frame_focusable_trigger_capture_id(
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
            "dropdown-menu-trigger",
            move |cx| {
                let trigger_id_out = trigger_id_out_for_render.clone();
                // Radix `DropdownMenu` defaults to `modal=true`, which hides underlay content from
                // the accessibility tree via `hideOthers(MenuContent)`. These tests assert trigger
                // semantics while the menu is open, so we run with `modal=false`.
                vec![DropdownMenu::new(open).modal(false).into_element(
                    cx,
                    move |cx| {
                        cx.pressable_with_id_props(move |cx, _st, id| {
                            let _ = cx
                                .app
                                .models_mut()
                                .update(&trigger_id_out, |v| *v = Some(id));
                            let props = PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            };
                            let children =
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())];
                            (props, children)
                        })
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

    fn render_frame_capture_trigger_id_with_direction(
        dir: LayoutDirection,
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
            "dropdown-menu-dir",
            move |cx| {
                direction_prim::with_direction_provider(cx, dir, |cx| {
                    vec![cx.container(
                        ContainerProps {
                            padding: Edges {
                                top: Px(100.0),
                                right: Px(0.0),
                                bottom: Px(0.0),
                                left: Px(500.0),
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let trigger_id_out = trigger_id_out_for_render.clone();
                            // See `render_frame_focusable_trigger_capture_id`: we need a non-modal
                            // menu so trigger semantics remain visible while open.
                            vec![DropdownMenu::new(open)
                                .modal(false)
                                .arrow(false)
                                .into_element(
                                    cx,
                                    move |cx| {
                                        cx.pressable_with_id_props(move |cx, _st, id| {
                                            let _ = cx
                                                .app
                                                .models_mut()
                                                .update(&trigger_id_out, |v| *v = Some(id));
                                            (
                                                PressableProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Px(Px(120.0));
                                                        layout.size.height = Length::Px(Px(40.0));
                                                        layout
                                                    },
                                                    enabled: true,
                                                    focusable: true,
                                                    a11y: PressableA11y {
                                                        label: Some(Arc::from("Trigger")),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                vec![cx
                                                    .container(ContainerProps::default(), |_cx| {
                                                        Vec::new()
                                                    })],
                                            )
                                        })
                                    },
                                    move |_cx| entries.clone(),
                                )]
                        },
                    )]
                })
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

    #[test]
    fn dropdown_menu_align_start_respects_direction_provider() {
        fn run(dir: LayoutDirection) -> (Rect, Rect) {
            let window = AppWindowId::default();
            let mut app = App::new();
            let mut ui: UiTree<App> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::Size::new(Px(1200.0), Px(700.0)),
            );
            let mut services = FakeServices::default();

            let open = app.models_mut().insert(true);
            let trigger_id_out = app.models_mut().insert(None);

            let entries = vec![DropdownMenuEntry::Item(
                DropdownMenuItem::new("Alpha").value("alpha"),
            )];

            // Two frames: first establishes trigger bounds; second mounts the overlay anchored to them.
            let _ = render_frame_capture_trigger_id_with_direction(
                dir,
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                trigger_id_out.clone(),
                entries.clone(),
            );
            let _ = render_frame_capture_trigger_id_with_direction(
                dir,
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open,
                trigger_id_out.clone(),
                entries,
            );

            let snap = ui.semantics_snapshot().expect("semantics snapshot");
            let trigger = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Trigger"))
                .expect("trigger semantics");
            let trigger_bounds = trigger.bounds;
            let alpha = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
                .expect("Alpha menu item");
            (trigger_bounds, alpha.bounds)
        }

        let (ltr_trigger, ltr_item) = run(LayoutDirection::Ltr);
        let ltr_trigger_center = ltr_trigger.origin.x.0 + ltr_trigger.size.width.0 * 0.5;
        let ltr_item_center = ltr_item.origin.x.0 + ltr_item.size.width.0 * 0.5;
        assert!(
            ltr_item_center > ltr_trigger_center,
            "expected LTR start alignment to place the menu content to the right of the trigger center; trigger={ltr_trigger:?} item={ltr_item:?}",
        );

        let (rtl_trigger, rtl_item) = run(LayoutDirection::Rtl);
        let rtl_trigger_center = rtl_trigger.origin.x.0 + rtl_trigger.size.width.0 * 0.5;
        let rtl_item_center = rtl_item.origin.x.0 + rtl_item.size.width.0 * 0.5;
        assert!(
            rtl_item_center < rtl_trigger_center,
            "expected RTL start alignment to place the menu content to the left of the trigger center; trigger={rtl_trigger:?} item={rtl_item:?}",
        );
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

    fn render_frame_with_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
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
            "dropdown-menu-dismiss-handler",
            move |cx| {
                vec![DropdownMenu::new(open)
                    .on_dismiss_request(on_dismiss_request)
                    .into_element(
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
    fn dropdown_menu_modal_outside_press_can_be_prevented_via_dismiss_handler() {
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

        let items = vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))];

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let _ = render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler.clone()),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(handler),
            items,
        );

        let outside = Point::new(Px(390.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn dropdown_menu_trigger_controls_menu_content_when_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let trigger_id_out = app.models_mut().insert(None);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha"))];

        // First frame: capture a stable trigger element id.
        let (_, trigger_element) = render_frame_focusable_trigger_capture_id(
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

        // Second frame: menu content is mounted.
        let (_, trigger_element_2) = render_frame_focusable_trigger_capture_id(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            trigger_id_out,
            entries,
        );
        assert_eq!(
            trigger_element_2, trigger_element,
            "expected trigger element id to be stable across open state"
        );

        let trigger_node = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "dropdown-menu-trigger",
            |cx| cx.node_for_element(trigger_element),
        )
        .expect("expected trigger element node");

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_sem = snap.nodes.iter().find(|n| n.id == trigger_node).unwrap_or_else(|| {
            use std::collections::BTreeMap;

            fn role_name(role: SemanticsRole) -> &'static str {
                match role {
                    SemanticsRole::Generic => "Generic",
                    SemanticsRole::Window => "Window",
                    SemanticsRole::Panel => "Panel",
                    SemanticsRole::Group => "Group",
                    SemanticsRole::Dialog => "Dialog",
                    SemanticsRole::AlertDialog => "AlertDialog",
                    SemanticsRole::Alert => "Alert",
                    SemanticsRole::Button => "Button",
                    SemanticsRole::Checkbox => "Checkbox",
                    SemanticsRole::Switch => "Switch",
                    SemanticsRole::Slider => "Slider",
                    SemanticsRole::ComboBox => "ComboBox",
                    SemanticsRole::RadioGroup => "RadioGroup",
                    SemanticsRole::RadioButton => "RadioButton",
                    SemanticsRole::TabList => "TabList",
                    SemanticsRole::Tab => "Tab",
                    SemanticsRole::TabPanel => "TabPanel",
                    SemanticsRole::MenuBar => "MenuBar",
                    SemanticsRole::Menu => "Menu",
                    SemanticsRole::MenuItem => "MenuItem",
                    SemanticsRole::MenuItemCheckbox => "MenuItemCheckbox",
                    SemanticsRole::MenuItemRadio => "MenuItemRadio",
                    SemanticsRole::Tooltip => "Tooltip",
                    SemanticsRole::Text => "Text",
                    SemanticsRole::TextField => "TextField",
                    SemanticsRole::List => "List",
                    SemanticsRole::ListItem => "ListItem",
                    SemanticsRole::ListBox => "ListBox",
                    SemanticsRole::ListBoxOption => "ListBoxOption",
                    SemanticsRole::TreeItem => "TreeItem",
                    SemanticsRole::Viewport => "Viewport",
                    _ => "Unknown",
                }
            }

            let roots: Vec<_> = snap
                .roots
                .iter()
                .map(|r| (r.root, r.z_index, r.blocks_underlay_input, r.hit_testable))
                .collect();

            let mut role_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
            for n in &snap.nodes {
                *role_counts.entry(role_name(n.role)).or_insert(0) += 1;
            }

            let sample: Vec<_> = snap
                .nodes
                .iter()
                .take(30)
                .map(|n| (n.id, n.role, n.label.as_deref(), n.test_id.as_deref()))
                .collect();

            panic!(
                "trigger semantics node; trigger_node={trigger_node:?} roots={roots:?} role_counts={role_counts:?} sample(id,role,label,test_id)={sample:?}"
            );
        });

        assert!(
            trigger_sem.flags.expanded,
            "expected trigger to expose expanded=true while open"
        );

        let menu_content = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Menu)
            .expect("menu content semantics node");

        assert!(
            trigger_sem.controls.contains(&menu_content.id),
            "expected trigger to control menu content; controls={:?} content={:?}",
            trigger_sem.controls,
            menu_content.id
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
    fn dropdown_menu_arrow_down_moves_focus_into_menu_when_already_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            underlay_id_out,
            vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Beta")),
            ],
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open,
            trigger_id_out,
            Rc::new(Cell::new(None)),
            vec![
                DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha")),
                DropdownMenuEntry::Item(DropdownMenuItem::new("Beta")),
            ],
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");

        assert_eq!(
            ui.focus(),
            Some(alpha.id),
            "ArrowDown on open trigger should move focus to first menu item"
        );
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
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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

        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal dropdown-menu to install pointer occlusion"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
        let submenu_models_out: fret_runtime::Model<Option<menu::sub::MenuSubmenuModels>> =
            app.models_mut().insert(None);

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
        let _ = render_frame_capture_submenu_models(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            submenu_models_out.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu.
        let _ = render_frame_capture_submenu_models(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            submenu_models_out.clone(),
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
                pointer_id: fret_core::PointerId(0),
                position: rect_center(more_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
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
        let _ = render_frame_capture_submenu_models(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            submenu_models_out.clone(),
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
        let _ = render_frame_capture_submenu_models(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            submenu_models_out.clone(),
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

        let models = app
            .models_mut()
            .read(&submenu_models_out, |v| v.clone())
            .ok()
            .flatten()
            .expect("submenu models");
        let _ = app
            .models_mut()
            .read(&models.geometry, |v| *v)
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.last_pointer, |v| *v)
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.trigger, |v| *v)
            .ok()
            .flatten();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(390.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let _ = app
            .models_mut()
            .read(&models.last_pointer, |v| *v)
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.close_timer, |v| *v)
            .ok()
            .flatten();

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

        let _ = app
            .models_mut()
            .read(&models.close_timer, |v| *v)
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu should remain visible during the close delay"
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: timer });

        let _ = app
            .models_mut()
            .read(&models.close_timer, |v| *v)
            .ok()
            .flatten();
        let _ = app
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();

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
            "submenu should close after the close timer fires"
        );
    }

    #[test]
    fn dropdown_menu_submenu_wheel_scroll_brings_late_items_into_view() {
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

        let submenu_entries: Vec<DropdownMenuEntry> = (0..40)
            .map(|i| DropdownMenuEntry::Item(DropdownMenuItem::new(format!("Sub {i}"))))
            .collect();
        let entries = vec![
            DropdownMenuEntry::Item(DropdownMenuItem::new("More").submenu(submenu_entries)),
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

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: rect_center(more_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
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

        // Third frame: hover does not open the submenu immediately.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: open_timer });

        // Fourth frame: open submenu.
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
        let first = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub 0"))
            .expect("Sub 0 menu item");

        let submenu_menu_id = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Menu)
            .find(|n| ui.is_descendant(n.id, first.id))
            .map(|n| n.id)
            .expect("submenu menu id");
        let submenu_menu_bounds = ui
            .debug_node_visual_bounds(submenu_menu_id)
            .or_else(|| ui.debug_node_bounds(submenu_menu_id))
            .expect("submenu menu bounds");
        let wheel_pos = rect_center(submenu_menu_bounds);

        let viewport_bounds = {
            let path = ui.debug_node_path(first.id);
            let mut out: Option<Rect> = None;
            for window in path.windows(2) {
                let parent = window[0];
                let child = window[1];
                let parent_bounds = ui
                    .debug_node_visual_bounds(parent)
                    .or_else(|| ui.debug_node_bounds(parent));
                let child_bounds = ui
                    .debug_node_visual_bounds(child)
                    .or_else(|| ui.debug_node_bounds(child));
                let (Some(parent_bounds), Some(child_bounds)) = (parent_bounds, child_bounds)
                else {
                    continue;
                };

                let same_origin = (parent_bounds.origin.x.0 - child_bounds.origin.x.0).abs() < 0.01
                    && (parent_bounds.origin.y.0 - child_bounds.origin.y.0).abs() < 0.01;
                let same_width =
                    (parent_bounds.size.width.0 - child_bounds.size.width.0).abs() < 0.01;
                let expands_vertically =
                    child_bounds.size.height.0 > parent_bounds.size.height.0 + 1.0;
                if same_origin && same_width && expands_vertically {
                    out = Some(parent_bounds);
                }
            }
            out.unwrap_or(submenu_menu_bounds)
        };

        for _ in 0..60 {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &Event::Pointer(PointerEvent::Wheel {
                    pointer_id: fret_core::PointerId(0),
                    position: wheel_pos,
                    delta: fret_core::Point::new(Px(0.0), Px(-80.0)),
                    modifiers: Modifiers::default(),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let snap = ui
            .semantics_snapshot()
            .expect("semantics snapshot after wheel (no rerender)");
        let last = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub 39"))
            .expect("last submenu item");
        let last_id_before_rerender = last.id;
        let last_bounds = ui
            .debug_node_visual_bounds(last.id)
            .or_else(|| ui.debug_node_bounds(last.id))
            .expect("last bounds");

        let menu_top = viewport_bounds.origin.y.0;
        let menu_bottom = viewport_bounds.origin.y.0 + viewport_bounds.size.height.0;
        let last_top = last_bounds.origin.y.0;
        let last_bottom = last_bounds.origin.y.0 + last_bounds.size.height.0;

        assert!(
            last_bottom > menu_top + 0.01 && last_top < menu_bottom - 0.01,
            "expected last submenu item to be visible after wheel scrolling without rerender; menu={viewport_bounds:?} last={last_bounds:?}"
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

        let snap = ui
            .semantics_snapshot()
            .expect("semantics snapshot after wheel");
        let last = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub 39"))
            .expect("last submenu item");
        let last_id_after_rerender = last.id;
        let last_bounds = ui
            .debug_node_visual_bounds(last.id)
            .or_else(|| ui.debug_node_bounds(last.id))
            .expect("last bounds");

        let menu_top = viewport_bounds.origin.y.0;
        let menu_bottom = viewport_bounds.origin.y.0 + viewport_bounds.size.height.0;
        let last_top = last_bounds.origin.y.0;
        let last_bottom = last_bounds.origin.y.0 + last_bounds.size.height.0;

        assert!(
            last_bottom > menu_top + 0.01 && last_top < menu_bottom - 0.01,
            "expected last submenu item to be visible after wheel scrolling; menu={viewport_bounds:?} last={last_bounds:?} ids=({last_id_before_rerender:?} -> {last_id_after_rerender:?})"
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
                pointer_id: fret_core::PointerId(0),
                position: rect_center(more_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
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

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected popover overlay to be active");
        let overlay_root_name = menu::dropdown_menu_root_name(overlay_id);
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
        let open_value = app
            .models_mut()
            .read(&submenu_models.open_value, |v| v.clone())
            .ok()
            .flatten()
            .expect("expected submenu open_value to be set");
        let expected_submenu_content_element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            &overlay_root_name,
            |cx| {
                menu::sub_content::submenu_content_semantics_id(cx, &overlay_root_name, &open_value)
            },
        );
        let expected_submenu_content_node = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            &overlay_root_name,
            |cx| cx.node_for_element(expected_submenu_content_element),
        );
        let Some(expected_submenu_content_node) = expected_submenu_content_node else {
            panic!(
                "expected submenu content element to be mounted; open_value={open_value:?} element={expected_submenu_content_element:?}",
            );
        };
        assert!(
            more.controls.contains(&expected_submenu_content_node),
            "expected submenu trigger to advertise a controls relationship to its submenu content; controls={:?} expected={:?}",
            more.controls,
            expected_submenu_content_node
        );

        let other = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Other"))
            .expect("Other menu item");

        // Choose a point near the "Other" item's right edge, so the pointer direction is towards
        // the right-side submenu panel and Radix-style pointer grace intent should apply.
        //
        // Note: the menu content may be wrapped in a render transform (motion). The semantics
        // snapshot bounds are not guaranteed to map 1:1 to interactive hit testing, so locate a
        // real hit-testable point for the menu item.
        let mut safe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(4) {
            for x in (0..=bounds.size.width.0 as i32).step_by(4) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                let Some(hit) = ui.debug_hit_test(pos).hit else {
                    continue;
                };
                if ui.debug_node_path(hit).contains(&other.id) {
                    safe_point = match safe_point {
                        None => Some(pos),
                        Some(prev) => {
                            if pos.x.0 > prev.x.0 {
                                Some(pos)
                            } else {
                                Some(prev)
                            }
                        }
                    };
                }
            }
        }
        let safe_point = safe_point.unwrap_or_else(|| {
            panic!(
                "failed to find hit-testable point for menu item; other={:?} other_bounds={:?} hit@center={:?}",
                other.id,
                other.bounds,
                ui.debug_hit_test(Point::new(
                    Px(other.bounds.origin.x.0 + other.bounds.size.width.0 * 0.5),
                    Px(other.bounds.origin.y.0 + other.bounds.size.height.0 * 0.5),
                )),
            );
        });

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
                pointer_id: fret_core::PointerId(0),
                position: safe_point,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
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

    #[test]
    fn dropdown_menu_submenu_items_propagate_test_ids() {
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
                DropdownMenuEntry::Item(DropdownMenuItem::new("Sub Alpha").test_id("sub-alpha")),
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
        let sub_alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha"))
            .expect("Sub Alpha submenu item");
        assert_eq!(
            sub_alpha.test_id.as_deref(),
            Some("sub-alpha"),
            "expected submenu item test_id to be preserved for deterministic automation"
        );
    }

    #[test]
    fn dropdown_menu_submenu_keyboard_open_transfers_focus_and_arrow_left_restores_focus() {
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
            open.clone(),
            entries.clone(),
        );

        let effects = app.flush_effects();
        let focus_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after <= Duration::from_millis(250) => {
                Some(*token)
            }
            _ => None,
        });
        let Some(focus_timer) = focus_timer else {
            panic!("expected submenu focus timer effect");
        };

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Timer { token: focus_timer },
        );

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
        let sub_alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha"))
            .expect("Sub Alpha submenu item");
        assert_eq!(
            ui.focus(),
            Some(sub_alpha.id),
            "expected keyboard-open to transfer focus into the submenu"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowLeft,
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
        let more_after_close = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More submenu trigger after close");
        assert_eq!(
            ui.focus(),
            Some(more_after_close.id),
            "ArrowLeft should restore focus to the submenu trigger"
        );
    }
}
