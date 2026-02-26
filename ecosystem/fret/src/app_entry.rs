//! `fret::App` builder-chain entry points.
//!
//! This module provides an ergonomic, desktop-first entry surface (ecosystem-level) while
//! preserving the golden-path driver's hotpatch-friendly posture (function-pointer hooks).

use crate::{Result, UiAppBuilder, ViewElements};

/// Builder-chain facade for creating and running a desktop-first Fret UI app.
///
/// Notes:
/// - This is an ecosystem-level convenience layer (not a kernel contract).
/// - The builder composes existing `fret` entry points (`fret::mvu` / `fret::app`) and applies
///   a default main window if none is configured.
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub struct App {
    root_name: &'static str,
    main_window: Option<(String, (f64, f64))>,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl App {
    /// Create a new app builder with a stable root name.
    ///
    /// `root_name` is used by the golden-path driver for IDs, diagnostics, and dev tooling.
    pub fn new(root_name: &'static str) -> Self {
        Self {
            root_name,
            main_window: None,
        }
    }

    /// Configure the main window (title + size).
    pub fn window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        self.main_window = Some((title.into(), size));
        self
    }

    /// Build a typed-message MVU app (`fret::mvu`) and return a runnable builder.
    pub fn mvu<P: crate::mvu::Program>(
        self,
    ) -> Result<UiAppBuilder<crate::mvu::MvuWindowState<P::State, P::Message>>> {
        let mut builder = crate::mvu::app::<P>(self.root_name)?;
        builder = self.apply_main_window(builder);
        Ok(builder)
    }

    /// Build a UI app from `init_window` + `view` and return a runnable builder.
    pub fn ui<S: 'static>(
        self,
        init_window: fn(&mut fret_app::App, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, fret_app::App>, &mut S) -> ViewElements,
    ) -> Result<UiAppBuilder<S>> {
        let mut builder = crate::app(self.root_name, init_window, view)?;
        builder = self.apply_main_window(builder);
        Ok(builder)
    }

    /// Convenience: build an MVU app and run it immediately.
    pub fn run_mvu<P: crate::mvu::Program>(self) -> Result<()> {
        self.mvu::<P>()?.run()
    }

    /// Convenience: build a UI app and run it immediately.
    pub fn run_ui<S: 'static>(
        self,
        init_window: fn(&mut fret_app::App, fret_core::AppWindowId) -> S,
        view: for<'a> fn(&mut fret_ui::ElementContext<'a, fret_app::App>, &mut S) -> ViewElements,
    ) -> Result<()> {
        self.ui(init_window, view)?.run()
    }

    fn apply_main_window<S: 'static>(self, builder: UiAppBuilder<S>) -> UiAppBuilder<S> {
        if let Some((title, size)) = self.main_window {
            return builder.with_main_window(title, size);
        }

        builder.with_main_window(self.root_name, (960.0, 720.0))
    }
}
