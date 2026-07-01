use fret_core::AppWindowId;
use fret_ui::{Invalidation, UiHost};

/// A stateful view object that renders into the existing declarative IR (`Ui`).
pub trait View: 'static {
    /// Initialize the view for a specific window.
    fn init(app: &mut crate::app::App, window: crate::WindowId) -> Self
    where
        Self: Sized;

    /// Render the view into declarative UI.
    fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui;
}

/// Explicit render-authoring helper capability for app-facing extracted helper functions.
///
/// This keeps helper signatures on one named lane without forcing them to accept the full `AppUi`
/// surface or a raw `ElementContext<'_, H>` type directly.
pub trait RenderContextAccess<'a, H: UiHost + 'a>: fret_ui::ElementContextAccess<'a, H> {
    fn app<'b>(&'b mut self) -> &'b H
    where
        'a: 'b,
    {
        &*self.elements().app
    }

    fn app_mut<'b>(&'b mut self) -> &'b mut H
    where
        'a: 'b,
    {
        &mut *self.elements().app
    }

    fn window_id(&mut self) -> AppWindowId {
        self.elements().window
    }

    fn environment_viewport_bounds(&mut self, invalidation: Invalidation) -> fret_core::Rect {
        self.elements().environment_viewport_bounds(invalidation)
    }

    fn with_theme<R>(&mut self, f: impl FnOnce(&fret_ui::Theme) -> R) -> R {
        f(self.elements().theme())
    }

    fn theme_snapshot(&mut self) -> fret_ui::ThemeSnapshot {
        self.with_theme(|theme| theme.snapshot())
    }
}

impl<'a, H: UiHost + 'a, T> RenderContextAccess<'a, H> for T where
    T: fret_ui::ElementContextAccess<'a, H>
{
}

/// Named default extracted-helper render lane for ordinary `fret` app code.
///
/// This is the app-facing façade over `RenderContextAccess<'a, crate::app::App>` so new helper
/// signatures can name the default lane directly without spelling the generic host parameter at
/// every callsite.
pub trait AppRenderContext<'a>: RenderContextAccess<'a, crate::app::App> {}

impl<'a, T> AppRenderContext<'a> for T where T: RenderContextAccess<'a, crate::app::App> {}
