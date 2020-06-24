# slabmap

[![Crates.io](https://img.shields.io/crates/v/slabmap.svg)](https://crates.io/crates/slabmap)
[![Docs.rs](https://docs.rs/slabmap/badge.svg)](https://docs.rs/crate/slabmap)
[![Actions Status](https://github.com/frozenlib/slabmap/workflows/build/badge.svg)](https://github.com/frozenlib/slabmap/actions)

This crate provides the type `SlabMap`.
`SlabMap` is HashMap-like collection that automatically determines the key.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
slabmap = "0.1"
```

## Examples

```rust
use slabmap::SlabMap;

let mut s = SlabMap::new();
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

## The difference between `SlabMap` and [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

- `SlabMap` can only use usize as a key.
- The key of `SlabMap` is determined automatically.
- `SlabMap` runs faster than `HashMap`.

## The difference between `SlabMap` and [`Slab`](https://docs.rs/slab/0.4.2/slab/struct.Slab.html)

Carl Lerche's [`slab`](https://crates.io/crates/slab) crate provides a slab implementation with a similar API.

For both `Slab` and `SlabMap`, after adding many elements to the collection, removing many element will reduce iterate performance.

However, unlike `Slab`, `SlabMap` can improve iterate performance by calling `optimize()`.

## Performance

The following chart shows the difference in performance between
[`BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html),
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html),
[`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html),
[`Slab`](https://docs.rs/slab/0.4.2/slab/struct.Slab.html) and
`SlabMap`.

### Insert

![insert performance](https://raw.githubusercontent.com/frozenlib/slabmap/images/bench/insert_large.svg?token=ACRPBICBTSIPIQNXU7QKMG267MEQK)

### Remove random elements

![remove random elements performance](https://raw.githubusercontent.com/frozenlib/slabmap/images/bench/remove_random_large_fast_only.svg?token=ACRPBIC62MWXTOVROVI4VVC67MERM)

### Random access

![random access performance](https://raw.githubusercontent.com/frozenlib/slabmap/images/bench/get_random.svg?token=ACRPBIBAHCKOCZ4VUWPMQ3S67MESK)

### Sequential access

![sequential access performance](https://raw.githubusercontent.com/frozenlib/slabmap/images/bench/iter_key_values.svg?token=ACRPBIGF2NHYAPVIAN35HKC67METS)

### Sequential access after removing elements from a 10,000-element collection

- x-axis : number of remaining elements
- y-axis : average time (lower is better)

![Sequential access after remove many elements performance](https://raw.githubusercontent.com/frozenlib/slabmap/images/bench/iter_key_values_removed.svg?token=ACRPBIAVZHDNT3CWUP6WAEC67MEUW)

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
