/*! Slab allocator for fast iterating by optimizing the free space.

# Examples

```
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
*/

use std::{fmt::Debug, mem::replace};
/**
A fast HashMap-like collection that automatically determines the key.

# Performance

| method     | computational complexity |
| ---------- | ------------------------ |
| `insert`   | `O(1)`                   |
| `remove`   | `O(1)`                   |
| `get`      | `O(1)`                   |
| `get_mut`  | `O(1)`                   |
| `optimize` | `O(n)`, `n = self.len()` |
*/
#[derive(Clone)]
pub struct Slab<T> {
    entries: Vec<Entry<T>>,
    idx_next_vacant: usize,
    len: usize,
    non_optimized: usize,
}
const INVALID_INDEX: usize = usize::MAX;

#[derive(Clone)]
enum Entry<T> {
    Occupied(T),
    VacantHead { vacant_body_len: usize },
    VacantTail { idx_next_vacant: usize },
}

impl<T> Slab<T> {
    /// Constructs a new, empty Slab<T>.
    /// The slab will not allocate until elements are pushed onto it.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            idx_next_vacant: INVALID_INDEX,
            len: 0,
            non_optimized: 0,
        }
    }

    /// Returns the number of elements in the slab.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slab contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Entry::Occupied(value) = self.entries.get(index)? {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Entry::Occupied(value) = self.entries.get_mut(index)? {
            Some(value)
        } else {
            None
        }
    }

    /// Inserts a value into the slab.
    ///
    /// Returns the key associated with the value.
    pub fn insert(&mut self, value: T) -> usize {
        let idx;
        if self.idx_next_vacant < self.entries.len() {
            idx = self.idx_next_vacant;
            self.idx_next_vacant = match self.entries[idx] {
                Entry::VacantHead { vacant_body_len } => {
                    if vacant_body_len > 0 {
                        self.entries[idx + 1] = Entry::VacantHead {
                            vacant_body_len: vacant_body_len - 1,
                        };
                    }
                    idx + 1
                }
                Entry::VacantTail { idx_next_vacant } => idx_next_vacant,
                Entry::Occupied(_) => unreachable!(),
            };
            self.entries[idx] = Entry::Occupied(value);
            self.non_optimized = self.non_optimized.saturating_sub(1);
        } else {
            idx = self.entries.len();
            self.entries.push(Entry::Occupied(value));
        }
        self.len += 1;
        idx
    }

    /// Removes a key from the slab, returning the value at the key if the key was previously in the slab.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index + 1 < self.entries.len() {
            let e = replace(
                &mut self.entries[index],
                Entry::VacantTail {
                    idx_next_vacant: self.idx_next_vacant,
                },
            );
            if let Entry::Occupied(value) = e {
                self.len -= 1;
                self.idx_next_vacant = index;
                self.non_optimized += 1;
                return Some(value);
            }
            replace(&mut self.entries[index], e);
        } else if index + 1 == self.entries.len() {
            let e = self.entries.remove(index);
            if let Entry::Occupied(value) = e {
                self.len -= 1;
                if self.len == 0 {
                    self.clear();
                }
                return Some(value);
            }
            self.entries.push(e);
        }
        None
    }

    /// Clears the slab, removing all values and optimize free spaces.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.idx_next_vacant = INVALID_INDEX;
        self.non_optimized = 0;
    }

    /// Retains only the elements specified by the predicate and optimize free spaces.
    pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        let mut f = f;
        let mut idx = 0;
        let mut idx_vacant_start = 0;
        self.idx_next_vacant = INVALID_INDEX;
        while let Some(e) = self.entries.get(idx) {
            match e {
                Entry::VacantTail { .. } => {
                    idx += 1;
                }
                Entry::VacantHead { vacant_body_len } => {
                    idx += vacant_body_len + 2;
                }
                Entry::Occupied(value) => {
                    if f(value) {
                        self.merge_vacant(idx_vacant_start, idx);
                        idx += 1;
                        idx_vacant_start = idx;
                    } else {
                        drop(value);
                        self.entries[idx] = Entry::VacantTail {
                            idx_next_vacant: INVALID_INDEX,
                        };
                        idx += 1;
                    }
                }
            }
        }
        self.merge_vacant(idx_vacant_start, self.entries.len());
        self.non_optimized = 0;
    }

    /// Optimizing the free space for speeding up iterations.
    ///
    /// If the free space has already been optimized, this method does nothing and completes with O(1).
    pub fn optimize(&mut self) {
        if !self.is_optimized() {
            self.retain(|_| true);
        }
    }
    fn is_optimized(&self) -> bool {
        self.non_optimized == 0
    }
    fn merge_vacant(&mut self, start: usize, end: usize) {
        if start < end {
            if end == self.entries.len() {
                self.entries.truncate(start);
            } else {
                if start + 2 <= end {
                    self.entries[start] = Entry::VacantHead {
                        vacant_body_len: end - start - 2,
                    }
                }
                self.entries[end - 1] = Entry::VacantTail {
                    idx_next_vacant: self.idx_next_vacant,
                };
                self.idx_next_vacant = start;
            }
        }
    }

    /// Gets an iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of `remove` calls, `optimize` should be called before calling this function.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.entries.iter().enumerate(),
            len: self.len,
            used: 0,
        }
    }

    /// Gets a mutable iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of `remove` calls, `optimize` should be called before calling this function.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.entries.iter_mut().enumerate(),
            len: self.len,
            used: 0,
        }
    }

    /// Gets an iterator over the keys of the slab, in sorted order.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.iter().map(|x| x.0)
    }

    /// Gets an iterator over the values of the slab.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|x| x.1)
    }
}
impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Debug> Debug for Slab<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut is_first = true;
        for (key, value) in self {
            if is_first {
                write!(f, " ,")?;
                is_first = false;
            }
            write!(f, "{}: {:?}", key, value)?;
        }
        write!(f, "}}")
    }
}

impl<T> std::ops::Index<usize> for Slab<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("out of index.")
    }
}
impl<T> std::ops::IndexMut<usize> for Slab<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("out of index.")
    }
}

impl<'a, T> IntoIterator for &'a Slab<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Slab<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the entries of a Slab.
pub struct Iter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Entry<T>>>,
    len: usize,
    used: usize,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.iter.next() {
            match e {
                (key, Entry::Occupied(value)) => {
                    self.used += 1;
                    return Some((key, value));
                }
                (_, Entry::VacantHead { vacant_body_len }) => {
                    self.iter.nth(*vacant_body_len);
                }
                (_, Entry::VacantTail { .. }) => {}
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.used;
        (len, Some(len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len - self.used
    }
}

/// A mutable iterator over the entries of a Slab.
pub struct IterMut<'a, T> {
    iter: std::iter::Enumerate<std::slice::IterMut<'a, Entry<T>>>,
    len: usize,
    used: usize,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(e) = self.iter.next() {
            match e {
                (key, Entry::Occupied(value)) => {
                    self.used += 1;
                    return Some((key, value));
                }
                (_, Entry::VacantHead { vacant_body_len }) => {
                    self.iter.nth(*vacant_body_len);
                }
                (_, Entry::VacantTail { .. }) => {}
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.used;
        (len, Some(len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len - self.used
    }
}
