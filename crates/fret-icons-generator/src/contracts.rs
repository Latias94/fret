use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratePackRequest {
    pub package_name: String,
    pub pack_id: String,
    pub vendor_namespace: String,
    pub output_dir: PathBuf,
    pub source: SourceSpec,
    pub dependency_spec: DependencySpec,
    pub generator_label: String,
    pub semantic_aliases: Vec<SemanticAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSpec {
    SvgDirectory(SvgDirectorySource),
    IconifyCollection(IconifyCollectionSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgDirectorySource {
    pub dir: PathBuf,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconifyCollectionSource {
    pub file: PathBuf,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySpec {
    Published {
        fret_version: String,
        rust_embed_version: String,
    },
    WorkspacePath {
        workspace_prefix: String,
        rust_embed_version: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticAlias {
    pub semantic_id: String,
    pub target_icon: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedPackReport {
    pub output_dir: PathBuf,
    pub package_name: String,
    pub pack_id: String,
    pub vendor_namespace: String,
    pub icon_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticAliasConfigFileV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub semantic_aliases: Vec<SemanticAlias>,
}
