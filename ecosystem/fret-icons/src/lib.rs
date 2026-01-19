//! Renderer-agnostic icon identity and registry for Fret component libraries.
//!
//! This crate is intentionally rendering-agnostic:
//! - Components depend on semantic icon IDs (`IconId`).
//! - Icon packs register assets as data (`IconSource`).
//! - Rendering (SVG raster caching, budgets, atlases) remains in the renderer layer.

use std::{borrow::Cow, collections::HashMap, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default)]
pub struct IconRegistry {
    icons: HashMap<IconId, IconSource>,
}

impl IconRegistry {
    pub fn register(&mut self, id: IconId, source: IconSource) {
        self.icons.insert(id, source);
    }

    pub fn register_svg_static(&mut self, id: IconId, svg: &'static [u8]) {
        self.register(id, IconSource::SvgStatic(svg));
    }

    pub fn register_svg_bytes(&mut self, id: IconId, svg: Arc<[u8]>) {
        self.register(id, IconSource::SvgBytes(svg));
    }

    pub fn alias(&mut self, id: IconId, target: IconId) {
        self.register(id, IconSource::Alias(target));
    }

    pub fn source(&self, id: &IconId) -> Option<&IconSource> {
        self.icons.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&IconId, &IconSource)> {
        self.icons.iter()
    }

    pub fn resolve_svg(&self, id: &IconId) -> Option<ResolvedSvg<'_>> {
        let mut current = id;
        for _ in 0..32 {
            match self.icons.get(current)? {
                IconSource::SvgStatic(bytes) => return Some(ResolvedSvg::Static(bytes)),
                IconSource::SvgBytes(bytes) => return Some(ResolvedSvg::Bytes(bytes)),
                IconSource::Alias(next) => current = next,
            }
        }
        None
    }

    pub fn resolve_svg_owned(&self, id: &IconId) -> Option<ResolvedSvgOwned> {
        self.resolve_svg(id).map(|r| match r {
            ResolvedSvg::Static(bytes) => ResolvedSvgOwned::Static(bytes),
            ResolvedSvg::Bytes(bytes) => ResolvedSvgOwned::Bytes(bytes.clone()),
        })
    }
}

pub enum ResolvedSvg<'a> {
    Static(&'static [u8]),
    Bytes(&'a Arc<[u8]>),
}

#[derive(Clone)]
pub enum ResolvedSvgOwned {
    Static(&'static [u8]),
    Bytes(Arc<[u8]>),
}

pub const MISSING_ICON_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M16 8 8 16"/><path d="M8 8l8 8"/></svg>"#;
