use std::ops::Range;
use super::Vec;

mod traits;

pub struct Iter<'a, V, T> {
    pub(super) iter: Range<usize>,
    pub(super) ptr: &'a Vec<V, T>,
}

pub struct IterMut<'a, V, T> {
    pub(super) iter: Range<usize>,
    pub(super) ptr: &'a mut Vec<V, T>,
}
