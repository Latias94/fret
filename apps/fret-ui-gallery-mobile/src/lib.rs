#![cfg_attr(target_os = "android", allow(clippy::needless_return))]

#[cfg(target_os = "android")]
use winit::platform::android::{EventLoopBuilderExtAndroid as _, activity::AndroidApp};

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    init_android_tracing();

    let mut builder = winit::event_loop::EventLoop::builder();
    builder.with_android_app(app);

    let event_loop = match builder.build() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = ?e, "failed to build Android event loop");
            return;
        }
    };

    if let Err(e) = fret_ui_gallery::run_with_event_loop(event_loop) {
        tracing::error!(error = ?e, "fret-ui-gallery failed");
    }
}

#[cfg(target_os = "android")]
fn init_android_tracing() {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let layer = match tracing_android::layer("fret") {
        Ok(layer) => layer,
        Err(err) => {
            let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
            tracing::warn!(error = ?err, "failed to initialize Android logcat tracing layer");
            return;
        }
    };

    let _ = tracing_subscriber::registry()
        .with(layer)
        .with(filter)
        .try_init();
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
