use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, DecorationLineStyle, Edges, FontId, FontWeight, Px,
    SemanticsRole, Size, TextOverflow, TextSpan, TextStyle, TextWrap, UnderlineStyle,
};
use fret_runtime::{
    CommandId, CommandScope, InputContext, InputDispatchPhase, KeymapService, MenuBar, MenuItem,
    MenuItemToggle, MenuItemToggleKind, Platform, PlatformCapabilities, WhenExpr,
    WindowCommandGatingSnapshot, best_effort_snapshot_for_window_with_input_ctx_fallback,
    format_sequence,
};
use fret_ui::action::{ActionCx, OnDismissRequest, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps, ScrollAxis, ScrollProps,
    SemanticsProps, SizeStyle, StyledTextProps, TextProps,
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

fn fnv1a_64(s: &str) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001B3;

    let mut hash = OFFSET_BASIS;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn stable_menu_key(raw: &str) -> String {
    let slug = diag_test_id_suffix(raw);
    if slug != "x" {
        return slug;
    }
    format!("u{:016x}", fnv1a_64(raw))
}

fn attributed_title_with_mnemonic_underline(
    title: Arc<str>,
    mnemonic: char,
    underline_color: Color,
) -> Option<AttributedText> {
    let mnemonic = mnemonic.to_ascii_lowercase();

    let mut start: Option<usize> = None;
    let mut len: usize = 0;
    for (ix, ch) in title.char_indices() {
        if ch.to_ascii_lowercase() == mnemonic {
            start = Some(ix);
            len = ch.len_utf8();
            break;
        }
    }
    let start = start?;

    let underline = UnderlineStyle {
        color: Some(underline_color),
        style: DecorationLineStyle::Solid,
    };

    let mut spans: Vec<TextSpan> = Vec::with_capacity(3);
    if start > 0 {
        spans.push(TextSpan::new(start));
    }
    spans.push(TextSpan {
        len,
        shaping: Default::default(),
        paint: fret_core::TextPaintStyle::default().with_underline(underline),
    });
    let after_start = start.saturating_add(len);
    if after_start < title.len() {
        spans.push(TextSpan::new(title.len().saturating_sub(after_start)));
    }

    Some(AttributedText::new(title, spans))
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
    toggle: Option<MenuItemToggle>,
    shortcut: Option<Arc<str>>,
    has_submenu: bool,
    keep_if_empty_submenu: bool,
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
    mnemonic: Option<char>,
    entries: Arc<[InWindowMenuEntry]>,
}

#[derive(Debug, Clone)]
pub struct InWindowMenubarFocusHandle {
    pub group_active: fret_runtime::Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    pub trigger_registry: fret_runtime::Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    pub last_focus_before_menubar: fret_runtime::Model<Option<GlobalElementId>>,
    pub focus_is_trigger: fret_runtime::Model<bool>,
}

#[derive(Default)]
struct InWindowMenubarMenuState {
    open: Option<fret_runtime::Model<bool>>,
}

#[derive(Default)]
struct InWindowMenubarBridgeState {
    last_focus_before_menubar: Option<fret_runtime::Model<Option<GlobalElementId>>>,
    focus_is_trigger: Option<fret_runtime::Model<bool>>,
}

#[track_caller]
fn ensure_last_focus_before_menubar_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group: GlobalElementId,
) -> fret_runtime::Model<Option<GlobalElementId>> {
    let existing = cx.with_state_for(group, InWindowMenubarBridgeState::default, |st| {
        st.last_focus_before_menubar.clone()
    });
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx.app.models_mut().insert(None);
    cx.with_state_for(group, InWindowMenubarBridgeState::default, |st| {
        st.last_focus_before_menubar = Some(model.clone());
    });
    model
}

#[track_caller]
fn ensure_menubar_focus_is_trigger_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group: GlobalElementId,
) -> fret_runtime::Model<bool> {
    let existing = cx.with_state_for(group, InWindowMenubarBridgeState::default, |st| {
        st.focus_is_trigger.clone()
    });
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx.app.models_mut().insert(false);
    cx.with_state_for(group, InWindowMenubarBridgeState::default, |st| {
        st.focus_is_trigger = Some(model.clone());
    });
    model
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
    toggle: Option<MenuItemToggle>,
    gating: &WindowCommandGatingSnapshot,
    shortcut_base_ctx: &InputContext,
    opts: &MenubarFromRuntimeOptions,
) -> InWindowMenuItem {
    let (label, shortcut, meta_enabled) = match cx.app.commands().get(command.clone()) {
        Some(meta) => {
            let shortcut = if opts.include_shortcuts {
                cx.app
                    .global::<KeymapService>()
                    .and_then(|svc| {
                        svc.keymap
                            .display_shortcut_for_command_sequence(shortcut_base_ctx, command)
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
            let enabled = gating.is_enabled_for_meta(command, meta.scope, meta.when.as_ref());
            (meta.title.clone(), shortcut, enabled)
        }
        None => (
            Arc::<str>::from(command.as_str()),
            None,
            gating.is_enabled_for_meta(command, CommandScope::App, None),
        ),
    };

    let item_enabled = item_when
        .map(|w| w.eval(gating.input_ctx()))
        .unwrap_or(true);
    let disabled = !meta_enabled || !item_enabled;

    InWindowMenuItem {
        label,
        value: Arc::<str>::from(command.as_str()),
        disabled,
        command: Some(command.clone()),
        toggle,
        shortcut,
        has_submenu: false,
        keep_if_empty_submenu: false,
    }
}

fn submenu_item(title: Arc<str>, value: Arc<str>, disabled: bool) -> InWindowMenuItem {
    InWindowMenuItem {
        label: title,
        value,
        disabled,
        command: None,
        toggle: None,
        shortcut: None,
        has_submenu: true,
        keep_if_empty_submenu: false,
    }
}

fn label_item(title: Arc<str>, value: Arc<str>, disabled: bool) -> InWindowMenuItem {
    InWindowMenuItem {
        label: title,
        value,
        disabled,
        command: None,
        toggle: None,
        shortcut: None,
        has_submenu: false,
        keep_if_empty_submenu: false,
    }
}

fn system_menu_placeholder_item(
    title: Arc<str>,
    value: Arc<str>,
    items: Vec<InWindowMenuEntry>,
) -> InWindowMenuEntry {
    InWindowMenuEntry::Submenu(InWindowSubmenu {
        trigger: InWindowMenuItem {
            label: title,
            value,
            disabled: true,
            command: None,
            toggle: None,
            shortcut: None,
            has_submenu: true,
            keep_if_empty_submenu: true,
        },
        entries: Arc::from(items.into_boxed_slice()),
    })
}

fn sanitize_entries(entries: Vec<InWindowMenuEntry>) -> Vec<InWindowMenuEntry> {
    let mut out: Vec<InWindowMenuEntry> = Vec::new();
    let mut last_was_separator = false;

    for entry in entries {
        match entry {
            InWindowMenuEntry::Separator => {
                if out.is_empty() || last_was_separator {
                    continue;
                }
                out.push(InWindowMenuEntry::Separator);
                last_was_separator = true;
            }
            InWindowMenuEntry::Item(item) => {
                out.push(InWindowMenuEntry::Item(item));
                last_was_separator = false;
            }
            InWindowMenuEntry::Submenu(mut submenu) => {
                let sanitized = sanitize_entries(submenu.entries.as_ref().to_vec());
                if sanitized.is_empty() && !submenu.trigger.keep_if_empty_submenu {
                    continue;
                }
                submenu.entries = Arc::from(sanitized.into_boxed_slice());
                out.push(InWindowMenuEntry::Submenu(submenu));
                last_was_separator = false;
            }
        }
    }

    while matches!(out.last(), Some(InWindowMenuEntry::Separator)) {
        out.pop();
    }

    out
}

fn build_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: &[MenuItem],
    gating: &WindowCommandGatingSnapshot,
    shortcut_base_ctx: &InputContext,
    opts: &MenubarFromRuntimeOptions,
    prefix: &str,
) -> Vec<InWindowMenuEntry> {
    let mut out = Vec::new();
    for item in items.iter() {
        match item {
            MenuItem::Separator => out.push(InWindowMenuEntry::Separator),
            MenuItem::SystemMenu {
                title,
                menu_type: _,
            } => {
                // In-window surfaces cannot materialize OS-owned menus. Keep a disabled placeholder
                // entry so the authored menu shape remains visible.
                let value: Arc<str> = Arc::from(format!(
                    "{prefix}.system_menu.{}",
                    stable_menu_key(title.as_ref())
                ));
                out.push(system_menu_placeholder_item(
                    title.clone(),
                    value,
                    Vec::new(),
                ));
            }
            MenuItem::Label { title } => {
                let child_key = stable_menu_key(title.as_ref());
                let value: Arc<str> = Arc::from(format!("{prefix}.label.{child_key}"));
                out.push(InWindowMenuEntry::Item(label_item(
                    title.clone(),
                    value,
                    true,
                )));
            }
            MenuItem::Command {
                command,
                when,
                toggle,
            } => out.push(InWindowMenuEntry::Item(command_item(
                cx,
                command,
                when.as_ref(),
                *toggle,
                gating,
                shortcut_base_ctx,
                opts,
            ))),
            MenuItem::Submenu { title, when, items } => {
                let child_key = stable_menu_key(title.as_ref());
                let value: Arc<str> = Arc::from(format!("{prefix}.submenu.{child_key}"));
                let disabled = when.as_ref().is_some_and(|w| !w.eval(gating.input_ctx()));
                let trigger = submenu_item(title.clone(), value, disabled);
                let child_prefix = format!("{prefix}.submenu.{child_key}");
                let entries =
                    build_entries(cx, items, gating, shortcut_base_ctx, opts, &child_prefix);
                out.push(InWindowMenuEntry::Submenu(InWindowSubmenu {
                    trigger,
                    entries: Arc::from(entries.into_boxed_slice()),
                }));
            }
        }
    }
    sanitize_entries(out)
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
    let normalized_menu_bar = menu_bar.clone().normalized();
    let group = cx.root_id();

    let theme = Theme::global(&*cx.app).snapshot();
    let border = theme.color_required("color.panel.border");
    let bg = theme.color_required("color.panel.background");

    let radius = theme.metric_required("metric.radius.sm");
    let pad = theme.metric_required("metric.padding.sm");

    let group_active = menubar_trigger_row::ensure_group_active_model(cx, group);
    let trigger_registry = menubar_trigger_row::ensure_group_registry_model(cx, group);
    let last_focus_before_menubar = ensure_last_focus_before_menubar_model(cx, group);
    let focus_is_trigger = ensure_menubar_focus_is_trigger_model(cx, group);

    let fallback_ctx = menu_fallback_input_context(cx, opts.platform);
    let gating =
        best_effort_snapshot_for_window_with_input_ctx_fallback(cx.app, cx.window, fallback_ctx);
    let shortcut_base_ctx = menu_shortcut_display_input_context(&gating, opts.platform);
    let menus: Vec<InWindowMenu> = normalized_menu_bar
        .menus
        .iter()
        .map(|menu| {
            let menu_key = stable_menu_key(menu.title.as_ref());
            let entries = build_entries(
                cx,
                &menu.items,
                &gating,
                &shortcut_base_ctx,
                &opts,
                &format!("menu.{menu_key}"),
            );
            let enabled = entries
                .iter()
                .any(|e| !matches!(e, InWindowMenuEntry::Separator));
            InWindowMenu {
                title: menu.title.clone(),
                enabled,
                mnemonic: menu.mnemonic,
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
    let last_focus_for_render = last_focus_before_menubar.clone();
    let focus_is_trigger_for_render = focus_is_trigger.clone();

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
            let last_focus_before_menubar = last_focus_for_render.clone();
            let focus_is_trigger = focus_is_trigger_for_render.clone();

            let focused = cx.focused_element();
            let focused_is_trigger = focused.is_some_and(|id| {
                cx.app
                    .models()
                    .read(&trigger_registry, |v| v.iter().any(|e| e.trigger == id))
                    .ok()
                    .unwrap_or(false)
            });
            let cur_focused_is_trigger = cx
                .app
                .models()
                .read(&focus_is_trigger, |v| *v)
                .ok()
                .unwrap_or(false);
            if cur_focused_is_trigger != focused_is_trigger {
                let _ = cx.app.models_mut().update(&focus_is_trigger, |v| {
                    *v = focused_is_trigger;
                });
            }

            let active = cx
                .app
                .models()
                .read(&group_active, |v| v.clone())
                .ok()
                .flatten();
            if let Some(active) = active.as_ref() {
                let is_open = cx
                    .app
                    .models()
                    .read(&active.open, |v| *v)
                    .ok()
                    .unwrap_or(false);
                if !is_open && !focused_is_trigger {
                    let _ = cx.app.models_mut().update(&group_active, |v| *v = None);
                }
            } else if let Some(focused) = focused {
                if !focused_is_trigger {
                    let current = cx
                        .app
                        .models()
                        .read(&last_focus_before_menubar, |v| *v)
                        .ok()
                        .flatten();
                    if current != Some(focused) {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&last_focus_before_menubar, |v| *v = Some(focused));
                    }
                }
            }

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
                                gap: Px(1.0),
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
            last_focus_before_menubar,
            focus_is_trigger,
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
            let (patient_click_sticky, patient_click_timer) =
                menubar_trigger_row::ensure_trigger_patient_click_models(cx, trigger_id);
            let is_open = cx.watch_model(&open).copied().unwrap_or(false);
            let group_has_active = cx.watch_model(&group_active).cloned().is_some();
            let show_mnemonics =
                matches!(opts.platform, Platform::Windows | Platform::Linux) && group_has_active;

            menubar_trigger_row::register_trigger_in_registry(
                cx,
                trigger_registry.clone(),
                trigger_id,
                open.clone(),
                enabled,
                menu.mnemonic,
            );
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

            cx.pressable_on_activate(menubar_trigger_row::toggle_on_activate(
                group_active.clone(),
                trigger_id,
                open.clone(),
                patient_click_sticky,
                patient_click_timer,
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

            let text_color = if enabled { fg } else { fg_disabled };
            let content = if show_mnemonics
                && let Some(mnemonic) = menu.mnemonic
                && let Some(rich) = attributed_title_with_mnemonic_underline(
                    menu.title.clone(),
                    mnemonic,
                    text_color,
                ) {
                cx.styled_text_props(StyledTextProps {
                    layout: LayoutStyle::default(),
                    rich,
                    style: Some(text_style),
                    color: Some(text_color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
            } else {
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: menu.title.clone(),
                    style: Some(text_style),
                    color: Some(text_color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
            };

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

    let first_item_focus_id: std::rc::Rc<std::cell::Cell<Option<GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let first_item_focus_id_for_children = first_item_focus_id.clone();

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
                                        first_item_focus_id_for_children.clone(),
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
                                                std::rc::Rc::new(std::cell::Cell::new(None)),
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

    let on_dismiss_request: Option<OnDismissRequest> = Some(Arc::new(move |host, acx, _req| {
        let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
        let _ = host
            .models_mut()
            .update(&group_active_for_dismiss, |v| *v = None);
        host.request_redraw(acx.window);
    }));

    let keyboard_entry_focus = first_item_focus_id.get().or_else(|| content_focus_id.get());
    let initial_focus = menu::root::MenuInitialFocusTargets::new()
        .pointer_content_focus(content_focus_id.get())
        .keyboard_entry_focus(keyboard_entry_focus);
    let request = menu::root::dismissible_menu_request_with_modal_and_dismiss_handler(
        cx,
        trigger_id,
        trigger_id,
        open,
        overlay_presence,
        overlay_children,
        overlay_root_name,
        initial_focus,
        None,
        None,
        on_dismiss_request,
        dismissible_on_pointer_move,
        true,
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
    first_item_focus_id: std::rc::Rc<std::cell::Cell<Option<GlobalElementId>>>,
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
                    first_item_focus_id.clone(),
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
                    first_item_focus_id.clone(),
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
    first_item_focus_id: std::rc::Rc<std::cell::Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let disabled = item.disabled;

    cx.pressable_with_id_props(|cx, st, item_id| {
        if !disabled && first_item_focus_id.get().is_none() {
            first_item_focus_id.set(Some(item_id));
        }

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

        let (role, checked) = match item.toggle {
            Some(toggle) => match toggle.kind {
                MenuItemToggleKind::Checkbox => {
                    (SemanticsRole::MenuItemCheckbox, Some(toggle.checked))
                }
                MenuItemToggleKind::Radio => (SemanticsRole::MenuItemRadio, Some(toggle.checked)),
            },
            None => (SemanticsRole::MenuItem, None),
        };

        let props = PressableProps {
            layout,
            enabled: !disabled,
            focusable: !disabled,
            focus_ring: None,
            focus_ring_bounds: None,
            a11y: PressableA11y {
                role: Some(role),
                label: Some(item.label.clone()),
                test_id: Some(diag_test_id("menubar-item", item.value.as_ref())),
                expanded,
                checked,
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

        let item_text_for_leading = item_text.clone();
        let leading = item.toggle.map(|toggle| {
            let item_text_for_leading = item_text_for_leading.clone();
            let symbol: Arc<str> = match toggle.kind {
                MenuItemToggleKind::Checkbox => {
                    if toggle.checked {
                        Arc::from("✓")
                    } else {
                        Arc::from("")
                    }
                }
                MenuItemToggleKind::Radio => {
                    if toggle.checked {
                        Arc::from("●")
                    } else {
                        Arc::from("")
                    }
                }
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Px(Px(16.0));
            layout.size.height = Length::Fill;

            cx.flex(
                FlexProps {
                    layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: symbol.clone(),
                        style: Some(item_text_for_leading.clone()),
                        color: Some(text_color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    })]
                },
            )
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
                    move |cx| {
                        let mut left_children: Vec<AnyElement> = Vec::new();
                        if let Some(leading) = leading {
                            left_children.push(leading);
                        }
                        left_children.push(label);
                        let left = cx.flex(
                            FlexProps {
                                layout: LayoutStyle::default(),
                                direction: fret_core::Axis::Horizontal,
                                gap: Px(8.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |_cx| left_children,
                        );

                        let mut out = vec![left];
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

fn menu_fallback_input_context<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    platform: Platform,
) -> InputContext {
    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let mut ctx = InputContext {
        platform,
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    };

    ctx.platform = platform;
    ctx.dispatch_phase = InputDispatchPhase::Bubble;
    ctx
}

fn menu_shortcut_display_input_context(
    gating: &WindowCommandGatingSnapshot,
    platform: Platform,
) -> InputContext {
    let mut ctx = gating.input_ctx().clone();
    ctx.platform = platform;
    ctx.dispatch_phase = InputDispatchPhase::Bubble;
    // Shortcut labels should remain stable even when command availability changes.
    ctx.edit_can_undo = true;
    ctx.edit_can_redo = true;
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Rect, Size};
    use fret_runtime::{CommandId, Menu, MenuBar, MenuItem};
    use fret_ui::tree::UiTree;

    fn plain_item(label: &str) -> InWindowMenuItem {
        InWindowMenuItem {
            label: Arc::from(label),
            value: Arc::from(label),
            disabled: false,
            command: None,
            toggle: None,
            shortcut: None,
            has_submenu: false,
            keep_if_empty_submenu: false,
        }
    }

    #[test]
    fn sanitize_entries_drops_leading_trailing_and_duplicate_separators() {
        let entries = vec![
            InWindowMenuEntry::Separator,
            InWindowMenuEntry::Separator,
            InWindowMenuEntry::Item(plain_item("A")),
            InWindowMenuEntry::Separator,
            InWindowMenuEntry::Separator,
            InWindowMenuEntry::Item(plain_item("B")),
            InWindowMenuEntry::Separator,
        ];

        let sanitized = sanitize_entries(entries);
        assert!(!matches!(
            sanitized.first(),
            Some(InWindowMenuEntry::Separator)
        ));
        assert!(!matches!(
            sanitized.last(),
            Some(InWindowMenuEntry::Separator)
        ));

        let mut prev_sep = false;
        for e in &sanitized {
            let is_sep = matches!(e, InWindowMenuEntry::Separator);
            assert!(!(prev_sep && is_sep), "expected no duplicate separators");
            prev_sep = is_sep;
        }
    }

    #[test]
    fn sanitize_entries_drops_empty_submenus_but_keeps_system_placeholders() {
        let empty_submenu = InWindowMenuEntry::Submenu(InWindowSubmenu {
            trigger: submenu_item(Arc::from("Empty"), Arc::from("empty"), false),
            entries: Arc::from(Vec::<InWindowMenuEntry>::new().into_boxed_slice()),
        });
        let system_placeholder =
            system_menu_placeholder_item(Arc::from("Services"), Arc::from("sys"), Vec::new());

        let sanitized = sanitize_entries(vec![empty_submenu, system_placeholder]);
        assert_eq!(sanitized.len(), 1);
        match &sanitized[0] {
            InWindowMenuEntry::Submenu(sub) => assert!(sub.trigger.keep_if_empty_submenu),
            _ => panic!("expected submenu placeholder"),
        }
    }

    #[test]
    fn mnemonic_underline_creates_valid_attributed_text() {
        let title: Arc<str> = Arc::from("File");
        let rich = attributed_title_with_mnemonic_underline(
            title.clone(),
            'i',
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        )
        .expect("expected mnemonic underline match");
        assert!(rich.is_valid());
        assert_eq!(rich.text, title);
        assert_eq!(rich.spans.len(), 3);
        assert!(rich.spans[0].paint.underline.is_none());
        assert!(rich.spans[1].paint.underline.is_some());
        assert!(rich.spans[2].paint.underline.is_none());

        let miss = attributed_title_with_mnemonic_underline(
            title.clone(),
            'z',
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        assert!(
            miss.is_none(),
            "expected no underline when mnemonic not present"
        );
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn escape_unwinds_submenu_then_menu_and_restores_focus() {
        use fret_app::App;
        use fret_runtime::Effect;
        use fret_ui_kit::OverlayController;
        use fret_ui_shadcn::shadcn_themes::{
            ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york_v4,
        };

        #[derive(Default)]
        struct FakeServices;

        impl fret_core::TextService for FakeServices {
            fn prepare(
                &mut self,
                _input: &fret_core::TextInput,
                _constraints: fret_core::TextConstraints,
            ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
                (
                    fret_core::TextBlobId::default(),
                    fret_core::TextMetrics {
                        size: Size::new(Px(10.0), Px(10.0)),
                        baseline: Px(8.0),
                    },
                )
            }

            fn release(&mut self, _blob: fret_core::TextBlobId) {}
        }

        impl fret_core::PathService for FakeServices {
            fn prepare(
                &mut self,
                _commands: &[fret_core::PathCommand],
                _style: fret_core::PathStyle,
                _constraints: fret_core::PathConstraints,
            ) -> (fret_core::PathId, fret_core::PathMetrics) {
                (
                    fret_core::PathId::default(),
                    fret_core::PathMetrics::default(),
                )
            }

            fn release(&mut self, _path: fret_core::PathId) {}
        }

        impl fret_core::SvgService for FakeServices {
            fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
                fret_core::SvgId::default()
            }

            fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
                true
            }
        }

        fn bounds() -> Rect {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            )
        }

        fn menu_bar() -> MenuBar {
            MenuBar {
                menus: vec![Menu {
                    title: Arc::from("Window"),
                    role: None,
                    mnemonic: None,
                    items: vec![
                        MenuItem::Submenu {
                            title: Arc::from("Split"),
                            when: None,
                            items: vec![MenuItem::Command {
                                command: CommandId::from("workspace.pane.split.right"),
                                when: None,
                                toggle: None,
                            }],
                        },
                        MenuItem::Command {
                            command: CommandId::from("test.noop"),
                            when: None,
                            toggle: None,
                        },
                    ],
                }],
            }
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: fret_core::AppWindowId,
            bounds: Rect,
            bar: &MenuBar,
        ) -> fret_core::SemanticsSnapshot {
            let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);

            apply_shadcn_new_york_v4(app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
            OverlayController::begin_frame(app, window);

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "menu",
                |cx| {
                    vec![menubar_from_runtime(
                        cx,
                        bar,
                        MenubarFromRuntimeOptions::default(),
                    )]
                },
            );
            ui.set_root(root);

            OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            ui.semantics_snapshot().expect("semantics snapshot").clone()
        }

        fn node_by_test_id<'a>(
            snap: &'a fret_core::SemanticsSnapshot,
            id: &str,
        ) -> &'a fret_core::SemanticsNode {
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(id))
                .unwrap_or_else(|| {
                    let available: Vec<&str> = snap
                        .nodes
                        .iter()
                        .filter_map(|n| n.test_id.as_deref())
                        .collect();
                    panic!(
                        "expected semantics node with test_id={id:?}; available_test_ids={available:?}"
                    )
                })
        }

        fn assert_focus_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) {
            let node_id = node_by_test_id(snap, id).id;
            assert_eq!(snap.focus, Some(node_id), "expected focus to be {id:?}");
        }

        fn assert_exists(snap: &fret_core::SemanticsSnapshot, id: &str) {
            assert!(
                snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id)),
                "expected a semantics node with test_id={id:?}"
            );
        }

        fn assert_not_exists(snap: &fret_core::SemanticsSnapshot, id: &str) {
            assert!(
                !snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id)),
                "expected no semantics node with test_id={id:?}"
            );
        }

        fn press_key(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            key: fret_core::KeyCode,
        ) {
            ui.dispatch_event(
                app,
                services,
                &fret_core::Event::KeyDown {
                    key,
                    modifiers: fret_core::Modifiers::default(),
                    repeat: false,
                },
            );
            ui.dispatch_event(
                app,
                services,
                &fret_core::Event::KeyUp {
                    key,
                    modifiers: fret_core::Modifiers::default(),
                },
            );
        }

        fn pointer_up(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            at: Point,
            is_click: bool,
        ) {
            ui.dispatch_event(
                app,
                services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                    pointer_id: fret_core::PointerId(0),
                    position: at,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    is_click,
                    click_count: 1,
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
        }

        fn pointer_down(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            at: Point,
        ) {
            ui.dispatch_event(
                app,
                services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                    pointer_id: fret_core::PointerId(0),
                    position: at,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    click_count: 1,
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
        }

        fn rect_center(r: Rect) -> Point {
            Point::new(
                Px(r.origin.x.0 + r.size.width.0 * 0.5),
                Px(r.origin.y.0 + r.size.height.0 * 0.5),
            )
        }

        let window = fret_core::AppWindowId::default();
        let mut app = App::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let bounds = bounds();
        let bar = menu_bar();

        // Open the menu by clicking the menubar trigger (pointer-up activation).
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        let window_trigger_bounds = node_by_test_id(&snap, "menubar-trigger-window").bounds;
        let window_trigger_center = rect_center(window_trigger_bounds);
        pointer_down(&mut ui, &mut app, &mut services, window_trigger_center);
        pointer_up(
            &mut ui,
            &mut app,
            &mut services,
            window_trigger_center,
            true,
        );
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        assert!(
            node_by_test_id(&snap, "menubar-trigger-window")
                .flags
                .expanded,
            "expected menubar trigger to be expanded after click"
        );
        assert_exists(&snap, "menubar-item-menu-window-submenu-split");

        // Focus the submenu trigger item without activating it.
        let split_trigger_bounds =
            node_by_test_id(&snap, "menubar-item-menu-window-submenu-split").bounds;
        let split_trigger_center = rect_center(split_trigger_bounds);
        pointer_down(&mut ui, &mut app, &mut services, split_trigger_center);
        pointer_up(
            &mut ui,
            &mut app,
            &mut services,
            split_trigger_center,
            false,
        );
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        assert_focus_test_id(&snap, "menubar-item-menu-window-submenu-split");

        // ArrowRight opens the submenu and (after focus-delay timer) focuses its first item.
        press_key(
            &mut ui,
            &mut app,
            &mut services,
            fret_core::KeyCode::ArrowRight,
        );
        let _snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        let focus_timer = app
            .flush_effects()
            .into_iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if after.is_zero() => Some(token),
                _ => None,
            })
            .expect("submenu focus timer");
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer { token: focus_timer },
        );
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        assert_exists(&snap, "menubar-item-workspace-pane-split-right");
        assert_focus_test_id(&snap, "menubar-item-workspace-pane-split-right");

        // Escape closes submenu and returns focus to the submenu trigger.
        press_key(&mut ui, &mut app, &mut services, fret_core::KeyCode::Escape);
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        assert_focus_test_id(&snap, "menubar-item-menu-window-submenu-split");
        assert_not_exists(&snap, "menubar-item-workspace-pane-split-right");

        // Escape closes menu and returns focus to the menubar trigger.
        press_key(&mut ui, &mut app, &mut services, fret_core::KeyCode::Escape);
        let snap = render_frame(&mut ui, &mut app, &mut services, window, bounds, &bar);
        assert_focus_test_id(&snap, "menubar-trigger-window");
        assert_not_exists(&snap, "menubar-item-menu-window-submenu-split");

        // Avoid unused warnings if the effect buffer changes across versions.
        let _ = app.flush_effects();
    }
}
