use fret_app::App;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;

use super::{State, devtools_dogfood_workflow_lines, devtools_first_open_lines, diag_section};

pub(super) fn first_open_reference_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let rows = devtools_first_open_lines(st.cfg.fs_out_dir.as_ref())
        .into_iter()
        .map(|line| cx.text(line))
        .collect::<Vec<_>>();
    diag_section(
        cx,
        "First-open Evidence Path",
        "Canonical docs, repo preflight, artifact roots, product-chain evidence, and smoke gate stay visible in the GUI shell.",
        rows,
    )
}

pub(super) fn dogfood_reference_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
) -> AnyElement {
    let rows = devtools_dogfood_workflow_lines(st.cfg.fs_out_dir.as_ref())
        .into_iter()
        .map(|line| cx.text(line))
        .collect::<Vec<_>>();
    diag_section(
        cx,
        "Dogfood Workflow",
        "UI gallery selector capture, script patching, run/pack, and offline viewer handoff stay visible from the GUI shell.",
        rows,
    )
}
