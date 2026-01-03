//! Compatibility shim for the old `fret-platform-desktop` crate name.
//!
//! Prefer `fret-platform-native`.

pub mod clipboard {
    pub use fret_platform_native::clipboard::*;
}

pub mod external_drop {
    pub use fret_platform_native::external_drop::*;
}

pub mod file_dialog {
    pub use fret_platform_native::file_dialog::*;
}

pub mod open_url {
    pub use fret_platform_native::open_url::*;
}

pub use fret_platform_native::*;
