#![allow(clippy::bool_comparison)]
#![allow(clippy::default_constructed_unit_structs)]

//! Workspace-shell building blocks (editor-grade app chrome).
//!
//! This crate intentionally lives in `ecosystem/`:
//! - It is policy-heavy and will iterate faster than `crates/fret-ui`.
//! - It should not expand the `fret-ui` runtime contract surface (ADR 0066).

pub mod close_policy;
mod command_scope;
pub mod commands;
mod focus_registry;
mod frame;
pub mod layout;
pub mod menu;
mod pane_content_focus;
pub mod panes;
pub mod tab_drag;
mod tab_strip;
pub mod tabs;
mod theme_tokens;

pub use command_scope::WorkspaceCommandScope;
pub use frame::{WorkspaceFrame, WorkspaceStatusBar, WorkspaceTopBar};
pub use pane_content_focus::WorkspacePaneContentFocusTarget;
pub use panes::workspace_pane_tree_element_with_resize;
pub use tab_drag::{DRAG_KIND_WORKSPACE_TAB, WorkspaceTabDragState};
pub use tab_strip::{WorkspaceTab, WorkspaceTabStrip};

#[cfg(test)]
mod source_policy_tests {
    const FRAME_RS: &str = include_str!("frame.rs");
    const COMMAND_SCOPE_RS: &str = include_str!("command_scope.rs");
    const PANE_CONTENT_FOCUS_RS: &str = include_str!("pane_content_focus.rs");

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect()
    }

    #[test]
    fn workspace_shell_wrappers_accept_typed_inputs_before_landing() {
        let frame = normalize_ws(FRAME_RS);
        let command_scope = normalize_ws(COMMAND_SCOPE_RS);
        let pane_focus = normalize_ws(PANE_CONTENT_FOCUS_RS);

        assert!(frame.contains("pubstructWorkspaceFrame<Center=AnyElement,Top=AnyElement,Left=AnyElement,Right=AnyElement,Bottom=AnyElement,>{"));
        assert!(frame.contains("Center:IntoUiElement<H>"));
        assert!(frame.contains("Top:IntoUiElement<H>"));
        assert!(frame.contains("Left:IntoUiElement<H>"));
        assert!(frame.contains("Right:IntoUiElement<H>"));
        assert!(frame.contains("Bottom:IntoUiElement<H>"));
        assert!(
            frame.contains(
                "pubfninto_element_in<'a,H:UiHost+'a,Cx>(self,cx:&mutCx)->AnyElementwhereCx:ElementContextAccess<'a,H>,Center:IntoUiElement<H>,Top:IntoUiElement<H>,Left:IntoUiElement<H>,Right:IntoUiElement<H>,Bottom:IntoUiElement<H>,"
            )
        );
        assert!(command_scope.contains("pubstructWorkspaceCommandScope<T=AnyElement>{"));
        assert!(command_scope.contains("T:IntoUiElement<H>"));
        assert!(pane_focus.contains("pubstructWorkspacePaneContentFocusTarget<T=AnyElement>{"));
        assert!(pane_focus.contains("T:IntoUiElement<H>"));
    }

    #[test]
    fn workspace_bar_aggregators_keep_explicit_raw_collection_seams() {
        let frame = normalize_ws(FRAME_RS);

        assert!(
            FRAME_RS.contains("This intentionally remains an explicit `AnyElement` landing seam")
        );
        assert!(
            frame.contains("pubfnleft(mutself,children:implIntoIterator<Item=AnyElement>)->Self{")
        );
        assert!(
            frame
                .contains("pubfncenter(mutself,children:implIntoIterator<Item=AnyElement>)->Self{")
        );
        assert!(
            frame.contains("pubfnright(mutself,children:implIntoIterator<Item=AnyElement>)->Self{")
        );
        assert!(
            frame.contains("pubfnleft(mutself,children:implIntoIterator<Item=AnyElement>)->Self{")
        );
        assert!(
            frame.contains("pubfnright(mutself,children:implIntoIterator<Item=AnyElement>)->Self{")
        );
    }
}
