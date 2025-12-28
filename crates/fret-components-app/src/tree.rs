use std::sync::Arc;

use fret_components_ui::tree::TreeEntry;
use fret_components_ui::{TreeRowRenderer, TreeRowState};
use fret_core::SemanticsRole;
use fret_runtime::CommandId;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementCx, UiHost};

/// A minimal “application default” Tree row renderer.
///
/// This is intentionally not shadcn-branded. It exists to provide a reasonable baseline
/// look-and-feel for common app shells (file trees, navigation trees, outlines).
pub struct AppTreeRowRenderer;

impl<H: UiHost> TreeRowRenderer<H> for AppTreeRowRenderer {
    fn render_row(
        &mut self,
        cx: &mut ElementCx<'_, H>,
        entry: &TreeEntry,
        _state: TreeRowState,
    ) -> Vec<AnyElement> {
        vec![cx.text(entry.label.as_ref())]
    }

    fn render_trailing(
        &mut self,
        cx: &mut ElementCx<'_, H>,
        entry: &TreeEntry,
        state: TreeRowState,
    ) -> Vec<AnyElement> {
        let mut out = Vec::new();

        // Placeholder “row actions” affordance: app code can override the renderer to provide
        // icons/menus, but this proves the trailing slot works end-to-end.
        let cmd = CommandId::new(format!("app.tree.action.{}", entry.id));
        out.push(cx.pressable(
            PressableProps {
                enabled: !state.disabled,
                on_click: (!state.disabled).then_some(cmd),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(Arc::from("Row action")),
                    selected: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            |cx, _st| vec![cx.text("...")],
        ));

        out
    }
}
