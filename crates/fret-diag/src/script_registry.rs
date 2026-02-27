use std::path::{Path, PathBuf};

use crate::util::read_json_value;

#[derive(Debug, Clone)]
pub(crate) struct PromotedScriptRegistryEntry {
    pub id: String,
    pub path: String,
}

pub(crate) struct PromotedScriptRegistry {
    entries: Vec<PromotedScriptRegistryEntry>,
}

impl PromotedScriptRegistry {
    pub(crate) fn load_from_path(path: &Path) -> Result<Self, String> {
        let Some(value) = read_json_value(path) else {
            return Err(format!(
                "failed to read promoted scripts registry: {}",
                path.display()
            ));
        };

        let Some(scripts) = value.get("scripts").and_then(|v| v.as_array()) else {
            return Err(format!(
                "invalid promoted scripts registry (missing scripts[]): {}",
                path.display()
            ));
        };

        let mut entries = Vec::with_capacity(scripts.len());
        for (ix, script) in scripts.iter().enumerate() {
            let id = script
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    format!(
                        "invalid promoted scripts registry entry scripts[{ix}].id: {}",
                        path.display()
                    )
                })?
                .to_string();
            let path = script
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    format!(
                        "invalid promoted scripts registry entry scripts[{ix}].path: {}",
                        path.display()
                    )
                })?
                .to_string();
            entries.push(PromotedScriptRegistryEntry { id, path });
        }

        Ok(Self { entries })
    }

    pub(crate) fn resolve_id(&self, id: &str) -> Option<&PromotedScriptRegistryEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub(crate) fn suggest_ids(&self, query: &str, max: usize) -> Vec<String> {
        if query.trim().is_empty() || max == 0 {
            return Vec::new();
        }

        let q = query.trim();
        let q_lower = q.to_ascii_lowercase();

        let mut scored: Vec<(u32, &str)> = Vec::new();
        for entry in &self.entries {
            let id = entry.id.as_str();
            let id_lower = id.to_ascii_lowercase();

            let score = if id_lower == q_lower {
                0
            } else if id_lower.starts_with(&q_lower) {
                10
            } else if id_lower.contains(&q_lower) {
                20
            } else {
                let overlap = token_overlap_score(q, id);
                // Push token-overlap suggestions to the end, but keep them stable.
                100 + (50_u32.saturating_sub(overlap))
            };

            scored.push((score, id));
        }

        scored.sort_by(|(a_score, a_id), (b_score, b_id)| {
            a_score
                .cmp(b_score)
                .then_with(|| a_id.len().cmp(&b_id.len()))
                .then_with(|| a_id.cmp(b_id))
        });

        scored
            .into_iter()
            .map(|(_score, id)| id.to_string())
            .filter(|id| id.to_ascii_lowercase() != q_lower)
            .take(max)
            .collect()
    }
}

fn token_overlap_score(query: &str, id: &str) -> u32 {
    let q_tokens: Vec<&str> = query
        .split(|c| c == '-' || c == '_' || c == ' ')
        .filter(|s| !s.is_empty())
        .collect();
    if q_tokens.is_empty() {
        return 0;
    }

    let id_lower = id.to_ascii_lowercase();
    q_tokens
        .into_iter()
        .map(|t| {
            let t = t.to_ascii_lowercase();
            u32::from(id_lower.contains(&t))
        })
        .sum()
}

pub(crate) fn promoted_registry_default_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("tools")
        .join("diag-scripts")
        .join("index.json")
}

pub(crate) fn normalize_script_id_query(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(stem) = raw.strip_suffix(".json") {
        return stem.to_string();
    }
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_registry(dir: &Path, scripts: &[(&str, &str)]) -> PathBuf {
        let path = dir.join("index.json");
        let payload = serde_json::json!({
            "scripts": scripts.iter().map(|(id, path)| serde_json::json!({"id": id, "path": path})).collect::<Vec<_>>(),
        });
        std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap()).unwrap();
        path
    }

    #[test]
    fn normalize_script_id_query_strips_json_suffix() {
        assert_eq!(
            normalize_script_id_query("ui-gallery-smoke.json"),
            "ui-gallery-smoke"
        );
        assert_eq!(
            normalize_script_id_query("ui-gallery-smoke"),
            "ui-gallery-smoke"
        );
    }

    #[test]
    fn registry_load_and_resolve_id() {
        let dir = std::env::temp_dir().join(format!(
            "fret-diag-registry-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let registry_path = write_registry(
            &dir,
            &[
                (
                    "ui-gallery-gesture-pinch-smoke",
                    "tools/diag-scripts/a.json",
                ),
                ("ui-gallery-gesture-tap-smoke", "tools/diag-scripts/b.json"),
            ],
        );
        let registry = PromotedScriptRegistry::load_from_path(&registry_path).unwrap();

        let resolved = registry
            .resolve_id("ui-gallery-gesture-pinch-smoke")
            .unwrap();
        assert_eq!(resolved.path, "tools/diag-scripts/a.json");
        assert!(registry.resolve_id("missing").is_none());
    }

    #[test]
    fn registry_suggests_reasonable_ids() {
        let dir = std::env::temp_dir().join(format!(
            "fret-diag-registry-suggest-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let registry_path = write_registry(
            &dir,
            &[
                (
                    "ui-gallery-gesture-pinch-smoke",
                    "tools/diag-scripts/a.json",
                ),
                ("ui-gallery-gesture-tap-smoke", "tools/diag-scripts/b.json"),
                (
                    "ui-gallery-pointer-kind-touch-click-smoke",
                    "tools/diag-scripts/c.json",
                ),
            ],
        );
        let registry = PromotedScriptRegistry::load_from_path(&registry_path).unwrap();

        let suggestions = registry.suggest_ids("pinch", 5);
        assert!(suggestions.iter().any(|s| s.contains("pinch")));
    }
}
