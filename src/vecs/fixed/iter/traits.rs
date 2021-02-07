use crate::prelude::*;
use super::{
    *,
    super::Handle,
};

impl<'a, T, S> From<IterMut<'a, T, S>> for Iter<'a, T, S> {
    fn from(iter: IterMut<'a, T, S>) -> Self {
        Iter {
            iter: iter.iter,
            ptr: &*iter.ptr,
        }
    }
}

impl<'a, T, S> Clone for Iter<'a, T, S> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
            ptr: self.ptr,
        }
    }
}

impl<T: Debug, S: Debug> Debug for Iter<'_, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f
            .debug_list()
            .entries(self.clone())
            .finish()
    }
}

impl<TL: PartialEq<TR>, TR, SL: PartialEq<SR>, SR> PartialEq<Iter<'_, TR, SR>> for Iter<'_, TL, SL> {
    fn eq(&self, other: &Iter<'_, TR, SR>) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        loop {
            match (left.next(), right.next()) {
                (Some(left), Some(right)) if left == right => {},
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

impl<T: Eq, S: Eq> Eq for Iter<'_, T, S> {}

impl<T: Hash, S: Hash> Hash for Iter<'_, T, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in self.clone() {
            value.hash(state);
        }
    }
}

impl<'a, T, S> Iterator for Iter<'a, T, S> {
    type Item = &'a Handle<T, [S]>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self
            .iter
            .next()
            .map(|ix| unsafe { self.ptr.get_unchecked(ix) })
    }
}

impl<'a, T, S> Iterator for IterMut<'a, T, S> {
    type Item = &'a mut Handle<T, [S]>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self
            .iter
            .next()
            .map(|ix| unsafe {
                // Cheating the lifetime
                &mut *(self.ptr.get_unchecked_mut(ix) as *mut _)
            })
    }
}
