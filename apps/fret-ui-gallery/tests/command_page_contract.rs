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
            "`CommandItem::children(...)` already covers row-level composability today. The deferred gap is the shared root context that upstream cmdk uses so `CommandInput`, `CommandList`, `CommandEmpty`, and `CommandGroup` can compose without manual query/selection wiring."
        ),
        "src/ui/pages/command.rs should keep row-level children support distinct from the deferred root shared-context API"
    );
    assert!(
        source.contains(
            "`CommandInput` / `CommandList` stay available for lower-level shell composition and legacy roving lists, but they do not share the cmdk query + active-descendant state machine."
        ),
        "src/ui/pages/command.rs should keep the lower-level shell status of CommandInput/CommandList explicit"
    );
    assert!(
        source.contains(
            "`CommandDialog::new(open, query, items)` wraps that palette with dialog lifecycle, input placeholder forwarding, close-on-select behavior, and open-change reason hooks for global command menus."
        ),
        "src/ui/pages/command.rs should keep CommandDialog's placeholder-forwarding surface explicit"
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
    assert!(
        source.contains(
            "Treat row-level children support and root-level shared-context support as separate questions: `CommandItem::children(...)` already ships, while split root composition still needs an explicit context contract."
        ),
        "src/ui/pages/command.rs should keep the item-level vs root-level composability distinction explicit in Notes"
    );
}

#[test]
fn command_docs_aligned_dialog_snippets_keep_upstream_placeholder_copy() {
    for relative in [
        "src/ui/snippets/command/basic.rs",
        "src/ui/snippets/command/shortcuts.rs",
        "src/ui/snippets/command/groups.rs",
        "src/ui/snippets/command/scrollable.rs",
    ] {
        let source = read(relative);
        assert!(
            source.contains(".placeholder(\"Type a command or search...\")"),
            "{relative} should keep the upstream Command dialog placeholder copy for docs-aligned examples"
        );
    }
}
