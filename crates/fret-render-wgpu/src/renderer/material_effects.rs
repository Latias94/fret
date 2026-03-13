use super::*;
use std::collections::hash_map::Entry;

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

pub(super) enum CustomEffectUnregisterOutcome {
    Missing,
    StillReferenced,
    Removed,
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

impl MaterialEffectState {
    pub(super) fn register_material(
        &mut self,
        desc: fret_core::MaterialDescriptor,
    ) -> fret_core::MaterialId {
        match self.materials_by_desc.entry(desc) {
            Entry::Occupied(entry) => {
                let id = *entry.get();
                if let Some(material) = self.materials.get_mut(id) {
                    material.refs = material.refs.saturating_add(1);
                }
                id
            }
            Entry::Vacant(entry) => {
                let id = self.materials.insert(MaterialEntry { desc, refs: 1 });
                entry.insert(id);
                self.materials_generation = self.materials_generation.wrapping_add(1);
                id
            }
        }
    }

    pub(super) fn unregister_material(&mut self, id: fret_core::MaterialId) -> bool {
        let Some(refs) = self.materials.get(id).map(|entry| entry.refs) else {
            return false;
        };

        if refs > 1 {
            if let Some(entry) = self.materials.get_mut(id) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return true;
        }

        let Some(entry) = self.materials.remove(id) else {
            return false;
        };

        self.materials_by_desc.remove(&entry.desc);
        self.materials_generation = self.materials_generation.wrapping_add(1);
        true
    }

    pub(super) fn find_custom_effect(
        &self,
        abi: CustomEffectAbi,
        user_source: &str,
    ) -> Option<fret_core::EffectId> {
        let hash = custom_effect_hash(abi, user_source.as_bytes());
        let ids = self.custom_effect_hash_index.get(&hash)?;
        ids.iter().copied().find(|&id| {
            let Some(existing) = self.custom_effects.get(id) else {
                return false;
            };
            existing.abi == abi && existing.raw_source.as_ref() == user_source
        })
    }

    pub(super) fn retain_custom_effect(&mut self, id: fret_core::EffectId) -> bool {
        let Some(entry) = self.custom_effects.get_mut(id) else {
            return false;
        };
        entry.refs = entry.refs.saturating_add(1);
        true
    }

    pub(super) fn insert_custom_effect(&mut self, entry: CustomEffectEntry) -> fret_core::EffectId {
        let hash = custom_effect_hash(entry.abi, entry.raw_source.as_bytes());
        let id = self.custom_effects.insert(entry);
        self.custom_effect_hash_index
            .entry(hash)
            .or_default()
            .push(id);
        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        id
    }

    pub(super) fn unregister_custom_effect(
        &mut self,
        id: fret_core::EffectId,
    ) -> CustomEffectUnregisterOutcome {
        let Some(refs) = self.custom_effects.get(id).map(|entry| entry.refs) else {
            return CustomEffectUnregisterOutcome::Missing;
        };

        if refs > 1 {
            if let Some(entry) = self.custom_effects.get_mut(id) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return CustomEffectUnregisterOutcome::StillReferenced;
        }

        let Some(entry) = self.custom_effects.remove(id) else {
            return CustomEffectUnregisterOutcome::Missing;
        };

        let hash = custom_effect_hash(entry.abi, entry.raw_source.as_bytes());
        if let Some(list) = self.custom_effect_hash_index.get_mut(&hash) {
            list.retain(|existing| *existing != id);
            if list.is_empty() {
                self.custom_effect_hash_index.remove(&hash);
            }
        }

        self.custom_effects_generation = self.custom_effects_generation.wrapping_add(1);
        CustomEffectUnregisterOutcome::Removed
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

pub(super) fn custom_effect_hash(abi: CustomEffectAbi, raw_source: &[u8]) -> u64 {
    mix_u64(hash_bytes(raw_source), custom_effect_hash_salt(abi))
}

fn custom_effect_hash_salt(abi: CustomEffectAbi) -> u64 {
    match abi {
        CustomEffectAbi::V1 => 1,
        CustomEffectAbi::V2 => 2,
        CustomEffectAbi::V3 => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_registry_deduplicates_and_tracks_refcounts() {
        let mut state = MaterialEffectState::default();
        let desc = fret_core::MaterialDescriptor::new(fret_core::MaterialKind::DotGrid);

        let first = state.register_material(desc);
        let second = state.register_material(desc);

        assert_eq!(first, second);
        assert_eq!(state.materials_generation, 1);
        assert!(state.materials.contains_key(first));

        assert!(state.unregister_material(first));
        assert_eq!(state.materials_generation, 1);
        assert!(state.materials.contains_key(first));

        assert!(state.unregister_material(first));
        assert_eq!(state.materials_generation, 2);
        assert!(!state.materials.contains_key(first));
        assert!(state.materials_by_desc.is_empty());

        assert!(!state.unregister_material(first));
    }
}
