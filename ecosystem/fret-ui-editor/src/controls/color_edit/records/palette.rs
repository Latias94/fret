use std::sync::Arc;

const COLOR_PRESETS: [(&str, u32); 12] = [
    ("Slate", 0x0f_17_2a),
    ("Red", 0xef_44_44),
    ("Orange", 0xf9_73_16),
    ("Amber", 0xf5_9e_0b),
    ("Yellow", 0xea_d3_08),
    ("Green", 0x22_c5_5e),
    ("Emerald", 0x10_b9_81),
    ("Cyan", 0x06_b6_d4),
    ("Blue", 0x3b_82_f6),
    ("Violet", 0x8b_5c_f6),
    ("Fuchsia", 0xd9_46_ef),
    ("White", 0xff_ff_ff),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorEditPaletteEntry {
    pub name: Arc<str>,
    pub rgb: u32,
}

impl ColorEditPaletteEntry {
    pub fn new(name: impl Into<Arc<str>>, rgb: u32) -> Self {
        Self {
            name: name.into(),
            rgb,
        }
    }

    pub fn with_rgb(mut self, rgb: u32) -> Self {
        self.rgb = rgb;
        self
    }
}

pub fn default_color_edit_palette() -> Arc<[ColorEditPaletteEntry]> {
    COLOR_PRESETS
        .iter()
        .map(|(name, rgb)| ColorEditPaletteEntry::new(*name, *rgb))
        .collect::<Vec<_>>()
        .into()
}
