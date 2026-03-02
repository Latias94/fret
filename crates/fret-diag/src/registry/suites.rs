use std::path::{Path, PathBuf};

use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};

pub(crate) struct SuiteRegistry {
    promoted: PromotedScriptRegistry,
}

impl SuiteRegistry {
    pub(crate) fn load_from_workspace_root(workspace_root: &Path) -> Result<Self, String> {
        let registry_path = promoted_registry_default_path(workspace_root);
        if !registry_path.is_file() {
            return Err(format!(
                "promoted scripts registry is missing: {}\n\
hint: generate it via `python tools/check_diag_scripts_registry.py --write`",
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
