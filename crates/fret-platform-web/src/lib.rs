//! Web/wasm32 platform services.
//!
//! This crate provides browser API integrations used by `fret-runtime::Effect`s (timers, file
//! inputs, IME bridge, etc.). It intentionally does **not** implement input/event mapping; use a
//! runner crate (e.g. `fret-runner-web`) for the winit/web event layer.
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-platform-web/README.md`.

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

mod ime_dom_state;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn platform_error_is_actionable_on_non_wasm_targets() {
        let err = PlatformError;
        assert_eq!(
            err.to_string(),
            "fret-platform-web is only available on wasm32"
        );

        fn assert_error(_err: &dyn std::error::Error) {}
        assert_error(&err);
    }
}
