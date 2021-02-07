use std::marker::PhantomData;
use crate::{
    alloc::Alloc,
    Handle,
};

mod traits;
mod iter;
mod implementation;

pub use iter::*;

/// Imitates a [`std::vec::Vec`] of a slice-based DST. All values have the
/// same slice length, which allows random-access. Guarantied to store all
/// values inline in the same allocation (sequentially), only using extra
/// space for inherent alignment-padding. The trade-off is that the
/// slice-lengths of the dynamically sized tail must all be the same, and is
/// stored alongside the pointer to the allocation/length/capacity. Thus, this
/// [`Vec`] is 4-words instead of 3-words.
///
/// # Panics / Aborts
///
/// Any operation that may increase the capacity will abort if there is a
/// failure to allocate, or panic if the [`usize`] math overflows beforehand.
///
/// # Uses
///
/// If your application knows at compile-time the length of a slice, you
/// should use `Vec<(T, [S; LENGTH])>`. Read no further.
///
/// However, the length may not be known until runtime. In this scenario, a
/// `Vec<(T, Vec<S>)>` or `Vec<(T, Box<[S]>)>` should be used instead, the
/// latter verifiably not wasting any space.
///
/// In the case of the length only known at runtime (like reading in columns
/// of a `.csv` file), those solutions may show evidence of degraded
/// performance from the random memory access patterns of said pointers. By
/// opting to use this implementation, there is a guaranty that all of the
/// data is contained in a single allocation, as-is the case of the
/// constant-length slice. Some applications may receive a performance benefit
/// by arranging the data in this way, while others may lose performance from
/// the overhead of this implementation.
///
/// # Usage
///
/// ```rust
/// use dst::FixedVec;
///
/// let mut vec = FixedVec::<Option<&str>, usize>::new(4);
/// vec.push_default();
/// let item = vec.get(0).unwrap();
/// assert_eq!(item.value, None);
/// assert_eq!(item.tail, [0, 0, 0, 0]);
///
/// vec.push(Some("Name"), [1, 2, 3, 4].iter().copied());
/// let item = vec.get(1).unwrap();
/// assert_eq!(item.value, Some("Name"));
/// assert_eq!(item.tail, [1, 2, 3, 4]);
/// ```

pub struct Vec<T, S> {
    ptr: Alloc,
    length: usize,
    slice: usize,
    _phantom: PhantomData<Handle<T, [S]>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum RemovalMode {
    /// Shifts entire tail to cover
    Shift,
    /// Reorders with the tail
    Replace,
}

#[cfg(test)]
mod test {
    use super::*;

    #[fn_fixture::snapshot("snapshot-tests/csv")]
    fn csv_to_vec(file: &str) -> Vec<(), &str> {
        let columns = file.lines().next().unwrap().split(',').count();

        let mut vec = Vec::new(columns);
        for line in file.lines().skip(1) {
            let line = line.trim_end();
            vec.push((), line.split(','));
        }
        vec
    }
}
