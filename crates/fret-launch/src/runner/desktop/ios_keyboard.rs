use std::{ptr::NonNull, sync::Mutex};

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::CGRect;
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSNotificationName, NSObjectProtocol,
};
use objc2_ui_kit::{
    NSValueUIGeometryExtensions as _, UIKeyboardFrameEndUserInfoKey,
    UIKeyboardWillChangeFrameNotification, UIKeyboardWillHideNotification, UIView,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

pub(crate) struct IosKeyboardTracker {
    keyboard_frame_screen: std::sync::Arc<Mutex<Option<CGRect>>>,
    _will_change_frame_observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
    _will_hide_observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}

impl IosKeyboardTracker {
    pub(crate) fn new() -> Self {
        let center = NSNotificationCenter::defaultCenter();
        let keyboard_frame_screen = std::sync::Arc::new(Mutex::new(None));

        let will_change_frame = {
            let keyboard_frame_screen = std::sync::Arc::clone(&keyboard_frame_screen);
            create_observer(
                &center,
                unsafe { UIKeyboardWillChangeFrameNotification },
                move |notification| {
                    let Some(user_info) = notification.userInfo() else {
                        return;
                    };

                    let Some(value) =
                        user_info.objectForKey(unsafe { UIKeyboardFrameEndUserInfoKey })
                    else {
                        return;
                    };

                    let Ok(value) = value.downcast::<objc2_foundation::NSValue>() else {
                        return;
                    };

                    let rect = unsafe { value.CGRectValue() };
                    if let Ok(mut guard) = keyboard_frame_screen.lock() {
                        *guard = Some(rect);
                    }
                },
            )
        };

        let will_hide = {
            let keyboard_frame_screen = std::sync::Arc::clone(&keyboard_frame_screen);
            create_observer(
                &center,
                unsafe { UIKeyboardWillHideNotification },
                move |_notification| {
                    if let Ok(mut guard) = keyboard_frame_screen.lock() {
                        *guard = None;
                    }
                },
            )
        };

        Self {
            keyboard_frame_screen,
            _will_change_frame_observer: will_change_frame,
            _will_hide_observer: will_hide,
        }
    }

    pub(crate) fn keyboard_frame_screen(&self) -> Option<CGRect> {
        self.keyboard_frame_screen.lock().ok().and_then(|g| *g)
    }
}

fn create_observer(
    center: &NSNotificationCenter,
    name: &NSNotificationName,
    handler: impl Fn(&NSNotification) + 'static,
) -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    let block = RcBlock::new(move |notification: NonNull<NSNotification>| {
        handler(unsafe { notification.as_ref() });
    });
    unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(name),
            None, // No sender filter
            None, // No queue, run on posting thread (i.e. main thread)
            &block,
        )
    }
}

fn rect_intersection_height(a: CGRect, b: CGRect) -> f64 {
    let ax0 = a.origin.x;
    let ay0 = a.origin.y;
    let ax1 = a.origin.x + a.size.width;
    let ay1 = a.origin.y + a.size.height;

    let bx0 = b.origin.x;
    let by0 = b.origin.y;
    let bx1 = b.origin.x + b.size.width;
    let by1 = b.origin.y + b.size.height;

    let x0 = ax0.max(bx0);
    let y0 = ay0.max(by0);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    let w = (x1 - x0).max(0.0);
    let h = (y1 - y0).max(0.0);

    if w <= 0.0 || h <= 0.0 { 0.0 } else { h }
}

pub(crate) fn keyboard_overlap_bottom_in_window_points(
    window: &dyn Window,
    keyboard_frame_screen: CGRect,
) -> Option<f32> {
    let view_ptr = match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::UiKit(handle) => handle.ui_view.as_ptr(),
        _ => return None,
    };

    // SAFETY: `ui_view` is a valid `UIView*` for the lifetime of the `Window`.
    let view: &UIView = unsafe { &*(view_ptr.cast::<UIView>()) };

    // UIKit's keyboard frame is in screen coordinates. Passing `None` matches typical Cocoa
    // idioms (`convertRect:fromView:nil`) to convert from window base coordinates.
    let keyboard_in_view = view.convertRect_fromView(keyboard_frame_screen, None);

    let bounds = view.bounds();
    let overlap_h = rect_intersection_height(bounds, keyboard_in_view);

    if overlap_h.is_finite() && overlap_h > 0.0 {
        Some(overlap_h.min(bounds.size.height) as f32)
    } else {
        Some(0.0)
    }
}
