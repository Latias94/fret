pub struct Memo<TDeps, TValue> {
    deps: Option<TDeps>,
    value: Option<TValue>,
}

impl<TDeps, TValue> Default for Memo<TDeps, TValue> {
    fn default() -> Self {
        Self {
            deps: None,
            value: None,
        }
    }
}

impl<TDeps: PartialEq, TValue> Memo<TDeps, TValue> {
    pub fn get_or_compute(&mut self, deps: TDeps, f: impl FnOnce() -> TValue) -> (&TValue, bool) {
        let should_recompute = self.deps.as_ref().map_or(true, |d| d != &deps);
        if should_recompute {
            self.deps = Some(deps);
            self.value = Some(f());
        }
        (self.value.as_ref().expect("memo value"), should_recompute)
    }

    pub fn reset(&mut self) {
        self.deps = None;
        self.value = None;
    }
}
