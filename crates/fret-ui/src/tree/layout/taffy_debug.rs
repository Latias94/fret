use super::*;
use crate::layout_engine::{DebugDumpNodeInfo, TaffyLayoutEngine};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy)]
struct LayoutSidecarRootRecord {
    capture_index: usize,
    kind: &'static str,
    root: NodeId,
    root_bounds: Rect,
    blocks_underlay_input: bool,
    blocks_underlay_focus: bool,
    hit_testable: bool,
}

fn layout_debug_node_info<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> DebugDumpNodeInfo {
    let Some(record) = crate::declarative::frame::element_record_for_node(app, window, node) else {
        return DebugDumpNodeInfo::default();
    };

    let mut label = format!("{:?}", record.instance);
    let mut debug = serde_json::Map::new();
    debug.insert(
        "element_id".to_string(),
        serde_json::json!(record.element.0),
    );
    debug.insert(
        "instance_kind".to_string(),
        serde_json::json!(record.instance.kind_name()),
    );

    let effective_test_id =
        crate::declarative::frame::effective_test_id_for_record(&record).map(ToString::to_string);
    let mut effective_role: Option<String> = None;
    let mut effective_label: Option<String> = None;

    match &record.instance {
        crate::declarative::frame::ElementInstance::Semantics(props) => {
            effective_role = Some(format!("{:?}", props.role));
            effective_label = props.label.as_ref().map(ToString::to_string);
        }
        crate::declarative::frame::ElementInstance::SemanticFlex(props) => {
            effective_role = Some(format!("{:?}", props.role));
        }
        _ => {}
    }

    if let Some(decoration) = record.semantics_decoration.as_ref() {
        let mut decoration_debug = serde_json::Map::new();
        if let Some(test_id) = decoration.test_id.as_ref() {
            decoration_debug.insert("test_id".to_string(), serde_json::json!(test_id.as_ref()));
        }
        if let Some(role) = decoration.role {
            let role = format!("{role:?}");
            decoration_debug.insert("role".to_string(), serde_json::json!(role));
            effective_role = Some(role);
        }
        if let Some(label_text) = decoration.label.as_ref() {
            decoration_debug.insert("label".to_string(), serde_json::json!(label_text.as_ref()));
            effective_label = Some(label_text.to_string());
        }
        if !decoration_debug.is_empty() {
            debug.insert(
                "semantics_decoration".to_string(),
                serde_json::Value::Object(decoration_debug),
            );
        }
    }

    if let Some(test_id) = effective_test_id.as_ref() {
        debug.insert("test_id".to_string(), serde_json::json!(test_id));
        label.push_str(&format!(" [test_id={test_id}]"));
    }
    if let Some(role) = effective_role.as_ref() {
        debug.insert("semantics_role".to_string(), serde_json::json!(role));
        label.push_str(&format!(" [semantics_role={role}]"));
    }
    if let Some(label_text) = effective_label.as_ref() {
        debug.insert("semantics_label".to_string(), serde_json::json!(label_text));
        label.push_str(&format!(" [semantics_label={label_text}]"));
    }
    if let Some(key_context) = record.key_context.as_ref() {
        debug.insert(
            "key_context".to_string(),
            serde_json::json!(key_context.as_ref()),
        );
    }

    DebugDumpNodeInfo {
        label: Some(label),
        debug: Some(serde_json::Value::Object(debug)),
    }
}

fn layout_debug_search_label<H: UiHost>(app: &mut H, window: AppWindowId, node: NodeId) -> String {
    layout_debug_node_info(app, window, node)
        .label
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn find_layout_debug_match_in_subtree<H: UiHost>(
    tree: &UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    root: NodeId,
    filter: &str,
) -> Option<NodeId> {
    let root_label = layout_debug_search_label(app, window, root);
    if root_label.contains(filter) {
        return Some(root);
    }

    let mut stack: Vec<NodeId> = vec![root];
    let mut visited: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }

        let label = layout_debug_search_label(app, window, node);
        if label.contains(filter) {
            return Some(node);
        }

        if let Some(node) = tree.nodes.get(node) {
            stack.extend(node.children.iter().copied());
        }
    }

    None
}

impl<H: UiHost> UiTree<H> {
    #[cfg(not(target_arch = "wasm32"))]
    fn layout_sidecar_roots(
        &self,
        fallback_root: NodeId,
        fallback_bounds: Rect,
    ) -> Vec<LayoutSidecarRootRecord> {
        let layer_roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .filter_map(|layer_id| self.layers.get(layer_id).map(|layer| layer.root))
            .collect();

        let mut seen: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
        let mut roots: Vec<LayoutSidecarRootRecord> = self
            .visible_layers_in_paint_order()
            .enumerate()
            .filter_map(|(capture_index, layer_id)| {
                let layer = self.layers.get(layer_id)?;
                if !seen.insert(layer.root) {
                    return None;
                }
                let root_bounds = self
                    .nodes
                    .get(layer.root)
                    .map(|node| node.bounds)
                    .unwrap_or(fallback_bounds);
                Some(LayoutSidecarRootRecord {
                    capture_index,
                    kind: "layer",
                    root: layer.root,
                    root_bounds,
                    blocks_underlay_input: layer.blocks_underlay_input,
                    blocks_underlay_focus: layer.blocks_underlay_focus,
                    hit_testable: layer.hit_testable,
                })
            })
            .collect();

        let viewport_bounds: std::collections::HashMap<NodeId, Rect> =
            self.viewport_roots().iter().copied().collect();
        for root in self.layout_engine.debug_independent_root_nodes() {
            if !seen.insert(root) {
                continue;
            }
            if !layer_roots.is_empty()
                && !self.is_reachable_from_any_root_via_children(root, &layer_roots)
            {
                continue;
            }

            let root_bounds = viewport_bounds
                .get(&root)
                .copied()
                .or_else(|| self.nodes.get(root).map(|node| node.bounds))
                .unwrap_or(fallback_bounds);
            let kind = if viewport_bounds.contains_key(&root) {
                "viewport"
            } else {
                "independent"
            };
            roots.push(LayoutSidecarRootRecord {
                capture_index: roots.len(),
                kind,
                root,
                root_bounds,
                blocks_underlay_input: false,
                blocks_underlay_focus: false,
                hit_testable: true,
            });
        }

        if roots.is_empty() {
            roots.push(LayoutSidecarRootRecord {
                capture_index: 0,
                kind: "fallback",
                root: fallback_root,
                root_bounds: fallback_bounds,
                blocks_underlay_input: false,
                blocks_underlay_focus: false,
                hit_testable: true,
            });
        }

        roots
    }

    pub(super) fn maybe_dump_taffy_subtree(
        &self,
        app: &mut H,
        window: AppWindowId,
        engine: &TaffyLayoutEngine,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
    ) {
        use std::sync::atomic::{AtomicU32, Ordering};

        let config = crate::runtime_config::ui_runtime_config();
        let Some(taffy_dump) = config.taffy_dump.as_ref() else {
            return;
        };

        static DUMP_COUNT: AtomicU32 = AtomicU32::new(0);
        let dump_max: Option<u32> = if config.taffy_dump_once {
            Some(1)
        } else {
            taffy_dump.max
        };
        if let Some(max) = dump_max {
            let prev = DUMP_COUNT.fetch_add(1, Ordering::SeqCst);
            if prev >= max {
                return;
            }
        }

        if let Some(filter) = taffy_dump.root_filter.as_ref()
            && !format!("{root:?}").contains(filter)
        {
            return;
        }

        // When debugging complex demos or golden-gated layouts, it is often easier to filter by a
        // stable element label (e.g. a `SemanticsProps.label`) than by ephemeral `NodeId`s.
        let dump_root = if let Some(filter) = taffy_dump.root_label_filter.as_ref() {
            let root_label = layout_debug_search_label(app, window, root);
            if root_label.contains(filter) {
                root
            } else {
                let mut stack: Vec<NodeId> = vec![root];
                let mut visited: std::collections::HashSet<NodeId> =
                    std::collections::HashSet::new();
                let mut found: Option<NodeId> = None;
                while let Some(node) = stack.pop() {
                    if !visited.insert(node) {
                        continue;
                    }

                    let label = layout_debug_search_label(app, window, node);
                    if label.contains(filter) {
                        found = Some(node);
                        break;
                    }

                    if let Some(node) = self.nodes.get(node) {
                        stack.extend(node.children.iter().copied());
                    }
                }

                let Some(found) = found else {
                    return;
                };

                found
            }
        } else {
            root
        };

        let out_dir = taffy_dump.out_dir.clone();

        let frame = app.frame_id().0;
        let root_slug: String = format!("{dump_root:?}")
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect();
        let filename = format!("taffy_{frame}_{root_slug}.json");

        let dump = engine.debug_dump_subtree_json_with_info(dump_root, |node| {
            layout_debug_node_info(app, window, node)
        });

        let wrapped = serde_json::json!({
            "meta": {
                "window": format!("{window:?}"),
                "root_bounds": {
                    "x": root_bounds.origin.x.0,
                    "y": root_bounds.origin.y.0,
                    "w": root_bounds.size.width.0,
                    "h": root_bounds.size.height.0,
                },
                "scale_factor": scale_factor,
            },
            "taffy": dump,
        });

        let result = std::fs::create_dir_all(&out_dir)
            .and_then(|_| {
                serde_json::to_vec_pretty(&wrapped)
                    .map_err(|e| std::io::Error::other(format!("serialize: {e}")))
            })
            .and_then(|bytes| {
                std::fs::write(std::path::Path::new(&out_dir).join(&filename), bytes)
            });

        match result {
            Ok(()) => tracing::info!(
                out_dir = %out_dir,
                filename = %filename,
                "wrote taffy debug dump"
            ),
            Err(err) => tracing::warn!(
                error = %err,
                out_dir = %out_dir,
                filename = %filename,
                "failed to write taffy debug dump"
            ),
        }
    }

    /// Write a Taffy layout dump for a subtree rooted at `root`.
    ///
    /// The dump includes both local and absolute rects plus a debug label per node. When
    /// `root_label_filter` is provided, the dump will search for the first node whose debug label
    /// contains the filter string and use that node as the dump root (falling back to `root` when
    /// the filter does not match anything).
    ///
    /// This is a debug-only escape hatch intended for diagnosing layout regressions and scroll /
    /// clipping issues. The output is JSON and is written to `out_dir`.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    pub fn debug_write_taffy_subtree_json(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
        root_label_filter: Option<&str>,
        out_dir: impl AsRef<std::path::Path>,
        filename_tag: &str,
    ) -> std::io::Result<std::path::PathBuf> {
        fn sanitize_for_filename(s: &str) -> String {
            s.chars()
                .map(|ch| match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
                    _ => '_',
                })
                .collect()
        }

        let dump_root = if let Some(filter) = root_label_filter {
            let root_label = layout_debug_search_label(app, window, root);
            if root_label.contains(filter) {
                root
            } else {
                let mut stack: Vec<NodeId> = vec![root];
                let mut visited: std::collections::HashSet<NodeId> =
                    std::collections::HashSet::new();
                let mut found: Option<NodeId> = None;
                while let Some(node) = stack.pop() {
                    if !visited.insert(node) {
                        continue;
                    }

                    let label = layout_debug_search_label(app, window, node);
                    if label.contains(filter) {
                        found = Some(node);
                        break;
                    }

                    if let Some(node) = self.nodes.get(node) {
                        stack.extend(node.children.iter().copied());
                    }
                }

                found.unwrap_or(root)
            }
        } else {
            root
        };

        let tag = sanitize_for_filename(filename_tag);
        let frame = app.frame_id().0;
        let root_slug = sanitize_for_filename(&format!("{dump_root:?}"));
        let filename = if tag.is_empty() {
            format!("taffy_{frame}_{root_slug}.json")
        } else {
            format!("taffy_{frame}_{tag}_{root_slug}.json")
        };

        let dump = self
            .layout_engine
            .debug_dump_subtree_json_with_info(dump_root, |node| {
                layout_debug_node_info(app, window, node)
            });

        let wrapped = serde_json::json!({
            "meta": {
                "window": format!("{window:?}"),
                "root_bounds": {
                    "x": root_bounds.origin.x.0,
                    "y": root_bounds.origin.y.0,
                    "w": root_bounds.size.width.0,
                    "h": root_bounds.size.height.0,
                },
                "scale_factor": scale_factor,
            },
            "taffy": dump,
        });

        let out_dir = out_dir.as_ref();
        std::fs::create_dir_all(out_dir)?;
        let path = out_dir.join(filename);
        let bytes = serde_json::to_vec_pretty(&wrapped)
            .map_err(|e| std::io::Error::other(format!("serialize: {e}")))?;
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    /// Write a bundle-scoped layout sidecar (Taffy dump) intended for scripted diagnostics runs.
    ///
    /// This is a diagnostics-only escape hatch and should remain best-effort. Tooling should treat
    /// missing sidecars as warnings rather than failures.
    ///
    /// The file name is stable: `layout.taffy.v1.json`.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    pub fn debug_write_layout_sidecar_taffy_v1_json(
        &self,
        app: &mut H,
        window: AppWindowId,
        root: NodeId,
        root_bounds: Rect,
        scale_factor: f32,
        root_label_filter: Option<&str>,
        out_dir: impl AsRef<std::path::Path>,
        captured_at_unix_ms: u64,
    ) -> std::io::Result<std::path::PathBuf> {
        let sidecar_roots = self.layout_sidecar_roots(root, root_bounds);
        let dump_root = if let Some(filter) = root_label_filter {
            sidecar_roots
                .iter()
                .rev()
                .find_map(|root_record| {
                    find_layout_debug_match_in_subtree(self, app, window, root_record.root, filter)
                })
                .unwrap_or(root)
        } else {
            root
        };

        let mut dump = self
            .layout_engine
            .debug_dump_subtree_json_with_info(dump_root, |node| {
                layout_debug_node_info(app, window, node)
            });
        let root_dumps = sidecar_roots
            .iter()
            .map(|root_record| {
                serde_json::json!({
                    "capture_index": root_record.capture_index,
                    "kind": root_record.kind,
                    "root": format!("{:?}", root_record.root),
                    "root_bounds": {
                        "x": root_record.root_bounds.origin.x.0,
                        "y": root_record.root_bounds.origin.y.0,
                        "w": root_record.root_bounds.size.width.0,
                        "h": root_record.root_bounds.size.height.0,
                    },
                    "blocks_underlay_input": root_record.blocks_underlay_input,
                    "blocks_underlay_focus": root_record.blocks_underlay_focus,
                    "hit_testable": root_record.hit_testable,
                    "dump": self.layout_engine.debug_dump_subtree_json_with_info(
                        root_record.root,
                        |node| layout_debug_node_info(app, window, node),
                    ),
                })
            })
            .collect::<Vec<_>>();
        if let Some(dump_obj) = dump.as_object_mut() {
            dump_obj.insert("roots".to_string(), serde_json::Value::Array(root_dumps));
        }

        let wrapped = serde_json::json!({
            "schema_version": "v1",
            "engine": "taffy",
            "captured_at_unix_ms": captured_at_unix_ms,
            "clip": {
                "max_nodes": 0u64,
                "max_bytes": 0u64,
                "clipped_nodes": 0u64,
                "clipped_bytes": 0u64,
            },
            "meta": {
                "window": format!("{window:?}"),
                "root_bounds": {
                    "x": root_bounds.origin.x.0,
                    "y": root_bounds.origin.y.0,
                    "w": root_bounds.size.width.0,
                    "h": root_bounds.size.height.0,
                },
                "scale_factor": scale_factor,
                "root_label_filter": root_label_filter,
                "captured_root_count": sidecar_roots.len(),
                "visible_layer_root_count": self.visible_layers_in_paint_order().count(),
            },
            "taffy": dump,
        });

        let out_dir = out_dir.as_ref();
        std::fs::create_dir_all(out_dir)?;
        let path = out_dir.join("layout.taffy.v1.json");
        let bytes = serde_json::to_vec(&wrapped)
            .map_err(|e| std::io::Error::other(format!("serialize: {e}")))?;
        std::fs::write(&path, bytes)?;
        Ok(path)
    }
}
