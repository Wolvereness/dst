use std::{
    marker::PhantomData,
    num::NonZeroUsize,
    ptr,
};

#[derive(Default, Copy, Clone)]
pub struct DefaultIter<T: Default>(PhantomData<fn() -> T>);

impl<T: Default> Iterator for DefaultIter<T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        Some(T::default())
    }
}

#[inline(always)]
pub fn can_try_alloc(alloc_size: usize) -> bool {
    alloc_size <= isize::MAX as usize
}

pub fn new_capacity_at_least_double(old_capacity: usize) -> Option<NonZeroUsize> {
    // Smallest number X, such that X is a power of 2 and X >= 2 * old capacity
    // 00001 -> 00010 -> 00100
    // 00011 -> 01000 -> 10000
    // 00101 -> 10000
    // old_capacity is never zero, so -1 is safe
    let leading_zeros = (old_capacity - 1).leading_zeros() - 1;
    if leading_zeros == 0 {
        // Any value in the inclusive range: [usize::max / 4 + 2, usize::max / 2]
        None
    } else {
        NonZeroUsize::new(
            1usize.rotate_right(leading_zeros)
        )
    }
}

pub trait PtrCopy {
    unsafe fn copy(src: *const u8, dst: *mut u8, count: usize);
}

pub enum CopyFn {}
impl PtrCopy for CopyFn {
    #[inline(always)]
    unsafe fn copy(src: *const u8, dst: *mut u8, count: usize) {
        ptr::copy(src, dst, count);
    }
}

pub enum CopyNonoverlappingFn {}
impl PtrCopy for CopyNonoverlappingFn {
    #[inline(always)]
    unsafe fn copy(src: *const u8, dst: *mut u8, count: usize) {
        ptr::copy_nonoverlapping(src, dst, count);
    }
}
