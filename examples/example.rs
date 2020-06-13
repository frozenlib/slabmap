fn main() {
    example_main();
    example_retain();
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

    s.retain(|x| x % 2 == 0);

    let value: Vec<_> = s.values().cloned().collect();
    assert_eq!(value, vec![10, 20]);
}
