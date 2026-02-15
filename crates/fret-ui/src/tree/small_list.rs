use super::*;
use std::mem::MaybeUninit;

/// # Safety
///
/// The caller must guarantee that every element in `slice` is initialized.
#[inline]
unsafe fn assume_init_slice_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    // SAFETY: `MaybeUninit<T>` has the same layout as `T`, and the caller guarantees initialization.
    //
    // Note: our pinned toolchain does not expose a standard-library helper for assuming init on
    // `&[MaybeUninit<T>]`, so we use the conventional `from_raw_parts` cast.
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<T>(), slice.len()) }
}

#[derive(Debug)]
pub(super) struct SmallNodeList<const N: usize> {
    len: usize,
    inline: [MaybeUninit<NodeId>; N],
    spill: Vec<NodeId>,
}

impl<const N: usize> Default for SmallNodeList<N> {
    fn default() -> Self {
        Self {
            len: 0,
            inline: [MaybeUninit::uninit(); N],
            spill: Vec::new(),
        }
    }
}

impl<const N: usize> SmallNodeList<N> {
    pub(super) fn set(&mut self, nodes: &[NodeId]) {
        if nodes.len() <= N {
            self.spill.clear();
            self.len = nodes.len();
            for (i, &id) in nodes.iter().enumerate() {
                self.inline[i].write(id);
            }
        } else {
            self.len = 0;
            self.spill.clear();
            self.spill.extend_from_slice(nodes);
        }
    }

    pub(super) fn as_slice(&self) -> &[NodeId] {
        if !self.spill.is_empty() {
            return self.spill.as_slice();
        }
        debug_assert!(self.len <= N);
        // SAFETY: when `spill` is empty, indices `0..len` are initialized via `set()`.
        unsafe { assume_init_slice_ref(&self.inline[..self.len]) }
    }
}

#[derive(Debug)]
pub(super) struct SmallCopyList<T: Copy, const N: usize> {
    len: usize,
    inline: [MaybeUninit<T>; N],
    spill: Vec<T>,
}

impl<T: Copy, const N: usize> Default for SmallCopyList<T, N> {
    fn default() -> Self {
        Self {
            len: 0,
            inline: [MaybeUninit::uninit(); N],
            spill: Vec::new(),
        }
    }
}

impl<T: Copy, const N: usize> SmallCopyList<T, N> {
    pub(super) fn push(&mut self, value: T) {
        if self.spill.is_empty() && self.len < N {
            self.inline[self.len].write(value);
            self.len += 1;
            debug_assert!(self.len <= N);
            return;
        }

        if self.spill.is_empty() {
            debug_assert!(self.len <= N);
            self.spill.reserve(self.len.saturating_add(1));
            // SAFETY: indices `0..len` are initialized while `spill` is empty.
            let inline = unsafe { assume_init_slice_ref(&self.inline[..self.len]) };
            self.spill.extend_from_slice(inline);
            self.len = 0;
        }

        self.spill.push(value);
    }

    pub(super) fn as_slice(&self) -> &[T] {
        if !self.spill.is_empty() {
            return self.spill.as_slice();
        }
        debug_assert!(self.len <= N);
        // SAFETY: indices `0..len` are initialized until we spill.
        unsafe { assume_init_slice_ref(&self.inline[..self.len]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn node(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    #[test]
    fn small_node_list_uses_inline_storage_for_small_slices() {
        let mut list: SmallNodeList<4> = SmallNodeList::default();
        let nodes = [node(1), node(2), node(3)];
        list.set(&nodes);

        assert!(list.spill.is_empty());
        assert_eq!(list.len, nodes.len());
        assert_eq!(list.as_slice(), nodes.as_slice());
    }

    #[test]
    fn small_node_list_spills_for_large_slices_and_can_return_to_inline() {
        let mut list: SmallNodeList<2> = SmallNodeList::default();

        let spilled = [node(10), node(11), node(12)];
        list.set(&spilled);
        assert_eq!(list.len, 0);
        assert_eq!(list.spill.as_slice(), spilled.as_slice());
        assert_eq!(list.as_slice(), spilled.as_slice());

        let inline = [node(20)];
        list.set(&inline);
        assert!(list.spill.is_empty());
        assert_eq!(list.len, inline.len());
        assert_eq!(list.as_slice(), inline.as_slice());
    }

    #[test]
    fn small_copy_list_stays_inline_until_full_and_then_spills_in_order() {
        let mut list: SmallCopyList<u32, 3> = SmallCopyList::default();

        list.push(1);
        list.push(2);
        list.push(3);
        assert!(list.spill.is_empty());
        assert_eq!(list.len, 3);
        assert_eq!(list.as_slice(), &[1, 2, 3]);

        list.push(4);
        assert_eq!(list.len, 0);
        assert_eq!(list.spill.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(list.as_slice(), &[1, 2, 3, 4]);

        list.push(5);
        assert_eq!(list.spill.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(list.as_slice(), &[1, 2, 3, 4, 5]);
    }
}
