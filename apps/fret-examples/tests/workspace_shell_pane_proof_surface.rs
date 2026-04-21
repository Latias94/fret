#[test]
fn workspace_shell_demo_keeps_shell_mounted_imui_pane_proof_explicit() {
    let source = include_str!("../src/workspace_shell_demo.rs");

    for needle in [
        "struct WorkspaceShellPaneProofState {",
        "fn workspace_shell_pane_proof<'a, Cx>(",
        "fret_imui::imui_build(cx, out, move |ui| {",
        "ui.child_region_with_options(",
        "\"workspace-shell-pane-{}-proof.shell\"",
        "\"workspace-shell-pane-{}-proof.toolbar\"",
        "\"workspace-shell-pane-{}-proof.tabs\"",
        "\"workspace-shell-pane-{}-proof.inspector\"",
        "\"workspace-shell-pane-{}-proof.status\"",
        "Decision: keep the current `child_region` seam for M3.",
        "vec![workspace_shell_pane_proof(",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell demo should keep the shell-mounted pane proof explicit; missing `{needle}`"
        );
    }
}
