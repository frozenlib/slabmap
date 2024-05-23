use std::time::Instant;

use crate::SlabMap;

#[test]
fn test_new() {
    let s = SlabMap::<u32>::new();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_with_capacity() {
    for cap in 0..100 {
        let s = SlabMap::<u32>::with_capacity(cap);
        assert!(s.capacity() >= cap);
    }
}

#[test]
fn test_retain() {
    let mut s = SlabMap::new();
    s.insert(10);
    s.insert(15);
    s.insert(20);
    s.insert(25);

    s.retain(|_idx, x| *x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
    assert_eq!(s.len(), 2);
}

#[test]
fn test_len() {
    let mut s = SlabMap::new();
    assert_eq!(s.len(), 0);

    let key1 = s.insert(10);
    let key2 = s.insert(15);

    assert_eq!(s.len(), 2);

    s.remove(key1);
    assert_eq!(s.len(), 1);

    s.remove(key2);
    assert_eq!(s.len(), 0);
}

#[test]
fn test_is_empty() {
    let mut s = SlabMap::new();
    assert!(s.is_empty());

    let key = s.insert("a");
    assert!(!s.is_empty());

    s.remove(key);
    assert!(s.is_empty());
}

#[test]
fn test_get() {
    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert_eq!(s.get(key), Some(&100));
    assert_eq!(s.get(key + 1), None);
}

#[test]
fn test_contains_key() {
    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert!(s.contains_key(key));
    assert!(!s.contains_key(key + 1));
}

#[test]
fn test_insert() {
    let mut s = SlabMap::new();
    let key_abc = s.insert("abc");
    let key_xyz = s.insert("xyz");

    assert_eq!(s[key_abc], "abc");
    assert_eq!(s[key_xyz], "xyz");
}

#[test]
fn test_insert_with_key() {
    let mut s = SlabMap::new();
    let key = s.insert_with_key(|key| format!("my key is {}", key));

    assert_eq!(s[key], format!("my key is {}", key));
}

#[test]
fn test_remove() {
    let mut s = SlabMap::new();
    let key = s.insert("a");
    assert_eq!(s.remove(key), Some("a"));
    assert_eq!(s.remove(key), None);
}

#[test]
fn test_clear() {
    let mut s = SlabMap::new();
    s.insert(1);
    s.insert(2);

    s.clear();

    assert!(s.is_empty());
}

#[test]
fn test_drain() {
    let mut s = SlabMap::new();
    let k0 = s.insert(10);
    let k1 = s.insert(20);

    let d: Vec<_> = s.drain().collect();
    let mut e = vec![(k0, 10), (k1, 20)];
    e.sort();

    assert!(s.is_empty());
    assert_eq!(d, e);
}

#[test]
fn test_optimize() {
    let mut s = SlabMap::new();
    const COUNT: usize = 1000000;
    for i in 0..COUNT {
        s.insert(i);
    }
    let keys: Vec<_> = s.keys().take(COUNT - 1).collect();
    for key in keys {
        s.remove(key);
    }

    s.optimize(); // if comment out this line, `s.values().sum()` to be slow.

    let begin = Instant::now();
    let sum: usize = s.values().sum();
    println!("sum : {}", sum);
    println!("duration : {} ms", (Instant::now() - begin).as_millis());
}

#[test]
fn insert_remove_capacity() {
    let mut s = SlabMap::new();
    let mut keys = Vec::new();
    for _ in 0..10 {
        s.insert(11);
    }
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn insert_remove_capacity_all() {
    let mut s = SlabMap::new();
    let mut keys = Vec::new();
    for _ in 0..100 {
        keys.push(s.insert(10));
    }
    let capacity = s.capacity();
    for _ in 0..1000 {
        for key in keys.drain(..) {
            s.remove(key);
        }
        for _ in 0..100 {
            keys.push(s.insert(10));
        }
    }
    assert_eq!(capacity, s.capacity());
}

#[test]
fn into_iter() {
    let mut s = SlabMap::new();
    let k0 = s.insert(0);
    let k1 = s.insert(1);
    let k2 = s.insert(2);
    s.remove(k1);

    let a: Vec<_> = s.into_iter().collect();
    let mut e = vec![(k0, 0), (k2, 2)];
    e.sort();

    assert_eq!(a, e);
}

#[test]
fn clone_from() {
    let mut s0 = SlabMap::new();
    let mut s1 = SlabMap::new();
    for _ in 0..10 {
        s0.insert(0);
    }
    for _ in 0..1000 {
        s1.insert(0);
    }
    let cap_old = s1.capacity();
    s1.clone_from(&s0);
    let cap_new = s1.capacity();
    assert_eq!(cap_old, cap_new);
}

#[test]
fn from_iter() {
    let s: SlabMap<usize> = [(5, 1), (0, 3)].into_iter().collect();
    assert_eq!(s.len(), 2, "len");
    assert_eq!(s[5], 1);
    assert_eq!(s[0], 3);
}

#[test]
fn merge_vacant() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13)].into_iter().collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (3, 13)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_insert_2() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();
    let key = s.insert(99);
    let e = vec![(0, 10), (key, 99), (4, 14)];
    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2time() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14), (5, 15)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.optimize();
    s.remove(4);
    s.optimize();

    let e = vec![(0, 10), (3, 13), (5, 15)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_2part() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(4);
    s.optimize();
    let e = vec![(0, 10), (3, 13)];

    let a: Vec<_> = s.iter().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.iter_mut().map(|(k, v)| (k, *v)).collect();
    assert_eq!(a, e);

    let a: Vec<_> = s.into_iter().collect();
    assert_eq!(a, e);
}

#[test]
fn merge_vacant_drain() {
    let mut s: SlabMap<_> = [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)]
        .into_iter()
        .collect();
    s.remove(1);
    s.remove(2);
    s.remove(3);
    s.optimize();

    let e = vec![(0, 10), (4, 14)];
    let a: Vec<_> = s.drain().collect();
    assert_eq!(a, e);
}

#[test]
fn reserve() {
    let mut s: SlabMap<u32> = SlabMap::new();
    s.reserve(10);
    assert!(s.capacity() >= 10);
}

#[test]
fn reserve_exact() {
    let mut s: SlabMap<u32> = SlabMap::new();
    s.reserve_exact(10);
    assert!(s.capacity() == 10);
}
