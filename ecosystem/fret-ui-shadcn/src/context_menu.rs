use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Edges, Point, Px, Rect, Size, TextStyle};
use fret_icons::ids;
use fret_runtime::{CommandId, Model, ModelId, WindowCommandGatingSnapshot};
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, Elements, FlexProps, InsetStyle, LayoutStyle, Length,
    MainAlign, Overflow, PointerRegionProps, PositionStyle, PressableProps, RingStyle,
    RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps, SizeStyle,
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
use fret_ui_kit::primitives::context_menu as menu;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::{
    ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence, Radius, Space, ui,
};

use crate::dropdown_menu::{DropdownMenuAlign, DropdownMenuSide};
use crate::overlay_motion;
use crate::popper_arrow::{self, DiamondArrowStyle};
use crate::shortcut_display::command_shortcut_label;

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

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContextMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub inset: bool,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
    pub submenu: Option<Vec<ContextMenuEntry>>,
    pub variant: ContextMenuItemVariant,
}

impl ContextMenuItem {
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
            test_id: None,
            trailing: None,
            submenu: None,
            variant: ContextMenuItemVariant::Default,
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

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn submenu(mut self, entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        self.submenu = Some(entries.into_iter().collect());
        self
    }

    pub fn variant(mut self, variant: ContextMenuItemVariant) -> Self {
        self.variant = variant;
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
/// In the upstream DOM implementation, this is a structural wrapper (Radix `Menu.Group`).
/// In Fret, we preserve this structure so it can appear in the semantics tree.
#[derive(Debug, Clone)]
pub struct ContextMenuGroup {
    pub entries: Vec<ContextMenuEntry>,
}

impl ContextMenuGroup {
    pub fn new(entries: impl IntoIterator<Item = ContextMenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
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
        let fg = theme.color_required("muted-foreground");
        let font_size = theme.metric_required("font.size");
        let font_line_height = theme.metric_required("font.line_height");

        ui::text(cx, self.text)
            .layout(LayoutRefinement::default().ml_auto())
            .text_size_px(font_size)
            .line_height_px(font_line_height)
            .font_normal()
            .letter_spacing_em(0.12)
            .nowrap()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

/// shadcn/ui `ContextMenuCheckboxItem` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuCheckboxItem {
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

impl ContextMenuCheckboxItem {
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
    pub leading: Option<AnyElement>,
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

    fn into_item(self, group_value: Model<Option<Arc<str>>>) -> ContextMenuRadioItem {
        ContextMenuRadioItem {
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

/// shadcn/ui `ContextMenuRadioItem` (v4).
#[derive(Debug, Clone)]
pub struct ContextMenuRadioItem {
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

fn estimated_menu_panel_height_for_entries(
    entries: &[ContextMenuEntry],
    row_height: Px,
    max_height: Px,
) -> Px {
    // new-york-v4: menu panels use `p-1` and `border`.
    let panel_padding_y = Px(8.0);
    let panel_border_y = Px(2.0);

    let mut height = Px(panel_padding_y.0 + panel_border_y.0);
    for entry in entries {
        match entry {
            ContextMenuEntry::Separator => {
                // new-york-v4: `Separator` uses `-mx-1 my-1` (1px line + 4px + 4px).
                height.0 += 9.0;
            }
            ContextMenuEntry::Label(_)
            | ContextMenuEntry::Item(_)
            | ContextMenuEntry::CheckboxItem(_)
            | ContextMenuEntry::RadioItem(_) => {
                height.0 += row_height.0.max(0.0);
            }
            ContextMenuEntry::Group(_) | ContextMenuEntry::RadioGroup(_) => {
                unreachable!("entries are flattened")
            }
        }
    }

    let height = height.0.max(0.0);
    Px(height.min(max_height.0.max(0.0)))
}

fn find_submenu_entries_by_value(
    entries: &[ContextMenuEntry],
    open_value: &str,
) -> Option<Vec<ContextMenuEntry>> {
    for entry in entries {
        match entry {
            ContextMenuEntry::Item(item) => {
                if item.value.as_ref() == open_value {
                    return item.submenu.clone();
                }
            }
            ContextMenuEntry::Group(group) => {
                if let Some(found) = find_submenu_entries_by_value(&group.entries, open_value) {
                    return Some(found);
                }
            }
            ContextMenuEntry::CheckboxItem(_)
            | ContextMenuEntry::RadioGroup(_)
            | ContextMenuEntry::RadioItem(_)
            | ContextMenuEntry::Label(_)
            | ContextMenuEntry::Separator => {}
        }
    }
    None
}

fn menu_structural_group<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    role: fret_core::SemanticsRole,
    children: I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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
        move |_cx| children,
    )
}

#[derive(Clone)]
struct ContextMenuRenderEnv {
    open: Model<bool>,
    gating: WindowCommandGatingSnapshot,
    reserve_leading_slot: bool,
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
    submenu_models: menu::sub::MenuSubmenuModels,
}

impl ContextMenuRenderEnv {
    fn render_entries<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        entries: &[ContextMenuEntry],
        item_ix: &mut usize,
    ) -> Elements {
        let mut out: Vec<AnyElement> = Vec::with_capacity(entries.len());

        for entry in entries {
            match entry {
                ContextMenuEntry::Group(group) => {
                    let children = self.render_entries(cx, &group.entries, item_ix);
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::RadioGroup(group) => {
                    let mut children: Vec<AnyElement> = Vec::with_capacity(group.items.len());
                    for spec in group.items.iter().cloned() {
                        children.push(self.render_radio_item(
                            cx,
                            spec.into_item(group.value.clone()),
                            item_ix,
                        ));
                    }
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::Label(label) => out.push(self.render_label(cx, label.clone())),
                ContextMenuEntry::Separator => out.push(self.render_separator(cx)),
                ContextMenuEntry::Item(item) => {
                    out.push(self.render_item(cx, item.clone(), item_ix))
                }
                ContextMenuEntry::CheckboxItem(item) => {
                    out.push(self.render_checkbox_item(cx, item.clone(), item_ix));
                }
                ContextMenuEntry::RadioItem(item) => {
                    out.push(self.render_radio_item(cx, item.clone(), item_ix));
                }
            }
        }

        out.into()
    }

    fn render_label<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        label: ContextMenuLabel,
    ) -> AnyElement {
        let pad_left = if label.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };
        let text = label.text;
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let label_fg = self.label_fg;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;

        cx.container(
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
                vec![
                    ui::text(cx, text)
                        .text_size_px(font_size)
                        .line_height_px(font_line_height)
                        .font_medium()
                        .nowrap()
                        .text_color(ColorRef::Color(label_fg))
                        .into_element(cx),
                ]
            },
        )
    }

    fn render_separator<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let border = self.border;
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(1.0));
                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout
                },
                padding: Edges::all(Px(0.0)),
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    fn render_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();
        let variant = item.variant;
        let pad_left = if item.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };

        let open_for_item = self.open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;
        let destructive_fg = self.destructive_fg;
        let destructive_bg = self.destructive_bg;

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);

                if !disabled {
                    cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                    if close_on_select {
                        cx.pressable_set_bool(&open_for_item, false);
                    }
                }

                let trailing = trailing.clone().or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

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
                    a11y: {
                        let mut a11y = menu::item::menu_item_a11y(a11y_label, None);
                        a11y.test_id = test_id.clone();
                        a11y.with_collection_position(collection_index, item_count)
                    },
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    fg
                };
                if st.hovered || st.pressed || st.focused {
                    if variant == ContextMenuItemVariant::Destructive {
                        row_bg = destructive_bg;
                        row_fg = destructive_fg;
                    } else {
                        row_bg = accent;
                        row_fg = accent_fg;
                    }
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading.clone(),
                    reserve_leading_slot,
                    trailing,
                    false,
                    None,
                    disabled,
                    row_bg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_left,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                );

                (props, children)
            })
        })
    }

    fn render_checkbox_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuCheckboxItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let checked = item.checked.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();

        let open_for_item = self.open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);

                let checked_now = cx.watch_model(&checked).copied().unwrap_or(false);
                if !disabled {
                    menu::checkbox_item::wire_toggle_on_activate(cx, checked.clone());
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open_for_item, false);
                }

                let trailing = trailing.clone().or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

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
                    a11y: menu::item::menu_item_checkbox_a11y(a11y_label, checked_now)
                        .with_collection_position(collection_index, item_count),
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading.clone(),
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(checked_now),
                    disabled,
                    row_bg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                );

                (props, children)
            })
        })
    }

    fn render_radio_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuRadioItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let group_value = item.group_value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();

        let open_for_item = self.open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let submenu_for_item = self.submenu_models.clone();
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            let selected = cx.watch_model(&group_value).cloned().flatten();
            let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);

            cx.pressable_with_id_props(move |cx, st, item_id| {
                menu::sub_content::wire_item(cx, item_id, disabled, &submenu_for_item);

                if !disabled {
                    menu::radio_group::wire_select_on_activate(
                        cx,
                        group_value.clone(),
                        value.clone(),
                    );
                }
                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                if !disabled && close_on_select {
                    cx.pressable_set_bool(&open_for_item, false);
                }

                let trailing = trailing.clone().or_else(|| {
                    command.as_ref().and_then(|cmd| {
                        command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                            .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                    })
                });

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
                    a11y: menu::item::menu_item_radio_a11y(a11y_label.clone(), is_selected)
                        .with_collection_position(collection_index, item_count),
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = fg;
                if st.hovered || st.pressed || st.focused {
                    row_bg = accent;
                    row_fg = accent_fg;
                }

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading.clone(),
                    reserve_leading_slot,
                    trailing,
                    false,
                    Some(is_selected),
                    disabled,
                    row_bg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_x,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                );

                (props, children)
            })
        })
    }
}

#[derive(Clone)]
struct ContextMenuContentRenderEnv {
    open: Model<bool>,
    gating: WindowCommandGatingSnapshot,
    reserve_leading_slot: bool,
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
    window_margin: Px,
    submenu_max_height_metric: Option<Px>,
    overlay_root_name_for_controls: Arc<str>,
    submenu_cfg: menu::sub::MenuSubmenuConfig,
    submenu_models: menu::sub::MenuSubmenuModels,
}

impl ContextMenuContentRenderEnv {
    fn render_entries<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        entries: &[ContextMenuEntry],
        item_ix: &mut usize,
    ) -> Elements {
        let mut out: Vec<AnyElement> = Vec::with_capacity(entries.len());

        for entry in entries {
            match entry {
                ContextMenuEntry::Group(group) => {
                    let children = self.render_entries(cx, &group.entries, item_ix);
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::RadioGroup(group) => {
                    let mut children: Vec<AnyElement> = Vec::with_capacity(group.items.len());
                    for spec in group.items.iter().cloned() {
                        children.push(self.render_radio_item(
                            cx,
                            spec.into_item(group.value.clone()),
                            item_ix,
                        ));
                    }
                    out.push(menu_structural_group(
                        cx,
                        fret_core::SemanticsRole::Group,
                        children,
                    ));
                }
                ContextMenuEntry::Label(label) => out.push(self.render_label(cx, label.clone())),
                ContextMenuEntry::Separator => out.push(self.render_separator(cx)),
                ContextMenuEntry::Item(item) => {
                    out.push(self.render_item(cx, item.clone(), item_ix))
                }
                ContextMenuEntry::CheckboxItem(item) => {
                    out.push(self.render_checkbox_item(cx, item.clone(), item_ix));
                }
                ContextMenuEntry::RadioItem(item) => {
                    out.push(self.render_radio_item(cx, item.clone(), item_ix));
                }
            }
        }

        out.into()
    }

    fn render_label<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        label: ContextMenuLabel,
    ) -> AnyElement {
        let pad_left = if label.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };
        let text = label.text;
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let label_fg = self.label_fg;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;

        cx.container(
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
                vec![
                    ui::text(cx, text)
                        .text_size_px(font_size)
                        .line_height_px(font_line_height)
                        .font_medium()
                        .nowrap()
                        .text_color(ColorRef::Color(label_fg))
                        .into_element(cx),
                ]
            },
        )
    }

    fn render_separator<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let border = self.border;
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Px(Px(1.0));
                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-4.0));
                    layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
                    layout
                },
                padding: Edges::all(Px(0.0)),
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    fn render_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let test_id = item.test_id.clone();
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();
        let has_submenu = item.submenu.is_some();
        let submenu_row_count_for_hint = item.submenu.clone().map(|entries| {
            let mut flat: Vec<ContextMenuEntry> = Vec::new();
            flatten_entries(&mut flat, entries);
            flat.len()
        });
        let variant = item.variant;
        let pad_left = if item.inset {
            self.pad_x_inset
        } else {
            self.pad_x
        };

        let open = self.open.clone();
        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;
        let destructive_fg = self.destructive_fg;
        let destructive_bg = self.destructive_bg;
        let window_margin = self.window_margin;
        let submenu_max_height_metric = self.submenu_max_height_metric;
        let overlay_root_name_for_controls = self.overlay_root_name_for_controls.clone();
        let submenu_cfg = self.submenu_cfg;
        let submenu_for_item = self.submenu_models.clone();

        cx.keyed(value.clone(), move |cx| {
            cx.pressable_with_id_props(move |cx, st, item_id| {
                let geometry_hint = has_submenu.then(|| {
                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);
                    let submenu_max_h = submenu_max_height_metric
                        .map(|h| Px(h.0.min(outer.size.height.0)))
                        .unwrap_or(outer.size.height);
                    let desired = menu::sub::estimated_desired_size_for_row_count(
                        Px(192.0),
                        Px(28.0),
                        submenu_row_count_for_hint.unwrap_or(1),
                        submenu_max_h,
                    );
                    menu::sub_trigger::MenuSubTriggerGeometryHint { outer, desired }
                });
                let is_open_submenu = menu::sub_trigger::wire(
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
                let mut a11y = menu::item::menu_item_a11y_with_controls(
                    a11y_label,
                    has_submenu.then_some(is_open_submenu),
                    controls,
                );
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
                    a11y: a11y.with_collection_position(collection_index, item_count),
                    ..Default::default()
                };

                let mut row_bg = fret_core::Color::TRANSPARENT;
                let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                    destructive_fg
                } else {
                    fg
                };
                if st.hovered || st.pressed || st.focused || is_open_submenu {
                    if variant == ContextMenuItemVariant::Destructive {
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
                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                        })
                    })
                };

                let children = menu_row_children(
                    cx,
                    label.clone(),
                    leading.clone(),
                    reserve_leading_slot,
                    trailing,
                    has_submenu,
                    None,
                    disabled,
                    row_bg,
                    row_fg,
                    text_style.clone(),
                    font_size,
                    font_line_height,
                    pad_left,
                    pad_x,
                    pad_y,
                    radius_sm,
                    text_disabled,
                );

                (props, children)
            })
        })
    }

    fn render_checkbox_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuCheckboxItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let checked = item.checked.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();
        let open = self.open.clone();

        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            let checked_now = cx.watch_model(&checked).copied().unwrap_or(false);
            cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Px(28.0));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: menu::item::menu_item_checkbox_a11y(a11y_label.clone(), checked_now)
                        .with_collection_position(collection_index, item_count),
                    ..Default::default()
                },
                move |cx, st| {
                    let checked_now = cx.watch_model(&checked).copied().unwrap_or(false);

                    if !disabled {
                        menu::checkbox_item::wire_toggle_on_activate(cx, checked.clone());
                    }
                    cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                    if !disabled && close_on_select {
                        cx.pressable_set_bool(&open, false);
                    }

                    let trailing = trailing.clone().or_else(|| {
                        command.as_ref().and_then(|cmd| {
                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                        })
                    });

                    let mut row_bg = fret_core::Color::TRANSPARENT;
                    let mut row_fg = fg;
                    if st.hovered || st.pressed || st.focused {
                        row_bg = accent;
                        row_fg = accent_fg;
                    }

                    menu_row_children(
                        cx,
                        label.clone(),
                        leading.clone(),
                        reserve_leading_slot,
                        trailing,
                        false,
                        Some(checked_now),
                        disabled,
                        row_bg,
                        row_fg,
                        text_style.clone(),
                        font_size,
                        font_line_height,
                        pad_x,
                        pad_x,
                        pad_y,
                        radius_sm,
                        text_disabled,
                    )
                },
            )
        })
    }

    fn render_radio_item<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        item: ContextMenuRadioItem,
        item_ix: &mut usize,
    ) -> AnyElement {
        let collection_index = *item_ix;
        *item_ix = (*item_ix).saturating_add(1);

        let label = item.label.clone();
        let value = item.value.clone();
        let group_value = item.group_value.clone();
        let a11y_label = item.a11y_label.clone().or_else(|| Some(label.clone()));
        let close_on_select = item.close_on_select;
        let command = item.command;
        let disabled = item.disabled
            || crate::command_gating::command_is_disabled_by_gating(
                &*cx.app,
                &self.gating,
                command.as_ref(),
            );
        let leading = item.leading.clone();
        let trailing = item.trailing.clone();
        let open = self.open.clone();

        let ring = self.ring;
        let item_count = self.item_count;
        let reserve_leading_slot = self.reserve_leading_slot;
        let text_style = self.text_style.clone();
        let font_size = self.font_size;
        let font_line_height = self.font_line_height;
        let pad_x = self.pad_x;
        let pad_y = self.pad_y;
        let radius_sm = self.radius_sm;
        let text_disabled = self.text_disabled;
        let fg = self.fg;
        let accent = self.accent;
        let accent_fg = self.accent_fg;

        cx.keyed(value.clone(), move |cx| {
            let selected = cx.watch_model(&group_value).cloned().flatten();
            let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);
            cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Px(28.0));
                        layout
                    },
                    enabled: !disabled,
                    focusable: !disabled,
                    focus_ring: Some(ring),
                    a11y: menu::item::menu_item_radio_a11y(a11y_label.clone(), is_selected)
                        .with_collection_position(collection_index, item_count),
                    ..Default::default()
                },
                move |cx, st| {
                    let selected = cx.watch_model(&group_value).cloned().flatten();
                    let is_selected = menu::radio_group::is_selected(selected.as_ref(), &value);

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

                    let trailing = trailing.clone().or_else(|| {
                        command.as_ref().and_then(|cmd| {
                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                        })
                    });

                    let mut row_bg = fret_core::Color::TRANSPARENT;
                    let mut row_fg = fg;
                    if st.hovered || st.pressed || st.focused {
                        row_bg = accent;
                        row_fg = accent_fg;
                    }

                    menu_row_children(
                        cx,
                        label.clone(),
                        leading.clone(),
                        reserve_leading_slot,
                        trailing,
                        false,
                        Some(is_selected),
                        disabled,
                        row_bg,
                        row_fg,
                        text_style.clone(),
                        font_size,
                        font_line_height,
                        pad_x,
                        pad_x,
                        pad_y,
                        radius_sm,
                        text_disabled,
                    )
                },
            )
        })
    }
}

fn menu_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    leading: Option<AnyElement>,
    reserve_leading_slot: bool,
    trailing: Option<AnyElement>,
    submenu: bool,
    indicator_on: Option<bool>,
    disabled: bool,
    row_bg: fret_core::Color,
    row_fg: fret_core::Color,
    text_style: TextStyle,
    _font_size: Px,
    _font_line_height: Px,
    pad_left: Px,
    pad_x: Px,
    pad_y: Px,
    radius_sm: Px,
    text_disabled: fret_core::Color,
) -> Elements {
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
            let has_leading_slot = leading.is_some() || reserve_leading_slot;
            let mut row: Vec<AnyElement> = Vec::with_capacity(
                usize::from(has_indicator)
                    + usize::from(has_leading_slot)
                    + 1
                    + usize::from(trailing.is_some())
                    + usize::from(submenu),
            );

            if let Some(is_on) = indicator_on {
                let indicator_fg = if disabled { text_disabled } else { row_fg };
                row.push(cx.flex(
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
                    move |cx| {
                        if !is_on {
                            return Vec::new();
                        }

                        vec![decl_icon::icon_with(
                            cx,
                            ids::ui::CHECK,
                            Some(Px(16.0)),
                            Some(ColorRef::Color(indicator_fg)),
                        )]
                    },
                ));
            }

            if let Some(l) = leading.clone() {
                row.push(menu_icon_slot(cx, l));
            } else if reserve_leading_slot {
                row.push(menu_icon_slot_empty(cx));
            }

            let style = text_style.clone();
            let mut text = ui::text(cx, label.clone())
                .layout(LayoutRefinement::default().w_full().min_w_0().flex_1())
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

            if submenu {
                row.push(submenu_chevron_right_icon(
                    cx,
                    if disabled { text_disabled } else { row_fg },
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
    )]
    .into()
}

fn submenu_chevron_right_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    fg: fret_core::Color,
) -> AnyElement {
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

fn context_menu_submenu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_value: Arc<str>,
    placed: Rect,
    entries: Vec<ContextMenuEntry>,
    open: Model<bool>,
    typeahead_timeout_ticks: u64,
    align_leading_icons: bool,
    submenu_models: menu::sub::MenuSubmenuModels,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);

    let entries_tree = entries;
    let mut flat: Vec<ContextMenuEntry> = Vec::new();
    flatten_entries(&mut flat, entries_tree.clone());
    let entries_flat = flat;

    let reserve_leading_slot = align_leading_icons
        && entries_flat.iter().any(|e| match e {
            ContextMenuEntry::Item(item) => item.leading.is_some(),
            ContextMenuEntry::CheckboxItem(item) => item.leading.is_some(),
            ContextMenuEntry::RadioItem(item) => item.leading.is_some(),
            ContextMenuEntry::Label(_)
            | ContextMenuEntry::Group(_)
            | ContextMenuEntry::RadioGroup(_)
            | ContextMenuEntry::Separator => false,
        });

    let item_count = entries_flat
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

    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries_flat
        .iter()
        .filter_map(|e| match e {
            ContextMenuEntry::Item(item) => Some((
                item.label.clone(),
                item.disabled
                    || crate::command_gating::command_is_disabled_by_gating(
                        &*cx.app,
                        &gating,
                        item.command.as_ref(),
                    ),
            )),
            ContextMenuEntry::CheckboxItem(item) => Some((
                item.label.clone(),
                item.disabled
                    || crate::command_gating::command_is_disabled_by_gating(
                        &*cx.app,
                        &gating,
                        item.command.as_ref(),
                    ),
            )),
            ContextMenuEntry::RadioItem(item) => Some((
                item.label.clone(),
                item.disabled
                    || crate::command_gating::command_is_disabled_by_gating(
                        &*cx.app,
                        &gating,
                        item.command.as_ref(),
                    ),
            )),
            ContextMenuEntry::Label(_)
            | ContextMenuEntry::Separator
            | ContextMenuEntry::Group(_)
            | ContextMenuEntry::RadioGroup(_) => None,
        })
        .unzip();

    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

    let border = theme.color_required("border");
    let radius_sm = MetricRef::radius(Radius::Sm).resolve(&theme);
    let panel_chrome = crate::ui_builder_ext::surfaces::menu_sub_style_chrome().rounded(Radius::Md);
    let ring = decl_style::focus_ring(&theme, radius_sm);
    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
    let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
    let font_size = theme.metric_required("font.size");
    let font_line_height = theme.metric_required("font.line_height");
    let text_style = TextStyle {
        font: fret_core::FontId::default(),
        size: font_size,
        weight: fret_core::FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(font_line_height),
        letter_spacing_em: None,
    };
    let text_disabled = alpha_mul(theme.color_required("foreground"), 0.5);
    let label_fg = theme.color_required("muted-foreground");
    let accent = theme.color_required("accent");
    let accent_fg = theme.color_required("accent-foreground");
    let fg = theme.color_required("foreground");
    let destructive_fg = theme.color_required("destructive");
    let destructive_bg = alpha_mul(destructive_fg, 0.12);

    let labelled_by_element = cx
        .app
        .models_mut()
        .read(&submenu_models.trigger, |v| *v)
        .ok()
        .flatten();

    menu::sub_content::submenu_panel_scroll_y_for_value_at(
        cx,
        open_value,
        placed,
        labelled_by_element,
        move |layout| {
            let mut props = decl_style::container_props(
                &theme,
                panel_chrome.clone(),
                LayoutRefinement::default(),
            );
            props.layout = layout;
            props
        },
        move |cx| {
            let render_env = ContextMenuRenderEnv {
                open: open.clone(),
                gating: gating.clone(),
                reserve_leading_slot,
                item_count,
                ring,
                border,
                radius_sm,
                pad_x,
                pad_x_inset,
                pad_y,
                font_size,
                font_line_height,
                text_style: text_style.clone(),
                text_disabled,
                label_fg,
                accent,
                accent_fg,
                fg,
                destructive_fg,
                destructive_bg,
                submenu_models: submenu_models.clone(),
            };
            let entries_tree = entries_tree.clone();

            let mut item_ix: usize = 0;
            let out = render_env.render_entries(cx, &entries_tree, &mut item_ix);

            vec![
                menu::sub_content::submenu_roving_group_apg_prefix_typeahead(
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
                            wrap: false,
                            disabled: disabled_arc.clone(),
                            ..Default::default()
                        },
                    },
                    labels_arc.clone(),
                    typeahead_timeout_ticks,
                    submenu_models.clone(),
                    move |_cx| out.clone(),
                ),
            ]
        },
    )
}

/// shadcn/ui `ContextMenu` root (v4).
///
/// This is a dismissible popover opened by a component-owned pointer policy:
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
    modal: bool,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    min_width: Px,
    submenu_min_width: Px,
    typeahead_timeout_ticks: u64,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    align_leading_icons: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
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
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl ContextMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            modal: true,
            align: DropdownMenuAlign::Start,
            // Match Radix/shadcn defaults:
            // `ContextMenuPrimitive.Content` uses `side="right" sideOffset={2} align="start"`.
            side: DropdownMenuSide::Right,
            side_offset: Px(2.0),
            window_margin: Px(0.0),
            min_width: Px(128.0),
            submenu_min_width: Px(128.0),
            typeahead_timeout_ticks: 30,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            align_leading_icons: true,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
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

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn submenu_min_width(mut self, min_width: Px) -> Self {
        self.submenu_min_width = min_width;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn align_leading_icons(mut self, align: bool) -> Self {
        self.align_leading_icons = align;
        self
    }

    /// Sets an optional open autofocus handler (Radix `onOpenAutoFocus`).
    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    /// Sets an optional close autofocus handler (Radix `onCloseAutoFocus`).
    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
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
        I: IntoIterator<Item = ContextMenuEntry>,
    {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let submenu_max_height_metric = theme.metric_by_key("component.context_menu.max_height");
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
                    .metric_by_key("component.context_menu.arrow_size")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                    .unwrap_or(Px(12.0))
            });
            let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_padding")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                    .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme))
            });

            let id = cx.root_id();
            let overlay_root_name = menu::context_menu_root_name(id);
            let overlay_root_name_for_controls: Arc<str> = Arc::from(overlay_root_name.clone());
            let content_id_for_trigger =
                menu::content_panel::menu_content_semantics_id(cx, &overlay_root_name);
            let trigger_element = trigger(cx);
            let trigger_element = menu::trigger::apply_menu_trigger_a11y(
                trigger_element,
                is_open,
                Some(content_id_for_trigger),
            );
            let trigger_id = trigger_element.id;

            menu::trigger::wire_open_on_shift_f10(cx, trigger_id, self.open.clone());

            let open = self.open;
            let on_dismiss_request = self.on_dismiss_request.clone();
            let on_open_auto_focus = self.on_open_auto_focus.clone();
            let on_close_auto_focus = self.on_close_auto_focus.clone().or_else(|| {
                Some(Arc::new(
                    |_host: &mut dyn fret_ui::action::UiFocusActionHost,
                     _cx: fret_ui::action::ActionCx,
                     req: &mut fret_ui::action::AutoFocusRequestCx| {
                        req.prevent_default();
                    },
                ))
            });
            let open_model_id = open.id();
            let anchor_store_model: Model<HashMap<ModelId, Point>> =
                menu::context_menu_anchor_store_model(cx.app);

            let base_pointer_policy = menu::context_menu_pointer_down_policy(open.clone());
            let pointer_policy = Arc::new({
                let anchor_store_model = anchor_store_model.clone();
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    let handled = base_pointer_policy(host, cx, down);
                    if handled {
                        let _ = host.models_mut().update(&anchor_store_model, |map| {
                            map.insert(open_model_id, down.position);
                        });
                    }
                    handled
                }
            });

            let pointer_policy_for_region = pointer_policy.clone();
            let trigger = cx.keyed((open_model_id, "context-menu-trigger-region"), move |cx| {
                let pointer_policy_for_region = pointer_policy_for_region.clone();
                cx.pointer_region(PointerRegionProps::default(), move |cx| {
                    cx.pointer_region_on_pointer_down(pointer_policy_for_region);
                    vec![trigger_element]
                })
            });

            let anchor_point = cx
                .watch_model(&anchor_store_model)
                .read_ref(|m| m.get(&open_model_id).copied())
                .ok()
                .flatten();
            let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
            let submenu = cx.with_root_name(&overlay_root_name, |cx| {
                menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
            });

            if overlay_presence.present {
                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let min_width = self.min_width;
                let submenu_min_width = self.submenu_min_width;
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let align_leading_icons = self.align_leading_icons;
                let modal = self.modal;
                let open_for_overlay = open.clone();
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();
                let first_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let first_item_focus_id_for_children = first_item_focus_id.clone();
                let first_item_focus_id_for_request = first_item_focus_id.clone();
                let direction = direction_prim::use_direction_in_scope(cx, None);

                let (overlay_children, dismissible_on_pointer_move) =
                    cx.with_root_name(&overlay_root_name, move |cx| {
                    let trigger_bounds =
                        overlay::anchor_bounds_for_element(cx, trigger_id);
                    let anchor = anchor_point.or_else(|| trigger_bounds.map(|r| r.origin));
                    let Some(anchor) = anchor else {
                        return (Vec::new(), None);
                    };

                    let entries_tree: Vec<ContextMenuEntry> = entries(cx).into_iter().collect();
                    let gating = crate::command_gating::snapshot_for_window(&*cx.app, cx.window);
                    let mut flat: Vec<ContextMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, entries_tree.clone());
                    let entries_flat = flat;
                    let reserve_leading_slot = align_leading_icons
                        && entries_flat.iter().any(|e| match e {
                            ContextMenuEntry::Item(item) => item.leading.is_some(),
                            ContextMenuEntry::CheckboxItem(item) => item.leading.is_some(),
                            ContextMenuEntry::RadioItem(item) => item.leading.is_some(),
                            ContextMenuEntry::Label(_)
                            | ContextMenuEntry::Group(_)
                            | ContextMenuEntry::RadioGroup(_)
                            | ContextMenuEntry::Separator => false,
                        });

                    let item_count = entries_flat
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
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries_flat
                        .iter()
                        .filter_map(|e| match e {
                            ContextMenuEntry::Item(item) => Some((
                                item.label.clone(),
                                item.disabled
                                    || crate::command_gating::command_is_disabled_by_gating(
                                        &*cx.app,
                                        &gating,
                                        item.command.as_ref(),
                                    ),
                            )),
                            ContextMenuEntry::CheckboxItem(item) => {
                                Some((
                                    item.label.clone(),
                                    item.disabled
                                        || crate::command_gating::command_is_disabled_by_gating(
                                            &*cx.app,
                                            &gating,
                                            item.command.as_ref(),
                                        ),
                                ))
                            }
                            ContextMenuEntry::RadioItem(item) => Some((
                                item.label.clone(),
                                item.disabled
                                    || crate::command_gating::command_is_disabled_by_gating(
                                        &*cx.app,
                                        &gating,
                                        item.command.as_ref(),
                                    ),
                            )),
                            ContextMenuEntry::Label(_)
                            | ContextMenuEntry::Separator
                            | ContextMenuEntry::Group(_)
                            | ContextMenuEntry::RadioGroup(_) => None,
                        })
                        .unzip();

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

                    let anchor_rect = overlay::anchor_rect_from_point(anchor);
                    let popper_placement =
                        popper::PopperContentPlacement::new(direction, side, align, side_offset)
                            .with_shift_cross_axis(true)
                            .with_arrow(arrow_options, arrow_protrusion);
                    let popper_vars = menu::context_menu_popper_vars(
                        outer,
                        anchor_rect,
                        min_width,
                        popper_placement,
                    );
                    let desired_w =
                        menu::context_menu_popper_desired_width(outer, anchor_rect, min_width);
                    let max_h = theme
                        .metric_by_key("component.context_menu.max_height")
                        .map(|h| Px(h.0.min(popper_vars.available_height.0)))
                        .unwrap_or(popper_vars.available_height);
                    let menu_font_line_height = theme.metric_required("font.line_height");
                    let menu_pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                    let menu_row_height = Px(menu_font_line_height.0 + menu_pad_y.0 * 2.0);
                    let desired_h =
                        estimated_menu_panel_height_for_entries(&entries_flat, menu_row_height, max_h);
                    let desired = Size::new(desired_w, desired_h);

                    let layout = popper::popper_content_layout_sized(
                        outer,
                        anchor_rect,
                        desired,
                        popper_placement,
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let extra_left = wrapper_insets.left;
                    let extra_top = wrapper_insets.top;
                    let origin = popper::popper_content_transform_origin(
                        &layout,
                        anchor_rect,
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
                    let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                    let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                    let font_size = theme.metric_required("font.size");
                    let font_line_height = theme.metric_required("font.line_height");
                    let text_style = TextStyle {
                        font: fret_core::FontId::default(),
                        size: font_size,
                        weight: fret_core::FontWeight::NORMAL,
                        slant: Default::default(),
                        line_height: Some(font_line_height),
                        letter_spacing_em: None,
                    };
                    let text_disabled = alpha_mul(theme.color_required("foreground"), 0.5);
                    let label_fg = theme.color_required("muted-foreground");
                    let accent = theme.color_required("accent");
                    let accent_fg = theme.color_required("accent-foreground");
                    let fg = theme.color_required("foreground");
                    let destructive_fg = theme.color_required("destructive");
                    let destructive_bg = alpha_mul(destructive_fg, 0.12);
                    let panel_bg = theme.color_required("popover.background");
                    let panel_chrome = crate::ui_builder_ext::surfaces::menu_style_chrome();

                    let entries_for_submenu = entries_tree.clone();
                    let entries = entries_tree.clone();
                    let open_for_submenu = open_for_overlay.clone();
                    let submenu_for_content = submenu.clone();
                    let submenu_for_panel = submenu.clone();

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

                    let (_content_id, content) = menu::content_panel::menu_content_semantics_with_id(
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
                                                    bg: panel_bg,
                                                    border: Some(border),
                                                    border_width: Px(1.0),
                                                },
                                            )
                                        })
                                        .flatten();

                                    let panel = menu::content_panel::menu_panel_container_at(
                                        cx,
                                        Rect::new(Point::new(extra_left, extra_top), placed.size),
                                        move |layout| {
                                            let mut props = decl_style::container_props(
                                                &theme,
                                                panel_chrome.clone(),
                                                LayoutRefinement::default(),
                                            );
                                            props.layout = layout;
                                            props
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
                                                        wrap: false,
                                                        disabled: disabled_arc.clone(),
                                                        ..Default::default()
                                                    },
                                                },
                                                 labels_arc.clone(),
                                                 typeahead_timeout_ticks,
                                                move |cx| {
                                                    let render_env = ContextMenuContentRenderEnv {
                                                        open: open_for_overlay.clone(),
                                                        gating: gating.clone(),
                                                        reserve_leading_slot,
                                                        item_count,
                                                        ring,
                                                        border,
                                                        radius_sm,
                                                        pad_x,
                                                        pad_x_inset,
                                                        pad_y,
                                                        font_size,
                                                        font_line_height,
                                                        text_style: text_style.clone(),
                                                        text_disabled,
                                                        label_fg,
                                                        accent,
                                                        accent_fg,
                                                        fg,
                                                        destructive_fg,
                                                        destructive_bg,
                                                        window_margin,
                                                        submenu_max_height_metric,
                                                        overlay_root_name_for_controls:
                                                            overlay_root_name_for_controls.clone(),
                                                        submenu_cfg,
                                                        submenu_models: submenu_for_content.clone(),
                                                    };

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
                                                    ContextMenuEntry::Group(group) => {
                                                        let children = render_env.render_entries(
                                                            cx,
                                                            &group.entries,
                                                            &mut item_ix,
                                                        );
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children,
                                                        ));
                                                    }
                                                    ContextMenuEntry::RadioGroup(group) => {
                                                        let group_value = group.value.clone();
                                                        let mut children: Vec<AnyElement> =
                                                            Vec::with_capacity(group.items.len());
                                                        for spec in group.items {
                                                            children.push(render_env.render_radio_item(
                                                                cx,
                                                                spec.into_item(group_value.clone()),
                                                                &mut item_ix,
                                                            ));
                                                        }
                                                        out.push(menu_structural_group(
                                                            cx,
                                                            fret_core::SemanticsRole::Group,
                                                            children,
                                                        ));
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
                                                                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
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
                                                    ContextMenuEntry::Item(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

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
                                                        let has_submenu = item.submenu.is_some();
                                                        let submenu_entries_for_hint =
                                                            item.submenu.clone().map(|entries| {
                                                                let mut flat: Vec<ContextMenuEntry> =
                                                                    Vec::new();
                                                                flatten_entries(&mut flat, entries);
                                                                flat
                                                            });
                                                        let variant = item.variant;
                                                        let pad_left =
                                                            if item.inset { pad_x_inset } else { pad_x };
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let submenu_for_item = submenu_for_content.clone();
                                                        let overlay_root_name_for_controls =
                                                            overlay_root_name_for_controls.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    let geometry_hint =
                                                                        has_submenu.then(|| {
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
                                                                                    menu_row_height,
                                                                                    submenu_max_h,
                                                                                );
                                                                            let desired = Size::new(
                                                                                submenu_min_width,
                                                                                desired_h,
                                                                            );
                                                                            menu::sub_trigger::MenuSubTriggerGeometryHint {
                                                                                outer,
                                                                                desired,
                                                                            }
                                                                        });
                                                                    let is_open_submenu = menu::sub_trigger::wire(
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
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                    }

                                                                    if !has_submenu && !disabled {
                                                                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(
                                                                                &open, false,
                                                                            );
                                                                        }
                                                                    }

                                                                    let controls = has_submenu.then(|| {
                                                                        menu::sub_content::submenu_content_semantics_id(
                                                                            cx,
                                                                            overlay_root_name_for_controls
                                                                                .as_ref(),
                                                                            &value,
                                                                        )
                                                                    });
                                                                    let mut a11y =
                                                                        menu::item::menu_item_a11y_with_controls(
                                                                            a11y_label,
                                                                            has_submenu.then_some(
                                                                                is_open_submenu,
                                                                            ),
                                                                            controls,
                                                                        );
                                                                    a11y.test_id = test_id.clone();
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
                                                                        a11y: a11y.with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = if variant == ContextMenuItemVariant::Destructive {
                                                                        destructive_fg
                                                                    } else {
                                                                        fg
                                                                    };
                                                                    if st.hovered
                                                                        || st.pressed
                                                                        || st.focused
                                                                        || is_open_submenu
                                                                    {
                                                                        if variant == ContextMenuItemVariant::Destructive {
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
                                                                                    .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                                                                            })
                                                                        })
                                                                    };

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        has_submenu,
                                                                        None,
                                                                        disabled,
                                                                    row_bg,
                                                                    row_fg,
                                                                    text_style.clone(),
                                                                    font_size,
                                                                    font_line_height,
                                                                    pad_left,
                                                                    pad_x,
                                                                    pad_y,
                                                                    radius_sm,
                                                                    text_disabled,
                                                                    );

                                                                    (props, children)
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
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    let checked_now = cx
                                                                        .watch_model(&checked)
                                                                        .copied()
                                                                        .unwrap_or(false);

                                                                    if !disabled {
                                                                        if first_item_focus_id_for_items
                                                                            .get()
                                                                            .is_none()
                                                                        {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        menu::checkbox_item::wire_toggle_on_activate(
                                                                            cx,
                                                                            checked.clone(),
                                                                        );
                                                                    }

                                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                                        command.clone(),
                                                                    );
                                                                    if !disabled && close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                                                                        })
                                                                    });

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered || st.pressed || st.focused {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        false,
                                                                        Some(checked_now),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    );

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
                                                                            a11y_label.clone(),
                                                                            checked_now,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    (props, children)
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
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();
                                                        let first_item_focus_id_for_items =
                                                            first_item_focus_id_for_children.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            let selected = cx
                                                                .watch_model(&group_value)
                                                                .cloned()
                                                                .flatten();
                                                            let is_selected = menu::radio_group::is_selected(
                                                                selected.as_ref(),
                                                                &value,
                                                            );
                                                            cx.pressable_with_id_props(
                                                                move |cx, st, item_id| {
                                                                    if !disabled {
                                                                        if first_item_focus_id_for_items
                                                                            .get()
                                                                            .is_none()
                                                                        {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        menu::radio_group::wire_select_on_activate(
                                                                            cx,
                                                                            group_value.clone(),
                                                                            value.clone(),
                                                                        );
                                                                    }

                                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                                        command.clone(),
                                                                    );
                                                                    if !disabled && close_on_select {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(cx, cmd, fret_runtime::Platform::current())
                                                                                .map(|text| ContextMenuShortcut::new(text).into_element(cx))
                                                                        })
                                                                    });

                                                                    let mut row_bg =
                                                                        fret_core::Color::TRANSPARENT;
                                                                    let mut row_fg = fg;
                                                                    if st.hovered || st.pressed || st.focused {
                                                                        row_bg = accent;
                                                                        row_fg = accent_fg;
                                                                    }

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        false,
                                                                        Some(is_selected),
                                                                        disabled,
                                                                        row_bg,
                                                                        row_fg,
                                                                        text_style.clone(),
                                                                        font_size,
                                                                        font_line_height,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_disabled,
                                                                    );

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
                                                                            a11y_label.clone(),
                                                                            is_selected,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    (props, children)
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
                                                move |_cx| vec![roving],
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

                    let content =
                        overlay_motion::wrap_opacity_and_render_transform(cx, opacity, transform, vec![content]);

                    let dismissible_on_pointer_move =
                        menu::root::submenu_pointer_move_handler(submenu.clone(), submenu_cfg);

                    let mut children = vec![content];
                    let submenu_open_value = cx
                        .app
                        .models_mut()
                        .read(&submenu_for_panel.open_value, |v| v.clone())
                        .ok()
                        .flatten();
                    let desired = submenu_open_value
                        .as_ref()
                        .and_then(|open_value| {
                            find_submenu_entries_by_value(&entries_for_submenu, open_value.as_ref())
                        })
                        .map(|submenu_entries| {
                            let mut flat: Vec<ContextMenuEntry> = Vec::new();
                            flatten_entries(&mut flat, submenu_entries);
                            let submenu_max_h = submenu_max_height_metric
                                .map(|h| Px(h.0.min(outer.size.height.0)))
                                .unwrap_or(outer.size.height);
                            let desired_h = estimated_menu_panel_height_for_entries(
                                &flat,
                                menu_row_height,
                                submenu_max_h,
                            );
                            Size::new(submenu_min_width, desired_h)
                        })
                        .unwrap_or_else(|| {
                            let submenu_max_h = submenu_max_height_metric
                                .map(|h| Px(h.0.min(outer.size.height.0)))
                                .unwrap_or(outer.size.height);
                            Size::new(submenu_min_width, submenu_max_h)
                        });
                    let submenu_is_open = submenu_open_value.is_some();
                    let submenu_motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                        cx,
                        submenu_is_open,
                        overlay_motion::SHADCN_MOTION_TICKS_100,
                        0,
                        0.95,
                        1.0,
                        overlay_motion::shadcn_ease,
                    );
                    let submenu_opacity = submenu_motion.opacity;
                    let submenu_scale = submenu_motion.scale;

                    let open_submenu = menu::sub::with_open_submenu_synced(
                        cx,
                        &submenu_for_panel,
                        outer,
                        desired,
                        |_cx, open_value, geometry| (open_value, geometry),
                    );

                    #[derive(Default)]
                    struct SubmenuLast {
                        open_value: Option<Arc<str>>,
                        geometry: Option<menu::sub::MenuSubmenuGeometry>,
                    }

                    let (last_value, last_geometry) = cx.with_state(SubmenuLast::default, |st| {
                        if let Some((open_value, geometry)) = open_submenu.as_ref() {
                            st.open_value = Some(open_value.clone());
                            st.geometry = Some(*geometry);
                        }
                        (st.open_value.clone(), st.geometry)
                    });

                    if submenu_motion.present {
                        let open_value = open_submenu
                            .as_ref()
                            .map(|(open_value, _)| open_value.clone())
                            .or(last_value);
                        let geometry = open_submenu
                            .map(|(_, geometry)| geometry)
                            .or(last_geometry);

                        let (Some(open_value), Some(geometry)) = (open_value, geometry) else {
                            return (children, Some(dismissible_on_pointer_move));
                        };

                        if let Some(submenu_entries) =
                            find_submenu_entries_by_value(&entries_for_submenu, open_value.as_ref())
                        {
                            let submenu_panel = context_menu_submenu_panel(
                                cx,
                                open_value.clone(),
                                geometry.floating,
                                submenu_entries,
                                open_for_submenu.clone(),
                                typeahead_timeout_ticks,
                                align_leading_icons,
                                submenu_for_panel.clone(),
                            );

                            let side =
                                overlay_motion::anchored_side(geometry.reference, geometry.floating);
                            let origin = overlay_motion::shadcn_transform_origin_for_anchored_rect(
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

                            let opacity = submenu_opacity;
                            let submenu_panel = cx.interactivity_gate(
                                submenu_motion.present,
                                submenu_is_open,
                                move |cx| {
                                    vec![overlay_motion::wrap_opacity_and_render_transform(
                                        cx,
                                        opacity,
                                        transform,
                                        vec![submenu_panel],
                                    )]
                                },
                            );
                            children.push(submenu_panel);
                        }
                    }

                    (children, Some(dismissible_on_pointer_move))
                });

                let request = menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
                    cx,
                    id,
                    trigger_id,
                    open,
                    overlay_presence,
                    overlay_children,
                    overlay_root_name,
                    menu::root::MenuInitialFocusTargets::new()
                        .pointer_content_focus(content_focus_id.get())
                        .keyboard_entry_focus(first_item_focus_id_for_request.get()),
                    on_open_auto_focus.clone(),
                    on_close_auto_focus.clone(),
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

    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fret_app::App;
    use fret_core::UiServices;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    };
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::FrameId;
    use fret_ui::element::PressableA11y;
    use fret_ui::tree::UiTree;

    #[test]
    fn estimated_menu_panel_height_clamps_to_max_height() {
        let entries: Vec<ContextMenuEntry> = (0..100)
            .map(|i| {
                ContextMenuEntry::Item(
                    ContextMenuItem::new(format!("Item {i}")).on_select(CommandId::new("noop")),
                )
            })
            .collect();
        let entries = entries.as_slice();

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height =
            super::estimated_menu_panel_height_for_entries(entries, row_height, max_height);
        assert_eq!(height, max_height);
    }

    #[test]
    fn estimated_menu_panel_height_shrinks_for_short_menus() {
        let entries = vec![
            ContextMenuEntry::Item(ContextMenuItem::new("Apple").on_select(CommandId::new("noop"))),
            ContextMenuEntry::Item(
                ContextMenuItem::new("Orange").on_select(CommandId::new("noop")),
            ),
        ];
        let entries = entries.as_slice();

        let row_height = Px(20.0);
        let max_height = Px(120.0);
        let height =
            super::estimated_menu_panel_height_for_entries(entries, row_height, max_height);
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

    fn render_frame_with_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
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
            move |cx| {
                vec![
                    ContextMenu::new(open)
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
                            |_cx| {
                                vec![
                                    ContextMenuEntry::Item(ContextMenuItem::new("Alpha")),
                                    ContextMenuEntry::Separator,
                                    ContextMenuEntry::Item(ContextMenuItem::new("Beta")),
                                    ContextMenuEntry::Item(ContextMenuItem::new("Gamma")),
                                ]
                            },
                        ),
                ]
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
    fn context_menu_modal_outside_press_can_be_prevented_via_dismiss_handler() {
        use fret_core::MouseButton;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let open = app.models_mut().insert(false);

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
        );

        let outside = Point::new(Px(390.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
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
            &Event::Pointer(fret_core::PointerEvent::Up {
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

    fn render_frame_focusable_trigger_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
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
            "context-menu-underlay",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0));
                            layout.inset.right = Some(Px(0.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.inset.bottom = Some(Px(0.0));
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );

                let trigger = ContextMenu::new(open).into_element(
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
                );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        modal: bool,
        underlay_clicked: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
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
            "context-menu-underlay-modal",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0));
                            layout.inset.right = Some(Px(0.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.inset.bottom = Some(Px(0.0));
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );

                let trigger = ContextMenu::new(open)
                    .modal(modal)
                    .on_dismiss_request(on_dismiss_request.clone())
                    .into_element(
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
                    );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
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
            "context-menu-underlay-entries",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0));
                            layout.inset.right = Some(Px(0.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.inset.bottom = Some(Px(0.0));
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );

                let trigger = ContextMenu::new(open).into_element(
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
                    move |_cx| entries.clone(),
                );

                // Keep the context-menu trigger above the underlay so the right-click open gesture
                // cannot be intercepted by the "underlay" pressable.
                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
        on_dismiss_request: Option<OnDismissRequest>,
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
            "context-menu-underlay-entries-dismiss",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0));
                            layout.inset.right = Some(Px(0.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.inset.bottom = Some(Px(0.0));
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_toggle_bool(&underlay_clicked);
                        Vec::new()
                    },
                );

                let trigger = ContextMenu::new(open)
                    .on_dismiss_request(on_dismiss_request)
                    .into_element(
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
                        move |_cx| entries.clone(),
                    );

                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        underlay_clicked: Model<bool>,
        entries: Vec<ContextMenuEntry>,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
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
            "context-menu-underlay-entries-autofocus",
            move |cx| {
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.left = Some(Px(0.0));
                            layout.inset.right = Some(Px(0.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.inset.bottom = Some(Px(0.0));
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("Underlay")),
                            test_id: Some(Arc::from("underlay")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    {
                        let underlay_id_out = underlay_id_out.clone();
                        move |cx, _st, id| {
                            underlay_id_out.set(Some(id));
                            cx.pressable_toggle_bool(&underlay_clicked);
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        }
                    },
                );

                let trigger =
                    ContextMenu::new(open)
                        .on_open_auto_focus(on_open_auto_focus.clone())
                        .on_close_auto_focus(on_close_auto_focus.clone())
                        .into_element(
                            cx,
                            {
                                let trigger_id_out = trigger_id_out.clone();
                                move |cx| {
                                    cx.pressable_with_id(
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
                                        move |cx, _st, id| {
                                            trigger_id_out.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    )
                                }
                            },
                            move |_cx| entries.clone(),
                        );

                vec![trigger, underlay]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_focusable_trigger_with_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        entries: Vec<ContextMenuEntry>,
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
            "context-menu-submenu-arrow-right",
            move |cx| {
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
        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha menu item");
        assert_eq!(ui.focus(), Some(alpha.id));
    }

    #[test]
    fn context_menu_focus_outside_can_be_prevented_via_dismiss_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let reason_cell: Arc<Mutex<Option<fret_ui::action::DismissReason>>> =
            Arc::new(Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            if matches!(req.reason, fret_ui::action::DismissReason::FocusOutside) {
                let mut lock = reason_cell_for_handler.lock().unwrap();
                *lock = Some(req.reason);
                req.prevent_default();
            }
        });

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let root = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries,
            Some(handler.clone()),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let underlay_node = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let position = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let _ = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries,
            Some(handler.clone()),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.set_focus(Some(underlay_node));

        let entries = vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))];
        let _ = render_frame_focusable_trigger_with_underlay_and_entries_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            entries,
            Some(handler),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(true),
            "expected menu to remain open when focus-outside dismissal is prevented"
        );
        assert_eq!(
            *reason_cell.lock().unwrap(),
            Some(fret_ui::action::DismissReason::FocusOutside)
        );
    }

    #[test]
    fn context_menu_keyboard_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out,
            Some(handler.clone()),
            None,
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

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

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out,
            Rc::new(Cell::new(None)),
            Some(handler),
            None,
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected preventDefault open autofocus to keep focus on trigger"
        );
    }

    #[test]
    fn context_menu_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let underlay_id_out_for_handler = underlay_id_out.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            if let Some(underlay) = underlay_id_out_for_handler.get() {
                host.request_focus(underlay);
            }
            req.prevent_default();
        });

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            None,
            Some(handler.clone()),
        );

        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        let underlay_id = underlay_id_out.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        ui.set_focus(Some(trigger_node));

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

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out.clone(),
            underlay_id_out.clone(),
            None,
            Some(handler.clone()),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _root = render_frame_focusable_trigger_with_underlay_and_entries_and_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
            trigger_id_out,
            underlay_id_out,
            None,
            Some(handler),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected preventDefault close autofocus to allow redirecting focus"
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
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
    fn context_menu_modal_outside_press_closes_without_activating_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close the menu, but must not activate/focus underlay.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
        assert_ne!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn context_menu_click_through_outside_press_closes_and_focuses_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            None,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=false => click-through).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is not installed.
        let _ = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            None,
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected click-through context menu to not install pointer occlusion"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close via outside-press observer and remain click-through.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to move to underlay after click-through dismissal"
        );
    }

    #[test]
    fn context_menu_click_through_outside_press_can_be_prevented_and_still_activates_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            Some(handler.clone()),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=false => click-through).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open.
        let _ = render_frame_focusable_trigger_with_underlay_modal_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_clicked.clone(),
            Some(handler),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Click the underlay: click-through should still activate the underlay, but the menu
        // should remain open since dismissal was prevented.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn context_menu_close_transition_is_click_through_and_drops_pointer_occlusion() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open context menu overlay");
        let overlay_root_name = menu::context_menu_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let underlay_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        // Click the underlay: should close the menu, but must not activate/focus underlay.
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
        assert_ne!(ui.focus(), Some(underlay_node));

        // Frame 3: close transition should drop pointer occlusion and become click-through.
        let _ = render_frame_focusable_trigger_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha")),
            "expected menu content to remain present during close transition"
        );

        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected close transition to drop pointer occlusion (click-through)"
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        // Click again while the menu is still present: must activate/focus the underlay now.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn context_menu_close_transition_does_not_drive_submenu_timers() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(280.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![
            ContextMenuEntry::Item(ContextMenuItem::new("More").submenu(vec![
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
            ])),
            ContextMenuEntry::Item(ContextMenuItem::new("Other")),
        ];

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries.clone(),
        );
        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        let trigger_bounds = ui.debug_node_bounds(trigger).expect("trigger bounds");
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open and locate the submenu trigger.
        let _ = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            entries.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More menu item");
        let more_center = Point::new(
            Px(more.bounds.origin.x.0 + more.bounds.size.width.0 / 2.0),
            Px(more.bounds.origin.y.0 + more.bounds.size.height.0 / 2.0),
        );

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open context menu overlay");
        let overlay_root_name = menu::context_menu_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let overlay_layer = ui.node_layer(overlay_node).expect("overlay layer");

        // Close via outside click to enter the close transition (present=true, interactive=false).
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
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
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Frame 3: close transition should be click-through and must not drive hover intent/timers.
        let _ = render_frame_focusable_trigger_with_underlay_and_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked,
            entries,
        );
        let _ = app.flush_effects();

        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::None,
            "expected close transition to be click-through"
        );

        let info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == overlay_layer)
            .expect("overlay layer info");
        assert!(info.visible);
        assert!(!info.hit_testable);
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_timer_events);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: more_center,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        let cfg = menu::sub::MenuSubmenuConfig::default();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == cfg.open_delay)),
            "expected close transition pointer move to not arm open-delay timer; effects={effects:?} pos={more_center:?} open_delay={:?}",
            cfg.open_delay
        );
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == cfg.close_delay)),
            "expected close transition pointer move to not arm close-delay timer; effects={effects:?} pos={more_center:?} close_delay={:?}",
            cfg.close_delay
        );
    }

    #[test]
    fn context_menu_submenu_safe_hover_corridor_cancels_close_timer_under_pointer_occlusion() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(280.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![
            ContextMenuEntry::Item(ContextMenuItem::new("More").submenu(vec![
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
            ])),
            ContextMenuEntry::Item(ContextMenuItem::new("Other")),
        ];

        // Frame 1: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger_with_entries(
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
        let trigger_pos = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 / 2.0),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 / 2.0),
        );

        // Right-click to open the context menu (modal=true by default).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_pos,
                button: fret_core::MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: open, ensure occlusion is active.
        let _root = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));
        let occlusion = fret_ui_kit::OverlayController::arbitration_snapshot(&ui).pointer_occlusion;
        assert_eq!(
            occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
            "expected modal context menu to install pointer occlusion"
        );

        // Hover "More" to arm the submenu open-delay timer.
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
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(
                    Px(more_bounds.origin.x.0 + more_bounds.size.width.0 / 2.0),
                    Px(more_bounds.origin.y.0 + more_bounds.size.height.0 / 2.0),
                ),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let open_delay = menu::sub::MenuSubmenuConfig::default().open_delay;
        let open_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == open_delay => Some(*token),
            _ => None,
        });
        let Some(open_timer) = open_timer else {
            panic!("expected submenu open-delay timer effect; effects={effects:?}");
        };
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token: open_timer });

        // Frame 3: after open timer fires, the submenu opens.
        let _root = render_frame_focusable_trigger_with_entries(
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
            .expect("Sub Alpha menu item");
        let sub_beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Beta"))
            .expect("Sub Beta menu item");

        let submenu_bounds = Rect::new(
            Point::new(
                Px(sub_alpha.bounds.origin.x.0.min(sub_beta.bounds.origin.x.0)),
                Px(sub_alpha.bounds.origin.y.0.min(sub_beta.bounds.origin.y.0)),
            ),
            Size::new(
                Px(
                    (sub_alpha.bounds.origin.x.0 + sub_alpha.bounds.size.width.0)
                        .max(sub_beta.bounds.origin.x.0 + sub_beta.bounds.size.width.0)
                        - sub_alpha.bounds.origin.x.0.min(sub_beta.bounds.origin.x.0),
                ),
                Px(
                    (sub_alpha.bounds.origin.y.0 + sub_alpha.bounds.size.height.0)
                        .max(sub_beta.bounds.origin.y.0 + sub_beta.bounds.size.height.0)
                        - sub_alpha.bounds.origin.y.0.min(sub_beta.bounds.origin.y.0),
                ),
            ),
        );

        let cfg = menu::sub::MenuSubmenuConfig::default();
        let close_delay = cfg.close_delay;
        let grace_geometry = menu::pointer_grace_intent::PointerGraceIntentGeometry {
            reference: more_bounds,
            floating: submenu_bounds,
        };

        // Pick a safe corridor point on the submenu side (to the right) so moving towards it can
        // cancel a pending close timer (Radix pointer-grace intent).
        let reference_right = more_bounds.origin.x.0 + more_bounds.size.width.0;
        let mut safe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(2) {
            for x in (0..=bounds.size.width.0 as i32).step_by(2) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                if pos.x.0 <= reference_right {
                    continue;
                }
                if more_bounds.contains(pos) || submenu_bounds.contains(pos) {
                    continue;
                }
                if !menu::pointer_grace_intent::last_pointer_is_safe(
                    pos,
                    grace_geometry,
                    cfg.safe_hover_buffer,
                ) {
                    continue;
                }
                safe_point = Some(pos);
                break;
            }
            if safe_point.is_some() {
                break;
            }
        }
        let safe_point = safe_point.unwrap_or_else(|| {
            panic!(
                "failed to find safe corridor point; more={more_bounds:?} submenu={submenu_bounds:?} geometry={grace_geometry:?}"
            )
        });

        // Pick an unsafe point to the left of the safe point, so moving to `safe_point` is
        // directionally towards the submenu (x increases).
        let mut unsafe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(4) {
            for x in (0..=bounds.size.width.0 as i32).step_by(4) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                if pos.x.0 >= safe_point.x.0 {
                    continue;
                }
                if more_bounds.contains(pos) || submenu_bounds.contains(pos) {
                    continue;
                }
                if menu::pointer_grace_intent::last_pointer_is_safe(
                    pos,
                    grace_geometry,
                    cfg.safe_hover_buffer,
                ) {
                    continue;
                }
                unsafe_point = Some(pos);
                break;
            }
            if unsafe_point.is_some() {
                break;
            }
        }
        let unsafe_point = unsafe_point.unwrap_or_else(|| {
            panic!(
                "failed to find unsafe point; safe_point={safe_point:?} more={more_bounds:?} submenu={submenu_bounds:?} geometry={grace_geometry:?}",
            )
        });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: unsafe_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        let close_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == close_delay => Some(*token),
            _ => None,
        });
        let Some(close_timer) = close_timer else {
            panic!(
                "expected unsafe pointer move to arm close-delay timer; effects={effects:?} unsafe_point={unsafe_point:?} close_delay={close_delay:?}"
            );
        };

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: safe_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::CancelTimer { token } if *token == close_timer)),
            "expected safe corridor pointer move to cancel close-delay timer; effects={effects:?} safe_point={safe_point:?} close_timer={close_timer:?}"
        );

        // Sanity: no new close timer should be armed when safe.
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == close_delay)),
            "expected safe corridor pointer move to not arm a new close-delay timer; effects={effects:?} safe_point={safe_point:?} close_delay={close_delay:?}"
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

    #[test]
    fn context_menu_submenu_opens_on_arrow_right_without_pointer_move() {
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

        let entries = vec![ContextMenuEntry::Item(
            ContextMenuItem::new("More").submenu(vec![
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Alpha")),
                ContextMenuEntry::Item(ContextMenuItem::new("Sub Beta")),
            ]),
        )];

        let _ = render_frame_focusable_trigger_with_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        let _ = render_frame_focusable_trigger_with_entries(
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

        let _ = render_frame_focusable_trigger_with_entries(
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
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha")
            }),
            "submenu items should render after ArrowRight opens the submenu"
        );
    }
}
