use super::*;

pub(super) struct MaterialEffectState {
    pub(super) materials: SlotMap<fret_core::MaterialId, MaterialEntry>,
    pub(super) materials_by_desc: HashMap<fret_core::MaterialDescriptor, fret_core::MaterialId>,
    pub(super) materials_generation: u64,
    pub(super) material_paint_budget_per_frame: u64,
    pub(super) material_distinct_budget_per_frame: usize,
    pub(super) custom_effects: SlotMap<fret_core::EffectId, CustomEffectEntry>,
    pub(super) custom_effect_hash_index: HashMap<u64, Vec<fret_core::EffectId>>,
    pub(super) custom_effects_generation: u64,
}

impl Default for MaterialEffectState {
    fn default() -> Self {
        Self {
            materials: SlotMap::with_key(),
            materials_by_desc: HashMap::new(),
            materials_generation: 0,
            material_paint_budget_per_frame: 50_000,
            material_distinct_budget_per_frame: 256,
            custom_effects: SlotMap::with_key(),
            custom_effect_hash_index: HashMap::new(),
            custom_effects_generation: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MaterialEntry {
    pub(super) desc: fret_core::MaterialDescriptor,
    pub(super) refs: u32,
}

#[derive(Clone, Debug)]
pub(super) struct CustomEffectEntry {
    pub(super) abi: CustomEffectAbi,
    pub(super) raw_source: Arc<str>,
    pub(super) wgsl_unmasked: Arc<str>,
    pub(super) wgsl_masked: Arc<str>,
    pub(super) wgsl_mask: Arc<str>,
    pub(super) refs: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CustomEffectAbi {
    V1,
    V2,
    V3,
}
