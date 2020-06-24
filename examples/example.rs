fn main() {
    example_main();
    example_retain();
    example_len();
    example_insert();
    example_insert_with_key();
}

fn example_main() {
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
}

fn example_retain() {
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

fn example_len() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    assert_eq!(s.len(), 0);

    let key1 = s.insert(10);
    let key2 = s.insert(15);

    assert_eq!(s.len(), 2);

    s.remove(key1);
    assert_eq!(s.len(), 1);

    s.remove(key2);
    assert_eq!(s.len(), 2);
}

fn example_insert() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key_abc = s.insert("abc");
    let key_xyz = s.insert("xyz");

    assert_eq!(s[key_abc], "abc");
    assert_eq!(s[key_xyz], "xyz");
}

fn example_insert_with_key() {
    use slabmap::SlabMap;

    let mut s = SlabMap::new();
    let key = s.insert_with_key(|key| format!("my key is {}", key));

    assert_eq!(s[key], format!("my key is {}", key));
}
