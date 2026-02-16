//! Internal AccessKit platform adapter (RawWindowHandle-based).

use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, Rect, TreeUpdate};

use winit::raw_window_handle::RawWindowHandle;

pub(crate) struct Adapter {
    inner: platform_impl::Adapter,
}

impl Adapter {
    pub(crate) fn new(
        window_handle: RawWindowHandle,
        activation_handler: impl 'static + ActivationHandler + Send,
        action_handler: impl 'static + ActionHandler + Send,
        deactivation_handler: impl 'static + DeactivationHandler + Send,
    ) -> Self {
        let inner = platform_impl::Adapter::new(
            window_handle,
            activation_handler,
            action_handler,
            deactivation_handler,
        );
        Self { inner }
    }

    pub(crate) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.inner.update_if_active(updater);
    }

    pub(crate) fn set_focus(&mut self, is_focused: bool) {
        self.inner.set_focus(is_focused);
    }

    pub(crate) fn set_window_bounds(&mut self, outer_bounds: Rect, inner_bounds: Rect) {
        self.inner.set_window_bounds(outer_bounds, inner_bounds);
    }
}

mod platform_impl {
    use super::*;

    #[cfg(target_os = "windows")]
    mod platform {
        use super::*;

        use accesskit_windows::{HWND, SubclassingAdapter};

        pub(super) struct PlatformAdapter {
            adapter: SubclassingAdapter,
        }

        impl PlatformAdapter {
            pub(super) fn new(
                window_handle: RawWindowHandle,
                activation_handler: impl 'static + ActivationHandler + Send,
                action_handler: impl 'static + ActionHandler + Send,
                _deactivation_handler: impl 'static + DeactivationHandler + Send,
            ) -> Self {
                let hwnd = match window_handle {
                    RawWindowHandle::Win32(handle) => handle.hwnd.get() as *mut _,
                    RawWindowHandle::WinRt(_) => {
                        unimplemented!("WinRT window handles not supported")
                    }
                    _ => unreachable!("unexpected RawWindowHandle for Windows"),
                };

                let adapter =
                    SubclassingAdapter::new(HWND(hwnd), activation_handler, action_handler);
                Self { adapter }
            }

            pub(super) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
                if let Some(events) = self.adapter.update_if_active(updater) {
                    events.raise();
                }
            }

            pub(super) fn set_focus(&mut self, _is_focused: bool) {}

            pub(super) fn set_window_bounds(&mut self, _outer_bounds: Rect, _inner_bounds: Rect) {}
        }
    }

    #[cfg(target_os = "macos")]
    mod platform {
        use super::*;

        use accesskit_macos::SubclassingAdapter;

        pub(super) struct PlatformAdapter {
            adapter: SubclassingAdapter,
        }

        impl PlatformAdapter {
            pub(super) fn new(
                window_handle: RawWindowHandle,
                activation_handler: impl 'static + ActivationHandler + Send,
                action_handler: impl 'static + ActionHandler + Send,
                _deactivation_handler: impl 'static + DeactivationHandler + Send,
            ) -> Self {
                let view = match window_handle {
                    RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
                    RawWindowHandle::UiKit(_) => {
                        unimplemented!("UIKit window handles not supported")
                    }
                    _ => unreachable!("unexpected RawWindowHandle for macOS"),
                };

                let adapter =
                    unsafe { SubclassingAdapter::new(view, activation_handler, action_handler) };
                Self { adapter }
            }

            pub(super) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
                if let Some(events) = self.adapter.update_if_active(updater) {
                    events.raise();
                }
            }

            pub(super) fn set_focus(&mut self, is_focused: bool) {
                if let Some(events) = self.adapter.update_view_focus_state(is_focused) {
                    events.raise();
                }
            }

            pub(super) fn set_window_bounds(&mut self, _outer_bounds: Rect, _inner_bounds: Rect) {}
        }
    }

    #[cfg(target_os = "linux")]
    mod platform {
        use super::*;

        pub(super) struct PlatformAdapter {
            adapter: accesskit_unix::Adapter,
        }

        impl PlatformAdapter {
            pub(super) fn new(
                _window_handle: RawWindowHandle,
                activation_handler: impl 'static + ActivationHandler + Send,
                action_handler: impl 'static + ActionHandler + Send,
                deactivation_handler: impl 'static + DeactivationHandler + Send,
            ) -> Self {
                let adapter = accesskit_unix::Adapter::new(
                    activation_handler,
                    action_handler,
                    deactivation_handler,
                );
                Self { adapter }
            }

            pub(super) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
                self.adapter.update_if_active(updater);
            }

            pub(super) fn set_focus(&mut self, is_focused: bool) {
                self.adapter.update_window_focus_state(is_focused);
            }

            pub(super) fn set_window_bounds(&mut self, outer_bounds: Rect, inner_bounds: Rect) {
                self.adapter
                    .set_root_window_bounds(outer_bounds, inner_bounds);
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    mod platform {
        use super::*;

        pub(super) struct PlatformAdapter;

        impl PlatformAdapter {
            pub(super) fn new(
                _window_handle: RawWindowHandle,
                _activation_handler: impl 'static + ActivationHandler + Send,
                _action_handler: impl 'static + ActionHandler + Send,
                _deactivation_handler: impl 'static + DeactivationHandler + Send,
            ) -> Self {
                Self
            }

            pub(super) fn update_if_active(&mut self, _updater: impl FnOnce() -> TreeUpdate) {}

            pub(super) fn set_focus(&mut self, _is_focused: bool) {}

            pub(super) fn set_window_bounds(&mut self, _outer_bounds: Rect, _inner_bounds: Rect) {}
        }
    }

    pub(super) struct Adapter {
        inner: platform::PlatformAdapter,
    }

    impl Adapter {
        pub(super) fn new(
            window_handle: RawWindowHandle,
            activation_handler: impl 'static + ActivationHandler + Send,
            action_handler: impl 'static + ActionHandler + Send,
            deactivation_handler: impl 'static + DeactivationHandler + Send,
        ) -> Self {
            Self {
                inner: platform::PlatformAdapter::new(
                    window_handle,
                    activation_handler,
                    action_handler,
                    deactivation_handler,
                ),
            }
        }

        pub(super) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
            self.inner.update_if_active(updater);
        }

        pub(super) fn set_focus(&mut self, is_focused: bool) {
            self.inner.set_focus(is_focused);
        }

        pub(super) fn set_window_bounds(&mut self, outer_bounds: Rect, inner_bounds: Rect) {
            self.inner.set_window_bounds(outer_bounds, inner_bounds);
        }
    }
}
