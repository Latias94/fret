mod driver;
mod harness;
mod spec;

mod ui;
pub use driver::{build_app, build_driver, build_runner_config, run};

#[cfg(not(target_arch = "wasm32"))]
pub use driver::run_with_event_loop;

#[cfg(test)]
mod authoring_surface_policy_tests {
    const MENUBAR: &str = include_str!("driver/menubar.rs");
    const ACTION_FIRST_VIEW: &str = include_str!("ui/snippets/command/action_first_view.rs");

    #[test]
    fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
        assert!(!MENUBAR.contains("fret::prelude"));
        assert!(MENUBAR.contains("use fret::workspace_menu::{"));

        assert!(!ACTION_FIRST_VIEW.contains("use fret::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("use fret::advanced::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("KernelApp"));
        assert!(!ACTION_FIRST_VIEW.contains("ViewCx<'_, '_, App>"));
        assert!(!ACTION_FIRST_VIEW.contains("ElementContext<'_, App>"));
    }
}
