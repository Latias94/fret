//! Explicit advanced/manual-assembly imports for power users and integration code.

/// Low-level view-runtime helpers kept off the default crate root.
pub mod view {
    pub use crate::view::{
        AppRenderDataExt, AppUiRenderRootState, ViewWindowState, render_root_with_app_ui,
        view_init_window, view_view,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
    pub use crate::view::view_record_engine_frame;
}

/// Dev-only helpers kept as an advanced compatibility lane for iteration workflows.
///
/// Prefer the owning `fret-launch::dev_state::*` surface directly in first-party or advanced code;
/// `fret/devloop` exists mainly as a discoverable alias on the app facade.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop", feature = "devloop"))]
pub mod dev {
    pub use fret_launch::dev_state::{
        DevStateExport, DevStateHook, DevStateHooks, DevStateSnapshot, DevStateWindowKeyRegistry,
    };
}

/// Low-level interop helpers kept off the default crate root.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub mod interop {
    pub use crate::interop::embedded_viewport;
    pub use crate::interop::run_native_with_driver;
}

/// Text helpers for advanced/manual render lanes.
///
/// These mirror the default `fret::app::text` helpers without constraining callers to the default
/// `App` host. Use them from manual `KernelApp` / custom-host examples that still need a named text
/// recipe instead of importing `fret-ui-kit`'s raw declarative text module.
pub mod text {
    use std::sync::Arc;

    /// Compact control/status readout text.
    pub fn control_readout<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_control_readout(cx.elements(), text)
    }

    /// Compact prose paragraph text.
    pub fn compact_paragraph<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_compact_paragraph(cx.elements(), text)
    }

    /// Section/chrome label text.
    pub fn section_chrome_label<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_section_chrome_label(cx.elements(), text)
    }

    /// Compact chrome glyph text.
    pub fn chrome_glyph<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_chrome_glyph(cx.elements(), text)
    }

    /// Inline code label text.
    pub fn code_label<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_code_label(cx.elements(), text)
    }

    /// Block code text.
    pub fn code_block<'a, H, Cx, T>(cx: &mut Cx, text: T) -> fret_ui::element::AnyElement
    where
        H: fret_ui::UiHost + 'a,
        Cx: fret_ui::ElementContextAccess<'a, H>,
        T: Into<Arc<str>>,
    {
        fret_ui_kit::declarative::text::text_code_block(cx.elements(), text)
    }
}

/// Explicit raw retained-tree and hook seams for advanced/manual assembly.
///
/// Import these from `fret::advanced::raw` at the call site that actually needs the raw runtime
/// seam. They intentionally stay out of `advanced::prelude::*` so advanced examples do not acquire
/// raw action/model hooks by accident.
pub mod raw;

pub use fret_app::App as KernelApp;

/// Low-level kernel facade kept off the default crate root.
#[cfg(feature = "desktop")]
pub use fret_framework as kernel;

/// Advanced driver/builder escape hatches.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub mod driver;

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use driver::{
    FretAppAdvancedExt, UiAppBuilder, UiAppBuilderAdvancedExt, UiAppDriver, ViewElements,
    run_native_with_configured_fn_driver, run_native_with_fn_driver,
    run_native_with_fn_driver_with_hooks, ui_app, ui_app_with_hooks,
};

/// Common imports for advanced/manual-assembly application code.
pub mod prelude;
