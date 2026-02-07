//! Fret's kernel facade crate.
//!
//! This crate is intended to be the small, memorable entry point for **manual/advanced assembly**
//! and integrations. It re-exports selected workspace crates behind opt-in feature flags, without
//! pulling ecosystem defaults (components, policies, tooling) into `crates/*`.
//!
//! For the batteries-included desktop-first experience, prefer:
//! - `fret-kit` (app entry points)
//! - `fretboard` (dev tooling)
//!
//! See ADR 0111: `docs/adr/0111-user-facing-crate-surfaces-and-golden-path.md`.

#[cfg(feature = "core")]
pub mod core {
    pub use fret_core::*;
}

#[cfg(feature = "app")]
pub mod app {
    pub use fret_app::*;
}

#[cfg(feature = "ui")]
pub mod ui {
    pub use fret_ui::*;
}

#[cfg(feature = "runtime")]
pub mod runtime {
    pub use fret_runtime::*;
}

#[cfg(feature = "render")]
pub mod render {
    pub use fret_render::*;
}

#[cfg(feature = "fonts")]
pub mod fonts {
    pub use fret_fonts::*;
}

#[cfg(feature = "platform-contracts")]
pub mod platform {
    pub use fret_platform::*;
}

#[cfg(feature = "platform-native")]
pub mod platform_native {
    pub use fret_platform_native::*;
}

#[cfg(feature = "platform-web")]
pub mod platform_web {
    pub use fret_platform_web::*;
}

#[cfg(feature = "runner-winit")]
pub mod runner_winit {
    pub use fret_runner_winit::*;
}

#[cfg(feature = "runner-web")]
pub mod runner_web {
    pub use fret_runner_web::*;
}

#[cfg(feature = "launch")]
pub mod launch {
    pub use fret_launch::*;
}

pub mod prelude {
    #[cfg(feature = "app")]
    pub use fret_app::{App, Effect};

    #[cfg(feature = "core")]
    pub use fret_core::{AppWindowId, CursorIcon, Event, Point, Px, Rect, Scene, Size};

    #[cfg(feature = "runtime")]
    pub use fret_runtime::{
        CommandId, Effect as RuntimeEffect, GlobalsHost, PlatformCapabilities, PlatformCompletion,
        TickId, UiHost,
    };

    #[cfg(feature = "ui")]
    pub use fret_ui::{ElementContext, ElementRuntime, UiTree};
}
