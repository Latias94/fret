#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

/// Take two owned values out of runner slots, restoring the first if the second is unavailable.
pub(crate) fn take_slot_pair<A, B>(
    first_slot: &mut Option<A>,
    second_slot: &mut Option<B>,
) -> Option<(A, B)> {
    let first = first_slot.take()?;
    let Some(second) = second_slot.take() else {
        *first_slot = Some(first);
        return None;
    };
    Some((first, second))
}

pub(crate) trait SlotPairOwner<A, B> {
    fn take_slot_pair(&mut self) -> Option<(A, B)>;
    fn restore_slot_pair(&mut self, first: A, second: B);
}

/// Run work with a temporarily owned slot pair and always restore it afterward.
pub(crate) fn with_slot_pair_restored<O, A, B, R>(
    owner: &mut O,
    f: impl FnOnce(&mut O, &mut A, &mut B) -> R,
) -> Option<R>
where
    O: SlotPairOwner<A, B>,
{
    let (mut first, mut second) = owner.take_slot_pair()?;
    let result = f(owner, &mut first, &mut second);
    owner.restore_slot_pair(first, second);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DummyOwner {
        first: Option<u32>,
        second: Option<String>,
        log: Vec<&'static str>,
    }

    impl SlotPairOwner<u32, String> for DummyOwner {
        fn take_slot_pair(&mut self) -> Option<(u32, String)> {
            self.log.push("take");
            take_slot_pair(&mut self.first, &mut self.second)
        }

        fn restore_slot_pair(&mut self, first: u32, second: String) {
            self.log.push("restore");
            self.first = Some(first);
            self.second = Some(second);
        }
    }

    #[test]
    fn take_slot_pair_rolls_back_first_when_second_is_missing() {
        let mut first = Some(7u32);
        let mut second = None::<String>;

        assert_eq!(take_slot_pair(&mut first, &mut second), None);
        assert_eq!(first, Some(7));
        assert_eq!(second, None);
    }

    #[test]
    fn with_slot_pair_restored_puts_mutated_values_back() {
        let mut owner = DummyOwner {
            first: Some(1),
            second: Some("web".to_string()),
            log: Vec::new(),
        };

        let result = with_slot_pair_restored(&mut owner, |owner, first, second| {
            assert!(owner.first.is_none());
            assert!(owner.second.is_none());
            *first = first.saturating_add(1);
            second.push_str("-frame");
            42usize
        });

        assert_eq!(result, Some(42));
        assert_eq!(owner.first, Some(2));
        assert_eq!(owner.second.as_deref(), Some("web-frame"));
        assert_eq!(owner.log, vec!["take", "restore"]);
    }

    #[test]
    fn with_slot_pair_restored_skips_callback_when_pair_is_unavailable() {
        let mut owner = DummyOwner {
            first: Some(3),
            second: None,
            log: Vec::new(),
        };
        let mut called = false;

        let result = with_slot_pair_restored(&mut owner, |_owner, _first, _second| {
            called = true;
        });

        assert_eq!(result, None);
        assert!(!called);
        assert_eq!(owner.first, Some(3));
        assert_eq!(owner.second, None);
        assert_eq!(owner.log, vec!["take"]);
    }
}
