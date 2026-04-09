use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeTargetKind {
    Bin,
    Example,
}

impl NativeTargetKind {
    pub(crate) fn cargo_flag(self) -> &'static str {
        match self {
            Self::Bin => "--bin",
            Self::Example => "--example",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Self::Bin => "binary target",
            Self::Example => "example target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WebTargetKind {
    Bin,
    Lib,
}

#[derive(Debug, Clone)]
pub(crate) struct SelectedNativeTarget {
    pub manifest_path: PathBuf,
    pub invocation_root: PathBuf,
    pub workspace_root: PathBuf,
    pub target_directory: PathBuf,
    pub package_name: String,
    pub needs_package_flag: bool,
    pub target_name: String,
    pub kind: NativeTargetKind,
}

#[derive(Debug, Clone)]
pub(crate) struct SelectedWebTarget {
    pub package_name: String,
    pub package_root: PathBuf,
    pub kind: WebTargetKind,
    pub target_name: Option<String>,
    pub index_html_path: PathBuf,
}

pub(crate) fn resolve_native_target(
    manifest_path: Option<&Path>,
    package_name: Option<&str>,
    bin_name: Option<&str>,
    example_name: Option<&str>,
) -> Result<SelectedNativeTarget, String> {
    let model = WorkspaceModel::load(manifest_path)?;
    model.select_native(package_name, bin_name, example_name)
}

pub(crate) fn resolve_web_target(
    manifest_path: Option<&Path>,
    package_name: Option<&str>,
    bin_name: Option<&str>,
) -> Result<SelectedWebTarget, String> {
    let model = WorkspaceModel::load(manifest_path)?;
    model.select_web(package_name, bin_name)
}

#[derive(Debug, Clone)]
struct WorkspaceModel {
    manifest_path: PathBuf,
    invocation_root: PathBuf,
    workspace_root: PathBuf,
    target_directory: PathBuf,
    packages: Vec<WorkspacePackage>,
}

#[derive(Debug, Clone)]
struct WorkspacePackage {
    name: String,
    manifest_path: PathBuf,
    default_run: Option<String>,
    bins: Vec<String>,
    examples: Vec<String>,
    lib_target: Option<String>,
}

impl WorkspacePackage {
    fn package_root(&self) -> Result<PathBuf, String> {
        self.manifest_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                format!(
                    "package `{}` manifest `{}` has no parent directory",
                    self.name,
                    self.manifest_path.display()
                )
            })
    }

    fn default_native_bin(&self) -> Option<&str> {
        if let Some(default_run) = self.default_run.as_deref()
            && self.bins.iter().any(|bin| bin == default_run)
        {
            return Some(default_run);
        }

        match self.bins.as_slice() {
            [only] => Some(only.as_str()),
            _ => None,
        }
    }

    fn default_web_target(&self) -> Option<ResolvedWebTarget<'_>> {
        if self.lib_target.is_some() {
            return Some(ResolvedWebTarget::Lib);
        }

        self.default_native_bin().map(ResolvedWebTarget::Bin)
    }
}

impl WorkspaceModel {
    fn load(manifest_path: Option<&Path>) -> Result<Self, String> {
        let manifest_path = resolve_manifest_path(manifest_path)?;
        let invocation_root = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                format!(
                    "manifest path `{}` has no parent directory",
                    manifest_path.display()
                )
            })?;

        let output = Command::new("cargo")
            .arg("metadata")
            .args(["--format-version", "1", "--no-deps", "--manifest-path"])
            .arg(&manifest_path)
            .output()
            .map_err(|err| format!("failed to run `cargo metadata`: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.trim();
            return Err(if detail.is_empty() {
                format!(
                    "`cargo metadata` failed for `{}` with status {}",
                    manifest_path.display(),
                    output.status
                )
            } else {
                format!(
                    "`cargo metadata` failed for `{}`: {}",
                    manifest_path.display(),
                    detail
                )
            });
        }

        let wire: CargoMetadataWire = serde_json::from_slice(&output.stdout)
            .map_err(|err| format!("failed to parse `cargo metadata` JSON: {err}"))?;
        let workspace_members: HashSet<&str> =
            wire.workspace_members.iter().map(String::as_str).collect();

        let mut packages: Vec<WorkspacePackage> = wire
            .packages
            .into_iter()
            .filter(|pkg| workspace_members.contains(pkg.id.as_str()))
            .map(WorkspacePackage::from_wire)
            .collect::<Result<Vec<_>, _>>()?;
        packages.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(Self {
            manifest_path,
            invocation_root,
            workspace_root: PathBuf::from(wire.workspace_root),
            target_directory: PathBuf::from(wire.target_directory),
            packages,
        })
    }

    fn select_native(
        &self,
        package_name: Option<&str>,
        bin_name: Option<&str>,
        example_name: Option<&str>,
    ) -> Result<SelectedNativeTarget, String> {
        if bin_name.is_some() && example_name.is_some() {
            return Err("cannot combine --bin and --example".to_string());
        }

        let package = match package_name {
            Some(name) => Some(self.find_package(name)?),
            None => self.manifest_package(),
        };

        match (package, bin_name, example_name) {
            (Some(package), Some(bin), None) => {
                self.select_native_from_package(package, NativeTargetKind::Bin, bin)
            }
            (Some(package), None, Some(example)) => {
                self.select_native_from_package(package, NativeTargetKind::Example, example)
            }
            (Some(package), None, None) => self.select_default_native_from_package(package),
            (None, Some(bin), None) => self.select_unique_native_target(NativeTargetKind::Bin, bin),
            (None, None, Some(example)) => {
                self.select_unique_native_target(NativeTargetKind::Example, example)
            }
            (None, None, None) => self.select_default_native_from_workspace(),
            _ => unreachable!("bin/example mutual exclusivity handled above"),
        }
    }

    fn select_web(
        &self,
        package_name: Option<&str>,
        bin_name: Option<&str>,
    ) -> Result<SelectedWebTarget, String> {
        let package = match package_name {
            Some(name) => Some(self.find_package(name)?),
            None => self.manifest_package(),
        };

        let selected = match (package, bin_name) {
            (Some(package), Some(bin)) => {
                if !package.bins.iter().any(|candidate| candidate == bin) {
                    return Err(format!(
                        "package `{}` does not define binary target `{}`\n  available binaries: {}",
                        package.name,
                        bin,
                        format_target_list(&package.bins)
                    ));
                }

                (package, WebTargetKind::Bin, Some(bin.to_string()))
            }
            (Some(package), None) => {
                let resolved = package.default_web_target().ok_or_else(|| {
                    format!(
                        "package `{}` does not expose a default web target\n  hint: add a lib target or rerun with --bin <name>",
                        package.name
                    )
                })?;
                self.select_web_from_default(package, resolved)
            }
            (None, Some(bin)) => {
                let matches: Vec<_> = self
                    .packages
                    .iter()
                    .filter(|package| package.bins.iter().any(|candidate| candidate == bin))
                    .collect();

                match matches.as_slice() {
                    [] => {
                        return Err(format!(
                            "binary target `{}` was not found in the selected workspace\n  hint: rerun with --package <name> if multiple packages exist",
                            bin
                        ));
                    }
                    [package] => (*package, WebTargetKind::Bin, Some(bin.to_string())),
                    _ => {
                        let packages = matches
                            .iter()
                            .map(|package| package.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(format!(
                            "binary target `{}` is defined in multiple packages: {}\n  hint: rerun with --package <name>",
                            bin, packages
                        ));
                    }
                }
            }
            (None, None) => {
                let defaults: Vec<_> = self
                    .packages
                    .iter()
                    .filter_map(|package| {
                        package.default_web_target().map(|target| (package, target))
                    })
                    .collect();

                match defaults.as_slice() {
                    [] => {
                        return Err(format!(
                            "no default web target could be inferred from `{}`\n  hint: rerun with --package <name> and, when needed, --bin <name>",
                            self.manifest_path.display()
                        ));
                    }
                    [(package, target)] => self.select_web_from_default(package, *target),
                    _ => {
                        let targets = defaults
                            .iter()
                            .map(|(package, target)| {
                                format!("{}::{}", package.name, target.label())
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(format!(
                            "manifest `{}` resolves to multiple web targets: {}\n  hint: rerun with --package <name> and, when needed, --bin <name>",
                            self.manifest_path.display(),
                            targets
                        ));
                    }
                }
            }
        };

        self.build_selected_web_target(selected.0, selected.1, selected.2)
    }

    fn select_native_from_package(
        &self,
        package: &WorkspacePackage,
        kind: NativeTargetKind,
        target_name: &str,
    ) -> Result<SelectedNativeTarget, String> {
        let targets = match kind {
            NativeTargetKind::Bin => &package.bins,
            NativeTargetKind::Example => &package.examples,
        };

        if !targets.iter().any(|candidate| candidate == target_name) {
            return Err(format!(
                "package `{}` does not define {} `{}`\n  available {}s: {}",
                package.name,
                kind.noun(),
                target_name,
                plural_label(kind),
                format_target_list(targets)
            ));
        }

        self.build_selected_native_target(package, kind, target_name.to_string())
    }

    fn select_default_native_from_package(
        &self,
        package: &WorkspacePackage,
    ) -> Result<SelectedNativeTarget, String> {
        if let Some(default_bin) = package.default_native_bin() {
            return self.build_selected_native_target(
                package,
                NativeTargetKind::Bin,
                default_bin.to_string(),
            );
        }

        if package.bins.is_empty() && !package.examples.is_empty() {
            return Err(format!(
                "package `{}` has no runnable binary target\n  available example targets: {}\n  hint: rerun with --example <name>",
                package.name,
                format_target_list(&package.examples)
            ));
        }

        if package.bins.is_empty() {
            return Err(format!(
                "package `{}` has no runnable binary target",
                package.name
            ));
        }

        Err(format!(
            "package `{}` has multiple binary targets: {}\n  hint: rerun with --bin <name>",
            package.name,
            format_target_list(&package.bins)
        ))
    }

    fn select_unique_native_target(
        &self,
        kind: NativeTargetKind,
        target_name: &str,
    ) -> Result<SelectedNativeTarget, String> {
        let matches: Vec<_> = self
            .packages
            .iter()
            .filter(|package| match kind {
                NativeTargetKind::Bin => package
                    .bins
                    .iter()
                    .any(|candidate| candidate == target_name),
                NativeTargetKind::Example => package
                    .examples
                    .iter()
                    .any(|candidate| candidate == target_name),
            })
            .collect();

        match matches.as_slice() {
            [] => Err(format!(
                "{} `{}` was not found in the selected workspace\n  hint: rerun with --package <name> if multiple packages exist",
                kind.noun(),
                target_name
            )),
            [package] => self.build_selected_native_target(package, kind, target_name.to_string()),
            _ => {
                let packages = matches
                    .iter()
                    .map(|package| package.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(format!(
                    "{} `{}` is defined in multiple packages: {}\n  hint: rerun with --package <name>",
                    kind.noun(),
                    target_name,
                    packages
                ))
            }
        }
    }

    fn select_default_native_from_workspace(&self) -> Result<SelectedNativeTarget, String> {
        let defaults: Vec<_> = self
            .packages
            .iter()
            .filter_map(|package| package.default_native_bin().map(|bin| (package, bin)))
            .collect();

        match defaults.as_slice() {
            [] => Err(format!(
                "no default runnable binary target could be inferred from `{}`\n  hint: rerun with --package <name> and --bin <name> (or --example <name>)",
                self.manifest_path.display()
            )),
            [(package, bin)] => self.build_selected_native_target(
                package,
                NativeTargetKind::Bin,
                (*bin).to_string(),
            ),
            _ => {
                let targets = defaults
                    .iter()
                    .map(|(package, bin)| format!("{}::{}", package.name, bin))
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(format!(
                    "manifest `{}` resolves to multiple runnable binary targets: {}\n  hint: rerun with --package <name> and/or --bin <name>",
                    self.manifest_path.display(),
                    targets
                ))
            }
        }
    }

    fn select_web_from_default<'a>(
        &self,
        package: &'a WorkspacePackage,
        target: ResolvedWebTarget<'a>,
    ) -> (&'a WorkspacePackage, WebTargetKind, Option<String>) {
        match target {
            ResolvedWebTarget::Lib => (package, WebTargetKind::Lib, package.lib_target.clone()),
            ResolvedWebTarget::Bin(name) => (package, WebTargetKind::Bin, Some(name.to_string())),
        }
    }

    fn build_selected_native_target(
        &self,
        package: &WorkspacePackage,
        kind: NativeTargetKind,
        target_name: String,
    ) -> Result<SelectedNativeTarget, String> {
        Ok(SelectedNativeTarget {
            manifest_path: self.manifest_path.clone(),
            invocation_root: self.invocation_root.clone(),
            workspace_root: self.workspace_root.clone(),
            target_directory: self.target_directory.clone(),
            package_name: package.name.clone(),
            needs_package_flag: self.manifest_path != package.manifest_path,
            target_name,
            kind,
        })
    }

    fn build_selected_web_target(
        &self,
        package: &WorkspacePackage,
        kind: WebTargetKind,
        target_name: Option<String>,
    ) -> Result<SelectedWebTarget, String> {
        let package_root = package.package_root()?;
        Ok(SelectedWebTarget {
            package_name: package.name.clone(),
            package_root: package_root.clone(),
            kind,
            target_name,
            index_html_path: package_root.join("index.html"),
        })
    }

    fn find_package(&self, package_name: &str) -> Result<&WorkspacePackage, String> {
        self.packages
            .iter()
            .find(|package| package.name == package_name)
            .ok_or_else(|| {
                let available = self
                    .packages
                    .iter()
                    .map(|package| package.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "package `{}` was not found in the selected workspace\n  available packages: {}",
                    package_name,
                    available
                )
            })
    }

    fn manifest_package(&self) -> Option<&WorkspacePackage> {
        self.packages
            .iter()
            .find(|package| package.manifest_path == self.manifest_path)
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolvedWebTarget<'a> {
    Bin(&'a str),
    Lib,
}

impl ResolvedWebTarget<'_> {
    fn label(self) -> String {
        match self {
            Self::Bin(name) => format!("bin:{name}"),
            Self::Lib => "lib".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CargoMetadataWire {
    workspace_root: String,
    target_directory: String,
    workspace_members: Vec<String>,
    packages: Vec<CargoPackageWire>,
}

#[derive(Debug, Deserialize)]
struct CargoPackageWire {
    id: String,
    name: String,
    manifest_path: String,
    default_run: Option<String>,
    targets: Vec<CargoTargetWire>,
}

#[derive(Debug, Deserialize)]
struct CargoTargetWire {
    name: String,
    kind: Vec<String>,
    crate_types: Vec<String>,
}

impl WorkspacePackage {
    fn from_wire(wire: CargoPackageWire) -> Result<Self, String> {
        let mut bins = Vec::new();
        let mut examples = Vec::new();
        let mut lib_target = None;

        for target in wire.targets {
            if target.kind.iter().any(|kind| kind == "bin") {
                bins.push(target.name.clone());
            }
            if target.kind.iter().any(|kind| kind == "example") {
                examples.push(target.name.clone());
            }
            let is_library_target = target.kind.iter().any(|kind| kind == "lib")
                || target
                    .crate_types
                    .iter()
                    .any(|crate_type| matches!(crate_type.as_str(), "cdylib" | "rlib"));
            if is_library_target && lib_target.is_none() {
                lib_target = Some(target.name.clone());
            }
        }

        bins.sort();
        examples.sort();

        Ok(Self {
            name: wire.name,
            manifest_path: normalize_existing_path(Path::new(&wire.manifest_path))?,
            default_run: wire.default_run,
            bins,
            examples,
            lib_target,
        })
    }
}

fn resolve_manifest_path(manifest_path: Option<&Path>) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir()
        .map_err(|err| format!("failed to read current directory: {err}"))?;
    let resolved = match manifest_path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd.join("Cargo.toml"),
    };

    if !resolved.is_file() {
        return Err(format!(
            "manifest path `{}` does not exist or is not a file",
            resolved.display()
        ));
    }

    normalize_existing_path(&resolved)
}

fn normalize_existing_path(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path)
        .map_err(|err| format!("failed to canonicalize `{}`: {err}", path.display()))
}

fn format_target_list(targets: &[String]) -> String {
    if targets.is_empty() {
        "<none>".to_string()
    } else {
        targets.join(", ")
    }
}

fn plural_label(kind: NativeTargetKind) -> &'static str {
    match kind {
        NativeTargetKind::Bin => "binary targets",
        NativeTargetKind::Example => "example targets",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        NativeTargetKind, ResolvedWebTarget, WebTargetKind, WorkspaceModel, WorkspacePackage,
    };

    fn model_with_packages(manifest_path: &str, packages: Vec<WorkspacePackage>) -> WorkspaceModel {
        WorkspaceModel {
            manifest_path: PathBuf::from(manifest_path),
            invocation_root: PathBuf::from("/tmp/project"),
            workspace_root: PathBuf::from("/tmp/project"),
            target_directory: PathBuf::from("/tmp/project/target"),
            packages,
        }
    }

    fn package(
        name: &str,
        manifest_path: &str,
        default_run: Option<&str>,
        bins: &[&str],
        examples: &[&str],
        lib_target: Option<&str>,
    ) -> WorkspacePackage {
        WorkspacePackage {
            name: name.to_string(),
            manifest_path: PathBuf::from(manifest_path),
            default_run: default_run.map(ToString::to_string),
            bins: bins.iter().map(|value| value.to_string()).collect(),
            examples: examples.iter().map(|value| value.to_string()).collect(),
            lib_target: lib_target.map(ToString::to_string),
        }
    }

    #[test]
    fn select_native_uses_package_default_run() {
        let model = model_with_packages(
            "/tmp/project/Cargo.toml",
            vec![package(
                "demo",
                "/tmp/project/Cargo.toml",
                Some("todo_demo"),
                &["todo_demo", "table_demo"],
                &[],
                None,
            )],
        );

        let selected = model
            .select_native(Some("demo"), None, None)
            .expect("default run should resolve");

        assert_eq!(selected.package_name, "demo");
        assert_eq!(selected.kind, NativeTargetKind::Bin);
        assert_eq!(selected.target_name, "todo_demo");
        assert!(!selected.needs_package_flag);
    }

    #[test]
    fn select_native_requires_package_when_bin_is_shared() {
        let model = model_with_packages(
            "/tmp/workspace/Cargo.toml",
            vec![
                package(
                    "alpha",
                    "/tmp/workspace/apps/alpha/Cargo.toml",
                    None,
                    &["app"],
                    &[],
                    None,
                ),
                package(
                    "beta",
                    "/tmp/workspace/apps/beta/Cargo.toml",
                    None,
                    &["app"],
                    &[],
                    None,
                ),
            ],
        );

        let err = model
            .select_native(None, Some("app"), None)
            .expect_err("shared bin should require package");
        assert!(err.contains("multiple packages"));
        assert!(err.contains("--package"));
    }

    #[test]
    fn select_native_defaults_to_manifest_package_when_not_at_workspace_root() {
        let model = model_with_packages(
            "/tmp/workspace/apps/demo/Cargo.toml",
            vec![
                package(
                    "demo",
                    "/tmp/workspace/apps/demo/Cargo.toml",
                    Some("todo_demo"),
                    &["todo_demo"],
                    &[],
                    None,
                ),
                package(
                    "other",
                    "/tmp/workspace/apps/other/Cargo.toml",
                    Some("other_demo"),
                    &["other_demo"],
                    &[],
                    None,
                ),
            ],
        );

        let selected = model
            .select_native(None, None, None)
            .expect("package manifest should default to its own package");
        assert_eq!(selected.package_name, "demo");
        assert_eq!(selected.target_name, "todo_demo");
        assert!(!selected.needs_package_flag);
    }

    #[test]
    fn select_native_requires_bin_when_package_has_multiple_bins() {
        let model = model_with_packages(
            "/tmp/project/Cargo.toml",
            vec![package(
                "demo",
                "/tmp/project/Cargo.toml",
                None,
                &["todo_demo", "table_demo"],
                &[],
                None,
            )],
        );

        let err = model
            .select_native(Some("demo"), None, None)
            .expect_err("multiple bins should be ambiguous");
        assert!(err.contains("multiple binary targets"));
        assert!(err.contains("--bin"));
    }

    #[test]
    fn select_native_requires_example_flag_for_example_only_package() {
        let model = model_with_packages(
            "/tmp/project/Cargo.toml",
            vec![package(
                "demo",
                "/tmp/project/Cargo.toml",
                None,
                &[],
                &["simple_todo"],
                None,
            )],
        );

        let err = model
            .select_native(Some("demo"), None, None)
            .expect_err("example-only package should need --example");
        assert!(err.contains("no runnable binary target"));
        assert!(err.contains("--example"));
    }

    #[test]
    fn select_web_prefers_lib_target() {
        let package = package(
            "web-app",
            "/tmp/workspace/apps/web-app/Cargo.toml",
            None,
            &["web_preview"],
            &[],
            Some("web_app"),
        );
        assert!(matches!(
            package.default_web_target(),
            Some(ResolvedWebTarget::Lib)
        ));
    }

    #[test]
    fn select_web_resolves_explicit_bin() {
        let model = model_with_packages(
            "/tmp/workspace/Cargo.toml",
            vec![package(
                "web-app",
                "/tmp/workspace/apps/web-app/Cargo.toml",
                None,
                &["preview"],
                &[],
                Some("web_app"),
            )],
        );

        let selected = model
            .select_web(Some("web-app"), Some("preview"))
            .expect("explicit web bin should resolve");
        assert_eq!(selected.package_name, "web-app");
        assert_eq!(selected.kind, WebTargetKind::Bin);
        assert_eq!(selected.target_name.as_deref(), Some("preview"));
        assert!(
            selected
                .index_html_path
                .ends_with("apps/web-app/index.html")
        );
    }

    #[test]
    fn select_web_defaults_to_manifest_package_when_not_at_workspace_root() {
        let model = model_with_packages(
            "/tmp/workspace/apps/web-app/Cargo.toml",
            vec![
                package(
                    "web-app",
                    "/tmp/workspace/apps/web-app/Cargo.toml",
                    None,
                    &[],
                    &[],
                    Some("web_app"),
                ),
                package(
                    "other-web",
                    "/tmp/workspace/apps/other-web/Cargo.toml",
                    None,
                    &[],
                    &[],
                    Some("other_web"),
                ),
            ],
        );

        let selected = model
            .select_web(None, None)
            .expect("package manifest should default to its own web package");
        assert_eq!(selected.package_name, "web-app");
        assert_eq!(selected.kind, WebTargetKind::Lib);
    }
}
