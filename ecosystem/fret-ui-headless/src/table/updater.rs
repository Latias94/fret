use std::sync::Arc;

/// TanStack-style updater: either a value replacement or a function of the previous value.
#[derive(Clone)]
pub enum Updater<T> {
    Value(T),
    Func(Arc<dyn Fn(&T) -> T + Send + Sync>),
}

impl<T> Updater<T> {
    pub fn apply(&self, old: &T) -> T
    where
        T: Clone,
    {
        match self {
            Self::Value(v) => v.clone(),
            Self::Func(f) => (f)(old),
        }
    }
}

impl<T> From<T> for Updater<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

pub fn functional_update<T>(old: &T, updater: &Updater<T>) -> T
where
    T: Clone,
{
    updater.apply(old)
}
