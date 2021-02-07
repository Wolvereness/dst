
[![Crates.io version](https://img.shields.io/crates/v/dst.svg)](https://crates.io/crates/dst)
[![docs.rs status](https://docs.rs/dst/badge.svg)](https://docs.rs/dst)
[![Crates.io license](https://img.shields.io/crates/l/dst.svg)](https://crates.io/crates/dst)
![Github Tests](https://github.com/Wolvereness/dst/workflows/Rust/badge.svg)

This crate is intended to provide data structures that use DSTs
(dynamically sized types).

# Overview

The goal of this crate is to provide data structures that can store DSTs
in a contiguous allocation. Some applications may have a performance
benefit for using a contiguous allocation as opposed to multiple pointers.

Currently, the only implementation is a `Vec` where the DST-tails are
slices of the same length.

## Usage

Add to dependencies:

```toml
[dependencies]
dst = "0.1.0"
```

Use in the code:

```rust
use dst::FixedVec;
fn main() {
    let mut vec = FixedVec::<Option<&str>, usize>::new(4);
    vec.push_default();
    let item = vec.get(0).unwrap();
    assert_eq!(item.value, None);
    assert_eq!(item.tail, [0, 0, 0, 0]);
    
    vec.push(Some("Name"), [1, 2, 3, 4].iter().copied());
    let item = vec.get(1).unwrap();
    assert_eq!(item.value, Some("Name"));
    assert_eq!(item.tail, [1, 2, 3, 4]);
}
```

# Flags

* `undefined_behavior` *(disabled by default)*

  This flag enables the project to build on stable Rust, but by utilizing
  code with *UB*. Specifically, invalid intermediate references are used to
  obtain a pointer to initialize data and to obtain layout information. See:
  
  * Raw-reference to value by syntax, 
    https://github.com/rust-lang/rust/issues/64490
  * Raw-reference to value by macro,
    https://github.com/rust-lang/rust/issues/73394
  * Size of value, https://github.com/rust-lang/rust/issues/69835
  * Layout of value, https://github.com/rust-lang/rust/issues/69835

* `unstable` *(enabled by default)*
  
  This flag enables use of unstable Rust APIs. Specifically it allows *UB* to
  be avoided, but also changes some of the crate's layout and size API to use
  `const` functions (presumably giving a runtime performance benefit). See
  https://github.com/rust-lang/rust/issues/46571
