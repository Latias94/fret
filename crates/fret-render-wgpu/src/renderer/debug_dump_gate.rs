use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy)]
pub(super) struct DumpFrameEnv {
    enabled: &'static str,
    frame: &'static str,
    after_frames: &'static str,
    every: &'static str,
    dir: &'static str,
    default_dir: &'static str,
}

impl DumpFrameEnv {
    pub(super) const fn new(
        enabled: &'static str,
        frame: &'static str,
        after_frames: &'static str,
        every: &'static str,
        dir: &'static str,
        default_dir: &'static str,
    ) -> Self {
        Self {
            enabled,
            frame,
            after_frames,
            every,
            dir,
            default_dir,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DumpFrameSettings {
    enabled: bool,
    frame: Option<u64>,
    after_frames: Option<u64>,
    every: Option<u64>,
}

pub(super) fn should_emit_dump_frame(
    frame_index: u64,
    env: DumpFrameEnv,
    one_shot_dumped: &AtomicBool,
) -> bool {
    should_emit_dump_frame_from_settings(frame_index, settings_from_env(env), one_shot_dumped)
}

pub(super) fn emit_dump_file(env: DumpFrameEnv, file_name: impl AsRef<str>, bytes: &[u8]) {
    let dir = dump_dir_from_env(env);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join(file_name.as_ref()), bytes);
}

fn settings_from_env(env: DumpFrameEnv) -> DumpFrameSettings {
    DumpFrameSettings {
        enabled: std::env::var_os(env.enabled).is_some_and(|value| !value.is_empty()),
        frame: parse_env_u64(env.frame),
        after_frames: parse_env_u64(env.after_frames),
        every: parse_env_u64(env.every),
    }
}

fn parse_env_u64(name: &str) -> Option<u64> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
}

fn dump_dir_from_env(env: DumpFrameEnv) -> PathBuf {
    std::env::var_os(env.dir)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".fret").join(env.default_dir))
}

fn should_emit_dump_frame_from_settings(
    frame_index: u64,
    settings: DumpFrameSettings,
    one_shot_dumped: &AtomicBool,
) -> bool {
    if !settings.enabled {
        return false;
    }

    if let Some(frame) = settings.frame {
        return frame_index == frame;
    }

    let after = settings.after_frames.unwrap_or(1);
    if frame_index < after {
        return false;
    }

    if let Some(every) = settings.every {
        return every > 0 && (frame_index - after).is_multiple_of(every);
    }

    !one_shot_dumped.swap(true, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(
        enabled: bool,
        frame: Option<u64>,
        after_frames: Option<u64>,
        every: Option<u64>,
    ) -> DumpFrameSettings {
        DumpFrameSettings {
            enabled,
            frame,
            after_frames,
            every,
        }
    }

    #[test]
    fn dump_frame_gate_requires_enabled_env() {
        let dumped = AtomicBool::new(false);

        assert!(!should_emit_dump_frame_from_settings(
            3,
            settings(false, Some(3), None, None),
            &dumped
        ));
        assert!(!dumped.load(Ordering::SeqCst));
    }

    #[test]
    fn dump_frame_gate_exact_frame_overrides_one_shot() {
        let dumped = AtomicBool::new(false);
        let settings = settings(true, Some(3), Some(10), None);

        assert!(!should_emit_dump_frame_from_settings(2, settings, &dumped));
        assert!(should_emit_dump_frame_from_settings(3, settings, &dumped));
        assert!(should_emit_dump_frame_from_settings(3, settings, &dumped));
        assert!(!dumped.load(Ordering::SeqCst));
    }

    #[test]
    fn dump_frame_gate_defaults_to_single_emit_after_frame_one() {
        let dumped = AtomicBool::new(false);
        let settings = settings(true, None, None, None);

        assert!(!should_emit_dump_frame_from_settings(0, settings, &dumped));
        assert!(should_emit_dump_frame_from_settings(1, settings, &dumped));
        assert!(!should_emit_dump_frame_from_settings(2, settings, &dumped));
    }

    #[test]
    fn dump_frame_gate_every_uses_after_as_origin() {
        let dumped = AtomicBool::new(false);
        let settings = settings(true, None, Some(3), Some(2));

        assert!(!should_emit_dump_frame_from_settings(2, settings, &dumped));
        assert!(should_emit_dump_frame_from_settings(3, settings, &dumped));
        assert!(!should_emit_dump_frame_from_settings(4, settings, &dumped));
        assert!(should_emit_dump_frame_from_settings(5, settings, &dumped));
        assert!(!dumped.load(Ordering::SeqCst));
    }

    #[test]
    fn dump_frame_gate_zero_every_disables_periodic_emit() {
        let dumped = AtomicBool::new(false);
        let settings = settings(true, None, Some(3), Some(0));

        assert!(!should_emit_dump_frame_from_settings(3, settings, &dumped));
        assert!(!should_emit_dump_frame_from_settings(4, settings, &dumped));
        assert!(!dumped.load(Ordering::SeqCst));
    }
}
