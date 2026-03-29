const LIB_RS: &str = include_str!("../src/lib.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn shared_response_contract_stays_small_and_ecosystem_friendly() {
    let normalized = normalize_ws(LIB_RS);
    let required_markers = [
        "pub struct Response {",
        "pub hovered: bool,",
        "pub pressed: bool,",
        "pub focused: bool,",
        "pub clicked: bool,",
        "pub changed: bool,",
        "pub rect: Option<Rect>,",
        "pub fn clicked(self) -> bool {",
        "pub fn changed(self) -> bool {",
    ];
    let forbidden_markers = [
        "secondary_clicked",
        "double_clicked",
        "context_menu_requested",
        "drag_started",
        "dragging",
        "drag_stopped",
        "long_pressed",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "Response should keep the small shared interaction contract used across ecosystem authoring frontends"
        );
    }

    for marker in forbidden_markers {
        assert!(
            !LIB_RS.contains(marker),
            "Response should stay free of richer facade-only interaction signals"
        );
    }
}

#[test]
fn ui_writer_contract_stays_minimal_and_frontend_generic() {
    let normalized = normalize_ws(LIB_RS);
    let required_markers = [
        "pub trait UiWriter<H: UiHost> {",
        "fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R;",
        "fn add(&mut self, element: AnyElement);",
        "fn extend<I>(&mut self, elements: I)",
        "fn mount<I>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> I)",
        "fn keyed<K: Hash, R>(&mut self, key: K, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {",
    ];
    let forbidden_markers = [
        "ImUi",
        "fret_ui_kit",
        "fret_ui_shadcn",
        "Model<",
        "UiBuilder",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "UiWriter should keep the minimal shared authoring surface for ecosystem adapters"
        );
    }

    for marker in forbidden_markers {
        assert!(
            !LIB_RS.contains(marker),
            "UiWriter should not couple fret-authoring to a concrete frontend or richer facade vocabulary"
        );
    }
}
