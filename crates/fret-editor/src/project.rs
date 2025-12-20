use fret_ui::TreeNode;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};
use tracing::warn;
use uuid::Uuid;

pub const PROJECT_ROOT: &str = ".fret/project";
pub const PROJECT_ASSETS_DIR: &str = ".fret/project/Assets";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetGuid(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetaV1 {
    pub version: u32,
    pub guid: Uuid,
}

impl AssetMetaV1 {
    pub fn new() -> Self {
        Self {
            version: 1,
            guid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectEntryKind {
    Directory,
    File,
}

#[derive(Debug, Clone)]
pub struct ProjectTreeSnapshot {
    pub roots: Vec<TreeNode>,
    pub revision: u64,
}

#[derive(Debug, Default)]
pub struct ProjectSelectionService {
    selected: Option<u64>,
    revision: u64,
}

impl ProjectSelectionService {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }

    pub fn set_selected(&mut self, selected: Option<u64>) {
        if self.selected == selected {
            return;
        }
        self.selected = selected;
        self.revision = self.revision.saturating_add(1);
    }
}

#[derive(Debug)]
pub struct ProjectService {
    assets_root: PathBuf,
    roots: Vec<TreeNode>,
    revision: u64,

    next_id: u64,
    id_by_path: HashMap<PathBuf, u64>,
    path_by_id: HashMap<u64, PathBuf>,
    kind_by_id: HashMap<u64, ProjectEntryKind>,
    guid_by_id: HashMap<u64, AssetGuid>,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new(PathBuf::from(PROJECT_ASSETS_DIR))
    }
}

impl ProjectService {
    pub fn new(assets_root: PathBuf) -> Self {
        Self {
            assets_root,
            roots: Vec::new(),
            revision: 0,
            next_id: 1,
            id_by_path: HashMap::new(),
            path_by_id: HashMap::new(),
            kind_by_id: HashMap::new(),
            guid_by_id: HashMap::new(),
        }
    }

    pub fn assets_root(&self) -> &Path {
        &self.assets_root
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn snapshot(&self) -> ProjectTreeSnapshot {
        ProjectTreeSnapshot {
            roots: self.roots.clone(),
            revision: self.revision,
        }
    }

    pub fn path_for_id(&self, id: u64) -> Option<&Path> {
        self.path_by_id.get(&id).map(|p| p.as_path())
    }

    pub fn kind_for_id(&self, id: u64) -> Option<ProjectEntryKind> {
        self.kind_by_id.get(&id).copied()
    }

    pub fn guid_for_id(&self, id: u64) -> Option<AssetGuid> {
        self.guid_by_id.get(&id).copied()
    }

    pub fn ensure_demo_assets_exist(&self) -> io::Result<()> {
        let root = &self.assets_root;
        std::fs::create_dir_all(root)?;

        let scenes = root.join("Scenes");
        let materials = root.join("Materials");
        let textures = root.join("Textures");
        std::fs::create_dir_all(&scenes)?;
        std::fs::create_dir_all(&materials)?;
        std::fs::create_dir_all(&textures)?;

        let scene_file = scenes.join("Main.scene");
        if !scene_file.exists() {
            std::fs::write(
                &scene_file,
                "Demo scene placeholder.\nThis file exists to validate .meta GUID behavior.\n",
            )?;
        }

        let mat_file = materials.join("Default.mat");
        if !mat_file.exists() {
            std::fs::write(&mat_file, "Demo material placeholder.\nformat: text\n")?;
        }

        let tex_file = textures.join("Checker.txt");
        if !tex_file.exists() {
            std::fs::write(
                &tex_file,
                "Demo texture placeholder.\n(Replace with a real image later.)\n",
            )?;
        }

        Ok(())
    }

    pub fn rescan(&mut self) -> io::Result<()> {
        let assets_root = self.assets_root.clone();

        if !assets_root.exists() {
            std::fs::create_dir_all(&assets_root)?;
        }

        let assets_id = self.id_for_path(&assets_root);
        self.kind_by_id
            .insert(assets_id, ProjectEntryKind::Directory);
        self.ensure_meta_for_path(assets_id, &assets_root)?;

        let children = self.scan_dir(&assets_root)?;
        self.roots = vec![TreeNode::new(assets_id, "Assets").with_children(children)];
        self.revision = self.revision.saturating_add(1);
        Ok(())
    }

    fn scan_dir(&mut self, dir: &Path) -> io::Result<Vec<TreeNode>> {
        let mut entries: Vec<(PathBuf, ProjectEntryKind)> = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if is_meta_file(&path) {
                continue;
            }
            let kind = if path.is_dir() {
                ProjectEntryKind::Directory
            } else {
                ProjectEntryKind::File
            };
            entries.push((path, kind));
        }

        entries.sort_by(|(a, ak), (b, bk)| {
            // Directories first, then files; then case-insensitive name sort.
            ak.cmp(bk)
                .then_with(|| file_name_lower(a).cmp(&file_name_lower(b)))
        });

        let mut out: Vec<TreeNode> = Vec::new();
        for (path, kind) in entries {
            let id = self.id_for_path(&path);
            self.kind_by_id.insert(id, kind);
            self.ensure_meta_for_path(id, &path)?;

            let label = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let node = match kind {
                ProjectEntryKind::File => TreeNode::new(id, label),
                ProjectEntryKind::Directory => {
                    let children = self.scan_dir(&path)?;
                    TreeNode::new(id, label).with_children(children)
                }
            };

            out.push(node);
        }

        Ok(out)
    }

    fn id_for_path(&mut self, path: &Path) -> u64 {
        if let Some(&id) = self.id_by_path.get(path) {
            return id;
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let path = path.to_path_buf();
        self.id_by_path.insert(path.clone(), id);
        self.path_by_id.insert(id, path);
        id
    }

    fn ensure_meta_for_path(&mut self, id: u64, path: &Path) -> io::Result<()> {
        let meta_path = meta_path_for(path);
        let meta = read_or_create_meta(&meta_path)?;
        self.guid_by_id.insert(id, AssetGuid(meta.guid));
        Ok(())
    }
}

fn file_name_lower(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| path.to_string_lossy().to_ascii_lowercase())
}

fn is_meta_file(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().ends_with(".meta"))
}

fn meta_path_for(path: &Path) -> PathBuf {
    let mut os: OsString = path.as_os_str().to_os_string();
    os.push(".meta");
    PathBuf::from(os)
}

fn read_or_create_meta(path: &Path) -> io::Result<AssetMetaV1> {
    if let Ok(bytes) = std::fs::read(path) {
        match serde_json::from_slice::<AssetMetaV1>(&bytes) {
            Ok(meta) if meta.version == 1 => return Ok(meta),
            Ok(meta) => {
                warn!(
                    meta_version = meta.version,
                    meta_path = %path.to_string_lossy(),
                    "unknown asset meta version; rewriting"
                );
            }
            Err(err) => {
                warn!(
                    error = %err,
                    meta_path = %path.to_string_lossy(),
                    "failed to parse asset meta; rewriting"
                );
            }
        }
    }

    let meta = AssetMetaV1::new();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(&meta)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)?;
    Ok(meta)
}
