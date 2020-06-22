/*! This crate provides the type [`Slab`].
[`Slab`] is HashMap-like collection that automatically determines the key.

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

use std::{fmt::Debug, iter::FusedIterator, mem::replace};
/**
A fast HashMap-like collection that automatically determines the key.
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
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            idx_next_vacant: INVALID_INDEX,
            len: 0,
            non_optimized: 0,
        }
    }

    /// Constructs a new, empty Slab<T> with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            idx_next_vacant: INVALID_INDEX,
            len: 0,
            non_optimized: 0,
        }
    }

    /// Returns the number of elements the slab can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the given Slab<T>.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    /// Reserves the minimum capacity for exactly additional more elements to be inserted in the given Slab<T>.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.entries.reserve_exact(additional);
    }

    /// Returns the number of elements in the slab.
    ///
    /// # Examples
    /// ```
    /// use slab_iter::Slab;
    ///
    /// let mut s = Slab::new();
    /// assert_eq!(s.len(), 0);
    ///
    /// let key1 = s.insert(10);
    /// let key2 = s.insert(15);
    ///
    /// assert_eq!(s.len(), 2);
    ///
    /// s.remove(key1);
    /// assert_eq!(s.len(), 1);
    ///
    /// s.remove(key2);
    /// assert_eq!(s.len(), 0);
    /// ```    
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slab contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        if let Entry::Occupied(value) = self.entries.get(key)? {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if let Entry::Occupied(value) = self.entries.get_mut(key)? {
            Some(value)
        } else {
            None
        }
    }

    /// Inserts a value into the slab.
    ///
    /// Returns the key associated with the value.
    pub fn insert(&mut self, key: T) -> usize {
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
            self.entries[idx] = Entry::Occupied(key);
            self.non_optimized = self.non_optimized.saturating_sub(1);
        } else {
            idx = self.entries.len();
            self.entries.push(Entry::Occupied(key));
        }
        self.len += 1;
        idx
    }

    /// Removes a key from the slab, returning the value at the key if the key was previously in the slab.
    pub fn remove(&mut self, key: usize) -> Option<T> {
        let is_last = key + 1 == self.entries.len();
        let e = self.entries.get_mut(key)?;
        if !matches!(e, Entry::Occupied(..)) {
            return None;
        }
        self.len -= 1;
        let e = if is_last {
            self.entries.pop().unwrap()
        } else {
            let e = replace(
                e,
                Entry::VacantTail {
                    idx_next_vacant: self.idx_next_vacant,
                },
            );
            self.idx_next_vacant = key;
            self.non_optimized += 1;
            e
        };
        if self.is_empty() {
            self.clear();
        }
        if let Entry::Occupied(value) = e {
            Some(value)
        } else {
            unreachable!()
        }
    }

    /// Clears the slab, removing all values and optimize free spaces.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
        self.idx_next_vacant = INVALID_INDEX;
        self.non_optimized = 0;
    }

    /// Clears the Slab, returning all values as an iterator and optimize free spaces.
    pub fn drain(&mut self) -> Drain<T> {
        let len = self.len;
        self.len = 0;
        self.idx_next_vacant = INVALID_INDEX;
        self.non_optimized = 0;
        Drain {
            iter: self.entries.drain(..),
            len,
        }
    }

    /// Retains only the elements specified by the predicate and optimize free spaces.
    ///
    /// ```
    /// use slab_iter::Slab;
    ///
    /// let mut s = Slab::new();
    /// s.insert(10);
    /// s.insert(15);
    /// s.insert(20);
    /// s.insert(25);
    ///
    /// s.retain(|_idx, value| *value % 2 == 0);
    ///
    /// let value: Vec<_> = s.values().cloned().collect();
    /// assert_eq!(value, vec![10, 20]);
    /// ```
    pub fn retain(&mut self, f: impl FnMut(usize, &mut T) -> bool) {
        let mut f = f;
        let mut idx = 0;
        let mut idx_vacant_start = 0;
        self.idx_next_vacant = INVALID_INDEX;
        while let Some(e) = self.entries.get_mut(idx) {
            match e {
                Entry::VacantTail { .. } => {
                    idx += 1;
                }
                Entry::VacantHead { vacant_body_len } => {
                    idx += *vacant_body_len + 2;
                }
                Entry::Occupied(value) => {
                    if f(idx, value) {
                        self.merge_vacant(idx_vacant_start, idx);
                        idx += 1;
                        idx_vacant_start = idx;
                    } else {
                        self.entries[idx] = Entry::VacantTail {
                            idx_next_vacant: INVALID_INDEX,
                        };
                        idx += 1;
                    }
                }
            }
        }
        self.entries.truncate(idx_vacant_start);
        self.non_optimized = 0;
    }

    /// Optimizing the free space for speeding up iterations.
    ///
    /// If the free space has already been optimized, this method does nothing and completes with O(1).
    pub fn optimize(&mut self) {
        if !self.is_optimized() {
            self.retain(|_, _| true);
        }
    }
    fn is_optimized(&self) -> bool {
        self.non_optimized == 0
    }
    fn merge_vacant(&mut self, start: usize, end: usize) {
        if start < end {
            if start < end - 1 {
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

    /// Gets an iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of [`remove`](Slab::remove) calls, [`optimize`](Slab::optimize) should be called before calling this function.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.entries.iter().enumerate(),
            len: self.len,
        }
    }

    /// Gets a mutable iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of [`remove`](Slab::remove) calls, [`optimize`](Slab::optimize) should be called before calling this function.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.entries.iter_mut().enumerate(),
            len: self.len,
        }
    }

    /// Gets an iterator over the keys of the slab, in sorted order.
    ///
    /// If you make a large number of [`remove`](Slab::remove) calls, [`optimize`](Slab::optimize) should be called before calling this function.
    #[inline]
    pub fn keys(&self) -> Keys<T> {
        Keys(self.iter())
    }

    /// Gets an iterator over the values of the slab.
    ///
    /// If you make a large number of [`remove`](Slab::remove) calls, [`optimize`](Slab::optimize) should be called before calling this function.
    #[inline]
    pub fn values(&self) -> Values<T> {
        Values(self.iter())
    }

    /// Gets a mutable iterator over the values of the slab.
    ///
    /// If you make a large number of [`remove`](Slab::remove) calls, [`optimize`](Slab::optimize) should be called before calling this function.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<T> {
        ValuesMut(self.iter_mut())
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

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("out of index.")
    }
}
impl<T> std::ops::IndexMut<usize> for Slab<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("out of index.")
    }
}

impl<T> IntoIterator for Slab<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.entries.into_iter(),
            len: self.len,
        }
    }
}

impl<'a, T> IntoIterator for &'a Slab<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Slab<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over the values of a Slab.
///
/// This struct is created by the `into_iter` method on [`Slab`] (provided by the IntoIterator trait).
pub struct IntoIter<T> {
    iter: std::vec::IntoIter<Entry<T>>,
    len: usize,
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some(value);
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(vacant_body_len + 1),
                Entry::VacantTail { .. } => self.iter.next(),
            }
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

/// A draining iterator for Slab<T>.
///
/// This struct is created by the [`drain`](Slab::drain) method on [`Slab`].
pub struct Drain<'a, T> {
    iter: std::vec::Drain<'a, Entry<T>>,
    len: usize,
}
impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some(value);
                }
                Entry::VacantHead { vacant_body_len } => self.iter.nth(vacant_body_len + 1),
                Entry::VacantTail { .. } => self.iter.next(),
            }
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

/// An iterator over the entries of a Slab.
///
/// This struct is created by the [`iter`](Slab::iter) method on [`Slab`].
pub struct Iter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e {
                (key, Entry::Occupied(value)) => {
                    self.len -= 1;
                    return Some((key, value));
                }
                (_, Entry::VacantHead { vacant_body_len }) => self.iter.nth(*vacant_body_len + 1),
                (_, Entry::VacantTail { .. }) => self.iter.next(),
            }
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

/// A mutable iterator over the entries of a Slab.
///
/// This struct is created by the [`iter_mut`](Slab::iter_mut) method on [`Slab`].
pub struct IterMut<'a, T> {
    iter: std::iter::Enumerate<std::slice::IterMut<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e {
                (key, Entry::Occupied(value)) => {
                    self.len -= 1;
                    return Some((key, value));
                }
                (_, Entry::VacantHead { vacant_body_len }) => self.iter.nth(*vacant_body_len + 1),
                (_, Entry::VacantTail { .. }) => self.iter.next(),
            }
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for IterMut<'a, T> {}
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

/// An iterator over the keys of a Slab.
///
/// This struct is created by the [`keys`](Slab::keys) method on [`Slab`].
pub struct Keys<'a, T>(Iter<'a, T>);
impl<'a, T> Iterator for Keys<'a, T> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Keys<'a, T> {}
impl<'a, T> ExactSizeIterator for Keys<'a, T> {}

/// An iterator over the values of a Slab.
///
/// This struct is created by the [`values`](Slab::values) method on [`Slab`].
pub struct Values<'a, T>(Iter<'a, T>);
impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Values<'a, T> {}
impl<'a, T> ExactSizeIterator for Values<'a, T> {}

/// A mutable iterator over the values of a Slab.
///
/// This struct is created by the [`values_mut`](Slab::values_mut) method on [`Slab`].
pub struct ValuesMut<'a, T>(IterMut<'a, T>);
impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for ValuesMut<'a, T> {}
impl<'a, T> ExactSizeIterator for ValuesMut<'a, T> {}
