fn main() {
    example_main();
    example_retain();
    example_len();
}

fn example_main() {
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
}

fn example_retain() {
    use slab_iter::Slab;

    let mut s = Slab::new();
    s.insert(10);
    s.insert(15);
    s.insert(20);
    s.insert(25);

    s.retain(|_idx, x| *x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
}

fn example_len() {
    use slab_iter::Slab;

    let mut s = Slab::new();
    assert_eq!(s.len(), 0);

    let key1 = s.insert(10);
    let key2 = s.insert(15);

    assert_eq!(s.len(), 2);

    s.remove(key1);
    assert_eq!(s.len(), 1);

    s.remove(key2);
    assert_eq!(s.len(), 2);
}
