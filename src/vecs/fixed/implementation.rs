use std::{
    alloc::{
        alloc,
        handle_alloc_error,
        Layout,
        realloc,
    },
    collections::Bound,
    hint::unreachable_unchecked,
    num::NonZeroUsize,
    ops::{
        Range,
        RangeBounds,
    },
    ptr::{
        drop_in_place,
        NonNull,
        read,
    },
};
use crate::{
    Handle,
    util::{
        can_try_alloc,
        CopyFn,
        CopyNonoverlappingFn,
        DefaultIter,
        new_capacity_at_least_double,
        PtrCopy,
    },
};
use super::*;

#[inline(always)]
fn convert_range(len: &usize, range: impl RangeBounds<usize>) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Included(ix) => *ix,
        Bound::Excluded(ix) => ix.saturating_add(1),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(ix) => ix.saturating_add(1),
        Bound::Excluded(ix) => *ix,
        Bound::Unbounded => *len,
    };
    start..end
}

impl<T, S> Vec<T, S> {
    /// Creates a new [`Vec`] that can contain items where the tail length is
    /// as provided. Will not allocate until an item is inserted or capacity
    /// reserved.
    pub fn new(slice_length: usize) -> Self {
        let size = Handle::<T, [S]>::size_slice(slice_length);
        assert_ne!(size, 0, "Zero-sized DST is pointless");

        Vec {
            ptr: None,
            length: 0,
            slice: slice_length,
            _phantom: Default::default(),
        }
    }

    /// Returns the maximum number of items before a reallocation is needed.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        if let Some((_, value)) = self.ptr {
            value.get()
        } else {
            0
        }
    }

    /// Returns the length of the tail for any/all items.
    #[inline(always)]
    pub fn slice_length(&self) -> usize {
        self.slice
    }

    /// Will insure it has enough space for the specified number of items,
    /// growing according to an internal criteria.
    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.capacity();
        if capacity - self.length >= additional {
            return;
        }
        self.alloc_grow(new_capacity_at_least_double(if additional <= capacity {
            // it at-least doubles, which means it will be at-least `additional`
            capacity
        } else {
            // This is the lowest number that is at-least half of the desired capacity.
            // Thus, at-least double provides at-least the desired capacity.
            additional
                .checked_add(self.length)
                .and_then(|v| v.checked_add(1))
                .expect("Overflow")
                / 2
        }).expect("Overflow"))
    }

    /// Will allocate exactly enough memory to insure it has enough space for
    /// the specified number of elements, or do nothing if it has already
    /// allocated enough.
    pub fn reserve_exact(&mut self, additional: usize) {
        if self.capacity() - self.length >= additional {
            return;
        }
        // X >= Z is always true when Z == 0, which would return
        // Y + Z > 0 is always true when Z != 0
        self.alloc_grow(unsafe { NonZeroUsize::new_unchecked(
            self.length.checked_add(additional).expect("Overflow")
        )})
    }

    pub(super) fn alloc_grow(&mut self, total: NonZeroUsize) {
        let layout = Handle::<T, [S]>::layout_slice(self.slice, total);
        if !can_try_alloc(layout.size()) {
            panic!("Overflow");
        }

        let ptr = if let Some((ptr, capacity)) = self.ptr {
            let old_layout = Handle::<T, [S]>::layout_slice(self.slice, capacity);
            unsafe {
                realloc(ptr.as_ptr(), old_layout, layout.size())
            }
        } else {
            unsafe {
                alloc(layout)
            }
        };

        if let Some(ptr) = NonNull::new(ptr) {
            self.ptr = Some((ptr, total))
        } else {
            handle_alloc_error(layout)
        }
    }

    fn do_push(&mut self, value: T, slice: impl IntoIterator<Item=S>) -> &mut Handle<T, [S]> {
        self.reserve(1);
        let mut slice = slice.into_iter();

        let handle = unsafe { get_ix!(T S self self.length) };
        Handle::populate(handle, self.slice..(self.slice + 1), value, &mut slice);

        // This puts it in the drop
        self.length += 1;

        unsafe { &mut *handle }
    }

    fn do_insert(&mut self, ix: usize, value: T, slice: impl IntoIterator<Item=S>) -> &mut Handle<T, [S]> {
        if ix > self.length {
            panic!("Out of bounds insert");
        }
        self.reserve(1);
        let ptr = if let Some((ptr, _)) = self.ptr {
            ptr
        } else {
            // At-least 1 space has been allocated.
            unsafe { unreachable_unchecked() }
        };
        let mut slice = slice.into_iter();

        let old_len = self.length;
        // Pre-poop the pants
        self.length = ix;

        self.shift_memory::<CopyFn>(ptr, ix + 1, ptr, ix, old_len - ix);
        let handle = unsafe { get_ix!(T S self self.length) };
        Handle::populate(handle, self.slice..(self.slice + 1), value, &mut slice);

        // Clean the pants
        self.length = old_len + 1;

        unsafe { &mut *handle }
    }

    /// Adds an item using an iterator containing at least enough values to
    /// populate the DST's slice.
    ///
    /// # Panics
    ///
    /// Panics if the iterator has insufficient element count.
    #[inline(always)]
    pub fn push(&mut self, value: T, slice: impl IntoIterator<Item=S>) {
        self.do_push(value, slice);
    }

    /// Inserts an item using an iterator containing at least enough values to
    /// populate the DST's slice.
    ///
    /// # Panics
    ///
    /// Panics if the iterator has insufficient element count. Panics if any
    /// index lower than the one provided has no item.
    #[inline(always)]
    pub fn insert(&mut self, ix: usize, value: T, slice: impl IntoIterator<Item=S>) {
        self.do_insert(ix, value, slice);
    }

    /// Removes the last inserted element as if it was immediately dropped.
    ///
    /// # Panics
    ///
    /// Panics if there are no items.
    pub fn pop(&mut self) {
        if self.length == 0 {
            panic!("No value to remove");
        }
        self.length -= 1;
        let handle = unsafe { get_ix!(T S self self.length) };
        unsafe { drop_in_place(handle) }
    }

    /// Removes the last inserted element and returns it.
    ///
    /// # Panics
    ///
    /// Panics if there are no items.
    pub fn pop_boxed(&mut self) -> Box<Handle<T, [S]>> {
        if self.length == 0 {
            panic!("No value to remove");
        }
        self.length -= 1;
        unsafe {
            let ptr = match self.ptr {
                None => unreachable_unchecked(),
                Some((ptr, _)) => ptr,
            };
            let layout = Layout::for_value(&*get_ix!(T S, ptr, self.slice, self.length,));

            let target = alloc(layout);
            let target = if let Some(target) = NonNull::new(target) {
                target
            } else {
                handle_alloc_error(layout);
            };
            self.shift_memory::<CopyNonoverlappingFn>(target, 0, ptr, self.length, 1);
            let target = get_ix!(T S, target, self.slice, 0,);
            Box::from_raw(target)
        }
    }

    /// Removes the last inserted element as if the slice part was immediately
    /// dropped, but returning the value.
    ///
    /// # Panics
    ///
    /// Panics if there are no items.
    pub fn pop_value(&mut self) -> T {
        if self.length == 0 {
            panic!("No value to remove");
        }
        self.length -= 1;

        let handle = unsafe { &mut *get_ix!(T S self self.length) };
        let slice = &mut handle.tail as *mut [S];
        let value = &handle.value as *const T;
        unsafe {
            drop_in_place(slice);
            read(value)
        }
    }

    /// Checks if there are any items.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns a reference, or `None` if out of bounds.
    #[inline(always)]
    pub fn get(&self, ix: usize) -> Option<&Handle<T, [S]>> {
        if ix >= self.length {
            None
        } else {
            Some(unsafe { self.get_unchecked(ix) })
        }
    }

    /// Returns a reference without bound-checking.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, ix: usize) -> &Handle<T, [S]> {
        &*get_ix!(T S self ix)
    }

    /// Returns a mutable reference, or `None` if out of bounds.
    #[inline(always)]
    pub fn get_mut(&mut self, ix: usize) -> Option<&mut Handle<T, [S]>> {
        if ix >= self.length {
            None
        } else {
            Some(unsafe { self.get_unchecked_mut(ix) })
        }
    }

    /// Returns a mutable reference without bound-checking.
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, ix: usize) -> &mut Handle<T, [S]> {
        &mut *get_ix!(T S self ix)
    }

    /// Returns an iterator that provides references.
    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, T, S> {
        self.into_iter()
    }

    /// Returns an iterator that provides mutable references.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, S> {
        self.into_iter()
    }

    fn remove_drop_range_impl(&mut self, ix: Range<usize>, mode: RemovalMode) {
        if ix.is_empty() {
            return;
        }
        if ix.start >= self.length || ix.end > self.length {
            panic!("Out of bounds");
        }
        let ptr = if let Some((ptr, _)) = self.ptr {
            ptr
        } else {
            // Self.length == 0 implies "Out of bounds"
            // Otherwise, there must exist an allocation
            unsafe { unreachable_unchecked() }
        };

        let old_len = self.length;
        // Pre-poop the pants
        self.length = ix.start;

        for ix in ix.clone() {
            let handle = unsafe { get_ix!(T S self ix) };
            unsafe { drop_in_place(handle) };
        }

        if mode == RemovalMode::Shift
            && old_len - ix.end > ix.len()
        {
            self.shift_memory::<CopyFn>(ptr, ix.start, ptr, ix.end, old_len - ix.end);
        } else {
            self.shift_memory::<CopyNonoverlappingFn>(ptr, ix.start, ptr, old_len - ix.len(), ix.len());
        }
        // Clean the pants
        self.length = old_len - ix.len();
    }

    /// Items the item at the index, shifting any later items.
    ///
    /// # Panics
    ///
    /// Panics if there is no item at the index.
    #[inline(always)]
    pub fn remove(&mut self, ix: usize) {
        self.remove_drop_range_impl(ix..(ix.saturating_add(1)), RemovalMode::Shift);
    }

    /// Removes the range of items, shifting any later items.
    ///
    /// # Panics
    ///
    /// Panics if any index as specified by a non-open bound in the range has
    /// no item.
    #[inline(always)]
    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) {
        self.remove_drop_range_impl(convert_range(&self.length, range), RemovalMode::Shift);
    }

    /// Removes the item at the index, replacing it with the last item.
    /// This avoids excessive copies, but by reordering.
    ///
    /// # Panics
    ///
    /// Panics if there is no item at the index.
    #[inline(always)]
    pub fn remove_replace(&mut self, ix: usize) {
        self.remove_drop_range_impl(ix..(ix.saturating_add(1)), RemovalMode::Replace);
    }

    /// Removes the range of items, replacing them with items from the end.
    /// This avoids excessive copies, but by reordering.
    ///
    /// # Panics
    ///
    /// Panics if any index as specified by a non-open bound in the range has
    /// no item.
    #[inline(always)]
    pub fn remove_range_replace(&mut self, range: impl RangeBounds<usize>) {
        self.remove_drop_range_impl(convert_range(&self.length, range), RemovalMode::Replace);
    }

    /// Removes the item at index as if the slice part was immediately dropped,
    /// but returning the value.
    ///
    /// # Panics
    ///
    /// Panics if there is no item at the index.
    pub fn remove_value(&mut self, ix: usize) -> T {
        if ix >= self.length {
            panic!("No value to remove");
        }
        let ptr = if let Some((ptr, _)) = self.ptr {
            ptr
        } else {
            // Self.length == 0 implies "No value to remove"
            // Otherwise, there must exist an allocation
            unsafe { unreachable_unchecked() }
        };

        let old_len = self.length;
        // Pre-poop the pants, and bonus it lets us re-use code
        self.length = ix + 1;
        let value = self.pop_value();

        self.shift_memory::<CopyFn>( ptr, ix, ptr, ix + 1, old_len - ix - 1);
        // Clean the pants
        self.length = old_len - 1;
        value
    }

    #[inline(always)]
    pub(super) fn shift_memory<F: PtrCopy>(
        &self,
        dst: NonNull<u8>,
        to: usize,
        src: NonNull<u8>,
        from: usize,
        length: usize,
    ) {
        let size = Handle::<T, [S]>::size_slice(self.slice);
        unsafe { F::copy(
            src.as_ptr().offset((from * size) as isize),
            dst.as_ptr().offset((to * size) as isize),
            size * length,
        ) };
    }

    /// This moves values from the specified [`Vec`] to this one.
    ///
    /// # Panics
    ///
    /// Panics if the values in the other [`Vec`] do not have the same
    /// tail-length. Panics if any of the other's indexes as specified by a
    /// non-open bound in the range has no item.
    #[inline(always)]
    pub fn move_from(&mut self, other: &mut Self, range: impl RangeBounds<usize>) {
        self.move_from_impl(other, convert_range(&other.length, range));
    }

    fn move_from_impl(&mut self, other: &mut Self, range: Range<usize>) {
        if self.slice != other.slice {
            panic!("Length mismatch");
        }
        if range.is_empty() {
            return;
        }
        if range.start >= other.length || range.end > other.length {
            panic!("Out of bounds");
        }
        self.reserve(range.len());

        let self_ptr = if let Some((ptr, _)) = self.ptr {
            ptr
        } else {
            // At least 1 space has been reserved
            unsafe { unreachable_unchecked() }
        };
        let other_ptr = if let Some((ptr, _)) = self.ptr {
            ptr
        } else {
            // Other.length == 0 implies "Out of bounds"
            // Otherwise, there must exist an allocation
            unsafe { unreachable_unchecked() }
        };

        let old_length = other.length;
        // Pre-poop the pants
        other.length = range.start;
        self.shift_memory::<CopyNonoverlappingFn>(self_ptr, self.length, other_ptr, other.length, range.len());
        self.length += range.len();
        self.shift_memory::<CopyFn>(other_ptr, range.start, other_ptr, range.end, old_length - range.end);
        // Clean the pants
        other.length = old_length - range.len();
    }
}

impl<T, S: Default> Vec<T, S> {
    /// Appends at the end using default to populate the slice.
    #[inline(always)]
    pub fn push_default_slice(&mut self, value: T) -> &mut Handle<T, [S]> {
        self.do_push(value, DefaultIter::default())
    }

    /// Inserts at the index using default to populate the slice.
    ///
    /// # Panics
    ///
    /// Panics if any index lower than the one provided has no item.
    #[inline(always)]
    pub fn insert_default_slice(&mut self, ix: usize, value: T) -> &mut Handle<T, [S]> {
        self.do_insert(ix, value, DefaultIter::default())
    }
}

impl<T: Default, S: Default> Vec<T, S> {
    /// Appends at the end using default to populate the value and slice.
    #[inline(always)]
    pub fn push_default(&mut self) -> &mut Handle<T, [S]> {
        self.do_push(T::default(), DefaultIter::default())
    }

    /// Inserts at the index using default to populate the value and slice.
    ///
    /// # Panics
    ///
    /// Panics if any index lower than the one provided has no item.
    #[inline(always)]
    pub fn insert_default(&mut self, ix: usize) -> &mut Handle<T, [S]> {
        self.do_insert(ix, T::default(), DefaultIter::default())
    }
}
