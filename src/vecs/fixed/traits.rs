use std::{
    alloc::dealloc,
    hint::unreachable_unchecked,
};
use crate::{
    Handle,
    prelude::*,
    util::CopyNonoverlappingFn,
};
use super::*;

unsafe impl<T, S> Send for Vec<T, S> where Handle<T, S>: Send {}
unsafe impl<T, S> Sync for Vec<T, S> where Handle<T, S>: Sync {}

impl<TL: PartialEq<TR>, TR, SL: PartialEq<SR>, SR> PartialEq<Vec<TR, SR>> for Vec<TL, SL> {
    fn eq(&self, other: &Vec<TR, SR>) -> bool {
        self.length == other.length
            && self.iter() == other.iter()
    }
}

impl<T: Eq, S: Eq> Eq for Vec<T, S> {}

impl<T: Hash, S: Hash> Hash for Vec<T, S> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().hash(state)
    }
}

impl<T: Debug, S: Debug> Debug for Vec<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f
            .debug_struct("Vec")
            .field("values", &self.into_iter())
            .field("capacity", &self.ptr
                .map(|(_, capacity)| capacity.get())
                .unwrap_or_default()
            )
            .finish()
    }
}

impl<'a, T, S> IntoIterator for &'a Vec<T, S> {
    type Item = &'a Handle<T, [S]>;
    type IntoIter = Iter<'a, T, S>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: 0..(self.length),
            ptr: self,
        }
    }
}

impl<'a, T, S> IntoIterator for &'a mut Vec<T, S> {
    type Item = &'a mut Handle<T, [S]>;
    type IntoIter = IterMut<'a, T, S>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: 0..(self.length),
            ptr: self,
        }
    }
}

impl<T, S> Drop for Vec<T, S> {
    fn drop(&mut self) {
        if !self.ptr.is_some() {
            return;
        }
        while !self.is_empty() {
            self.pop()
        }
        if let Some((ptr, capacity)) = self.ptr.take() {
            unsafe { dealloc(
                ptr.as_ptr(),
                Handle::<T, [S]>::layout_slice(self.slice, capacity),
            ) }
        }

    }
}

impl<T: Copy, S: Copy> Clone for Vec<T, S> {
    fn clone(&self) -> Self {
        let &Vec {
            ptr,
            length,
            slice,
            _phantom,
        } = self;
        let mut new = Self::new(slice);
        if let Some((ptr, capacity)) = ptr {
            new.alloc_grow(capacity);
            if let Some((new_ptr, _)) = new.ptr {
                new.shift_memory::<CopyNonoverlappingFn>(new_ptr, 0, ptr, 0, length);
            } else {
                // alloc_grow insures ptr populated
                unsafe { unreachable_unchecked() }
            }
            new.length = length;
        }
        new
    }
}