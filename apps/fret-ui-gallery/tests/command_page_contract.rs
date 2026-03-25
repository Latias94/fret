use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()))
}

#[test]
fn command_page_records_docs_vs_registry_alignment_axes() {
    let source = read("src/ui/pages/command.rs");
    assert!(
        source.contains(
            "The demo follows the public docs example surface (`max-w-sm`, rounded border, copyable example shape), while recipe-owned chrome is validated separately against the registry source."
        ),
        "src/ui/pages/command.rs should record the docs-example vs registry-chrome alignment split for Command"
    );
}

#[test]
fn command_page_records_shell_vs_cmdk_split_children_contract() {
    let source = read("src/ui/pages/command.rs");
    assert!(
        source.contains(
            "`CommandInput` / `CommandList` stay available for lower-level shell composition and legacy roving lists, but they do not share the cmdk query + active-descendant state machine."
        ),
        "src/ui/pages/command.rs should keep the lower-level shell status of CommandInput/CommandList explicit"
    );
    assert!(
        source.contains(
            "`Composable Shell (Fret)` shows the current explicit manual lane: share a query model between `CommandInput` and `CommandList` when you need a custom shell, but keep cmdk-style active-descendant, committed selection, and dialog lifecycle on `CommandPalette` / `CommandDialog`."
        ),
        "src/ui/pages/command.rs should explain the current explicit manual shell lane for Command"
    );
    assert!(
        source.contains(
            "upstream cmdk composes those parts through shared internal state, so promoting the same shape in Fret would first require an explicit shared context contract for query, active row, and selection rather than ad-hoc glue."
        ),
        "src/ui/pages/command.rs should explain why the default surface does not promote split children authoring yet"
    );
}
