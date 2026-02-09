use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, Rect, SemanticsRole, Size, TextStyle,
};
use fret_icons::ids;
use fret_runtime::{CommandId, Model, WindowCommandGatingSnapshot};
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, Elements, FlexProps, InsetStyle, LayoutStyle, Length,
    MainAlign, Overflow, PositionStyle, PressableA11y, PressableProps, RovingFlexProps,
    RovingFocusProps, ScrollAxis, ScrollProps, SemanticsProps, SizeStyle,
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
use fret_ui_kit::primitives::menubar as menu;
use fret_ui_kit::primitives::menubar::trigger_row as menubar_trigger_row;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    Radius, Space, WidgetState, WidgetStateProperty, WidgetStates, ui,
};

use crate::overlay_motion;
use crate::shortcut_display::command_shortcut_label;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn is_dark_background(theme: &Theme) -> bool {
    let bg = theme.color_required("background");
    let luma = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
    luma < 0.5
}

fn estimate_text_width(text: &str, font_size: Px) -> Px {
    // Heuristic: our menu panel sizing needs a deterministic width before layout, but the
    // declarative menu rows use flex-1 labels. We approximate the max-content width from the text
    // length, matching shadcn's "min-width but grow to fit" behavior closely enough for goldens.
    let avg = font_size.0 * 0.537;
    Px(text.chars().count() as f32 * avg)
}

fn menu_panel_desired_size(
    entries: &[MenubarEntry],
    min_width: Px,
    font_size: Px,
    font_line_height: Px,
    pad_x: Px,
    pad_x_inset: Px,
    pad_y: Px,
) -> Size {
    let item_line = font_line_height.0.max(16.0);
    let row_height = Px(item_line + pad_y.0 * 2.0);

    // new-york-v4: menu panels use `p-1` + `border`.
    let mut height = Px(10.0);
    let mut max_row_width = 0.0f32;

    for entry in entries {
        match entry {
            MenubarEntry::Separator => {
                // new-york-v4: `Separator` uses `-mx-1 my-1` (1px line + 4px + 4px).
                height.0 += 9.0;
            }
            MenubarEntry::Label(label) => {
                height.0 += row_height.0;
                let pad_left = if label.inset { pad_x_inset } else { pad_x };
                let row_w = pad_left.0 + estimate_text_width(&label.text, font_size).0 + pad_x.0;
                max_row_width = max_row_width.max(row_w);
            }
            MenubarEntry::Item(item) => {
                height.0 += row_height.0;
                let pad_left = if item.inset { pad_x_inset } else { pad_x };
                let row_w = pad_left.0 + estimate_text_width(&item.label, font_size).0 + pad_x.0;
                max_row_width = max_row_width.max(row_w);
            }
            MenubarEntry::CheckboxItem(item) => {
                height.0 += row_height.0;
                let pad_left = pad_x_inset;
                let row_w = pad_left.0 + estimate_text_width(&item.label, font_size).0 + pad_x.0;
                max_row_width = max_row_width.max(row_w);
            }
            MenubarEntry::RadioItem(item) => {
                height.0 += row_height.0;
                let pad_left = pad_x_inset;
                let row_w = pad_left.0 + estimate_text_width(&item.label, font_size).0 + pad_x.0;
                max_row_width = max_row_width.max(row_w);
            }
            MenubarEntry::Submenu(submenu) => {
                height.0 += row_height.0;
                let pad_left = if submenu.trigger.inset {
                    pad_x_inset
                } else {
                    pad_x
                };
                let chevron_w = 16.0;
                let row_w = pad_left.0
                    + estimate_text_width(&submenu.trigger.label, font_size).0
                    + pad_x.0
                    + chevron_w;
                max_row_width = max_row_width.max(row_w);
            }
            MenubarEntry::Group(_) | MenubarEntry::RadioGroup(_) => {}
        }
    }

    // new-york-v4: content uses `p-1` + `border` with border-box sizing.
    let chrome_x = 10.0;
    let width = Px(min_width.0.max(max_row_width + chrome_x));
    Size::new(width, height)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenubarItemVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct MenubarItem {
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
    pub variant: MenubarItemVariant,
}

impl MenubarItem {
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
            variant: MenubarItemVariant::Default,
        }
    }

    pub fn submenu(self, entries: impl IntoIterator<Item = MenubarEntry>) -> MenubarSubmenu {
        MenubarSubmenu::new(self, entries)
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: MenubarItemVariant) -> Self {
        self.variant = variant;
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

#[derive(Debug, Clone)]
pub struct MenubarSubmenu {
    pub trigger: MenubarItem,
    pub entries: Arc<[MenubarEntry]>,
}

impl MenubarSubmenu {
    pub fn new(trigger: MenubarItem, entries: impl IntoIterator<Item = MenubarEntry>) -> Self {
        let entries: Vec<MenubarEntry> = entries.into_iter().collect();
        Self {
            trigger,
            entries: Arc::from(entries.into_boxed_slice()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenubarEntry {
    Item(MenubarItem),
    CheckboxItem(MenubarCheckboxItem),
    RadioGroup(MenubarRadioGroup),
    RadioItem(MenubarRadioItem),
    Label(MenubarLabel),
    Group(MenubarGroup),
    Submenu(MenubarSubmenu),
    Separator,
}

/// shadcn/ui `MenubarLabel` (v4).
#[derive(Debug, Clone)]
pub struct MenubarLabel {
    pub text: Arc<str>,
    pub inset: bool,
}

impl MenubarLabel {
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

/// shadcn/ui `MenubarGroup` (v4).
///
/// In the upstream DOM implementation, this is a structural wrapper. In Fret, we currently treat
/// it as a transparent grouping node and simply flatten its entries for rendering/navigation.
#[derive(Debug, Clone)]
pub struct MenubarGroup {
    pub entries: Vec<MenubarEntry>,
}

impl MenubarGroup {
    pub fn new(entries: impl IntoIterator<Item = MenubarEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

/// shadcn/ui `MenubarShortcut` (v4).
///
/// This is typically rendered as trailing, muted text inside a menu item.
#[derive(Debug, Clone)]
pub struct MenubarShortcut {
    pub text: Arc<str>,
}

impl MenubarShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("muted-foreground");
        let font_size = theme.metric_required("font.size");
        let font_line_height = theme.metric_required("font.line_height");

        ui::text(cx, self.text)
            // new-york-v4: `ml-auto` to push shortcut to the trailing edge.
            .ml_auto()
            .text_size_px(font_size)
            .line_height_px(font_line_height)
            .font_normal()
            .letter_spacing_em(0.12)
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx)
    }
}

/// shadcn/ui `MenubarCheckboxItem` (v4).
#[derive(Debug, Clone)]
pub struct MenubarCheckboxItem {
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

impl MenubarCheckboxItem {
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

/// shadcn/ui `MenubarRadioGroup` (v4).
#[derive(Debug, Clone)]
pub struct MenubarRadioGroup {
    pub value: Model<Option<Arc<str>>>,
    pub items: Vec<MenubarRadioItemSpec>,
}

impl MenubarRadioGroup {
    pub fn new(value: Model<Option<Arc<str>>>) -> Self {
        Self {
            value,
            items: Vec::new(),
        }
    }

    pub fn item(mut self, item: MenubarRadioItemSpec) -> Self {
        self.items.push(item);
        self
    }
}

#[derive(Debug, Clone)]
pub struct MenubarRadioItemSpec {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub leading: Option<AnyElement>,
    pub disabled: bool,
    pub close_on_select: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
}

impl MenubarRadioItemSpec {
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

    fn into_item(self, group_value: Model<Option<Arc<str>>>) -> MenubarRadioItem {
        MenubarRadioItem {
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

/// shadcn/ui `MenubarRadioItem` (v4).
#[derive(Debug, Clone)]
pub struct MenubarRadioItem {
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

impl MenubarRadioItem {
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

fn flatten_entries(into: &mut Vec<MenubarEntry>, entries: Vec<MenubarEntry>) {
    for entry in entries {
        match entry {
            MenubarEntry::Group(group) => flatten_entries(into, group.entries),
            MenubarEntry::RadioGroup(group) => {
                for item in group.items {
                    into.push(MenubarEntry::RadioItem(item.into_item(group.value.clone())));
                }
            }
            other => into.push(other),
        }
    }
}

fn menu_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    leading: Option<AnyElement>,
    reserve_leading_slot: bool,
    trailing: Option<AnyElement>,
    indicator_on: Option<bool>,
    has_submenu: bool,
    bg: Color,
    fg: Color,
    pad_left: Px,
    pad_x: Px,
    pad_y: Px,
    radius_sm: Px,
    text_style: TextStyle,
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
            background: Some(bg),
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(radius_sm),
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
                    + usize::from(has_submenu),
            );

            if let Some(is_on) = indicator_on {
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
                            Some(ColorRef::Color(fg)),
                        )]
                    },
                ));
            }

            if let Some(l) = leading.clone() {
                row.push(menu_icon_slot(cx, l));
            } else if reserve_leading_slot {
                row.push(menu_icon_slot_empty(cx));
            }

            let mut label_text = ui::text(cx, label.clone())
                .w_full()
                .min_w_0()
                .flex_1()
                .basis_0()
                .text_size_px(text_style.size)
                .font_weight(text_style.weight)
                .text_color(ColorRef::Color(fg))
                .nowrap();
            if let Some(line_height) = text_style.line_height {
                label_text = label_text.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                label_text = label_text.letter_spacing_em(letter_spacing_em);
            }
            row.push(label_text.into_element(cx));

            if let Some(t) = trailing.clone() {
                row.push(t);
            }

            if has_submenu {
                row.push(submenu_chevron_right_text(cx, fg, text_style.clone()));
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

fn submenu_chevron_right_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    fg: Color,
    _text_style: TextStyle,
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

#[derive(Clone)]
pub struct Menubar {
    menus: Vec<MenubarMenuEntries>,
    modal: bool,
    disabled: bool,
    typeahead_timeout_ticks: u64,
    align_leading_icons: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Menubar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menubar")
            .field("menus_len", &self.menus.len())
            .field("modal", &self.modal)
            .field("disabled", &self.disabled)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl Menubar {
    pub fn new(menus: impl IntoIterator<Item = MenubarMenuEntries>) -> Self {
        let menus = menus.into_iter().collect();
        Self {
            menus,
            modal: false,
            disabled: false,
            typeahead_timeout_ticks: 30,
            align_leading_icons: true,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Controls whether outside interactions are blocked while submenus are open (Base UI `modal`).
    ///
    /// Default is `false` to preserve current shadcn menubar behavior.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
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

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let radius = self
                .chrome
                .radius
                .as_ref()
                .map(|m| m.resolve(&theme))
                .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme));
            let border_width = self
                .chrome
                .border_width
                .as_ref()
                .map(|m| m.resolve(&theme))
                .unwrap_or(Px(1.0));
            let border = self
                .chrome
                .border_color
                .as_ref()
                .map(|c| c.resolve(&theme))
                .unwrap_or_else(|| theme.color_required("border"));

            let pad_x = MetricRef::space(Space::N1).resolve(&theme);
            let pad_y = MetricRef::space(Space::N1).resolve(&theme);
            let padding = self.chrome.padding.clone().unwrap_or_default();
            let pad_top = padding.top.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
            let pad_right = padding.right.map(|m| m.resolve(&theme)).unwrap_or(pad_x);
            let pad_bottom = padding.bottom.map(|m| m.resolve(&theme)).unwrap_or(pad_y);
            let pad_left = padding.left.map(|m| m.resolve(&theme)).unwrap_or(pad_x);

            let gap = MetricRef::space(Space::N1).resolve(&theme);
            let bg = self
                .chrome
                .background
                .as_ref()
                .map(|c| c.resolve(&theme))
                .unwrap_or_else(|| theme.color_required("background"));
            let shadow = decl_style::shadow_xs(&theme, radius);
            let layout = decl_style::layout_style(&theme, self.layout.clone());

            let disabled = self.disabled;
            let modal = self.modal;
            let menus = self.menus;
            let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
            let align_leading_icons = self.align_leading_icons;
            let on_dismiss_request = self.on_dismiss_request.clone();
            let on_open_auto_focus = self.on_open_auto_focus.clone();
            let on_close_auto_focus = self.on_close_auto_focus.clone();

            let trigger_labels: Arc<[Arc<str>]> = Arc::from(
                menus
                    .iter()
                    .map(|m| m.menu.label.clone())
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            let trigger_disabled: Arc<[bool]> = Arc::from(
                menus
                    .iter()
                    .map(|m| disabled || m.menu.disabled)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );

            let menus: Vec<MenubarMenuEntries> = menus
                .into_iter()
                .map(|menu| {
                    menu.modal(modal)
                        .on_dismiss_request(on_dismiss_request.clone())
                        .on_open_auto_focus(on_open_auto_focus.clone())
                        .on_close_auto_focus(on_close_auto_focus.clone())
                })
                .collect();

            with_menubar_provider_state(cx, disabled, |cx| {
                cx.semantics(
                    SemanticsProps {
                        layout: LayoutStyle::default(),
                        role: SemanticsRole::MenuBar,
                        disabled,
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout,
                                padding: Edges {
                                    top: pad_top,
                                    right: pad_right,
                                    bottom: pad_bottom,
                                    left: pad_left,
                                },
                                background: Some(bg),
                                shadow: Some(shadow),
                                border: Edges::all(border_width),
                                border_color: Some(border),
                                corner_radii: Corners::all(radius),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![roving_focus_group::roving_focus_group_apg(
                                    cx,
                                    RovingFlexProps {
                                        flex: FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap,
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        roving: RovingFocusProps {
                                            enabled: !disabled,
                                            wrap: true,
                                            disabled: trigger_disabled.clone(),
                                        },
                                    },
                                    roving_focus_group::TypeaheadPolicy::Prefix {
                                        labels: trigger_labels.clone(),
                                        timeout_ticks: typeahead_timeout_ticks,
                                    },
                                    move |cx| {
                                        menus
                                            .into_iter()
                                            .map(|m| {
                                                m.align_leading_icons(align_leading_icons)
                                                    .into_element(cx)
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )]
                            },
                        )]
                    },
                )
            })
        })
    }
}

#[derive(Default)]
struct MenubarMenuState {
    open: Option<Model<bool>>,
    close_prevent_auto_focus: Option<Model<bool>>,
    keyboard_focus_last_on_open: Option<Model<bool>>,
}

#[derive(Debug, Default, Clone)]
struct MenubarProviderState {
    disabled: bool,
}

fn menubar_disabled_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> bool {
    cx.inherited_state_where::<MenubarProviderState>(|st| st.disabled)
        .is_some_and(|st| st.disabled)
}

#[track_caller]
fn with_menubar_provider_state<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    disabled: bool,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MenubarProviderState::default, |st| {
        let prev = st.disabled;
        st.disabled = disabled;
        prev
    });
    let out = f(cx);
    cx.with_state(MenubarProviderState::default, |st| {
        st.disabled = prev;
    });
    out
}

#[derive(Clone)]
pub struct MenubarMenu {
    label: Arc<str>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    window_margin: Px,
    side_offset: Px,
    typeahead_timeout_ticks: u64,
}

impl std::fmt::Debug for MenubarMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenubarMenu")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("window_margin", &self.window_margin)
            .field("side_offset", &self.side_offset)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl MenubarMenu {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            test_id: None,
            window_margin: Px(8.0),
            side_offset: Px(8.0),
            typeahead_timeout_ticks: 30,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn entries(self, entries: impl IntoIterator<Item = MenubarEntry>) -> MenubarMenuEntries {
        let entries: Vec<MenubarEntry> = entries.into_iter().collect();
        MenubarMenuEntries {
            menu: self,
            entries: Arc::from(entries.into_boxed_slice()),
            modal: false,
            align_leading_icons: true,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }
}

#[derive(Clone)]
pub struct MenubarMenuEntries {
    menu: MenubarMenu,
    entries: Arc<[MenubarEntry]>,
    modal: bool,
    align_leading_icons: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
}

impl std::fmt::Debug for MenubarMenuEntries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenubarMenuEntries")
            .field("label", &self.menu.label)
            .field("disabled", &self.menu.disabled)
            .field("entries_len", &self.entries.len())
            .field("modal", &self.modal)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl MenubarMenuEntries {
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn align_leading_icons(mut self, align: bool) -> Self {
        self.align_leading_icons = align;
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let group = cx.root_id();
        let key = self.menu.label.clone();
        let entries = self.entries.clone();
        let modal = self.modal;
        let align_leading_icons = self.align_leading_icons;
        let on_dismiss_request = self.on_dismiss_request.clone();
        let on_open_auto_focus = self.on_open_auto_focus.clone();
        let on_close_auto_focus = self.on_close_auto_focus.clone();
        cx.keyed(key, |cx| {
            let group_active = menubar_trigger_row::ensure_group_active_model(cx, group);
            let trigger_registry = menubar_trigger_row::ensure_group_registry_model(cx, group);

            let open = cx.with_state(MenubarMenuState::default, |st| st.open.clone());
            let open = if let Some(open) = open {
                open
            } else {
                let open = cx.app.models_mut().insert(false);
                cx.with_state(MenubarMenuState::default, |st| st.open = Some(open.clone()));
                open
            };

            let close_prevent_auto_focus =
                cx.with_state(MenubarMenuState::default, |st| st.close_prevent_auto_focus.clone());
            let close_prevent_auto_focus = if let Some(model) = close_prevent_auto_focus {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state(MenubarMenuState::default, |st| {
                    st.close_prevent_auto_focus = Some(model.clone())
                });
                model
            };
            let keyboard_focus_last_on_open = cx.with_state(MenubarMenuState::default, |st| {
                st.keyboard_focus_last_on_open.clone()
            });
            let keyboard_focus_last_on_open = if let Some(model) = keyboard_focus_last_on_open {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state(MenubarMenuState::default, |st| {
                    st.keyboard_focus_last_on_open = Some(model.clone())
                });
                model
            };

            let theme = Theme::global(&*cx.app).clone();
            let enabled = !(self.menu.disabled || menubar_disabled_in_scope(cx));

            let radius = MetricRef::radius(Radius::Sm).resolve(&theme);
            let ring = decl_style::focus_ring(&theme, radius);
            let bg_hover = theme.color_required("accent");
            let bg_open = alpha_mul(bg_hover, 0.35);
            let fg = theme.color_required("foreground");
            let fg_muted = theme.color_required("muted-foreground");

            let font_size = theme.metric_required("font.size");
            let font_line_height = theme.metric_required("font.line_height");
            let text_style = TextStyle {
                font: FontId::default(),
                size: font_size,
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(font_line_height),
                letter_spacing_em: None,
            };

            let label = self.menu.label.clone();
            let test_id = self.menu.test_id.clone();

            cx.pressable_with_id_props(|cx, st, trigger_id| {
                let (patient_click_sticky, patient_click_timer) =
                    menubar_trigger_row::ensure_trigger_patient_click_models(cx, trigger_id);
                let first_item_focus_id: Rc<Cell<Option<GlobalElementId>>> =
                    Rc::new(Cell::new(None));
                let last_item_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                if enabled {
                    let open_for_arrow_keys = open.clone();
                    let first_item_focus_id_for_arrow_keys = first_item_focus_id.clone();
                    let last_item_focus_id_for_arrow_keys = last_item_focus_id.clone();
                    let keyboard_focus_last_on_open_for_arrow_keys =
                        keyboard_focus_last_on_open.clone();
                    cx.key_on_key_down_for(
                        trigger_id,
                        Arc::new(move |host, _acx, down| {
                            if down.repeat {
                                return false;
                            }

                            let focus_last = match down.key {
                                fret_core::KeyCode::ArrowDown => false,
                                fret_core::KeyCode::ArrowUp => true,
                                _ => return false,
                            };

                            let is_open = host
                                .models_mut()
                                .read(&open_for_arrow_keys, |v| *v)
                                .ok()
                                .unwrap_or(false);

                            if !is_open {
                                let _ = host.models_mut().update(
                                    &keyboard_focus_last_on_open_for_arrow_keys,
                                    |v| *v = focus_last,
                                );
                                let _ = host.models_mut().update(&open_for_arrow_keys, |v| *v = true);
                                return true;
                            }

                            let target = if focus_last {
                                last_item_focus_id_for_arrow_keys.get()
                            } else {
                                first_item_focus_id_for_arrow_keys.get()
                            };
                            let Some(target) = target else {
                                return false;
                            };
                            host.request_focus(target);
                            true
                        }),
                    );
                }

                menubar_trigger_row::register_trigger_in_registry(
                    cx,
                    trigger_registry.clone(),
                    trigger_id,
                    open.clone(),
                    enabled,
                    None,
                );

                let mut trigger_layout = LayoutStyle::default();
                trigger_layout.size.height = Length::Auto;
                trigger_layout.size.width = Length::Auto;

                menubar_trigger_row::sync_trigger_row_state(
                    cx,
                    group_active.clone(),
                    trigger_id,
                    open.clone(),
                    patient_click_sticky.clone(),
                    patient_click_timer.clone(),
                    enabled,
                    st.hovered,
                    st.pressed,
                    st.focused,
                );
                cx.pressable_add_on_activate(menubar_trigger_row::toggle_on_activate(
                    group_active.clone(),
                    trigger_id,
                    open.clone(),
                    patient_click_sticky,
                    patient_click_timer,
                ));

                let model_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
                if !enabled && model_open {
                    let _ = cx.app.models_mut().update(&open, |v| *v = false);
                }
                let is_open = model_open && enabled;
                let keyboard_focus_last_on_open_now = cx
                    .watch_model(&keyboard_focus_last_on_open)
                    .layout()
                    .copied()
                    .unwrap_or(false);
                if is_open {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&close_prevent_auto_focus, |v| *v = false);
                }
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
                let overlay_root_name = menu::menubar_root_name(trigger_id);
                let overlay_root_name_for_controls: Arc<str> = Arc::from(overlay_root_name.clone());
                let content_id_for_trigger =
                    menu::content_panel::menu_content_semantics_id(cx, &overlay_root_name);
                let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
                let submenu = cx.with_root_name(&overlay_root_name, |cx| {
                    menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
                });
                let mut states = WidgetStates::from_pressable(cx, st, enabled);
                states.set(WidgetState::Open, is_open);

                let trigger_bg_prop = WidgetStateProperty::new(None)
                    .when(WidgetStates::HOVERED, Some(alpha_mul(bg_hover, 0.8)))
                    .when(WidgetStates::ACTIVE, Some(alpha_mul(bg_hover, 0.8)))
                    .when(WidgetStates::FOCUSED, Some(alpha_mul(bg_hover, 0.8)))
                    .when(WidgetStates::OPEN, Some(bg_open))
                    .when(WidgetStates::DISABLED, None);

                let trigger_bg = *trigger_bg_prop.resolve(states);

                let props = PressableProps {
                    layout: trigger_layout,
                    enabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::MenuItem),
                        label: Some(label.clone()),
                        test_id: test_id.clone(),
                        expanded: Some(is_open),
                        controls_element: Some(content_id_for_trigger.0),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                if overlay_presence.present && enabled {
                    let side_offset = self.menu.side_offset;
                    let window_margin = self.menu.window_margin;
                    let typeahead_timeout_ticks = self.menu.typeahead_timeout_ticks;
                    let on_dismiss_request: Option<fret_ui::action::OnDismissRequest> = {
                        let open_for_dismiss = open.clone();
                        let close_prevent_auto_focus_for_dismiss = close_prevent_auto_focus.clone();
                        let user_on_dismiss_request = on_dismiss_request.clone();
                        Some(Arc::new(
                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                  cx: fret_ui::action::ActionCx,
                                  req: &mut fret_ui::action::DismissRequestCx| {
                                let prevent_auto_focus = matches!(
                                    req.reason,
                                    fret_ui::action::DismissReason::OutsidePress { .. }
                                        | fret_ui::action::DismissReason::FocusOutside
                                );
                                let _ = host.models_mut().update(
                                    &close_prevent_auto_focus_for_dismiss,
                                    |v| *v = prevent_auto_focus,
                                );

                                if let Some(user) = user_on_dismiss_request.as_ref() {
                                    user(host, cx, req);
                                }

                                if !req.default_prevented() {
                                    let _ =
                                        host.models_mut().update(&open_for_dismiss, |v| *v = false);
                                }
                            },
                        ))
                    };
                    let on_close_auto_focus: Option<fret_ui::action::OnCloseAutoFocus> = {
                        let close_prevent_auto_focus_for_hook = close_prevent_auto_focus.clone();
                        on_close_auto_focus.clone().or_else(|| {
                            Some(Arc::new(
                                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                                      _cx: fret_ui::action::ActionCx,
                                      req: &mut fret_ui::action::AutoFocusRequestCx| {
                                    let prevent = host
                                        .models_mut()
                                        .read(&close_prevent_auto_focus_for_hook, |v| *v)
                                        .ok()
                                        .unwrap_or(false);
                                    if prevent {
                                        req.prevent_default();
                                    }
                                },
                            ))
                        })
                    };
                    let group_active = group_active;
                    let open_for_overlay = open.clone();
                    let text_style = text_style.clone();
                    let entries = entries.clone();
                    let trigger_registry_for_overlay = trigger_registry.clone();
                    let content_focus_id: Rc<Cell<Option<GlobalElementId>>> =
                        Rc::new(Cell::new(None));
                    let content_focus_id_for_children = content_focus_id.clone();
                    let content_focus_id_for_children_for_content =
                        content_focus_id_for_children.clone();
                    let content_focus_id_for_children_for_submenu =
                        content_focus_id_for_children.clone();
                    let first_item_focus_id_for_request = first_item_focus_id.clone();
                    let first_item_focus_id_for_children = first_item_focus_id.clone();
                    let first_item_focus_id_for_children_for_content =
                        first_item_focus_id_for_children.clone();
                    let last_item_focus_id_for_request = last_item_focus_id.clone();
                    let last_item_focus_id_for_children = last_item_focus_id.clone();
                    let last_item_focus_id_for_children_for_content =
                        last_item_focus_id_for_children.clone();
                    let direction = direction_prim::use_direction_in_scope(cx, None);

                    let (overlay_children, dismissible_on_pointer_move) =
                        cx.with_root_name(&overlay_root_name, move |cx| {
                        let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                            return (Vec::new(), None);
                        };
                        let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                        let mut flat: Vec<MenubarEntry> = Vec::new();
                        flatten_entries(&mut flat, entries.iter().cloned().collect());
                        let entries: Arc<[MenubarEntry]> = Arc::from(flat.into_boxed_slice());

                        let pad_y = MetricRef::space(Space::N1p5).resolve(&theme);
                        let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                        let pad_x_inset = MetricRef::space(Space::N8).resolve(&theme);
                        let font_size = theme.metric_required("font.size");
                        let font_line_height = theme.metric_required("font.line_height");
                        let mut desired = menu_panel_desired_size(
                            &entries,
                            Px(192.0),
                            font_size,
                            font_line_height,
                            pad_x,
                            pad_x_inset,
                            pad_y,
                        );
                        let popper_placement = popper::PopperContentPlacement::new(
                            direction,
                            Side::Bottom,
                            Align::Start,
                            side_offset,
                        )
                        .with_align_offset(Px(-4.0));

                        let popper_vars =
                            menu::menubar_popper_vars(outer, anchor, desired.width, popper_placement);
                        let desired_w =
                            menu::menubar_popper_desired_width(outer, anchor, desired.width);
                        let max_h = theme
                            .metric_by_key("component.menubar.max_height")
                            .map(|h| Px(h.0.min(popper_vars.available_height.0)))
                            .unwrap_or(popper_vars.available_height);
                        desired.width = desired_w;
                        desired.height = Px(desired.height.0.min(max_h.0));

                        let layout =
                            popper::popper_content_layout_sized(outer, anchor, desired, popper_placement);
                        let placed = layout.rect;
                        let origin = popper::popper_content_transform_origin(&layout, anchor, None);
                        let transform = overlay_motion::shadcn_popper_presence_transform(
                            layout.side,
                            origin,
                            opacity,
                            scale,
                            true,
                        );
                        let reserve_leading_slot = align_leading_icons
                            && entries.iter().any(|e| match e {
                                MenubarEntry::Item(item) => item.leading.is_some(),
                                MenubarEntry::CheckboxItem(item) => item.leading.is_some(),
                                MenubarEntry::RadioItem(item) => item.leading.is_some(),
                                MenubarEntry::Submenu(submenu) => submenu.trigger.leading.is_some(),
                                MenubarEntry::Label(_)
                                | MenubarEntry::Group(_)
                                | MenubarEntry::RadioGroup(_)
                                | MenubarEntry::Separator => false,
                            });

                        let item_count = entries
                            .iter()
                            .filter(|e| {
                                matches!(
                                    e,
                                    MenubarEntry::Item(_)
                                        | MenubarEntry::CheckboxItem(_)
                                        | MenubarEntry::RadioItem(_)
                                        | MenubarEntry::Submenu(_)
                                )
                            })
                            .count();

                        let gating: WindowCommandGatingSnapshot =
                            crate::command_gating::snapshot_for_window(&*cx.app, cx.window);

                        let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                            .iter()
                            .filter_map(|e| match e {
                                MenubarEntry::Item(item) => Some((
                                    item.label.clone(),
                                    item.disabled
                                        || crate::command_gating::command_is_disabled_by_gating(
                                            &*cx.app,
                                            &gating,
                                            item.command.as_ref(),
                                        ),
                                )),
                                MenubarEntry::CheckboxItem(item) => {
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
                                MenubarEntry::RadioItem(item) => Some((
                                    item.label.clone(),
                                    item.disabled
                                        || crate::command_gating::command_is_disabled_by_gating(
                                            &*cx.app,
                                            &gating,
                                            item.command.as_ref(),
                                        ),
                                )),
                                MenubarEntry::Submenu(submenu) => {
                                    Some((
                                        submenu.trigger.label.clone(),
                                        submenu.trigger.disabled
                                            || crate::command_gating::command_is_disabled_by_gating(
                                                &*cx.app,
                                                &gating,
                                                submenu.trigger.command.as_ref(),
                                            ),
                                    ))
                                }
                                MenubarEntry::Label(_)
                                | MenubarEntry::Separator
                                | MenubarEntry::Group(_)
                                | MenubarEntry::RadioGroup(_) => None,
                            })
                            .unzip();

                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                        let disabled_arc: Arc<[bool]> =
                            Arc::from(disabled_flags.clone().into_boxed_slice());
                        let active = roving_focus_group::first_enabled(&disabled_flags);

                        let roving = RovingFocusProps {
                            enabled: true,
                            wrap: false,
                            disabled: disabled_arc,
                            ..Default::default()
                        };

                        let border = theme.color_required("border");
                        let radius_sm = MetricRef::radius(Radius::Sm).resolve(&theme);
                        let item_ring = decl_style::focus_ring(&theme, radius_sm);
                        let destructive_fg = theme.color_required("destructive");
                        let destructive_bg_alpha = if is_dark_background(&theme) { 0.20 } else { 0.10 };
                        let destructive_bg = theme
                            .color_by_key(if destructive_bg_alpha >= 0.2 {
                                "destructive/20"
                            } else {
                                "destructive/10"
                            })
                            .unwrap_or_else(|| alpha_mul(destructive_fg, destructive_bg_alpha));

                        let panel_chrome = crate::ui_builder_ext::surfaces::menu_style_chrome();
                        let submenu_chrome =
                            crate::ui_builder_ext::surfaces::menu_sub_style_chrome().rounded(Radius::Md);

                        let open_for_submenu = open_for_overlay.clone();
                        let submenu_for_content = submenu.clone();
                        let submenu_for_panel = submenu.clone();
                        let submenu_for_panel_for_content = submenu_for_panel.clone();
                        let entries_for_content = entries.clone();
                        let entries_for_submenu = entries.clone();
                        let group_active_for_content = group_active.clone();
                        let group_active_for_submenu = group_active.clone();
                        let text_style_for_content = text_style.clone();
                        let text_style_for_submenu = text_style.clone();
                        let trigger_registry_for_overlay_for_content =
                            trigger_registry_for_overlay.clone();

                        let theme_for_content = theme.clone();
                        let (_content_id, content) =
                            menu::content_panel::menu_content_semantics_with_id(
                                cx,
                                LayoutStyle {
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
                                    overflow: Overflow::Clip,
                                    ..Default::default()
                                },
                                move |cx| {
                                    let theme = theme_for_content.clone();
                                    let local_placed = Rect::new(
                                        fret_core::Point::new(Px(0.0), Px(0.0)),
                                        placed.size,
                                    );
                                    let theme_for_panel_layout = theme.clone();
                                    let panel_chrome_for_panel = panel_chrome.clone();
                                    vec![menu::content_panel::menu_panel_container_at(
                                        cx,
                                        local_placed,
                                        move |layout| {
                                            let mut props = decl_style::container_props(
                                                &theme_for_panel_layout,
                                                panel_chrome_for_panel.clone(),
                                                LayoutRefinement::default(),
                                            );
                                            props.layout = layout;
                                            props
                                        },
                                        move |cx| {
                                            let content_focus_id_for_panel =
                                                content_focus_id_for_children_for_content.clone();
                                            let group_active_for_switch =
                                                group_active_for_content.clone();
                                            let trigger_registry_for_switch =
                                                trigger_registry_for_overlay_for_content.clone();
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
                                              labels_arc.clone(),
                                              typeahead_timeout_ticks,
                                              submenu_for_panel_for_content.clone(),
                                              move |cx| {
                                                  let mut out: Vec<AnyElement> =
                                                      Vec::with_capacity(entries_for_content.len());

                                                  let mut item_ix: usize = 0;

                                                 for (idx, entry) in
                                                     entries_for_content.iter().enumerate()
                                                {
                                                                 match entry {
                                                                     MenubarEntry::Separator => {
                                                                         out.push(cx.container(
                                                                             ContainerProps {
                                                                 layout: {
                                                                     let mut layout =
                                                                         LayoutStyle::default();
                                                                     layout.size.width =
                                                                         Length::Fill;
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
                                                                 background: Some(border),
                                                                 ..Default::default()
                                                             },
                                                                |_| Vec::new(),
                                                            ));
                                                        }
                                                        MenubarEntry::Label(label) => {
                                                            let text = label.text.clone();
                                                            let fg = alpha_mul(fg_muted, 0.85);
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
                                                                        .text_color(ColorRef::Color(fg))
                                                                        .nowrap()
                                                                        .into_element(cx)]
                                                                },
                                                            ));
                                                        }
                                                        MenubarEntry::CheckboxItem(item) => {
                                                            let collection_index = item_ix;
                                                            item_ix = item_ix.saturating_add(1);

                                                            let focusable =
                                                                active.is_some_and(|a| a == idx);
                                                            let label = item.label.clone();
                                                            let value = item.value.clone();
                                                            let checked = item.checked.clone();
                                                            let a11y_label = item.a11y_label.clone();
                                                            let command = item.command.clone();
                                                            let item_enabled = !item.disabled
                                                                && enabled
                                                                && !crate::command_gating::command_is_disabled_by_gating(
                                                                    &*cx.app,
                                                                    &gating,
                                                                    command.as_ref(),
                                                                );
                                                            let trailing = item.trailing.clone();
                                                            let leading = item.leading.clone();
                                                            let close_on_select = item.close_on_select;
                                                            let open = open_for_overlay.clone();
                                                            let group_active =
                                                                group_active_for_content.clone();
                                                            let text_style =
                                                                text_style_for_content.clone();
                                                            let submenu_for_item =
                                                                submenu_for_content.clone();
                                                            let trigger_registry =
                                                                trigger_registry_for_overlay_for_content.clone();
                                                            let first_item_focus_id_for_items =
                                                                first_item_focus_id_for_children_for_content.clone();
                                                            let last_item_focus_id_for_items =
                                                                last_item_focus_id_for_children_for_content.clone();

                                                            let _theme = theme.clone();
                                                            out.push(cx.keyed(value.clone(), move |cx| {
                                                                cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                    let checked_now = cx
                                                                        .watch_model(&checked)
                                                                        .copied()
                                                                        .unwrap_or(false);
                                                                    if item_enabled {
                                                                        if first_item_focus_id_for_items.get().is_none() {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        last_item_focus_id_for_items
                                                                            .set(Some(item_id));
                                                                    }

                                                                    let _ = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        !item_enabled,
                                                                        false,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        None,
                                                                    );
                                                                    menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                        cx,
                                                                        item_id,
                                                                        group_active.clone(),
                                                                        trigger_registry.clone(),
                                                                    );

                                                                    if item_enabled {
                                                                        menu::checkbox_item::wire_toggle_on_activate(
                                                                            cx,
                                                                            checked.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(&open, false);

                                                                            let group_active_for_activate =
                                                                                group_active.clone();
                                                                            cx.pressable_add_on_activate(
                                                                                Arc::new(move |host, _cx, _reason| {
                                                                                    let _ = host
                                                                                        .models_mut()
                                                                                        .update(&group_active_for_activate, |v| *v = None);
                                                                                }),
                                                                            );
                                                                        }
                                                                    }

                                                                    let mut states =
                                                                        WidgetStates::from_pressable(
                                                                            cx,
                                                                            st,
                                                                            item_enabled,
                                                                        );
                                                                    states.set(WidgetState::Open, false);

                                                                    let highlight_bg = bg_hover;
                                                                    let bg_prop = WidgetStateProperty::new(
                                                                        Color::TRANSPARENT,
                                                                    )
                                                                    .when(WidgetStates::HOVERED, highlight_bg)
                                                                    .when(WidgetStates::ACTIVE, highlight_bg)
                                                                    .when(WidgetStates::FOCUSED, highlight_bg)
                                                                    .when(
                                                                        WidgetStates::DISABLED,
                                                                        Color::TRANSPARENT,
                                                                    );
                                                                    let fg_prop =
                                                                        WidgetStateProperty::new(fg)
                                                                            .when(
                                                                                WidgetStates::DISABLED,
                                                                                alpha_mul(fg_muted, 0.85),
                                                                            );

                                                                    let bg = *bg_prop.resolve(states);
                                                                    let fg = *fg_prop.resolve(states);

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(
                                                                                cx,
                                                                                cmd,
                                                                                fret_runtime::Platform::current(),
                                                                            )
                                                                            .map(|text| {
                                                                                MenubarShortcut::new(text).into_element(cx)
                                                                            })
                                                                        })
                                                                    });

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.height =
                                                                                Length::Auto;
                                                                            layout
                                                                        },
                                                                        enabled: item_enabled,
                                                                        focusable,
                                                                        focus_ring: Some(item_ring),
                                                                        a11y: menu::item::menu_item_checkbox_a11y(
                                                                            a11y_label.or_else(|| Some(label.clone())),
                                                                            checked_now,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        Some(checked_now),
                                                                        false,
                                                                        bg,
                                                                        fg,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_style.clone(),
                                                                    );

                                                                    (props, children)
                                                                })
                                                            }));
                                                        }
                                                        MenubarEntry::RadioItem(item) => {
                                                            let collection_index = item_ix;
                                                            item_ix = item_ix.saturating_add(1);

                                                            let focusable =
                                                                active.is_some_and(|a| a == idx);
                                                            let label = item.label.clone();
                                                            let value = item.value.clone();
                                                            let group_value = item.group_value.clone();
                                                            let a11y_label = item.a11y_label.clone();
                                                            let command = item.command.clone();
                                                            let item_enabled = !item.disabled
                                                                && enabled
                                                                && !crate::command_gating::command_is_disabled_by_gating(
                                                                    &*cx.app,
                                                                    &gating,
                                                                    command.as_ref(),
                                                                );
                                                            let trailing = item.trailing.clone();
                                                            let leading = item.leading.clone();
                                                            let close_on_select = item.close_on_select;
                                                            let open = open_for_overlay.clone();
                                                            let group_active =
                                                                group_active_for_content.clone();
                                                            let text_style =
                                                                text_style_for_content.clone();
                                                            let submenu_for_item =
                                                                submenu_for_content.clone();
                                                            let trigger_registry =
                                                                trigger_registry_for_overlay_for_content.clone();
                                                            let first_item_focus_id_for_items =
                                                                first_item_focus_id_for_children_for_content.clone();
                                                            let last_item_focus_id_for_items =
                                                                last_item_focus_id_for_children_for_content.clone();

                                                            let _theme = theme.clone();
                                                            out.push(cx.keyed(value.clone(), move |cx| {
                                                                cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                    let selected = cx
                                                                        .watch_model(&group_value)
                                                                        .cloned()
                                                                        .flatten();
                                                                    let is_selected = menu::radio_group::is_selected(
                                                                        selected.as_ref(),
                                                                        &value,
                                                                    );
                                                                    if item_enabled {
                                                                        if first_item_focus_id_for_items.get().is_none() {
                                                                            first_item_focus_id_for_items
                                                                                .set(Some(item_id));
                                                                        }
                                                                        last_item_focus_id_for_items
                                                                            .set(Some(item_id));
                                                                    }

                                                                    let _ = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        !item_enabled,
                                                                        false,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        None,
                                                                    );
                                                                    menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                        cx,
                                                                        item_id,
                                                                        group_active.clone(),
                                                                        trigger_registry.clone(),
                                                                    );

                                                                    if item_enabled {
                                                                        menu::radio_group::wire_select_on_activate(
                                                                            cx,
                                                                            group_value.clone(),
                                                                            value.clone(),
                                                                        );
                                                                        cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                        if close_on_select {
                                                                            cx.pressable_set_bool(&open, false);

                                                                            let group_active_for_activate =
                                                                                group_active.clone();
                                                                            cx.pressable_add_on_activate(
                                                                                Arc::new(move |host, _cx, _reason| {
                                                                                    let _ = host
                                                                                        .models_mut()
                                                                                        .update(&group_active_for_activate, |v| *v = None);
                                                                                }),
                                                                            );
                                                                        }
                                                                    }

                                                                    let mut states =
                                                                        WidgetStates::from_pressable(
                                                                            cx,
                                                                            st,
                                                                            item_enabled,
                                                                        );
                                                                    states.set(WidgetState::Open, false);

                                                                    let highlight_bg = bg_hover;
                                                                    let bg_prop = WidgetStateProperty::new(
                                                                        Color::TRANSPARENT,
                                                                    )
                                                                    .when(WidgetStates::HOVERED, highlight_bg)
                                                                    .when(WidgetStates::ACTIVE, highlight_bg)
                                                                    .when(WidgetStates::FOCUSED, highlight_bg)
                                                                    .when(
                                                                        WidgetStates::DISABLED,
                                                                        Color::TRANSPARENT,
                                                                    );
                                                                    let fg_prop =
                                                                        WidgetStateProperty::new(fg)
                                                                            .when(
                                                                                WidgetStates::DISABLED,
                                                                                alpha_mul(fg_muted, 0.85),
                                                                            );

                                                                    let bg = *bg_prop.resolve(states);
                                                                    let fg = *fg_prop.resolve(states);

                                                                    let trailing = trailing.clone().or_else(|| {
                                                                        command.as_ref().and_then(|cmd| {
                                                                            command_shortcut_label(
                                                                                cx,
                                                                                cmd,
                                                                                fret_runtime::Platform::current(),
                                                                            )
                                                                            .map(|text| {
                                                                                MenubarShortcut::new(text).into_element(cx)
                                                                            })
                                                                        })
                                                                    });

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.height =
                                                                                Length::Auto;
                                                                            layout
                                                                        },
                                                                        enabled: item_enabled,
                                                                        focusable,
                                                                        focus_ring: Some(item_ring),
                                                                        a11y: menu::item::menu_item_radio_a11y(
                                                                            a11y_label.or_else(|| Some(label.clone())),
                                                                            is_selected,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        Some(is_selected),
                                                                        false,
                                                                        bg,
                                                                        fg,
                                                                        pad_x,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_style.clone(),
                                                                    );

                                                                    (props, children)
                                                                })
                                                            }));
                                                        }
                                                        MenubarEntry::Group(_)
                                                        | MenubarEntry::RadioGroup(_) => {
                                                            unreachable!("entries are flattened")
                                                        }
                                                        MenubarEntry::Item(item)
                                                        | MenubarEntry::Submenu(MenubarSubmenu {
                                                            trigger: item,
                                                            ..
                                                        }) => {
                                                            let collection_index = item_ix;
                                                            item_ix = item_ix.saturating_add(1);

                                                             let focusable =
                                                                 active.is_some_and(|a| a == idx);
                                                             let label = item.label.clone();
                                                               let a11y_label =
                                                                    item.a11y_label.clone();
                                                                let command = item.command.clone();
                                                            let item_enabled = !item.disabled
                                                                && enabled
                                                                && !crate::command_gating::command_is_disabled_by_gating(
                                                                    &*cx.app,
                                                                    &gating,
                                                                    command.as_ref(),
                                                                );
                                                               let trailing = item.trailing.clone();
                                                               let leading = item.leading.clone();
                                                               let close_on_select = item.close_on_select;
                                                             let variant = item.variant;
                                                               let open = open_for_overlay.clone();
                                                               let group_active =
                                                                   group_active_for_content.clone();
                                                             let text_style =
                                                                 text_style_for_content.clone();
                                                              let has_submenu =
                                                                   matches!(entry, MenubarEntry::Submenu(_));
                                                              let submenu_desired_for_hint =
                                                                  if let MenubarEntry::Submenu(submenu) = entry {
                                                                      let mut flat: Vec<MenubarEntry> =
                                                                          Vec::new();
                                                                      flatten_entries(
                                                                          &mut flat,
                                                                          submenu
                                                                              .entries
                                                                              .iter()
                                                                              .cloned()
                                                                              .collect(),
                                                                      );
                                                                      let submenu_max_h = theme
                                                                          .metric_by_key(
                                                                              "component.menubar.max_height",
                                                                          )
                                                                          .map(|h| {
                                                                              Px(h.0.min(
                                                                                  outer.size.height.0,
                                                                              ))
                                                                          })
                                                                          .unwrap_or(outer.size.height);
                                                                      let desired = menu_panel_desired_size(
                                                                          &flat,
                                                                          Px(128.0),
                                                                          font_size,
                                                                          font_line_height,
                                                                          pad_x,
                                                                          pad_x_inset,
                                                                          pad_y,
                                                                      );
                                                                      Some(Size::new(
                                                                          desired.width,
                                                                          Px(desired
                                                                              .height
                                                                              .0
                                                                              .min(submenu_max_h.0)),
                                                                      ))
                                                                  } else {
                                                                      None
                                                                  };

                                                               let submenu_for_item =
                                                                   submenu_for_content.clone();
                                                               let trigger_registry =
                                                                   trigger_registry_for_overlay_for_content.clone();
                                                              let value = item.value.clone();
                                                              let test_id = item.test_id.clone();
                                                               let pad_left =
                                                                   if item.inset { pad_x_inset } else { pad_x };
                                                            let _theme = theme.clone();
                                                                let overlay_root_name_for_controls =
                                                                    overlay_root_name_for_controls.clone();
                                                                let first_item_focus_id_for_items =
                                                                    first_item_focus_id_for_children_for_content.clone();
                                                                let last_item_focus_id_for_items =
                                                                    last_item_focus_id_for_children_for_content.clone();
                                                                out.push(cx.keyed(value.clone(), move |cx| {
                                                                    cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                      if item_enabled {
                                                                          if first_item_focus_id_for_items.get().is_none() {
                                                                              first_item_focus_id_for_items
                                                                                  .set(Some(item_id));
                                                                          }
                                                                          last_item_focus_id_for_items
                                                                              .set(Some(item_id));
                                                                      }
                                                                     let geometry_hint = submenu_desired_for_hint.map(|desired| {
                                                                         menu::sub_trigger::MenuSubTriggerGeometryHint {
                                                                             outer,
                                                                             desired,
                                                                         }
                                                                    });
                                                                    let expanded = menu::sub_trigger::wire(
                                                                        cx,
                                                                        st,
                                                                        item_id,
                                                                        !item_enabled,
                                                                        has_submenu,
                                                                        value.clone(),
                                                                        &submenu_for_item,
                                                                        submenu_cfg,
                                                                        geometry_hint,
                                                                    );
                                                                    menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                        cx,
                                                                        item_id,
                                                                        group_active.clone(),
                                                                        trigger_registry.clone(),
                                                                    );

                                                                     if !has_submenu {
                                                                         cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                        if item_enabled && close_on_select {
                                                                            cx.pressable_set_bool(&open, false);

                                                                            let group_active_for_activate =
                                                                                group_active.clone();
                                                                            cx.pressable_add_on_activate(
                                                                                Arc::new(move |host, _cx, _reason| {
                                                                                    let _ = host
                                                                                        .models_mut()
                                                                                        .update(&group_active_for_activate, |v| *v = None);
                                                                                }),
                                                                            );
                                                                        }
                                                                     }

                                                                    let mut states =
                                                                        WidgetStates::from_pressable(
                                                                            cx,
                                                                            st,
                                                                            item_enabled,
                                                                        );
                                                                    states.set(
                                                                        WidgetState::Open,
                                                                        expanded.unwrap_or(false),
                                                                    );

                                                                    let highlight_bg =
                                                                        if variant == MenubarItemVariant::Destructive {
                                                                            destructive_bg
                                                                        } else {
                                                                            bg_hover
                                                                        };
                                                                    let base_fg =
                                                                        if variant == MenubarItemVariant::Destructive {
                                                                            destructive_fg
                                                                        } else {
                                                                            fg
                                                                        };

                                                                    let bg_prop = WidgetStateProperty::new(
                                                                        Color::TRANSPARENT,
                                                                    )
                                                                    .when(WidgetStates::HOVERED, highlight_bg)
                                                                    .when(WidgetStates::ACTIVE, highlight_bg)
                                                                    .when(WidgetStates::FOCUSED, highlight_bg)
                                                                    .when(WidgetStates::OPEN, highlight_bg)
                                                                    .when(
                                                                        WidgetStates::DISABLED,
                                                                        Color::TRANSPARENT,
                                                                    );
                                                                    let fg_prop =
                                                                        WidgetStateProperty::new(base_fg)
                                                                            .when(
                                                                                WidgetStates::DISABLED,
                                                                                alpha_mul(fg_muted, 0.85),
                                                                            );

                                                                    let bg = *bg_prop.resolve(states);
                                                                    let fg = *fg_prop.resolve(states);

                                                                    let props = PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.height =
                                                                                Length::Auto;
                                                                            layout
                                                                        },
                                                                        enabled: item_enabled,
                                                                        focusable,
                                                                        focus_ring: Some(item_ring),
                                                                        a11y: {
                                                                            let controls = has_submenu.then(|| {
                                                                                menu::sub_content::submenu_content_semantics_id(
                                                                                    cx,
                                                                                    overlay_root_name_for_controls
                                                                                        .as_ref(),
                                                                                    &value,
                                                                                )
                                                                            });
                                                                            let mut a11y = menu::item::menu_item_a11y_with_controls(
                                                                                a11y_label.or_else(|| {
                                                                                    Some(label.clone())
                                                                                }),
                                                                                expanded,
                                                                                controls,
                                                                            )
                                                                            ;
                                                                            a11y.test_id = test_id.clone();
                                                                            a11y.with_collection_position(
                                                                                collection_index,
                                                                                item_count,
                                                                            )
                                                                        },
                                                                        ..Default::default()
                                                                    };

                                                                    let trailing = if has_submenu {
                                                                        trailing.clone()
                                                                    } else {
                                                                        trailing.clone().or_else(|| {
                                                                            command.as_ref().and_then(|cmd| {
                                                                                command_shortcut_label(
                                                                                    cx,
                                                                                    cmd,
                                                                                    fret_runtime::Platform::current(),
                                                                                )
                                                                                .map(|text| {
                                                                                    MenubarShortcut::new(text)
                                                                                        .into_element(cx)
                                                                                })
                                                                            })
                                                                        })
                                                                    };

                                                                    let children = menu_row_children(
                                                                        cx,
                                                                        label.clone(),
                                                                        leading.clone(),
                                                                        reserve_leading_slot,
                                                                        trailing,
                                                                        None,
                                                                        has_submenu,
                                                                        bg,
                                                                        fg,
                                                                        pad_left,
                                                                        pad_x,
                                                                        pad_y,
                                                                        radius_sm,
                                                                        text_style.clone(),
                                                                    );

                                                                    (props, children)
                                                                })
                                                            }));

                                                            #[cfg(any())]
                                                            out.push(cx.pressable(
                                                                PressableProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.size.width =
                                                                            Length::Fill;
                                                                        layout.size.height =
                                                                            Length::Auto;
                                                                        layout
                                                                    },
                                                                    enabled: item_enabled,
                                                                    focusable,
                                                                    focus_ring: Some(item_ring),
                                                                    a11y: menu::item::menu_item_a11y(
                                                                        a11y_label.or_else(|| {
                                                                            Some(label.clone())
                                                                        }),
                                                                        None,
                                                                    )
                                                                    .with_collection_position(
                                                                        collection_index,
                                                                        item_count,
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                                move |cx, st| {
                                                                    cx.pressable_dispatch_command_if_enabled_opt(command);
                                                                    cx.pressable_set_bool(&open, false);
                                                                    let group_active_for_activate =
                                                                        group_active.clone();
                                                                    cx.pressable_add_on_activate(
                                                                        Arc::new(move |host, _cx, _reason| {
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(&group_active_for_activate, |v| *v = None);
                                                                        }),
                                                                    );

                                                                    let mut states =
                                                                        WidgetStates::from_pressable(
                                                                            cx,
                                                                            st,
                                                                            item_enabled,
                                                                        );
                                                                    states.set(WidgetState::Open, false);

                                                                    let highlight_bg = bg_hover;
                                                                    let bg_prop = WidgetStateProperty::new(
                                                                        Color::TRANSPARENT,
                                                                    )
                                                                    .when(WidgetStates::HOVERED, highlight_bg)
                                                                    .when(WidgetStates::ACTIVE, highlight_bg)
                                                                    .when(WidgetStates::FOCUSED, highlight_bg)
                                                                    .when(
                                                                        WidgetStates::DISABLED,
                                                                        Color::TRANSPARENT,
                                                                    );
                                                                    let fg_prop =
                                                                        WidgetStateProperty::new(fg)
                                                                            .when(
                                                                                WidgetStates::DISABLED,
                                                                                alpha_mul(fg_muted, 0.85),
                                                                            );

                                                                    let bg = *bg_prop.resolve(states);
                                                                    let fg = *fg_prop.resolve(states);

                                                                    vec![cx.container(
                                                                        ContainerProps {
                                                                            layout: LayoutStyle::default(),
                                                                            padding: Edges {
                                                                                top: pad_y,
                                                                                right: pad_x,
                                                                                bottom: pad_y,
                                                                                left: pad_x,
                                                                            },
                                                                            background: Some(bg),
                                                                            shadow: None,
                                                                            border: Edges::all(
                                                                                Px(0.0),
                                                                            ),
                                                                            border_color: None,
                                                                            corner_radii: Corners::all(
                                                                                radius_sm,
                                                                            ),
                                                                        },
                                                                        move |cx| {
                                                                            let mut label_text = ui::text(cx, label.clone())
                                                                                .text_size_px(text_style.size)
                                                                                .font_weight(text_style.weight)
                                                                                .text_color(ColorRef::Color(fg))
                                                                                .nowrap();
                                                                            if let Some(line_height) = text_style.line_height {
                                                                                label_text = label_text.line_height_px(line_height);
                                                                            }
                                                                            if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                                                                                label_text = label_text.letter_spacing_em(letter_spacing_em);
                                                                            }
                                                                            vec![label_text.into_element(cx)]
                                                                        },
                                                                    )]
                                                                },
                                                            ));
                                                        }
                                                    }
                                                }

                                                out
                                            },
                                        );
                                        menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                            cx,
                                            roving.id,
                                            group_active_for_switch,
                                            trigger_registry_for_switch,
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
                                            overflow: fret_ui::element::Overflow::Clip,
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
                            .as_deref()
                            .and_then(|open_value| {
                                entries_for_submenu.iter().find_map(|entry| {
                                    let MenubarEntry::Submenu(submenu) = entry else {
                                        return None;
                                    };
                                    (submenu.trigger.value.as_ref() == open_value)
                                        .then_some(submenu.entries.clone())
                                })
                            })
                            .map(|submenu_entries| {
                                let mut flat: Vec<MenubarEntry> = Vec::new();
                                flatten_entries(
                                    &mut flat,
                                    submenu_entries.iter().cloned().collect::<Vec<_>>(),
                                );
                                menu_panel_desired_size(
                                    &flat,
                                    Px(128.0),
                                    font_size,
                                    font_line_height,
                                    pad_x,
                                    pad_x_inset,
                                    pad_y,
                                )
                            })
                            .unwrap_or_else(|| {
                                menu_panel_desired_size(
                                    &[],
                                    Px(128.0),
                                    font_size,
                                    font_line_height,
                                    pad_x,
                                    pad_x_inset,
                                    pad_y,
                                )
                            });
                        let submenu_max_h = theme
                            .metric_by_key("component.menubar.max_height")
                            .map(|h| Px(h.0.min(outer.size.height.0)))
                            .unwrap_or(outer.size.height);
                        let desired = Size::new(desired.width, Px(desired.height.0.min(submenu_max_h.0)));
                        let submenu_is_open = submenu_open_value.is_some();
                        let submenu_motion =
                            radix_presence::scale_fade_presence_with_durations_and_easing(
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
                            let geometry = open_submenu.map(|(_, geometry)| geometry).or(last_geometry);

                            let (Some(open_value), Some(geometry)) = (open_value, geometry) else {
                                return (children, Some(dismissible_on_pointer_move));
                            };

                            let placed = geometry.floating;
                            let submenu_entries = entries_for_submenu.iter().find_map(|e| {
                                let MenubarEntry::Submenu(submenu) = e else {
                                    return None;
                                };
                                (submenu.trigger.value.as_ref() == open_value.as_ref())
                                    .then_some(submenu.entries.clone())
                            });

                            if let Some(submenu_entries) = submenu_entries {
                                    let mut flat: Vec<MenubarEntry> = Vec::new();
                                    flatten_entries(
                                        &mut flat,
                                        submenu_entries.iter().cloned().collect(),
                                    );
                                    let submenu_entries: Arc<[MenubarEntry]> =
                                        Arc::from(flat.into_boxed_slice());
                                     let reserve_leading_slot = align_leading_icons
                                         && submenu_entries.iter().any(|e| match e {
                                             MenubarEntry::Item(item) => item.leading.is_some(),
                                             MenubarEntry::CheckboxItem(item) => item.leading.is_some(),
                                             MenubarEntry::RadioItem(item) => item.leading.is_some(),
                                             MenubarEntry::Submenu(submenu) => {
                                                 submenu.trigger.leading.is_some()
                                             }
                                             MenubarEntry::Label(_)
                                             | MenubarEntry::Group(_)
                                             | MenubarEntry::RadioGroup(_)
                                             | MenubarEntry::Separator => false,
                                         });

                                    let gating: WindowCommandGatingSnapshot =
                                        crate::command_gating::snapshot_for_window(&*cx.app, cx.window);

                                     let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) =
                                         submenu_entries
                                             .iter()
                                             .filter_map(|e| match e {
                                                 MenubarEntry::Item(item) => {
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
                                                 MenubarEntry::CheckboxItem(item) => {
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
                                                 MenubarEntry::RadioItem(item) => {
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
                                                 MenubarEntry::Submenu(submenu) => Some((
                                                     submenu.trigger.label.clone(),
                                                     submenu.trigger.disabled
                                                         || crate::command_gating::command_is_disabled_by_gating(
                                                             &*cx.app,
                                                             &gating,
                                                             submenu.trigger.command.as_ref(),
                                                         ),
                                                 )),
                                                 MenubarEntry::Label(_)
                                                 | MenubarEntry::Separator
                                                 | MenubarEntry::Group(_)
                                                 | MenubarEntry::RadioGroup(_) => None,
                                             })
                                             .unzip();

                                    let labels_arc: Arc<[Arc<str>]> =
                                        Arc::from(labels.into_boxed_slice());
                                    let disabled_arc: Arc<[bool]> = Arc::from(
                                        disabled_flags.clone().into_boxed_slice(),
                                    );
                                    let active = roving_focus_group::first_enabled(&disabled_flags);
                                    let item_count = submenu_entries
                                        .iter()
                                        .filter(|e| {
                                            matches!(
                                                e,
                                                MenubarEntry::Item(_)
                                                    | MenubarEntry::CheckboxItem(_)
                                                    | MenubarEntry::RadioItem(_)
                                                    | MenubarEntry::Submenu(_)
                                            )
                                        })
                                        .count();

                                    let roving = RovingFocusProps {
                                        enabled: true,
                                        wrap: false,
                                        disabled: disabled_arc,
                                        ..Default::default()
                                    };

                                    let submenu_entries_for_panel = submenu_entries.clone();
                                    let open_for_submenu = open_for_submenu.clone();
                                    let group_active = group_active_for_submenu.clone();
                                    let trigger_registry_for_switch =
                                        trigger_registry_for_overlay.clone();
                                    let text_style = text_style_for_submenu.clone();
                                    let submenu_models_for_panel = submenu_for_panel.clone();
                                    let labelled_by_element = cx
                                        .app
                                        .models_mut()
                                        .read(&submenu_models_for_panel.trigger, |v| *v)
                                        .ok()
                                        .flatten();
                                    let item_ring = item_ring;

                                    let theme_for_panel_layout = theme.clone();
                                    let submenu_chrome_for_panel = submenu_chrome.clone();
                                    let submenu_panel = menu::sub_content::submenu_panel_scroll_y_for_value_at(
                                        cx,
                                        open_value.clone(),
                                        placed,
                                        labelled_by_element,
                                        move |layout| {
                                            let mut props = decl_style::container_props(
                                                &theme_for_panel_layout,
                                                submenu_chrome_for_panel.clone(),
                                                LayoutRefinement::default(),
                                            );
                                            props.layout = layout;
                                            props
                                        },
                                        move |cx| {
                                            let content_focus_id_for_panel =
                                                content_focus_id_for_children_for_submenu.clone();
                                            let group_active_for_switch = group_active.clone();
                                            let trigger_registry_for_switch =
                                                trigger_registry_for_switch.clone();
                                            let trigger_registry_for_items =
                                                trigger_registry_for_switch.clone();
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
                                                            roving,
                                                        },
                                                         labels_arc.clone(),
                                                         typeahead_timeout_ticks,
                                                         move |cx| {
                                                             let gating: WindowCommandGatingSnapshot =
                                                                 crate::command_gating::snapshot_for_window(
                                                                     &*cx.app,
                                                                     cx.window,
                                                                 );
                                                             let mut out: Vec<AnyElement> =
                                                                 Vec::with_capacity(submenu_entries_for_panel.len());
                                                             let mut item_ix: usize = 0;

                                                            for (idx, entry) in
                                                                submenu_entries_for_panel.iter().enumerate()
                                                            {
                                                                match entry {
                                                                    MenubarEntry::Separator => {
                                                                        out.push(cx.container(
                                                                            ContainerProps {
                                                                                layout: {
                                                                                    let mut layout =
                                                                                        LayoutStyle::default();
                                                                                    layout.size.width =
                                                                                        Length::Fill;
                                                                                    layout.size.height =
                                                                                        Length::Px(Px(1.0));
                                                                                    // new-york-v4: `Separator` uses `-mx-1 my-1`.
                                                                                    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                    layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                                    layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                                    layout
                                                                                },
                                                                                background: Some(border),
                                                                                ..Default::default()
                                                                            },
                                                                             |_| Vec::new(),
                                                                         ));
                                                                     }
                                                                    MenubarEntry::Label(label) => {
                                                                        let text = label.text.clone();
                                                                        let fg = alpha_mul(fg_muted, 0.85);
                                                                        let pad_left = if label.inset {
                                                                            pad_x_inset
                                                                        } else {
                                                                            pad_x
                                                                        };
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
                                                                                    .text_color(ColorRef::Color(fg))
                                                                                    .nowrap()
                                                                                    .into_element(cx)]
                                                                            },
                                                                        ));
                                                                    }
                                                                    MenubarEntry::CheckboxItem(item) => {
                                                                        let collection_index = item_ix;
                                                                        item_ix = item_ix.saturating_add(1);

                                                                        let focusable = active.is_some_and(|a| a == idx);
                                                                        let label = item.label.clone();
                                                                        let a11y_label = item.a11y_label.clone();
                                                                        let command = item.command.clone();
                                                                        let item_enabled = !item.disabled
                                                                            && !crate::command_gating::command_is_disabled_by_gating(
                                                                                &*cx.app,
                                                                                &gating,
                                                                                command.as_ref(),
                                                                            );
                                                                        let trailing = item.trailing.clone();
                                                                        let leading = item.leading.clone();
                                                                        let close_on_select = item.close_on_select;
                                                                        let open = open_for_submenu.clone();
                                                                        let group_active = group_active.clone();
                                                                        let trigger_registry =
                                                                            trigger_registry_for_items.clone();
                                                                        let submenu_for_key = submenu_models_for_panel.clone();
                                                                        let value = item.value.clone();
                                                                        let checked = item.checked.clone();
                                                                        let text_style = text_style.clone();

                                                                        let theme = theme.clone();
                                                                        out.push(cx.keyed(value.clone(), move |cx| {
                                                                            cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                                menu::sub_content::wire_item(
                                                                                    cx,
                                                                                    item_id,
                                                                                    !item_enabled,
                                                                                    &submenu_for_key,
                                                                                );
                                                                                menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                                    cx,
                                                                                    item_id,
                                                                                    group_active.clone(),
                                                                                    trigger_registry.clone(),
                                                                                );

                                                                                let checked_now = cx
                                                                                    .watch_model(&checked)
                                                                                    .copied()
                                                                                    .unwrap_or(false);
                                                                                if item_enabled {
                                                                                    menu::checkbox_item::wire_toggle_on_activate(
                                                                                        cx,
                                                                                        checked.clone(),
                                                                                    );
                                                                                }
                                                                                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                                if item_enabled && close_on_select {
                                                                                    cx.pressable_set_bool(&open, false);
                                                                                    let group_active_for_activate = group_active.clone();
                                                                                    cx.pressable_add_on_activate(
                                                                                        Arc::new(move |host, _cx, _reason| {
                                                                                            let _ = host
                                                                                                .models_mut()
                                                                                                .update(&group_active_for_activate, |v| *v = None);
                                                                                        }),
                                                                                    );
                                                                                }

                                                                                let mut states =
                                                                                    WidgetStates::from_pressable(
                                                                                        cx,
                                                                                        st,
                                                                                        item_enabled,
                                                                                    );
                                                                                states.set(WidgetState::Open, false);

                                                                                let highlight_bg =
                                                                                    theme.color_required("accent");
                                                                                let bg_prop =
                                                                                    WidgetStateProperty::new(Color::TRANSPARENT)
                                                                                        .when(WidgetStates::HOVERED, highlight_bg)
                                                                                        .when(WidgetStates::ACTIVE, highlight_bg)
                                                                                        .when(WidgetStates::FOCUSED, highlight_bg)
                                                                                        .when(WidgetStates::DISABLED, Color::TRANSPARENT);
                                                                                let fg_prop =
                                                                                    WidgetStateProperty::new(fg).when(
                                                                                        WidgetStates::DISABLED,
                                                                                        alpha_mul(fg_muted, 0.85),
                                                                                    );

                                                                                let bg = *bg_prop.resolve(states);
                                                                                let fg = *fg_prop.resolve(states);

                                                                                let trailing = trailing.clone().or_else(|| {
                                                                                    command.as_ref().and_then(|cmd| {
                                                                                        command_shortcut_label(
                                                                                            cx,
                                                                                            cmd,
                                                                                            fret_runtime::Platform::current(),
                                                                                        )
                                                                                        .map(|text| {
                                                                                            MenubarShortcut::new(text).into_element(cx)
                                                                                        })
                                                                                    })
                                                                                });

                                                                                let props = PressableProps {
                                                                                    layout: {
                                                                                        let mut layout =
                                                                                            LayoutStyle::default();
                                                                                        layout.size.width =
                                                                                            Length::Fill;
                                                                                        layout.size.height =
                                                                                            Length::Auto;
                                                                                        layout
                                                                                    },
                                                                                    enabled: item_enabled,
                                                                                    focusable,
                                                                                    focus_ring: Some(item_ring),
                                                                                    a11y: menu::item::menu_item_checkbox_a11y(
                                                                                        a11y_label.or_else(|| Some(label.clone())),
                                                                                        checked_now,
                                                                                    )
                                                                                    .with_collection_position(
                                                                                        collection_index,
                                                                                        item_count,
                                                                                    ),
                                                                                    ..Default::default()
                                                                                };

                                                                                let children = menu_row_children(
                                                                                    cx,
                                                                                    label.clone(),
                                                                                    leading.clone(),
                                                                                    reserve_leading_slot,
                                                                                    trailing,
                                                                                    Some(checked_now),
                                                                                    false,
                                                                                    bg,
                                                                                    fg,
                                                                                    pad_x,
                                                                                    pad_x,
                                                                                    pad_y,
                                                                                    radius_sm,
                                                                                    text_style.clone(),
                                                                                );

                                                                                (props, children)
                                                                            })
                                                                        }));
                                                                    }
                                                                    MenubarEntry::RadioItem(item) => {
                                                                        let collection_index = item_ix;
                                                                        item_ix = item_ix.saturating_add(1);

                                                                        let focusable = active.is_some_and(|a| a == idx);
                                                                        let label = item.label.clone();
                                                                        let a11y_label = item.a11y_label.clone();
                                                                        let command = item.command.clone();
                                                                        let item_enabled = !item.disabled
                                                                            && !crate::command_gating::command_is_disabled_by_gating(
                                                                                &*cx.app,
                                                                                &gating,
                                                                                command.as_ref(),
                                                                            );
                                                                        let trailing = item.trailing.clone();
                                                                        let leading = item.leading.clone();
                                                                        let close_on_select = item.close_on_select;
                                                                        let open = open_for_submenu.clone();
                                                                        let group_active = group_active.clone();
                                                                        let trigger_registry =
                                                                            trigger_registry_for_items.clone();
                                                                        let submenu_for_key = submenu_models_for_panel.clone();
                                                                        let value = item.value.clone();
                                                                        let group_value = item.group_value.clone();
                                                                        let text_style = text_style.clone();

                                                                        let theme = theme.clone();
                                                                        out.push(cx.keyed(value.clone(), move |cx| {
                                                                            cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                                menu::sub_content::wire_item(
                                                                                    cx,
                                                                                    item_id,
                                                                                    !item_enabled,
                                                                                    &submenu_for_key,
                                                                                );
                                                                                menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                                    cx,
                                                                                    item_id,
                                                                                    group_active.clone(),
                                                                                    trigger_registry.clone(),
                                                                                );

                                                                                let selected = cx
                                                                                    .watch_model(&group_value)
                                                                                    .cloned()
                                                                                    .flatten();
                                                                                let is_selected = menu::radio_group::is_selected(
                                                                                    selected.as_ref(),
                                                                                    &value,
                                                                                );
                                                                                if item_enabled {
                                                                                    menu::radio_group::wire_select_on_activate(
                                                                                        cx,
                                                                                        group_value.clone(),
                                                                                        value.clone(),
                                                                                    );
                                                                                }
                                                                                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                                if item_enabled && close_on_select {
                                                                                    cx.pressable_set_bool(&open, false);
                                                                                    let group_active_for_activate = group_active.clone();
                                                                                    cx.pressable_add_on_activate(
                                                                                        Arc::new(move |host, _cx, _reason| {
                                                                                            let _ = host
                                                                                                .models_mut()
                                                                                                .update(&group_active_for_activate, |v| *v = None);
                                                                                        }),
                                                                                    );
                                                                                }

                                                                                let mut states =
                                                                                    WidgetStates::from_pressable(
                                                                                        cx,
                                                                                        st,
                                                                                        item_enabled,
                                                                                    );
                                                                                states.set(WidgetState::Open, false);

                                                                                let highlight_bg =
                                                                                    theme.color_required("accent");
                                                                                let bg_prop =
                                                                                    WidgetStateProperty::new(Color::TRANSPARENT)
                                                                                        .when(WidgetStates::HOVERED, highlight_bg)
                                                                                        .when(WidgetStates::ACTIVE, highlight_bg)
                                                                                        .when(WidgetStates::FOCUSED, highlight_bg)
                                                                                        .when(WidgetStates::DISABLED, Color::TRANSPARENT);
                                                                                let fg_prop =
                                                                                    WidgetStateProperty::new(fg).when(
                                                                                        WidgetStates::DISABLED,
                                                                                        alpha_mul(fg_muted, 0.85),
                                                                                    );

                                                                                let bg = *bg_prop.resolve(states);
                                                                                let fg = *fg_prop.resolve(states);

                                                                                let trailing = trailing.clone().or_else(|| {
                                                                                    command.as_ref().and_then(|cmd| {
                                                                                        command_shortcut_label(
                                                                                            cx,
                                                                                            cmd,
                                                                                            fret_runtime::Platform::current(),
                                                                                        )
                                                                                        .map(|text| {
                                                                                            MenubarShortcut::new(text).into_element(cx)
                                                                                        })
                                                                                    })
                                                                                });

                                                                                let props = PressableProps {
                                                                                    layout: {
                                                                                        let mut layout =
                                                                                            LayoutStyle::default();
                                                                                        layout.size.width =
                                                                                            Length::Fill;
                                                                                        layout.size.height =
                                                                                            Length::Auto;
                                                                                        layout
                                                                                    },
                                                                                    enabled: item_enabled,
                                                                                    focusable,
                                                                                    focus_ring: Some(item_ring),
                                                                                    a11y: menu::item::menu_item_radio_a11y(
                                                                                        a11y_label.or_else(|| Some(label.clone())),
                                                                                        is_selected,
                                                                                    )
                                                                                    .with_collection_position(
                                                                                        collection_index,
                                                                                        item_count,
                                                                                    ),
                                                                                    ..Default::default()
                                                                                };

                                                                                let children = menu_row_children(
                                                                                    cx,
                                                                                    label.clone(),
                                                                                    leading.clone(),
                                                                                    reserve_leading_slot,
                                                                                    trailing,
                                                                                    Some(is_selected),
                                                                                    false,
                                                                                    bg,
                                                                                    fg,
                                                                                    pad_x,
                                                                                    pad_x,
                                                                                    pad_y,
                                                                                    radius_sm,
                                                                                    text_style.clone(),
                                                                                );

                                                                                (props, children)
                                                                            })
                                                                        }));
                                                                    }
                                                                    MenubarEntry::Item(item) => {
                                                                        let collection_index = item_ix;
                                                                        item_ix = item_ix.saturating_add(1);

                                                                        let focusable = active.is_some_and(|a| a == idx);
                                                                        let label = item.label.clone();
                                                                        let a11y_label = item.a11y_label.clone();
                                                                        let test_id = item.test_id.clone();
                                                                        let command = item.command.clone();
                                                                        let item_enabled = !item.disabled
                                                                            && !crate::command_gating::command_is_disabled_by_gating(
                                                                                &*cx.app,
                                                                                &gating,
                                                                                command.as_ref(),
                                                                            );
                                                                        let trailing = item.trailing.clone();
                                                                        let leading = item.leading.clone();
                                                                        let close_on_select = item.close_on_select;
                                                                        let variant = item.variant;
                                                                        let open = open_for_submenu.clone();
                                                                        let group_active = group_active.clone();
                                                                        let trigger_registry =
                                                                            trigger_registry_for_items.clone();
                                                                        let submenu_for_key = submenu_models_for_panel.clone();
                                                                        let value = item.value.clone();
                                                                        let pad_left =
                                                                            if item.inset { pad_x_inset } else { pad_x };
                                                                        let text_style = text_style.clone();

                                                                        let theme = theme.clone();
                                                                        out.push(cx.keyed(value.clone(), move |cx| {
                                                                            cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                                menu::sub_content::wire_item(
                                                                                    cx,
                                                                                    item_id,
                                                                                    !item_enabled,
                                                                                    &submenu_for_key,
                                                                                );
                                                                                menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                                                    cx,
                                                                                    item_id,
                                                                                    group_active.clone(),
                                                                                    trigger_registry.clone(),
                                                                                );

                                                                                cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                                                                if item_enabled && close_on_select {
                                                                                    cx.pressable_set_bool(&open, false);

                                                                                    let group_active_for_activate =
                                                                                        group_active.clone();
                                                                                    cx.pressable_add_on_activate(
                                                                                        Arc::new(move |host, _cx, _reason| {
                                                                                            let _ = host
                                                                                                .models_mut()
                                                                                                .update(&group_active_for_activate, |v| *v = None);
                                                                                        }),
                                                                                    );
                                                                                }

                                                                                let mut states =
                                                                                    WidgetStates::from_pressable(
                                                                                        cx,
                                                                                        st,
                                                                                        item_enabled,
                                                                                    );
                                                                                states.set(WidgetState::Open, false);

                                                                                let highlight_bg =
                                                                                    if variant == MenubarItemVariant::Destructive {
                                                                                        destructive_bg
                                                                                    } else {
                                                                                        theme.color_required("accent")
                                                                                    };
                                                                                let base_fg =
                                                                                    if variant == MenubarItemVariant::Destructive {
                                                                                        destructive_fg
                                                                                    } else {
                                                                                        fg
                                                                                    };

                                                                                let bg_prop =
                                                                                    WidgetStateProperty::new(Color::TRANSPARENT)
                                                                                        .when(WidgetStates::HOVERED, highlight_bg)
                                                                                        .when(WidgetStates::ACTIVE, highlight_bg)
                                                                                        .when(WidgetStates::FOCUSED, highlight_bg)
                                                                                        .when(WidgetStates::DISABLED, Color::TRANSPARENT);
                                                                                let fg_prop =
                                                                                    WidgetStateProperty::new(base_fg).when(
                                                                                        WidgetStates::DISABLED,
                                                                                        alpha_mul(fg_muted, 0.85),
                                                                                    );

                                                                                let bg = *bg_prop.resolve(states);
                                                                                let fg = *fg_prop.resolve(states);

                                                                                let trailing = trailing.clone().or_else(|| {
                                                                                    command.as_ref().and_then(|cmd| {
                                                                                        command_shortcut_label(
                                                                                            cx,
                                                                                            cmd,
                                                                                            fret_runtime::Platform::current(),
                                                                                        )
                                                                                        .map(|text| {
                                                                                            MenubarShortcut::new(text).into_element(cx)
                                                                                        })
                                                                                    })
                                                                                });

                                                                                let props = PressableProps {
                                                                                    layout: {
                                                                                        let mut layout =
                                                                                            LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout.size.height = Length::Auto;
                                                                                        layout
                                                                                    },
                                                                                    enabled: item_enabled,
                                                                                    focusable,
                                                                                    focus_ring: Some(item_ring),
                                                                                    a11y: {
                                                                                        let mut a11y =
                                                                                            menu::item::menu_item_a11y(
                                                                                                a11y_label.or_else(|| {
                                                                                                    Some(label.clone())
                                                                                                }),
                                                                                                None,
                                                                                            );
                                                                                        a11y.test_id = test_id.clone();
                                                                                        a11y.with_collection_position(
                                                                                            collection_index,
                                                                                            item_count,
                                                                                        )
                                                                                    },
                                                                                    ..Default::default()
                                                                                };

                                                                                let children = menu_row_children(
                                                                                    cx,
                                                                                    label.clone(),
                                                                                    leading.clone(),
                                                                                    reserve_leading_slot,
                                                                                    trailing,
                                                                                    None,
                                                                                    false,
                                                                                    bg,
                                                                                    fg,
                                                                                    pad_left,
                                                                                    pad_x,
                                                                                    pad_y,
                                                                                    radius_sm,
                                                                                    text_style.clone(),
                                                                                );

                                                                                (props, children)
                                                                            })
                                                                        }));
                                                                    }
                                                                    MenubarEntry::Group(_)
                                                                    | MenubarEntry::RadioGroup(_) => {
                                                                        unreachable!("entries are flattened")
                                                                    }
                                                                    MenubarEntry::Submenu(_submenu) => {}
                                                                }
                                                            }

                                                            out
                                                        },
                                                    );
                                            menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
                                                cx,
                                                roving.id,
                                                group_active_for_switch,
                                                trigger_registry_for_switch,
                                            );
                                            if content_focus_id_for_panel.get().is_none() {
                                                content_focus_id_for_panel.set(Some(roving.id));
                                            }
                                            vec![roving]
                                        },
                                    );

                                    let side = overlay_motion::anchored_side(
                                        geometry.reference,
                                        geometry.floating,
                                    );
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
                                    let submenu_panel_content = submenu_panel;
                                    let submenu_panel = cx.interactivity_gate(
                                        submenu_motion.present,
                                        submenu_is_open,
                                        move |cx| {
                                            vec![overlay_motion::wrap_opacity_and_render_transform(
                                                cx,
                                                opacity,
                                                transform,
                                                vec![submenu_panel_content],
                                            )]
                                        },
                                    );

                                    children.push(submenu_panel);
                            }
                        }

                        (children, Some(dismissible_on_pointer_move))
                    });

                    let request =
                        menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
                        cx,
                        trigger_id,
                        trigger_id,
                        open,
                        overlay_presence,
                        overlay_children,
                        overlay_root_name,
                        menu::root::MenuInitialFocusTargets::new()
                            .pointer_content_focus(content_focus_id.get())
                            .keyboard_entry_focus(if keyboard_focus_last_on_open_now {
                                last_item_focus_id_for_request.get()
                            } else {
                                first_item_focus_id_for_request.get()
                            }),
                        on_open_auto_focus.clone(),
                        on_close_auto_focus.clone(),
                        on_dismiss_request.clone(),
                        dismissible_on_pointer_move,
                        modal,
                    );
                    OverlayController::request(cx, request);
                    if keyboard_focus_last_on_open_now {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&keyboard_focus_last_on_open, |v| *v = false);
                    }
                }

                let content = cx.container(
                    ContainerProps {
                        layout: LayoutStyle::default(),
                        padding: Edges {
                            top: Px(4.0),
                            right: Px(8.0),
                            bottom: Px(4.0),
                            left: Px(8.0),
                        },
                        background: trigger_bg,
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(radius),
                        ..Default::default()
                    },
                    move |cx| {
                        let mut label_text = ui::text(cx, label.clone())
                            .text_size_px(text_style.size)
                            .font_weight(text_style.weight)
                            .text_color(ColorRef::Color(if enabled { fg } else { fg_muted }))
                            .nowrap();
                        if let Some(line_height) = text_style.line_height {
                            label_text = label_text.line_height_px(line_height);
                        }
                        if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                            label_text = label_text.letter_spacing_em(letter_spacing_em);
                        }
                        vec![label_text.into_element(cx)]
                    },
                );

                (props, vec![content])
            })
        })
    }
}

pub fn menubar<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = MenubarMenuEntries>,
{
    Menubar::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, MouseButtons, Point, Rect, TextBlobId,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::FrameId;
    use fret_ui::action::OnOpenAutoFocus;
    use fret_ui::tree::UiTree;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn menubar_modal_builder_updates_flag() {
        let menubar = Menubar::new(std::iter::empty::<MenubarMenuEntries>()).modal(true);
        assert!(menubar.modal);

        let menubar = Menubar::new(std::iter::empty::<MenubarMenuEntries>())
            .modal(true)
            .modal(false);
        assert!(!menubar.modal);
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

    fn center(r: fret_core::Rect) -> Point {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 / 2.0),
            Px(r.origin.y.0 + r.size.height.0 / 2.0),
        )
    }

    fn menu_trigger_bounds(snap: &fret_core::SemanticsSnapshot, label: &str) -> Rect {
        snap.nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(label))
            .map(|n| n.bounds)
            .unwrap_or_else(|| panic!("missing menu trigger {label:?}"))
    }

    fn menu_trigger_expanded(snap: &fret_core::SemanticsSnapshot, label: &str) -> bool {
        snap.nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(label))
            .map(|n| n.flags.expanded)
            .unwrap_or(false)
    }

    fn menu_trigger_node_id(snap: &fret_core::SemanticsSnapshot, label: &str) -> fret_core::NodeId {
        snap.nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(label))
            .map(|n| n.id)
            .unwrap_or_else(|| panic!("missing menu trigger {label:?}"))
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        render_frame_with_disabled(ui, app, services, window, bounds, false);
    }

    fn render_frame_with_disabled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        disabled: bool,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "menubar", |cx| {
                vec![
                    Menubar::new(vec![
                        MenubarMenu::new("File").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("New")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Open")),
                            MenubarEntry::Item(MenubarItem::new("Exit")),
                        ]),
                        MenubarMenu::new("Edit").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("Undo")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Redo")),
                        ]),
                    ])
                    .disabled(disabled)
                    .into_element(cx),
                ]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_open_auto_focus(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "menubar", |cx| {
                vec![
                    Menubar::new(vec![
                        MenubarMenu::new("File").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("New")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Open")),
                            MenubarEntry::Item(MenubarItem::new("Exit")),
                        ]),
                        MenubarMenu::new("Edit").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("Undo")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Redo")),
                        ]),
                    ])
                    .on_open_auto_focus(on_open_auto_focus)
                    .into_element(cx),
                ]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        on_dismiss_request: Option<OnDismissRequest>,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "menubar", |cx| {
                vec![
                    Menubar::new(vec![
                        MenubarMenu::new("File").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("New")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Open")),
                            MenubarEntry::Item(MenubarItem::new("Exit")),
                        ]),
                        MenubarMenu::new("Edit").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("Undo")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Redo")),
                        ]),
                    ])
                    .on_dismiss_request(on_dismiss_request)
                    .into_element(cx),
                ]
            });
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        underlay_clicked: Model<bool>,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "menubar-underlay",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
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

                vec![
                    underlay,
                    menubar(cx, |_cx| {
                        vec![
                            MenubarMenu::new("File").entries(vec![
                                MenubarEntry::Item(MenubarItem::new("New")),
                                MenubarEntry::Separator,
                                MenubarEntry::Item(MenubarItem::new("Open")),
                                MenubarEntry::Item(MenubarItem::new("Exit")),
                            ]),
                            MenubarMenu::new("Edit").entries(vec![
                                MenubarEntry::Item(MenubarItem::new("Undo")),
                                MenubarEntry::Separator,
                                MenubarEntry::Item(MenubarItem::new("Redo")),
                            ]),
                        ]
                    }),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_underlay_and_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        underlay_clicked: Model<bool>,
        on_dismiss_request: Option<OnDismissRequest>,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "menubar-underlay-dismiss",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
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

                vec![
                    underlay,
                    Menubar::new(vec![
                        MenubarMenu::new("File").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("New")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Open")),
                            MenubarEntry::Item(MenubarItem::new("Exit")),
                        ]),
                        MenubarMenu::new("Edit").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("Undo")),
                            MenubarEntry::Separator,
                            MenubarEntry::Item(MenubarItem::new("Redo")),
                        ]),
                    ])
                    .on_dismiss_request(on_dismiss_request)
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_submenu(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "menubar-submenu",
            |cx| {
                vec![menubar(cx, |_cx| {
                    vec![
                        MenubarMenu::new("File").entries(vec![
                            MenubarEntry::Item(MenubarItem::new("New")),
                            MenubarEntry::Submenu(MenubarItem::new("More").submenu(vec![
                                MenubarEntry::Item(MenubarItem::new("Sub Alpha")),
                                MenubarEntry::Item(MenubarItem::new("Sub Beta")),
                            ])),
                            MenubarEntry::Item(MenubarItem::new("Exit")),
                        ]),
                        MenubarMenu::new("Edit")
                            .entries(vec![MenubarEntry::Item(MenubarItem::new("Undo"))]),
                    ]
                })]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_frame_with_underlay_and_submenu(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        underlay_clicked: Model<bool>,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "menubar-underlay-submenu",
            move |cx| {
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
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

                vec![
                    underlay,
                    menubar(cx, |_cx| {
                        vec![
                            MenubarMenu::new("File").entries(vec![
                                MenubarEntry::Item(MenubarItem::new("New")),
                                MenubarEntry::Submenu(MenubarItem::new("More").submenu(vec![
                                    MenubarEntry::Item(MenubarItem::new("Sub Alpha")),
                                    MenubarEntry::Item(MenubarItem::new("Sub Beta")),
                                ])),
                                MenubarEntry::Item(MenubarItem::new("Exit")),
                            ]),
                            MenubarMenu::new("Edit")
                                .entries(vec![MenubarEntry::Item(MenubarItem::new("Undo"))]),
                        ]
                    }),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn menubar_hover_switches_open_menu() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Frame 0: render and locate triggers.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));
        let edit = center(menu_trigger_bounds(&snap0, "Edit"));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 1: "File" is expanded.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));
        assert!(!menu_trigger_expanded(snap1, "Edit"));

        // Hover over "Edit" while a menu is open should switch without click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: edit,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Hover switching is timer-driven (small delay) to avoid accidental flicker while the
        // pointer travels across triggers.
        let effects = app.flush_effects();
        let switch_delay = std::time::Duration::from_millis(100);
        let switch_timer = effects.iter().find_map(|e| match e {
            Effect::SetTimer { token, after, .. } if *after == switch_delay => Some(*token),
            _ => None,
        });
        let Some(switch_timer) = switch_timer else {
            panic!("expected hover-switch SetTimer effect; effects={effects:?}");
        };
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer {
                token: switch_timer,
            },
        );

        // Frame 2: after the timer fires, the hovered menu opens.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap2, "Edit"));

        // Frame 3: the previously-open menu is fully closed.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap3 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!menu_trigger_expanded(snap3, "File"));
        assert!(menu_trigger_expanded(snap3, "Edit"));
    }

    #[test]
    fn menubar_disabled_hides_content_even_when_menu_open_model_true() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Open one menu first (open model = true in component state), then switch root to disabled.
        render_frame_with_disabled(&mut ui, &mut app, &mut services, window, bounds, false);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_disabled(&mut ui, &mut app, &mut services, window, bounds, false);
        let opened = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            menu_trigger_expanded(opened, "File"),
            "baseline: menu should be open before disabling root"
        );

        // Disabled root should immediately gate expanded semantics, while content can still be
        // present for one close-transition frame.
        render_frame_with_disabled(&mut ui, &mut app, &mut services, window, bounds, true);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !menu_trigger_expanded(&snap, "File") && !menu_trigger_expanded(&snap, "Edit"),
            "disabled menubar should not expose expanded trigger semantics"
        );

        // Next frame: close transition settles and menu content should no longer be exposed.
        render_frame_with_disabled(&mut ui, &mut app, &mut services, window, bounds, true);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().all(|n| {
                !(n.role == SemanticsRole::MenuItem
                    && matches!(
                        n.label.as_deref(),
                        Some("New") | Some("Open") | Some("Exit") | Some("Undo") | Some("Redo")
                    ))
            }),
            "disabled menubar should keep menu content hidden"
        );
    }

    #[test]
    fn menubar_triggers_roving_moves_focus_with_arrow_keys() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = menu_trigger_node_id(&snap0, "File");
        let edit = menu_trigger_node_id(&snap0, "Edit");

        ui.set_focus(Some(file));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(ui.focus(), Some(edit));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowLeft,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(ui.focus(), Some(file));

        // Wrap behavior: ArrowLeft from the first trigger wraps to the last trigger.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowLeft,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(ui.focus(), Some(edit));
    }

    #[test]
    fn menubar_opens_on_arrow_down_from_focused_trigger() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot");
        let file = menu_trigger_node_id(snap0, "File");
        ui.set_focus(Some(file));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));
        assert!(
            snap1.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New")
            }),
            "menu items should render after ArrowDown opens the menubar menu"
        );

        let first_item = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New"))
            .expect("New menu item");
        let focus = ui.focus().expect("expected focus after keyboard-open");
        assert!(
            ui.debug_node_path(focus).contains(&first_item.id),
            "keyboard-open should move focus into the first menu item (Radix entry focus)"
        );
    }

    #[test]
    fn menubar_opens_on_arrow_up_from_focused_trigger_and_focuses_last_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot");
        let file = menu_trigger_node_id(snap0, "File");
        ui.set_focus(Some(file));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowUp,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));
        assert!(
            snap1.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Exit")
            }),
            "menu items should render after ArrowUp opens the menubar menu"
        );

        let last_item = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Exit"))
            .expect("Exit menu item");
        let focus = ui.focus().expect("expected focus after keyboard-open");
        assert!(
            ui.debug_node_path(focus).contains(&last_item.id),
            "ArrowUp keyboard-open should move focus into the last menu item"
        );
    }

    #[test]
    fn menubar_keyboard_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        render_frame_with_open_auto_focus(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(handler.clone()),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot");
        let file = menu_trigger_node_id(snap0, "File");
        ui.set_focus(Some(file));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        render_frame_with_open_auto_focus(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(handler),
        );
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(file),
            "expected preventDefault open autofocus to keep focus on trigger"
        );
    }

    #[test]
    fn menubar_pointer_open_focuses_content_not_first_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Frame 0: render and locate triggers.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_node = menu_trigger_node_id(&snap0, "File");
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        ui.set_focus(Some(file_node));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 1: open menu should be present in semantics.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        let first_item = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New"))
            .expect("New menu item");

        let focus = ui.focus().expect("expected focus after pointer-open");
        assert_ne!(
            focus, first_item.id,
            "pointer-open should not move focus to the first menu item (Radix onEntryFocus preventDefault)"
        );
        assert_ne!(
            focus, file_node,
            "pointer-open should focus menu content/roving container rather than keeping trigger focus"
        );
    }

    #[test]
    fn menubar_pointer_hover_hit_test_targets_menu_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        assert!(menu_trigger_expanded(&snap1, "File"));

        let new_item = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New"))
            .expect("New menu item");
        let new_pos = center(new_item.bounds);

        let hover_changes_before = ui.debug_stats().hover_pressable_target_changes;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: new_pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let hover_changes_after = ui.debug_stats().hover_pressable_target_changes;
        assert!(
            hover_changes_after > hover_changes_before,
            "expected hover edge after pointer move: before={hover_changes_before} after={hover_changes_after}"
        );

        // Hover state must persist across renders. If the hovered pressable's element identity
        // changes between frames, the runtime will clear the hover target during GC and the next
        // pointer move will re-trigger a hover edge.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap_after_render = ui.semantics_snapshot().expect("semantics snapshot").clone();
        assert!(
            menu_trigger_expanded(&snap_after_render, "File"),
            "expected File menu to remain open across frames"
        );
        let new_item_after_render = snap_after_render
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New"))
            .expect("New menu item after render");
        assert_eq!(
            new_item_after_render.id, new_item.id,
            "expected menu item identity to be stable across frames"
        );
        let hover_changes_after_render = ui.debug_stats().hover_pressable_target_changes;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: new_pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let hover_changes_after_repeat_move = ui.debug_stats().hover_pressable_target_changes;
        assert_eq!(
            hover_changes_after_repeat_move, hover_changes_after_render,
            "expected repeated pointer move in a new frame to keep the same hovered pressable: before={hover_changes_after_render} after={hover_changes_after_repeat_move}"
        );

        let hit = ui.debug_hit_test(new_pos);
        let hit_node = hit.hit;
        let hit_kind =
            hit_node.and_then(|node| ui.debug_declarative_instance_kind(&mut app, window, node));
        let new_kind = ui.debug_declarative_instance_kind(&mut app, window, new_item.id);
        let hit_layer = hit_node.and_then(|node| ui.node_layer(node));
        let new_layer = ui.node_layer(new_item.id);
        let hit_path_kinds: Vec<&'static str> = hit_node
            .map(|node| ui.debug_node_path(node))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|node| ui.debug_declarative_instance_kind(&mut app, window, node))
            .collect();

        if std::env::var_os("FRET_DEBUG_MENUBAR_HIT_PATH").is_some() {
            let hit_path = hit_node
                .map(|node| ui.debug_node_path(node))
                .unwrap_or_default();
            eprintln!("menubar hit-test debug: position={new_pos:?}");
            eprintln!(
                "  hit={hit_node:?} kind={hit_kind:?} layer={hit_layer:?} active_layer_roots={:?} barrier_root={:?}",
                hit.active_layer_roots, hit.barrier_root
            );
            eprintln!(
                "  new_item={:?} kind={new_kind:?} layer={new_layer:?}",
                new_item.id
            );
            eprintln!("  hit_path:");
            for node in hit_path {
                let kind = ui.debug_declarative_instance_kind(&mut app, window, node);
                eprintln!("    {node:?} kind={kind:?} layer={:?}", ui.node_layer(node));
            }
        }

        assert!(
            hit_node.is_some_and(|node| node == new_item.id || ui.is_descendant(new_item.id, node)),
            "expected hit-test at {new_pos:?} to land within menu item 'New' (node={:?} kind={new_kind:?} layer={new_layer:?}); got hit={hit_node:?} kind={hit_kind:?} layer={hit_layer:?} path_kinds={hit_path_kinds:?}",
            new_item.id,
        );
    }

    #[test]
    #[ignore]
    fn menubar_hover_highlights_menu_item_background() {
        fn overlap_area(a: Rect, b: Rect) -> f32 {
            let ax0 = a.origin.x.0;
            let ay0 = a.origin.y.0;
            let ax1 = ax0 + a.size.width.0;
            let ay1 = ay0 + a.size.height.0;

            let bx0 = b.origin.x.0;
            let by0 = b.origin.y.0;
            let bx1 = bx0 + b.size.width.0;
            let by1 = by0 + b.size.height.0;

            let x0 = ax0.max(bx0);
            let y0 = ay0.max(by0);
            let x1 = ax1.min(bx1);
            let y1 = ay1.min(by1);

            let w = (x1 - x0).max(0.0);
            let h = (y1 - y0).max(0.0);
            w * h
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        assert!(menu_trigger_expanded(&snap1, "File"));

        let new_item = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New"))
            .expect("New menu item");
        let new_pos = center(new_item.bounds);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: new_pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, window, bounds);

        let theme = Theme::global(&app).clone();
        let expected_bg = theme.color_required("accent");

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let mut best: Option<(Rect, fret_core::Color, f32)> = None;
        for op in scene.ops() {
            let fret_core::SceneOp::Quad {
                rect, background, ..
            } = op
            else {
                continue;
            };
            if background.a <= 0.01 {
                continue;
            }
            let score = overlap_area(*rect, new_item.bounds);
            if score <= 0.0 {
                continue;
            }
            if best.is_none_or(|(_, _, best_score)| score > best_score) {
                best = Some((*rect, *background, score));
            }
        }
        let (_, bg, _) = best.expect("hovered menu item background quad");

        let tol = 0.03;
        assert!(
            (bg.r - expected_bg.r).abs() <= tol
                && (bg.g - expected_bg.g).abs() <= tol
                && (bg.b - expected_bg.b).abs() <= tol,
            "expected hover background close to accent={expected_bg:?}, got={bg:?}"
        );
    }

    #[test]
    fn menubar_outside_press_click_through_closes_without_overriding_underlay_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        let underlay_clicked = app.models_mut().insert(false);

        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_node = menu_trigger_node_id(&snap0, "File");
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        let underlay_node = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        ui.set_focus(Some(file_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!menu_trigger_expanded(snap2, "File"));
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to remain on underlay after click-through outside-press dismissal"
        );
    }

    #[test]
    fn menubar_outside_press_can_be_prevented_via_dismiss_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        // Frame 0: render and locate triggers.
        render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(handler.clone()),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 1: "File" is expanded.
        render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(handler.clone()),
        );
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        // Outside press should call the dismiss handler but remain open due to prevent_default.
        let outside = Point::new(Px(470.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
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
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(handler),
        );
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap2, "File"));
        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn menubar_focus_outside_can_be_prevented_via_dismiss_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        let underlay_clicked = app.models_mut().insert(false);

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

        render_frame_with_underlay_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
            Some(handler.clone()),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_node = menu_trigger_node_id(&snap0, "File");
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        let underlay_node = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        ui.set_focus(Some(file_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_underlay_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
            Some(handler.clone()),
        );
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        ui.set_focus(Some(underlay_node));
        render_frame_with_underlay_and_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked,
            Some(handler),
        );
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            menu_trigger_expanded(snap2, "File"),
            "expected menu to remain open when focus-outside dismissal is prevented"
        );
        assert_eq!(
            *reason_cell.lock().unwrap(),
            Some(fret_ui::action::DismissReason::FocusOutside)
        );
    }

    #[test]
    fn menubar_close_transition_remains_click_through() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        let underlay_clicked = app.models_mut().insert(false);

        // Frame 0: render and open the File menu.
        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_node = menu_trigger_node_id(&snap0, "File");
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        let underlay_node = snap0
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("underlay"))
            .map(|n| n.id)
            .expect("underlay node");

        ui.set_focus(Some(file_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 1: menu should be interactive.
        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        // Click the underlay to close the menu (click-through dismissal).
        let underlay_pos = Point::new(Px(10.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 2: close transition. The menu should no longer be interactive, but the menu content
        // may remain present during motion. Underlay must remain interactive (click-through).
        render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!menu_trigger_expanded(snap2, "File"));
        assert!(
            snap2
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New")),
            "expected menu content to remain present during close transition"
        );
        assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
        assert_eq!(ui.focus(), Some(underlay_node));

        // Click again while the menu is still present: must continue to be click-through.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_clicked),
            Some(false),
            "expected underlay click to remain interactive during close transition"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to remain on underlay"
        );
    }

    #[test]
    fn menubar_items_have_collection_position_metadata_excluding_separators() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Frame 0: render and locate triggers.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 1: open menu should be present in semantics.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");

        assert!(menu_trigger_expanded(snap1, "File"));

        let interesting = ["File", "Edit", "New", "Open", "Exit", "Undo", "Redo"];
        let observed: Vec<(SemanticsRole, Option<&str>, Option<u32>, Option<u32>)> = snap1
            .nodes
            .iter()
            .filter(|n| n.label.as_deref().is_some_and(|l| interesting.contains(&l)))
            .map(|n| (n.role, n.label.as_deref(), n.pos_in_set, n.set_size))
            .collect();

        let open = snap1
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Open"))
            .unwrap_or_else(|| panic!("Open node not found; observed={observed:?}"));
        assert_eq!(open.pos_in_set, Some(2));
        assert_eq!(open.set_size, Some(3));
    }

    #[test]
    fn menubar_submenu_opens_on_arrow_right_and_closes_on_arrow_left_restoring_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));

        let more = snap1
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More submenu trigger");
        ui.set_focus(Some(more.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap2, "More"));
        assert!(
            snap2
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu items should render after ArrowRight opens the submenu"
        );

        let sub_alpha = snap2
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha"))
            .expect("Sub Alpha submenu item");
        ui.set_focus(Some(sub_alpha.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowLeft,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap3 = ui.semantics_snapshot().expect("semantics snapshot");

        let more_after_close = snap3
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More submenu trigger after close");
        assert_eq!(
            ui.focus(),
            Some(more_after_close.id),
            "ArrowLeft should restore focus to the submenu trigger"
        );

        for _ in 0..overlay_motion::SHADCN_MOTION_TICKS_100 {
            render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        }

        let snap4 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap4
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu should unmount after the close transition completes"
        );
    }

    #[test]
    fn menubar_submenu_safe_hover_corridor_cancels_close_timer() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(700.0), Px(320.0)),
        );

        // Frame 1: render and open the "File" menu (so the "More" submenu trigger is visible).
        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);

        // Hover "More" to arm the submenu open-delay timer.
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let more = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More"))
            .expect("More submenu trigger");
        let more_bounds = more.bounds;
        let more_center = Point::new(
            Px(more_bounds.origin.x.0 + more_bounds.size.width.0 / 2.0),
            Px(more_bounds.origin.y.0 + more_bounds.size.height.0 / 2.0),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: more_center,
                buttons: MouseButtons::default(),
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
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer { token: open_timer },
        );

        // Frame 3: after open timer fires, the submenu opens.
        render_frame_with_submenu(&mut ui, &mut app, &mut services, window, bounds);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Sub Alpha")
            }),
            "submenu items should render after open-delay fires"
        );

        let cfg = menu::sub::MenuSubmenuConfig::default();
        let close_delay = cfg.close_delay;
        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open menubar menu overlay");
        let overlay_root_name = menu::menubar_root_name(overlay_id);
        let submenu_models = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            &overlay_root_name,
            |cx| menu::sub::ensure_models(cx),
        );
        let geometry = app
            .models_mut()
            .read(&submenu_models.geometry, |v| *v)
            .ok()
            .flatten();
        let Some(geometry) = geometry else {
            panic!("expected submenu geometry to be available after open");
        };
        let grace_geometry = menu::pointer_grace_intent::PointerGraceIntentGeometry {
            reference: geometry.reference,
            floating: geometry.floating,
        };

        // Pick a safe corridor point on the submenu side (to the right) so moving towards it can
        // cancel a pending close timer (Radix pointer-grace intent).
        let reference_right =
            grace_geometry.reference.origin.x.0 + grace_geometry.reference.size.width.0;
        let mut safe_point: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(2) {
            for x in (0..=bounds.size.width.0 as i32).step_by(2) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                if pos.x.0 <= reference_right {
                    continue;
                }
                if grace_geometry.reference.contains(pos) || grace_geometry.floating.contains(pos) {
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
            panic!("failed to find safe corridor point; geometry={grace_geometry:?}")
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
                if grace_geometry.reference.contains(pos) || grace_geometry.floating.contains(pos) {
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
                "failed to find unsafe point; safe_point={safe_point:?} geometry={grace_geometry:?}",
            )
        });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: unsafe_point,
                buttons: MouseButtons::default(),
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
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: safe_point,
                buttons: MouseButtons::default(),
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
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::SetTimer { after, .. } if *after == close_delay)),
            "expected safe corridor pointer move to not arm a new close-delay timer; effects={effects:?} safe_point={safe_point:?} close_delay={close_delay:?}"
        );
    }

    #[test]
    fn menubar_close_transition_disables_safe_hover_observers_and_timers() {
        use fret_runtime::Effect;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(700.0), Px(320.0)),
        );

        let underlay_clicked = app.models_mut().insert(false);

        // Frame 1: render and open the "File" menu (so the "More" submenu trigger is visible).
        render_frame_with_underlay_and_submenu(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file_pos = center(menu_trigger_bounds(&snap0, "File"));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: file_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_underlay_and_submenu(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );

        let overlay_id = OverlayController::stack_snapshot_for_window(&ui, &mut app, window)
            .topmost_popover
            .expect("expected an open menubar menu overlay");
        let overlay_root_name = menu::menubar_root_name(overlay_id);
        let overlay_root = fret_ui::elements::global_root(window, &overlay_root_name);
        let overlay_node =
            fret_ui::elements::node_for_element(&mut app, window, overlay_root).expect("overlay");
        let layer = ui.node_layer(overlay_node).expect("overlay layer");

        let open_info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == layer)
            .expect("overlay layer info");
        assert!(open_info.visible);
        assert!(open_info.hit_testable);
        assert!(open_info.wants_pointer_move_events);
        assert!(open_info.wants_timer_events);

        // Close via outside click to enter the close transition (`present=true`, `interactive=false`).
        let underlay_pos = Point::new(Px(10.0), Px(300.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame_with_underlay_and_submenu(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            underlay_clicked.clone(),
        );

        let close_info = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == layer)
            .expect("overlay layer info");
        assert!(close_info.visible);
        assert!(!close_info.hit_testable);
        assert_eq!(
            close_info.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
        assert!(!close_info.wants_pointer_move_events);
        assert!(!close_info.wants_timer_events);

        // Pointer moves during the close transition must not arm submenu timers.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        let cfg = menu::sub::MenuSubmenuConfig::default();
        assert!(
            !effects.iter().any(|e| matches!(
                e,
                Effect::SetTimer { after, .. } if *after == cfg.open_delay || *after == cfg.close_delay
            )),
            "expected close transition pointer move to not arm submenu timers; effects={effects:?}"
        );
    }
}
