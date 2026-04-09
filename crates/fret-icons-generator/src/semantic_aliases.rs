use std::collections::BTreeSet;
use std::path::Path;

use crate::GeneratePackError;
use crate::contracts::{SemanticAlias, SemanticAliasConfigFileV1};

const SEMANTIC_ALIAS_CONFIG_SCHEMA_V1: u32 = 1;

pub(crate) fn sanitize_semantic_aliases(
    aliases: Vec<SemanticAlias>,
) -> Result<Vec<SemanticAlias>, GeneratePackError> {
    let mut seen_ids = BTreeSet::new();
    let mut sanitized = Vec::with_capacity(aliases.len());

    for alias in aliases {
        let semantic_id = alias.semantic_id.trim().to_string();
        let target_icon = alias.target_icon.trim().to_string();

        if semantic_id.is_empty() {
            return Err(GeneratePackError::EmptySemanticAliasId);
        }
        if !semantic_id.starts_with("ui.") {
            return Err(GeneratePackError::SemanticAliasMustUseUiNamespace { semantic_id });
        }
        if !seen_ids.insert(semantic_id.clone()) {
            return Err(GeneratePackError::DuplicateSemanticAliasId { semantic_id });
        }

        sanitized.push(SemanticAlias {
            semantic_id,
            target_icon,
        });
    }

    Ok(sanitized)
}

pub fn load_semantic_aliases_json_file(
    path: &Path,
) -> Result<Vec<SemanticAlias>, GeneratePackError> {
    if !path.exists() {
        return Err(GeneratePackError::MissingSemanticAliasConfigFile(
            path.display().to_string(),
        ));
    }
    if !path.is_file() {
        return Err(GeneratePackError::SemanticAliasConfigPathNotFile(
            path.display().to_string(),
        ));
    }

    let content = std::fs::read_to_string(path)?;
    let config: SemanticAliasConfigFileV1 = serde_json::from_str(&content)?;
    if config.schema_version != SEMANTIC_ALIAS_CONFIG_SCHEMA_V1 {
        return Err(
            GeneratePackError::UnsupportedSemanticAliasConfigSchemaVersion {
                expected: SEMANTIC_ALIAS_CONFIG_SCHEMA_V1,
                actual: config.schema_version,
            },
        );
    }

    sanitize_semantic_aliases(config.semantic_aliases)
}

#[cfg(test)]
mod tests {
    use super::load_semantic_aliases_json_file;
    use crate::GeneratePackError;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn semantic_alias_config_file_loads_v1_json() {
        let root = make_temp_dir("fret-icons-generator-semantic-alias-config");
        let config_file = root.join("semantic-aliases.json");
        std::fs::write(
            &config_file,
            r#"{
  "schema_version": 1,
  "semantic_aliases": [
    { "semantic_id": "ui.search", "target_icon": "search" }
  ]
}"#,
        )
        .expect("write semantic alias config");

        let aliases = load_semantic_aliases_json_file(&config_file)
            .expect("semantic alias config should load");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].semantic_id, "ui.search");
        assert_eq!(aliases[0].target_icon, "search");
    }

    #[test]
    fn semantic_alias_config_file_rejects_unsupported_schema_version() {
        let root = make_temp_dir("fret-icons-generator-semantic-alias-schema");
        let config_file = root.join("semantic-aliases.json");
        std::fs::write(
            &config_file,
            r#"{
  "schema_version": 2,
  "semantic_aliases": []
}"#,
        )
        .expect("write semantic alias config");

        let err = load_semantic_aliases_json_file(&config_file)
            .expect_err("unsupported schema version should fail");
        assert!(matches!(
            err,
            GeneratePackError::UnsupportedSemanticAliasConfigSchemaVersion { .. }
        ));
    }
}
