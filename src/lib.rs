#![cfg_attr(not(feature = "undefined_behavior"), feature(layout_for_ptr))]

#![cfg_attr(feature = "unstable", feature(
    const_raw_ptr_deref,
    const_mut_refs,
    const_align_of_val_raw,
    const_size_of_val_raw,
    const_slice_from_raw_parts,
))]

//! This crate is intended to provide data structures that use DSTs
//! (dynamically sized types).
//!
//! # Overview
//!
//! The goal of this crate is to provide data structures that can store DSTs
//! in a contiguous allocation. Some applications may have a performance
//! benefit for using a contiguous allocation as opposed to multiple pointers.
//!
//! Currently, the only implementation is a `Vec` where the DST-tails are
//! slices of the same length.

#[cfg(
    any(
        all(
            feature = "undefined_behavior",
            feature = "unstable",
        ),
        not(any(
            feature = "undefined_behavior",
            feature = "unstable",
        )),
    )
)]
compile_error!("Must have exactly one feature of \
    `undefined_behavior` or `unstable`. See \
    https://github.com/rust-lang/rust/issues/69835 (align/size of types) and \
    https://github.com/rust-lang/rust/issues/64490 or \
    https://github.com/rust-lang/rust/issues/73394 (field pointers).\
");

macro_rules! get_ix {
    ($T:tt $S:tt $s:ident $ix:expr) => {
        (
            get_ix!($T $S,
                if let Some((ptr, _)) = $s.ptr { ptr } else { std::hint::unreachable_unchecked() },
                $s.slice,
                $ix,
            )
        )
    };
    ($T:tt $S:tt, $ptr:expr, $len:expr, $ix:expr,) => {
        (
            std::ptr::slice_from_raw_parts_mut::<$S>(
                $ptr
                    .as_ptr()
                    .offset(($ix * Handle::<$T, [$S]>::size_slice($len)) as isize)
                    as *mut $S,
                $len,
            )
                as *mut Handle<$T, [$S]>
        )
    };
}

mod prelude;
mod vecs;
mod handle;
mod alloc;
mod util;

pub use handle::Handle;
pub use vecs::{
    fixed::{
        Vec as FixedVec,
        Iter as FixedVecIter,
        IterMut as FixedVecIterMut,
    },
};
