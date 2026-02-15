use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug)]
pub(crate) struct RestartTrigger {
    file: FileTrigger,
}

impl RestartTrigger {
    pub(crate) fn from_env(now: Instant) -> Option<Self> {
        let file = FileTrigger::from_env(now)?;
        Some(Self { file })
    }

    pub(crate) fn next_poll_at(&self) -> Instant {
        self.file.next_poll_at
    }

    pub(crate) fn poll(&mut self, now: Instant) -> bool {
        self.file.poll(now)
    }
}

#[derive(Debug)]
struct FileTrigger {
    path: PathBuf,
    poll_interval: Duration,
    next_poll_at: Instant,
    last_marker: Option<String>,
}

impl FileTrigger {
    fn from_env(now: Instant) -> Option<Self> {
        let path = std::env::var_os("FRET_WATCH_RESTART_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)?;

        let poll_interval = std::env::var("FRET_WATCH_RESTART_POLL_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(150));

        let last_marker = std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let mtime: SystemTime = std::fs::metadata(&path).ok()?.modified().ok()?;
                Some(format!("{:?}", mtime))
            });

        Some(Self {
            path,
            poll_interval,
            next_poll_at: now + poll_interval,
            last_marker,
        })
    }

    fn poll(&mut self, now: Instant) -> bool {
        if now < self.next_poll_at {
            return false;
        }
        self.next_poll_at = now + self.poll_interval;

        let marker = std::fs::read_to_string(&self.path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let mtime = std::fs::metadata(&self.path)
                    .ok()
                    .and_then(|m| m.modified().ok())?;
                Some(format!("{:?}", mtime))
            });

        let Some(marker) = marker else {
            return false;
        };

        if self
            .last_marker
            .as_ref()
            .is_some_and(|prev| prev == &marker)
        {
            return false;
        }
        self.last_marker = Some(marker);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_trigger_file_polling_observes_changes() {
        struct EnvGuard {
            key: &'static str,
        }

        impl EnvGuard {
            fn set(key: &'static str, value: &str) -> Self {
                unsafe {
                    std::env::set_var(key, value);
                }
                Self { key }
            }
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                unsafe {
                    std::env::remove_var(self.key);
                }
            }
        }

        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        path.push(format!("fret_restart_trigger_test_{unique}.touch"));
        std::fs::write(&path, "a").expect("write");

        let _guard = EnvGuard::set(
            "FRET_WATCH_RESTART_TRIGGER_PATH",
            path.to_string_lossy().as_ref(),
        );

        let now = Instant::now();
        let mut trigger = RestartTrigger::from_env(now).expect("trigger");
        assert!(!trigger.poll(now));

        std::fs::write(&path, "b").expect("write");
        assert!(trigger.poll(now + Duration::from_millis(200)));

        let _ = std::fs::remove_file(&path);
    }
}
