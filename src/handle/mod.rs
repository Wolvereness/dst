mod traits;
mod implementation;

/// This struct contains a value and the unsized-tail. It cannot be
/// normally instantiated.
pub struct Handle<V, T: ?Sized> {
    pub value: V,
    pub tail: T,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_check() {
        use std::mem::size_of;

        type T = (String, Vec<()>, Vec<usize>);
        for len in 0..4 {
            assert_eq!(
                size_of::<&str>() * len,
                Handle::<(), [&str]>::size_slice(len),
            );
            assert_eq!(
                size_of::<&str>() * len + size_of::<T>(),
                Handle::<T, [&str]>::size_slice(len),
            );
        }
    }
}
