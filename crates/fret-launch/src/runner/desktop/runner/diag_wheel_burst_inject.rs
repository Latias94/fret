use std::path::PathBuf;

use fret_core::{AppWindowId, Modifiers, Point, PointerType, Px};
use slotmap::KeyData;

#[derive(Debug, Clone)]
pub(super) struct WheelBurstInjectRequestV1 {
    pub(super) window: Option<AppWindowId>,
    pub(super) position: Point,
    pub(super) delta_x: f32,
    pub(super) delta_y: f32,
    pub(super) count: u32,
    pub(super) modifiers: Modifiers,
    pub(super) pointer_type: PointerType,
}

#[derive(Debug)]
pub(super) struct DiagWheelBurstInject {
    request_path: PathBuf,
    trigger_path: PathBuf,
    last_trigger_stamp: Option<u64>,
    pending: Option<WheelBurstInjectRequestV1>,
}

impl DiagWheelBurstInject {
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
            request_path: out_dir.join("wheel_burst.request.txt"),
            trigger_path: out_dir.join("wheel_burst.touch"),
            last_trigger_stamp: None,
            pending: None,
        })
    }

    fn poll(&mut self) {
        let stamp = match std::fs::read_to_string(&self.trigger_path) {
            Ok(text) => text
                .lines()
                .rev()
                .find_map(|line| line.trim().parse::<u64>().ok()),
            Err(_) => None,
        };
        let Some(stamp) = stamp else {
            return;
        };
        if self.last_trigger_stamp.is_some_and(|prev| prev >= stamp) {
            return;
        }
        self.last_trigger_stamp = Some(stamp);

        let text = match std::fs::read_to_string(&self.request_path) {
            Ok(t) => t,
            Err(_) => return,
        };
        let Some(req) = parse_wheel_burst_inject_request_v1(&text) else {
            return;
        };
        if req.count == 0 {
            return;
        }
        self.pending = Some(req);
    }

    pub(super) fn take_for_window(
        &mut self,
        window: AppWindowId,
    ) -> Option<WheelBurstInjectRequestV1> {
        if let Some(req) = self.pending.as_ref()
            && req.window.is_some_and(|w| w != window)
        {
            // Request targets a different window; keep it pending until that window is active.
            let _ = req;
        } else if self.pending.is_some() {
            return self.pending.take();
        }

        self.poll();

        if let Some(req) = self.pending.as_ref()
            && req.window.is_some_and(|w| w != window)
        {
            return None;
        }
        self.pending.take()
    }
}

fn parse_pointer_type_v1(value: &str) -> Option<PointerType> {
    match value.trim() {
        "mouse" => Some(PointerType::Mouse),
        "touch" => Some(PointerType::Touch),
        "pen" => Some(PointerType::Pen),
        _ => None,
    }
}

fn parse_wheel_burst_inject_request_v1(text: &str) -> Option<WheelBurstInjectRequestV1> {
    let mut schema_version: Option<u32> = None;
    let mut window_ffi: Option<u64> = None;
    let mut x_px: Option<f32> = None;
    let mut y_px: Option<f32> = None;
    let mut delta_x: Option<f32> = None;
    let mut delta_y: Option<f32> = None;
    let mut count: Option<u32> = None;
    let mut pointer_type: Option<PointerType> = None;

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
            "x_px" => x_px = v.parse::<f64>().ok().map(|v| v as f32),
            "y_px" => y_px = v.parse::<f64>().ok().map(|v| v as f32),
            "delta_x" => delta_x = v.parse::<f64>().ok().map(|v| v as f32),
            "delta_y" => delta_y = v.parse::<f64>().ok().map(|v| v as f32),
            "count" => count = v.parse().ok(),
            "pointer_kind" => pointer_type = parse_pointer_type_v1(v),
            _ => {}
        }
    }

    if schema_version != Some(1) {
        return None;
    }

    Some(WheelBurstInjectRequestV1 {
        window: window_ffi.map(|w| AppWindowId::from(KeyData::from_ffi(w))),
        position: Point::new(Px(x_px?), Px(y_px?)),
        delta_x: delta_x.unwrap_or(0.0),
        delta_y: delta_y.unwrap_or(0.0),
        count: count.unwrap_or(1),
        modifiers: Modifiers::default(),
        pointer_type: pointer_type.unwrap_or(PointerType::Mouse),
    })
}

impl<D: super::WinitAppDriver> super::WinitRunner<D> {
    pub(super) fn poll_diag_wheel_burst_inject(
        &mut self,
        window: fret_core::AppWindowId,
    ) -> Option<WheelBurstInjectRequestV1> {
        let mut svc = self.diag_wheel_burst_inject.take()?;
        let req = svc.take_for_window(window);
        self.diag_wheel_burst_inject = Some(svc);
        req
    }
}
