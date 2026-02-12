use crate::{CommandId, KeyChord, WhenExpr};
use fret_core::{KeyCode, Modifiers};

use super::wire::{KeySpecV1, KeymapFileAny, KeymapFileV1, KeysAny};
use super::{Binding, Keymap, KeymapError, KeymapLoadOptions, PlatformFilter, WhenValidationMode};

impl Keymap {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeymapError> {
        Self::from_bytes_with_options(bytes, KeymapLoadOptions::default())
    }

    pub fn from_bytes_with_options(
        bytes: &[u8],
        options: KeymapLoadOptions,
    ) -> Result<Self, KeymapError> {
        let parsed: KeymapFileAny =
            serde_json::from_slice(bytes).map_err(|source| KeymapError::ParseFailed { source })?;
        Self::from_any(parsed, options)
    }

    pub fn from_v1(file: KeymapFileV1) -> Result<Self, KeymapError> {
        Self::from_v1_with_options(file, KeymapLoadOptions::default())
    }

    pub fn from_v1_with_options(
        file: KeymapFileV1,
        options: KeymapLoadOptions,
    ) -> Result<Self, KeymapError> {
        if file.keymap_version != 1 {
            return Err(KeymapError::UnsupportedVersion(file.keymap_version));
        }

        let mut out = Keymap::empty();
        for (index, b) in file.bindings.into_iter().enumerate() {
            let platform = match b.platform.as_deref().unwrap_or("all") {
                "all" => PlatformFilter::All,
                "macos" => PlatformFilter::Macos,
                "windows" => PlatformFilter::Windows,
                "linux" => PlatformFilter::Linux,
                "web" => PlatformFilter::Web,
                other => {
                    return Err(KeymapError::UnknownPlatform {
                        index,
                        value: other.into(),
                    });
                }
            };

            let chord = parse_keys(index, b.keys)?;

            let when = if let Some(when) = b.when.as_deref() {
                Some(parse_when(index, when, options.when_validation)?)
            } else {
                None
            };

            let command = b.command.map(CommandId::new);

            out.push_binding(Binding {
                platform,
                sequence: vec![chord],
                when,
                command,
            });
        }

        Ok(out)
    }

    fn from_any(file: KeymapFileAny, options: KeymapLoadOptions) -> Result<Self, KeymapError> {
        match file.keymap_version {
            1 => {
                let mut out = Keymap::empty();
                for (index, b) in file.bindings.into_iter().enumerate() {
                    let platform = match b.platform.as_deref().unwrap_or("all") {
                        "all" => PlatformFilter::All,
                        "macos" => PlatformFilter::Macos,
                        "windows" => PlatformFilter::Windows,
                        "linux" => PlatformFilter::Linux,
                        "web" => PlatformFilter::Web,
                        other => {
                            return Err(KeymapError::UnknownPlatform {
                                index,
                                value: other.into(),
                            });
                        }
                    };

                    let KeysAny::Single(keys) = b.keys else {
                        return Err(KeymapError::UnsupportedVersion(1));
                    };

                    let chord = parse_keys(index, keys)?;

                    let when = if let Some(when) = b.when.as_deref() {
                        Some(parse_when(index, when, options.when_validation)?)
                    } else {
                        None
                    };

                    let command = b.command.map(CommandId::new);

                    out.push_binding(Binding {
                        platform,
                        sequence: vec![chord],
                        when,
                        command,
                    });
                }
                Ok(out)
            }
            2 => {
                let mut out = Keymap::empty();
                for (index, b) in file.bindings.into_iter().enumerate() {
                    let platform = match b.platform.as_deref().unwrap_or("all") {
                        "all" => PlatformFilter::All,
                        "macos" => PlatformFilter::Macos,
                        "windows" => PlatformFilter::Windows,
                        "linux" => PlatformFilter::Linux,
                        "web" => PlatformFilter::Web,
                        other => {
                            return Err(KeymapError::UnknownPlatform {
                                index,
                                value: other.into(),
                            });
                        }
                    };

                    let key_specs = match b.keys {
                        KeysAny::Single(keys) => vec![keys],
                        KeysAny::Sequence(seq) => seq,
                    };
                    if key_specs.is_empty() {
                        return Err(KeymapError::EmptyKeys { index });
                    }

                    let mut sequence: Vec<KeyChord> = Vec::with_capacity(key_specs.len());
                    for keys in key_specs {
                        sequence.push(parse_keys(index, keys)?);
                    }

                    let when = if let Some(when) = b.when.as_deref() {
                        Some(parse_when(index, when, options.when_validation)?)
                    } else {
                        None
                    };

                    let command = b.command.map(CommandId::new);

                    out.push_binding(Binding {
                        platform,
                        sequence,
                        when,
                        command,
                    });
                }
                Ok(out)
            }
            other => Err(KeymapError::UnsupportedVersion(other)),
        }
    }
}

fn parse_keys(index: usize, keys: KeySpecV1) -> Result<KeyChord, KeymapError> {
    let key: KeyCode = keys.key.parse().map_err(|_| KeymapError::UnknownKeyToken {
        index,
        token: keys.key.clone(),
    })?;

    let mut mods = Modifiers::default();
    for m in keys.mods {
        let token = m.to_ascii_lowercase();
        match token.as_str() {
            "shift" => mods.shift = true,
            "ctrl" | "control" => mods.ctrl = true,
            "alt" | "option" => mods.alt = true,
            "altgr" | "alt_gr" | "altgraph" => mods.alt_gr = true,
            "meta" | "cmd" | "command" => mods.meta = true,
            other => {
                return Err(KeymapError::UnknownModifier {
                    index,
                    value: other.into(),
                });
            }
        }
    }
    Ok(KeyChord::new(key, mods))
}

fn parse_when(index: usize, when: &str, mode: WhenValidationMode) -> Result<WhenExpr, KeymapError> {
    let expr =
        WhenExpr::parse(when).map_err(|e| KeymapError::WhenParseFailed { index, error: e })?;
    if mode == WhenValidationMode::Strict {
        expr.validate()
            .map_err(|e| KeymapError::WhenValidationFailed {
                index,
                error: e.to_string(),
            })?;
    }
    Ok(expr)
}
