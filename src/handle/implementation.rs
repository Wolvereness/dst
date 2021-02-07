use std::{
    alloc::Layout,
    mem,
    num::NonZeroUsize,
    ops::Range,
    ptr::{
        null,
        slice_from_raw_parts,
        write,
    },
};
use crate::util::can_try_alloc;
use super::Handle;

impl<T, S> Handle<T, [S]> {
    /// Returns the number of items inserted.
    ///
    /// # Panics
    ///
    /// When the number of items provided by the iterator is not enough to fit
    /// in the provided range `slice_len`.
    pub(crate) fn populate(
        handle: *mut Self,
        slice_len: Range<usize>,
        value: T,
        slice: &mut impl Iterator<Item=S>,
    ) -> usize {
        unsafe { write(Self::value_ptr(handle), value) };
        Self::extend(handle, slice_len, 0, slice)
    }

    /// Returns the *total* number of items inserted, including previous.
    ///
    /// # Panics
    ///
    /// When the number of items provided by the iterator is not enough to fit
    /// in the provided range `slice_len`.
    pub(crate) fn extend(
        handle: *mut Self,
        slice_len: Range<usize>,
        start: usize,
        slice: &mut impl Iterator<Item=S>,
    ) -> usize {
        let slice_values = Self::tail_ptr(handle) as *mut S;

        for offset in start..(slice_len.end) {
            if let Some(value) = slice.next() {
                unsafe {
                    write(
                        slice_values.offset(offset as isize),
                        value,
                    )
                }
            } else {
                if !slice_len.contains(&offset) {
                    panic!("Not enough values to populate handle")
                }
                return offset;
            }
        }
        slice_len.end
    }

    pub(crate) fn layout_slice(slice_size: usize, count: NonZeroUsize) -> Layout {
        let size = Self::size_slice(slice_size)
            .checked_mul(count.get())
            .expect("Overflow");
        if !can_try_alloc(size) {
            panic!("Overflow");
        }
        Layout::from_size_align(
            size,
            Self::alignment_slice(slice_size),
        )
            .expect("Bad Layout")
    }
}

#[cfg(feature = "unstable")]
impl<T, S> Handle<T, [S]> {
    #[inline(always)]
    pub(crate) const fn null_ptr_slice(slice_size: usize) -> *const Self {
        slice_from_raw_parts::<S>(null(), slice_size)
            as *const Self
    }
}

#[cfg(not(feature = "unstable"))]
impl<T, S> Handle<T, [S]> {
    #[inline(always)]
    pub(crate) fn null_ptr_slice(slice_size: usize) -> *const Self {
        slice_from_raw_parts::<S>(null(), slice_size)
            as *const Self
    }
}

#[cfg(all(not(feature = "undefined_behavior"), feature = "unstable"))]
impl<V, T: ?Sized> Handle<V, T> {
    #[inline(always)]
    pub(crate) const fn tail_ptr(handle: *mut Self) -> *mut T {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/51911
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/57349
        unsafe {
            std::ptr::addr_of_mut!((*handle).tail) as *mut T
        }
    }

    #[inline(always)]
    pub(crate) const fn value_ptr(handle: *mut Self) -> *mut V {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/51911
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/57349
        unsafe {
            std::ptr::addr_of_mut!((*handle).value) as *mut V
        }
    }

    #[inline(always)]
    pub(crate) const fn size(ptr: *const Self) -> usize {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/46571
        unsafe {
            mem::size_of_val_raw::<Self>(ptr)
        }
    }

    #[inline(always)]
    pub(crate) const fn alignment(ptr: *const Self) -> usize {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/46571
        unsafe {
            mem::align_of_val_raw::<Self>(ptr)
        }
    }
}

#[cfg(all(not(feature = "undefined_behavior"), feature = "unstable"))]
impl<T, S> Handle<T, [S]> {
    #[inline(always)]
    pub(crate) const fn size_slice(slice_size: usize) -> usize {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/46571
        Self::size(Self::null_ptr_slice(slice_size))
    }

    #[inline(always)]
    pub(crate) const fn alignment_slice(slice_size: usize) -> usize {
        // TODO: unstable pending https://github.com/rust-lang/rust/issues/46571
        Self::alignment(Self::null_ptr_slice(slice_size))
    }
}

#[cfg(feature = "undefined_behavior")]
impl<V, T: ?Sized> Handle<V, T> {
    #[inline(always)]
    pub(crate) fn tail_ptr(handle: *mut Self) -> *mut T {
        unsafe {
            // FIXME: UB, pending https://github.com/rust-lang/rust/issues/64490 or https://github.com/rust-lang/rust/issues/73394
            &mut (*handle).tail as *mut T
        }
    }

    #[inline(always)]
    pub(crate) fn value_ptr(handle: *mut Self) -> *mut V {
        unsafe {
            // FIXME: UB, pending https://github.com/rust-lang/rust/issues/64490 or https://github.com/rust-lang/rust/issues/73394
            &mut (*handle).value as *mut V
        }
    }
    #[inline(always)]
    pub(crate) fn alignment(ptr: *const Self) -> usize {
        // FIXME: UB, pending https://github.com/rust-lang/rust/issues/69835
        mem::align_of_val(unsafe {&*ptr})
    }

    #[inline(always)]
    pub(crate) fn size(ptr: *const Self) -> usize {
        // FIXME: UB, pending https://github.com/rust-lang/rust/issues/69835
        mem::size_of_val(unsafe {&*ptr})
    }
}

#[cfg(feature = "undefined_behavior")]
impl<T, S> Handle<T, [S]> {
    #[inline(always)]
    pub(crate) fn alignment_slice(slice_size: usize) -> usize {
        // FIXME: UB, pending https://github.com/rust-lang/rust/issues/69835
        Self::alignment(Self::null_ptr_slice(slice_size))
    }

    #[inline(always)]
    pub(crate) fn size_slice(slice_size: usize) -> usize {
        // FIXME: UB, pending https://github.com/rust-lang/rust/issues/69835
        Self::size(Self::null_ptr_slice(slice_size))
    }
}
