use fret_core::{MaterialDescriptor, MaterialId, MaterialRegistrationError, MaterialService};
use std::collections::HashMap;

/// App-owned catalog for renderer-registered visual primitives.
///
/// This is intentionally ecosystem-first: it provides a stable place to cache `MaterialId` handles
/// (and future visual IDs) without leaking backend handles into components.
///
/// Storage: app model / app global state.
///
/// Rationale:
/// - `MaterialId` values are renderer-owned and must be registered via `MaterialService`.
/// - keeping the cache app-owned avoids hidden global state and makes lifecycles explicit.
#[derive(Debug, Default)]
pub struct VisualCatalog {
    pub materials: MaterialCatalog,
}

/// App-owned cache for renderer-registered Tier B materials.
///
/// This caches `MaterialId` handles keyed by the portable `MaterialDescriptor` contract.
#[derive(Debug, Default)]
pub struct MaterialCatalog {
    by_desc: HashMap<MaterialDescriptor, MaterialId>,
}

impl MaterialCatalog {
    pub fn get(&self, desc: MaterialDescriptor) -> Option<MaterialId> {
        self.by_desc.get(&desc).copied()
    }

    pub fn get_or_register(
        &mut self,
        service: &mut dyn MaterialService,
        desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        if let Some(id) = self.get(desc) {
            return Ok(id);
        }
        let id = service.register_material(desc)?;
        self.by_desc.insert(desc, id);
        Ok(id)
    }

    pub fn unregister_all(&mut self, service: &mut dyn MaterialService) {
        for id in self.by_desc.values().copied() {
            let _ = service.unregister_material(id);
        }
        self.by_desc.clear();
    }

    pub fn clear_local(&mut self) {
        self.by_desc.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{MaterialKind, MaterialService};
    use slotmap::SlotMap;

    #[derive(Default)]
    struct FakeMaterialService {
        materials: SlotMap<MaterialId, MaterialDescriptor>,
        register_calls: u32,
        unregister_calls: u32,
    }

    impl MaterialService for FakeMaterialService {
        fn register_material(
            &mut self,
            desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            self.register_calls += 1;
            Ok(self.materials.insert(desc))
        }

        fn unregister_material(&mut self, id: MaterialId) -> bool {
            self.unregister_calls += 1;
            self.materials.remove(id).is_some()
        }
    }

    #[test]
    fn catalog_caches_by_descriptor() {
        let mut service = FakeMaterialService::default();
        let mut cat = MaterialCatalog::default();
        let desc = MaterialDescriptor::new(MaterialKind::DotGrid);

        let a = cat
            .get_or_register(&mut service, desc)
            .expect("register must succeed");
        let b = cat
            .get_or_register(&mut service, desc)
            .expect("cached register must succeed");
        assert_eq!(a, b);
        assert_eq!(service.register_calls, 1);
    }

    #[test]
    fn unregister_all_clears_cache() {
        let mut service = FakeMaterialService::default();
        let mut cat = MaterialCatalog::default();
        let d0 = MaterialDescriptor::new(MaterialKind::DotGrid);
        let d1 = MaterialDescriptor::new(MaterialKind::Noise);
        let _ = cat.get_or_register(&mut service, d0).unwrap();
        let _ = cat.get_or_register(&mut service, d1).unwrap();

        cat.unregister_all(&mut service);
        assert!(cat.by_desc.is_empty());
        assert_eq!(service.unregister_calls, 2);
    }
}
