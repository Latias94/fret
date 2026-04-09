//! Renderer-agnostic icon identity and registry for Fret component libraries.
//!
//! This crate is intentionally rendering-agnostic:
//! - Components depend on semantic icon IDs (`IconId`).
//! - Icon packs register icon definitions (source + fallback + presentation).
//! - Rendering (SVG raster caching, budgets, atlases) remains in the renderer layer.

use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const MAX_FREEZE_WARNING_LOGS: usize = 8;
static FREEZE_WARNING_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconId(Cow<'static, str>);

impl IconId {
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self(key.into())
    }

    pub const fn new_static(key: &'static str) -> Self {
        Self(Cow::Borrowed(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub mod ids {
    use super::IconId;

    pub mod ui {
        use super::IconId;

        pub const ALERT_TRIANGLE: IconId = IconId::new_static("ui.alert.triangle");
        pub const ARROW_LEFT: IconId = IconId::new_static("ui.arrow.left");
        pub const ARROW_RIGHT: IconId = IconId::new_static("ui.arrow.right");
        pub const BOOK: IconId = IconId::new_static("ui.book");
        pub const CHECK: IconId = IconId::new_static("ui.check");
        pub const COPY: IconId = IconId::new_static("ui.copy");
        pub const CHEVRON_LEFT: IconId = IconId::new_static("ui.chevron.left");
        pub const CHEVRON_DOWN: IconId = IconId::new_static("ui.chevron.down");
        pub const CHEVRON_RIGHT: IconId = IconId::new_static("ui.chevron.right");
        pub const CHEVRON_UP: IconId = IconId::new_static("ui.chevron.up");
        pub const CHEVRONS_UP_DOWN: IconId = IconId::new_static("ui.chevrons.up_down");
        pub const CLOSE: IconId = IconId::new_static("ui.close");
        pub const EYE: IconId = IconId::new_static("ui.eye");
        pub const EYE_OFF: IconId = IconId::new_static("ui.eye.off");
        pub const FILE: IconId = IconId::new_static("ui.file");
        pub const GIT_COMMIT: IconId = IconId::new_static("ui.git.commit");
        pub const FOLDER: IconId = IconId::new_static("ui.folder");
        pub const FOLDER_OPEN: IconId = IconId::new_static("ui.folder.open");
        pub const LOADER: IconId = IconId::new_static("ui.loader");
        pub const MORE_HORIZONTAL: IconId = IconId::new_static("ui.more.horizontal");
        pub const MINUS: IconId = IconId::new_static("ui.minus");
        pub const PLUS: IconId = IconId::new_static("ui.plus");
        pub const PLAY: IconId = IconId::new_static("ui.play");
        pub const SEARCH: IconId = IconId::new_static("ui.search");
        pub const RESET: IconId = IconId::new_static("ui.reset");
        pub const SETTINGS: IconId = IconId::new_static("ui.settings");
        pub const SLASH: IconId = IconId::new_static("ui.slash");
        pub const STATUS_FAILED: IconId = IconId::new_static("ui.status.failed");
        pub const STATUS_PENDING: IconId = IconId::new_static("ui.status.pending");
        pub const STATUS_RUNNING: IconId = IconId::new_static("ui.status.running");
        pub const STATUS_SUCCEEDED: IconId = IconId::new_static("ui.status.succeeded");
        pub const TOOL: IconId = IconId::new_static("ui.tool");
    }
}

#[derive(Debug, Clone)]
pub enum IconSource {
    SvgStatic(&'static [u8]),
    SvgBytes(Arc<[u8]>),
    Alias(IconId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconRenderMode {
    #[default]
    Mask,
    OriginalColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IconPresentation {
    pub render_mode: IconRenderMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconGlyphFallback {
    pub glyph: char,
    pub font_family: Cow<'static, str>,
}

impl IconGlyphFallback {
    pub fn new(glyph: char, font_family: impl Into<Cow<'static, str>>) -> Self {
        Self {
            glyph,
            font_family: font_family.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconFallback {
    Glyph(IconGlyphFallback),
}

#[derive(Debug, Clone)]
pub struct IconDefinition {
    pub source: IconSource,
    pub fallback: Option<IconFallback>,
    pub presentation: IconPresentation,
}

impl IconDefinition {
    pub fn new(source: IconSource) -> Self {
        Self {
            source,
            fallback: None,
            presentation: IconPresentation::default(),
        }
    }

    pub fn svg_static(svg: &'static [u8]) -> Self {
        Self::new(IconSource::SvgStatic(svg))
    }

    pub fn svg_bytes(svg: Arc<[u8]>) -> Self {
        Self::new(IconSource::SvgBytes(svg))
    }

    pub fn alias(target: IconId) -> Self {
        Self::new(IconSource::Alias(target))
    }

    pub fn with_presentation(mut self, presentation: IconPresentation) -> Self {
        self.presentation = presentation;
        self
    }

    pub fn with_render_mode(mut self, render_mode: IconRenderMode) -> Self {
        self.presentation.render_mode = render_mode;
        self
    }

    pub fn with_fallback(mut self, fallback: IconFallback) -> Self {
        self.fallback = Some(fallback);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPackImportModel {
    Generated,
    Vendored,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconPackMetadata {
    pub pack_id: &'static str,
    pub vendor_namespace: &'static str,
    pub import_model: IconPackImportModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledIconPackMetadataConflict {
    pub existing: IconPackMetadata,
    pub attempted: IconPackMetadata,
}

impl std::fmt::Display for InstalledIconPackMetadataConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "icon pack metadata conflict for pack_id `{}`: existing vendor namespace `{}` / import model `{:?}`, attempted vendor namespace `{}` / import model `{:?}`",
            self.existing.pack_id,
            self.existing.vendor_namespace,
            self.existing.import_model,
            self.attempted.vendor_namespace,
            self.attempted.import_model
        )
    }
}

impl std::error::Error for InstalledIconPackMetadataConflict {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconInstallFailureKind {
    RegistryFreezeFailed,
    MetadataConflict,
}

impl IconInstallFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryFreezeFailed => "registry_freeze_failed",
            Self::MetadataConflict => "metadata_conflict",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconInstallFailureReport {
    pub surface: &'static str,
    pub pack_id: Option<&'static str>,
    pub kind: IconInstallFailureKind,
    pub details: Vec<String>,
}

impl IconInstallFailureReport {
    pub fn registry_freeze(
        surface: &'static str,
        pack_id: Option<&'static str>,
        errors: &[ResolveError],
    ) -> Self {
        Self {
            surface,
            pack_id,
            kind: IconInstallFailureKind::RegistryFreezeFailed,
            details: errors.iter().map(|error| format!("{error:?}")).collect(),
        }
    }

    pub fn metadata_conflict(
        surface: &'static str,
        conflict: &InstalledIconPackMetadataConflict,
    ) -> Self {
        Self {
            surface,
            pack_id: Some(conflict.existing.pack_id),
            kind: IconInstallFailureKind::MetadataConflict,
            details: vec![
                format!(
                    "existing vendor namespace `{}` / import model `{:?}`",
                    conflict.existing.vendor_namespace, conflict.existing.import_model
                ),
                format!(
                    "attempted vendor namespace `{}` / import model `{:?}`",
                    conflict.attempted.vendor_namespace, conflict.attempted.import_model
                ),
            ],
        }
    }
}

impl std::fmt::Display for IconInstallFailureReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "icon install failure in `{}`", self.surface)?;
        if let Some(pack_id) = self.pack_id {
            write!(f, " for pack `{pack_id}`")?;
        }
        match self.kind {
            IconInstallFailureKind::RegistryFreezeFailed => {
                write!(f, ": failed to freeze icon registry")?;
            }
            IconInstallFailureKind::MetadataConflict => {
                write!(f, ": conflicting installed icon pack metadata")?;
            }
        }
        if !self.details.is_empty() {
            write!(f, " [{}]", self.details.join(" | "))?;
        }
        Ok(())
    }
}

thread_local! {
    static CURRENT_ICON_INSTALL_FAILURE_REPORT: RefCell<Option<IconInstallFailureReport>> = const {
        RefCell::new(None)
    };
}

struct IconInstallFailureReportScope;

impl IconInstallFailureReportScope {
    fn install(report: IconInstallFailureReport) -> Self {
        CURRENT_ICON_INSTALL_FAILURE_REPORT.with(|slot| {
            *slot.borrow_mut() = Some(report);
        });
        Self
    }
}

impl Drop for IconInstallFailureReportScope {
    fn drop(&mut self) {
        CURRENT_ICON_INSTALL_FAILURE_REPORT.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}

/// Returns the current icon-install failure report for diagnostics/panic-hook integration.
///
/// This is only populated while one of the explicit icon install helpers is panicking.
pub fn current_icon_install_failure_report_for_diagnostics() -> Option<IconInstallFailureReport> {
    CURRENT_ICON_INSTALL_FAILURE_REPORT.with(|slot| slot.borrow().clone())
}

fn panic_with_icon_install_failure(report: IconInstallFailureReport) -> ! {
    let _scope = IconInstallFailureReportScope::install(report.clone());
    panic!("{report}");
}

/// Panic with a known icon-install failure report for registry-freeze failures.
pub fn panic_on_icon_registry_freeze_failure(
    surface: &'static str,
    pack_id: Option<&'static str>,
    errors: Vec<ResolveError>,
) -> ! {
    panic_with_icon_install_failure(IconInstallFailureReport::registry_freeze(
        surface, pack_id, &errors,
    ))
}

/// Panic with a known icon-install failure report for installed-pack metadata conflicts.
pub fn panic_on_icon_pack_metadata_conflict(
    surface: &'static str,
    conflict: InstalledIconPackMetadataConflict,
) -> ! {
    panic_with_icon_install_failure(IconInstallFailureReport::metadata_conflict(
        surface, &conflict,
    ))
}

/// Data-first icon-pack registration contract.
///
/// This stays deliberately small: icon packs remain registry/data oriented rather than growing a
/// shared trait surface. App/bootstrap layers may use this value to keep pack metadata/provenance
/// explicit while still deferring actual install policy to the owning layer.
#[derive(Debug, Clone, Copy)]
pub struct IconPackRegistration {
    pub metadata: IconPackMetadata,
    pub register: fn(&mut IconRegistry),
}

impl IconPackRegistration {
    pub const fn new(metadata: IconPackMetadata, register: fn(&mut IconRegistry)) -> Self {
        Self { metadata, register }
    }

    pub fn register_into_registry(self, registry: &mut IconRegistry) {
        (self.register)(registry);
    }
}

/// App-owned record of installed icon packs and their provenance metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InstalledIconPacks {
    packs: Vec<IconPackMetadata>,
}

impl InstalledIconPacks {
    pub fn record(
        &mut self,
        metadata: IconPackMetadata,
    ) -> Result<bool, InstalledIconPackMetadataConflict> {
        if let Some(existing) = self
            .packs
            .iter()
            .find(|existing| existing.pack_id == metadata.pack_id)
        {
            if *existing == metadata {
                return Ok(false);
            }
            return Err(InstalledIconPackMetadataConflict {
                existing: *existing,
                attempted: metadata,
            });
        }
        self.packs.push(metadata);
        Ok(true)
    }

    pub fn contains(&self, pack_id: &str) -> bool {
        self.packs
            .iter()
            .any(|metadata| metadata.pack_id == pack_id)
    }

    pub fn entries(&self) -> &[IconPackMetadata] {
        &self.packs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterError {
    DuplicateId { id: IconId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    Missing {
        requested: IconId,
        missing: IconId,
        chain: Vec<IconId>,
    },
    AliasLoop {
        requested: IconId,
        chain: Vec<IconId>,
    },
    AliasDepthExceeded {
        requested: IconId,
        chain: Vec<IconId>,
        max_depth: usize,
    },
}

#[derive(Debug, Default)]
pub struct IconRegistry {
    icons: HashMap<IconId, IconDefinition>,
}

impl IconRegistry {
    pub const MAX_ALIAS_DEPTH: usize = 64;

    pub fn register(&mut self, id: IconId, source: IconSource) -> Option<IconSource> {
        self.register_icon(id, IconDefinition::new(source))
            .map(|existing| existing.source)
    }

    pub fn try_register(&mut self, id: IconId, source: IconSource) -> Result<(), RegisterError> {
        self.try_register_icon(id, IconDefinition::new(source))
    }

    pub fn register_icon(
        &mut self,
        id: IconId,
        definition: IconDefinition,
    ) -> Option<IconDefinition> {
        self.icons.insert(id, definition)
    }

    pub fn try_register_icon(
        &mut self,
        id: IconId,
        definition: IconDefinition,
    ) -> Result<(), RegisterError> {
        if self.icons.contains_key(&id) {
            return Err(RegisterError::DuplicateId { id });
        }
        self.icons.insert(id, definition);
        Ok(())
    }

    pub fn register_svg_static(&mut self, id: IconId, svg: &'static [u8]) -> Option<IconSource> {
        self.register(id, IconSource::SvgStatic(svg))
    }

    pub fn register_svg_bytes(&mut self, id: IconId, svg: Arc<[u8]>) -> Option<IconSource> {
        self.register(id, IconSource::SvgBytes(svg))
    }

    pub fn register_svg_static_with_presentation(
        &mut self,
        id: IconId,
        svg: &'static [u8],
        presentation: IconPresentation,
    ) -> Option<IconDefinition> {
        self.register_icon(
            id,
            IconDefinition::svg_static(svg).with_presentation(presentation),
        )
    }

    pub fn register_svg_bytes_with_presentation(
        &mut self,
        id: IconId,
        svg: Arc<[u8]>,
        presentation: IconPresentation,
    ) -> Option<IconDefinition> {
        self.register_icon(
            id,
            IconDefinition::svg_bytes(svg).with_presentation(presentation),
        )
    }

    pub fn alias(&mut self, id: IconId, target: IconId) -> Option<IconSource> {
        self.register(id, IconSource::Alias(target))
    }

    /// Register an alias only if the ID is not already present in the registry.
    ///
    /// This is useful when multiple icon packs may provide semantic `ui.*` aliases and we want a
    /// stable "first registered wins" policy without requiring compile-time feature gating.
    /// Call [`alias`](Self::alias) or [`register`](Self::register) afterwards if app/bootstrap code
    /// intentionally wants to override that semantic default.
    ///
    /// Returns `true` if the alias was registered.
    pub fn alias_if_missing(&mut self, id: IconId, target: IconId) -> bool {
        if self.icons.contains_key(&id) {
            return false;
        }
        self.icons.insert(id, IconDefinition::alias(target));
        true
    }

    pub fn contains(&self, id: &IconId) -> bool {
        self.icons.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.icons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }

    pub fn icon_ids(&self) -> impl Iterator<Item = &IconId> {
        self.icons.keys()
    }

    pub fn resolve_icon(&self, id: &IconId) -> Result<ResolvedIcon<'_>, ResolveError> {
        let definition = self.resolve_definition(id)?;
        let svg = match &definition.source {
            IconSource::SvgStatic(bytes) => ResolvedSvg::Static(bytes),
            IconSource::SvgBytes(bytes) => ResolvedSvg::Bytes(bytes),
            IconSource::Alias(_) => unreachable!("alias chains must be resolved before returning"),
        };

        Ok(ResolvedIcon {
            svg,
            fallback: definition.fallback.as_ref(),
            presentation: definition.presentation,
        })
    }

    pub fn resolve(&self, id: &IconId) -> Result<ResolvedSvg<'_>, ResolveError> {
        self.resolve_icon(id).map(|resolved| resolved.svg)
    }

    pub fn resolve_icon_owned(&self, id: &IconId) -> Result<ResolvedIconOwned, ResolveError> {
        self.resolve_icon(id).map(ResolvedIcon::into_owned)
    }

    pub fn resolve_owned(&self, id: &IconId) -> Result<ResolvedSvgOwned, ResolveError> {
        self.resolve(id).map(ResolvedSvg::into_owned)
    }

    pub fn resolve_icon_or_missing_owned(&self, id: &IconId) -> ResolvedIconOwned {
        self.resolve_icon_owned(id)
            .unwrap_or_else(|_| ResolvedIconOwned {
                svg: ResolvedSvgOwned::Static(MISSING_ICON_SVG),
                fallback: None,
                presentation: IconPresentation::default(),
            })
    }

    pub fn resolve_or_missing_owned(&self, id: &IconId) -> ResolvedSvgOwned {
        self.resolve_owned(id)
            .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
    }

    pub fn freeze(&self) -> Result<FrozenIconRegistry, Vec<ResolveError>> {
        FrozenIconRegistry::from_registry_ref(self)
    }

    pub fn freeze_or_default_with_context(&self, context: &str) -> FrozenIconRegistry {
        let (frozen, errors) = FrozenIconRegistry::from_registry_ref_collecting_errors(self);
        if errors.is_empty() {
            frozen
        } else {
            emit_freeze_warning(context, &errors);
            frozen
        }
    }

    fn resolve_definition(&self, id: &IconId) -> Result<&IconDefinition, ResolveError> {
        let mut current = id;
        let mut chain = vec![id.clone()];

        for _ in 0..Self::MAX_ALIAS_DEPTH {
            let Some(definition) = self.icons.get(current) else {
                return Err(ResolveError::Missing {
                    requested: id.clone(),
                    missing: current.clone(),
                    chain,
                });
            };

            match &definition.source {
                IconSource::Alias(next) => {
                    if chain.iter().any(|seen| seen == next) {
                        chain.push(next.clone());
                        return Err(ResolveError::AliasLoop {
                            requested: id.clone(),
                            chain,
                        });
                    }
                    chain.push(next.clone());
                    current = next;
                }
                IconSource::SvgStatic(_) | IconSource::SvgBytes(_) => return Ok(definition),
            }
        }

        Err(ResolveError::AliasDepthExceeded {
            requested: id.clone(),
            chain,
            max_depth: Self::MAX_ALIAS_DEPTH,
        })
    }
}

fn emit_freeze_warning(context: &str, errors: &[ResolveError]) {
    let index = FREEZE_WARNING_COUNT.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_FREEZE_WARNING_LOGS {
        return;
    }

    eprintln!(
        "[fret-icons] freeze failed: context={context}, errors={}",
        errors.len()
    );

    for error in errors.iter().take(3) {
        eprintln!("[fret-icons]   {error:?}");
    }

    if errors.len() > 3 {
        eprintln!("[fret-icons]   ... and {} more", errors.len() - 3);
    }

    if index + 1 == MAX_FREEZE_WARNING_LOGS {
        eprintln!(
            "[fret-icons] further freeze warnings are suppressed after {} logs",
            MAX_FREEZE_WARNING_LOGS
        );
    }
}

#[derive(Debug, Default)]
pub struct IconRegistryBuilder {
    registry: IconRegistry,
}

impl IconRegistryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, id: IconId, source: IconSource) -> Self {
        self.registry.register(id, source);
        self
    }

    pub fn register_icon(mut self, id: IconId, definition: IconDefinition) -> Self {
        self.registry.register_icon(id, definition);
        self
    }

    pub fn register_svg_static(mut self, id: IconId, svg: &'static [u8]) -> Self {
        self.registry.register_svg_static(id, svg);
        self
    }

    pub fn register_svg_bytes(mut self, id: IconId, svg: Arc<[u8]>) -> Self {
        self.registry.register_svg_bytes(id, svg);
        self
    }

    pub fn alias(mut self, id: IconId, target: IconId) -> Self {
        self.registry.alias(id, target);
        self
    }

    pub fn registry_mut(&mut self) -> &mut IconRegistry {
        &mut self.registry
    }

    pub fn build(self) -> IconRegistry {
        self.registry
    }

    pub fn freeze(self) -> Result<FrozenIconRegistry, Vec<ResolveError>> {
        FrozenIconRegistry::from_registry(self.registry)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrozenIconRegistry {
    icons: HashMap<IconId, ResolvedIconOwned>,
}

impl FrozenIconRegistry {
    pub fn from_registry(registry: IconRegistry) -> Result<Self, Vec<ResolveError>> {
        Self::from_registry_ref(&registry)
    }

    pub fn from_registry_ref(registry: &IconRegistry) -> Result<Self, Vec<ResolveError>> {
        let (frozen, errors) = Self::from_registry_ref_collecting_errors(registry);
        if errors.is_empty() {
            Ok(frozen)
        } else {
            Err(errors)
        }
    }

    fn from_registry_ref_collecting_errors(registry: &IconRegistry) -> (Self, Vec<ResolveError>) {
        let mut ids: Vec<IconId> = registry.icon_ids().cloned().collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));

        let mut icons = HashMap::with_capacity(ids.len());
        let mut errors = Vec::new();

        for id in ids {
            match registry.resolve_icon_owned(&id) {
                Ok(resolved) => {
                    icons.insert(id, resolved);
                }
                Err(error) => errors.push(error),
            }
        }

        (Self { icons }, errors)
    }

    pub fn icon_ids(&self) -> impl Iterator<Item = &IconId> {
        self.icons.keys()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&IconId, &ResolvedIconOwned)> {
        self.icons.iter()
    }

    pub fn collect_icons_owned(&self) -> Vec<(IconId, ResolvedIconOwned)> {
        let mut ids: Vec<IconId> = self.icon_ids().cloned().collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.into_iter()
            .filter_map(|id| {
                self.resolve_icon(&id)
                    .cloned()
                    .map(|resolved| (id, resolved))
            })
            .collect()
    }

    pub fn collect_owned(&self) -> Vec<(IconId, ResolvedSvgOwned)> {
        self.collect_icons_owned()
            .into_iter()
            .map(|(id, resolved)| (id, resolved.svg))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.icons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }

    pub fn resolve_icon(&self, id: &IconId) -> Option<&ResolvedIconOwned> {
        self.icons.get(id)
    }

    pub fn resolve(&self, id: &IconId) -> Option<&ResolvedSvgOwned> {
        self.icons.get(id).map(|resolved| &resolved.svg)
    }

    pub fn resolve_icon_or_missing_owned(&self, id: &IconId) -> ResolvedIconOwned {
        self.resolve_icon(id).cloned().unwrap_or(ResolvedIconOwned {
            svg: ResolvedSvgOwned::Static(MISSING_ICON_SVG),
            fallback: None,
            presentation: IconPresentation::default(),
        })
    }

    pub fn resolve_or_missing_owned(&self, id: &IconId) -> ResolvedSvgOwned {
        self.resolve(id)
            .cloned()
            .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
    }
}

#[derive(Debug)]
pub struct ResolvedIcon<'a> {
    pub svg: ResolvedSvg<'a>,
    pub fallback: Option<&'a IconFallback>,
    pub presentation: IconPresentation,
}

impl ResolvedIcon<'_> {
    pub fn into_owned(self) -> ResolvedIconOwned {
        ResolvedIconOwned {
            svg: self.svg.into_owned(),
            fallback: self.fallback.cloned(),
            presentation: self.presentation,
        }
    }
}

#[derive(Debug)]
pub enum ResolvedSvg<'a> {
    Static(&'static [u8]),
    Bytes(&'a Arc<[u8]>),
}

impl ResolvedSvg<'_> {
    pub fn into_owned(self) -> ResolvedSvgOwned {
        match self {
            Self::Static(bytes) => ResolvedSvgOwned::Static(bytes),
            Self::Bytes(bytes) => ResolvedSvgOwned::Bytes(bytes.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResolvedSvgOwned {
    Static(&'static [u8]),
    Bytes(Arc<[u8]>),
}

impl ResolvedSvgOwned {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Static(bytes) => bytes,
            Self::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedIconOwned {
    pub svg: ResolvedSvgOwned,
    pub fallback: Option<IconFallback>,
    pub presentation: IconPresentation,
}

pub const MISSING_ICON_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M16 8 8 16"/><path d="M8 8l8 8"/></svg>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_alias_loop_reports_error() {
        let mut registry = IconRegistry::default();
        registry.alias(IconId::new_static("a"), IconId::new_static("b"));
        registry.alias(IconId::new_static("b"), IconId::new_static("a"));

        let error = registry
            .resolve(&IconId::new_static("a"))
            .expect_err("loop must return an error");

        match error {
            ResolveError::AliasLoop { chain, .. } => {
                assert!(chain.len() >= 3);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn freeze_resolves_aliases() {
        let registry = IconRegistryBuilder::new()
            .register_svg_static(IconId::new_static("base"), b"<svg/>" as &[u8])
            .alias(IconId::new_static("alias"), IconId::new_static("base"))
            .build();

        let frozen = FrozenIconRegistry::from_registry(registry).expect("freeze must succeed");

        let resolved = frozen.resolve_or_missing_owned(&IconId::new_static("alias"));
        assert_eq!(resolved.as_bytes(), b"<svg/>");
    }

    #[test]
    fn freeze_or_default_returns_empty_when_all_entries_fail() {
        let mut registry = IconRegistry::default();
        registry.alias(IconId::new_static("a"), IconId::new_static("b"));
        registry.alias(IconId::new_static("b"), IconId::new_static("a"));

        let frozen = registry.freeze_or_default_with_context("test.freeze_or_default");
        assert!(frozen.is_empty());
    }

    #[test]
    fn freeze_or_default_keeps_valid_icons_when_other_entries_fail() {
        let mut registry = IconRegistry::default();
        let _ = registry.register_svg_static(IconId::new_static("ok"), b"<svg/>");
        registry.alias(IconId::new_static("a"), IconId::new_static("b"));
        registry.alias(IconId::new_static("b"), IconId::new_static("a"));

        let frozen = registry.freeze_or_default_with_context("test.freeze_partial");
        assert_eq!(
            frozen
                .resolve_or_missing_owned(&IconId::new_static("ok"))
                .as_bytes(),
            b"<svg/>"
        );
        assert!(frozen.resolve(&IconId::new_static("a")).is_none());
    }

    #[test]
    fn resolve_icon_preserves_presentation_and_fallback_through_aliases() {
        let definition = IconDefinition::svg_static(b"<svg/>" as &[u8])
            .with_render_mode(IconRenderMode::OriginalColors)
            .with_fallback(IconFallback::Glyph(IconGlyphFallback::new(
                '?',
                "fallback-ui",
            )));

        let registry = IconRegistryBuilder::new()
            .register_icon(IconId::new_static("base"), definition)
            .alias(IconId::new_static("alias"), IconId::new_static("base"))
            .build();

        let resolved = registry
            .resolve_icon_owned(&IconId::new_static("alias"))
            .expect("alias must resolve to full icon definition");

        assert_eq!(
            resolved.presentation.render_mode,
            IconRenderMode::OriginalColors
        );
        match resolved.fallback {
            Some(IconFallback::Glyph(glyph)) => {
                assert_eq!(glyph.glyph, '?');
                assert_eq!(glyph.font_family.as_ref(), "fallback-ui");
            }
            other => panic!("unexpected fallback: {other:?}"),
        }
        assert_eq!(resolved.svg.as_bytes(), b"<svg/>");
    }

    #[test]
    fn icon_pack_registration_registers_through_explicit_value_object() {
        fn register_demo(registry: &mut IconRegistry) {
            let _ = registry.register_svg_static(IconId::new_static("demo.icon"), b"<svg/>");
        }

        let pack = IconPackRegistration::new(
            IconPackMetadata {
                pack_id: "demo-pack",
                vendor_namespace: "demo",
                import_model: IconPackImportModel::Manual,
            },
            register_demo,
        );

        let mut registry = IconRegistry::default();
        pack.register_into_registry(&mut registry);

        assert!(registry.contains(&IconId::new_static("demo.icon")));
    }

    #[test]
    fn installed_icon_packs_dedupes_same_pack_metadata() {
        let metadata = IconPackMetadata {
            pack_id: "demo-pack",
            vendor_namespace: "demo",
            import_model: IconPackImportModel::Generated,
        };

        let mut installed = InstalledIconPacks::default();
        assert_eq!(installed.record(metadata), Ok(true));
        assert_eq!(installed.record(metadata), Ok(false));
        assert!(installed.contains("demo-pack"));
        assert_eq!(installed.entries(), &[metadata]);
    }

    #[test]
    fn installed_icon_packs_reject_conflicting_metadata_for_same_pack_id() {
        let mut installed = InstalledIconPacks::default();
        installed
            .record(IconPackMetadata {
                pack_id: "demo-pack",
                vendor_namespace: "demo",
                import_model: IconPackImportModel::Generated,
            })
            .expect("first record should succeed");

        let err = installed
            .record(IconPackMetadata {
                pack_id: "demo-pack",
                vendor_namespace: "other-demo",
                import_model: IconPackImportModel::Vendored,
            })
            .expect_err("conflicting metadata should fail");

        assert_eq!(err.existing.pack_id, "demo-pack");
        assert_eq!(err.existing.vendor_namespace, "demo");
        assert_eq!(err.attempted.vendor_namespace, "other-demo");
    }

    #[test]
    fn icon_install_failure_report_scope_is_only_visible_while_installed() {
        let report = IconInstallFailureReport::registry_freeze(
            "test.icon.install",
            Some("demo-pack"),
            &[ResolveError::AliasLoop {
                requested: IconId::new_static("demo.icon"),
                chain: vec![
                    IconId::new_static("demo.icon"),
                    IconId::new_static("demo.icon"),
                ],
            }],
        );

        assert_eq!(current_icon_install_failure_report_for_diagnostics(), None);
        {
            let _scope = IconInstallFailureReportScope::install(report.clone());
            assert_eq!(
                current_icon_install_failure_report_for_diagnostics(),
                Some(report)
            );
        }
        assert_eq!(current_icon_install_failure_report_for_diagnostics(), None);
    }

    #[test]
    fn panic_on_icon_registry_freeze_failure_uses_human_readable_message() {
        let result = std::panic::catch_unwind(|| {
            panic_on_icon_registry_freeze_failure(
                "test.icon.install",
                Some("demo-pack"),
                vec![ResolveError::AliasLoop {
                    requested: IconId::new_static("demo.icon"),
                    chain: vec![
                        IconId::new_static("demo.icon"),
                        IconId::new_static("demo.icon"),
                    ],
                }],
            );
        });

        let panic = result.expect_err("freeze failure should panic");
        let message = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                panic
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
            })
            .expect("string panic payload");
        assert!(message.contains("icon install failure in `test.icon.install`"));
        assert!(message.contains("for pack `demo-pack`"));
        assert!(message.contains("failed to freeze icon registry"));
        assert_eq!(current_icon_install_failure_report_for_diagnostics(), None);
    }

    #[test]
    fn panic_on_icon_pack_metadata_conflict_uses_human_readable_message() {
        let result = std::panic::catch_unwind(|| {
            panic_on_icon_pack_metadata_conflict(
                "test.icon.install",
                InstalledIconPackMetadataConflict {
                    existing: IconPackMetadata {
                        pack_id: "demo-pack",
                        vendor_namespace: "demo",
                        import_model: IconPackImportModel::Generated,
                    },
                    attempted: IconPackMetadata {
                        pack_id: "demo-pack",
                        vendor_namespace: "other-demo",
                        import_model: IconPackImportModel::Vendored,
                    },
                },
            );
        });

        let panic = result.expect_err("metadata conflict should panic");
        let message = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                panic
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
            })
            .expect("string panic payload");
        assert!(message.contains("icon install failure in `test.icon.install`"));
        assert!(message.contains("for pack `demo-pack`"));
        assert!(message.contains("conflicting installed icon pack metadata"));
        assert_eq!(current_icon_install_failure_report_for_diagnostics(), None);
    }
}
