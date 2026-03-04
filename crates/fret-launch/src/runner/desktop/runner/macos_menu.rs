use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use fret_core::{AppWindowId, KeyCode};
use fret_runtime::{
    CommandId, CommandScope, InputContext, InputDispatchPhase, Keymap, KeymapService, MenuBar,
    MenuItem, MenuRole, OsAction, Platform, PlatformCapabilities, SystemMenuType, WhenExpr,
    WindowCommandGatingSnapshot,
};
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, NSObject, Sel};
use objc2::{ClassType, MainThreadMarker, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSDeleteFunctionKey, NSDownArrowFunctionKey, NSEndFunctionKey,
    NSEventModifierFlags, NSF1FunctionKey, NSF2FunctionKey, NSF3FunctionKey, NSF4FunctionKey,
    NSF5FunctionKey, NSF6FunctionKey, NSF7FunctionKey, NSF8FunctionKey, NSF9FunctionKey,
    NSF10FunctionKey, NSF11FunctionKey, NSF12FunctionKey, NSF13FunctionKey, NSF14FunctionKey,
    NSF15FunctionKey, NSF16FunctionKey, NSF17FunctionKey, NSF18FunctionKey, NSF19FunctionKey,
    NSF20FunctionKey, NSF21FunctionKey, NSF22FunctionKey, NSF23FunctionKey, NSF24FunctionKey,
    NSF25FunctionKey, NSF26FunctionKey, NSF27FunctionKey, NSF28FunctionKey, NSF29FunctionKey,
    NSF30FunctionKey, NSF31FunctionKey, NSF32FunctionKey, NSF33FunctionKey, NSF34FunctionKey,
    NSF35FunctionKey, NSHomeFunctionKey, NSLeftArrowFunctionKey, NSMenu, NSMenuItem,
    NSPageDownFunctionKey, NSPageUpFunctionKey, NSRightArrowFunctionKey, NSUpArrowFunctionKey,
};
use objc2_foundation::{NSInteger, NSString};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::{event_loop::EventLoopProxy, window::Window};

use super::RunnerUserEvent;

static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy> = OnceLock::new();
static PROXY_EVENTS: OnceLock<Arc<Mutex<Vec<RunnerUserEvent>>>> = OnceLock::new();
static MENU_DELEGATE_CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

thread_local! {
    static MENU_STATE: RefCell<MacosMenuState> = RefCell::new(MacosMenuState::default());
}

#[derive(Debug, Clone)]
struct MacosMenuItemDef {
    command: CommandId,
    command_when: Option<WhenExpr>,
    item_when: Option<WhenExpr>,
    command_scope: CommandScope,
}

#[derive(Debug)]
struct MacosMenuState {
    delegate: Option<Retained<AnyObject>>,
    main_menu: Option<Retained<NSMenu>>,
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
            delegate: None,
            main_menu: None,
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
    MENU_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state
            .ns_window_to_app_window
            .insert(ns_window as isize, app_window);
    });
}

pub(crate) fn unregister_window(window: &dyn Window) {
    let Some(ns_window) = ns_window_id(window) else {
        return;
    };
    MENU_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.ns_window_to_app_window.remove(&(ns_window as isize));
    });
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

    MENU_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.cached_keymap = keymap;
        state.cached_caps = caps;
    });
}

pub(crate) fn sync_command_gating_from_app(app: &fret_app::App) {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let windows: Vec<AppWindowId> = {
        let mut out = Vec::new();
        MENU_STATE.with(|state| {
            let Ok(state) = state.try_borrow() else {
                return;
            };
            out.extend(state.ns_window_to_app_window.values().copied());
        });
        out
    };

    let mut by_window: HashMap<AppWindowId, WindowCommandGatingSnapshot> = HashMap::new();
    for window in windows {
        let fallback_input_ctx = InputContext {
            platform: Platform::Macos,
            caps: caps.clone(),
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
        let snapshot = fret_runtime::snapshot_for_window_with_input_ctx_fallback(
            app,
            window,
            fallback_input_ctx,
        );
        by_window.insert(window, snapshot);
    }

    MENU_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.cached_caps = caps;
        for (window, snapshot) in by_window {
            state.cached_gating_by_window.insert(window, snapshot);
        }
    });
}

pub(crate) fn set_app_menu_bar(app: &fret_app::App, menu_bar: &MenuBar) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let delegate_class: &'static AnyClass = MENU_DELEGATE_CLASS.get_or_init(menu_delegate_class);

    let normalized_menu_bar = menu_bar.clone().normalized();

    let (commands, keymap, caps) = {
        let commands = app.commands();
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

    let base_ctx = InputContext {
        platform: Platform::Macos,
        caps: caps.clone(),
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

    MENU_STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.cached_keymap = keymap;
        state.cached_caps = caps;

        if state.delegate.is_none() {
            let delegate: Retained<AnyObject> = unsafe { msg_send![delegate_class, new] };
            state.delegate = Some(delegate);
        }
        let delegate_ptr: Option<*const AnyObject> = state.delegate.as_ref().map(Retained::as_ptr);

        let appkit_app = NSApplication::sharedApplication(mtm);
        let main_menu = NSMenu::new(mtm);
        state.tag_to_def.clear();
        state.next_tag = 1;

        let mut app_menu: Option<&fret_runtime::Menu> = None;
        let mut other_menus: Vec<&fret_runtime::Menu> = Vec::new();
        for menu in &normalized_menu_bar.menus {
            if menu.role == Some(MenuRole::App) && app_menu.is_none() {
                app_menu = Some(menu);
            } else {
                other_menus.push(menu);
            }
        }
        let menus_iter = app_menu.into_iter().chain(other_menus);

        for menu in menus_iter {
            let title = NSString::from_str(&menu.title);
            let submenu = NSMenu::new(mtm);
            submenu.setTitle(&title);
            unsafe {
                let delegate = delegate_ptr.map(|p| &*p);
                let _: () = msg_send![&submenu, setDelegate: delegate];
            }

            for item in &menu.items {
                append_menu_item(mtm, &mut state, &submenu, item, commands, &base_ctx);
            }

            let menu_item = NSMenuItem::new(mtm);
            menu_item.setTitle(&title);
            menu_item.setSubmenu(Some(&submenu));
            main_menu.addItem(&menu_item);

            if menu.role == Some(MenuRole::Window) {
                appkit_app.setWindowsMenu(Some(&submenu));
            }

            if menu.role == Some(MenuRole::Help) {
                appkit_app.setHelpMenu(Some(&submenu));
            }

            if menu.role == Some(MenuRole::App) {
                unsafe {
                    let _: () = msg_send![&appkit_app, setAppleMenu: &*submenu];
                }
            }
        }

        appkit_app.setMainMenu(Some(&main_menu));

        state.main_menu = Some(main_menu);
    });
}

pub(crate) fn hide_app() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.hide(None);
}

pub(crate) fn show_about_panel() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.orderFrontStandardAboutPanel(None);
}

pub(crate) fn hide_other_apps() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.hideOtherApplications(None);
}

pub(crate) fn unhide_all_apps() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.unhideAllApplications(None);
}

fn append_menu_item(
    mtm: MainThreadMarker,
    state: &mut MacosMenuState,
    menu: &NSMenu,
    item: &MenuItem,
    commands: &fret_runtime::CommandRegistry,
    base_ctx: &InputContext,
) {
    let delegate_ptr: Option<*const AnyObject> = state.delegate.as_ref().map(Retained::as_ptr);

    match item {
        MenuItem::Separator => {
            let sep = NSMenuItem::separatorItem(mtm);
            menu.addItem(&sep);
        }
        MenuItem::SystemMenu { title, menu_type } => {
            let title = NSString::from_str(title);
            let system_item = NSMenuItem::new(mtm);
            system_item.setTitle(&title);

            let submenu = NSMenu::new(mtm);
            submenu.setTitle(&title);
            unsafe {
                let delegate = delegate_ptr.map(|p| &*p);
                let _: () = msg_send![&submenu, setDelegate: delegate];
            }
            system_item.setSubmenu(Some(&submenu));

            match menu_type {
                SystemMenuType::Services => {
                    let app = NSApplication::sharedApplication(mtm);
                    app.setServicesMenu(Some(&submenu));
                }
            }

            menu.addItem(&system_item);
        }
        MenuItem::Label { title } => {
            let title = NSString::from_str(title);
            let item = NSMenuItem::new(mtm);
            item.setTitle(&title);
            item.setEnabled(false);
            menu.addItem(&item);
        }
        MenuItem::Submenu { title, items, .. } => {
            let title = NSString::from_str(title);
            let submenu_item = NSMenuItem::new(mtm);
            submenu_item.setTitle(&title);

            let submenu = NSMenu::new(mtm);
            submenu.setTitle(&title);
            unsafe {
                let delegate = delegate_ptr.map(|p| &*p);
                let _: () = msg_send![&submenu, setDelegate: delegate];
            }

            for item in items {
                append_menu_item(mtm, state, &submenu, item, commands, base_ctx);
            }
            submenu_item.setSubmenu(Some(&submenu));
            menu.addItem(&submenu_item);
        }
        MenuItem::Command { command, when, .. } => {
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
            let label = NSString::from_str(&label);
            let empty = NSString::from_str("");
            let item = NSMenuItem::new(mtm);
            item.setTitle(&label);
            item.setKeyEquivalent(&empty);
            unsafe {
                item.setAction(Some(selector));
            }
            if let Some(delegate_ptr) = delegate_ptr {
                unsafe {
                    item.setTarget(Some(&*delegate_ptr));
                }
            }
            item.setTag(tag);

            if let Some(seq) = state
                .cached_keymap
                .display_shortcut_for_command_sequence(base_ctx, command)
                && seq.len() == 1
                && let Some(eq) = key_equivalent_for_code(seq[0].key)
            {
                let mask = chord_modifiers_to_mask(seq[0]);
                let eq = NSString::from_str(&eq);
                item.setKeyEquivalent(&eq);
                item.setKeyEquivalentModifierMask(mask);
            }

            menu.addItem(&item);
        }
    }
}

fn chord_modifiers_to_mask(chord: fret_runtime::KeyChord) -> NSEventModifierFlags {
    let mut mask = NSEventModifierFlags::empty();
    if chord.mods.meta {
        mask |= NSEventModifierFlags::Command;
    }
    if chord.mods.ctrl {
        mask |= NSEventModifierFlags::Control;
    }
    if chord.mods.alt || chord.mods.alt_gr {
        mask |= NSEventModifierFlags::Option;
    }
    if chord.mods.shift {
        mask |= NSEventModifierFlags::Shift;
    }
    mask
}

fn key_equivalent_for_code(code: KeyCode) -> Option<String> {
    use keyboard_types::Code;

    let from_u32 = |v: u32| char::from_u32(v).map(|c| c.to_string());

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

        Code::ArrowUp => from_u32(NSUpArrowFunctionKey)?,
        Code::ArrowDown => from_u32(NSDownArrowFunctionKey)?,
        Code::ArrowLeft => from_u32(NSLeftArrowFunctionKey)?,
        Code::ArrowRight => from_u32(NSRightArrowFunctionKey)?,
        Code::PageUp => from_u32(NSPageUpFunctionKey)?,
        Code::PageDown => from_u32(NSPageDownFunctionKey)?,
        Code::Home => from_u32(NSHomeFunctionKey)?,
        Code::End => from_u32(NSEndFunctionKey)?,
        Code::Delete => from_u32(NSDeleteFunctionKey)?,

        Code::F1 => from_u32(NSF1FunctionKey)?,
        Code::F2 => from_u32(NSF2FunctionKey)?,
        Code::F3 => from_u32(NSF3FunctionKey)?,
        Code::F4 => from_u32(NSF4FunctionKey)?,
        Code::F5 => from_u32(NSF5FunctionKey)?,
        Code::F6 => from_u32(NSF6FunctionKey)?,
        Code::F7 => from_u32(NSF7FunctionKey)?,
        Code::F8 => from_u32(NSF8FunctionKey)?,
        Code::F9 => from_u32(NSF9FunctionKey)?,
        Code::F10 => from_u32(NSF10FunctionKey)?,
        Code::F11 => from_u32(NSF11FunctionKey)?,
        Code::F12 => from_u32(NSF12FunctionKey)?,
        Code::F13 => from_u32(NSF13FunctionKey)?,
        Code::F14 => from_u32(NSF14FunctionKey)?,
        Code::F15 => from_u32(NSF15FunctionKey)?,
        Code::F16 => from_u32(NSF16FunctionKey)?,
        Code::F17 => from_u32(NSF17FunctionKey)?,
        Code::F18 => from_u32(NSF18FunctionKey)?,
        Code::F19 => from_u32(NSF19FunctionKey)?,
        Code::F20 => from_u32(NSF20FunctionKey)?,
        Code::F21 => from_u32(NSF21FunctionKey)?,
        Code::F22 => from_u32(NSF22FunctionKey)?,
        Code::F23 => from_u32(NSF23FunctionKey)?,
        Code::F24 => from_u32(NSF24FunctionKey)?,
        Code::F25 => from_u32(NSF25FunctionKey)?,
        Code::F26 => from_u32(NSF26FunctionKey)?,
        Code::F27 => from_u32(NSF27FunctionKey)?,
        Code::F28 => from_u32(NSF28FunctionKey)?,
        Code::F29 => from_u32(NSF29FunctionKey)?,
        Code::F30 => from_u32(NSF30FunctionKey)?,
        Code::F31 => from_u32(NSF31FunctionKey)?,
        Code::F32 => from_u32(NSF32FunctionKey)?,
        Code::F33 => from_u32(NSF33FunctionKey)?,
        Code::F34 => from_u32(NSF34FunctionKey)?,
        Code::F35 => from_u32(NSF35FunctionKey)?,

        _ => return None,
    })
}

fn ns_window_id(window: &dyn Window) -> Option<*mut AnyObject> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::AppKit(h) = handle.as_raw() else {
        return None;
    };
    let ns_view = h.ns_view.as_ptr().cast::<AnyObject>();
    if ns_view.is_null() {
        return None;
    }
    unsafe {
        let ns_window: *mut AnyObject = msg_send![ns_view, window];
        (!ns_window.is_null()).then_some(ns_window)
    }
}

fn key_window_ptr() -> Option<isize> {
    let mtm = MainThreadMarker::new()?;
    let app = NSApplication::sharedApplication(mtm);
    let key_window: *mut AnyObject = unsafe { msg_send![&app, keyWindow] };
    (!key_window.is_null()).then_some(key_window as isize)
}

fn main_window_ptr() -> Option<isize> {
    let mtm = MainThreadMarker::new()?;
    let app = NSApplication::sharedApplication(mtm);
    let main_window: *mut AnyObject = unsafe { msg_send![&app, mainWindow] };
    (!main_window.is_null()).then_some(main_window as isize)
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

fn menu_delegate_class() -> &'static AnyClass {
    let mut builder =
        ClassBuilder::new(c"FretMenuDelegate", NSObject::class()).expect("FretMenuDelegate class");
    unsafe {
        builder.add_method(
            sel!(fretMenuItemInvoked:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(cut:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(copy:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(paste:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(selectAll:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(undo:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(redo:),
            fret_menu_item_invoked as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem),
        );
        builder.add_method(
            sel!(validateMenuItem:),
            fret_validate_menu_item
                as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenuItem) -> Bool,
        );
        builder.add_method(
            sel!(menuWillOpen:),
            fret_menu_will_open as extern "C-unwind" fn(*mut AnyObject, Sel, *mut NSMenu),
        );
    }

    builder.register()
}

extern "C-unwind" fn fret_menu_item_invoked(
    _this: *mut AnyObject,
    _cmd: Sel,
    item: *mut NSMenuItem,
) {
    let Some(item) = (unsafe { item.as_ref() }) else {
        return;
    };
    let tag: NSInteger = item.tag();
    let Some((window, command)) = MENU_STATE.with(|state| {
        let Ok(state) = state.try_borrow() else {
            return None;
        };
        let def = state.tag_to_def.get(&tag)?;
        let window = active_app_window_id(&state);
        Some((window, def.command.clone()))
    }) else {
        return;
    };

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

extern "C-unwind" fn fret_menu_will_open(_this: *mut AnyObject, _cmd: Sel, _menu: *mut NSMenu) {
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

extern "C-unwind" fn fret_validate_menu_item(
    _this: *mut AnyObject,
    _cmd: Sel,
    item: *mut NSMenuItem,
) -> Bool {
    let Some(item) = (unsafe { item.as_ref() }) else {
        return Bool::YES;
    };
    let tag: NSInteger = item.tag();

    MENU_STATE.with(|state| {
        let Ok(state) = state.try_borrow() else {
            return Bool::YES;
        };
        let Some(def) = state.tag_to_def.get(&tag) else {
            return Bool::YES;
        };

        let active_window = active_app_window_id(&state);

        let caps = state.cached_caps.clone();
        let fallback = InputContext {
            platform: Platform::Macos,
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

        let gating = active_window
            .and_then(|w| state.cached_gating_by_window.get(&w).cloned())
            .unwrap_or_else(|| WindowCommandGatingSnapshot::new(fallback, HashMap::new()));

        let enabled =
            gating.is_enabled_for_meta(&def.command, def.command_scope, def.command_when.as_ref())
                && def
                    .item_when
                    .as_ref()
                    .map(|w| w.eval_with_key_contexts(gating.input_ctx(), gating.key_contexts()))
                    .unwrap_or(true);
        Bool::new(enabled)
    })
}
