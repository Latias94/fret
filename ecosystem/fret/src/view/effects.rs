use fret_ui::UiHost;
#[cfg(feature = "shadcn")]
use fret_ui::action::UiActionHostAdapter;

use super::AppUi;

/// Grouped render-time effect helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiEffects<'view, 'cx, 'a, H: UiHost> {
    pub(super) cx: &'view mut AppUi<'cx, 'a, H>,
}

impl<'view, 'cx, 'a, H: UiHost> AppUiEffects<'view, 'cx, 'a, H> {
    pub fn take_transient(self, key: u64) -> bool {
        self.cx.cx.take_transient_for(self.cx.action_root, key)
    }

    /// Emit a shadcn/Sonner default toast from the app facade without exposing the raw action host.
    #[cfg(feature = "shadcn")]
    pub fn toast_message(
        self,
        title: impl Into<std::sync::Arc<str>>,
        options: crate::shadcn::ToastMessageOptions,
    ) -> crate::shadcn::ToastId {
        let window = self.cx.window_id();
        let sonner = crate::shadcn::Sonner::global(self.cx.app_mut());
        let mut host = UiActionHostAdapter {
            app: self.cx.app_mut(),
        };
        sonner.toast_message(&mut host, window, title, options)
    }

    /// Emit a shadcn/Sonner success toast from the app facade without exposing the raw action host.
    #[cfg(feature = "shadcn")]
    pub fn toast_success(
        self,
        title: impl Into<std::sync::Arc<str>>,
        options: crate::shadcn::ToastMessageOptions,
    ) -> crate::shadcn::ToastId {
        let window = self.cx.window_id();
        let sonner = crate::shadcn::Sonner::global(self.cx.app_mut());
        let mut host = UiActionHostAdapter {
            app: self.cx.app_mut(),
        };
        sonner.toast_success_message(&mut host, window, title, options)
    }

    /// Emit a shadcn/Sonner error toast from the app facade without exposing the raw action host.
    #[cfg(feature = "shadcn")]
    pub fn toast_error(
        self,
        title: impl Into<std::sync::Arc<str>>,
        options: crate::shadcn::ToastMessageOptions,
    ) -> crate::shadcn::ToastId {
        let window = self.cx.window_id();
        let sonner = crate::shadcn::Sonner::global(self.cx.app_mut());
        let mut host = UiActionHostAdapter {
            app: self.cx.app_mut(),
        };
        sonner.toast_error_message(&mut host, window, title, options)
    }

    /// Dismiss all shadcn/Sonner toasts for the current window from the app facade.
    #[cfg(feature = "shadcn")]
    pub fn toast_dismiss_all(self) -> usize {
        let window = self.cx.window_id();
        let sonner = crate::shadcn::Sonner::global(self.cx.app_mut());
        let mut host = UiActionHostAdapter {
            app: self.cx.app_mut(),
        };
        sonner.dismiss_all(&mut host, window)
    }
}
