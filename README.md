# slab-map

[![Crates.io](https://img.shields.io/crates/v/slab-map.svg)](https://crates.io/crates/slab-map)
[![Docs.rs](https://docs.rs/slab-map/badge.svg)](https://docs.rs/crate/slab-map)
[![Actions Status](https://github.com/frozenlib/slab-map/workflows/build/badge.svg)](https://github.com/frozenlib/slab-map/actions)

This crate provides the type `SlabMap`.
`SlabMap` is HashMap-like collection that automatically determines the key.

## Install

Add this to your Cargo.toml:

```toml
[dependencies]
slab-map = "0.1"
```

## Examples

```rust
use slab_map::SlabMap;

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

![insert performance](https://raw.githubusercontent.com/frozenlib/slab-map/images/bench/insert_large.svg?token=ACRPBIFOTM4JI3Z6U2PCUQS67LXU2)

### Remove random elements

![remove random elements performance](https://raw.githubusercontent.com/frozenlib/slab-map/images/bench/remove_random_large_fast_only.svg?token=ACRPBIHJK667KLBDLNIUODS67LXXC)

### Random access

![random access performance](https://raw.githubusercontent.com/frozenlib/slab-map/images/bench/get_random.svg?token=ACRPBIBA7RXH5ZX47PVYSC267LXZO)

### Sequential access

![sequential access performance](https://raw.githubusercontent.com/frozenlib/slab-map/images/bench/iter_key_values.svg?token=ACRPBIBN74IOFD54PBCSX3S67LYEG)

### Sequential access after remove many elements

- x-axis : number of remaining elements
- y-axis : duration (lower is better)

![Sequential access after remove many elements performance](https://raw.githubusercontent.com/frozenlib/slab-map/images/bench/iter_key_values_removed.svg?token=ACRPBIGZ6MZMLD5E22MKH5267LYFA)

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
