//! Material 3 menu.
//!
//! Outcome-oriented implementation:
//! - Token-driven container + list-item colors/sizing via `md.comp.menu.*`.
//! - Roving focus + APG/Base UI-style navigation, including disabled-but-focusable items.
//! - State layer + bounded ripple on items.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Px, Rect, SemanticsCheckedState, SemanticsRole, Size,
    SvgFit, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::{IconId, ids};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableKeyActivation, PressableProps, RovingFlexProps,
    SemanticsDecoration, SemanticsProps, ShadowStyle, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::elements::GlobalElementId;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::primitives::menu as menu_primitive;
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config_in_scope,
};
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::foundation::style_overrides::merge_style_override_slots;
use crate::foundation::surface::material_surface_style;
use crate::foundation::test_id::{
    optional_chrome_part_test_id, optional_part_test_id, part_test_id,
};
use crate::tokens::menu as menu_tokens;

#[derive(Debug, Clone, Copy)]
struct MenuItemLayout {
    one_line_height: Px,
    two_line_height: Px,
    min_width: Px,
    max_width: Px,
    horizontal_padding: Px,
    icon_size: Px,
    slot_gap: Px,
    section_label_height: Px,
    vertical_padding: Px,
    divider_height: Px,
    divider_margin_total: Px,
}

impl MenuItemLayout {
    fn height_for(self, item: &MenuItem) -> Px {
        if item.has_supporting_text() {
            self.two_line_height
        } else {
            self.one_line_height
        }
    }

    fn estimated_panel_height_for_entries(self, entries: &[MenuEntry], max_height: Px) -> Px {
        let h =
            self.vertical_padding.0.max(0.0) * 2.0 + estimated_menu_entries_height(entries, self);
        Px(h.clamp(1.0, max_height.0.max(1.0)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct MenuStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub container_corner_radii: OverrideSlot<Corners>,
    pub container_elevation: OverrideSlot<Px>,
    pub item_min_width: OverrideSlot<Px>,
    pub item_max_width: OverrideSlot<Px>,
    pub item_label_color: OverrideSlot<ColorRef>,
    pub item_icon_color: OverrideSlot<ColorRef>,
    pub item_supporting_text_color: OverrideSlot<ColorRef>,
    pub item_trailing_text_color: OverrideSlot<ColorRef>,
    pub item_state_layer_color: OverrideSlot<ColorRef>,
    pub section_label_color: OverrideSlot<ColorRef>,
    pub item_label_text_style: OverrideSlot<TextStyle>,
    pub item_supporting_text_style: OverrideSlot<TextStyle>,
    pub item_trailing_text_style: OverrideSlot<TextStyle>,
    pub section_label_text_style: OverrideSlot<TextStyle>,
}

impl MenuStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn container_corner_radii(mut self, corners: WidgetStateProperty<Option<Corners>>) -> Self {
        self.container_corner_radii = Some(corners);
        self
    }

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
        self
    }

    pub fn item_min_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.item_min_width = Some(width);
        self
    }

    pub fn item_max_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.item_max_width = Some(width);
        self
    }

    pub fn item_label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.item_label_color = Some(color);
        self
    }

    pub fn item_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.item_icon_color = Some(color);
        self
    }

    pub fn item_supporting_text_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_supporting_text_color = Some(color);
        self
    }

    pub fn item_trailing_text_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_trailing_text_color = Some(color);
        self
    }

    pub fn item_state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.item_state_layer_color = Some(color);
        self
    }

    pub fn section_label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.section_label_color = Some(color);
        self
    }

    pub fn item_label_text_style(mut self, style: WidgetStateProperty<Option<TextStyle>>) -> Self {
        self.item_label_text_style = Some(style);
        self
    }

    pub fn item_supporting_text_style(
        mut self,
        style: WidgetStateProperty<Option<TextStyle>>,
    ) -> Self {
        self.item_supporting_text_style = Some(style);
        self
    }

    pub fn item_trailing_text_style(
        mut self,
        style: WidgetStateProperty<Option<TextStyle>>,
    ) -> Self {
        self.item_trailing_text_style = Some(style);
        self
    }

    pub fn section_label_text_style(
        mut self,
        style: WidgetStateProperty<Option<TextStyle>>,
    ) -> Self {
        self.section_label_text_style = Some(style);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        merge_style_override_slots!(
            self,
            other,
            [
                container_background,
                container_corner_radii,
                container_elevation,
                item_min_width,
                item_max_width,
                item_label_color,
                item_icon_color,
                item_supporting_text_color,
                item_trailing_text_color,
                item_state_layer_color,
                section_label_color,
                item_label_text_style,
                item_supporting_text_style,
                item_trailing_text_style,
                section_label_text_style,
            ]
        )
    }
}

#[derive(Debug, Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Label(MenuLabel),
    Group(MenuGroup),
    Separator,
}

#[derive(Debug, Clone)]
pub struct MenuLabel {
    text: Arc<str>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl MenuLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct MenuGroup {
    pub(crate) entries: Vec<MenuEntry>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl MenuGroup {
    pub fn new(entries: impl IntoIterator<Item = MenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = MenuEntry>) -> Self {
        self.entries = entries.into_iter().collect();
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
}

/// Material 3 submenu trigger helper.
#[derive(Debug, Clone)]
pub struct MenuSubTrigger {
    item: MenuItem,
}

impl MenuSubTrigger {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            item: MenuItem::new(label),
        }
    }

    pub fn refine(mut self, f: impl FnOnce(MenuItem) -> MenuItem) -> Self {
        self.item = f(self.item);
        self
    }
}

/// Material 3 submenu content helper.
#[derive(Debug, Clone)]
pub struct MenuSubContent {
    entries: Vec<MenuEntry>,
}

impl MenuSubContent {
    pub fn new(entries: impl IntoIterator<Item = MenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

/// Material 3 submenu authoring helper.
#[derive(Debug, Clone)]
pub struct MenuSub {
    trigger: MenuSubTrigger,
    content: MenuSubContent,
}

impl MenuSub {
    pub fn new(trigger: MenuSubTrigger, content: MenuSubContent) -> Self {
        Self { trigger, content }
    }

    pub fn into_entry(self) -> MenuEntry {
        let mut item = self.trigger.item;
        item.submenu = Some(self.content.entries);
        MenuEntry::Item(item)
    }
}

impl From<MenuItem> for MenuEntry {
    fn from(value: MenuItem) -> Self {
        Self::Item(value)
    }
}

impl From<MenuLabel> for MenuEntry {
    fn from(value: MenuLabel) -> Self {
        Self::Label(value)
    }
}

impl From<MenuGroup> for MenuEntry {
    fn from(value: MenuGroup) -> Self {
        Self::Group(value)
    }
}

impl From<MenuSub> for MenuEntry {
    fn from(value: MenuSub) -> Self {
        value.into_entry()
    }
}

#[derive(Clone)]
enum MenuItemKind {
    Plain,
    Checkbox {
        checked: Model<bool>,
    },
    Radio {
        selected: Model<Option<Arc<str>>>,
        value: Arc<str>,
    },
}

impl std::fmt::Debug for MenuItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => f.write_str("Plain"),
            Self::Checkbox { .. } => f.write_str("Checkbox"),
            Self::Radio { value, .. } => f.debug_struct("Radio").field("value", value).finish(),
        }
    }
}

impl MenuItemKind {
    fn role(&self) -> SemanticsRole {
        match self {
            Self::Plain => SemanticsRole::MenuItem,
            Self::Checkbox { .. } => SemanticsRole::MenuItemCheckbox,
            Self::Radio { .. } => SemanticsRole::MenuItemRadio,
        }
    }

    fn is_checkable(&self) -> bool {
        !matches!(self, Self::Plain)
    }

    fn checked<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<bool> {
        match self {
            Self::Plain => None,
            Self::Checkbox { checked } => Some(cx.watch_model(checked).layout().copied_or(false)),
            Self::Radio { selected, value } => {
                let current = cx.watch_model(selected).layout().cloned().flatten();
                Some(
                    current
                        .as_ref()
                        .is_some_and(|v| v.as_ref() == value.as_ref()),
                )
            }
        }
    }

    fn activate(
        &self,
        host: &mut dyn fret_ui::action::UiActionHost,
        window: fret_core::AppWindowId,
    ) {
        match self {
            Self::Plain => {}
            Self::Checkbox { checked } => {
                let next = !host.models_mut().get_copied(checked).unwrap_or(false);
                let _ = host.models_mut().update(checked, |value| *value = next);
                host.request_redraw(window);
            }
            Self::Radio { selected, value } => {
                let already_selected = host
                    .models_mut()
                    .read(selected, |current| {
                        current
                            .as_ref()
                            .is_some_and(|selected| selected.as_ref() == value.as_ref())
                    })
                    .ok()
                    .unwrap_or(false);
                if !already_selected {
                    let next = value.clone();
                    let _ = host
                        .models_mut()
                        .update(selected, |current| *current = Some(next.clone()));
                    host.request_redraw(window);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MenuItem {
    label: Arc<str>,
    value: Arc<str>,
    kind: MenuItemKind,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    supporting_text: Option<Arc<str>>,
    shortcut: Option<Arc<str>>,
    pub(crate) submenu: Option<Vec<MenuEntry>>,
    pub(crate) disabled: bool,
    pub(crate) on_select: Option<OnActivate>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("kind", &self.kind)
            .field("leading_icon", &self.leading_icon)
            .field("trailing_icon", &self.trailing_icon)
            .field("supporting_text", &self.supporting_text)
            .field("shortcut", &self.shortcut)
            .field("submenu", &self.submenu)
            .field("disabled", &self.disabled)
            .field("on_select", &self.on_select.is_some())
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl MenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            value: label.clone(),
            label,
            kind: MenuItemKind::Plain,
            leading_icon: None,
            trailing_icon: None,
            supporting_text: None,
            shortcut: None,
            submenu: None,
            disabled: false,
            on_select: None,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn checkbox(checked: Model<bool>, label: impl Into<Arc<str>>) -> Self {
        Self {
            kind: MenuItemKind::Checkbox { checked },
            ..Self::new(label)
        }
    }

    pub fn radio(
        selected: Model<Option<Arc<str>>>,
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind: MenuItemKind::Radio {
                selected,
                value: value.into(),
            },
            ..Self::new(label)
        }
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn shortcut(mut self, text: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    pub fn submenu(mut self, entries: impl IntoIterator<Item = MenuEntry>) -> Self {
        self.submenu = Some(entries.into_iter().collect());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, on_select: OnActivate) -> Self {
        self.on_select = Some(on_select);
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

    pub(crate) fn append_on_select(&mut self, next: OnActivate) {
        let prev = self.on_select.clone();
        self.on_select = Some(Arc::new(move |host, cx, reason| {
            if let Some(prev) = prev.as_ref() {
                prev(host, cx, reason);
            }
            next(host, cx, reason);
        }));
    }

    pub(crate) fn has_supporting_text(&self) -> bool {
        self.supporting_text.is_some()
    }

    fn has_submenu(&self) -> bool {
        self.submenu.is_some()
    }
}

#[derive(Clone)]
pub(crate) struct MaterialMenuSubmenuContext {
    pub(crate) current_models: Option<menu_primitive::sub::MenuSubmenuModels>,
    pub(crate) child_models: menu_primitive::sub::MenuSubmenuModels,
    pub(crate) cfg: menu_primitive::sub::MenuSubmenuConfig,
    pub(crate) outer: Rect,
    pub(crate) submenu_min_width: Px,
    pub(crate) submenu_max_height: Px,
    pub(crate) overlay_root_name: Arc<str>,
    pub(crate) test_id_prefix: Option<Arc<str>>,
}

impl MaterialMenuSubmenuContext {
    pub(crate) fn root(
        child_models: menu_primitive::sub::MenuSubmenuModels,
        cfg: menu_primitive::sub::MenuSubmenuConfig,
        outer: Rect,
        submenu_min_width: Px,
        submenu_max_height: Px,
        overlay_root_name: Arc<str>,
        test_id_prefix: Option<Arc<str>>,
    ) -> Self {
        Self {
            current_models: None,
            child_models,
            cfg,
            outer,
            submenu_min_width,
            submenu_max_height,
            overlay_root_name,
            test_id_prefix,
        }
    }

    fn child(
        &self,
        current_models: menu_primitive::sub::MenuSubmenuModels,
        child_models: menu_primitive::sub::MenuSubmenuModels,
    ) -> Self {
        Self {
            current_models: Some(current_models),
            child_models,
            cfg: self.cfg,
            outer: self.outer,
            submenu_min_width: self.submenu_min_width,
            submenu_max_height: self.submenu_max_height,
            overlay_root_name: self.overlay_root_name.clone(),
            test_id_prefix: self.test_id_prefix.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Menu {
    entries: Vec<MenuEntry>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: MenuStyle,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            a11y_label: None,
            test_id: None,
            style: MenuStyle::default(),
        }
    }

    pub fn entries(mut self, entries: Vec<MenuEntry>) -> Self {
        self.entries = entries;
        self
    }

    pub fn style(mut self, style: MenuStyle) -> Self {
        self.style = self.style.merged(style);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_initial_focus_id(cx, Rc::new(std::cell::Cell::new(None)))
    }

    pub(crate) fn into_element_with_initial_focus_id<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    ) -> AnyElement {
        self.into_element_with_submenu_context(cx, initial_focus_id_out, None)
    }

    pub(crate) fn into_element_with_submenu_context<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
        submenu_ctx: Option<MaterialMenuSubmenuContext>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let Menu {
                entries,
                a11y_label,
                test_id,
                style,
            } = self;
            let (item_layout, vertical_padding, container_bg, shadow, corner) =
                resolve_material_menu_panel(cx, &style);

            let chrome_test_id = test_id.as_ref().map(|id| part_test_id(id, "chrome"));
            let sem = SemanticsProps {
                role: SemanticsRole::Menu,
                label: a11y_label,
                test_id,
                ..Default::default()
            };
            let style: Arc<MenuStyle> = Arc::new(style);

            cx.semantics(sem, move |cx| {
                vec![material_menu_panel_body(
                    cx,
                    entries,
                    item_layout,
                    vertical_padding,
                    container_bg,
                    shadow,
                    corner,
                    chrome_test_id,
                    style,
                    initial_focus_id_out,
                    submenu_ctx,
                )]
            })
        })
    }
}

fn resolve_material_menu_panel<H: UiHost>(
    cx: &ElementContext<'_, H>,
    style: &MenuStyle,
) -> (MenuItemLayout, Px, Color, Option<ShadowStyle>, Corners) {
    let theme = Theme::global(&*cx.app);
    let states = WidgetStates::empty();
    let min_width = resolve_override_slot_with(
        style.item_min_width.as_ref(),
        states,
        |v| *v,
        || menu_tokens::item_min_width(theme),
    );
    let mut max_width = resolve_override_slot_with(
        style.item_max_width.as_ref(),
        states,
        |v| *v,
        || menu_tokens::item_max_width(theme),
    );
    if max_width.0 < min_width.0 {
        max_width = min_width;
    }
    let vertical_padding = menu_tokens::container_vertical_padding(theme);
    let item_layout = MenuItemLayout {
        one_line_height: menu_tokens::list_item_height_for_supporting(theme, false),
        two_line_height: menu_tokens::list_item_height_for_supporting(theme, true),
        min_width,
        max_width,
        horizontal_padding: menu_tokens::item_horizontal_padding(theme),
        icon_size: menu_tokens::item_icon_size(theme),
        slot_gap: menu_tokens::item_slot_gap(theme),
        section_label_height: menu_tokens::section_label_height(theme),
        vertical_padding,
        divider_height: menu_tokens::divider_height(theme),
        divider_margin_total: Px(8.0),
    };

    let container_bg = resolve_override_slot_with(
        style.container_background.as_ref(),
        states,
        |color| color.resolve(theme),
        || menu_tokens::container_background(theme),
    );
    let elevation = resolve_override_slot_with(
        style.container_elevation.as_ref(),
        states,
        |v| *v,
        || menu_tokens::container_elevation(theme),
    );
    let shadow_color = menu_tokens::container_shadow_color(theme);
    let corner = resolve_override_slot_with(
        style.container_corner_radii.as_ref(),
        states,
        |v| *v,
        || menu_tokens::container_shape(theme),
    );
    let surface =
        material_surface_style(theme, container_bg, elevation, Some(shadow_color), corner);
    (
        item_layout,
        vertical_padding,
        surface.background,
        surface.shadow,
        corner,
    )
}

#[allow(clippy::too_many_arguments)]
fn material_menu_panel_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Vec<MenuEntry>,
    item_layout: MenuItemLayout,
    vertical_padding: Px,
    container_bg: Color,
    shadow: Option<ShadowStyle>,
    corner: Corners,
    chrome_test_id: Option<Arc<str>>,
    style: Arc<MenuStyle>,
    initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    submenu_ctx: Option<MaterialMenuSubmenuContext>,
) -> AnyElement {
    let mut disabled: Vec<bool> = Vec::new();
    let mut typeahead_items: Vec<Arc<str>> = Vec::new();
    collect_menu_roving_metadata(&items, &mut disabled, &mut typeahead_items);

    let count = disabled.len();
    let roving_disabled: Arc<[bool]> = Arc::from(vec![false; count]);
    let typeahead_items: Arc<[Arc<str>]> = Arc::from(typeahead_items);

    let mut roving = RovingFlexProps::default();
    roving.flex.direction = Axis::Vertical;
    roving.flex.gap = Px(0.0).into();
    roving.flex.align = CrossAlign::Stretch;
    roving.flex.justify = MainAlign::Start;
    roving.flex.layout.size.width = Length::Auto;
    roving.flex.layout.size.min_width = Some(Length::Px(item_layout.min_width));
    roving.flex.layout.size.max_width = Some(Length::Px(item_layout.max_width));
    roving.roving = fret_ui::element::RovingFocusProps {
        enabled: true,
        wrap: true,
        disabled: roving_disabled,
    };

    cx.container(
        ContainerProps {
            background: Some(container_bg),
            shadow,
            corner_radii: corner,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Auto;
                l.size.min_width = Some(Length::Px(item_layout.min_width));
                l.size.max_width = Some(Length::Px(item_layout.max_width));
                l.overflow = Overflow::Clip;
                l
            },
            padding: Edges {
                left: Px(0.0),
                right: Px(0.0),
                top: vertical_padding,
                bottom: vertical_padding,
            }
            .into(),
            ..Default::default()
        },
        move |cx| {
            let mut children = Vec::new();
            if let Some(test_id) = chrome_test_id.clone() {
                children.push(absolute_fill_test_id_marker(cx, test_id));
            }
            children.push(cx.roving_flex(roving, move |cx| {
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use fret_ui::action::RovingNavigateResult;

                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    let forward = match it.key {
                        KeyCode::ArrowDown => Some(true),
                        KeyCode::ArrowUp => Some(false),
                        _ => None,
                    };

                    if it.key == KeyCode::Home {
                        let target = (0..it.len).find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }
                    if it.key == KeyCode::End {
                        let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }

                    let Some(forward) = forward else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let current = it
                        .current
                        .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                    let Some(current) = current else {
                        return RovingNavigateResult::Handled { target: None };
                    };

                    let len = it.len;
                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));

                roving_typeahead_prefix_arc_str_always_wrap(cx, typeahead_items.clone(), 30);

                let mut item_idx = 0usize;
                render_menu_entries(
                    cx,
                    &items,
                    item_layout,
                    style.clone(),
                    &mut item_idx,
                    count,
                    initial_focus_id_out.clone(),
                    submenu_ctx,
                )
            }));
            children
        },
    )
}

fn estimated_menu_entries_height(entries: &[MenuEntry], layout: MenuItemLayout) -> f32 {
    let mut h = 0.0;
    for entry in entries {
        match entry {
            MenuEntry::Item(item) => h += layout.height_for(item).0.max(0.0),
            MenuEntry::Label(_) => h += layout.section_label_height.0.max(0.0),
            MenuEntry::Group(group) => {
                h += estimated_menu_entries_height(&group.entries, layout);
            }
            MenuEntry::Separator => {
                h += layout.divider_height.0.max(0.0) + layout.divider_margin_total.0.max(0.0);
            }
        }
    }
    h
}

pub(crate) fn menu_submenu_entries_by_value(
    entries: &[MenuEntry],
    open_value: &str,
) -> Option<Vec<MenuEntry>> {
    for entry in entries {
        match entry {
            MenuEntry::Item(item) => {
                if item.value.as_ref() == open_value {
                    return item.submenu.clone();
                }
                if let Some(submenu) = item.submenu.as_deref()
                    && let Some(found) = menu_submenu_entries_by_value(submenu, open_value)
                {
                    return Some(found);
                }
            }
            MenuEntry::Group(group) => {
                if let Some(found) = menu_submenu_entries_by_value(&group.entries, open_value) {
                    return Some(found);
                }
            }
            MenuEntry::Label(_) | MenuEntry::Separator => {}
        }
    }
    None
}

fn material_menu_test_id_slug(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "submenu".to_owned()
    } else {
        out
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn material_menu_submenu_panel_tree<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    entries: Vec<MenuEntry>,
    open_value: Arc<str>,
    geometry: menu_primitive::sub::MenuSubmenuGeometry,
    current_models: menu_primitive::sub::MenuSubmenuModels,
    style: MenuStyle,
    submenu_ctx: MaterialMenuSubmenuContext,
) -> AnyElement {
    let labelled_by_element = cx
        .app
        .models_mut()
        .read(&current_models.trigger, |v| *v)
        .ok()
        .flatten();

    let panel_width = geometry.floating.size.width;
    let panel_style = style
        .clone()
        .item_min_width(WidgetStateProperty::new(Some(panel_width)))
        .item_max_width(WidgetStateProperty::new(Some(panel_width)));
    let panel_test_id = submenu_ctx.test_id_prefix.as_ref().map(|prefix| {
        Arc::<str>::from(format!(
            "{prefix}.submenu-{}",
            material_menu_test_id_slug(open_value.as_ref())
        ))
    });

    let nested_entries_cell: Rc<RefCell<Option<Vec<MenuEntry>>>> = Rc::new(RefCell::new(None));
    let nested_entries_cell_for_panel = nested_entries_cell.clone();
    let nested_models_cell: Rc<RefCell<Option<menu_primitive::sub::MenuSubmenuModels>>> =
        Rc::new(RefCell::new(None));
    let nested_models_cell_for_panel = nested_models_cell.clone();
    let current_models_for_panel = current_models.clone();
    let submenu_ctx_for_panel = submenu_ctx.clone();
    let panel_style_for_panel = panel_style.clone();
    let entries_for_panel = entries.clone();

    let mut panel = menu_primitive::sub_content::submenu_panel_scroll_y_for_value_at(
        cx,
        open_value.clone(),
        geometry.floating,
        labelled_by_element,
        |layout| ContainerProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let child_models = menu_primitive::root::sync_root_open_and_ensure_submenu(
                cx,
                true,
                cx.root_id(),
                submenu_ctx_for_panel.cfg,
            );
            cx.dismissible_add_on_pointer_move(menu_primitive::root::submenu_pointer_move_handler(
                child_models.clone(),
                submenu_ctx_for_panel.cfg,
            ));
            *nested_models_cell_for_panel.borrow_mut() = Some(child_models.clone());

            let child_open_value = cx
                .app
                .models_mut()
                .read(&child_models.open_value, |v| v.clone())
                .ok()
                .flatten();
            let child_entries = child_open_value.as_deref().and_then(|open_value| {
                menu_submenu_entries_by_value(&entries_for_panel, open_value)
            });
            *nested_entries_cell_for_panel.borrow_mut() = child_entries;

            let child_ctx =
                submenu_ctx_for_panel.child(current_models_for_panel.clone(), child_models);
            let (item_layout, vertical_padding, container_bg, shadow, corner) =
                resolve_material_menu_panel(cx, &panel_style_for_panel);
            vec![material_menu_panel_body(
                cx,
                entries_for_panel,
                item_layout,
                vertical_padding,
                container_bg,
                shadow,
                corner,
                None,
                Arc::new(panel_style_for_panel),
                Rc::new(std::cell::Cell::new(None)),
                Some(child_ctx),
            )]
        },
    );

    if let Some(test_id) = panel_test_id {
        panel = panel.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let mut children = vec![panel];

    if let Some(child_models) = nested_models_cell.borrow().clone() {
        let child_open_value = cx
            .watch_model(&child_models.open_value)
            .layout()
            .cloned()
            .unwrap_or(None);
        if child_open_value.is_some() {
            let child_entries = nested_entries_cell.borrow().clone();
            let desired = child_entries
                .as_ref()
                .map(|entries| {
                    let (layout, _, _, _, _) = resolve_material_menu_panel(cx, &style);
                    let desired_h = layout.estimated_panel_height_for_entries(
                        entries,
                        submenu_ctx.submenu_max_height,
                    );
                    Size::new(submenu_ctx.submenu_min_width, desired_h)
                })
                .unwrap_or_else(|| {
                    Size::new(
                        submenu_ctx.submenu_min_width,
                        submenu_ctx.submenu_max_height,
                    )
                });
            let open_child = menu_primitive::sub::with_open_submenu_synced(
                cx,
                &child_models,
                submenu_ctx.outer,
                desired,
                |_cx, open_value, geometry| (open_value, geometry),
            );
            if let (Some((open_value, geometry)), Some(entries)) = (open_child, child_entries) {
                children.push(material_menu_submenu_panel_tree(
                    cx,
                    entries,
                    open_value,
                    geometry,
                    child_models,
                    style,
                    submenu_ctx,
                ));
            }
        }
    }

    if children.len() == 1 {
        children.pop().expect("submenu panel")
    } else {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout.overflow = Overflow::Visible;
                    layout
                },
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

fn collect_menu_roving_metadata(
    entries: &[MenuEntry],
    disabled: &mut Vec<bool>,
    typeahead_items: &mut Vec<Arc<str>>,
) {
    for entry in entries {
        match entry {
            MenuEntry::Item(item) => {
                disabled.push(item.disabled);
                typeahead_items.push(
                    item.a11y_label
                        .clone()
                        .unwrap_or_else(|| item.label.clone()),
                );
            }
            MenuEntry::Group(group) => {
                collect_menu_roving_metadata(&group.entries, disabled, typeahead_items);
            }
            MenuEntry::Label(_) | MenuEntry::Separator => {}
        }
    }
}

fn render_menu_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    entries: &[MenuEntry],
    item_layout: MenuItemLayout,
    style: Arc<MenuStyle>,
    item_idx: &mut usize,
    item_count: usize,
    initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    submenu_ctx: Option<MaterialMenuSubmenuContext>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::with_capacity(entries.len());
    for entry in entries {
        match entry {
            MenuEntry::Separator => {
                out.push(menu_separator(cx));
            }
            MenuEntry::Label(label) => {
                out.push(material_menu_label(
                    cx,
                    label.clone(),
                    item_layout,
                    style.clone(),
                ));
            }
            MenuEntry::Group(group) => {
                out.push(material_menu_group(
                    cx,
                    group.clone(),
                    item_layout,
                    style.clone(),
                    item_idx,
                    item_count,
                    initial_focus_id_out.clone(),
                    submenu_ctx.clone(),
                ));
            }
            MenuEntry::Item(it) => {
                let tab_stop = *item_idx == 0;
                out.push(material_menu_item(
                    cx,
                    it.clone(),
                    item_layout,
                    style.clone(),
                    tab_stop,
                    *item_idx,
                    item_count,
                    initial_focus_id_out.clone(),
                    submenu_ctx.clone(),
                ));
                *item_idx += 1;
            }
        }
    }
    out
}

fn absolute_fill_test_id_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: Arc<str>,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.inset = fret_ui::element::InsetStyle {
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };

    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(test_id),
            layout,
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    )
}

fn menu_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (h, c) = {
        let theme = Theme::global(&*cx.app);
        (
            menu_tokens::divider_height(theme),
            menu_tokens::divider_color(theme),
        )
    };

    let mut props = ContainerProps::default();
    props.background = Some(c);
    props.layout.size.height = Length::Px(h);
    props.layout.size.width = Length::Fill;
    props.layout.margin.top = fret_ui::element::MarginEdge::Px(Px(4.0));
    props.layout.margin.bottom = fret_ui::element::MarginEdge::Px(Px(4.0));
    cx.container(props, |_cx| vec![])
}

fn material_menu_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group: MenuGroup,
    layout: MenuItemLayout,
    style: Arc<MenuStyle>,
    item_idx: &mut usize,
    item_count: usize,
    initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    submenu_ctx: Option<MaterialMenuSubmenuContext>,
) -> AnyElement {
    let children = render_menu_entries(
        cx,
        &group.entries,
        layout,
        style,
        item_idx,
        item_count,
        initial_focus_id_out,
        submenu_ctx,
    );

    let mut group_layout = fret_ui::element::LayoutStyle::default();
    group_layout.size.width = Length::Auto;
    group_layout.size.min_width = Some(Length::Px(layout.min_width));
    group_layout.size.max_width = Some(Length::Px(layout.max_width));

    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Group,
            label: group.a11y_label,
            test_id: group.test_id,
            layout: group_layout,
            ..Default::default()
        },
        move |_cx| children,
    )
}

fn material_menu_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: MenuLabel,
    layout: MenuItemLayout,
    style: Arc<MenuStyle>,
) -> AnyElement {
    let text_test_id = optional_part_test_id(label.test_id.as_ref(), "text");
    let states = WidgetStates::empty();
    let (label_color, label_style) = {
        let theme = Theme::global(&*cx.app);
        let color = resolve_override_slot_with(
            style.section_label_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || menu_tokens::section_label_color(theme),
        );
        let text_style = resolve_override_slot_with(
            style.section_label_text_style.as_ref(),
            states,
            |style| style.clone(),
            || menu_tokens::section_label_text_style(theme),
        );
        (color, text_style)
    };

    let text = menu_item_label(
        cx,
        &label.text,
        label_style,
        label_color,
        text_test_id,
        true,
    );

    let mut row = FlexProps::default();
    row.layout.size.width = Length::Auto;
    row.layout.size.height = Length::Px(layout.section_label_height);
    row.layout.size.min_width = Some(Length::Px(layout.min_width));
    row.layout.size.max_width = Some(Length::Px(layout.max_width));
    row.layout.overflow = Overflow::Clip;
    row.direction = Axis::Horizontal;
    row.justify = MainAlign::Start;
    row.align = CrossAlign::Center;
    row.padding = Edges {
        left: layout.horizontal_padding,
        right: layout.horizontal_padding,
        top: Px(0.0),
        bottom: Px(0.0),
    }
    .into();

    let a11y_label = label
        .a11y_label
        .clone()
        .unwrap_or_else(|| label.text.clone());
    let mut el = cx.flex(row, move |_cx| vec![text]);
    if let Some(test_id) = label.test_id.clone() {
        el = el.test_id(test_id);
    }
    el.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Text)
            .label(a11y_label),
    )
}

fn material_menu_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item: MenuItem,
    layout: MenuItemLayout,
    style: Arc<MenuStyle>,
    tab_stop: bool,
    idx: usize,
    set_size: usize,
    initial_focus_id_out: Rc<std::cell::Cell<Option<GlobalElementId>>>,
    submenu_ctx: Option<MaterialMenuSubmenuContext>,
) -> AnyElement {
    let chrome_test_id = optional_chrome_part_test_id(item.test_id.as_ref());
    let leading_icon_test_id = optional_part_test_id(item.test_id.as_ref(), "leading-icon");
    let label_test_id = optional_part_test_id(item.test_id.as_ref(), "label");
    let supporting_text_test_id = optional_part_test_id(item.test_id.as_ref(), "supporting-text");
    let shortcut_test_id = optional_part_test_id(item.test_id.as_ref(), "shortcut");
    let trailing_icon_test_id = optional_part_test_id(item.test_id.as_ref(), "trailing-icon");
    let submenu_chevron_test_id = optional_part_test_id(item.test_id.as_ref(), "submenu-chevron");
    let item_disabled = item.disabled;

    let mut element = cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !item.disabled;
        let item_height = layout.height_for(&item);
        let has_submenu = item.has_submenu();
        let checked = (!has_submenu).then(|| item.kind.checked(cx)).flatten();

        if tab_stop && initial_focus_id_out.get().is_none() {
            initial_focus_id_out.set(Some(pressable_id));
        }

        let mut is_open_submenu = false;
        if let Some(submenu_ctx) = submenu_ctx.as_ref() {
            if let Some(current_models) = submenu_ctx.current_models.as_ref() {
                menu_primitive::sub_content::wire_item(
                    cx,
                    pressable_id,
                    item.disabled,
                    current_models,
                );
            }

            let geometry_hint = has_submenu.then(|| {
                let submenu_max_height = Px(submenu_ctx
                    .submenu_max_height
                    .0
                    .min(submenu_ctx.outer.size.height.0));
                let desired_h = item
                    .submenu
                    .as_deref()
                    .map(|entries| {
                        layout.estimated_panel_height_for_entries(entries, submenu_max_height)
                    })
                    .unwrap_or(submenu_max_height);
                menu_primitive::sub_trigger::MenuSubTriggerGeometryHint {
                    outer: submenu_ctx.outer,
                    desired: Size::new(submenu_ctx.submenu_min_width, desired_h),
                }
            });

            is_open_submenu = menu_primitive::sub_trigger::wire(
                cx,
                st,
                pressable_id,
                item.disabled,
                has_submenu,
                item.value.clone(),
                &submenu_ctx.child_models,
                submenu_ctx.cfg,
                geometry_hint,
            )
            .unwrap_or(false);
        }

        let controls_element = if has_submenu {
            submenu_ctx.as_ref().map(|submenu_ctx| {
                menu_primitive::sub_content::submenu_content_semantics_id(
                    cx,
                    submenu_ctx.overlay_root_name.as_ref(),
                    &item.value,
                )
            })
        } else {
            None
        };

        let a11y = PressableA11y {
            role: Some(if has_submenu {
                SemanticsRole::MenuItem
            } else {
                item.kind.role()
            }),
            label: item.a11y_label.clone().or_else(|| Some(item.label.clone())),
            test_id: item.test_id.clone(),
            checked,
            checked_state: checked.map(|value| {
                if value {
                    SemanticsCheckedState::True
                } else {
                    SemanticsCheckedState::False
                }
            }),
            expanded: has_submenu.then_some(is_open_submenu),
            controls_element: controls_element.map(|id| id.0),
            pos_in_set: Some((idx + 1) as u32),
            set_size: Some(set_size as u32),
            ..Default::default()
        };

        if enabled && !has_submenu {
            if item.kind.is_checkable() {
                let kind = item.kind.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                    kind.activate(host, action_cx.window);
                }));
            }
            if let Some(handler) = item.on_select.clone() {
                cx.pressable_add_on_activate(handler);
            }
        }

        let pressable_props = PressableProps {
            enabled: true,
            focusable: tab_stop,
            key_activation: if enabled {
                PressableKeyActivation::EnterAndSpace
            } else {
                PressableKeyActivation::None
            },
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Auto;
                l.size.height = Length::Px(item_height);
                l.size.min_width = Some(Length::Px(layout.min_width));
                l.size.max_width = Some(Length::Px(layout.max_width));
                l.overflow = Overflow::Visible;
                {
                    let theme = Theme::global(&*cx.app);
                    enforce_minimum_interactive_size(&mut l, theme);
                }
                l
            },
            focus_ring: None,
            focus_ring_always_paint: false,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Auto;
            props.layout.size.height = Length::Fill;
            props.layout.size.min_width = Some(Length::Px(layout.min_width));
            props.layout.size.max_width = Some(Length::Px(layout.max_width));
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && (st.hovered || is_open_submenu);
                let is_focused = enabled && st.focused && focus_visible;

                let interaction = if is_pressed {
                    menu_tokens::MenuItemInteraction::Pressed
                } else if is_focused {
                    menu_tokens::MenuItemInteraction::Focused
                } else if is_hovered {
                    menu_tokens::MenuItemInteraction::Hovered
                } else {
                    menu_tokens::MenuItemInteraction::Default
                };

                let mut states = WidgetStates::from_pressable(cx, st, enabled);
                states.set(WidgetState::Selected, checked == Some(true));
                let (
                    label_color,
                    icon_color,
                    supporting_text_color,
                    trailing_text_color,
                    state_layer_color,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    label_style,
                    supporting_text_style,
                    trailing_text_style,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let (token_label_color, token_state_layer_color, state_layer_target) =
                        menu_tokens::item_outcomes(theme, enabled, interaction);
                    let label_color = resolve_override_slot_with(
                        style.item_label_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || token_label_color,
                    );
                    let icon_color = resolve_override_slot_with(
                        style.item_icon_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || menu_tokens::item_icon_color(theme, enabled),
                    );
                    let supporting_text_color = resolve_override_slot_with(
                        style.item_supporting_text_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || menu_tokens::item_supporting_text_color(theme, enabled),
                    );
                    let trailing_text_color = resolve_override_slot_with(
                        style.item_trailing_text_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || menu_tokens::item_trailing_text_color(theme, enabled),
                    );
                    let state_layer_color = resolve_override_slot_with(
                        style.item_state_layer_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || token_state_layer_color,
                    );

                    let ripple_base_opacity = menu_tokens::pressed_state_layer_opacity(theme);
                    let config = material_pressable_indication_config_in_scope(&*cx, None);

                    let default_label_style = menu_tokens::item_label_text_style(theme);
                    let label_style = resolve_override_slot_with(
                        style.item_label_text_style.as_ref(),
                        states,
                        |s| s.clone(),
                        || default_label_style,
                    );
                    let default_supporting_text_style =
                        menu_tokens::item_supporting_text_style(theme);
                    let supporting_text_style = resolve_override_slot_with(
                        style.item_supporting_text_style.as_ref(),
                        states,
                        |s| s.clone(),
                        || default_supporting_text_style,
                    );
                    let default_trailing_text_style = menu_tokens::item_trailing_text_style(theme);
                    let trailing_text_style = resolve_override_slot_with(
                        style.item_trailing_text_style.as_ref(),
                        states,
                        |s| s.clone(),
                        || default_trailing_text_style,
                    );

                    (
                        label_color,
                        icon_color,
                        supporting_text_color,
                        trailing_text_color,
                        state_layer_color,
                        state_layer_target,
                        ripple_base_opacity,
                        config,
                        label_style,
                        supporting_text_style,
                        trailing_text_style,
                    )
                };
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    Corners::all(Px(0.0)),
                    RippleClip::Bounded,
                    state_layer_color,
                    is_pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );
                let label_el = menu_item_label(
                    cx,
                    &item.label,
                    label_style,
                    label_color,
                    label_test_id.clone(),
                    true,
                );
                let text_body = if let Some(supporting_text) = item.supporting_text.clone() {
                    let supporting_el = menu_item_label(
                        cx,
                        &supporting_text,
                        supporting_text_style,
                        supporting_text_color,
                        supporting_text_test_id.clone(),
                        true,
                    );
                    menu_item_text_body(cx, label_el, Some(supporting_el))
                } else {
                    label_el
                };

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Auto;
                row.layout.size.height = Length::Px(item_height);
                row.layout.size.min_width = Some(Length::Px(layout.min_width));
                row.layout.size.max_width = Some(Length::Px(layout.max_width));
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Start;
                row.align = CrossAlign::Center;
                row.gap = layout.slot_gap.into();
                row.padding = Edges {
                    left: layout.horizontal_padding,
                    right: layout.horizontal_padding,
                    top: Px(0.0),
                    bottom: Px(0.0),
                }
                .into();

                let leading_icon = item.leading_icon.clone().or_else(|| {
                    (item.kind.is_checkable() && checked == Some(true)).then_some(ids::ui::CHECK)
                });
                let reserve_leading = item.kind.is_checkable() || leading_icon.is_some();
                let direction = crate::foundation::context::material_layout_direction_in_scope(cx);
                let submenu_chevron = match direction {
                    fret_ui::overlay_placement::LayoutDirection::Rtl => ids::ui::CHEVRON_LEFT,
                    fret_ui::overlay_placement::LayoutDirection::Ltr => ids::ui::CHEVRON_RIGHT,
                };
                let trailing_icon = item
                    .trailing_icon
                    .clone()
                    .or_else(|| has_submenu.then_some(submenu_chevron));
                let trailing_icon_test_id = if has_submenu && item.trailing_icon.is_none() {
                    submenu_chevron_test_id.clone()
                } else {
                    trailing_icon_test_id.clone()
                };
                let shortcut = item.shortcut.clone();

                let mut chrome = cx.flex(row, move |cx| {
                    let mut children = vec![overlay];
                    if reserve_leading {
                        children.push(menu_item_icon_slot(
                            cx,
                            leading_icon.clone(),
                            layout.icon_size,
                            icon_color,
                            leading_icon_test_id.clone(),
                        ));
                    }
                    children.push(text_body);
                    if let Some(shortcut) = shortcut.clone() {
                        children.push(menu_item_label(
                            cx,
                            &shortcut,
                            trailing_text_style,
                            trailing_text_color,
                            shortcut_test_id.clone(),
                            false,
                        ));
                    }
                    if let Some(icon) = trailing_icon.clone() {
                        children.push(menu_item_icon_slot(
                            cx,
                            Some(icon),
                            layout.icon_size,
                            icon_color,
                            trailing_icon_test_id.clone(),
                        ));
                    }
                    children
                });
                if let Some(test_id) = chrome_test_id.clone() {
                    chrome = chrome.test_id(test_id);
                }
                vec![chrome]
            })
        });

        (pressable_props, vec![pointer_region])
    });

    if item_disabled {
        element = element.attach_semantics(
            SemanticsDecoration::default()
                .disabled(true)
                .invokable(false),
        );
    }

    element
}

fn menu_item_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &Arc<str>,
    style: TextStyle,
    color: Color,
    test_id: Option<Arc<str>>,
    fill: bool,
) -> AnyElement {
    let mut props = TextProps::new(text.clone());
    props.style = Some(style);
    props.color = Some(color);
    props.layout.size.width = Length::Auto;
    props.layout.size.min_width = Some(Length::Px(Px(0.0)));
    if fill {
        props.layout.flex.grow = 1.0;
        props.layout.flex.basis = Length::Px(Px(0.0));
    } else {
        props.layout.flex.shrink = 1.0;
    }
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;
    let mut text = cx.text_props(props);
    if let Some(test_id) = test_id {
        text = text.test_id(test_id);
    }
    text
}

fn menu_item_text_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: AnyElement,
    supporting: Option<AnyElement>,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.direction = Axis::Vertical;
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Stretch;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_width = Some(Length::Px(Px(0.0)));
    props.layout.flex.grow = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));

    cx.flex(props, move |_cx| {
        let mut children = vec![label];
        if let Some(supporting) = supporting {
            children.push(supporting);
        }
        children
    })
}

fn menu_item_icon_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: Option<IconId>,
    size: Px,
    color: Color,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.layout.flex.shrink = 0.0;

    let mut slot = cx.container(props, move |cx| {
        icon.as_ref()
            .map(|icon| vec![menu_item_icon(cx, icon, size, color)])
            .unwrap_or_default()
    });
    if let Some(test_id) = test_id {
        slot = slot.test_id(test_id);
    }
    slot
}

fn menu_item_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}

fn roving_typeahead_prefix_arc_str_always_wrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
) {
    use fret_ui::action::{ActionCx, OnRovingTypeahead, RovingTypeaheadCx, UiActionHost};

    #[derive(Debug, Default)]
    struct TypeaheadBuffer {
        timeout_ticks: u64,
        last_tick: Option<u64>,
        query: String,
    }

    impl TypeaheadBuffer {
        fn new(timeout_ticks: u64) -> Self {
            Self {
                timeout_ticks,
                last_tick: None,
                query: String::new(),
            }
        }

        fn push_char(&mut self, ch: char, tick: u64) {
            if ch.is_whitespace() {
                return;
            }
            let expired = self
                .last_tick
                .is_some_and(|last| tick.saturating_sub(last) > self.timeout_ticks);
            if expired {
                self.query.clear();
            }
            self.last_tick = Some(tick);
            self.query.extend(ch.to_lowercase());
        }

        fn active_query(&mut self, tick: u64) -> Option<&str> {
            let expired = self
                .last_tick
                .is_some_and(|last| tick.saturating_sub(last) > self.timeout_ticks);
            if expired {
                self.query.clear();
                self.last_tick = None;
                return None;
            }
            if self.query.is_empty() {
                None
            } else {
                Some(self.query.as_str())
            }
        }
    }

    let buffer: Rc<RefCell<TypeaheadBuffer>> =
        Rc::new(RefCell::new(TypeaheadBuffer::new(timeout_ticks)));
    let handler: OnRovingTypeahead = Arc::new(
        move |_host: &mut dyn UiActionHost, _cx: ActionCx, it: RovingTypeaheadCx| {
            let tick = it.tick;
            let ch = it.input;

            let mut buf = buffer.borrow_mut();
            buf.push_char(ch, tick);
            let query = buf.active_query(tick)?;

            let current = it.current.unwrap_or(0);
            let mut matches: Vec<usize> = labels
                .iter()
                .enumerate()
                .filter_map(|(idx, label)| {
                    let label = label.to_lowercase();
                    label.starts_with(query).then_some(idx)
                })
                .collect();
            if matches.is_empty() {
                return None;
            }

            // If query is a single character, skip the current match to allow cycling.
            if query.chars().count() == 1 {
                matches.retain(|&idx| idx != current);
                if matches.is_empty() {
                    return None;
                }
            }

            // Always wrap: prefer the next match after current, otherwise the first.

            matches
                .iter()
                .copied()
                .find(|&idx| idx > current)
                .or_else(|| matches.into_iter().next())
        },
    );

    cx.roving_add_on_typeahead(handler);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{Point, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(220.0), Px(160.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    #[test]
    fn menu_item_labels_can_shrink_within_menu_rows() {
        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        let label =
            Arc::<str>::from("A very long menu item label that should shrink within the menu row");

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "m3-menu", |cx| {
            Menu::new()
                .entries(vec![MenuEntry::Item(MenuItem::new(label.clone()))])
                .into_element(cx)
        });

        let label = find_text_by_content(&el, label.as_ref()).expect("menu item label text");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.size.width, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.flex.grow, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Px(Px(0.0)));
    }
}
