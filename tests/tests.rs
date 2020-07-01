#[test]
fn test_new() {
    use slabmap::SlabMap;

    let s = SlabMap::<u32>::new();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_with_capacity() {
    use slabmap::SlabMap;

    for cap in 0..100 {
        let s = SlabMap::<u32>::with_capacity(cap);
        assert!(s.capacity() >= cap);
    }
}

#[test]
fn test_retain() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    s.insert(10);
    s.insert(15);
    s.insert(20);
    s.insert(25);

    s.retain(|_idx, x| *x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
}

#[test]
fn test_len() {
    use slabmap::SlabMap;

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
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    assert_eq!(s.is_empty(), true);

    let key = s.insert("a");
    assert_eq!(s.is_empty(), false);

    s.remove(key);
    assert_eq!(s.is_empty(), true);
}

#[test]
fn test_get() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert_eq!(s.get(key), Some(&100));
    assert_eq!(s.get(key + 1), None);
}

#[test]
fn test_contains_key() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key = s.insert(100);

    assert_eq!(s.contains_key(key), true);
    assert_eq!(s.contains_key(key + 1), false);
}

#[test]
fn test_insert() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key_abc = s.insert("abc");
    let key_xyz = s.insert("xyz");

    assert_eq!(s[key_abc], "abc");
    assert_eq!(s[key_xyz], "xyz");
}

#[test]
fn test_insert_with_key() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key = s.insert_with_key(|key| format!("my key is {}", key));

    assert_eq!(s[key], format!("my key is {}", key));
}

#[test]
fn test_remove() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key = s.insert("a");
    assert_eq!(s.remove(key), Some("a"));
    assert_eq!(s.remove(key), None);
}

#[test]
fn test_clear() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    s.insert(1);
    s.insert(2);

    s.clear();

    assert_eq!(s.is_empty(), true);
}

#[test]
fn test_drain() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    s.insert(10);
    s.insert(20);

    let d: Vec<_> = s.drain().collect();

    assert_eq!(s.is_empty(), true);
    assert_eq!(d, vec![10, 20]);
}

#[test]
fn test_optimize() {
    use slabmap::SlabMap;
    use std::time::Instant;

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
