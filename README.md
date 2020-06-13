# slab-iter

[![Crates.io](https://img.shields.io/crates/v/slab-iter.svg)](https://crates.io/crates/slab-iter)
[![Docs.rs](https://docs.rs/slab-iter/badge.svg)](https://docs.rs/crate/slab-iter)
[![Actions Status](https://github.com/frozenlib/slab-iter/workflows/build/badge.svg)](https://github.com/frozenlib/slab-iter/actions)

This crate provides the type `Slab`.
`Slab` is HashMap-like collection that automatically determines the key.

Slab will iterate slower if you add a lot of values and then remove a lot of values.
But, calling [`Slab::optimize`] restores the speed.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
slab-iter = "0.1"
```

## Examples

```rust
use slab_iter::Slab;

let mut s = Slab::new();
let key_a = s.insert("aaa");
let key_b = s.insert("bbb");

assert_eq!(s[key_a], "aaa");
assert_eq!(s[key_b], "bbb");

s.optimize();
for (key, value) in &s {
    println!("{} -> {}", key, value);
}

let value = s.remove(key_a);
assert_eq!(value, Some("aaa"));
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
