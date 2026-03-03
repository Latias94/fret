use std::path::{Path, PathBuf};

use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};
use serde::Deserialize;

pub(crate) struct SuiteRegistry {
    promoted: PromotedScriptRegistry,
}

pub(crate) struct SuiteResolver {
    registry: Option<SuiteRegistry>,
}

impl SuiteRegistry {
    pub(crate) fn load_from_workspace_root(workspace_root: &Path) -> Result<Self, String> {
        let registry_path = promoted_registry_default_path(workspace_root);
        if !registry_path.is_file() {
            return Err(format!(
                "promoted scripts registry is missing: {}\n\
hint: generate it via `cargo run -p fretboard -- diag registry write`",
                registry_path.display()
            ));
        }
        let promoted = PromotedScriptRegistry::load_from_path(&registry_path)?;
        Ok(Self { promoted })
    }

    pub(crate) fn try_load_from_workspace_root(
        workspace_root: &Path,
    ) -> Result<Option<Self>, String> {
        let registry_path = promoted_registry_default_path(workspace_root);
        if !registry_path.is_file() {
            return Ok(None);
        }
        let promoted = PromotedScriptRegistry::load_from_path(&registry_path)?;
        Ok(Some(Self { promoted }))
    }

    pub(crate) fn list_suites(&self) -> Vec<(String, usize)> {
        use std::collections::BTreeMap;
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for e in self.promoted.entries() {
            for s in &e.suite_memberships {
                *counts.entry(s.as_str()).or_insert(0) += 1;
            }
        }
        counts
            .into_iter()
            .map(|(suite, scripts_total)| (suite.to_string(), scripts_total))
            .collect()
    }

    pub(crate) fn resolve_promoted_suite_scripts(
        &self,
        workspace_root: &Path,
        suite: &str,
    ) -> Option<Vec<PathBuf>> {
        let mut scripts: Vec<PathBuf> = self
            .promoted
            .resolve_suite(suite)
            .into_iter()
            .map(|e| crate::paths::resolve_path(workspace_root, PathBuf::from(e.path.as_str())))
            .collect();
        scripts.sort();
        scripts.dedup();
        if scripts.is_empty() {
            None
        } else {
            Some(scripts)
        }
    }
}

impl SuiteResolver {
    pub(crate) fn try_load_from_workspace_root(workspace_root: &Path) -> Result<Self, String> {
        Ok(Self {
            registry: SuiteRegistry::try_load_from_workspace_root(workspace_root)?,
        })
    }

    pub(crate) fn suite_dir_exists(workspace_root: &Path, suite: &str) -> bool {
        workspace_root
            .join("tools")
            .join("diag-scripts")
            .join("suites")
            .join(suite)
            .is_dir()
    }

    pub(crate) fn scripts_from_suite_dir(
        workspace_root: &Path,
        suite: &str,
    ) -> Result<Vec<PathBuf>, String> {
        #[derive(Debug, Deserialize)]
        struct SuiteManifestV1 {
            schema_version: u64,
            kind: String,
            scripts: Vec<String>,
        }

        let suite_root = workspace_root
            .join("tools")
            .join("diag-scripts")
            .join("suites")
            .join(suite);

        for name in ["suite.json", "_suite.json"] {
            let manifest_path = suite_root.join(name);
            if !manifest_path.is_file() {
                continue;
            }

            let bytes = std::fs::read(&manifest_path).map_err(|e| e.to_string())?;
            let manifest: SuiteManifestV1 =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

            if manifest.schema_version != 1 {
                return Err(format!(
                    "invalid suite manifest schema_version (expected 1): {}",
                    manifest.schema_version
                ));
            }
            if manifest.kind != "diag_script_suite_manifest" {
                return Err(format!(
                    "invalid suite manifest kind (expected diag_script_suite_manifest): {:?}",
                    manifest.kind
                ));
            }
            if manifest.scripts.is_empty() {
                return Err(format!(
                    "suite manifest contains no scripts: {}",
                    manifest_path.display()
                ));
            }

            let mut out: Vec<PathBuf> = Vec::with_capacity(manifest.scripts.len());
            for raw in manifest.scripts {
                if raw.trim().is_empty() {
                    return Err(format!(
                        "suite manifest contains an empty script path: {}",
                        manifest_path.display()
                    ));
                }
                let resolved = crate::paths::resolve_path(workspace_root, PathBuf::from(raw));
                if !resolved.exists() {
                    return Err(format!(
                        "suite manifest script path does not exist: {} (manifest: {})",
                        resolved.display(),
                        manifest_path.display()
                    ));
                }
                out.push(resolved);
            }

            out.sort();
            out.dedup();
            return Ok(out);
        }

        let inputs = vec![format!("tools/diag-scripts/suites/{suite}")];
        crate::paths::expand_script_inputs(workspace_root, &inputs)
    }

    pub(crate) fn resolve_suite_scripts(
        &self,
        workspace_root: &Path,
        suite: &str,
    ) -> Result<Option<Vec<PathBuf>>, String> {
        if let Some(registry) = self.registry.as_ref()
            && let Some(scripts) = registry.resolve_promoted_suite_scripts(workspace_root, suite)
        {
            return Ok(Some(scripts));
        }
        if Self::suite_dir_exists(workspace_root, suite) {
            return Ok(Some(Self::scripts_from_suite_dir(workspace_root, suite)?));
        }
        Ok(None)
    }
}
