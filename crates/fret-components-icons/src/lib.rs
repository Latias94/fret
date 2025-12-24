//! Renderer-agnostic icon registry for Fret component libraries.
//!
//! This crate intentionally does **not** prescribe a rendering representation (SVG, SDF, atlas,
//! text glyphs, etc.). The initial MVP uses text-glyph fallbacks so that `fret-components-ui`
//! can ship an `IconButton` without a dedicated vector pipeline.

use std::{borrow::Cow, collections::HashMap};

use fret_core::{FontId, Px};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IconId(Cow<'static, str>);

impl IconId {
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct IconGlyph {
    pub text: Cow<'static, str>,
    pub font: FontId,
    pub size: Px,
}

impl IconGlyph {
    pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
        Self {
            text: text.into(),
            font: FontId::default(),
            size: Px(13.0),
        }
    }

    pub fn with_size(mut self, size: Px) -> Self {
        self.size = size;
        self
    }

    pub fn with_font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }
}

#[derive(Debug)]
pub struct IconRegistry {
    glyphs: HashMap<IconId, IconGlyph>,
}

impl IconRegistry {
    pub fn register_glyph(&mut self, id: IconId, glyph: IconGlyph) {
        self.glyphs.insert(id, glyph);
    }

    pub fn glyph(&self, id: &IconId) -> Option<&IconGlyph> {
        self.glyphs.get(id)
    }

    pub fn ensure_builtin_glyphs(&mut self) {
        for (id, glyph) in builtin_glyphs() {
            self.glyphs.entry(id).or_insert(glyph);
        }
    }
}

impl Default for IconRegistry {
    fn default() -> Self {
        let mut out = Self {
            glyphs: HashMap::new(),
        };
        out.ensure_builtin_glyphs();
        out
    }
}

fn builtin_glyphs() -> Vec<(IconId, IconGlyph)> {
    vec![
        (IconId::new("check"), IconGlyph::new("✓")),
        (IconId::new("chevron_down"), IconGlyph::new("▾")),
        (IconId::new("close"), IconGlyph::new("×")),
        (IconId::new("play"), IconGlyph::new("▶")),
        (IconId::new("settings"), IconGlyph::new("⚙")),
    ]
}
