/*! This crate provides the type [`SlabMap`].
[`SlabMap`] is HashMap-like collection that automatically determines the key.

# Examples

```
use slabmap::SlabMap;

let mut s = SlabMap::new();
let key_a = s.insert("aaa");
let key_b = s.insert("bbb");

assert_eq!(s[key_a], "aaa");
assert_eq!(s[key_b], "bbb");

for (key, value) in &s {
    println!("{} -> {}", key, value);
}

assert_eq!(s.remove(key_a), Some("aaa"));
assert_eq!(s.remove(key_a), None);
```
*/

pub mod slab_map;
pub mod small_slab_map;

#[doc(inline)]
pub use slab_map::SlabMap;

#[doc(inline)]
pub use small_slab_map::SmallSlabMap;
