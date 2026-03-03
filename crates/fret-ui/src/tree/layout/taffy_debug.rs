use super::*;
use crate::layout_engine::TaffyLayoutEngine;

impl<H: UiHost> UiTree<H> {
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
            let root_label = crate::declarative::frame::element_record_for_node(app, window, root)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
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

                    let label =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|r| format!("{:?}", r.instance))
                            .unwrap_or_default();
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

        let dump = engine.debug_dump_subtree_json(dump_root, |node| {
            crate::declarative::frame::element_record_for_node(app, window, node)
                .map(|r| format!("{:?}", r.instance))
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
            let root_label = crate::declarative::frame::element_record_for_node(app, window, root)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
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

                    let label =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|r| format!("{:?}", r.instance))
                            .unwrap_or_default();
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
            .debug_dump_subtree_json(dump_root, |node| {
                crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|r| format!("{:?}", r.instance))
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
        let dump_root = if let Some(filter) = root_label_filter {
            let root_label = crate::declarative::frame::element_record_for_node(app, window, root)
                .map(|r| format!("{:?}", r.instance))
                .unwrap_or_default();
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

                    let label =
                        crate::declarative::frame::element_record_for_node(app, window, node)
                            .map(|r| format!("{:?}", r.instance))
                            .unwrap_or_default();
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

        let dump = self
            .layout_engine
            .debug_dump_subtree_json(dump_root, |node| {
                crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|r| format!("{:?}", r.instance))
            });

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
