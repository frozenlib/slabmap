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

use std::{
    collections::TryReserveError,
    fmt::Debug,
    iter::{Enumerate, FusedIterator},
    mem::replace,
};

use derive_ex::derive_ex;

#[cfg(test)]
mod tests;

/**
A fast HashMap-like collection that automatically determines the key.
*/

#[derive_ex(Clone(bound(T)))]
pub struct SlabMap<T> {
    entries: Vec<Entry<T>>,
    next_vacant_idx: usize,
    len: usize,
    non_optimized_count: usize,
}
const INVALID_INDEX: usize = usize::MAX;

#[derive(Clone, Debug)]
enum Entry<T> {
    Occupied(T),
    VacantHead { vacant_body_len: usize },
    VacantTail { next_vacant_idx: usize },
}

impl<T> SlabMap<T> {
    /// Constructs a new, empty `SlabMap<T>`.
    /// The SlabMap will not allocate until elements are pushed onto it.
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_vacant_idx: INVALID_INDEX,
            len: 0,
            non_optimized_count: 0,
        }
    }

    /// Constructs a new, empty `SlabMap<T>` with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next_vacant_idx: INVALID_INDEX,
            len: 0,
            non_optimized_count: 0,
        }
    }

    /// Constructs as new `SlabMap<T>` from keys and values with at least the specified capacity.
    pub fn from_iter_with_capacity(
        iter: impl IntoIterator<Item = (usize, T)>,
        capacity: usize,
    ) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for (key, value) in iter {
            if key >= entries.len() {
                entries.resize_with(key + 1, || Entry::VacantTail {
                    next_vacant_idx: INVALID_INDEX,
                });
            }
            entries[key] = Entry::Occupied(value);
        }
        let mut this = Self {
            entries,
            next_vacant_idx: INVALID_INDEX,
            len: usize::MAX,
            non_optimized_count: usize::MAX,
        };
        this.force_optimize();
        this
    }

    /// Returns the number of elements the SlabMap can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the given `SlabMap<T>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(self.entries_additional(additional));
    }

    /// Try to reserve capacity for at least additional more elements to be inserted in the given `SlabMap<T>`.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.entries
            .try_reserve(self.entries_additional(additional))
    }

    /// Reserves the minimum capacity for exactly additional more elements to be inserted in the given `SlabMap<T>`.
    ///
    /// # Panics
    /// Panics if the new capacity overflows usize.    
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.entries
            .reserve_exact(self.entries_additional(additional));
    }

    /// Try to reserve the minimum capacity for exactly additional more elements to be inserted in the given `SlabMap<T>`.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.entries
            .try_reserve_exact(self.entries_additional(additional))
    }

    #[inline]
    fn entries_additional(&self, additional: usize) -> usize {
        additional.saturating_sub(self.entries.len() - self.len)
    }

    /// Returns the number of elements in the SlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
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

    /// Returns true if the SlabMap contains no elements.
    ///    
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// assert_eq!(s.is_empty(), true);
    ///
    /// let key = s.insert("a");
    /// assert_eq!(s.is_empty(), false);
    ///
    /// s.remove(key);
    /// assert_eq!(s.is_empty(), true);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.get(key), Some(&100));
    /// assert_eq!(s.get(key + 1), None);
    /// ```
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

    /// Returns true if the SlabMap contains a value for the specified key.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert(100);
    ///
    /// assert_eq!(s.contains_key(key), true);
    /// assert_eq!(s.contains_key(key + 1), false);
    /// ```
    #[inline]
    pub fn contains_key(&self, key: usize) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a value into the SlabMap.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key_abc = s.insert("abc");
    /// let key_xyz = s.insert("xyz");
    ///
    /// assert_eq!(s[key_abc], "abc");
    /// assert_eq!(s[key_xyz], "xyz");
    /// ```
    pub fn insert(&mut self, value: T) -> usize {
        self.insert_with_key(|_| value)
    }

    /// Inserts a value given by `f` into the SlabMap. The key to be associated with the value is passed to `f`.
    ///
    /// Returns the key associated with the value.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert_with_key(|key| format!("my key is {}", key));
    ///
    /// assert_eq!(s[key], format!("my key is {}", key));
    /// ```
    #[inline]
    pub fn insert_with_key(&mut self, f: impl FnOnce(usize) -> T) -> usize {
        let idx;
        if self.next_vacant_idx < self.entries.len() {
            idx = self.next_vacant_idx;
            self.next_vacant_idx = match self.entries[idx] {
                Entry::VacantHead { vacant_body_len } => {
                    if vacant_body_len > 0 {
                        self.entries[idx + 1] = Entry::VacantHead {
                            vacant_body_len: vacant_body_len - 1,
                        };
                    }
                    idx + 1
                }
                Entry::VacantTail { next_vacant_idx } => next_vacant_idx,
                Entry::Occupied(_) => unreachable!(),
            };
            self.entries[idx] = Entry::Occupied(f(idx));
            self.non_optimized_count = self.non_optimized_count.saturating_sub(1);
        } else {
            idx = self.entries.len();
            self.entries.push(Entry::Occupied(f(idx)));
        }
        self.len += 1;
        idx
    }

    /// Removes a key from the SlabMap, returning the value at the key if the key was previously in the SlabMap.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let key = s.insert("a");
    /// assert_eq!(s.remove(key), Some("a"));
    /// assert_eq!(s.remove(key), None);
    /// ```
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
                    next_vacant_idx: self.next_vacant_idx,
                },
            );
            self.next_vacant_idx = key;
            self.non_optimized_count += 1;
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

    /// Clears the SlabMap, removing all values and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// s.insert(1);
    /// s.insert(2);
    ///
    /// s.clear();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.len = 0;
        self.next_vacant_idx = INVALID_INDEX;
        self.non_optimized_count = 0;
    }

    /// Clears the SlabMap, returning all values as an iterator and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
    /// let k0 = s.insert(10);
    /// let k1 = s.insert(20);
    ///
    /// let d: Vec<_> = s.drain().collect();
    /// let mut e = vec![(k0, 10), (k1, 20)];
    /// e.sort();
    ///
    /// assert_eq!(s.is_empty(), true);
    /// assert_eq!(d, e);
    /// ```
    pub fn drain(&mut self) -> Drain<T> {
        let len = self.len;
        self.len = 0;
        self.next_vacant_idx = INVALID_INDEX;
        self.non_optimized_count = 0;
        Drain {
            iter: self.entries.drain(..).enumerate(),
            len,
        }
    }

    /// Retains only the elements specified by the predicate and optimize free spaces.
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    ///
    /// let mut s = SlabMap::new();
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
        self.merge_vacants(f)
    }
    fn merge_vacants(&mut self, mut f: impl FnMut(usize, &mut T) -> bool) {
        let mut idx = 0;
        let mut vacant_head_idx = 0;
        let mut prev_vacant_tail_idx = None;
        let mut len = 0;
        self.next_vacant_idx = INVALID_INDEX;
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
                        self.set_vacants(vacant_head_idx, idx, &mut prev_vacant_tail_idx);
                        idx += 1;
                        len += 1;
                        vacant_head_idx = idx;
                    } else {
                        self.entries[idx] = Entry::VacantTail {
                            next_vacant_idx: INVALID_INDEX,
                        };
                        idx += 1;
                    }
                }
            }
        }
        self.entries.truncate(vacant_head_idx);
        self.non_optimized_count = 0;
        self.len = len;
    }
    fn set_vacants(
        &mut self,
        vacant_head_idx: usize,
        vacant_end_idx: usize,
        prev_vacant_tail_idx: &mut Option<usize>,
    ) {
        if vacant_head_idx >= vacant_end_idx {
            return;
        }
        if self.next_vacant_idx == INVALID_INDEX {
            self.next_vacant_idx = vacant_head_idx;
        }
        if vacant_head_idx + 2 <= vacant_end_idx {
            self.entries[vacant_head_idx] = Entry::VacantHead {
                vacant_body_len: vacant_end_idx - (vacant_head_idx + 2),
            };
        }
        self.entries[vacant_end_idx - 1] = Entry::VacantTail {
            next_vacant_idx: INVALID_INDEX,
        };
        if let Some(prev_vacant_tail_idx) = *prev_vacant_tail_idx {
            self.entries[prev_vacant_tail_idx] = Entry::VacantTail {
                next_vacant_idx: vacant_head_idx,
            };
        }
        *prev_vacant_tail_idx = Some(vacant_end_idx - 1);
    }

    /// Optimizing the free space for speeding up iterations.
    ///
    /// If the free space has already been optimized, this method does nothing and completes with O(1).
    ///
    /// # Examples
    /// ```
    /// use slabmap::SlabMap;
    /// use std::time::Instant;
    ///
    /// let mut s = SlabMap::new();
    /// const COUNT: usize = 1000000;
    /// for i in 0..COUNT {
    ///     s.insert(i);
    /// }
    /// let keys: Vec<_> = s.keys().take(COUNT - 1).collect();
    /// for key in keys {
    ///     s.remove(key);
    /// }
    ///
    /// s.optimize(); // if comment out this line, `s.values().sum()` to be slow.
    ///
    /// let begin = Instant::now();
    /// let sum: usize = s.values().sum();
    /// println!("sum : {}", sum);
    /// println!("duration : {} ms", (Instant::now() - begin).as_millis());
    /// ```
    pub fn optimize(&mut self) {
        if !self.is_optimized() {
            self.force_optimize();
        }
    }
    fn force_optimize(&mut self) {
        self.merge_vacants(|_, _| true);
    }

    #[inline]
    fn is_optimized(&self) -> bool {
        self.non_optimized_count == 0
    }

    /// Gets an iterator over the entries of the SlabMap, sorted by key.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.entries.iter().enumerate(),
            len: self.len,
        }
    }

    /// Gets a mutable iterator over the entries of the slab, sorted by key.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.entries.iter_mut().enumerate(),
            len: self.len,
        }
    }

    /// Gets an iterator over the keys of the SlabMap, in sorted order.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn keys(&self) -> Keys<T> {
        Keys(self.iter())
    }

    /// Gets an iterator over the values of the SlabMap.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values(&self) -> Values<T> {
        Values(self.iter())
    }

    /// Gets a mutable iterator over the values of the SlabMap.
    ///
    /// If you make a large number of [`remove`](SlabMap::remove) calls, [`optimize`](SlabMap::optimize) should be called before calling this function.
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<T> {
        ValuesMut(self.iter_mut())
    }
}
impl<T> Default for SlabMap<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Debug> Debug for SlabMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> std::ops::Index<usize> for SlabMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("out of index.")
    }
}
impl<T> std::ops::IndexMut<usize> for SlabMap<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("out of index.")
    }
}

impl<T> FromIterator<(usize, T)> for SlabMap<T> {
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iter: I) -> Self {
        Self::from_iter_with_capacity(iter, 0)
    }
}

impl<T> IntoIterator for SlabMap<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.entries.into_iter().enumerate(),
            len: self.len,
        }
    }
}

impl<'a, T> IntoIterator for &'a SlabMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut SlabMap<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An owning iterator over the values of a SlabMap.
///
/// This struct is created by the `into_iter` method on [`SlabMap`] (provided by the IntoIterator trait).
pub struct IntoIter<T> {
    iter: Enumerate<std::vec::IntoIter<Entry<T>>>,
    len: usize,
}
impl<T> Iterator for IntoIter<T> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e.1 {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((e.0, value));
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

/// A draining iterator for `SlabMap<T>`.
///
/// This struct is created by the [`drain`](SlabMap::drain) method on [`SlabMap`].
pub struct Drain<'a, T> {
    iter: Enumerate<std::vec::Drain<'a, Entry<T>>>,
    len: usize,
}
impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (usize, T);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut e_opt = self.iter.next();
        while let Some(e) = e_opt {
            e_opt = match e.1 {
                Entry::Occupied(value) => {
                    self.len -= 1;
                    return Some((e.0, value));
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

/// An iterator over the entries of a SlabMap.
///
/// This struct is created by the [`iter`](SlabMap::iter) method on [`SlabMap`].
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

/// A mutable iterator over the entries of a SlabMap.
///
/// This struct is created by the [`iter_mut`](SlabMap::iter_mut) method on [`SlabMap`].
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}
impl<'a, T> FusedIterator for IterMut<'a, T> {}
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

/// An iterator over the keys of a SlabMap.
///
/// This struct is created by the [`keys`](SlabMap::keys) method on [`SlabMap`].
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Keys<'a, T> {}
impl<'a, T> ExactSizeIterator for Keys<'a, T> {}

/// An iterator over the values of a SlabMap.
///
/// This struct is created by the [`values`](SlabMap::values) method on [`SlabMap`].
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for Values<'a, T> {}
impl<'a, T> ExactSizeIterator for Values<'a, T> {}

/// A mutable iterator over the values of a SlabMap.
///
/// This struct is created by the [`values_mut`](SlabMap::values_mut) method on [`SlabMap`].
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
    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
}
impl<'a, T> FusedIterator for ValuesMut<'a, T> {}
impl<'a, T> ExactSizeIterator for ValuesMut<'a, T> {}
