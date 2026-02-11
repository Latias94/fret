use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeymapFileV1 {
    pub keymap_version: u32,
    pub bindings: Vec<BindingV1>,
}

#[derive(Debug, Deserialize)]
pub struct BindingV1 {
    pub command: Option<String>,
    pub platform: Option<String>,
    pub when: Option<String>,
    pub keys: KeySpecV1,
}

#[derive(Debug, Deserialize)]
pub struct KeySpecV1 {
    pub mods: Vec<String>,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct KeymapFileAny {
    pub keymap_version: u32,
    pub bindings: Vec<BindingAny>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BindingAny {
    pub command: Option<String>,
    pub platform: Option<String>,
    pub when: Option<String>,
    pub keys: KeysAny,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum KeysAny {
    Single(KeySpecV1),
    Sequence(Vec<KeySpecV1>),
}
