//! Renderer-agnostic icon identity and registry for Fret component libraries.
//!
//! This crate is intentionally rendering-agnostic:
//! - Components depend on semantic icon IDs (`IconId`).
//! - Icon packs register assets as data (`IconSource`).
//! - Rendering (SVG raster caching, budgets, atlases) remains in the renderer layer.

use std::{borrow::Cow, collections::HashMap, sync::Arc};

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

        pub const CHECK: IconId = IconId::new_static("ui.check");
        pub const CHEVRON_DOWN: IconId = IconId::new_static("ui.chevron.down");
        pub const CHEVRON_RIGHT: IconId = IconId::new_static("ui.chevron.right");
        pub const CHEVRON_UP: IconId = IconId::new_static("ui.chevron.up");
        pub const CLOSE: IconId = IconId::new_static("ui.close");
        pub const MORE_HORIZONTAL: IconId = IconId::new_static("ui.more.horizontal");
        pub const MINUS: IconId = IconId::new_static("ui.minus");
        pub const PLAY: IconId = IconId::new_static("ui.play");
        pub const SEARCH: IconId = IconId::new_static("ui.search");
        pub const SETTINGS: IconId = IconId::new_static("ui.settings");
        pub const SLASH: IconId = IconId::new_static("ui.slash");
    }
}

#[derive(Debug, Clone)]
pub enum IconSource {
    SvgStatic(&'static [u8]),
    SvgBytes(Arc<[u8]>),
    Alias(IconId),
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
    icons: HashMap<IconId, IconSource>,
}

impl IconRegistry {
    pub const MAX_ALIAS_DEPTH: usize = 64;

    pub fn register(&mut self, id: IconId, source: IconSource) -> Option<IconSource> {
        self.icons.insert(id, source)
    }

    pub fn try_register(&mut self, id: IconId, source: IconSource) -> Result<(), RegisterError> {
        if self.icons.contains_key(&id) {
            return Err(RegisterError::DuplicateId { id });
        }
        self.icons.insert(id, source);
        Ok(())
    }

    pub fn register_svg_static(&mut self, id: IconId, svg: &'static [u8]) -> Option<IconSource> {
        self.register(id, IconSource::SvgStatic(svg))
    }

    pub fn register_svg_bytes(&mut self, id: IconId, svg: Arc<[u8]>) -> Option<IconSource> {
        self.register(id, IconSource::SvgBytes(svg))
    }

    pub fn alias(&mut self, id: IconId, target: IconId) -> Option<IconSource> {
        self.register(id, IconSource::Alias(target))
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

    pub fn resolve(&self, id: &IconId) -> Result<ResolvedSvg<'_>, ResolveError> {
        let mut current = id;
        let mut chain = vec![id.clone()];

        for _ in 0..Self::MAX_ALIAS_DEPTH {
            let Some(source) = self.icons.get(current) else {
                return Err(ResolveError::Missing {
                    requested: id.clone(),
                    missing: current.clone(),
                    chain,
                });
            };

            match source {
                IconSource::SvgStatic(bytes) => return Ok(ResolvedSvg::Static(bytes)),
                IconSource::SvgBytes(bytes) => return Ok(ResolvedSvg::Bytes(bytes)),
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
            }
        }

        Err(ResolveError::AliasDepthExceeded {
            requested: id.clone(),
            chain,
            max_depth: Self::MAX_ALIAS_DEPTH,
        })
    }

    pub fn resolve_owned(&self, id: &IconId) -> Result<ResolvedSvgOwned, ResolveError> {
        self.resolve(id).map(ResolvedSvg::into_owned)
    }

    pub fn resolve_or_missing(&self, id: &IconId) -> ResolvedSvg<'_> {
        self.resolve(id)
            .unwrap_or(ResolvedSvg::Static(MISSING_ICON_SVG))
    }

    pub fn resolve_or_missing_owned(&self, id: &IconId) -> ResolvedSvgOwned {
        self.resolve_owned(id)
            .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
    }

    pub fn collect_resolved_owned(&self) -> Vec<(IconId, ResolvedSvgOwned)> {
        let mut ids: Vec<IconId> = self.icon_ids().cloned().collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.into_iter()
            .filter_map(|id| self.resolve_owned(&id).ok().map(|resolved| (id, resolved)))
            .collect()
    }

    pub fn freeze(&self) -> Result<FrozenIconRegistry, Vec<ResolveError>> {
        FrozenIconRegistry::from_registry_ref(self)
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
    icons: HashMap<IconId, ResolvedSvgOwned>,
}

impl FrozenIconRegistry {
    pub fn from_registry(registry: IconRegistry) -> Result<Self, Vec<ResolveError>> {
        Self::from_registry_ref(&registry)
    }

    pub fn from_registry_ref(registry: &IconRegistry) -> Result<Self, Vec<ResolveError>> {
        let mut ids: Vec<IconId> = registry.icon_ids().cloned().collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));

        let mut icons = HashMap::with_capacity(ids.len());
        let mut errors = Vec::new();

        for id in ids {
            match registry.resolve_owned(&id) {
                Ok(resolved) => {
                    icons.insert(id, resolved);
                }
                Err(error) => errors.push(error),
            }
        }

        if errors.is_empty() {
            Ok(Self { icons })
        } else {
            Err(errors)
        }
    }

    pub fn icon_ids(&self) -> impl Iterator<Item = &IconId> {
        self.icons.keys()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&IconId, &ResolvedSvgOwned)> {
        self.icons.iter()
    }

    pub fn collect_owned(&self) -> Vec<(IconId, ResolvedSvgOwned)> {
        let mut ids: Vec<IconId> = self.icon_ids().cloned().collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.into_iter()
            .filter_map(|id| self.resolve(&id).cloned().map(|resolved| (id, resolved)))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.icons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }

    pub fn resolve(&self, id: &IconId) -> Option<&ResolvedSvgOwned> {
        self.icons.get(id)
    }

    pub fn resolve_or_missing_owned(&self, id: &IconId) -> ResolvedSvgOwned {
        self.resolve(id)
            .cloned()
            .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
    }
}

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
}
