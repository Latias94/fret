use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use fret_core::AppWindowId;
use fret_runtime::{
    CommandId, InputContext, Keymap, KeymapService, MenuBar, MenuItem, Platform, WhenExpr,
    WindowCommandEnabledService, WindowInputContextService,
};
use winit::event_loop::EventLoopProxy;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use super::RunnerUserEvent;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HWND;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateMenu, CreatePopupMenu, DestroyMenu, DrawMenuBar, HMENU, MF_POPUP,
    MF_SEPARATOR, MF_STRING, SetMenu,
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnableMenuItem, GetMenuItemCount, GetMenuItemID, MF_BYCOMMAND, MF_ENABLED, MF_GRAYED,
    ModifyMenuW, WM_COMMAND, WM_INITMENUPOPUP,
};

#[derive(Debug, Clone)]
struct WindowsMenuItemDef {
    command: CommandId,
    label: Arc<str>,
    command_when: Option<WhenExpr>,
    item_when: Option<WhenExpr>,
}

#[derive(Debug, Default)]
struct WindowsMenuHookState {
    hwnd_to_app_window: HashMap<isize, AppWindowId>,
    hwnd_to_item_defs: HashMap<isize, HashMap<u16, WindowsMenuItemDef>>,
    hwnd_to_command_enabled: HashMap<isize, HashMap<CommandId, bool>>,
    hwnd_to_input_ctx: HashMap<isize, InputContext>,
    cached_keymap: Arc<Keymap>,
}

static MENU_HOOK_STATE: LazyLock<Mutex<WindowsMenuHookState>> =
    LazyLock::new(|| Mutex::new(WindowsMenuHookState::default()));
static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy> = OnceLock::new();
static PROXY_EVENTS: OnceLock<Arc<Mutex<Vec<RunnerUserEvent>>>> = OnceLock::new();

pub(crate) fn set_event_loop_proxy(
    proxy: EventLoopProxy,
    events: Arc<Mutex<Vec<RunnerUserEvent>>>,
) {
    let _ = EVENT_LOOP_PROXY.set(proxy);
    let _ = PROXY_EVENTS.set(events);
}

pub(crate) fn register_window(window: &dyn Window, app_window: AppWindowId) {
    let Some(hwnd) = hwnd_for_window(window) else {
        return;
    };
    let Ok(mut state) = MENU_HOOK_STATE.lock() else {
        return;
    };
    state.hwnd_to_app_window.insert(hwnd as isize, app_window);
}

pub(crate) fn unregister_window(window: &dyn Window) {
    let Some(hwnd) = hwnd_for_window(window) else {
        return;
    };
    let Ok(mut state) = MENU_HOOK_STATE.lock() else {
        return;
    };
    state.hwnd_to_app_window.remove(&(hwnd as isize));
    state.hwnd_to_item_defs.remove(&(hwnd as isize));
    state.hwnd_to_command_enabled.remove(&(hwnd as isize));
    state.hwnd_to_input_ctx.remove(&(hwnd as isize));
}

pub(crate) fn msg_hook(msg: *const c_void) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::MSG;

        if msg.is_null() {
            return false;
        }

        let msg = msg as *const MSG;
        let message = unsafe { (*msg).message };
        if message == WM_INITMENUPOPUP {
            let hwnd = unsafe { (*msg).hwnd } as isize;
            let popup = unsafe { (*msg).wParam } as HMENU;
            apply_popup_menu_state(hwnd, popup);
            return false;
        }
        if message != WM_COMMAND {
            return false;
        }

        let Some(proxy) = EVENT_LOOP_PROXY.get() else {
            return false;
        };
        let Some(events) = PROXY_EVENTS.get() else {
            return false;
        };

        // Only handle menu/accelerator commands (lParam == 0 indicates menu/accelerator).
        let lparam = unsafe { (*msg).lParam };
        if lparam != 0 {
            return false;
        }

        let hwnd = unsafe { (*msg).hwnd } as isize;
        let wparam = unsafe { (*msg).wParam } as u32;
        let item_id: u16 = (wparam & 0xFFFF) as u16;

        let (app_window, command) = {
            let Ok(state) = MENU_HOOK_STATE.lock() else {
                return false;
            };
            let Some(app_window) = state.hwnd_to_app_window.get(&hwnd).copied() else {
                return false;
            };
            let Some(map) = state.hwnd_to_item_defs.get(&hwnd) else {
                return false;
            };
            let Some(def) = map.get(&item_id) else {
                return false;
            };
            (app_window, def.command.clone())
        };

        if let Ok(mut queue) = events.lock() {
            queue.push(RunnerUserEvent::WindowsMenuCommand {
                window: app_window,
                command,
            });
        }
        proxy.wake_up();
        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = msg;
        false
    }
}

#[cfg(target_os = "windows")]
fn apply_popup_menu_state(hwnd: isize, popup: HMENU) {
    if popup.is_null() {
        return;
    }

    let count = unsafe { GetMenuItemCount(popup) };
    if count <= 0 {
        return;
    }

    let mut ids: Vec<u16> = Vec::new();
    for index in 0..count {
        let id = unsafe { GetMenuItemID(popup, index) };
        if id == u32::MAX {
            continue;
        }
        let Ok(id) = u16::try_from(id) else {
            continue;
        };
        ids.push(id);
    }

    if ids.is_empty() {
        return;
    }

    let mut updates: Vec<(u16, Vec<u16>, bool)> = Vec::new();
    {
        let Ok(state) = MENU_HOOK_STATE.lock() else {
            return;
        };
        let Some(defs_by_id) = state.hwnd_to_item_defs.get(&hwnd) else {
            return;
        };
        let keymap = state.cached_keymap.clone();
        let command_enabled = state
            .hwnd_to_command_enabled
            .get(&hwnd)
            .cloned()
            .unwrap_or_default();
        let input_ctx = state
            .hwnd_to_input_ctx
            .get(&hwnd)
            .cloned()
            .unwrap_or(InputContext {
                platform: Platform::Windows,
                caps: Default::default(),
                ui_has_modal: false,
                focus_is_text_input: false,
                edit_can_undo: true,
                edit_can_redo: true,
                dispatch_phase: Default::default(),
            });

        for id in ids {
            let Some(def) = defs_by_id.get(&id) else {
                continue;
            };

            let enabled = def
                .command_when
                .as_ref()
                .map(|w| w.eval(&input_ctx))
                .unwrap_or(true)
                && def
                    .item_when
                    .as_ref()
                    .map(|w| w.eval(&input_ctx))
                    .unwrap_or(true);
            let enabled = enabled && command_enabled.get(&def.command).copied().unwrap_or(true);

            let shortcut = keymap.display_shortcut_for_command_sequence(&input_ctx, &def.command);
            let text = if let Some(seq) = shortcut {
                format!(
                    "{}\t{}",
                    def.label,
                    fret_runtime::format_sequence(input_ctx.platform, &seq)
                )
            } else {
                def.label.to_string()
            };

            updates.push((id, to_wide(&text), enabled));
        }
    }

    if updates.is_empty() {
        return;
    }

    for (id, text_wide, enabled) in updates {
        unsafe {
            let _ = ModifyMenuW(
                popup,
                id as u32,
                MF_BYCOMMAND | MF_STRING,
                id as usize,
                text_wide.as_ptr(),
            );
            let flags = MF_BYCOMMAND | if enabled { MF_ENABLED } else { MF_GRAYED };
            let _ = EnableMenuItem(popup, id as u32, flags);
        }
    }
}

#[cfg(target_os = "windows")]
fn hwnd_for_window(window: &dyn Window) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    Some(handle.hwnd.get() as HWND)
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
pub(crate) struct WindowsMenuBar {
    pub(crate) handle: HMENU,
}

#[cfg(target_os = "windows")]
impl WindowsMenuBar {}

#[cfg(target_os = "windows")]
impl Drop for WindowsMenuBar {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }
        unsafe {
            let _ = DestroyMenu(self.handle);
        }
        self.handle = std::ptr::null_mut();
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn set_window_menu_bar(
    app: &fret_app::App,
    window: &dyn Window,
    app_window: AppWindowId,
    menu_bar: &MenuBar,
) -> Option<WindowsMenuBar> {
    let hwnd = hwnd_for_window(window)?;

    let commands = app.commands();
    let keymap = app.global::<KeymapService>().map(|svc| &svc.keymap);
    let caps = app
        .global::<fret_runtime::PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    let input_ctx = app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(app_window))
        .cloned()
        .unwrap_or(InputContext {
            platform: Platform::Windows,
            caps,
            ui_has_modal: false,
            focus_is_text_input: false,
            edit_can_undo: true,
            edit_can_redo: true,
            dispatch_phase: Default::default(),
        });

    let enabled = app
        .global::<WindowCommandEnabledService>()
        .and_then(|svc| svc.snapshot(app_window))
        .cloned()
        .unwrap_or_default();

    let (menu, defs_by_id) = build_menu_bar(menu_bar, commands, keymap, &input_ctx)?;

    unsafe {
        SetMenu(hwnd, menu.handle);
        DrawMenuBar(hwnd);
    }

    let Ok(mut state) = MENU_HOOK_STATE.lock() else {
        return None;
    };
    state.hwnd_to_app_window.insert(hwnd as isize, app_window);
    state.hwnd_to_item_defs.insert(hwnd as isize, defs_by_id);
    state.hwnd_to_command_enabled.insert(hwnd as isize, enabled);
    state.hwnd_to_input_ctx.insert(hwnd as isize, input_ctx);
    state.cached_keymap = Arc::new(keymap.cloned().unwrap_or_default());

    drop(state);
    Some(menu)
}

pub(crate) fn sync_keymap_from_app(app: &fret_app::App) {
    #[cfg(target_os = "windows")]
    {
        let keymap = app
            .global::<KeymapService>()
            .map(|svc| svc.keymap.clone())
            .unwrap_or_default();

        let Ok(mut state) = MENU_HOOK_STATE.lock() else {
            return;
        };
        state.cached_keymap = Arc::new(keymap);
    }
}

pub(crate) fn sync_input_context_from_app(app: &fret_app::App) {
    #[cfg(target_os = "windows")]
    {
        let caps = app
            .global::<fret_runtime::PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let snapshots = app.global::<WindowInputContextService>();

        let windows: Vec<(isize, AppWindowId)> = {
            let Ok(state) = MENU_HOOK_STATE.lock() else {
                return;
            };
            state
                .hwnd_to_app_window
                .iter()
                .map(|(&hwnd, &window)| (hwnd, window))
                .collect()
        };

        let mut by_hwnd: HashMap<isize, InputContext> = HashMap::new();
        for (hwnd, window) in windows {
            let input_ctx = snapshots
                .and_then(|svc| svc.snapshot(window))
                .cloned()
                .unwrap_or(InputContext {
                    platform: Platform::Windows,
                    caps: caps.clone(),
                    ui_has_modal: false,
                    focus_is_text_input: false,
                    edit_can_undo: true,
                    edit_can_redo: true,
                    dispatch_phase: Default::default(),
                });
            by_hwnd.insert(hwnd, input_ctx);
        }

        let Ok(mut state) = MENU_HOOK_STATE.lock() else {
            return;
        };
        for (hwnd, input_ctx) in by_hwnd {
            state.hwnd_to_input_ctx.insert(hwnd, input_ctx);
        }
    }
}

pub(crate) fn sync_command_enabled_from_app(app: &fret_app::App) {
    #[cfg(target_os = "windows")]
    {
        let snapshots = app.global::<WindowCommandEnabledService>();

        let windows: Vec<(isize, AppWindowId)> = {
            let Ok(state) = MENU_HOOK_STATE.lock() else {
                return;
            };
            state
                .hwnd_to_app_window
                .iter()
                .map(|(&hwnd, &window)| (hwnd, window))
                .collect()
        };

        let mut by_hwnd: HashMap<isize, HashMap<CommandId, bool>> = HashMap::new();
        for (hwnd, window) in windows {
            let enabled = snapshots
                .and_then(|svc| svc.snapshot(window))
                .cloned()
                .unwrap_or_default();
            by_hwnd.insert(hwnd, enabled);
        }

        let Ok(mut state) = MENU_HOOK_STATE.lock() else {
            return;
        };
        for (hwnd, enabled) in by_hwnd {
            state.hwnd_to_command_enabled.insert(hwnd, enabled);
        }
    }
}

#[cfg(target_os = "windows")]
fn build_menu_bar(
    menu_bar: &MenuBar,
    commands: &fret_runtime::CommandRegistry,
    keymap: Option<&fret_runtime::Keymap>,
    input_ctx: &InputContext,
) -> Option<(WindowsMenuBar, HashMap<u16, WindowsMenuItemDef>)> {
    let root = unsafe { CreateMenu() };
    if root.is_null() {
        return None;
    }

    let mut next_id: u16 = 1;
    let mut defs_by_id: HashMap<u16, WindowsMenuItemDef> = HashMap::new();

    for menu in &menu_bar.menus {
        let popup = unsafe { CreatePopupMenu() };
        if popup.is_null() {
            continue;
        }

        for item in &menu.items {
            append_menu_item(
                popup,
                item,
                commands,
                keymap,
                input_ctx,
                &mut next_id,
                &mut defs_by_id,
            );
        }

        let title = to_wide(&menu.title);
        unsafe {
            let _ = AppendMenuW(root, MF_POPUP, popup as usize, title.as_ptr());
        }
    }

    Some((WindowsMenuBar { handle: root }, defs_by_id))
}

#[cfg(target_os = "windows")]
fn append_menu_item(
    menu: HMENU,
    item: &MenuItem,
    commands: &fret_runtime::CommandRegistry,
    keymap: Option<&fret_runtime::Keymap>,
    input_ctx: &InputContext,
    next_id: &mut u16,
    defs_by_id: &mut HashMap<u16, WindowsMenuItemDef>,
) {
    match item {
        MenuItem::Separator => unsafe {
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        },
        MenuItem::SystemMenu { .. } => {
            // Windows HMENU does not have a direct equivalent for macOS system-managed menus.
            // Keep the runtime model portable by treating these as no-ops in the Win32 mapping.
        }
        MenuItem::Submenu { title, items, .. } => {
            let popup = unsafe { CreatePopupMenu() };
            if popup.is_null() {
                return;
            }
            for item in items {
                append_menu_item(
                    popup, item, commands, keymap, input_ctx, next_id, defs_by_id,
                );
            }
            let title = to_wide(title);
            unsafe {
                let _ = AppendMenuW(menu, MF_POPUP, popup as usize, title.as_ptr());
            }
        }
        MenuItem::Command { command, when } => {
            let id = *next_id;
            *next_id = next_id.saturating_add(1);

            let (label, command_when) = match commands.get(command.clone()) {
                Some(meta) => (meta.title.clone(), meta.when.clone()),
                None => (Arc::<str>::from(command.as_str()), None),
            };
            defs_by_id.insert(
                id,
                WindowsMenuItemDef {
                    command: command.clone(),
                    label: label.clone(),
                    command_when,
                    item_when: when.clone(),
                },
            );

            let shortcut =
                keymap.and_then(|km| km.display_shortcut_for_command_sequence(input_ctx, command));
            let text = if let Some(seq) = shortcut {
                format!(
                    "{}\t{}",
                    label,
                    fret_runtime::format_sequence(input_ctx.platform, &seq)
                )
            } else {
                label.to_string()
            };
            let wide = to_wide(&text);

            unsafe {
                let _ = AppendMenuW(menu, MF_STRING, id as usize, wide.as_ptr());
            }
        }
    }
}
