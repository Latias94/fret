#![cfg_attr(target_os = "android", allow(clippy::needless_return))]

#[cfg(target_os = "android")]
use winit::platform::android::{EventLoopBuilderExtAndroid as _, activity::AndroidApp};

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    let mut builder = winit::event_loop::EventLoop::builder();
    builder.with_android_app(app);

    let event_loop = match builder.build() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to build Android event loop: {e:?}");
            return;
        }
    };

    if let Err(e) = fret_ui_gallery::run_with_event_loop(event_loop) {
        eprintln!("fret-ui-gallery failed: {e:?}");
    }
}

/// iOS embedding entrypoint (for an eventual Xcode wrapper).
///
/// Note: iOS apps must call into Rust from the main thread.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn fret_ui_gallery_ios_main() {
    if let Err(e) = fret_ui_gallery::run() {
        eprintln!("fret-ui-gallery failed: {e:?}");
    }
}
