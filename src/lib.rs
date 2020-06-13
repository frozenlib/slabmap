use std::{fmt::Debug, mem::replace};

#[derive(Clone)]
pub struct Slab<T> {
    entries: Vec<Entry<T>>,
    idx_next_vacant: usize,
    len: usize,
}
const INVALID_INDEX: usize = usize::MAX;

#[derive(Clone)]
enum Entry<T> {
    Occupied(T),
    VacantHead { vacant_body_len: usize },
    VacantTail { idx_next_vacant: usize },
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            idx_next_vacant: INVALID_INDEX,
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Entry::Occupied(value) = self.entries.get(index)? {
            Some(value)
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Entry::Occupied(value) = self.entries.get_mut(index)? {
            Some(value)
        } else {
            None
        }
    }

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
        } else {
            idx = self.entries.len();
            self.entries.push(Entry::Occupied(value));
        }
        self.len += 1;
        idx
    }
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
    pub fn clear(&mut self) {
        self.entries.clear();
        self.idx_next_vacant = INVALID_INDEX;
    }
    pub fn optimize(&mut self) {
        if !matches!(
            self.entries.get(self.idx_next_vacant),
            Some(Entry::VacantTail { .. })
        ) {
            return;
        }

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
                Entry::Occupied(_) => {
                    self.merge_vacant(idx_vacant_start, idx);
                    idx += 1;
                    idx_vacant_start = idx;
                }
            }
        }
    }
    fn merge_vacant(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
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

    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.entries.iter().enumerate(),
            len: self.len,
            used: 0,
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            iter: self.entries.iter_mut().enumerate(),
            len: self.len,
            used: 0,
        }
    }

    pub fn keys<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.iter().map(|x| x.0)
    }
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.iter().map(|x| x.1)
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
