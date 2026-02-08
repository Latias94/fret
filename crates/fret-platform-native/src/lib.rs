//! Native (non-wasm) platform implementations for `fret-platform` contracts.
//!
//! This crate is intentionally native-only:
//! - uses native clipboard/file-dialog/open-url backends (`arboard`, `rfd`, `webbrowser`)
//! - uses real filesystem paths for external drops and file dialog selections
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-platform-native/README.md`.

pub mod clipboard;
pub mod external_drop;
pub mod file_dialog;
pub mod open_url;

// -----------------------------------------------------------------------------
// Stable re-exports (native platform surface)
// -----------------------------------------------------------------------------
pub use clipboard::{DesktopClipboard, NativeClipboard};
pub use external_drop::{DesktopExternalDrop, NativeExternalDrop};
pub use file_dialog::{DesktopFileDialog, NativeFileDialog};
pub use open_url::{DesktopOpenUrl, NativeOpenUrl};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use fret_core::{ExternalDropToken, FileDialogToken};
    use fret_platform::external_drop::ExternalDropReadLimits;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let pid = std::process::id();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{prefix}-{pid}-{now}"));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).expect("write temp file");
        path
    }

    #[test]
    fn external_drop_read_paths_enforces_max_total_bytes() {
        let dir = TempDirGuard::new("fret-platform-native-external-drop");
        let f1 = write_file(dir.path(), "one.txt", b"abcd");
        let f2 = write_file(dir.path(), "two.txt", b"efgh");

        let token = ExternalDropToken(1);
        let event = NativeExternalDrop::read_paths(
            token,
            vec![f1.clone(), f2.clone()],
            ExternalDropReadLimits {
                max_total_bytes: 5,
                max_file_bytes: 1024,
                max_files: 10,
            },
        );

        assert_eq!(event.token, token);
        assert_eq!(event.files.len(), 1);
        assert_eq!(event.files[0].name, "one.txt");
        assert_eq!(event.files[0].bytes, b"abcd");

        assert_eq!(event.errors.len(), 1);
        assert_eq!(event.errors[0].name, "two.txt");
        assert!(event.errors[0].message.contains("next_total"));
        assert!(event.errors[0].message.contains("max_total_bytes"));
    }

    #[test]
    fn file_dialog_read_paths_enforces_max_total_bytes() {
        let dir = TempDirGuard::new("fret-platform-native-file-dialog");
        let f1 = write_file(dir.path(), "one.txt", b"abcd");
        let f2 = write_file(dir.path(), "two.txt", b"efgh");

        let token = FileDialogToken(1);
        let event = NativeFileDialog::read_paths(
            token,
            vec![f1, f2],
            ExternalDropReadLimits {
                max_total_bytes: 5,
                max_file_bytes: 1024,
                max_files: 10,
            },
        );

        assert_eq!(event.token, token);
        assert_eq!(event.files.len(), 1);
        assert_eq!(event.files[0].name, "one.txt");
        assert_eq!(event.files[0].bytes, b"abcd");

        assert_eq!(event.errors.len(), 1);
        assert_eq!(event.errors[0].name, "two.txt");
        assert!(event.errors[0].message.contains("selection too large"));
        assert!(event.errors[0].message.contains("max_total_bytes"));
    }

    #[test]
    fn external_drop_read_paths_skips_oversize_files_and_continues() {
        let dir = TempDirGuard::new("fret-platform-native-external-drop-oversize");
        let big = write_file(dir.path(), "big.bin", &[0u8; 12]);
        let ok = write_file(dir.path(), "ok.bin", &[1u8; 4]);

        let token = ExternalDropToken(2);
        let event = NativeExternalDrop::read_paths(
            token,
            vec![big, ok],
            ExternalDropReadLimits {
                max_total_bytes: 1024,
                max_file_bytes: 10,
                max_files: 10,
            },
        );

        assert_eq!(event.token, token);
        assert_eq!(event.files.len(), 1);
        assert_eq!(event.files[0].name, "ok.bin");
        assert_eq!(event.files[0].bytes, vec![1u8; 4]);

        assert_eq!(event.errors.len(), 1);
        assert_eq!(event.errors[0].name, "big.bin");
        assert!(event.errors[0].message.contains("file too large"));
        assert!(event.errors[0].message.contains("max_file_bytes"));
    }
}
