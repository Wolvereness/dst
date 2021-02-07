use std::{
    num::NonZeroUsize,
    ptr::NonNull,
};

type PtrCapPair = (NonNull<u8>, NonZeroUsize);
pub type Alloc = Option<PtrCapPair>;

#[cfg(test)]
mod test {
    use std::mem;
    use super::*;

    #[test]
    fn same_size() {
        assert_eq!(
            mem::size_of::<PtrCapPair>(),
            mem::size_of::<Alloc>(),
        )
    }

    #[test]
    fn same_align() {
        assert_eq!(
            mem::align_of::<PtrCapPair>(),
            mem::align_of::<Alloc>(),
        )
    }
}
