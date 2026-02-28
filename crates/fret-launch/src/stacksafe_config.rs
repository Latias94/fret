#[cfg(not(target_arch = "wasm32"))]
pub fn configure_stacksafe_from_env() {
    fret_launch_desktop::configure_stacksafe_from_env();
}

#[cfg(target_arch = "wasm32")]
pub fn configure_stacksafe_from_env() {}
