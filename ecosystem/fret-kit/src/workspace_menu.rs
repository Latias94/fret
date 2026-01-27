use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, Size, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, KeymapService, MenuBar, MenuItem, Platform,
    PlatformCapabilities, WhenExpr, WindowCommandEnabledService, WindowInputContextService,
    format_sequence,
};
use fret_ui::action::{ActionCx, OnDismissRequest, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps,
    SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};

use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::menubar as menu;
use fret_ui_kit::primitives::menubar::trigger_row as menubar_trigger_row;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::{OverlayController, OverlayPresence};

fn diag_test_id_suffix(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut last_was_dash = false;
    for ch in raw.chars() {
        let ch = if ch.is_ascii() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    let out = out.trim_matches('-');
    if out.is_empty() {
        return "x".to_string();
    }
    out.to_string()
}

fn diag_test_id(prefix: &str, raw: &str) -> Arc<str> {
    Arc::<str>::from(format!("{prefix}-{}", diag_test_id_suffix(raw)))
}

#[derive(Debug, Clone)]
pub struct MenubarFromRuntimeOptions {
    pub platform: Platform,
    pub include_shortcuts: bool,
}

impl MenubarFromRuntimeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    pub fn include_shortcuts(mut self, include: bool) -> Self {
        self.include_shortcuts = include;
        self
    }
}

#[derive(Debug, Clone)]
struct InWindowMenuItem {
    label: Arc<str>,
    value: Arc<str>,
    disabled: bool,
    command: Option<CommandId>,
    shortcut: Option<Arc<str>>,
    has_submenu: bool,
}

#[derive(Debug, Clone)]
struct InWindowSubmenu {
    trigger: InWindowMenuItem,
    entries: Arc<[InWindowMenuEntry]>,
}

#[derive(Debug, Clone)]
enum InWindowMenuEntry {
    Separator,
    Item(InWindowMenuItem),
    Submenu(InWindowSubmenu),
}

#[derive(Debug, Clone)]
struct InWindowMenu {
    title: Arc<str>,
    enabled: bool,
    entries: Arc<[InWindowMenuEntry]>,
}

#[derive(Debug, Clone)]
pub struct InWindowMenubarFocusHandle {
    pub group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    pub trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
}

#[derive(Default)]
struct InWindowMenubarMenuState {
    open: Option<fret_runtime::Model<bool>>,
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn menu_panel_desired_size(entries: &[InWindowMenuEntry], min_width: Px, row_height: Px) -> Size {
    let mut height = Px(8.0);
    for entry in entries {
        match entry {
            InWindowMenuEntry::Separator => height.0 += 9.0,
            InWindowMenuEntry::Item(_) | InWindowMenuEntry::Submenu(_) => height.0 += row_height.0,
        }
    }
    Size::new(min_width, height)
}

fn flatten_entries(out: &mut Vec<InWindowMenuEntry>, entries: &[InWindowMenuEntry]) {
    for entry in entries {
        match entry {
            InWindowMenuEntry::Submenu(submenu) => {
                out.push(InWindowMenuEntry::Submenu(submenu.clone()));
                flatten_entries(out, &submenu.entries);
            }
            _ => out.push(entry.clone()),
        }
    }
}

fn find_submenu_entries_by_value(
    entries: &[InWindowMenuEntry],
    value: &str,
) -> Option<Arc<[InWindowMenuEntry]>> {
    for entry in entries {
        match entry {
            InWindowMenuEntry::Submenu(submenu) => {
                if submenu.trigger.value.as_ref() == value {
                    return Some(submenu.entries.clone());
                }
                if let Some(found) = find_submenu_entries_by_value(&submenu.entries, value) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

fn roving_labels_and_disabled(entries: &[InWindowMenuEntry]) -> (Arc<[Arc<str>]>, Arc<[bool]>) {
    let mut labels: Vec<Arc<str>> = Vec::new();
    let mut disabled: Vec<bool> = Vec::new();
    for entry in entries {
        match entry {
            InWindowMenuEntry::Item(item) => {
                labels.push(item.label.clone());
                disabled.push(item.disabled);
            }
            InWindowMenuEntry::Submenu(submenu) => {
                labels.push(submenu.trigger.label.clone());
                disabled.push(submenu.trigger.disabled);
            }
            InWindowMenuEntry::Separator => {}
        }
    }
    (
        Arc::from(labels.into_boxed_slice()),
        Arc::from(disabled.into_boxed_slice()),
    )
}

fn command_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    command: &CommandId,
    item_when: Option<&WhenExpr>,
    base_ctx: &InputContext,
    opts: &MenubarFromRuntimeOptions,
) -> InWindowMenuItem {
    let (label, shortcut, meta_disabled) = match cx.app.commands().get(command.clone()) {
        Some(meta) => {
            let label = meta.title.clone();
            let shortcut = if opts.include_shortcuts {
                cx.app
                    .global::<KeymapService>()
                    .and_then(|svc| {
                        svc.keymap
                            .display_shortcut_for_command_sequence(base_ctx, command)
                    })
                    .or_else(|| {
                        meta.default_keybindings
                            .iter()
                            .find(|kb| kb.platform.matches(opts.platform))
                            .map(|kb| kb.sequence.clone())
                    })
                    .map(|seq| Arc::<str>::from(format_sequence(opts.platform, &seq)))
            } else {
                None
            };
            let meta_disabled = meta.when.as_ref().is_some_and(|w| !w.eval(base_ctx));
            (label, shortcut, meta_disabled)
        }
        None => (Arc::<str>::from(command.as_str()), None, false),
    };

    let item_disabled = item_when.is_some_and(|w| !w.eval(base_ctx));
    let command_disabled = cx
        .app
        .global::<WindowCommandEnabledService>()
        .and_then(|svc| svc.enabled(cx.window, command))
        == Some(false);
    let disabled = meta_disabled || item_disabled || command_disabled;

    InWindowMenuItem {
        label,
        value: Arc::<str>::from(command.as_str()),
        disabled,
        command: Some(command.clone()),
        shortcut,
        has_submenu: false,
    }
}

fn submenu_item(title: Arc<str>, value: Arc<str>, disabled: bool) -> InWindowMenuItem {
    InWindowMenuItem {
        label: title,
        value,
        disabled,
        command: None,
        shortcut: None,
        has_submenu: true,
    }
}

fn system_menu_placeholder_item(
    title: Arc<str>,
    value: Arc<str>,
    items: Vec<InWindowMenuEntry>,
) -> InWindowMenuEntry {
    InWindowMenuEntry::Submenu(InWindowSubmenu {
        trigger: submenu_item(title, value, true),
        entries: Arc::from(items.into_boxed_slice()),
    })
}

fn build_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: &[MenuItem],
    base_ctx: &InputContext,
    opts: &MenubarFromRuntimeOptions,
    prefix: &str,
) -> Vec<InWindowMenuEntry> {
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        match item {
            MenuItem::Separator => out.push(InWindowMenuEntry::Separator),
            MenuItem::SystemMenu {
                title,
                menu_type: _,
            } => {
                // In-window surfaces cannot materialize OS-owned menus. Keep a disabled placeholder
                // entry so the authored menu shape remains visible.
                let value: Arc<str> = Arc::from(format!("{prefix}.system_menu.{idx}"));
                out.push(system_menu_placeholder_item(
                    title.clone(),
                    value,
                    Vec::new(),
                ));
            }
            MenuItem::Command { command, when } => out.push(InWindowMenuEntry::Item(command_item(
                cx,
                command,
                when.as_ref(),
                base_ctx,
                opts,
            ))),
            MenuItem::Submenu { title, when, items } => {
                let value: Arc<str> = Arc::from(format!("{prefix}.submenu.{idx}"));
                let disabled = when.as_ref().is_some_and(|w| !w.eval(base_ctx));
                let trigger = submenu_item(title.clone(), value, disabled);
                let child_prefix = format!("{prefix}.submenu.{idx}");
                let entries = build_entries(cx, items, base_ctx, opts, &child_prefix);
                out.push(InWindowMenuEntry::Submenu(InWindowSubmenu {
                    trigger,
                    entries: Arc::from(entries.into_boxed_slice()),
                }));
            }
        }
    }
    out
}

/// Render an in-window menubar from the data-only `fret-runtime` [`MenuBar`].
///
/// This bridge intentionally does not depend on `fret-ui-shadcn` so apps can choose their own
/// chrome/recipe layer while keeping menu structure derived from commands (ADR 0023).
pub fn menubar_from_runtime<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    menu_bar: &MenuBar,
    opts: MenubarFromRuntimeOptions,
) -> AnyElement {
    let (el, _handle) = menubar_from_runtime_with_focus_handle(cx, menu_bar, opts);
    el
}

pub fn menubar_from_runtime_with_focus_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    menu_bar: &MenuBar,
    opts: MenubarFromRuntimeOptions,
) -> (AnyElement, InWindowMenubarFocusHandle) {
    let group = cx.root_id();

    let theme = Theme::global(&*cx.app).clone();
    let border = theme.color_required("color.panel.border");
    let bg = theme.color_required("color.panel.background");

    let radius = theme.metric_required("metric.radius.sm");
    let pad = theme.metric_required("metric.padding.sm");

    let group_active = menubar_trigger_row::ensure_group_active_model(cx, group);
    let trigger_registry = menubar_trigger_row::ensure_group_registry_model(cx, group);

    let base_ctx = menu_shortcut_input_context(cx, opts.platform);
    let menus: Vec<InWindowMenu> = menu_bar
        .menus
        .iter()
        .enumerate()
        .map(|(idx, menu)| {
            let entries = build_entries(cx, &menu.items, &base_ctx, &opts, &format!("menu.{idx}"));
            let enabled = entries
                .iter()
                .any(|e| !matches!(e, InWindowMenuEntry::Separator));
            InWindowMenu {
                title: menu.title.clone(),
                enabled,
                entries: Arc::from(entries.into_boxed_slice()),
            }
        })
        .collect();

    let trigger_labels: Arc<[Arc<str>]> = Arc::from(
        menus
            .iter()
            .map(|m| m.title.clone())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let trigger_disabled: Arc<[bool]> = Arc::from(
        menus
            .iter()
            .map(|m| !m.enabled)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );

    let group_active_for_render = group_active.clone();
    let trigger_registry_for_render = trigger_registry.clone();

    let element = cx.semantics(
        SemanticsProps {
            layout: LayoutStyle::default(),
            role: SemanticsRole::MenuBar,
            disabled: false,
            ..Default::default()
        },
        move |cx| {
            let group_active = group_active_for_render.clone();
            let trigger_registry = trigger_registry_for_render.clone();

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle::default(),
                    padding: Edges::all(Px(0.0)),
                    background: Some(bg),
                    shadow: None,
                    border: Edges::all(Px(1.0)),
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
                                gap: Px(0.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            roving: RovingFocusProps {
                                enabled: true,
                                wrap: true,
                                disabled: trigger_disabled.clone(),
                            },
                        },
                        roving_focus_group::TypeaheadPolicy::Prefix {
                            labels: trigger_labels.clone(),
                            timeout_ticks: 30,
                        },
                        move |cx| {
                            menus
                                .iter()
                                .cloned()
                                .map(|menu| {
                                    render_menu_from_runtime(
                                        cx,
                                        group_active.clone(),
                                        trigger_registry.clone(),
                                        menu,
                                        pad,
                                        &opts,
                                    )
                                })
                                .collect::<Vec<_>>()
                        },
                    )]
                },
            )]
        },
    );

    (
        element,
        InWindowMenubarFocusHandle {
            group_active,
            trigger_registry,
        },
    )
}

fn render_menu_from_runtime<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    menu: InWindowMenu,
    pad: Px,
    opts: &MenubarFromRuntimeOptions,
) -> AnyElement {
    let key = menu.title.clone();
    cx.keyed(key, |cx| {
        let open = cx.with_state(InWindowMenubarMenuState::default, |st| st.open.clone());
        let open = if let Some(open) = open {
            open
        } else {
            let open = cx.app.models_mut().insert(false);
            cx.with_state(InWindowMenubarMenuState::default, |st| {
                st.open = Some(open.clone())
            });
            open
        };

        let theme = Theme::global(&*cx.app).clone();
        let enabled = menu.enabled;

        let bg_hover = theme.color_required("color.hover.background");
        let bg_open = alpha_mul(theme.color_required("color.selection.background"), 0.35);
        let fg = theme.color_required("color.text.primary");
        let fg_disabled = theme.color_required("color.text.disabled");
        let ring = fret_ui::element::RingStyle {
            placement: fret_ui::element::RingPlacement::Outset,
            width: Px(1.0),
            offset: Px(1.0),
            color: theme.color_required("color.focus.ring"),
            offset_color: None,
            corner_radii: Corners::all(theme.metric_required("metric.radius.sm")),
        };

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

        cx.pressable_with_id_props(|cx, st, trigger_id| {
            let is_open = cx.watch_model(&open).copied().unwrap_or(false);

            menubar_trigger_row::register_trigger_in_registry(
                cx,
                trigger_registry.clone(),
                trigger_id,
                open.clone(),
                enabled,
            );
            menubar_trigger_row::sync_trigger_row_state(
                cx,
                group_active.clone(),
                trigger_id,
                open.clone(),
                enabled,
                st.hovered,
                st.pressed,
                st.focused,
            );

            cx.pressable_on_activate(menubar_trigger_row::toggle_on_activate(
                group_active.clone(),
                trigger_id,
                open.clone(),
            ));
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button == fret_core::MouseButton::Left {
                    host.request_focus(trigger_id);
                    host.request_redraw(action_cx.window);
                }
                fret_ui::action::PressablePointerDownResult::Continue
            }));
            menu::wire_menubar_open_on_arrow_keys(cx, trigger_id, open.clone());

            let overlay_root_name = menu::menubar_root_name(trigger_id);
            let content_id_for_trigger =
                menu::content_panel::menu_content_semantics_id::<H>(cx, &overlay_root_name);

            let trigger_bg = if is_open {
                Some(bg_open)
            } else if st.hovered || st.pressed || st.focused {
                Some(bg_hover)
            } else {
                None
            };

            let props = PressableProps {
                layout: LayoutStyle::default(),
                enabled,
                focusable: true,
                focus_ring: Some(ring),
                focus_ring_bounds: None,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::MenuItem),
                    label: Some(menu.title.clone()),
                    test_id: Some(diag_test_id("menubar-trigger", menu.title.as_ref())),
                    expanded: Some(is_open),
                    controls_element: Some(content_id_for_trigger.0),
                    ..Default::default()
                },
            };

            let overlay_presence = OverlayPresence::instant(is_open);
            if overlay_presence.present && enabled {
                request_menu_overlay(
                    cx,
                    &theme,
                    trigger_id,
                    open.clone(),
                    overlay_root_name.clone(),
                    overlay_presence,
                    &menu.entries,
                    group_active.clone(),
                    trigger_registry.clone(),
                    opts,
                );
            }

            let content = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: menu.title.clone(),
                style: Some(text_style),
                color: Some(if enabled { fg } else { fg_disabled }),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            });

            let content = cx.container(
                ContainerProps {
                    layout: LayoutStyle::default(),
                    padding: Edges {
                        top: Px(4.0),
                        right: pad,
                        bottom: Px(4.0),
                        left: pad,
                    },
                    background: trigger_bg,
                    shadow: None,
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(theme.metric_required("metric.radius.sm")),
                    ..Default::default()
                },
                |_cx| vec![content],
            );

            (props, vec![content])
        })
    })
}

#[allow(clippy::too_many_arguments)]
fn request_menu_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    trigger_id: GlobalElementId,
    open: fret_runtime::Model<bool>,
    overlay_root_name: String,
    overlay_presence: OverlayPresence,
    entries: &[InWindowMenuEntry],
    group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    opts: &MenubarFromRuntimeOptions,
) {
    let window_margin = Px(8.0);
    let side_offset = Px(6.0);
    let pad = theme.metric_required("metric.padding.sm");
    let radius = theme.metric_required("metric.radius.sm");

    let item_font_size = theme.metric_required("font.size");
    let item_line_height = theme.metric_required("font.line_height");
    let item_text = TextStyle {
        font: FontId::default(),
        size: item_font_size,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(item_line_height),
        letter_spacing_em: None,
    };

    let row_height = Px(item_line_height.0 + 8.0);

    let (labels, disabled_flags) = roving_labels_and_disabled(entries);

    let content_focus_id: std::rc::Rc<std::cell::Cell<Option<GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let content_focus_id_for_children = content_focus_id.clone();

    let open_for_overlay = open.clone();
    let group_active_for_overlay = group_active.clone();
    let trigger_registry_for_overlay = trigger_registry.clone();
    let item_text_for_overlay = item_text.clone();

    let open_for_dismiss = open.clone();
    let group_active_for_dismiss = group_active.clone();

    let direction = direction_prim::use_direction_in_scope(cx, None);
    let (overlay_children, dismissible_on_pointer_move) =
        cx.with_root_name(&overlay_root_name, move |cx| {
            let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                return (Vec::new(), None);
            };
            let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

            let desired = menu_panel_desired_size(entries, Px(220.0), row_height);
            let placement = popper::PopperContentPlacement::new(
                direction,
                fret_ui::overlay_placement::Side::Bottom,
                fret_ui::overlay_placement::Align::Start,
                side_offset,
            );
            let vars = menu::menubar_popper_vars(outer, anchor, desired.width, placement);
            let desired_w = menu::menubar_popper_desired_width(outer, anchor, desired.width);
            let desired = Size::new(desired_w, Px(desired.height.0.min(vars.available_height.0)));

            let layout = popper::popper_content_layout_sized(outer, anchor, desired, placement);
            let placed = layout.rect;

            let submenu_cfg = menu::sub::MenuSubmenuConfig::default();
            let submenu_for_panel =
                menu::root::sync_root_open_and_ensure_submenu(cx, true, cx.root_id(), submenu_cfg);

            let bg = theme.color_required("color.menu.background");
            let border = theme.color_required("color.menu.border");

            let open_for_panel_items = open_for_overlay.clone();
            let group_active_for_panel_items = group_active_for_overlay.clone();
            let trigger_registry_for_panel_items = trigger_registry_for_overlay.clone();
            let submenu_for_panel_items = submenu_for_panel.clone();
            let item_text_for_panel_items = item_text_for_overlay.clone();

            let panel = menu::content_panel::menu_panel_at(
                cx,
                placed,
                move |layout| ContainerProps {
                    layout,
                    padding: Edges::all(Px(4.0)),
                    background: Some(bg),
                    shadow: None,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.scroll(
                        ScrollProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                overflow: Overflow::Clip,
                                ..Default::default()
                            },
                            axis: ScrollAxis::Y,
                            ..Default::default()
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
                                    roving: RovingFocusProps {
                                        enabled: true,
                                        wrap: false,
                                        disabled: disabled_flags.clone(),
                                    },
                                },
                                labels.clone(),
                                30,
                                move |cx| {
                                    let roving_id = cx.root_id();
                                    if content_focus_id_for_children.get().is_none() {
                                        content_focus_id_for_children.set(Some(roving_id));
                                    }
                                    render_menu_entries(
                                        cx,
                                        theme,
                                        entries,
                                        open_for_panel_items.clone(),
                                        group_active_for_panel_items.clone(),
                                        trigger_registry_for_panel_items.clone(),
                                        opts,
                                        Some((submenu_for_panel_items.clone(), submenu_cfg)),
                                        pad,
                                        item_text_for_panel_items.clone(),
                                    )
                                },
                            )]
                        },
                    )]
                },
            );

            let dismissible_on_pointer_move =
                menu::root::submenu_pointer_move_handler(submenu_for_panel.clone(), submenu_cfg);

            let mut children = vec![panel];
            if let Some(submenu_open_value) = cx
                .watch_model(&submenu_for_panel.open_value)
                .cloned()
                .unwrap_or(None)
            {
                if let Some(submenu_entries) =
                    find_submenu_entries_by_value(entries, submenu_open_value.as_ref())
                {
                    let mut flat: Vec<InWindowMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, &submenu_entries);
                    let desired = menu_panel_desired_size(&flat, Px(180.0), row_height);
                    let desired =
                        Size::new(desired.width, Px(desired.height.0.min(outer.size.height.0)));

                    if let Some((open_value, geometry)) = menu::sub::with_open_submenu_synced(
                        cx,
                        &submenu_for_panel,
                        outer,
                        desired,
                        |_cx, open_value, geometry| (open_value, geometry),
                    ) {
                        let labelled_by_element = cx
                            .app
                            .models_mut()
                            .read(&submenu_for_panel.trigger, |v| *v)
                            .ok()
                            .flatten();
                        let open_for_submenu_items = open_for_overlay.clone();
                        let group_active_for_submenu_items = group_active_for_overlay.clone();
                        let trigger_registry_for_submenu_items =
                            trigger_registry_for_overlay.clone();
                        let submenu_for_submenu_items = submenu_for_panel.clone();
                        let item_text_for_submenu_items = item_text_for_overlay.clone();
                        let submenu_panel = menu::sub_content::submenu_panel_scroll_y_for_value_at(
                            cx,
                            open_value.clone(),
                            geometry.floating,
                            labelled_by_element,
                            move |layout| ContainerProps {
                                layout,
                                padding: Edges::all(Px(4.0)),
                                background: Some(bg),
                                shadow: None,
                                border: Edges::all(Px(1.0)),
                                border_color: Some(border),
                                corner_radii: Corners::all(radius),
                                ..Default::default()
                            },
                            move |cx| {
                                let (submenu_labels, submenu_disabled) =
                                    roving_labels_and_disabled(&submenu_entries);
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
                                                disabled: submenu_disabled,
                                            },
                                        },
                                        submenu_labels,
                                        30,
                                        submenu_for_submenu_items.clone(),
                                        move |cx| {
                                            render_menu_entries(
                                                cx,
                                                theme,
                                                &submenu_entries,
                                                open_for_submenu_items.clone(),
                                                group_active_for_submenu_items.clone(),
                                                trigger_registry_for_submenu_items.clone(),
                                                opts,
                                                Some((
                                                    submenu_for_submenu_items.clone(),
                                                    submenu_cfg,
                                                )),
                                                pad,
                                                item_text_for_submenu_items.clone(),
                                            )
                                        },
                                    ),
                                ]
                            },
                        );
                        children.push(submenu_panel);
                    }
                }
            }

            (children, Some(dismissible_on_pointer_move))
        });

    let on_dismiss_request: Option<OnDismissRequest> = Some(Arc::new(move |host, acx, _reason| {
        let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
        let _ = host
            .models_mut()
            .update(&group_active_for_dismiss, |v| *v = None);
        host.request_redraw(acx.window);
    }));

    let request = menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
        cx,
        trigger_id,
        trigger_id,
        open,
        overlay_presence,
        overlay_children,
        overlay_root_name,
        menu::root::MenuInitialFocusTargets::new()
            .pointer_content_focus(content_focus_id.get())
            .keyboard_entry_focus(content_focus_id.get()),
        None,
        None,
        on_dismiss_request,
        dismissible_on_pointer_move,
        false,
    );
    OverlayController::request(cx, request);
}

fn render_menu_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    entries: &[InWindowMenuEntry],
    open: fret_runtime::Model<bool>,
    group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    opts: &MenubarFromRuntimeOptions,
    submenu: Option<(menu::sub::MenuSubmenuModels, menu::sub::MenuSubmenuConfig)>,
    pad: Px,
    item_text: TextStyle,
) -> Vec<AnyElement> {
    let fg = theme.color_required("color.text.primary");
    let fg_muted = theme.color_required("color.text.muted");
    let fg_disabled = theme.color_required("color.text.disabled");
    let item_hover = theme.color_required("color.menu.item.hover");

    let mut out = Vec::new();
    for entry in entries {
        match entry {
            InWindowMenuEntry::Separator => {
                let layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(Px(1.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                out.push(cx.container(
                    ContainerProps {
                        layout,
                        background: Some(theme.color_required("color.menu.border")),
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                ));
            }
            InWindowMenuEntry::Item(item) => {
                out.push(render_menu_item(
                    cx,
                    theme,
                    item,
                    open.clone(),
                    group_active.clone(),
                    trigger_registry.clone(),
                    opts,
                    submenu.as_ref(),
                    pad,
                    item_text.clone(),
                    fg,
                    fg_muted,
                    fg_disabled,
                    item_hover,
                ));
            }
            InWindowMenuEntry::Submenu(submenu_entry) => {
                out.push(render_menu_item(
                    cx,
                    theme,
                    &submenu_entry.trigger,
                    open.clone(),
                    group_active.clone(),
                    trigger_registry.clone(),
                    opts,
                    submenu.as_ref(),
                    pad,
                    item_text.clone(),
                    fg,
                    fg_muted,
                    fg_disabled,
                    item_hover,
                ));
            }
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn render_menu_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    item: &InWindowMenuItem,
    open: fret_runtime::Model<bool>,
    group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    _opts: &MenubarFromRuntimeOptions,
    submenu: Option<&(menu::sub::MenuSubmenuModels, menu::sub::MenuSubmenuConfig)>,
    pad: Px,
    item_text: TextStyle,
    fg: Color,
    fg_muted: Color,
    fg_disabled: Color,
    item_hover: Color,
) -> AnyElement {
    let disabled = item.disabled;

    cx.pressable_with_id_props(|cx, st, item_id| {
        menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
            cx,
            item_id,
            group_active.clone(),
            trigger_registry.clone(),
        );

        if let Some((models, _cfg)) = submenu {
            menu::sub_content::wire_item(cx, item_id, disabled, models);
        }

        let mut expanded: Option<bool> = None;
        let mut controls_element: Option<u64> = None;
        if let Some((models, cfg)) = submenu
            && item.has_submenu
        {
            let geometry_hint = menu::sub_trigger::MenuSubTriggerGeometryHint {
                outer: overlay::outer_bounds_with_window_margin(cx.bounds, Px(8.0)),
                desired: Size::new(Px(180.0), Px(200.0)),
            };
            expanded = menu::sub_trigger::wire(
                cx,
                st,
                item_id,
                disabled,
                true,
                item.value.clone(),
                models,
                *cfg,
                Some(geometry_hint),
            );

            let overlay_root_name = menu::root::menu_overlay_root_name(item_id);
            let submenu_content_id = menu::sub_content::submenu_content_semantics_id::<H>(
                cx,
                &overlay_root_name,
                &item.value,
            );
            controls_element = Some(submenu_content_id.0);
        }

        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.min_height = Some(Px(28.0));

        let props = PressableProps {
            layout,
            enabled: !disabled,
            focusable: !disabled,
            focus_ring: None,
            focus_ring_bounds: None,
            a11y: PressableA11y {
                role: Some(SemanticsRole::MenuItem),
                label: Some(item.label.clone()),
                test_id: Some(diag_test_id("menubar-item", item.value.as_ref())),
                expanded,
                controls_element,
                ..Default::default()
            },
        };

        if let Some(command) = item.command.clone() {
            let open_for_close = open.clone();
            let group_active_for_close = group_active.clone();
            cx.pressable_add_on_activate(Arc::new(
                move |host: &mut dyn UiActionHost, acx: ActionCx, _reason| {
                    host.dispatch_command(Some(acx.window), command.clone());
                    let _ = host.models_mut().update(&open_for_close, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&group_active_for_close, |v| *v = None);
                    host.request_redraw(acx.window);
                },
            ));
        }
        cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
            if down.button == fret_core::MouseButton::Left {
                host.request_focus(item_id);
                host.request_redraw(action_cx.window);
            }
            fret_ui::action::PressablePointerDownResult::Continue
        }));

        let bg = if st.hovered || st.focused || st.pressed {
            Some(item_hover)
        } else {
            None
        };

        let text_color = if disabled { fg_disabled } else { fg };
        let shortcut_color = if disabled { fg_disabled } else { fg_muted };

        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: item.label.clone(),
            style: Some(item_text.clone()),
            color: Some(text_color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let trailing = if let Some(shortcut) = item.shortcut.clone() {
            Some(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: shortcut,
                style: Some(item_text.clone()),
                color: Some(shortcut_color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }))
        } else if item.has_submenu {
            Some(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from("›"),
                style: Some(item_text),
                color: Some(shortcut_color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }))
        } else {
            None
        };

        let row = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges {
                    top: Px(4.0),
                    right: pad,
                    bottom: Px(4.0),
                    left: pad,
                },
                background: bg,
                shadow: None,
                border: Edges::all(Px(0.0)),
                border_color: None,
                corner_radii: Corners::all(theme.metric_required("metric.radius.sm")),
                ..Default::default()
            },
            move |cx| {
                let mut inner_layout = LayoutStyle::default();
                inner_layout.size.width = Length::Fill;
                vec![cx.flex(
                    FlexProps {
                        layout: inner_layout,
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| {
                        let mut out = vec![label];
                        if let Some(trailing) = trailing {
                            out.push(trailing);
                        }
                        out
                    },
                )]
            },
        );

        (props, vec![row])
    })
}

fn menu_shortcut_input_context<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    platform: Platform,
) -> InputContext {
    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let snapshot = cx
        .app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .cloned();

    let mut ctx = snapshot.unwrap_or(InputContext {
        platform,
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        edit_can_undo: true,
        edit_can_redo: true,
        dispatch_phase: InputDispatchPhase::Bubble,
    });

    ctx.platform = platform;
    ctx.dispatch_phase = InputDispatchPhase::Bubble;
    ctx
}

impl Default for MenubarFromRuntimeOptions {
    fn default() -> Self {
        Self {
            platform: Platform::current(),
            include_shortcuts: true,
        }
    }
}
