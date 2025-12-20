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
    selected_guid: Option<AssetGuid>,
    revision: u64,
}

impl ProjectSelectionService {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn selected_guid(&self) -> Option<AssetGuid> {
        self.selected_guid
    }

    pub fn set_selected_guid(&mut self, selected: Option<AssetGuid>) {
        if self.selected_guid == selected {
            return;
        }
        self.selected_guid = selected;
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
    id_by_guid: HashMap<AssetGuid, u64>,
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
            id_by_guid: HashMap::new(),
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

    pub fn guid_for_path(&self, path: &Path) -> Option<AssetGuid> {
        let id = self.id_by_path.get(path).copied()?;
        self.guid_for_id(id)
    }

    pub fn id_for_guid(&self, guid: AssetGuid) -> Option<u64> {
        self.id_by_guid.get(&guid).copied()
    }

    pub fn import_files(
        &mut self,
        sources: impl IntoIterator<Item = PathBuf>,
    ) -> io::Result<Vec<AssetGuid>> {
        let dest_dir = self.assets_root.join("Imports");
        std::fs::create_dir_all(&dest_dir)?;

        let mut imported: Vec<AssetGuid> = Vec::new();

        for src in sources {
            if src.is_dir() {
                warn!(path = %src.to_string_lossy(), "skipping directory import (not supported yet)");
                continue;
            }
            if !src.is_file() {
                warn!(path = %src.to_string_lossy(), "skipping non-file import");
                continue;
            }

            let Some(file_name) = src.file_name() else {
                warn!(path = %src.to_string_lossy(), "skipping import without file name");
                continue;
            };

            let dest_path = unique_dest_path(&dest_dir, file_name);
            std::fs::copy(&src, &dest_path)?;

            let meta_path = meta_path_for(&dest_path);
            let meta = read_or_create_meta(&meta_path)?;
            imported.push(AssetGuid(meta.guid));
        }

        Ok(imported)
    }

    pub fn move_guid_into_folder(
        &mut self,
        dragged: AssetGuid,
        dest_folder: AssetGuid,
    ) -> io::Result<()> {
        let Some(src_id) = self.id_for_guid(dragged) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "unknown dragged guid",
            ));
        };
        let Some(dest_id) = self.id_for_guid(dest_folder) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "unknown destination folder guid",
            ));
        };

        let Some(src_path) = self.path_for_id(src_id).map(|p| p.to_path_buf()) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "missing source path",
            ));
        };
        let Some(dest_path) = self.path_for_id(dest_id).map(|p| p.to_path_buf()) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "missing destination path",
            ));
        };

        if src_path == self.assets_root {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot move Assets root",
            ));
        }

        if self.kind_for_id(dest_id) != Some(ProjectEntryKind::Directory) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination is not a folder",
            ));
        }

        if src_path.is_dir() && (dest_path == src_path || dest_path.starts_with(&src_path)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot move a folder into itself",
            ));
        }

        let Some(file_name) = src_path.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source has no file name",
            ));
        };
        let to = unique_dest_path(&dest_path, file_name);
        if to == src_path {
            return Ok(());
        }

        move_path_and_meta(&src_path, &to)
    }

    pub fn rename_entry(&mut self, id: u64, new_file_name: &str) -> io::Result<()> {
        if new_file_name.contains(std::path::MAIN_SEPARATOR) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "new_file_name must be a single path segment",
            ));
        }

        let from = self
            .path_by_id
            .get(&id)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unknown entry id"))?;
        let parent = from
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "entry has no parent"))?;
        let to = parent.join(new_file_name);
        move_path_and_meta(&from, &to)
    }

    pub fn move_entry_to_folder(&mut self, id: u64, folder: &str) -> io::Result<()> {
        let from = self
            .path_by_id
            .get(&id)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unknown entry id"))?;
        let file_name = from
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "entry has no file name"))?;

        let dest_dir = self.assets_root.join(folder);
        std::fs::create_dir_all(&dest_dir)?;

        let to = dest_dir.join(file_name);
        move_path_and_meta(&from, &to)
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

        self.roots.clear();
        self.next_id = 1;
        self.id_by_path.clear();
        self.path_by_id.clear();
        self.kind_by_id.clear();
        self.guid_by_id.clear();
        self.id_by_guid.clear();

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
        let guid = AssetGuid(meta.guid);
        self.guid_by_id.insert(id, guid);
        self.id_by_guid.insert(guid, id);
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

fn unique_dest_path(dest_dir: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
    let candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ext = Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().to_string());

    for i in 1..=1000u32 {
        let name = match &ext {
            Some(ext) => format!("{stem}_{i}.{ext}"),
            None => format!("{stem}_{i}"),
        };
        let p = dest_dir.join(name);
        if !p.exists() {
            return p;
        }
    }

    dest_dir.join(format!("file-{}", Uuid::new_v4()))
}

fn move_path_and_meta(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination already exists",
        ));
    }

    let meta_from = meta_path_for(from);
    let meta_to = meta_path_for(to);

    let meta_exists = meta_from.exists();
    if meta_exists && meta_to.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination meta already exists",
        ));
    }

    std::fs::rename(from, to)?;

    if meta_exists {
        if let Err(err) = std::fs::rename(&meta_from, &meta_to) {
            let _ = std::fs::rename(to, from);
            return Err(err);
        }
    } else {
        let _ = read_or_create_meta(&meta_to)?;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> io::Result<Self> {
            let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
            std::fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn collect_ids(node: &TreeNode, out: &mut Vec<u64>) {
        out.push(node.id);
        for child in &node.children {
            collect_ids(child, out);
        }
    }

    fn find_id_by_path_ends_with(service: &ProjectService, suffix: &str) -> Option<u64> {
        let snapshot = service.snapshot();
        let mut ids: Vec<u64> = Vec::new();
        for root in &snapshot.roots {
            collect_ids(root, &mut ids);
        }
        for id in ids {
            let path = service.path_for_id(id)?;
            if path.to_string_lossy().ends_with(suffix) {
                return Some(id);
            }
        }
        None
    }

    #[test]
    fn guid_preserved_on_rename() -> io::Result<()> {
        let temp = TempDir::new("fret-project-rename")?;
        let assets_root = temp.path().join("Assets");
        std::fs::create_dir_all(&assets_root)?;

        let file = assets_root.join("A.txt");
        std::fs::write(&file, "hello")?;

        let mut service = ProjectService::new(assets_root.clone());
        service.rescan()?;

        let id = find_id_by_path_ends_with(&service, "A.txt").expect("A.txt exists");
        let guid = service.guid_for_id(id).expect("guid exists");

        let meta_path_before = meta_path_for(&file);
        let meta_before: AssetMetaV1 = serde_json::from_slice(&std::fs::read(&meta_path_before)?)?;
        assert_eq!(meta_before.guid, guid.0);

        service.rename_entry(id, "A_renamed.txt")?;
        service.rescan()?;

        let new_id = service
            .id_for_guid(guid)
            .expect("entry still present after rename");
        let new_path = service
            .path_for_id(new_id)
            .expect("path for renamed entry exists");
        assert!(new_path.to_string_lossy().ends_with("A_renamed.txt"));

        let meta_path_after = meta_path_for(new_path);
        let meta_after: AssetMetaV1 = serde_json::from_slice(&std::fs::read(&meta_path_after)?)?;
        assert_eq!(meta_after.guid, guid.0);

        Ok(())
    }

    #[test]
    fn guid_preserved_on_move() -> io::Result<()> {
        let temp = TempDir::new("fret-project-move")?;
        let assets_root = temp.path().join("Assets");
        std::fs::create_dir_all(&assets_root)?;

        let file = assets_root.join("B.txt");
        std::fs::write(&file, "hello")?;

        let mut service = ProjectService::new(assets_root.clone());
        service.rescan()?;

        let id = find_id_by_path_ends_with(&service, "B.txt").expect("B.txt exists");
        let guid = service.guid_for_id(id).expect("guid exists");

        service.move_entry_to_folder(id, "Moved")?;
        service.rescan()?;

        let new_id = service
            .id_for_guid(guid)
            .expect("entry still present after move");
        let new_path = service
            .path_for_id(new_id)
            .expect("path for moved entry exists");
        let new_path_str = new_path.to_string_lossy();
        assert!(new_path_str.contains("Moved"));
        assert!(new_path_str.ends_with("B.txt"));

        let meta_path_after = meta_path_for(new_path);
        let meta_after: AssetMetaV1 = serde_json::from_slice(&std::fs::read(&meta_path_after)?)?;
        assert_eq!(meta_after.guid, guid.0);

        Ok(())
    }

    #[test]
    fn import_creates_meta_and_returns_guid() -> io::Result<()> {
        let temp = TempDir::new("fret-project-import")?;
        let assets_root = temp.path().join("Assets");
        std::fs::create_dir_all(&assets_root)?;

        let source_dir = temp.path().join("Source");
        std::fs::create_dir_all(&source_dir)?;
        let src = source_dir.join("Imported.txt");
        std::fs::write(&src, "hello")?;

        let mut service = ProjectService::new(assets_root.clone());
        let imported = service.import_files([src.clone()])?;
        assert_eq!(imported.len(), 1);

        let dest = assets_root.join("Imports").join("Imported.txt");
        assert!(dest.exists());
        let meta = meta_path_for(&dest);
        assert!(meta.exists());

        let meta_value: AssetMetaV1 = serde_json::from_slice(&std::fs::read(&meta)?)?;
        assert_eq!(meta_value.guid, imported[0].0);

        Ok(())
    }

    #[test]
    fn move_by_guid_preserves_guid() -> io::Result<()> {
        let temp = TempDir::new("fret-project-move-guid")?;
        let assets_root = temp.path().join("Assets");
        std::fs::create_dir_all(&assets_root)?;
        std::fs::create_dir_all(assets_root.join("Dest"))?;

        let file = assets_root.join("C.txt");
        std::fs::write(&file, "hello")?;

        let mut service = ProjectService::new(assets_root.clone());
        service.rescan()?;

        let src_id = find_id_by_path_ends_with(&service, "C.txt").expect("C.txt exists");
        let src_guid = service.guid_for_id(src_id).expect("guid exists");

        let dest_id = find_id_by_path_ends_with(&service, "Dest").expect("Dest exists");
        let dest_guid = service.guid_for_id(dest_id).expect("dest guid exists");

        service.move_guid_into_folder(src_guid, dest_guid)?;
        service.rescan()?;

        let new_id = service.id_for_guid(src_guid).expect("still present");
        let new_path = service.path_for_id(new_id).expect("path exists");
        let new_path_str = new_path.to_string_lossy();
        assert!(new_path_str.contains("Dest"));
        assert!(new_path_str.ends_with("C.txt"));

        let meta_path_after = meta_path_for(new_path);
        let meta_after: AssetMetaV1 = serde_json::from_slice(&std::fs::read(&meta_path_after)?)?;
        assert_eq!(meta_after.guid, src_guid.0);

        Ok(())
    }
}
