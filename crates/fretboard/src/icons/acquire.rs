use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::contracts::AcquireIconifyCollectionArgs;

const HTTP_USER_AGENT: &str = "fretboard-icons-acquire";
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcquireIconifyCollectionReport {
    pub snapshot_path: PathBuf,
    pub provenance_path: PathBuf,
    pub collection: String,
    pub icon_count: usize,
    pub alias_count: usize,
}

pub(crate) fn run_iconify_collection_acquire_contract(
    args: AcquireIconifyCollectionArgs,
) -> Result<AcquireIconifyCollectionReport, String> {
    run_iconify_collection_acquire_contract_with_client(args, &HttpIconifyApiClient::default())
}

fn run_iconify_collection_acquire_contract_with_client(
    args: AcquireIconifyCollectionArgs,
    client: &dyn IconifyApiClient,
) -> Result<AcquireIconifyCollectionReport, String> {
    let collection = sanitize_collection_prefix(&args.collection)?;
    let requested_icons = normalize_requested_icons(&args.icons)?;
    let api_base_url = sanitize_api_base_url(&args.api_base_url)?;
    let snapshot_path = args.out;
    let provenance_path = args
        .provenance_out
        .unwrap_or_else(|| default_provenance_path(&snapshot_path));

    if snapshot_path == provenance_path {
        return Err("snapshot output path and provenance output path must differ".to_string());
    }

    let collection_info = client.fetch_collection_info(&api_base_url, &collection)?;
    let mut snapshot = client.fetch_icon_subset(&api_base_url, &collection, &requested_icons)?;
    if snapshot.prefix.as_deref().is_none_or(str::is_empty) {
        snapshot.prefix = Some(collection.clone());
    }

    if snapshot.icons.is_empty() && snapshot.aliases.is_empty() {
        return Err(format!(
            "acquired Iconify snapshot for `{collection}` is empty; expected at least one icon or alias"
        ));
    }

    let snapshot_json = serde_json::to_string_pretty(&snapshot)
        .map_err(|err| format!("failed to serialize acquired snapshot: {err}"))?;
    write_text_file(&snapshot_path, &snapshot_json)?;

    let provenance = IconifyAcquisitionProvenanceV1 {
        schema_version: 1,
        acquisition_kind: "iconify-collection".to_string(),
        collection: collection.clone(),
        request: AcquisitionRequestRecord {
            mode: "subset".to_string(),
            requested_icons: requested_icons.clone(),
        },
        source: AcquisitionSourceRecord {
            api_base_url: api_base_url.clone(),
            collection_info_url: build_collection_info_url(&api_base_url, &collection),
            icons_url: build_icons_url(&api_base_url, &collection, &requested_icons),
        },
        upstream: UpstreamMetadataRecord {
            title: collection_info.title,
            total: collection_info.total,
            collection_info: collection_info.info,
        },
        snapshot: SnapshotRecord {
            digest_algorithm: "blake3".to_string(),
            digest_hex: blake3::hash(snapshot_json.as_bytes()).to_hex().to_string(),
            icon_count: snapshot.icons.len(),
            alias_count: snapshot.aliases.len(),
        },
    };
    let provenance_json = serde_json::to_string_pretty(&provenance)
        .map_err(|err| format!("failed to serialize acquisition provenance: {err}"))?;
    write_text_file(&provenance_path, &provenance_json)?;

    Ok(AcquireIconifyCollectionReport {
        snapshot_path,
        provenance_path,
        collection,
        icon_count: snapshot.icons.len(),
        alias_count: snapshot.aliases.len(),
    })
}

fn sanitize_collection_prefix(prefix: &str) -> Result<String, String> {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return Err("collection prefix must not be empty".to_string());
    }
    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        Ok(trimmed.to_string())
    } else {
        Err(format!(
            "collection prefix `{trimmed}` must use lowercase ascii letters, digits, or `-`"
        ))
    }
}

fn normalize_requested_icons(icons: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = BTreeSet::new();
    for icon in icons {
        let trimmed = icon.trim();
        if trimmed.is_empty() {
            return Err("icon names must not be empty".to_string());
        }
        normalized.insert(trimmed.to_string());
    }

    if normalized.is_empty() {
        return Err("at least one `--icon <NAME>` is required for acquisition proof".to_string());
    }

    Ok(normalized.into_iter().collect())
}

fn sanitize_api_base_url(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        Ok(trimmed.to_string())
    } else {
        Err(format!(
            "api base url `{trimmed}` must start with http:// or https://"
        ))
    }
}

fn default_provenance_path(snapshot_path: &Path) -> PathBuf {
    let parent = snapshot_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = snapshot_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("iconify-collection");
    parent.join(format!("{stem}.provenance.json"))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create parent directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    Ok(())
}

fn write_text_file(path: &Path, contents: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    std::fs::write(path, contents)
        .map_err(|err| format!("failed to write `{}`: {err}", path.display()))
}

fn build_collection_info_url(api_base_url: &str, collection: &str) -> String {
    format!("{api_base_url}/collection?prefix={collection}&info=true")
}

fn build_icons_url(api_base_url: &str, collection: &str, icons: &[String]) -> String {
    format!("{api_base_url}/{collection}.json?icons={}", icons.join(","))
}

trait IconifyApiClient {
    fn fetch_collection_info(
        &self,
        api_base_url: &str,
        collection: &str,
    ) -> Result<IconifyCollectionInfoResponse, String>;

    fn fetch_icon_subset(
        &self,
        api_base_url: &str,
        collection: &str,
        icons: &[String],
    ) -> Result<IconifyCollectionSnapshot, String>;
}

struct HttpIconifyApiClient {
    agent: ureq::Agent,
}

impl Default for HttpIconifyApiClient {
    fn default() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(20))
            .timeout_write(Duration::from_secs(20))
            .build();
        Self { agent }
    }
}

impl HttpIconifyApiClient {
    fn read_json<T: for<'de> Deserialize<'de>>(
        &self,
        request: ureq::Request,
        context: &str,
    ) -> Result<T, String> {
        let response = request
            .set("User-Agent", HTTP_USER_AGENT)
            .set("Accept", "application/json")
            .call()
            .map_err(|err| match err {
                ureq::Error::Status(status, response) => {
                    format!(
                        "{context} failed with HTTP {status} {}",
                        response.status_text()
                    )
                }
                ureq::Error::Transport(error) => format!("{context} request failed: {error}"),
            })?;

        let mut reader = response.into_reader();
        let mut bytes = Vec::new();
        let mut buf = [0_u8; 16 * 1024];
        loop {
            let read = reader
                .read(&mut buf)
                .map_err(|err| format!("{context} read failed: {err}"))?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..read]);
            if bytes.len() > MAX_RESPONSE_BYTES {
                return Err(format!(
                    "{context} response exceeded {} MiB",
                    MAX_RESPONSE_BYTES / (1024 * 1024)
                ));
            }
        }

        serde_json::from_slice(&bytes).map_err(|err| format!("{context} JSON decode failed: {err}"))
    }
}

impl IconifyApiClient for HttpIconifyApiClient {
    fn fetch_collection_info(
        &self,
        api_base_url: &str,
        collection: &str,
    ) -> Result<IconifyCollectionInfoResponse, String> {
        let request = self
            .agent
            .get(&format!("{api_base_url}/collection"))
            .query("prefix", collection)
            .query("info", "true");
        self.read_json(request, "collection metadata fetch")
    }

    fn fetch_icon_subset(
        &self,
        api_base_url: &str,
        collection: &str,
        icons: &[String],
    ) -> Result<IconifyCollectionSnapshot, String> {
        let request = self
            .agent
            .get(&format!("{api_base_url}/{collection}.json"))
            .query("icons", &icons.join(","));
        self.read_json(request, "icon subset fetch")
    }
}

#[derive(Debug, Clone, Serialize)]
struct IconifyAcquisitionProvenanceV1 {
    schema_version: u32,
    acquisition_kind: String,
    collection: String,
    request: AcquisitionRequestRecord,
    source: AcquisitionSourceRecord,
    upstream: UpstreamMetadataRecord,
    snapshot: SnapshotRecord,
}

#[derive(Debug, Clone, Serialize)]
struct AcquisitionRequestRecord {
    mode: String,
    requested_icons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AcquisitionSourceRecord {
    api_base_url: String,
    collection_info_url: String,
    icons_url: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpstreamMetadataRecord {
    title: Option<String>,
    total: Option<u32>,
    collection_info: Option<IconifyCollectionInfo>,
}

#[derive(Debug, Clone, Serialize)]
struct SnapshotRecord {
    digest_algorithm: String,
    digest_hex: String,
    icon_count: usize,
    alias_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct IconifyCollectionInfoResponse {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    total: Option<u32>,
    #[serde(default)]
    info: Option<IconifyCollectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IconifyCollectionInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    author: Option<IconifyAuthor>,
    #[serde(default)]
    license: Option<IconifyLicense>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    palette: Option<bool>,
    #[serde(default)]
    total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum IconifyAuthor {
    Simple(String),
    Detailed {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        url: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum IconifyLicense {
    Simple(String),
    Detailed {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        spdx: Option<String>,
        #[serde(default)]
        url: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IconifyCollectionSnapshot {
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    icons: BTreeMap<String, IconifyIcon>,
    #[serde(default)]
    aliases: BTreeMap<String, IconifyAlias>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IconifyIcon {
    body: String,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    rotate: Option<u8>,
    #[serde(default, rename = "hFlip")]
    h_flip: Option<bool>,
    #[serde(default, rename = "vFlip")]
    v_flip: Option<bool>,
    #[serde(default)]
    hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IconifyAlias {
    parent: String,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
    #[serde(default)]
    left: Option<f64>,
    #[serde(default)]
    top: Option<f64>,
    #[serde(default)]
    rotate: Option<u8>,
    #[serde(default, rename = "hFlip")]
    h_flip: Option<bool>,
    #[serde(default, rename = "vFlip")]
    v_flip: Option<bool>,
    #[serde(default)]
    hidden: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::contracts::{
        IconImportCommandArgs, IconImportSourceContract, IconsCommandArgs, IconsCommandContract,
        ImportCommonArgs, ImportIconifyCollectionArgs,
    };
    use crate::icons::run_repo_icons_contract;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone)]
    struct FixtureIconifyApiClient {
        collection_info: IconifyCollectionInfoResponse,
        snapshot: IconifyCollectionSnapshot,
    }

    impl IconifyApiClient for FixtureIconifyApiClient {
        fn fetch_collection_info(
            &self,
            _api_base_url: &str,
            _collection: &str,
        ) -> Result<IconifyCollectionInfoResponse, String> {
            Ok(self.collection_info.clone())
        }

        fn fetch_icon_subset(
            &self,
            _api_base_url: &str,
            _collection: &str,
            _icons: &[String],
        ) -> Result<IconifyCollectionSnapshot, String> {
            Ok(self.snapshot.clone())
        }
    }

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn read_json(path: &Path) -> serde_json::Value {
        let text = std::fs::read_to_string(path).expect("read json file");
        serde_json::from_str(&text).expect("parse json")
    }

    fn repo_workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo workspace root should resolve")
    }

    fn make_repo_local_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = repo_workspace_root()
            .join("local")
            .join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create repo-local dir");
        dir
    }

    #[test]
    fn acquire_iconify_collection_writes_subset_snapshot_and_provenance() {
        let root = make_temp_dir("fretboard-iconify-acquire");
        let snapshot_path = root.join("mdi-home.json");
        let client = FixtureIconifyApiClient {
            collection_info: IconifyCollectionInfoResponse {
                title: Some("Material Design Icons".to_string()),
                total: Some(1234),
                info: Some(IconifyCollectionInfo {
                    name: Some("Material Design Icons".to_string()),
                    author: Some(IconifyAuthor::Detailed {
                        name: Some("Pictogrammers".to_string()),
                        url: Some("https://pictogrammers.com".to_string()),
                    }),
                    license: Some(IconifyLicense::Detailed {
                        title: Some("Apache License 2.0".to_string()),
                        spdx: Some("Apache-2.0".to_string()),
                        url: Some("https://www.apache.org/licenses/LICENSE-2.0".to_string()),
                    }),
                    height: Some(24),
                    category: Some("general".to_string()),
                    palette: Some(false),
                    total: Some(1234),
                }),
            },
            snapshot: IconifyCollectionSnapshot {
                prefix: Some("mdi".to_string()),
                width: Some(24.0),
                height: Some(24.0),
                left: None,
                top: None,
                icons: BTreeMap::from([
                    (
                        "account".to_string(),
                        IconifyIcon {
                            body: "<path d='M2 2h20v20H2z'/>".to_string(),
                            width: None,
                            height: None,
                            left: None,
                            top: None,
                            rotate: None,
                            h_flip: None,
                            v_flip: None,
                            hidden: None,
                        },
                    ),
                    (
                        "home".to_string(),
                        IconifyIcon {
                            body: "<path d='M3 3h18v18H3z'/>".to_string(),
                            width: None,
                            height: None,
                            left: None,
                            top: None,
                            rotate: None,
                            h_flip: None,
                            v_flip: None,
                            hidden: None,
                        },
                    ),
                ]),
                aliases: BTreeMap::from([(
                    "house".to_string(),
                    IconifyAlias {
                        parent: "home".to_string(),
                        width: None,
                        height: None,
                        left: None,
                        top: None,
                        rotate: None,
                        h_flip: None,
                        v_flip: None,
                        hidden: None,
                    },
                )]),
            },
        };

        let report = run_iconify_collection_acquire_contract_with_client(
            AcquireIconifyCollectionArgs {
                collection: "mdi".to_string(),
                icons: vec![
                    "home".to_string(),
                    "account".to_string(),
                    "home".to_string(),
                ],
                out: snapshot_path.clone(),
                provenance_out: None,
                api_base_url: "https://api.iconify.design".to_string(),
            },
            &client,
        )
        .expect("acquisition should succeed");

        assert_eq!(report.collection, "mdi");
        assert_eq!(report.icon_count, 2);
        assert_eq!(report.alias_count, 1);
        assert_eq!(report.snapshot_path, snapshot_path);
        assert_eq!(
            report.provenance_path,
            root.join("mdi-home.provenance.json")
        );

        let snapshot_json = read_json(&report.snapshot_path);
        assert_eq!(snapshot_json["prefix"], "mdi");
        assert_eq!(snapshot_json["icons"].as_object().map(|m| m.len()), Some(2));
        assert_eq!(
            snapshot_json["aliases"].as_object().map(|m| m.len()),
            Some(1)
        );

        let provenance_json = read_json(&report.provenance_path);
        assert_eq!(provenance_json["schema_version"], 1);
        assert_eq!(provenance_json["acquisition_kind"], "iconify-collection");
        assert_eq!(provenance_json["collection"], "mdi");
        assert_eq!(provenance_json["request"]["mode"], "subset");
        assert_eq!(
            provenance_json["request"]["requested_icons"],
            serde_json::json!(["account", "home"])
        );
        assert_eq!(provenance_json["snapshot"]["icon_count"], 2);
        assert_eq!(provenance_json["snapshot"]["alias_count"], 1);
        assert_eq!(provenance_json["snapshot"]["digest_algorithm"], "blake3");
        assert_eq!(
            provenance_json["upstream"]["title"],
            "Material Design Icons"
        );
        assert_eq!(
            provenance_json["upstream"]["collection_info"]["palette"],
            false
        );
    }

    #[test]
    fn acquire_iconify_collection_rejects_empty_icon_list() {
        let root = make_temp_dir("fretboard-iconify-acquire-empty");
        let client = FixtureIconifyApiClient {
            collection_info: IconifyCollectionInfoResponse {
                title: None,
                total: None,
                info: None,
            },
            snapshot: IconifyCollectionSnapshot {
                prefix: Some("mdi".to_string()),
                width: Some(24.0),
                height: Some(24.0),
                left: None,
                top: None,
                icons: BTreeMap::new(),
                aliases: BTreeMap::new(),
            },
        };

        let err = run_iconify_collection_acquire_contract_with_client(
            AcquireIconifyCollectionArgs {
                collection: "mdi".to_string(),
                icons: Vec::new(),
                out: root.join("mdi.json"),
                provenance_out: None,
                api_base_url: "https://api.iconify.design".to_string(),
            },
            &client,
        )
        .expect_err("empty icon list should fail");

        assert!(err.contains("at least one `--icon <NAME>` is required"));
    }

    #[test]
    fn acquire_iconify_collection_rejects_same_snapshot_and_provenance_path() {
        let root = make_temp_dir("fretboard-iconify-acquire-paths");
        let out = root.join("mdi.json");
        let client = FixtureIconifyApiClient {
            collection_info: IconifyCollectionInfoResponse {
                title: None,
                total: None,
                info: None,
            },
            snapshot: IconifyCollectionSnapshot {
                prefix: Some("mdi".to_string()),
                width: Some(24.0),
                height: Some(24.0),
                left: None,
                top: None,
                icons: BTreeMap::from([(
                    "home".to_string(),
                    IconifyIcon {
                        body: "<path d='M0 0h24v24H0z'/>".to_string(),
                        width: None,
                        height: None,
                        left: None,
                        top: None,
                        rotate: None,
                        h_flip: None,
                        v_flip: None,
                        hidden: None,
                    },
                )]),
                aliases: BTreeMap::new(),
            },
        };

        let err = run_iconify_collection_acquire_contract_with_client(
            AcquireIconifyCollectionArgs {
                collection: "mdi".to_string(),
                icons: vec!["home".to_string()],
                out: out.clone(),
                provenance_out: Some(out),
                api_base_url: "https://api.iconify.design".to_string(),
            },
            &client,
        )
        .expect_err("same output path should fail");

        assert!(err.contains("must differ"));
    }

    #[test]
    fn acquired_snapshot_flows_into_existing_repo_import_path() {
        let root = make_temp_dir("fretboard-iconify-acquire-import");
        let snapshot_path = root.join("mdi-home.json");
        let output_dir = make_repo_local_dir("fretboard-iconify-acquire-pack");
        let client = FixtureIconifyApiClient {
            collection_info: IconifyCollectionInfoResponse {
                title: Some("Material Design Icons".to_string()),
                total: Some(1234),
                info: Some(IconifyCollectionInfo {
                    name: Some("Material Design Icons".to_string()),
                    author: None,
                    license: Some(IconifyLicense::Simple("Apache-2.0".to_string())),
                    height: Some(24),
                    category: None,
                    palette: Some(false),
                    total: Some(1234),
                }),
            },
            snapshot: IconifyCollectionSnapshot {
                prefix: Some("mdi".to_string()),
                width: Some(24.0),
                height: Some(24.0),
                left: None,
                top: None,
                icons: BTreeMap::from([(
                    "home".to_string(),
                    IconifyIcon {
                        body: "<path d='M3 3h18v18H3z'/>".to_string(),
                        width: None,
                        height: None,
                        left: None,
                        top: None,
                        rotate: None,
                        h_flip: None,
                        v_flip: None,
                        hidden: None,
                    },
                )]),
                aliases: BTreeMap::new(),
            },
        };

        let report = run_iconify_collection_acquire_contract_with_client(
            AcquireIconifyCollectionArgs {
                collection: "mdi".to_string(),
                icons: vec!["home".to_string()],
                out: snapshot_path.clone(),
                provenance_out: None,
                api_base_url: "https://api.iconify.design".to_string(),
            },
            &client,
        )
        .expect("acquisition should succeed");

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::IconifyCollection(
                        ImportIconifyCollectionArgs {
                            source: report.snapshot_path.clone(),
                            common: ImportCommonArgs {
                                crate_name: "mdi-acquired-pack".to_string(),
                                vendor_namespace: "mdi".to_string(),
                                pack_id: None,
                                path: Some(output_dir.clone()),
                                source_label: Some("mdi-acquired-subset".to_string()),
                                semantic_aliases: None,
                                no_check: false,
                            },
                        },
                    ),
                }),
            },
            &repo_workspace_root(),
        )
        .expect("existing import path should accept acquired snapshot");

        assert!(output_dir.join("Cargo.toml").exists());
        let pack_provenance = read_json(&output_dir.join("pack-provenance.json"));
        assert_eq!(pack_provenance["source"]["kind"], "iconify-collection");
        assert_eq!(pack_provenance["source"]["label"], "mdi-acquired-subset");
    }
}
