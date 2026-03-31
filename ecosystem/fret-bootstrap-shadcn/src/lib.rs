//! Thin shadcn recipe integrations for `fret-bootstrap`.

use fret_app::App;
use fret_bootstrap::ui_app_driver::{CommandPaletteOverlayCx, UiAppDriver, ViewElements};
use fret_ui::ElementContext;
use fret_ui_shadcn::facade as shadcn;

/// Install the default shadcn command palette overlay on a bootstrap UI driver.
pub fn with_shadcn_command_palette<S>(driver: UiAppDriver<S>) -> UiAppDriver<S> {
    driver.command_palette_overlay(render_command_palette_overlay)
}

/// Render the default shadcn command palette overlay.
pub fn render_command_palette_overlay(
    cx: &mut ElementContext<'_, App>,
    overlay: CommandPaletteOverlayCx,
    out: &mut ViewElements,
) {
    let entries: Vec<shadcn::CommandEntry> = if overlay.open {
        fret_ui_kit::command::command_catalog_entries_from_host_commands_with_options(
            cx,
            fret_ui_kit::command::CommandCatalogOptions::default(),
        )
        .into_iter()
        .map(Into::into)
        .collect()
    } else {
        Vec::new()
    };

    let dialog = shadcn::CommandDialog::new(overlay.models.open, overlay.models.query, Vec::new())
        .entries(entries)
        .a11y_label("Command palette")
        .into_element(cx, |cx| {
            cx.interactivity_gate_props(
                fret_ui::element::InteractivityGateProps {
                    present: false,
                    interactive: false,
                    ..Default::default()
                },
                |_| vec![],
            )
        });
    out.push(dialog);
}
