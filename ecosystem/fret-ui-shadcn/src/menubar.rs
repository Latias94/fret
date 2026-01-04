use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, SemanticsProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::roving_focus;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::menu;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct MenubarItem {
    pub label: Arc<str>,
    pub disabled: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
}

impl MenubarItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            command: None,
            a11y_label: None,
        }
    }

    pub fn submenu(self, entries: Vec<MenubarEntry>) -> MenubarSubmenu {
        MenubarSubmenu::new(self, entries)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
}

#[derive(Debug, Clone)]
pub struct MenubarSubmenu {
    pub trigger: MenubarItem,
    pub entries: Arc<[MenubarEntry]>,
}

impl MenubarSubmenu {
    pub fn new(trigger: MenubarItem, entries: Vec<MenubarEntry>) -> Self {
        Self {
            trigger,
            entries: Arc::from(entries.into_boxed_slice()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenubarEntry {
    Item(MenubarItem),
    Submenu(MenubarSubmenu),
    Separator,
}

#[derive(Clone)]
pub struct Menubar {
    menus: Vec<MenubarMenuEntries>,
    disabled: bool,
    typeahead_timeout_ticks: u64,
}

impl std::fmt::Debug for Menubar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menubar")
            .field("menus_len", &self.menus.len())
            .field("disabled", &self.disabled)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl Menubar {
    pub fn new(menus: Vec<MenubarMenuEntries>) -> Self {
        Self {
            menus,
            disabled: false,
            typeahead_timeout_ticks: 30,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme
                .color_by_key("border")
                .unwrap_or(theme.colors.panel_border);
            let radius = theme.metrics.radius_sm;
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            let pad_y = MetricRef::space(Space::N2).resolve(&theme);
            let gap = MetricRef::space(Space::N1).resolve(&theme);

            let disabled = self.disabled;
            let menus = self.menus;
            let typeahead_timeout_ticks = self.typeahead_timeout_ticks;

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
                            layout: LayoutStyle::default(),
                            padding: Edges {
                                top: pad_y,
                                right: pad_x,
                                bottom: pad_y,
                                left: pad_x,
                            },
                            background: Some(theme.colors.panel_background),
                            shadow: None,
                            border: Edges::all(Px(1.0)),
                            border_color: Some(border),
                            corner_radii: Corners::all(radius),
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
                                move |cx| menus.into_iter().map(|m| m.into_element(cx)).collect(),
                            )]
                        },
                    )]
                },
            )
        })
    }
}

#[derive(Clone)]
struct MenubarActive {
    trigger: GlobalElementId,
    open: Model<bool>,
}

#[derive(Default)]
struct MenubarGroupState {
    active: Option<Model<Option<MenubarActive>>>,
}

#[derive(Default)]
struct MenubarMenuState {
    open: Option<Model<bool>>,
}

#[derive(Clone)]
pub struct MenubarMenu {
    label: Arc<str>,
    disabled: bool,
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
            window_margin: Px(8.0),
            side_offset: Px(4.0),
            typeahead_timeout_ticks: 30,
        }
    }

    pub fn entries(self, entries: Vec<MenubarEntry>) -> MenubarMenuEntries {
        MenubarMenuEntries {
            menu: self,
            entries: Arc::from(entries.into_boxed_slice()),
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
}

impl std::fmt::Debug for MenubarMenuEntries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenubarMenuEntries")
            .field("label", &self.menu.label)
            .field("disabled", &self.menu.disabled)
            .field("entries_len", &self.entries.len())
            .finish()
    }
}

impl MenubarMenuEntries {
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let group = cx.root_id();
        let key = self.menu.label.clone();
        let entries = self.entries.clone();
        cx.keyed(key, |cx| {
            let group_active =
                cx.with_state_for(group, MenubarGroupState::default, |st| st.active.clone());
            let group_active = if let Some(group_active) = group_active {
                group_active
            } else {
                let group_active = cx.app.models_mut().insert(None);
                cx.with_state_for(group, MenubarGroupState::default, |st| {
                    st.active = Some(group_active.clone());
                });
                group_active
            };

            let open = cx.with_state(MenubarMenuState::default, |st| st.open.clone());
            let open = if let Some(open) = open {
                open
            } else {
                let open = cx.app.models_mut().insert(false);
                cx.with_state(MenubarMenuState::default, |st| st.open = Some(open.clone()));
                open
            };

            let theme = Theme::global(&*cx.app).clone();
            let enabled = !self.menu.disabled;

            let radius = theme.metrics.radius_sm;
            let ring = decl_style::focus_ring(&theme, radius);
            let bg_hover = theme.colors.hover_background;
            let bg_open = alpha_mul(theme.colors.selection_background, 0.35);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or(theme.colors.text_primary);
            let fg_muted = theme
                .color_by_key("muted-foreground")
                .unwrap_or(theme.colors.text_muted);

            let text_style = TextStyle {
                font: FontId::default(),
                size: theme.metrics.font_size,
                weight: FontWeight::MEDIUM,
                line_height: Some(theme.metrics.font_line_height),
                letter_spacing_em: None,
            };

            let label = self.menu.label.clone();

            cx.pressable_with_id_props(|cx, st, trigger_id| {
                if enabled {
                    menu::trigger::wire_open_on_arrow_keys(cx, trigger_id, open.clone());
                }

                let mut trigger_layout = LayoutStyle::default();
                trigger_layout.size.height = Length::Auto;
                trigger_layout.size.width = Length::Auto;

                let active_value = cx.watch_model(&group_active).cloned().flatten();
                let is_open = cx.watch_model(&open).copied().unwrap_or(false);

                if active_value
                    .as_ref()
                    .is_some_and(|active_value| active_value.trigger != trigger_id)
                    && is_open
                {
                    let _ = cx.app.models_mut().update(&open, |v| *v = false);
                }

                if active_value
                    .as_ref()
                    .is_some_and(|active_value| active_value.trigger == trigger_id)
                    && !is_open
                {
                    let _ = cx.app.models_mut().update(&group_active, |v| *v = None);
                }

                if active_value.is_none() && is_open {
                    let open_for_state = open.clone();
                    let _ = cx.app.models_mut().update(&group_active, |v| {
                        *v = Some(MenubarActive {
                            trigger: trigger_id,
                            open: open_for_state,
                        });
                    });
                }

                let active_value = cx.watch_model(&group_active).cloned().flatten();
                if enabled
                    && st.hovered
                    && !st.pressed
                    && active_value
                        .as_ref()
                        .is_some_and(|active_value| active_value.trigger != trigger_id)
                {
                    if let Some(prev) = active_value.as_ref() {
                        let _ = cx.app.models_mut().update(&prev.open, |v| *v = false);
                    }
                    let _ = cx.app.models_mut().update(&open, |v| *v = true);
                    let open_for_state = open.clone();
                    let _ = cx.app.models_mut().update(&group_active, |v| {
                        *v = Some(MenubarActive {
                            trigger: trigger_id,
                            open: open_for_state,
                        });
                    });
                }

                let group_active_for_activate = group_active.clone();
                let open_for_activate = open.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
                    let cur = host.models_mut().get_cloned(&group_active_for_activate).flatten();
                    match cur {
                        Some(cur) if cur.trigger == trigger_id => {
                            let _ = host.models_mut().update(&open_for_activate, |v| *v = false);
                            let _ =
                                host.models_mut().update(&group_active_for_activate, |v| *v = None);
                        }
                        prev => {
                            if let Some(prev) = prev {
                                let _ = host.models_mut().update(&prev.open, |v| *v = false);
                            }
                            let _ = host.models_mut().update(&open_for_activate, |v| *v = true);
                            let open_for_state = open_for_activate.clone();
                            let _ = host.models_mut().update(&group_active_for_activate, |v| {
                                *v = Some(MenubarActive {
                                    trigger: trigger_id,
                                    open: open_for_state,
                                });
                            });
                        }
                    }
                }));

                let is_open = cx.watch_model(&open).copied().unwrap_or(false);
                let overlay_root_name = OverlayController::popover_root_name(trigger_id);
                let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
                let submenu = cx.with_root_name(&overlay_root_name, |cx| {
                    menu::root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), submenu_cfg)
                });
                let trigger_bg = if is_open {
                    Some(bg_open)
                } else if st.hovered || st.pressed {
                    Some(alpha_mul(bg_hover, 0.8))
                } else {
                    None
                };

                let props = PressableProps {
                    layout: trigger_layout,
                    enabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::MenuItem),
                        label: Some(label.clone()),
                        expanded: Some(is_open),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                if is_open && enabled {
                    let side_offset = self.menu.side_offset;
                    let window_margin = self.menu.window_margin;
                    let typeahead_timeout_ticks = self.menu.typeahead_timeout_ticks;
                    let group_active = group_active;
                    let open_for_overlay = open.clone();
                    let text_style = text_style.clone();
                    let entries = entries.clone();

                    let (overlay_children, dismissible_on_pointer_move) =
                        cx.with_root_name(&overlay_root_name, move |cx| {
                        let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                            return (Vec::new(), None);
                        };
                        let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);
                        let estimated = fret_core::Size::new(Px(240.0), Px(220.0));

                        let placed = anchored_panel_bounds_sized(
                            outer,
                            anchor,
                            estimated,
                            side_offset,
                            Side::Bottom,
                            Align::Start,
                        );

                        let item_count = entries
                            .iter()
                            .filter(|e| matches!(e, MenubarEntry::Item(_) | MenubarEntry::Submenu(_)))
                            .count();

                        let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                            .iter()
                            .map(|e| match e {
                                MenubarEntry::Item(item) => (item.label.clone(), item.disabled),
                                MenubarEntry::Submenu(submenu) => {
                                    (submenu.trigger.label.clone(), submenu.trigger.disabled)
                                }
                                MenubarEntry::Separator => (Arc::from(""), true),
                            })
                            .unzip();

                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                        let disabled_arc: Arc<[bool]> =
                            Arc::from(disabled_flags.clone().into_boxed_slice());
                        let active = roving_focus::first_enabled(&disabled_flags);

                        let roving = RovingFocusProps {
                            enabled: true,
                            wrap: true,
                            disabled: disabled_arc,
                            ..Default::default()
                        };

                        let border = theme
                            .color_by_key("border")
                            .unwrap_or(theme.colors.panel_border);
                        let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                        let item_ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                        let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                        let pad_y = MetricRef::space(Space::N2).resolve(&theme);

                        let open_for_submenu = open_for_overlay.clone();
                        let submenu_for_content = submenu.clone();
                        let submenu_for_panel = submenu.clone();
                        let entries_for_content = entries.clone();
                        let entries_for_submenu = entries.clone();
                        let group_active_for_content = group_active.clone();
                        let group_active_for_submenu = group_active.clone();
                        let text_style_for_content = text_style.clone();
                        let text_style_for_submenu = text_style.clone();

                        let content = cx.semantics(
                            SemanticsProps {
                                layout: LayoutStyle::default(),
                                role: SemanticsRole::Menu,
                                ..Default::default()
                            },
                            move |cx| {
                                vec![menu::content_panel::menu_panel_container_at(
                                    cx,
                                    placed,
                                    move |layout| ContainerProps {
                                        layout,
                                        padding: Edges::all(Px(6.0)),
                                        background: Some(theme.colors.menu_background),
                                        shadow: Some(shadow),
                                        border: Edges::all(Px(1.0)),
                                        border_color: Some(border),
                                        corner_radii: Corners::all(theme.metrics.radius_sm),
                                    },
                                    move |cx| {
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
                                            labels_arc.clone(),
                                            typeahead_timeout_ticks,
                                            move |cx| {
                                                let mut out: Vec<AnyElement> =
                                                    Vec::with_capacity(entries_for_content.len());

                                                let mut item_ix: usize = 0;

                                                for (idx, entry) in entries_for_content
                                                    .iter()
                                                    .enumerate()
                                                {
                                                    let entry_value: Arc<str> =
                                                        Arc::from(format!("entry:{idx}"));
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
                                                                        layout
                                                                    },
                                                                    background: Some(border),
                                                                    ..Default::default()
                                                                },
                                                                |_| Vec::new(),
                                                            ));
                                                        }
                                                        MenubarEntry::Item(item)
                                                        | MenubarEntry::Submenu(MenubarSubmenu {
                                                            trigger: item,
                                                            ..
                                                        }) => {
                                                            let collection_index = item_ix;
                                                            item_ix = item_ix.saturating_add(1);

                                                            let item_enabled =
                                                                !item.disabled && enabled;
                                                            let focusable =
                                                                active.is_some_and(|a| a == idx);
                                                            let label = item.label.clone();
                                                            let a11y_label =
                                                                item.a11y_label.clone();
                                                            let command = item.command.clone();
                                                            let open = open_for_overlay.clone();
                                                            let group_active =
                                                                group_active_for_content.clone();
                                                            let text_style =
                                                                text_style_for_content.clone();
                                                            let has_submenu =
                                                                matches!(entry, MenubarEntry::Submenu(_));

                                                            let submenu_for_item =
                                                                submenu_for_content.clone();
                                                            let value = entry_value.clone();
                                                            out.push(cx.keyed(value.clone(), move |cx| {
                                                                cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                    let geometry_hint = has_submenu.then_some(
                                                                        menu::sub_trigger::MenuSubTriggerGeometryHint {
                                                                            outer,
                                                                            desired: fret_core::Size::new(
                                                                                Px(240.0),
                                                                                Px(1.0e9),
                                                                            ),
                                                                        },
                                                                    );
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

                                                                    if !has_submenu {
                                                                        cx.pressable_dispatch_command_opt(command);
                                                                        if item_enabled {
                                                                            cx.pressable_set_bool(&open, false);
                                                                        }

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

                                                                    let mut bg = Color::TRANSPARENT;
                                                                    if st.hovered || st.pressed {
                                                                        bg = alpha_mul(
                                                                            theme.colors.menu_item_hover,
                                                                            0.9,
                                                                        );
                                                                    }
                                                                    let fg = if item_enabled {
                                                                        fg
                                                                    } else {
                                                                        alpha_mul(fg_muted, 0.85)
                                                                    };

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
                                                                        a11y: menu::item::menu_item_a11y(
                                                                            a11y_label.or_else(|| {
                                                                                Some(label.clone())
                                                                            }),
                                                                            expanded,
                                                                        )
                                                                        .with_collection_position(
                                                                            collection_index,
                                                                            item_count,
                                                                        ),
                                                                        ..Default::default()
                                                                    };

                                                                    let children = vec![cx.container(
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
                                                                            border: Edges::all(Px(0.0)),
                                                                            border_color: None,
                                                                            corner_radii: Corners::all(
                                                                                theme.metrics.radius_sm,
                                                                            ),
                                                                        },
                                                                        move |cx| {
                                                                            vec![cx.text_props(TextProps {
                                                                                layout: LayoutStyle::default(),
                                                                                text: label.clone(),
                                                                                style: Some(text_style.clone()),
                                                                                color: Some(fg),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Clip,
                                                                            })]
                                                                        },
                                                                    )];

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
                                                                    cx.pressable_dispatch_command_opt(command);
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

                                                                    let mut bg =
                                                                        Color::TRANSPARENT;
                                                                    if st.hovered || st.pressed {
                                                                        bg = alpha_mul(
                                                                            theme
                                                                                .colors
                                                                                .menu_item_hover,
                                                                            0.9,
                                                                        );
                                                                    }
                                                                    let fg = if item_enabled {
                                                                        fg
                                                                    } else {
                                                                        alpha_mul(fg_muted, 0.85)
                                                                    };

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
                                                                                theme.metrics.radius_sm,
                                                                            ),
                                                                        },
                                                                        move |cx| {
                                                                            vec![cx.text_props(
                                                                                TextProps {
                                                                                    layout: LayoutStyle::default(),
                                                                                    text: label.clone(),
                                                                                    style: Some(
                                                                                        text_style.clone(),
                                                                                    ),
                                                                                    color: Some(fg),
                                                                                    wrap: TextWrap::None,
                                                                                    overflow:
                                                                                        TextOverflow::Clip,
                                                                                },
                                                                            )]
                                                                        },
                                                                    )]
                                                                },
                                                            ));
                                                        }
                                                    }
                                                }

                                                out
                                            },
                                        )]
                                    },
                                )]
                            },
                        );

                        let dismissible_on_pointer_move =
                            menu::root::submenu_pointer_move_handler(submenu.clone(), submenu_cfg);

                        let mut children = vec![content];
                        let desired = fret_core::Size::new(Px(240.0), Px(1.0e9));
                        let open_submenu = menu::sub::with_open_submenu(
                            cx,
                            &submenu_for_panel,
                            outer,
                            desired,
                            |_cx, open_value, geometry| (open_value, geometry.floating),
                        );

                        if let Some((open_value, placed)) = open_submenu {
                            let submenu_entries =
                                entries_for_submenu.iter().enumerate().find_map(|(idx, e)| {
                                    let MenubarEntry::Submenu(submenu) = e else {
                                        return None;
                                    };
                                    let value = format!("entry:{idx}");
                                    (value.as_str() == open_value.as_ref())
                                        .then_some(submenu.entries.clone())
                                });

                            if let Some(submenu_entries) = submenu_entries {
                                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) =
                                        submenu_entries
                                            .iter()
                                            .map(|e| match e {
                                                MenubarEntry::Item(item) => {
                                                    (item.label.clone(), item.disabled)
                                                }
                                                MenubarEntry::Submenu(submenu) => (
                                                    submenu.trigger.label.clone(),
                                                    submenu.trigger.disabled,
                                                ),
                                                MenubarEntry::Separator => (Arc::from(""), true),
                                            })
                                            .unzip();

                                    let labels_arc: Arc<[Arc<str>]> =
                                        Arc::from(labels.into_boxed_slice());
                                    let disabled_arc: Arc<[bool]> = Arc::from(
                                        disabled_flags.clone().into_boxed_slice(),
                                    );
                                    let active = roving_focus::first_enabled(&disabled_flags);
                                    let item_count = submenu_entries
                                        .iter()
                                        .filter(|e| {
                                            matches!(
                                                e,
                                                MenubarEntry::Item(_) | MenubarEntry::Submenu(_)
                                            )
                                        })
                                        .count();

                                    let roving = RovingFocusProps {
                                        enabled: true,
                                        wrap: true,
                                        disabled: disabled_arc,
                                        ..Default::default()
                                    };

                                    let submenu_entries_for_panel = submenu_entries.clone();
                                    let open_for_submenu = open_for_submenu.clone();
                                    let group_active = group_active_for_submenu.clone();
                                    let text_style = text_style_for_submenu.clone();
                                    let submenu_models_for_panel = submenu_for_panel.clone();
                                    let item_ring = item_ring;

                                    let submenu_panel = menu::sub_content::submenu_panel_at(
                                        cx,
                                        placed,
                                        move |layout| ContainerProps {
                                            layout,
                                            padding: Edges::all(Px(6.0)),
                                            background: Some(theme.colors.menu_background),
                                            shadow: Some(shadow),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border),
                                            corner_radii: Corners::all(theme.metrics.radius_sm),
                                        },
                                        move |cx| {
                                            vec![menu::content::menu_roving_group_apg_prefix_typeahead(
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
                                                            let mut out: Vec<AnyElement> =
                                                                Vec::with_capacity(submenu_entries_for_panel.len());
                                                            let mut item_ix: usize = 0;

                                                            for (idx, entry) in submenu_entries_for_panel.iter().enumerate() {
                                                                let entry_value: Arc<str> =
                                                                    Arc::from(format!("submenu_entry:{idx}"));
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
                                                                                    layout
                                                                                },
                                                                                background: Some(border),
                                                                                ..Default::default()
                                                                            },
                                                                            |_| Vec::new(),
                                                                        ));
                                                                    }
                                                                    MenubarEntry::Item(item) => {
                                                                        let collection_index = item_ix;
                                                                        item_ix = item_ix.saturating_add(1);

                                                                        let item_enabled = !item.disabled;
                                                                        let focusable = active.is_some_and(|a| a == idx);
                                                                        let label = item.label.clone();
                                                                        let a11y_label = item.a11y_label.clone();
                                                                        let command = item.command.clone();
                                                                        let open = open_for_submenu.clone();
                                                                        let group_active = group_active.clone();
                                                                        let submenu_for_key = submenu_models_for_panel.clone();
                                                                        let value = entry_value.clone();
                                                                        let text_style = text_style.clone();

                                                                        out.push(cx.keyed(value.clone(), move |cx| {
                                                                            cx.pressable_with_id_props(move |cx, st, item_id| {
                                                                                menu::sub_content::wire_item(
                                                                                    cx,
                                                                                    item_id,
                                                                                    !item_enabled,
                                                                                    &submenu_for_key,
                                                                                );

                                                                                cx.pressable_dispatch_command_opt(command);
                                                                                if item_enabled {
                                                                                    cx.pressable_set_bool(&open, false);
                                                                                }

                                                                                let group_active_for_activate =
                                                                                    group_active.clone();
                                                                                cx.pressable_add_on_activate(
                                                                                    Arc::new(move |host, _cx, _reason| {
                                                                                        let _ = host
                                                                                            .models_mut()
                                                                                            .update(&group_active_for_activate, |v| *v = None);
                                                                                    }),
                                                                                );

                                                                                let mut bg = Color::TRANSPARENT;
                                                                                if st.hovered || st.pressed {
                                                                                    bg = alpha_mul(
                                                                                        theme.colors.menu_item_hover,
                                                                                        0.9,
                                                                                    );
                                                                                }
                                                                                let fg = if item_enabled {
                                                                                    fg
                                                                                } else {
                                                                                    alpha_mul(fg_muted, 0.85)
                                                                                };

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
                                                                                    a11y: menu::item::menu_item_a11y(
                                                                                        a11y_label.or_else(|| Some(label.clone())),
                                                                                        None,
                                                                                    )
                                                                                    .with_collection_position(
                                                                                        collection_index,
                                                                                        item_count,
                                                                                    ),
                                                                                    ..Default::default()
                                                                                };

                                                                                let children = vec![cx.container(
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
                                                                                        border: Edges::all(Px(0.0)),
                                                                                        border_color: None,
                                                                                        corner_radii: Corners::all(theme.metrics.radius_sm),
                                                                                    },
                                                                                    move |cx| {
                                                                                        vec![cx.text_props(TextProps {
                                                                                            layout: LayoutStyle::default(),
                                                                                            text: label.clone(),
                                                                                            style: Some(text_style.clone()),
                                                                                            color: Some(fg),
                                                                                            wrap: TextWrap::None,
                                                                                            overflow: TextOverflow::Clip,
                                                                                        })]
                                                                                    },
                                                                                )];

                                                                                (props, children)
                                                                            })
                                                                        }));
                                                                    }
                                                                    MenubarEntry::Submenu(_submenu) => {}
                                                                }
                                                            }

                                                            out
                                                        },
                                                    )]
                                        },
                                    );

                                    children.push(submenu_panel);
                            }
                        }

                        (children, Some(dismissible_on_pointer_move))
                    });

                    let mut request = OverlayRequest::dismissible_popover(
                        trigger_id,
                        trigger_id,
                        open,
                        OverlayPresence::instant(true),
                        overlay_children,
                    );
                    request.root_name = Some(overlay_root_name);
                    request.dismissible_on_pointer_move = dismissible_on_pointer_move;
                    OverlayController::request(cx, request);
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
                    },
                    move |cx| {
                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: label.clone(),
                            style: Some(text_style.clone()),
                            color: Some(if enabled { fg } else { fg_muted }),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })]
                    },
                );

                (props, vec![content])
            })
        })
    }
}

pub fn menubar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<MenubarMenuEntries>,
) -> AnyElement {
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
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "menubar", |cx| {
                vec![menubar(cx, |_cx| {
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
                })]
            });
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

    #[test]
    fn menubar_hover_switches_open_menu() {
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
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
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
                position: edit,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        // Frame 2: switching begins (the hovered menu opens in the same frame).
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
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
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
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
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

        assert!(
            !snap3
                .nodes
                .iter()
                .any(|n| n.role == SemanticsRole::MenuItem
                    && n.label.as_deref() == Some("Sub Alpha")),
            "submenu should close on ArrowLeft"
        );

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
    }
}
