use std::collections::BTreeSet;
use std::path::Path;

use crate::GeneratePackError;
use crate::contracts::{
    PresentationDefaults, PresentationDefaultsConfigFileV1, PresentationOverride,
};

const PRESENTATION_DEFAULTS_CONFIG_SCHEMA_V1: u32 = 1;

pub(crate) fn sanitize_presentation_defaults(
    defaults: PresentationDefaults,
) -> Result<PresentationDefaults, GeneratePackError> {
    let mut seen = BTreeSet::new();
    let mut sanitized = Vec::with_capacity(defaults.icon_overrides.len());

    for override_entry in defaults.icon_overrides {
        let icon_name = override_entry.icon_name.trim().to_string();
        if icon_name.is_empty() {
            return Err(GeneratePackError::EmptyPresentationOverrideIconName);
        }
        if !seen.insert(icon_name.clone()) {
            return Err(GeneratePackError::DuplicatePresentationOverride { icon_name });
        }
        sanitized.push(PresentationOverride {
            icon_name,
            render_mode: override_entry.render_mode,
        });
    }

    Ok(PresentationDefaults {
        default_render_mode: defaults.default_render_mode,
        icon_overrides: sanitized,
    })
}

pub fn load_presentation_defaults_json_file(
    path: &Path,
) -> Result<PresentationDefaults, GeneratePackError> {
    if !path.exists() {
        return Err(GeneratePackError::MissingPresentationDefaultsConfigFile(
            path.display().to_string(),
        ));
    }
    if !path.is_file() {
        return Err(GeneratePackError::PresentationDefaultsConfigPathNotFile(
            path.display().to_string(),
        ));
    }

    let content = std::fs::read_to_string(path)?;
    let config: PresentationDefaultsConfigFileV1 = serde_json::from_str(&content)?;
    if config.schema_version != PRESENTATION_DEFAULTS_CONFIG_SCHEMA_V1 {
        return Err(
            GeneratePackError::UnsupportedPresentationDefaultsConfigSchemaVersion {
                expected: PRESENTATION_DEFAULTS_CONFIG_SCHEMA_V1,
                actual: config.schema_version,
            },
        );
    }

    sanitize_presentation_defaults(PresentationDefaults {
        default_render_mode: config.default_render_mode,
        icon_overrides: config.icon_overrides,
    })
}

#[cfg(test)]
mod tests {
    use super::load_presentation_defaults_json_file;
    use crate::GeneratePackError;
    use crate::contracts::PresentationRenderMode;
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
    fn presentation_defaults_config_file_loads_v1_json() {
        let root = make_temp_dir("fret-icons-generator-presentation-defaults");
        let config_file = root.join("presentation-defaults.json");
        std::fs::write(
            &config_file,
            r#"{
  "schema_version": 1,
  "default_render_mode": "mask",
  "icon_overrides": [
    { "icon_name": "brand-logo", "render_mode": "original-colors" }
  ]
}"#,
        )
        .expect("write presentation defaults config");

        let defaults = load_presentation_defaults_json_file(&config_file)
            .expect("presentation defaults config should load");
        assert_eq!(
            defaults.default_render_mode,
            Some(PresentationRenderMode::Mask)
        );
        assert_eq!(defaults.icon_overrides.len(), 1);
        assert_eq!(defaults.icon_overrides[0].icon_name, "brand-logo");
        assert_eq!(
            defaults.icon_overrides[0].render_mode,
            PresentationRenderMode::OriginalColors
        );
    }

    #[test]
    fn presentation_defaults_config_file_rejects_unsupported_schema_version() {
        let root = make_temp_dir("fret-icons-generator-presentation-schema");
        let config_file = root.join("presentation-defaults.json");
        std::fs::write(
            &config_file,
            r#"{
  "schema_version": 2,
  "icon_overrides": []
}"#,
        )
        .expect("write presentation defaults config");

        let err = load_presentation_defaults_json_file(&config_file)
            .expect_err("unsupported schema version should fail");
        assert!(matches!(
            err,
            GeneratePackError::UnsupportedPresentationDefaultsConfigSchemaVersion { .. }
        ));
    }
}
