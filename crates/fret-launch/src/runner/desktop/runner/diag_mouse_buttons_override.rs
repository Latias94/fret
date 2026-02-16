use std::path::PathBuf;
use std::time::SystemTime;

use fret_core::AppWindowId;
use slotmap::KeyData;

#[derive(Debug)]
pub(super) struct DiagMouseButtonsOverride {
    request_path: PathBuf,
    trigger_path: PathBuf,
    last_trigger_mtime: Option<SystemTime>,
}

#[derive(Debug, Default)]
struct MouseButtonsOverrideV1 {
    window: Option<AppWindowId>,
    left: Option<bool>,
    right: Option<bool>,
    middle: Option<bool>,
}

impl DiagMouseButtonsOverride {
    pub(super) fn from_env() -> Option<Self> {
        let out_dir_env = std::env::var_os("FRET_DIAG_DIR").filter(|v| !v.is_empty());
        let enabled =
            std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()) || out_dir_env.is_some();
        if !enabled {
            return None;
        }

        let out_dir = out_dir_env
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));

        Some(Self {
            request_path: out_dir.join("mouse_buttons.override.txt"),
            trigger_path: out_dir.join("mouse_buttons.touch"),
            last_trigger_mtime: None,
        })
    }

    fn poll<D: super::WinitAppDriver>(&mut self, runner: &mut super::WinitRunner<D>) -> bool {
        let modified = match std::fs::metadata(&self.trigger_path).and_then(|m| m.modified()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if self.last_trigger_mtime.is_some_and(|prev| prev >= modified) {
            return false;
        }
        self.last_trigger_mtime = Some(modified);

        let text = match std::fs::read_to_string(&self.request_path) {
            Ok(t) => t,
            Err(_) => return false,
        };

        let Some(req) = parse_mouse_buttons_override_v1(&text) else {
            return false;
        };

        let any = req.left.is_some() || req.right.is_some() || req.middle.is_some();
        if !any {
            return false;
        }

        if let Some(left) = req.left {
            runner.left_mouse_down = left;
        }

        match req.window {
            Some(window) => {
                if let Some(state) = runner.windows.get_mut(window) {
                    if let Some(left) = req.left {
                        state.platform.input.pressed_buttons.left = left;
                    }
                    if let Some(right) = req.right {
                        state.platform.input.pressed_buttons.right = right;
                    }
                    if let Some(middle) = req.middle {
                        state.platform.input.pressed_buttons.middle = middle;
                    }
                }
            }
            None => {
                for state in runner.windows.values_mut() {
                    if let Some(left) = req.left {
                        state.platform.input.pressed_buttons.left = left;
                    }
                    if let Some(right) = req.right {
                        state.platform.input.pressed_buttons.right = right;
                    }
                    if let Some(middle) = req.middle {
                        state.platform.input.pressed_buttons.middle = middle;
                    }
                }
            }
        }

        true
    }
}

fn parse_bool_v1(value: &str) -> Option<bool> {
    let v = value.trim();
    match v {
        "1" | "true" | "True" | "TRUE" => Some(true),
        "0" | "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

fn parse_mouse_buttons_override_v1(text: &str) -> Option<MouseButtonsOverrideV1> {
    let mut schema_version: Option<u32> = None;
    let mut window_ffi: Option<u64> = None;
    let mut left: Option<bool> = None;
    let mut right: Option<bool> = None;
    let mut middle: Option<bool> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once('=')?;
        let k = k.trim();
        let v = v.trim();
        match k {
            "schema_version" => schema_version = v.parse().ok(),
            "window" => window_ffi = v.parse().ok(),
            "left" => left = parse_bool_v1(v),
            "right" => right = parse_bool_v1(v),
            "middle" => middle = parse_bool_v1(v),
            _ => {}
        }
    }

    if schema_version != Some(1) {
        return None;
    }

    Some(MouseButtonsOverrideV1 {
        window: window_ffi.map(|w| AppWindowId::from(KeyData::from_ffi(w))),
        left,
        right,
        middle,
    })
}

impl<D: super::WinitAppDriver> super::WinitRunner<D> {
    pub(super) fn poll_diag_mouse_buttons_override(&mut self) -> bool {
        let Some(mut svc) = self.diag_mouse_buttons_override.take() else {
            return false;
        };
        let updated = svc.poll(self);
        self.diag_mouse_buttons_override = Some(svc);
        updated
    }
}
