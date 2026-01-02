use std::sync::Arc;
use std::time::Duration;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::safe_hover::safe_hover_contains;
use fret_components_ui::overlay;
use fret_components_ui::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};
use fret_core::{
    Edges, FontId, FontWeight, KeyCode, Point, Px, Rect, SemanticsRole, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Effect, Model, TimerToken};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PointerRegionProps, PositionStyle, PressableA11y, PressableProps, RovingFlexProps,
    RovingFocusProps, ScrollAxis, ScrollProps, SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementContext, Theme, UiHost};

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
    Label(DropdownMenuLabel),
    Group(DropdownMenuGroup),
    Separator,
}

#[derive(Debug, Clone)]
pub struct DropdownMenuItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub disabled: bool,
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
            disabled: false,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
}

impl DropdownMenuLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
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

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metrics.font_size,
                weight: FontWeight::NORMAL,
                line_height: Some(theme.metrics.font_line_height),
                letter_spacing_em: Some(0.12),
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
            other => into.push(other),
        }
    }
}

#[derive(Default)]
struct DropdownMenuSubmenuState {
    open_value: Option<Model<Option<Arc<str>>>>,
    trigger: Option<Model<Option<fret_ui::GlobalElementId>>>,
    last_pointer: Option<Model<Option<Point>>>,
    geometry: Option<Model<Option<DropdownMenuSubmenuGeometry>>>,
    close_timer: Option<Model<Option<TimerToken>>>,
    was_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DropdownMenuSubmenuGeometry {
    reference: Rect,
    floating: Rect,
}

const SUBMENU_SAFE_HOVER_BUFFER: Px = Px(6.0);
const SUBMENU_CLOSE_DELAY: Duration = Duration::from_millis(120);

/// shadcn/ui `Dropdown Menu` (v4).
///
/// This is a dismissible popover overlay (non-modal) backed by the component-layer overlay
/// manager (`fret-components-ui/overlay_controller.rs`).
#[derive(Clone)]
pub struct DropdownMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    min_width: Px,
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            let was_open = cx.with_state(DropdownMenuSubmenuState::default, |st| st.was_open);
            if is_open && !was_open {
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.open_value.clone())
                {
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.trigger.clone())
                {
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.last_pointer.clone())
                {
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.geometry.clone())
                {
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.close_timer.clone())
                {
                    let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
                    if let Some(token) = token {
                        cx.app.push_effect(Effect::CancelTimer { token });
                    }
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                cx.with_state(DropdownMenuSubmenuState::default, |st| st.was_open = true);
            } else if !is_open && was_open {
                if let Some(model) =
                    cx.with_state(DropdownMenuSubmenuState::default, |st| st.close_timer.clone())
                {
                    let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
                    if let Some(token) = token {
                        cx.app.push_effect(Effect::CancelTimer { token });
                    }
                    let _ = cx.app.models_mut().update(&model, |v| *v = None);
                }
                cx.with_state(DropdownMenuSubmenuState::default, |st| st.was_open = false);
            }

            if is_open {
                let overlay_root_name = OverlayController::popover_root_name(trigger_id);

                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let open = self.open;
                let open_for_overlay = open.clone();
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let min_width = self.min_width;

                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let theme = &theme;
                    let anchor = overlay::anchor_bounds_for_element(cx, trigger_id);
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };

                    let mut flat: Vec<DropdownMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, entries(cx));
                    let entries = flat;

                    let submenu_open =
                        cx.with_state(DropdownMenuSubmenuState::default, |st| st.open_value.clone());
                    let submenu_open = if let Some(submenu_open) = submenu_open {
                        submenu_open
                    } else {
                        let submenu_open = cx.app.models_mut().insert(None);
                        cx.with_state(DropdownMenuSubmenuState::default, |st| {
                            st.open_value = Some(submenu_open.clone());
                        });
                        submenu_open
                    };

                    let submenu_trigger =
                        cx.with_state(DropdownMenuSubmenuState::default, |st| st.trigger.clone());
                    let submenu_trigger = if let Some(submenu_trigger) = submenu_trigger {
                        submenu_trigger
                    } else {
                        let submenu_trigger = cx.app.models_mut().insert(None);
                        cx.with_state(DropdownMenuSubmenuState::default, |st| {
                            st.trigger = Some(submenu_trigger.clone());
                        });
                        submenu_trigger
                    };

                    let submenu_last_pointer =
                        cx.with_state(DropdownMenuSubmenuState::default, |st| st.last_pointer.clone());
                    let submenu_last_pointer = if let Some(submenu_last_pointer) = submenu_last_pointer {
                        submenu_last_pointer
                    } else {
                        let submenu_last_pointer = cx.app.models_mut().insert(None);
                        cx.with_state(DropdownMenuSubmenuState::default, |st| {
                            st.last_pointer = Some(submenu_last_pointer.clone());
                        });
                        submenu_last_pointer
                    };

                    let submenu_geometry =
                        cx.with_state(DropdownMenuSubmenuState::default, |st| st.geometry.clone());
                    let submenu_geometry = if let Some(submenu_geometry) = submenu_geometry {
                        submenu_geometry
                    } else {
                        let submenu_geometry = cx.app.models_mut().insert(None);
                        cx.with_state(DropdownMenuSubmenuState::default, |st| {
                            st.geometry = Some(submenu_geometry.clone());
                        });
                        submenu_geometry
                    };

                    let submenu_close_timer =
                        cx.with_state(DropdownMenuSubmenuState::default, |st| st.close_timer.clone());
                    let submenu_close_timer = if let Some(submenu_close_timer) = submenu_close_timer {
                        submenu_close_timer
                    } else {
                        let submenu_close_timer = cx.app.models_mut().insert(None);
                        cx.with_state(DropdownMenuSubmenuState::default, |st| {
                            st.close_timer = Some(submenu_close_timer.clone());
                        });
                        submenu_close_timer
                    };

                    let item_count = entries
                        .iter()
                        .filter(|e| matches!(e, DropdownMenuEntry::Item(_)))
                        .count();
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                        .iter()
                        .map(|e| match e {
                            DropdownMenuEntry::Item(item) => (item.label.clone(), item.disabled),
                            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {
                                (Arc::from(""), true)
                            }
                            DropdownMenuEntry::Group(_) => unreachable!("groups are flattened"),
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

                    let placed =
                        anchored_panel_bounds_sized(outer, anchor, desired, side_offset, side, align);

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N2).resolve(&theme);
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
                    let submenu_open_for_menu = submenu_open.clone();
                    let submenu_trigger_for_menu = submenu_trigger.clone();
                    let submenu_geometry_for_menu = submenu_geometry.clone();
                    let submenu_close_timer_for_menu = submenu_close_timer.clone();
                    let submenu_last_pointer_for_menu = submenu_last_pointer.clone();

                    let open_for_timer = submenu_open.clone();
                    let trigger_for_timer = submenu_trigger.clone();
                    let last_pointer_for_timer = submenu_last_pointer.clone();
                    let geometry_for_timer = submenu_geometry.clone();
                    let close_timer_for_timer = submenu_close_timer.clone();
                    let overlay_root_element = cx.root_id();
                    cx.timer_on_timer_for(
                        overlay_root_element,
                        Arc::new(move |host, acx, token| {
                            let armed = host
                                .models_mut()
                                .read(&close_timer_for_timer, |v| *v)
                                .ok()
                                .flatten();
                            if armed != Some(token) {
                                return false;
                            }

                            let _ = host.models_mut().update(&open_for_timer, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&trigger_for_timer, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&last_pointer_for_timer, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&geometry_for_timer, |v| *v = None);
                            let _ = host
                                .models_mut()
                                .update(&close_timer_for_timer, |v| *v = None);
                            host.request_redraw(acx.window);
                            true
                        }),
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
                            vec![cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
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
                                    padding: Edges::all(Px(4.0)),
                                    background: Some(bg),
                                    shadow: Some(shadow),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(border),
                                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
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
                                            vec![cx.roving_flex(
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
                                                move |cx| {
                                                    cx.roving_nav_apg();
                                                    cx.roving_typeahead_prefix_arc_str(
                                                        labels_arc.clone(),
                                                        typeahead_timeout_ticks,
                                                    );

                                                    let font_size = theme.metrics.font_size;
                                                    let font_line_height =
                                                        theme.metrics.font_line_height;
                                                    let radius_sm = theme.metrics.radius_sm;
                                                    let text_disabled = theme.colors.text_disabled;
                                                    let destructive_fg = theme
                                                        .color_by_key("destructive")
                                                        .or_else(|| {
                                                            theme.color_by_key(
                                                                "destructive.background",
                                                            )
                                                        })
                                                        .unwrap_or(theme.colors.text_primary);

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
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: LayoutStyle::default(),
                                                                padding: Edges {
                                                                    top: pad_y,
                                                                    right: pad_x,
                                                                    bottom: pad_y,
                                                                    left: pad_x,
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
                                                                    overflow: TextOverflow::Ellipsis,
                                                                    color: Some(fg),
                                                                })]
                                                            },
                                                        ));
                                                    }
                                                    DropdownMenuEntry::Group(_) => {
                                                        unreachable!("groups are flattened")
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
                                                                            layout
                                                                        },
                                                                        padding: Edges::all(Px(0.0)),
                                                                        background: Some(border),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ));
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
                                                        let command = item.command;
                                                        let trailing = item.trailing.clone();
                                                        let variant = item.variant;
                                                        let has_submenu = item.submenu.is_some();
                                                        let open = open_for_menu.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.keyed(value.clone(), |cx| {
                                                            cx.pressable_with_id_props(|cx, st, item_id| {
                                                                if !disabled {
                                                                    let submenu_open_for_hover = submenu_open_for_menu.clone();
                                                                    let submenu_trigger_for_hover =
                                                                        submenu_trigger_for_menu.clone();
                                                                    let submenu_geometry_for_hover =
                                                                        submenu_geometry_for_menu.clone();
                                                                    let submenu_last_pointer_for_hover =
                                                                        submenu_last_pointer_for_menu.clone();
                                                                    let submenu_close_timer_for_hover =
                                                                        submenu_close_timer_for_menu.clone();
                                                                    let value_for_hover = value.clone();
                                                                    cx.pressable_add_on_hover_change(Arc::new(
                                                                        move |host, acx, is_hovered| {
                                                                            if !is_hovered {
                                                                                return;
                                                                            }

                                                                            let token = host
                                                                                .models_mut()
                                                                                .read(&submenu_close_timer_for_hover, |v| *v)
                                                                                .ok()
                                                                                .flatten();
                                                                            if let Some(token) = token {
                                                                                host.push_effect(Effect::CancelTimer { token });
                                                                            }
                                                                            let _ = host.models_mut().update(
                                                                                &submenu_close_timer_for_hover,
                                                                                |v| *v = None,
                                                                            );
                                                                            let _ = host.models_mut().update(
                                                                                &submenu_geometry_for_hover,
                                                                                |v| *v = None,
                                                                            );
                                                                            let _ = host.models_mut().update(
                                                                                &submenu_last_pointer_for_hover,
                                                                                |v| *v = None,
                                                                            );

                                                                            if has_submenu {
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_open_for_hover,
                                                                                    |v| *v = Some(value_for_hover.clone()),
                                                                                );
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_trigger_for_hover,
                                                                                    |v| *v = Some(acx.target),
                                                                                );
                                                                            } else {
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_open_for_hover,
                                                                                    |v| *v = None,
                                                                                );
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_trigger_for_hover,
                                                                                    |v| *v = None,
                                                                                );
                                                                            }

                                                                            host.request_redraw(acx.window);
                                                                        },
                                                                    ));
                                                                }

                                                                if !disabled && st.hovered {
                                                                    let token = cx
                                                                        .app
                                                                        .models_mut()
                                                                        .read(&submenu_close_timer_for_menu, |v| *v)
                                                                        .ok()
                                                                        .flatten();
                                                                    if let Some(token) = token {
                                                                        cx.app.push_effect(Effect::CancelTimer { token });
                                                                    }
                                                                    let _ = cx.app.models_mut().update(
                                                                        &submenu_close_timer_for_menu,
                                                                        |v| *v = None,
                                                                    );

                                                                    if has_submenu {
                                                                        let already_open = cx
                                                                            .app
                                                                            .models_mut()
                                                                            .read(&submenu_open_for_menu, |v| {
                                                                                v.as_ref()
                                                                                    .is_some_and(|cur| {
                                                                                        cur.as_ref() == value.as_ref()
                                                                                    })
                                                                            })
                                                                            .ok()
                                                                            .unwrap_or(false);

                                                                        if !already_open {
                                                                            let _ = cx.app.models_mut().update(
                                                                                &submenu_open_for_menu,
                                                                                |v| *v = Some(value.clone()),
                                                                            );
                                                                            let _ = cx.app.models_mut().update(
                                                                                &submenu_trigger_for_menu,
                                                                                |v| *v = Some(item_id),
                                                                            );
                                                                        } else {
                                                                            let trigger_missing = cx
                                                                                .app
                                                                                .models_mut()
                                                                                .read(&submenu_trigger_for_menu, |v| v.is_none())
                                                                                .ok()
                                                                                .unwrap_or(true);
                                                                            if trigger_missing {
                                                                                let _ = cx.app.models_mut().update(
                                                                                    &submenu_trigger_for_menu,
                                                                                    |v| *v = Some(item_id),
                                                                                );
                                                                            }
                                                                        }
                                                                    } else {
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_open_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_trigger_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_geometry_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                    }
                                                                }

                                                                if !disabled
                                                                    && st.focused
                                                                    && cx
                                                                        .app
                                                                        .models_mut()
                                                                        .read(
                                                                            &submenu_last_pointer_for_menu,
                                                                            |v| v.is_none(),
                                                                        )
                                                                        .ok()
                                                                        .unwrap_or(true)
                                                                {
                                                                    let open_value = cx
                                                                        .app
                                                                        .models_mut()
                                                                        .read(&submenu_open_for_menu, |v| v.clone())
                                                                        .ok()
                                                                        .flatten();
                                                                    let open_trigger = cx
                                                                        .app
                                                                        .models_mut()
                                                                        .read(&submenu_trigger_for_menu, |v| *v)
                                                                        .ok()
                                                                        .flatten();
                                                                    let is_open_here = open_value.as_ref().is_some_and(|cur| {
                                                                        cur.as_ref() == value.as_ref()
                                                                    }) && open_trigger == Some(item_id);

                                                                    if !is_open_here {
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_open_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_trigger_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_geometry_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                        let token = cx
                                                                            .app
                                                                            .models_mut()
                                                                            .read(&submenu_close_timer_for_menu, |v| *v)
                                                                            .ok()
                                                                            .flatten();
                                                                        if let Some(token) = token {
                                                                            cx.app.push_effect(
                                                                                Effect::CancelTimer { token },
                                                                            );
                                                                        }
                                                                        let _ = cx.app.models_mut().update(
                                                                            &submenu_close_timer_for_menu,
                                                                            |v| *v = None,
                                                                        );
                                                                    }
                                                                }

                                                                if has_submenu {
                                                                    let submenu_open_for_activate = submenu_open_for_menu.clone();
                                                                    let submenu_trigger_for_activate = submenu_trigger_for_menu.clone();
                                                                    let submenu_close_timer_for_activate =
                                                                        submenu_close_timer_for_menu.clone();
                                                                    let value_for_activate = value.clone();
                                                                    cx.pressable_add_on_activate(Arc::new(
                                                                        move |host, acx, _reason| {
                                                                            if disabled {
                                                                                return;
                                                                            }
                                                                            let token = host
                                                                                .models_mut()
                                                                                .read(&submenu_close_timer_for_activate, |v| *v)
                                                                                .ok()
                                                                                .flatten();
                                                                            if let Some(token) = token {
                                                                                host.push_effect(Effect::CancelTimer { token });
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_close_timer_for_activate,
                                                                                    |v| *v = None,
                                                                                );
                                                                            }
                                                                            let _ = host.models_mut().update(&submenu_open_for_activate, |v| {
                                                                                *v = Some(value_for_activate.clone());
                                                                            });
                                                                            let _ = host.models_mut().update(&submenu_trigger_for_activate, |v| {
                                                                                *v = Some(acx.target);
                                                                            });
                                                                            host.request_redraw(acx.window);
                                                                        },
                                                                    ));
                                                                } else {
                                                                    cx.pressable_dispatch_command_opt(command);
                                                                    if !disabled {
                                                                        cx.pressable_set_bool(&open, false);
                                                                    }
                                                                }

                                                                // Submenu keyboard affordances (minimal):
                                                                // - ArrowRight opens the submenu for this item (if any).
                                                                // - ArrowLeft closes any open submenu.
                                                                //
                                                                // Focus transfer into the submenu is not wired yet; users can still
                                                                // interact via pointer or Tab navigation.
                                                                let key_has_submenu = has_submenu;
                                                                let submenu_open_for_key = submenu_open_for_menu.clone();
                                                                let submenu_trigger_for_key = submenu_trigger_for_menu.clone();
                                                                let submenu_geometry_for_key =
                                                                    submenu_geometry_for_menu.clone();
                                                                let submenu_close_timer_for_key =
                                                                    submenu_close_timer_for_menu.clone();
                                                                let value_for_key = value.clone();
                                                                cx.key_on_key_down_for(
                                                                    item_id,
                                                                    Arc::new(move |host, acx, down| {
                                                                        if down.repeat {
                                                                            return false;
                                                                        }
                                                                        match down.key {
                                                                            KeyCode::ArrowRight => {
                                                                                if !key_has_submenu {
                                                                                    return false;
                                                                                }
                                                                                let token = host
                                                                                    .models_mut()
                                                                                    .read(&submenu_close_timer_for_key, |v| *v)
                                                                                    .ok()
                                                                                    .flatten();
                                                                                if let Some(token) = token {
                                                                                    host.push_effect(Effect::CancelTimer { token });
                                                                                    let _ = host.models_mut().update(
                                                                                        &submenu_close_timer_for_key,
                                                                                        |v| *v = None,
                                                                                    );
                                                                                }
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_open_for_key,
                                                                                    |v| *v = Some(value_for_key.clone()),
                                                                                );
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_trigger_for_key,
                                                                                    |v| *v = Some(acx.target),
                                                                                );
                                                                                host.request_redraw(acx.window);
                                                                                true
                                                                            }
                                                                            KeyCode::ArrowLeft => {
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_open_for_key,
                                                                                    |v| *v = None,
                                                                                );
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_trigger_for_key,
                                                                                    |v| *v = None,
                                                                                );
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_geometry_for_key,
                                                                                    |v| *v = None,
                                                                                );
                                                                                let token = host
                                                                                    .models_mut()
                                                                                    .read(&submenu_close_timer_for_key, |v| *v)
                                                                                    .ok()
                                                                                    .flatten();
                                                                                if let Some(token) = token {
                                                                                    host.push_effect(Effect::CancelTimer { token });
                                                                                }
                                                                                let _ = host.models_mut().update(
                                                                                    &submenu_close_timer_for_key,
                                                                                    |v| *v = None,
                                                                                );
                                                                                host.request_redraw(acx.window);
                                                                                true
                                                                            }
                                                                            _ => false,
                                                                        }
                                                                    }),
                                                                );

                                                                let is_open_submenu = cx
                                                                    .watch_model(&submenu_open_for_menu)
                                                                    .cloned()
                                                                    .unwrap_or(None)
                                                                    .as_ref()
                                                                    .is_some_and(|cur| cur.as_ref() == value.as_ref());

                                                                if has_submenu && is_open_submenu {
                                                                    let _ = cx.app.models_mut().update(
                                                                        &submenu_trigger_for_menu,
                                                                        |v| {
                                                                            if v.is_none() {
                                                                                *v = Some(item_id);
                                                                            }
                                                                        },
                                                                    );

                                                                    if let Some(trigger_anchor) =
                                                                        overlay::anchor_bounds_for_element(cx, item_id)
                                                                    {
                                                                        let outer =
                                                                            overlay::outer_bounds_with_window_margin(
                                                                                cx.bounds,
                                                                                window_margin,
                                                                            );
                                                                        let desired =
                                                                            Size::new(Px(192.0), Px(1.0e9));
                                                                        let placed =
                                                                            anchored_panel_bounds_sized(
                                                                                outer,
                                                                                trigger_anchor,
                                                                                desired,
                                                                                Px(2.0),
                                                                                Side::Right,
                                                                                Align::Start,
                                                                            );
                                                                        let geometry = DropdownMenuSubmenuGeometry {
                                                                            reference: trigger_anchor,
                                                                            floating: placed,
                                                                        };
                                                                        let _ = cx
                                                                            .app
                                                                            .models_mut()
                                                                            .update(&submenu_geometry_for_menu, |v| {
                                                                                if v.as_ref()
                                                                                    == Some(&geometry)
                                                                                {
                                                                                    return;
                                                                                }
                                                                                *v = Some(geometry);
                                                                            });
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
                                                                    a11y: PressableA11y {
                                                                        role: Some(SemanticsRole::MenuItem),
                                                                        label: a11y_label,
                                                                        expanded: has_submenu.then_some(is_open_submenu),
                                                                        ..Default::default()
                                                                    }
                                                                    .with_collection_position(collection_index, item_count),
                                                                    ..Default::default()
                                                                };

                                                                let mut row_bg = fret_core::Color::TRANSPARENT;
                                                                let mut row_fg = if variant == DropdownMenuItemVariant::Destructive {
                                                                    destructive_fg
                                                                } else {
                                                                    fg
                                                                };
                                                                if st.hovered || st.pressed || st.focused {
                                                                    row_bg = accent;
                                                                    row_fg = accent_fg;
                                                                }

                                                                let children = vec![cx.container(
                                                                    ContainerProps {
                                                                        layout: LayoutStyle::default(),
                                                                        padding: Edges {
                                                                            top: pad_y,
                                                                            right: pad_x,
                                                                            bottom: pad_y,
                                                                            left: pad_x,
                                                                        },
                                                                        background: Some(row_bg),
                                                                        corner_radii: fret_core::Corners::all(radius_sm),
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| {
                                                                        let mut row: Vec<AnyElement> = Vec::with_capacity(
                                                                            2 + usize::from(trailing.is_some()),
                                                                        );
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
                                                                        } else if has_submenu {
                                                                            let fg = theme
                                                                                .color_by_key("muted.foreground")
                                                                                .or_else(|| theme.color_by_key("muted-foreground"))
                                                                                .unwrap_or(theme.colors.text_muted);
                                                                            row.push(cx.text_props(TextProps {
                                                                                layout: LayoutStyle::default(),
                                                                                text: Arc::from(">"),
                                                                                style: Some(TextStyle {
                                                                                    font: FontId::default(),
                                                                                    size: font_size,
                                                                                    weight: FontWeight::MEDIUM,
                                                                                    line_height: Some(font_line_height),
                                                                                    letter_spacing_em: None,
                                                                                }),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Clip,
                                                                                color: Some(fg),
                                                                            }));
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
                                            )]
                                        },
                                    )]
                                },
                            )]
                        },
                    );

                    let pointer_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let content = cx.pointer_region(
                        PointerRegionProps {
                            layout: pointer_layout,
                            enabled: true,
                        },
                        move |cx| {
                            let last_pointer_for_hook = submenu_last_pointer.clone();
                            let geometry_for_hook = submenu_geometry.clone();
                            let close_timer_for_hook = submenu_close_timer.clone();
                            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                                let _ = host.models_mut().update(&last_pointer_for_hook, |v| {
                                    *v = Some(mv.position);
                                });
                                let geometry = host
                                    .models_mut()
                                    .read(&geometry_for_hook, |v| *v)
                                    .ok()
                                    .flatten();
                                if let Some(geometry) = geometry {
                                    let safe = safe_hover_contains(
                                        mv.position,
                                        geometry.reference,
                                        geometry.floating,
                                        SUBMENU_SAFE_HOVER_BUFFER,
                                    );

                                    let pending = host
                                        .models_mut()
                                        .read(&close_timer_for_hook, |v| *v)
                                        .ok()
                                        .flatten();
                                    if safe {
                                        if let Some(token) = pending {
                                            host.push_effect(Effect::CancelTimer { token });
                                            let _ = host
                                                .models_mut()
                                                .update(&close_timer_for_hook, |v| *v = None);
                                        }
                                    } else if pending.is_none() {
                                        let token = host.next_timer_token();
                                        host.push_effect(Effect::SetTimer {
                                            window: Some(acx.window),
                                            token,
                                            after: SUBMENU_CLOSE_DELAY,
                                            repeat: None,
                                        });
                                        let _ = host
                                            .models_mut()
                                            .update(&close_timer_for_hook, |v| *v = Some(token));
                                    }
                                }
                                host.request_redraw(acx.window);
                                false
                            }));

                            let mut children = vec![content];

                            let open_value = cx
                                .app
                                .models_mut()
                                .read(&submenu_open, |v| v.clone())
                                .ok()
                                .flatten();

                            if let Some(open_value) = open_value {
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
                                    let outer = overlay::outer_bounds_with_window_margin(
                                        cx.bounds,
                                        window_margin,
                                    );
                                    let desired = Size::new(Px(192.0), Px(1.0e9));

                                    let geometry = cx
                                        .app
                                        .models_mut()
                                        .read(&submenu_geometry, |v| *v)
                                        .ok()
                                        .flatten()
                                        .or_else(|| {
                                            let open_trigger = cx
                                                .app
                                                .models_mut()
                                                .read(&submenu_trigger, |v| *v)
                                                .ok()
                                                .flatten()?;
                                            let trigger_anchor =
                                                overlay::anchor_bounds_for_element(cx, open_trigger)?;
                                            let placed = anchored_panel_bounds_sized(
                                                outer,
                                                trigger_anchor,
                                                desired,
                                                Px(2.0),
                                                Side::Right,
                                                Align::Start,
                                            );
                                            Some(DropdownMenuSubmenuGeometry {
                                                reference: trigger_anchor,
                                                floating: placed,
                                            })
                                        });

                                    if let Some(geometry) = geometry {
                                        let _ = cx.app.models_mut().update(&submenu_geometry, |v| {
                                            if v.as_ref() == Some(&geometry) {
                                                return;
                                            }
                                            *v = Some(geometry);
                                        });

                                        let mut flat: Vec<DropdownMenuEntry> = Vec::new();
                                        flatten_entries(&mut flat, submenu_entries);
                                        let submenu_entries = flat;
                                        let item_count = submenu_entries
                                            .iter()
                                            .filter(|e| matches!(e, DropdownMenuEntry::Item(_)))
                                            .count();

                                            let font_size = theme.metrics.font_size;
                                            let font_line_height = theme.metrics.font_line_height;
                                            let radius_sm = theme.metrics.radius_sm;
                                            let text_disabled = theme.colors.text_disabled;
                                            let destructive_fg = theme
                                                .color_by_key("destructive")
                                                .or_else(|| theme.color_by_key("destructive.background"))
                                                .unwrap_or(theme.colors.text_primary);
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

                                            let submenu = cx.semantics(
                                                SemanticsProps {
                                                    layout: LayoutStyle::default(),
                                                    role: SemanticsRole::Menu,
                                                    ..Default::default()
                                                },
                                                move |cx| {
                                                    let mut item_ix: usize = 0;
                                                    let mut rows: Vec<AnyElement> =
                                                        Vec::with_capacity(submenu_entries.len());

                                                    for entry in submenu_entries.clone() {
                                                        match entry {
                                                            DropdownMenuEntry::Label(label) => {
                                                                let text = label.text.clone();
                                                                rows.push(cx.text_props(TextProps {
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
                                                                    overflow: TextOverflow::Ellipsis,
                                                                    color: Some(label_fg),
                                                                }));
                                                            }
                                                            DropdownMenuEntry::Group(_) => {
                                                                unreachable!("groups are flattened")
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
                                                                            layout
                                                                        },
                                                                        padding: Edges::all(Px(0.0)),
                                                                        background: Some(border),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ));
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
                                                                let command = item.command;
                                                                let trailing = item.trailing.clone();
                                                                let variant = item.variant;
                                                                let open = open_for_submenu.clone();
                                                                let submenu_open_for_key =
                                                                    submenu_open.clone();
                                                                let submenu_trigger_for_key =
                                                                    submenu_trigger.clone();
                                                                let text_style = text_style.clone();

                                                                rows.push(cx.keyed(value.clone(), |cx| {
                                                                    cx.pressable_with_id_props(
                                                                        |cx, st, item_id| {
                                                                            cx.pressable_dispatch_command_opt(command);
                                                                            if !disabled {
                                                                                cx.pressable_set_bool(&open, false);
                                                                            }

                                                                            let submenu_open_for_key =
                                                                                submenu_open_for_key.clone();
                                                                            let submenu_trigger_for_key =
                                                                                submenu_trigger_for_key.clone();
                                                                            cx.key_on_key_down_for(
                                                                                item_id,
                                                                                Arc::new(move |host, acx, down| {
                                                                                    if down.repeat {
                                                                                        return false;
                                                                                    }
                                                                                    if down.key != KeyCode::ArrowLeft {
                                                                                        return false;
                                                                                    }

                                                                                    let _ = host.models_mut().update(
                                                                                        &submenu_open_for_key,
                                                                                        |v| *v = None,
                                                                                    );
                                                                                    let _ = host.models_mut().update(
                                                                                        &submenu_trigger_for_key,
                                                                                        |v| *v = None,
                                                                                    );
                                                                                    host.request_redraw(acx.window);
                                                                                    true
                                                                                }),
                                                                            );

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
                                                                                a11y: PressableA11y {
                                                                                    role: Some(SemanticsRole::MenuItem),
                                                                                    label: a11y_label,
                                                                                    ..Default::default()
                                                                                }
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
                                                                                row_bg = accent;
                                                                                row_fg = accent_fg;
                                                                            }

                                                                            let children = vec![cx.container(
                                                                                ContainerProps {
                                                                                    layout: LayoutStyle::default(),
                                                                                    padding: Edges {
                                                                                        top: pad_y,
                                                                                        right: pad_x,
                                                                                        bottom: pad_y,
                                                                                        left: pad_x,
                                                                                    },
                                                                                    background: Some(row_bg),
                                                                                    corner_radii: fret_core::Corners::all(radius_sm),
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx| {
                                                                                    let mut row: Vec<AnyElement> = Vec::with_capacity(
                                                                                        1 + usize::from(trailing.is_some()),
                                                                                    );
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
                                                                            )];

                                                                            (props, children)
                                                                        },
                                                                    )
                                                                }));
                                                            }
                                                        }
                                                    }

                                                    vec![cx.container(
                                                        ContainerProps {
                                                            layout: LayoutStyle {
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
                                                            vec![cx.flex(
                                                                FlexProps {
                                                                    layout: LayoutStyle::default(),
                                                                    direction: fret_core::Axis::Vertical,
                                                                    gap: Px(0.0),
                                                                    padding: Edges::all(Px(0.0)),
                                                                    justify: MainAlign::Start,
                                                                    align: CrossAlign::Stretch,
                                                                    wrap: false,
                                                                },
                                                                move |_cx| rows.clone(),
                                                            )]
                                                        },
                                                    )]
                                                },
                                            );

                                        children.push(submenu);
                                    }
                                }
                            }

                            children
                        },
                    );

                    vec![content]
                });

                let mut request = OverlayRequest::dismissible_popover(
                    trigger_id,
                    trigger_id,
                    open,
                    OverlayPresence::instant(true),
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
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
    use fret_runtime::FrameId;
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

    fn rect_center(rect: Rect) -> Point {
        Point::new(
            Px(rect.origin.x.0 + rect.size.width.0 / 2.0),
            Px(rect.origin.y.0 + rect.size.height.0 / 2.0),
        )
    }

    #[test]
    fn dropdown_menu_submenu_opens_on_hover_and_closes_on_leave() {
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

        // Third frame: hover "More" should open the submenu.
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
            more.flags.expanded,
            "submenu trigger should report expanded=true after hover opens the submenu"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu items should render when hovering the submenu trigger"
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

        // Fourth frame: leaving the safe corridor arms a short close delay (submenu remains visible).
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

        // Fifth frame: after the close timer fires, the submenu closes.
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
