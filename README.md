# slab-iter

[![Crates.io](https://img.shields.io/crates/v/slab-iter.svg)](https://crates.io/crates/slab-iter)
[![Docs.rs](https://docs.rs/slab-iter/badge.svg)](https://docs.rs/crate/slab-iter)
[![Actions Status](https://github.com/frozenlib/slab-iter/workflows/build/badge.svg)](https://github.com/frozenlib/slab-iter/actions)

This crate provides the type `Slab`.
`Slab` is HashMap-like collection that automatically determines the key.

## The difference between `Slab` and `HashMap`

- `Slab` can only use usize as a key.
- The key of `Slab` is determined automatically.
- `Slab` runs faster than `HashMap`.

## Comparison with Similar Crates

- [`slab`](https://crates.io/crates/slab), Carl Lerche's slab crate provides a slab implementation with a similar API.

  For both `slab` and `slab-iter`, after adding many elements to the collection, removing many element will reduce iterate performance.

  However, unlike `slab`, `slab-iter` can improve iterate performance by calling `slab_iter::Slab::optimize`.

## Performance

The following chart shows the difference in performance between `slab_iter::Slab`, `slab::Slab`, `Vec`, `HashMap` and `BTreeMap`.

### Insert

![insert performance](https://raw.githubusercontent.com/frozenlib/slab-iter/images/bench/insert_large.svg?token=ACRPBIGGRPGVK7YZL7LZTVC67GB4K)

### Remove random elements

### Random access

### Sequential access

### Sequential access after remove many elements

- x-axis : number of remaining elements
- y-axis : duration (lower is better)

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
