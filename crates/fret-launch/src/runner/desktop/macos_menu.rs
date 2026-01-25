#![cfg(target_os = "macos")]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use cocoa::{
    appkit::{
        NSApp, NSApplication, NSDeleteFunctionKey, NSDownArrowFunctionKey, NSEndFunctionKey,
        NSEventModifierFlags, NSF1FunctionKey, NSF2FunctionKey, NSF3FunctionKey, NSF4FunctionKey,
        NSF5FunctionKey, NSF6FunctionKey, NSF7FunctionKey, NSF8FunctionKey, NSF9FunctionKey,
        NSF10FunctionKey, NSF11FunctionKey, NSF12FunctionKey, NSF13FunctionKey, NSF14FunctionKey,
        NSF15FunctionKey, NSF16FunctionKey, NSF17FunctionKey, NSF18FunctionKey, NSF19FunctionKey,
        NSF20FunctionKey, NSF21FunctionKey, NSF22FunctionKey, NSF23FunctionKey, NSF24FunctionKey,
        NSF25FunctionKey, NSF26FunctionKey, NSF27FunctionKey, NSF28FunctionKey, NSF29FunctionKey,
        NSF30FunctionKey, NSF31FunctionKey, NSF32FunctionKey, NSF33FunctionKey, NSF34FunctionKey,
        NSF35FunctionKey, NSHomeFunctionKey, NSLeftArrowFunctionKey, NSMenu, NSMenuItem,
        NSPageDownFunctionKey, NSPageUpFunctionKey, NSRightArrowFunctionKey, NSUpArrowFunctionKey,
    },
    base::{id, nil},
    foundation::NSInteger,
};
use fret_core::{AppWindowId, KeyCode};
use fret_runtime::{
    CommandId, CommandScope, InputContext, InputDispatchPhase, Keymap, KeymapService, MenuBar,
    MenuItem, MenuRole, OsAction, Platform, PlatformCapabilities, SystemMenuType, WhenExpr,
    WindowCommandGatingSnapshot,
};
use objc::{
    declare::ClassDecl,
    msg_send,
    runtime::{BOOL, Class, NO, Object, Sel, YES},
    sel, sel_impl,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::{event_loop::EventLoopProxy, window::Window};

use super::RunnerUserEvent;

static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy> = OnceLock::new();
static PROXY_EVENTS: OnceLock<Arc<Mutex<Vec<RunnerUserEvent>>>> = OnceLock::new();
static MENU_STATE: OnceLock<Mutex<MacosMenuState>> = OnceLock::new();
static MENU_DELEGATE_CLASS: OnceLock<&'static Class> = OnceLock::new();

#[derive(Debug, Clone)]
struct MacosMenuItemDef {
    command: CommandId,
    command_when: Option<WhenExpr>,
    item_when: Option<WhenExpr>,
    os_action: Option<OsAction>,
    command_scope: CommandScope,
}

#[derive(Debug)]
struct MacosMenuState {
    delegate: id,
    main_menu: id,
    tag_to_def: HashMap<NSInteger, MacosMenuItemDef>,
    ns_window_to_app_window: HashMap<isize, AppWindowId>,
    cached_keymap: Keymap,
    cached_caps: PlatformCapabilities,
    cached_gating_by_window: HashMap<AppWindowId, WindowCommandGatingSnapshot>,
    next_tag: NSInteger,
}

impl Default for MacosMenuState {
    fn default() -> Self {
        Self {
            delegate: nil,
            main_menu: nil,
            tag_to_def: HashMap::new(),
            ns_window_to_app_window: HashMap::new(),
            cached_keymap: Keymap::default(),
            cached_caps: PlatformCapabilities::default(),
            cached_gating_by_window: HashMap::new(),
            next_tag: 1,
        }
    }
}

pub(crate) fn set_event_loop_proxy(
    proxy: EventLoopProxy,
    events: Arc<Mutex<Vec<RunnerUserEvent>>>,
) {
    let _ = EVENT_LOOP_PROXY.set(proxy);
    let _ = PROXY_EVENTS.set(events);
}

pub(crate) fn register_window(window: &dyn Window, app_window: AppWindowId) {
    let Some(ns_window) = ns_window_id(window) else {
        return;
    };
    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(mut state) = state.lock() else {
        return;
    };
    state
        .ns_window_to_app_window
        .insert(ns_window as isize, app_window);
}

pub(crate) fn unregister_window(window: &dyn Window) {
    let Some(ns_window) = ns_window_id(window) else {
        return;
    };
    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(mut state) = state.lock() else {
        return;
    };
    state.ns_window_to_app_window.remove(&(ns_window as isize));
}

pub(crate) fn sync_keymap_from_app(app: &fret_app::App) {
    let keymap = app
        .global::<KeymapService>()
        .map(|svc| svc.keymap.clone())
        .unwrap_or_default();

    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(mut state) = state.lock() else {
        return;
    };
    state.cached_keymap = keymap;
    state.cached_caps = caps;
}

pub(crate) fn sync_command_gating_from_app(app: &fret_app::App) {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let windows: Vec<AppWindowId> = {
        let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
        let Ok(state) = state.lock() else {
            return;
        };
        state.ns_window_to_app_window.values().copied().collect()
    };

    let mut by_window: HashMap<AppWindowId, WindowCommandGatingSnapshot> = HashMap::new();
    for window in windows {
        let fallback_input_ctx = InputContext::fallback(Platform::Macos, caps.clone());
        let snapshot = fret_runtime::best_effort_snapshot_for_window_with_input_ctx_fallback(
            app,
            window,
            fallback_input_ctx,
        );
        by_window.insert(window, snapshot);
    }

    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(mut state) = state.lock() else {
        return;
    };
    state.cached_caps = caps;
    for (window, snapshot) in by_window {
        state.cached_gating_by_window.insert(window, snapshot);
    }
}

pub(crate) fn set_app_menu_bar(app: &fret_app::App, menu_bar: &MenuBar) {
    let delegate_class = MENU_DELEGATE_CLASS.get_or_init(menu_delegate_class);
    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));

    let (commands, keymap, caps) = {
        let commands = app.commands().clone();
        let keymap = app
            .global::<KeymapService>()
            .map(|svc| svc.keymap.clone())
            .unwrap_or_default();
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        (commands, keymap, caps)
    };

    let base_ctx = InputContext::fallback(Platform::Macos, caps.clone());

    let Ok(mut state) = state.lock() else {
        return;
    };
    state.cached_keymap = keymap;
    state.cached_caps = caps;

    unsafe {
        if state.delegate == nil {
            let delegate: id = msg_send![delegate_class, new];
            state.delegate = delegate;
        }

        let main_menu = NSMenu::new(nil).autorelease();

        state.tag_to_def.clear();
        state.next_tag = 1;

        let mut app_menu: Option<&fret_runtime::Menu> = None;
        let mut other_menus: Vec<&fret_runtime::Menu> = Vec::new();
        for menu in &menu_bar.menus {
            if menu.role == Some(MenuRole::App) && app_menu.is_none() {
                app_menu = Some(menu);
            } else {
                other_menus.push(menu);
            }
        }
        let menus_iter = app_menu.into_iter().chain(other_menus);

        for menu in menus_iter {
            let submenu = NSMenu::new(nil).autorelease();
            let title = ns_string(&menu.title);
            submenu.setTitle_(title);
            submenu.setDelegate_(state.delegate);

            for item in &menu.items {
                append_menu_item(&mut state, submenu, item, &commands, &base_ctx);
            }

            let menu_item = NSMenuItem::new(nil).autorelease();
            menu_item.setTitle_(title);
            menu_item.setSubmenu_(submenu);
            main_menu.addItem_(menu_item);

            if menu.role == Some(MenuRole::Window) {
                let app = NSApp();
                app.setWindowsMenu_(submenu);
            }

            if menu.role == Some(MenuRole::Help) {
                let app = NSApp();
                let _: () = msg_send![app, setHelpMenu: submenu];
            }

            if menu.role == Some(MenuRole::App) {
                let app = NSApp();
                let _: () = msg_send![app, setAppleMenu: submenu];
            }
        }

        let app = NSApp();
        let _: () = msg_send![app, setMainMenu: main_menu];

        state.main_menu = main_menu;
    }
}

pub(crate) fn hide_app() {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, hide: nil];
    }
}

pub(crate) fn show_about_panel() {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, orderFrontStandardAboutPanel: nil];
    }
}

pub(crate) fn hide_other_apps() {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, hideOtherApplications: nil];
    }
}

pub(crate) fn unhide_all_apps() {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, unhideAllApplications: nil];
    }
}

unsafe fn append_menu_item(
    state: &mut MacosMenuState,
    menu: id,
    item: &MenuItem,
    commands: &fret_runtime::CommandRegistry,
    base_ctx: &InputContext,
) {
    match item {
        MenuItem::Separator => {
            let sep = NSMenuItem::separatorItem(nil);
            menu.addItem_(sep);
        }
        MenuItem::SystemMenu { title, menu_type } => {
            let system_item = NSMenuItem::new(nil).autorelease();
            system_item.setTitle_(ns_string(title));
            let submenu = NSMenu::new(nil).autorelease();
            submenu.setTitle_(ns_string(title));
            submenu.setDelegate_(state.delegate);
            system_item.setSubmenu_(submenu);

            match menu_type {
                SystemMenuType::Services => {
                    let app = NSApp();
                    app.setServicesMenu_(system_item);
                }
            }

            menu.addItem_(system_item);
        }
        MenuItem::Submenu { title, items, .. } => {
            let submenu_item = NSMenuItem::new(nil).autorelease();
            submenu_item.setTitle_(ns_string(title));

            let submenu = NSMenu::new(nil).autorelease();
            submenu.setTitle_(ns_string(title));
            submenu.setDelegate_(state.delegate);
            for item in items {
                append_menu_item(state, submenu, item, commands, base_ctx);
            }
            submenu_item.setSubmenu_(submenu);
            menu.addItem_(submenu_item);
        }
        MenuItem::Command { command, when } => {
            let (label, command_when, os_action, command_scope) =
                match commands.get(command.clone()) {
                    Some(meta) => (
                        meta.title.clone(),
                        meta.when.clone(),
                        meta.os_action,
                        meta.scope,
                    ),
                    None => (
                        Arc::<str>::from(command.as_str()),
                        None,
                        None,
                        CommandScope::Window,
                    ),
                };

            let tag = state.next_tag;
            state.next_tag = state.next_tag.saturating_add(1);
            state.tag_to_def.insert(
                tag,
                MacosMenuItemDef {
                    command: command.clone(),
                    command_when,
                    item_when: when.clone(),
                    os_action,
                    command_scope,
                },
            );

            let selector = match os_action {
                Some(OsAction::Cut) => sel!(cut:),
                Some(OsAction::Copy) => sel!(copy:),
                Some(OsAction::Paste) => sel!(paste:),
                Some(OsAction::SelectAll) => sel!(selectAll:),
                Some(OsAction::Undo) => sel!(undo:),
                Some(OsAction::Redo) => sel!(redo:),
                None => sel!(fretMenuItemInvoked:),
            };
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(ns_string(&label), selector, ns_string(""))
                .autorelease();
            item.setTarget_(state.delegate);
            item.setTag_(tag);

            if let Some(seq) = state
                .cached_keymap
                .display_shortcut_for_command_sequence(base_ctx, command)
                && seq.len() == 1
                && let Some(eq) = key_equivalent_for_code(seq[0].key)
            {
                let mask = chord_modifiers_to_mask(seq[0]);
                item.setKeyEquivalent_(ns_string(&eq));
                item.setKeyEquivalentModifierMask_(mask);
            }

            menu.addItem_(item);
        }
    }
}

fn chord_modifiers_to_mask(chord: fret_runtime::KeyChord) -> NSEventModifierFlags {
    let mut mask = NSEventModifierFlags::empty();
    if chord.mods.meta {
        mask |= NSEventModifierFlags::NSCommandKeyMask;
    }
    if chord.mods.ctrl {
        mask |= NSEventModifierFlags::NSControlKeyMask;
    }
    if chord.mods.alt || chord.mods.alt_gr {
        mask |= NSEventModifierFlags::NSAlternateKeyMask;
    }
    if chord.mods.shift {
        mask |= NSEventModifierFlags::NSShiftKeyMask;
    }
    mask
}

fn key_equivalent_for_code(code: KeyCode) -> Option<String> {
    use keyboard_types::Code;

    let from_u16 = |v: u16| String::from_utf16(&[v]).ok();

    Some(match code {
        Code::KeyA => "a".to_string(),
        Code::KeyB => "b".to_string(),
        Code::KeyC => "c".to_string(),
        Code::KeyD => "d".to_string(),
        Code::KeyE => "e".to_string(),
        Code::KeyF => "f".to_string(),
        Code::KeyG => "g".to_string(),
        Code::KeyH => "h".to_string(),
        Code::KeyI => "i".to_string(),
        Code::KeyJ => "j".to_string(),
        Code::KeyK => "k".to_string(),
        Code::KeyL => "l".to_string(),
        Code::KeyM => "m".to_string(),
        Code::KeyN => "n".to_string(),
        Code::KeyO => "o".to_string(),
        Code::KeyP => "p".to_string(),
        Code::KeyQ => "q".to_string(),
        Code::KeyR => "r".to_string(),
        Code::KeyS => "s".to_string(),
        Code::KeyT => "t".to_string(),
        Code::KeyU => "u".to_string(),
        Code::KeyV => "v".to_string(),
        Code::KeyW => "w".to_string(),
        Code::KeyX => "x".to_string(),
        Code::KeyY => "y".to_string(),
        Code::KeyZ => "z".to_string(),

        Code::Digit0 => "0".to_string(),
        Code::Digit1 => "1".to_string(),
        Code::Digit2 => "2".to_string(),
        Code::Digit3 => "3".to_string(),
        Code::Digit4 => "4".to_string(),
        Code::Digit5 => "5".to_string(),
        Code::Digit6 => "6".to_string(),
        Code::Digit7 => "7".to_string(),
        Code::Digit8 => "8".to_string(),
        Code::Digit9 => "9".to_string(),

        Code::Space => " ".to_string(),
        Code::Tab => "\t".to_string(),
        Code::Enter | Code::NumpadEnter => "\r".to_string(),
        Code::Escape => String::from_utf16(&[0x1b]).ok()?,
        Code::Backspace => String::from_utf16(&[0x7f]).ok()?,

        Code::ArrowUp => from_u16(NSUpArrowFunctionKey)?,
        Code::ArrowDown => from_u16(NSDownArrowFunctionKey)?,
        Code::ArrowLeft => from_u16(NSLeftArrowFunctionKey)?,
        Code::ArrowRight => from_u16(NSRightArrowFunctionKey)?,
        Code::PageUp => from_u16(NSPageUpFunctionKey)?,
        Code::PageDown => from_u16(NSPageDownFunctionKey)?,
        Code::Home => from_u16(NSHomeFunctionKey)?,
        Code::End => from_u16(NSEndFunctionKey)?,
        Code::Delete => from_u16(NSDeleteFunctionKey)?,

        Code::F1 => from_u16(NSF1FunctionKey)?,
        Code::F2 => from_u16(NSF2FunctionKey)?,
        Code::F3 => from_u16(NSF3FunctionKey)?,
        Code::F4 => from_u16(NSF4FunctionKey)?,
        Code::F5 => from_u16(NSF5FunctionKey)?,
        Code::F6 => from_u16(NSF6FunctionKey)?,
        Code::F7 => from_u16(NSF7FunctionKey)?,
        Code::F8 => from_u16(NSF8FunctionKey)?,
        Code::F9 => from_u16(NSF9FunctionKey)?,
        Code::F10 => from_u16(NSF10FunctionKey)?,
        Code::F11 => from_u16(NSF11FunctionKey)?,
        Code::F12 => from_u16(NSF12FunctionKey)?,
        Code::F13 => from_u16(NSF13FunctionKey)?,
        Code::F14 => from_u16(NSF14FunctionKey)?,
        Code::F15 => from_u16(NSF15FunctionKey)?,
        Code::F16 => from_u16(NSF16FunctionKey)?,
        Code::F17 => from_u16(NSF17FunctionKey)?,
        Code::F18 => from_u16(NSF18FunctionKey)?,
        Code::F19 => from_u16(NSF19FunctionKey)?,
        Code::F20 => from_u16(NSF20FunctionKey)?,
        Code::F21 => from_u16(NSF21FunctionKey)?,
        Code::F22 => from_u16(NSF22FunctionKey)?,
        Code::F23 => from_u16(NSF23FunctionKey)?,
        Code::F24 => from_u16(NSF24FunctionKey)?,
        Code::F25 => from_u16(NSF25FunctionKey)?,
        Code::F26 => from_u16(NSF26FunctionKey)?,
        Code::F27 => from_u16(NSF27FunctionKey)?,
        Code::F28 => from_u16(NSF28FunctionKey)?,
        Code::F29 => from_u16(NSF29FunctionKey)?,
        Code::F30 => from_u16(NSF30FunctionKey)?,
        Code::F31 => from_u16(NSF31FunctionKey)?,
        Code::F32 => from_u16(NSF32FunctionKey)?,
        Code::F33 => from_u16(NSF33FunctionKey)?,
        Code::F34 => from_u16(NSF34FunctionKey)?,
        Code::F35 => from_u16(NSF35FunctionKey)?,

        _ => return None,
    })
}

fn ns_string(s: &str) -> id {
    unsafe {
        cocoa::foundation::NSString::alloc(nil)
            .init_str(s)
            .autorelease()
    }
}

fn ns_window_id(window: &dyn Window) -> Option<id> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::AppKit(h) = handle.as_raw() else {
        return None;
    };
    let ns_view: id = h.ns_view.as_ptr() as id;
    if ns_view == nil {
        return None;
    }
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        (ns_window != nil).then_some(ns_window)
    }
}

fn key_window_ptr() -> Option<isize> {
    unsafe {
        let app = NSApp();
        let key_window: id = msg_send![app, keyWindow];
        if key_window == nil {
            return None;
        }
        Some(key_window as isize)
    }
}

fn main_window_ptr() -> Option<isize> {
    unsafe {
        let app = NSApp();
        let main_window: id = msg_send![app, mainWindow];
        if main_window == nil {
            return None;
        }
        Some(main_window as isize)
    }
}

fn active_app_window_id(state: &MacosMenuState) -> Option<AppWindowId> {
    let from_ptr = |p: isize| state.ns_window_to_app_window.get(&p).copied();

    key_window_ptr()
        .and_then(from_ptr)
        .or_else(|| main_window_ptr().and_then(from_ptr))
        .or_else(|| {
            (state.ns_window_to_app_window.len() == 1).then(|| {
                *state
                    .ns_window_to_app_window
                    .values()
                    .next()
                    .expect("len==1")
            })
        })
}

fn menu_delegate_class() -> &'static Class {
    let superclass = Class::get("NSObject").expect("NSObject class");
    let mut decl = ClassDecl::new("FretMenuDelegate", superclass).expect("FretMenuDelegate class");

    unsafe {
        decl.add_method(
            sel!(fretMenuItemInvoked:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(cut:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(copy:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(paste:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(selectAll:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(undo:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(redo:),
            fret_menu_item_invoked as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(validateMenuItem:),
            fret_validate_menu_item as extern "C" fn(&Object, Sel, id) -> BOOL,
        );
        decl.add_method(
            sel!(menuWillOpen:),
            fret_menu_will_open as extern "C" fn(&Object, Sel, id),
        );
    }

    decl.register()
}

extern "C" fn fret_menu_item_invoked(_this: &Object, _cmd: Sel, item: id) {
    let tag: NSInteger = unsafe { msg_send![item, tag] };
    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(state) = state.lock() else {
        return;
    };
    let Some(def) = state.tag_to_def.get(&tag) else {
        return;
    };
    let window = active_app_window_id(&state);
    let command = def.command.clone();

    let Some(events) = PROXY_EVENTS.get() else {
        return;
    };
    let Some(proxy) = EVENT_LOOP_PROXY.get() else {
        return;
    };
    if let Ok(mut queue) = events.lock() {
        queue.push(RunnerUserEvent::MacosMenuCommand { window, command });
    }
    proxy.wake_up();
}

extern "C" fn fret_menu_will_open(_this: &Object, _cmd: Sel, _menu: id) {
    let Some(events) = PROXY_EVENTS.get() else {
        return;
    };
    let Some(proxy) = EVENT_LOOP_PROXY.get() else {
        return;
    };
    if let Ok(mut queue) = events.lock() {
        queue.push(RunnerUserEvent::MacosMenuWillOpen);
    }
    proxy.wake_up();
}

extern "C" fn fret_validate_menu_item(_this: &Object, _cmd: Sel, item: id) -> BOOL {
    let tag: NSInteger = unsafe { msg_send![item, tag] };

    let state = MENU_STATE.get_or_init(|| Mutex::new(MacosMenuState::default()));
    let Ok(state) = state.lock() else {
        return YES;
    };
    let Some(def) = state.tag_to_def.get(&tag) else {
        return YES;
    };

    let active_window = active_app_window_id(&state);

    let caps = state.cached_caps.clone();
    let fallback = InputContext::fallback(Platform::Macos, caps);

    let gating = active_window
        .and_then(|w| state.cached_gating_by_window.get(&w).cloned())
        .unwrap_or_else(|| WindowCommandGatingSnapshot::new(fallback, HashMap::new()));

    let enabled =
        gating.is_enabled_for_meta(&def.command, def.command_scope, def.command_when.as_ref())
            && def
                .item_when
                .as_ref()
                .map(|w| w.eval(gating.input_ctx()))
                .unwrap_or(true);
    if enabled { YES } else { NO }
}
