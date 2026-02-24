#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestSnapshotV1 {
    pub position: PointV1,
    pub hit: Option<u64>,
    pub active_layer_roots: Vec<u64>,
    pub barrier_root: Option<u64>,
    #[serde(default)]
    pub focus_barrier_root: Option<u64>,
    /// Stable, script-friendly labels for each scope root.
    ///
    /// Prefer this over `active_layer_roots` when validating behavior across refactors, since node
    /// ids are not stable between runs.
    #[serde(default)]
    pub scope_roots: Vec<UiHitTestScopeRootV1>,
}

impl UiHitTestSnapshotV1 {
    fn from_tree(position: Point, ui: &mut UiTree<App>) -> Self {
        let hit_test = ui.debug_hit_test_routing(position);
        let arbitration = ui.input_arbitration_snapshot();
        let layers = ui.debug_layers_in_paint_order();
        Self::from_hit_test_with_layers(position, hit_test, arbitration.focus_barrier_root, &layers)
    }

    fn from_hit_test_with_layers(
        position: Point,
        hit_test: UiDebugHitTest,
        focus_barrier_root: Option<NodeId>,
        layers: &[UiDebugLayerInfo],
    ) -> Self {
        let mut scope_roots = Vec::new();
        if let Some(root) = hit_test.barrier_root {
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "modal_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: None,
                pointer_occlusion: None,
                blocks_underlay_input: None,
                hit_testable: None,
            });
        }

        let mut by_root: HashMap<NodeId, &UiDebugLayerInfo> = HashMap::new();
        for layer in layers {
            by_root.insert(layer.root, layer);
        }

        if let Some(root) = focus_barrier_root {
            let info = by_root.get(&root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "focus_barrier_root".to_string(),
                root: key_to_u64(root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        for root in &hit_test.active_layer_roots {
            let info = by_root.get(root);
            scope_roots.push(UiHitTestScopeRootV1 {
                kind: "layer_root".to_string(),
                root: key_to_u64(*root),
                layer_id: info.map(|l| l.id.data().as_ffi()),
                pointer_occlusion: info.map(|l| pointer_occlusion_label(l.pointer_occlusion)),
                blocks_underlay_input: info.map(|l| l.blocks_underlay_input),
                hit_testable: info.map(|l| l.hit_testable),
            });
        }

        Self {
            position: PointV1::from(position),
            hit: hit_test.hit.map(key_to_u64),
            active_layer_roots: hit_test
                .active_layer_roots
                .into_iter()
                .map(key_to_u64)
                .collect(),
            barrier_root: hit_test.barrier_root.map(key_to_u64),
            focus_barrier_root: focus_barrier_root.map(key_to_u64),
            scope_roots,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHitTestScopeRootV1 {
    /// Stable scope root kind (e.g. `modal_barrier_root`, `layer_root`).
    pub kind: String,
    /// Node id of the root (not stable between runs; treat as an in-run reference only).
    pub root: u64,
    /// When `kind=layer_root`, the corresponding layer id (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<u64>,
    /// Pointer occlusion mode for the layer root (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_occlusion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_underlay_input: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_testable: Option<bool>,
}
